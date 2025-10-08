// Number of working slots
const N: usize = 7;

// Card in u8:
// suit  num
// 0000 0000
//    | Color (0 black, 1 red)
//
// Example, ♥ J:
// 0001 1011
const CARDS: [u8; 52] = [
    0b0000_0001,
    0b0000_0010,
    0b0000_0011,
    0b0000_0100,
    0b0000_0101,
    0b0000_0110,
    0b0000_0111,
    0b0000_1000,
    0b0000_1001,
    0b0000_1010,
    0b0000_1011,
    0b0000_1100,
    0b0000_1101,
    0b0001_0001,
    0b0001_0010,
    0b0001_0011,
    0b0001_0100,
    0b0001_0101,
    0b0001_0110,
    0b0001_0111,
    0b0001_1000,
    0b0001_1001,
    0b0001_1010,
    0b0001_1011,
    0b0001_1100,
    0b0001_1101,
    0b0010_0001,
    0b0010_0010,
    0b0010_0011,
    0b0010_0100,
    0b0010_0101,
    0b0010_0110,
    0b0010_0111,
    0b0010_1000,
    0b0010_1001,
    0b0010_1010,
    0b0010_1011,
    0b0010_1100,
    0b0010_1101,
    0b0011_0001,
    0b0011_0010,
    0b0011_0011,
    0b0011_0100,
    0b0011_0101,
    0b0011_0110,
    0b0011_0111,
    0b0011_1000,
    0b0011_1001,
    0b0011_1010,
    0b0011_1011,
    0b0011_1100,
    0b0011_1101,
];

fn card_to_ind(card: u8) -> usize {
    let num = (card & 0b0000_1111) as usize;
    let suit = ((card & 0b1111_0000) >> 4) as usize;

    suit * 13 + num
}

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
    let mut deck = CARDS;

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
            state.deck |= 1 << card_to_ind(card);
        }

        state
    }
}

fn main() {
    let state = SolitareState::new();

    println!("{:02x?}", state);
}
