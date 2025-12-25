#![allow(warnings)]
use std::{hint::black_box, time::{Duration, Instant}};

use vault::{
    board::Board, defs::{Color, Piece, PieceType}, fens::{FEN_START, KIWIPETE}, movegen::{MOVE_FLAG_NONE, MOVE_FLAG_PAWN_START, Move, MoveList}, perft::perft, squares::squares::{B1, B8, C3, C6, E2, E4, E5, F7}
};
/*
lto = "fat"
codegen-units = 1
for release giga speed
*/

// cargo test mm_ && cargo test _gener && cargo test perft_depth_4 -r
fn test() {
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

fn main() {
    // test();
    // let mut position = Board::new(KIWIPETE);
    // let start = Instant::now();
    // println!("{}", perft(&mut position, 5));
    // let total_duration = start.elapsed();
    // println!("Time: {:?}", total_duration);

    let mut position = Board::new(FEN_START);
    let mv = Move::new(E2, E4, PieceType::NONE.index(), MOVE_FLAG_PAWN_START, PieceType::NONE.index());
    position.make_move(mv);
    println!("{position}");
    position.side = Color::WHITE;
    println!("{}", position.evaluate());
}
