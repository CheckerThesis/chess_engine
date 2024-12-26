use crate::defs::{fr2sq, sq64, Files, Ranks};

static BIT_TABLE: [u64; 64] = [
    63, 30, 3, 32, 25, 41, 22, 33, 15, 50, 42, 13, 11, 53, 19, 34, 61, 29, 2,
    51, 21, 43, 45, 10, 18, 47, 1, 54, 9, 57, 0, 35, 62, 31, 40, 4, 49, 5, 52,
    26, 60, 6, 23, 44, 46, 27, 56, 16, 7, 39, 48, 24, 59, 14, 12, 55, 38, 28,
    58, 20, 37, 17, 36, 8
];

pub fn pop_bit(bitboard: &mut u64) -> u64 {
    // Isolates the least significant bit (LSB) that is set to 1
    let least_significant_bit = *bitboard & (*bitboard as i64).wrapping_neg() as u64;

    // Performs a bit-scan forward operation to get the index of the LSB
    let lsb_index: u64 = least_significant_bit.trailing_zeros() as u64;

    // Clear the LSB from the bitboard
    *bitboard &= *bitboard - 1;

    // Since the trailing_zeros function returns the index from the least significant bit position (0),
    // we need to convert it to a 64-based square index suitable for a chessboard
    lsb_index
}

pub fn count_bits(mut bit: u64) -> u64 {
    let mut r = 0;

    while bit != 0 {
        bit &= bit - 1;
        r += 1;
    }

    r
}

pub fn print_bitboard(bitboard: u64) {
    for rank in (Ranks::Rank1 as u8..=Ranks::Rank8 as u8).rev() {
        for file in Files::FileA as u8..=Files::FileH as u8 {
            let square: u8 = fr2sq(file, rank);
            let square64: u8 = sq64(square);

            if ((1 << square64) & bitboard) != 0 {
                print!("X");
            } else {
                print!("-");
            }
        }
        println!();
    }
}
