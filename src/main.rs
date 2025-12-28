#![allow(warnings)]
use std::{hint::black_box, sync::Arc, time::{Duration, Instant}};

use vault::{
    board::Board, defs::{Color, Piece, PieceType}, fens::{FEN_START, KIWIPETE}, movegen::{MOVE_FLAG_NONE, MOVE_FLAG_PAWN_START, Move, MoveList, attacks::square_attacked}, perft::perft, search::Search, squares::squares::{B1, B8, C3, C6, E2, E4, E5, F7}
};
/*
lto = "fat"
codegen-units = 1
for release giga speed
*/

// cargo test mm_ && cargo test _gener && cargo test perft_depth_4 -r
fn test_perft_changes() {
    const SAMPLES: usize = 3;
    const ITERATIONS: u32 = 3;

    let mut samples: Vec<Duration> = Vec::with_capacity(SAMPLES);
    for i in 1..=SAMPLES {
        let start = Instant::now();
        
        for _ in 0..ITERATIONS {
            let mut position = Board::new(KIWIPETE);
            black_box(perft(&mut position, black_box(5)));
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

fn test_search_changes() {
    // const SAMPLES: usize = 7;
    // const ITERATIONS: u32 = 6;
    // const DEPTH: u8 = 6;
    const SAMPLES: usize = 7;
    const ITERATIONS: u32 = 7;
    const DEPTH: u8 = 8;

    let mut samples: Vec<Duration> = Vec::with_capacity(SAMPLES);
    for i in 1..=SAMPLES {
        let start = Instant::now();
        
        for _ in 0..ITERATIONS {
            let mut position = Board::new(KIWIPETE);
            let search = Arc::new(Search::new(20));
            black_box(position.iterative_deepen(&search, black_box(DEPTH), false));
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

// Future:
// Bucket transposition table
fn main() {
    test_search_changes();

    // let search = Arc::new(Search::new(20));
    // let mut position = Board::new(KIWIPETE);
    // let mut movelist = MoveList::new();
    // movelist.generate_all_moves(&position);
    // println!("{movelist}");
}
