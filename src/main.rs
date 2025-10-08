use std::fmt::Display;

use crossterm::style::Stylize;

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

        write!(f, "{}{}", highlighted_card, pad)?;

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

#[derive(Debug, Clone, Copy)]
struct SolitareState {
    deck: u64,            // 1 bit per card, suits ordered: ♠, ♥, ♣, ♦
    targets: [u8; 4],     // Number of "solved" cards for each suit
    slots: [[u8; 14]; N], // Working slots
    slots_lens: [u8; N],  // Combo: 4 low bits: len, 4 high bits: n hidden
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
    NoHighlight,
    Target(u8),
    Deck(u8),
    Slot(u8, u8),
}

impl SolitareState {
    fn new() -> Self {
        let mut state = Self {
            deck: 0,
            targets: [0; 4],
            slots: [[0; 14]; N],
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
                write!(f, "{}", "🂠 ".dark_grey())?;
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

        writeln!(f, "\n")?;

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
                    write!(f, "  ")?;
                } else if row_ind < n_hidden {
                    write!(f, "{}", "🂠 ".blue())?;
                } else {
                    write!(
                        f,
                        "{}",
                        Card(self.slots[col_ind][row_ind as usize])
                            .highlight(col_ind == hl_col && row_ind >= hl_row)
                    )?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Display for SolitareState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.render(f, Highlight::NoHighlight)
    }
}

struct HighlightedSolitareState(SolitareState, Highlight);

impl Display for HighlightedSolitareState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.render(f, self.1)
    }
}

fn main() {
    let mut state = SolitareState::new();

    state.slots_lens[5] &= 0x0f;
    state.slots_lens[5] |= 2 << 4;

    println!("{}", HighlightedSolitareState(state, Highlight::Slot(5, 3)));
}
