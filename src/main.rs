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
        (self.suit() * 13 + self.rank()) as usize
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
}

impl Display for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
        }
        .on_white();

        write!(f, "{}{}", colored_card, " ".on_white())?;

        Ok(())
    }
}

// Number of working slots
const N: usize = 7;

#[derive(Debug, Clone)]
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
}

impl Display for SolitareState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..52 {
            write!(f, "{}", Card::from_index(i))?;
        }

        Ok(())
    }
}

fn main() {
    let state = SolitareState::new();

    println!("{state}");
}
