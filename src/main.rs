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
mod makemove;
mod perft;
mod search;
mod pvtable;
mod evaluate;
mod uci;

use std::{sync::Mutex, time::Instant, vec};

use attack::{square_attacked, test_square_attacked};
use bitboards::{print_bitboard, pop_bit, count_bits};
use board::{check_board, debug_board, parse_fen, print_board};
use colored::Colorize;
use data::PIECE_CHAR;
use defs::{captured, clear_bit, fr2sq, from_square, print_binary, promoted, set_bit, sq64, to_square, Board, Files::*, MoveList, Ranks::*, SearchInfo, BLACK, BLACK_PASSED_MASK, BOARD_SQUARE_NUMBER, BOTH, FILES_BOARD, FILE_BB_MASK, ISOLATED_MASK, MOVE_FLAG_PAWN_START, MVV_LVA_SCORES, NO_MOVE, RANKS_BOARD, RANK_BB_MASK, SIDE_KEY, SQ120_TO_SQ64, SQ64_TO_SQ120, WHITE, WHITE_PASSED_MASK};
use evaluate::evaluate_position;
use io::{parse_move, print_move, print_move_list, print_square};
use makemove::{make_move, take_move};
use movegen::generate_all_moves;
use perft::perft_test;
use pvtable::get_pv_line;
use search::{is_repetition, search_position};
use uci::uci_loop;
use crate::defs::{Squares::*, Pieces::*};

use std::io as std_io; // Import std::io as std_io

const FEN_START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
const FEN_WHITE_PAWNS: &str = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";
const FEN_BLACK_PAWNS: &str = "rnbqkbnr/p1p1p3/3p3p/1p1p4/2P1Pp2/8/PP1P1PpP/RNBQKB1R b KQkq e3 0 1";
const FEN_KNIGHTS_KINGS: &str = "5k2/1n6/4n3/6N1/8/3N4/8/5K2 w - - 0 1";
const FEN_ROOKS: &str = "6k1/8/5r2/8/1nR5/5N2/8/6K1 b - - 0 1";
const FEN_QUEENS: &str = "6k1/8/4nq2/8/1nQ5/5N2/1N6/6K1 b - - 0 1";
const FEN_BISHOPS: &str = "6k1/1b6/4n3/8/1n4B1/1B3N2/1N6/2b3K1 b - - 0 1";
const FEN_CASTLE1: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
const FEN_CASTLE2: &str = "3rk2r/8/8/8/8/8/6p1/R3K2R b KQk - 0 1";
const FEN_TRICKY: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
const FEN_48: &str = "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N w - - 0 1";
const FEN_60: &str = "2rr3k/pp3pp1/1nnqbN1p/3pN3/2pP4/2P3Q1/PPB4P/R4RK1 w - - 0 1";
const FEN_61: &str = "r1b1k2r/ppppnppp/2n2q2/2b5/3NP3/2P1B3/PP3PPP/RN1QKB1R w KQkq - 0 1";

const FEN_WIKI3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
const FEN_WIKI4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
const FEN_WIKI4R: &str = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";
const FEN_WIKI5: &str = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
const FEN_WIKI6: &str = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";

fn main() {
    uci_loop();
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
let position: &mut Board = &mut Board::default();
let result = parse_fen(START_FEN, position);
match result {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(position);
parse_fen(&fen2, position);
print_board(position);
parse_fen(&fen3, position);
print_board(position);
parse_fen(&fen4, position);
print_board(position);

-----------------------------
Pawn bitboards:
let fen5 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

let position: &mut Board = &mut Board::default();
// debug_board(position);
let result = parse_fen(fen5, position);
match result {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(position);

println!("WhitePawn");
print_bitboard(position.pawns[WHITE as usize]);
println!("BlackPawns");
print_bitboard(position.pawns[BLACK as usize]);
println!("BothPawns");
print_bitboard(position.pawns[BOTH as usize]);
}

-----------------------------
Attack squares:
let fen5 = "8/3q1p2/8/5P2/4Q3/8/8/8 w KQkq - 0 1";

let position: &mut Board = &mut Board::default();
let result = parse_fen(fen5, position);
match result {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(position);

test_square_attacked(WHITE, position);
println!();
test_square_attacked(BLACK, position);

-----------------------------
Move integer bits:
let fen5 = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

let position: &mut Board = &mut Board::default();
// debug_board(position);
match parse_fen(fen5, position) {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(position);
match check_board(position) {
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
let position: &mut Board = &mut Board::default();
// debug_board(position);
match parse_fen(fen5, position) {
    Ok(_) => print!(""),
    Err(e) => println!("{}", e),
}
print_board(position);
match check_board(position) {
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

-----------------------------
See moves generated:
let position: &mut Board = &mut Board::default();
    match parse_fen(&FEN_START, position) {
        Ok(_) => print!(""),
        Err(e) => eprintln!("{}", e),
    }
    println!();

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    let mut test = String::new();
    for move_number in 0..move_list.count {
        let the_move = move_list.moves[move_number].el_move;
        println!("move_number: {}", move_number);

        if !make_move(position, the_move) { continue; }

        println!("MADE: {}", print_move(the_move));
        print_board(position);

        take_move(position);
        println!("TAKEN: {}", print_move(the_move));
        print_board(position);
        std_io::stdin().read_line(&mut test).expect("Failed to read");
-----------------------------
MVVLVA:
for attacker in WhitePawn as usize..BlackKing as usize {
        for victim in WhitePawn as usize..BlackKing as usize {
            println!("{} x {} = {}", PIECE_CHAR.chars().nth(attacker as usize).unwrap_or(' '), PIECE_CHAR.chars().nth(victim as usize).unwrap_or(' '), MVV_LVA_SCORES[victim][attacker]);
        }
    }
-----------------------------
Manual UCI:
let position: &mut Board = &mut Board::default();
    parse_fen(&FEN_61, position);
    let info = &mut SearchInfo::default();
    let mut user_input = String::new();

    loop {
        // break;
        print_board(position);
        user_input.clear();
        println!("Enter a move: ");

        std_io::stdin().read_line(&mut user_input).expect("Error");

        user_input = user_input.trim().to_string();

        if user_input == "q" {
            break;
        } else if user_input == "t" {
            take_move(position);
        } else if user_input == "p" {
            perft_test(5, position);
            // let maximum = get_pv_line(4, position);
            // print!("\nPvLine of {} moves: ", maximum);

            // for pv_number in 0..maximum {
            //     let el_move = position.pv_array[pv_number];
            //     print!(" {}", print_move(el_move));
            // }
            // println!();

        } else if user_input == "s" {
            info.depth = 7;
            info.time = Instant::now();
            info.time_set = true;
            info.stop_time = 3;
            search_position(position, info);
        } else {
            let the_move = parse_move(&user_input, position);
            if the_move != NO_MOVE {
                position.store_pv_move(the_move);
                make_move(position, the_move);

                // if is_repetition(position) { println!("{}", "REPETITION SEEN".green()); }
            } else {
                println!("Move not parsed");
            }
        }
    }
*/
