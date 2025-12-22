#![allow(warnings)]
use std::{hint::black_box, time::{Duration, Instant}};

use chess_engine_2::{
    board::{Board, print_bitboard},
    defs::{Color, FILES_BOARD, Piece, PieceType, RANKS_BOARD, Ranks},
    fens::{FEN_1, FEN_ENPASSANT, FEN_PROMOTION_BLACK, FEN_PROMOTION_WHITE, FEN_SQUARE_ATTACKED, FEN_START, KIWIPETE},
    movegen::{MoveList, bitboards::RANK_BB_MASK},
    perft::{perft, perft_divide}
};
/*
NEXT: PERFT

TODO Test difference between packing bits for moves and a full struct
TODO Move the LazyLock to a build.rs file that generates the random at compile time
TODO Optimize make_move function
*/

fn main() {
    let total_runs = 31;
    let mut times: Vec<Duration> = Vec::with_capacity(total_runs);
    for i in 1..=total_runs {
        let mut position = Board::new(KIWIPETE);
        
        let start = Instant::now();
        let result = black_box(perft(&mut position, black_box(4)));
        let duration = start.elapsed();
        times.push(duration);

        println!("Run {}: {:?} | Nodes: {}", i, duration, result);
    }

    let valid_times = &times[1..]; 
    let sum: Duration = valid_times.iter().sum();
    let avg = sum / valid_times.len() as u32;

    println!("---------------------------------------------------");
    println!("Average time (excluding warm-up): {:?}", avg);
}
