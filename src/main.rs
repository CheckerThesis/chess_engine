#![allow(warnings)]
use std::{hint::black_box, time::{Duration, Instant}};

use vault::{
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

// cargo test mm_ && cargo test _gener && cargo test perft_depth_4 -r

fn main() {
    const SAMPLES: usize = 6;
    const ITERATIONS: u32 = 6;

    let mut samples: Vec<Duration> = Vec::with_capacity(SAMPLES);

    for i in 1..=SAMPLES {
        let start = Instant::now();
        
        for _ in 0..ITERATIONS {
            let mut position = Board::new(KIWIPETE);
            black_box(perft(&mut position, black_box(4)));
        }
        
        let total_duration = start.elapsed();
        let avg_per_run = total_duration / ITERATIONS;
        
        samples.push(avg_per_run);
        
        println!("Sample {}: {:?}", i, avg_per_run);
    }

    let valid_samples = &mut samples[1..]; 
    valid_samples.sort();
    let median = valid_samples[valid_samples.len() / 2];

    println!("Final Median Time: {:?}", median);
}
