mod defs;
mod bitboards;
mod hashkeys;
mod board;

use bitboards::{print_bitboard, pop_bit, count_bits};

use defs::{clear_bit, set_bit, sq64};
use crate::defs::{Squares::*};

fn main() {
}
// Print board with borders:
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

// Test pop and set/clear masks:
// let mut board: u64 = 0;
//     // decimal 0 in u64 is:     decimal 1 in u64 is:
//     // 00000000                 00000000
//     // 00000000                 00000000
//     // 00000000                 00000000
//     // 00000000                 00000000
//     // 00000000                 00000000
//     // 00000000                 00000000
//     // 00000000                 00000000
//     // 00000000                 00000001 = c
//     // we use 1 << n, c moves n times left, then or's it with our original number
//     board |= 1 << sq64(D2 as u8);
//     board |= 1 << sq64(D3 as u8);
//     board |= 1 << sq64(D4 as u8);
//     board |= 1 << sq64(H2 as u8);
//     println!("Popped bit: {}\nCount board: {}", pop_bit(&mut board), count_bits(board));

//     print_bitboard(board);
//     println!();
//     set_bit(&mut board, B7 as u8);
//     print_bitboard(board);
//     println!();
//     clear_bit(&mut board, D3 as u8);
//     print_bitboard(board);
