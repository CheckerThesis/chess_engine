#![allow(dead_code)]

mod defs;
mod bitboards;
mod hashkeys;
mod board;
mod data;
mod attack;
mod io;
mod movegen;
mod validate;

use std::sync::Mutex;

use attack::{square_attacked, test_square_attacked};
use bitboards::{print_bitboard, pop_bit, count_bits};

use board::{check_board, debug_board, parse_fen, print_board};
use defs::{captured, clear_bit, fr2sq, from_square, print_binary, promoted, set_bit, sq64, to_square, Board, Files::*, MoveList, Ranks::*, BLACK, BOARD_SQUARE_NUMBER, BOTH, FILES_BOARD, MOVE_FLAG_PAWN_START, RANKS_BOARD, SIDE_KEY, SQ120_TO_SQ64, SQ64_TO_SQ120, START_FEN, WHITE};
use io::{print_move, print_move_list, print_square};
use movegen::generate_all_moves;
use crate::defs::{Squares::*, Pieces::*};

fn main() {
    let fen_white_pawns = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";
    let fen_black_pawns = "rnbqkbnr/p1p1p3/3p3p/1p1p4/2P1Pp2/8/PP1P1PpP/RNBQKB1R b KQkq e3 0 1";
    let fen_knights_kings = "5k2/1n6/4n3/6N1/8/3N4/8/5K2 w - - 0 1";
    let fen_rooks = "6k1/8/5r2/8/1nR5/5N2/8/6K1 b - - 0 1";
    let fen_queens = "6k1/8/4nq2/8/1nQ5/5N2/1N6/6K1 b - - 0 1";
    let fen_bishops = "6k1/1b6/4n3/8/1n4B1/1B3N2/1N6/2b3K1 b - - 0 1";

    let my_board: &mut Board = &mut Board::default();
    match parse_fen(fen_bishops, my_board) {
        Ok(_) => print!(""),
        Err(e) => println!("{}", e),
    }
    print_board(my_board);
    println!();

    let move_list = &mut MoveList::default();
    generate_all_moves(my_board, move_list);
}
/*
-----------------------------
Test pop and set/clear masks:
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
board |= 1 << sq64(D2 as u8);
board |= 1 << sq64(D3 as u8);
board |= 1 << sq64(D4 as u8);
board |= 1 << sq64(H2 as u8);
println!("Popped bit: {}\nCount board: {}", pop_bit(&mut board), count_bits(board));

print_bitboard(board);
println!();
set_bit(&mut board, B7 as u8);
print_bitboard(board);
println!();
clear_bit(&mut board, D3 as u8);
print_bitboard(board);

-----------------------------
Understand Rust nested arrays:
let test = [[1; 120]; 13];
println!("test outer length: {}\ntest inner length: {}", test.len(), test[0].len());

-----------------------------
Check print_board
let fen2 = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
let fen3 = "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2";
let fen4 = "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2";
let my_board: &mut Board = &mut Board::default();
let result = parse_fen(START_FEN, my_board);
match result {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(my_board);
parse_fen(&fen2, my_board);
print_board(my_board);
parse_fen(&fen3, my_board);
print_board(my_board);
parse_fen(&fen4, my_board);
print_board(my_board);

-----------------------------
Pawn bitboards:
let fen5 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

let my_board: &mut Board = &mut Board::default();
// debug_board(my_board);
let result = parse_fen(fen5, my_board);
match result {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(my_board);

println!("WhitePawn");
print_bitboard(my_board.pawns[WHITE as usize]);
println!("BlackPawns");
print_bitboard(my_board.pawns[BLACK as usize]);
println!("BothPawns");
print_bitboard(my_board.pawns[BOTH as usize]);
}

-----------------------------
Attack squares:
let fen5 = "8/3q1p2/8/5P2/4Q3/8/8/8 w KQkq - 0 1";

let my_board: &mut Board = &mut Board::default();
let result = parse_fen(fen5, my_board);
match result {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(my_board);

test_square_attacked(WHITE, my_board);
println!();
test_square_attacked(BLACK, my_board);

-----------------------------
Move integer bits:
let fen5 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

let my_board: &mut Board = &mut Board::default();
// debug_board(my_board);
match parse_fen(fen5, my_board) {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(my_board);
match check_board(my_board) {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}

println!();
let mut el_move: u64 = 0;
let from: u64 = 6;
let to: u64 = 12;
let cap: u64 = WhiteRook as u64;
let pro: u64 = BlackRook as u64;
el_move = from | (to << 7) | (cap << 14) | (pro << 20);
println!("dec: {}   hex: {:x}", el_move, el_move);
print_binary(el_move);
println!("from: {}  to: {}  captured: {}    promoted: {}", from_square(el_move), to_square(el_move), captured(el_move), promoted(el_move));

/*
how the move flags work is by &ing everything out because it's all 0
"el_move & MOVE_FLAG_PAWN_START", then if the bit is set, & will keep the bit, else 0
*/
el_move |= MOVE_FLAG_PAWN_START; // comment this in and out
println!("is pawn start: {}", el_move & MOVE_FLAG_PAWN_START) != 0;

-----------------------------
Algebraic moves (send io into gui):
let fen5 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
let my_board: &mut Board = &mut Board::default();
// debug_board(my_board);
match parse_fen(fen5, my_board) {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(my_board);
match check_board(my_board) {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
println!();

let mut el_move: u64 = 0;
let from = A2 as u64;
let to = H7 as u64;

let capture = WhiteRook as u64;
let promote = BlackKing as u64;

el_move = from | (to << 7) | (capture << 14) | (promote << 20);

println!("from: {}  to: {}  capture: {}  promote: {}",
    from_square(el_move),
    to_square(el_move),
    captured(el_move),
    promoted(el_move)
);
println!("algebraic from: {}\nalgebraic to: {}\nalgebraic move: {}",
    print_square(from as u8),
    print_square(to as u8),
    print_move(el_move)
);
*/
