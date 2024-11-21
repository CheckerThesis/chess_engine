mod defs;
mod bitboards;

use bitboards::{print_bitboard, pop_bit, count_bits};

use crate::defs::{Squares::*, SET_MASK};

fn main() {
    let mut board: u64 = 0;
    // decimal 0 in u64 is:     decimal 1 in u64 is:
    // 00000000                 00000000
    // 00000000                 00000000
    // 00000000                 00000000
    // 00000000                 00000000
    // 00000000                 00000000
    // 00000000                 00000000
    // 00000000                 00000000
    // 00000000                 00000001 = c
    // we use 1 << n, c moves n times left, then or's it with our original number
    board |= 1 << SQ64!(D2);
    board |= 1 << SQ64!(D3);
    board |= 1 << SQ64!(D4);
    println!("Popped bit: {}\nCount board: {}", pop_bit(&mut board), count_bits(board));

    print_bitboard(board);
}
// print board with borders
// for i in 0..BOARD_SQUARE_NUMBER {
//     if i % 10 == 0 {
//         println!();
//     }
//     print!("{}   ", SQ120_TO_SQ64[i]);
// }
// for i in 0..64 {
//     if i % 8 == 0 {
//         println!();
//     }
//     print!("{}   ", SQ64_TO_SQ120[i])
// }
