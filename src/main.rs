#![allow(dead_code)]

mod defs;
mod bitboards;
mod hashkeys;
mod board;
mod data;
mod attack;

use attack::{square_attacked, test_square_attacked};
use bitboards::{print_bitboard, pop_bit, count_bits};

use board::{check_board, debug_board, parse_fen, print_board};
use defs::{clear_bit, fr2sq, set_bit, sq64, Board, Files::*, Ranks::*, BLACK, BOARD_SQUARE_NUMBER, BOTH, FILES_BOARD, RANKS_BOARD, SIDE_KEY, SQ120_TO_SQ64, SQ64_TO_SQ120, START_FEN, WHITE};
use crate::defs::{Squares::*, Pieces::*};

fn main() {
    // let fen5 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let fen5 = "8/3q1p2/8/5P2/4Q3/8/8/8 w KQkq - 0 1";

    let my_board: &mut Board = &mut Board::default();
    // debug_board(my_board);
    let result = parse_fen(fen5, my_board);
    match result {
        Ok(_) => print!(""),
        Err(e) => println!("{}", e),
    }
    print_board(my_board);

    // let result2 = check_board(my_board);
    // match result2 {
    //     Ok(_) => print!(""),
    //     Err(e) => println!("{}", e),
    // }

    test_square_attacked(WHITE, my_board);
    println!();
    test_square_attacked(BLACK, my_board);

}
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
// print_board(my_board);x
// parse_fen(&fen4, my_board);
// print_board(my_board);

// -----------------------------
// Pawn bitboards
// let fen5 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

//     let my_board: &mut Board = &mut Board::default();
//     // debug_board(my_board);
//     let result = parse_fen(fen5, my_board);
//     match result {
//         Ok(_) => print!(""),
//         Err(e) => println!("{}", e),
//     }
//     print_board(my_board);

//     println!("WhitePawn");
//     print_bitboard(my_board.pawns[WHITE as usize]);
//     println!("BlackPawns");
//     print_bitboard(my_board.pawns[BLACK as usize]);
//     println!("BothPawns");
//     print_bitboard(my_board.pawns[BOTH as usize]);
// }
