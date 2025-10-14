use std::{
    fmt::Display,
    io::{Stdout, stdout},
};

use crossterm::{
    cursor,
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode,
        KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    style::Stylize,
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};

const TWICE_WIDTH: bool = true;
const PRINT_PADDING: bool = true;

// Card in u8:
// suit rank
// 0000 0000
//    | Color (0 black, 1 red)
//
// Example, ♥ J:
// 0001 1011
struct Card(u8);

impl Card {
    fn from_index(i: usize) -> Self {
        let rank = (i % 13 + 1) as u8;
        let suit = (i / 13) as u8;

        Self::from_suit_rank(suit, rank)
    }

    fn from_suit_rank(suit: u8, rank: u8) -> Self {
        assert!(suit < 4 && rank <= 13);

        Self((suit << 4) | rank)
    }

    fn to_ind(&self) -> usize {
        (self.suit() * 13 + self.rank() - 1) as usize
    }

    fn rank(&self) -> u8 {
        self.0 & 0b0000_1111
    }

    fn suit(&self) -> u8 {
        self.0 >> 4
    }

    fn is_red(&self) -> bool {
        (self.0 >> 4) & 1 == 1
    }

    fn render(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        highlight: bool,
    ) -> std::fmt::Result {
        let rank = self.rank();
        let rank_offset = if let 1..=11 = rank { rank } else { rank + 1 };

        let suit = self.suit();
        let suit_offset = [0, 1, 3, 2][suit as usize] << 4;

        let card_char =
            char::from_u32('🂠' as u32 + suit_offset + rank_offset as u32)
                .unwrap();

        let colored_card = if self.is_red() {
            card_char.red()
        } else {
            card_char.black()
        };

        let (highlighted_card, pad) = if highlight {
            (colored_card.on_dark_green(), " ".on_dark_green())
        } else {
            (colored_card.on_white(), " ".on_white())
        };

        if TWICE_WIDTH {
            if PRINT_PADDING {
                write!(f, "{}{}", highlighted_card, pad)?;
            } else {
                write!(f, "{} ", highlighted_card)?;
            }
        } else {
            write!(f, "{}", highlighted_card)?;
        }

        Ok(())
    }

    fn highlight(self, highlight: bool) -> HighlightedCard {
        HighlightedCard(self, highlight)
    }
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.render(f, false)
    }
}

struct HighlightedCard(Card, bool);

impl Display for HighlightedCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.render(f, self.1)
    }
}

// Number of working slots
const N: usize = 7;
const MAX_HEIGHT: usize = N - 1 + 13;

#[derive(Debug, Clone, Copy)]
struct SolitareState {
    deck: u64,        // 1 bit per card, suits ordered: ♠, ♥, ♣, ♦
    targets: [u8; 4], // Number of "solved" cards for each suit
    slots: [[u8; MAX_HEIGHT]; N], // Working slots
    slots_lens: [u8; N], // Combo: 4 low bits: len, 4 high bits: n hidden
}

fn shuffle(data: &mut [u8]) {
    for i in 0..data.len() {
        let j = rand::random_range(i..data.len());

        data.swap(i, j);
    }
}

fn shuffled_deck() -> [u8; 52] {
    let mut deck = [0; 52];

    for (i, x) in deck.iter_mut().enumerate() {
        *x = Card::from_index(i).0;
    }

    shuffle(&mut deck);

    deck
}

#[derive(Debug, Clone, Copy)]
enum Highlight {
    None,
    Target(u8),
    Deck(u8),
    Slot(u8, u8),
}

impl SolitareState {
    fn new() -> Self {
        let mut state = Self {
            deck: 0,
            targets: [0; 4],
            slots: [[0; MAX_HEIGHT]; N],
            slots_lens: [0; N],
        };

        let deck = shuffled_deck();
        let mut cur_card = 0;

        // Dealing to slots:
        for i in 0..N {
            for j in i..N {
                state.slots[j][i] = deck[cur_card];
                cur_card += 1;
            }

            state.slots_lens[i] = ((i << 4) as u8) | ((i + 1) as u8);
        }

        // Counting which are left for remaining deck
        for &card in deck.iter().skip(cur_card) {
            state.deck |= 1 << Card(card).to_ind();
        }

        state
    }

