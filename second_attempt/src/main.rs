#![allow(dead_code)]

mod defs;
mod bitboards;
mod hashkeys;
mod board;
mod data;

use bitboards::{print_bitboard, pop_bit, count_bits};

use board::{debug_board, parse_fen, print_board};
use defs::{clear_bit, set_bit, sq64, Board, BOARD_SQUARE_NUMBER, FILES_BOARD, RANKS_BOARD, SQ120_TO_SQ64, SQ64_TO_SQ120, START_FEN};
use crate::defs::Squares::*;

fn main() {
    let fen5 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    let my_board: &mut Board = &mut Board::default();
    // debug_board(my_board);
    let result = parse_fen(fen5, my_board);
    match result {
        Ok(_) => print!(""),
        Err(e) => println!("{}", e),
    }
    print_board(my_board);

    // println!("SQ120-SQ64");
    // for i in 0..BOARD_SQUARE_NUMBER {
    //     if i % 10 == 0 {
    //         println!();
    //     }
    //     print!("{:>4}", SQ120_TO_SQ64[i]);
    // }
    // println!();
    // println!("SQ64-SQ120");
    // for i in 0..64 {
    //     if i % 8 == 0 {
    //         println!();
    //     }
    //     print!("{:>4}", SQ64_TO_SQ120[i])
    // }

}
// -----------------------------
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

// -----------------------------
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

// -----------------------------
// Understand Rust nested arrays
// let test = [[1; 120]; 13];
// println!("test outer length: {}\ntest inner length: {}", test.len(), test[0].len());

// -----------------------------
// Check print_board
// let fen2 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
// let fen3 = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
// let fen4 = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
// let my_board: &mut Board = &mut Board::default();
// let result = parse_fen(START_FEN, my_board);
// match result {
//     Ok(_) => print!(""),
//     Err(e) => println!("{}", e),
// }
// print_board(my_board);
// parse_fen(&fen2, my_board);
// print_board(my_board);
// parse_fen(&fen3, my_board);
// print_board(my_board);
// parse_fen(&fen4, my_board);
// print_board(my_board);
