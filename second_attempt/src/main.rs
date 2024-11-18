mod defs;
use crate::defs::{BOARD_SQUARE_NUMBER, SQ120_TO_SQ64, SQ64_TO_SQ120};

fn main() {
    for i in 0..BOARD_SQUARE_NUMBER {
        if i % 10 == 0 {
            println!();
        }
        print!("{}   ", SQ120_TO_SQ64[i]);
    }

    println!();
    println!();

    for i in 0..64 {
        if i % 8 == 0 {
            println!();
        }
        print!("{}   ", SQ64_TO_SQ120[i])
    }
}