    fn render(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        highlight: Highlight,
    ) -> std::fmt::Result {
        let hl_ind = if let Highlight::Target(i) = highlight {
            i as usize
        } else {
            4 // Out of bounds, will never hit
        };

        for suit in 0..4 {
            if self.targets[suit] == 0 {
                write!(f, "{}", "🂠".dark_grey())?;
                if TWICE_WIDTH {
                    write!(f, " ")?;
                }
            } else {
                write!(
                    f,
                    "{}",
                    Card::from_suit_rank(suit as u8, self.targets[suit])
                        .highlight(suit == hl_ind),
                )?;
            }
        }

        write!(f, " ┃ ")?;

        let mut remaining_deck = self.deck;
        let mut i: usize = 0;

        let hl_ind = if let Highlight::Deck(i) = highlight {
            i as u32
        } else {
            52 // Will never hit
        };

        for j in 0..self.deck.count_ones() {
            let skip = remaining_deck.trailing_zeros() + 1;

            i += skip as usize;
            remaining_deck >>= skip;

            write!(f, "{}", Card::from_index(i - 1).highlight(j == hl_ind))?;
        }

        writeln!(f, "\n\r")?;

        let max_height =
            self.slots_lens.iter().map(|l| l & 0x0f).max().unwrap();

        let (hl_col, hl_row) = if let Highlight::Slot(i, j) = highlight {
            (i as usize, j)
        } else {
            (N + 1, max_height + 1) // Too high, will never hit
        };

        for row_ind in 0..max_height {
            for col_ind in 0..N {
                let col_len = self.slots_lens[col_ind] & 0x0f;
                let n_hidden = self.slots_lens[col_ind] >> 4;
                if row_ind >= col_len {
                    write!(f, " ")?;
                    if TWICE_WIDTH {
                        write!(f, " ")?;
                    }
                } else if row_ind < n_hidden {
                    write!(f, "{}", "🂠".blue())?;
                    if TWICE_WIDTH {
                        write!(f, " ")?;
                    }
                } else {
                    write!(
                        f,
                        "{}",
                        Card(self.slots[col_ind][row_ind as usize])
                            .highlight(col_ind == hl_col && row_ind >= hl_row)
                    )?;
                }
            }
            writeln!(f, "\r")?;
        }

        Ok(())
    }

    fn highlight(self, highlight: Highlight) -> HighlightedSolitareState {
        HighlightedSolitareState(self, highlight)
    }
}

impl Display for SolitareState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.render(f, Highlight::None)
    }
}

struct HighlightedSolitareState(SolitareState, Highlight);

impl Display for HighlightedSolitareState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.render(f, self.1)
    }
}

struct GameState {
    out: Stdout,
    state: SolitareState,
    selected: Highlight,
}

impl GameState {
    fn new() -> Self {
        Self {
            out: stdout(),
            state: SolitareState::new(),
            selected: Highlight::None,
        }
    }

    fn coord_to_selection(col: u16, row: u16) -> Highlight {
        match (col, row, TWICE_WIDTH) {
            (_, 2.., _) => {
                let slot = if TWICE_WIDTH { col / 2 } else { col };
                let row = row - 2;

                Highlight::Slot(slot as u8, row as u8)
            }
            (..8, 0, true) => Highlight::Target((col / 2) as u8),
            (..4, 0, false) => Highlight::Target(col as u8),
            (11.., 0, true) => Highlight::Deck(((col - 11) / 2) as u8),
            (7.., 0, false) => Highlight::Deck((col - 7) as u8),
            _ => Highlight::None,
        }
    }

    // [src, dst]
    fn is_selection_valid(&mut self, selection: Highlight) -> [bool; 2] {
        match selection {
            Highlight::None => [false; 2],
            Highlight::Target(i) => {
                if i < 4 {
                    [self.state.targets[i as usize] > 0, true]
                } else {
                    [false; 2]
                }
            }
            Highlight::Deck(i) => {
                [(i as u32) < self.state.deck.count_ones(), false]
            }
            Highlight::Slot(col, row) => {
                if (col as usize) < N {
                    let slot = self.state.slots_lens[col as usize];
                    let n_cards = slot & 0x0f;
                    let n_hidden = slot >> 4;

                    [(n_hidden..n_cards).contains(&row), true]
                } else {
                    [false; 2]
                }
            }
        }
    }

