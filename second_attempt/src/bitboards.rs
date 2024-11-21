use crate::{defs::{Ranks, Files}, FR2SQ, SQ64};

pub fn print_bitboard(bitboard: u64) {
    let mut shift: u64 = 1;

    let mut square: u8 = 0;
    let mut square64: u8 = 0;

    println!("\n");
    for rank in (Ranks::Rank1 as u8..=Ranks::Rank8 as u8).rev() {
        for file in Files::FileA as u8..=Files::FileH as u8 {
            square = FR2SQ!(file, rank);
            square64 = SQ64!(square);

            if ((shift << square64) & bitboard) != 0 {
                print!("X");
            } else {
                print!("-");
            }
        }
        println!();
    }
}
