use crate::{defs::{Files, Ranks, SQ120_TO_SQ64, SQ64_TO_SQ120}, FR2SQ};

fn init_sq_120to64() {
    let mut square64: usize = 0;

    for rank in Ranks::Rank1 as u32..=Ranks::Rank8 as u32 {
        for file in Files::FileA as u32..=Files::FileH as u32 {
            let square = FR2SQ!(file, rank) as usize;
            unsafe {
                SQ64_TO_SQ120[square64] = square;
                SQ120_TO_SQ64[square] = square64;
            }
            square64 += 1;
        }
    }
}

pub fn all_init() {
    init_sq_120to64()
}