    fn try_move(&mut self, selection: Highlight) {
        let mut multiple = false;

        let card = match self.selected {
            Highlight::None => {
                self.exit_game_mode();
                panic!("Trying to move without selected card!")
            }
            Highlight::Target(suit) => {
                let rank = self.state.targets[suit as usize];

                Card::from_suit_rank(suit, rank)
            }
            Highlight::Deck(i) => {
                let mut deck = self.state.deck;
                let mut card_ind = 0;

                for _ in 0..=i {
                    let skip = deck.trailing_zeros() + 1;
                    deck >>= skip;
                    card_ind += skip;
                }

                card_ind -= 1;

                Card::from_index(card_ind as usize)
            }
            Highlight::Slot(col, row) => {
                let slot_height = self.state.slots_lens[col as usize] & 0x0f;

                if row + 1 < slot_height {
                    multiple = true;
                }

                Card(self.state.slots[col as usize][row as usize])
            }
        };

        println!("\n\nTrying to move: {}", card);

        match selection {
            Highlight::None => {
                self.exit_game_mode();
                panic!("Trying to move without selected destination!")
            }
            Highlight::Target(_) => {
                let suit = card.suit();
                if card.rank() != self.state.targets[suit as usize] + 1
                    || multiple
                {
                    self.selected = selection;
                } else {
                    self.state.targets[suit as usize] += 1;

                    match self.selected {
                        Highlight::None => unreachable!(),
                        Highlight::Target(_) => unreachable!(),
                        Highlight::Deck(_) => {
                            self.state.deck &= !(1 << card.to_ind())
                        }
                        Highlight::Slot(col, _) => {
                            let slot = &mut self.state.slots_lens[col as usize];
                            let n_cards = (*slot & 0x0f) - 1;
                            let mut n_hidden = *slot >> 4;

                            if n_hidden > 0 && n_hidden == n_cards {
                                n_hidden -= 1;
                            }

                            *slot = (n_hidden << 4) | n_cards;
                        }
                    }

                    self.selected = Highlight::None;
                }
            }
            Highlight::Deck(_) => self.selected = selection,
            Highlight::Slot(col, _) => {
                let slot = self.state.slots_lens[col as usize];
                let slot_len = slot & 0x0f;
                let slot_hidden = slot >> 4;

                // First check for legality of move:
                let legal = if slot_len == 0 {
                    card.rank() == 13
                } else {
                    let target_card = Card(
                        self.state.slots[col as usize][slot_len as usize - 1],
                    );

                    (card.rank() + 1 == target_card.rank())
                        && (card.is_red() ^ target_card.is_red())
                };

                if legal {
                    // Then performing the move

                    if !multiple {
                        self.state.slots[col as usize][slot_len as usize] =
                            card.0;
                        self.state.slots_lens[col as usize] =
                            (slot_hidden << 4) | (slot_len + 1);
                    }

                    match self.selected {
                        Highlight::None => unreachable!(),
                        Highlight::Target(suit) => {
                            self.state.targets[suit as usize] -= 1
                        }
                        Highlight::Deck(_) => {
                            self.state.deck &= !(1 << card.to_ind())
                        }
                        Highlight::Slot(from_col, row) => {
                            let slot =
                                &mut self.state.slots_lens[from_col as usize];
                            if !multiple {
                                let n_cards = (*slot & 0x0f) - 1;
                                let mut n_hidden = *slot >> 4;

                                if n_hidden > 0 && n_hidden == n_cards {
                                    n_hidden -= 1;
                                }

                                *slot = (n_hidden << 4) | n_cards;
                            } else {
                                let n_cards = *slot & 0x0f;
                                let n_moved = n_cards - row;
                                let new_n_cards = n_cards - n_moved;

                                let mut n_hidden = *slot >> 4;

                                if n_hidden > 0 && n_hidden == new_n_cards {
                                    n_hidden -= 1;
                                }

                                *slot = (n_hidden << 4) | new_n_cards;

                                for i in 0..n_cards {
                                    self.state.slots[col as usize]
                                        [(slot_len + i) as usize] =
                                        self.state.slots[from_col as usize]
                                            [(row + i) as usize]
                                }

                                let new_to_slot_len = slot_len + n_moved;

                                self.state.slots_lens[col as usize] =
                                    (slot_hidden << 4) | new_to_slot_len;
                            }
                        }
                    }

                    self.selected = Highlight::None;
                } else {
                    self.selected = selection;
                }
            }
        }
    }

    fn enter_game_mode(&mut self) {
        enable_raw_mode().unwrap();

        execute!(
            self.out,
            EnableMouseCapture,
            EnterAlternateScreen,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .unwrap();
    }

    fn exit_game_mode(&mut self) {
        execute!(
            self.out,
            DisableMouseCapture,
            cursor::Show,
            LeaveAlternateScreen
        )
        .unwrap();

        disable_raw_mode().unwrap()
    }

    fn run(&mut self) {
        self.enter_game_mode();

        println!("{}", self.state);

        while let Ok(x) = event::read() {
            match x {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    kind: _,
                    state: _,
                }) => break,

                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    kind: _,
                    state: _,
                }) => {
                    self.selected = Highlight::None;
                    execute!(self.out, cursor::MoveTo(0, 0)).unwrap();
                    println!("{}", self.state);
                }

                Event::Mouse(MouseEvent {
                    kind: MouseEventKind::Down(MouseButton::Left),
                    column,
                    row,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    let new_selection = Self::coord_to_selection(column, row);

                    let [valid_src, valid_dst] =
                        self.is_selection_valid(new_selection);

                    match (valid_src, valid_dst, self.selected) {
                        (false, _, Highlight::None) => {}
                        (true, _, Highlight::None) => {
                            self.selected = new_selection
                        }
                        (
                            _,
                            true,
                            Highlight::Target(_)
                            | Highlight::Deck(_)
                            | Highlight::Slot(_, _),
                        ) => self.try_move(new_selection),
                        (false, _, _) => self.selected = Highlight::None,
                        (true, _, _) => self.selected = new_selection,
                    }

                    execute!(
                        self.out,
                        cursor::MoveTo(0, 0),
                        terminal::Clear(terminal::ClearType::All)
                    )
                    .unwrap();
                    print!("{}", self.state.highlight(self.selected));

                    // println!("Row: {row:3}\n\rCol: {column:3}\r");
                    // execute!(self.out, cursor::MoveUp(2)).unwrap();
                }

                _ => {}
            }
        }

        self.exit_game_mode();
    }
}

fn main() {
    let mut game = GameState::new();

    // game.state.targets[2] = 6;
    // game.state.slots_lens[3] &= 0x0f;

    game.run();
}
