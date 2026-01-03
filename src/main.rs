#![allow(warnings)]
use std::{hint::black_box, sync::{Arc, atomic::Ordering}, time::{Duration, Instant}};

use vault::{
    board::Board, defs::{Color, Piece, PieceType}, fens::{ENDGAME1, FEN_MATE_IN_4, FEN_START, ITALIAN, KIWIPETE, MIDDLEGAME1, POSITION3, POSITION4}, movegen::{MOVE_FLAG_NONE, MOVE_FLAG_PAWN_START, Move, MoveList, attacks::square_attacked}, perft::perft, search::Search, squares::squares::{B1, B8, C3, C6, E2, E4, E5, F7}, sts::run_sts, transposition_table::{self, TranspositionTable}
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
        let mut nodes = 0;
        let transposition_table = Arc::new(TranspositionTable::new(20));
        for _ in 0..ITERATIONS {
            let mut position = Board::new(KIWIPETE);
            let search = Arc::new(Search::new(transposition_table.clone(), 1000));
            black_box(position.iterative_deepen(&search, black_box(DEPTH), false));
            nodes = search.nodes_visited.load(Ordering::Relaxed);
        }
        
        let total_duration = start.elapsed();
        let avg_per_run = total_duration / ITERATIONS;
        
        samples.push(avg_per_run);
        
        println!("Sample {}: {:?} Nodes: {}", i, avg_per_run, nodes);
    }

    let valid_samples = &mut samples[1..]; 
    valid_samples.sort();
    let median = valid_samples[valid_samples.len() / 2];

    println!("Final Median Time: {:?}", median);
}

fn test_move_ordering(depth: u8) -> (f64, f64) {
    let test_positions = [FEN_START, KIWIPETE, POSITION3, POSITION4, ITALIAN, MIDDLEGAME1, ENDGAME1];
    let test_positions_names = ["Start", "Kiwipete", "3", "4", "Italian", "Mid game", "End game"];

    let mut total_nodes = 0;
    let total_start = Instant::now();

    for (i, fen) in test_positions.iter().enumerate() {
        let mut transposition_table = Arc::new(TranspositionTable::new(20));
        let mut position = Board::new(fen);
        let search = Arc::new(Search::new(transposition_table.clone(), 0));
        
        let pos_start = Instant::now();
        position.iterative_deepen(&search, depth, false);
        let pos_time = pos_start.elapsed();

        let nodes = search.nodes_visited.load(Ordering::Relaxed);
        total_nodes += nodes;

        println!("{}: {}, {:.2}ms", test_positions_names[i], nodes, pos_time.as_secs_f64() * 1000.0);
    }
    let total_time = total_start.elapsed();
    println!(
        "TOTAL: {} nodes, {:.2}s ({:.2} Mnps)", 
        total_nodes, 
        total_time.as_secs_f64(),
        total_nodes as f64 / total_time.as_secs_f64() / 1_000_000.0
    );

    (total_nodes as f64, total_time.as_secs_f64())
}

// TODO Countermove heuristic

// Future:
// Bucket transposition table
fn main() {
    let mut nodes_sum: f64 = 0.0;
    let mut time_sum: f64 = 0.0;
    let iterations = 3;

    println!("History");

    for i in 0..iterations {
        let (nodes, time) = test_move_ordering(9);
        nodes_sum += nodes;
        time_sum += time;
    }

    println!("--------");
    println!("Nodes: {}   Time: {:.2}s   ({:.2} Mnps)", 
        nodes_sum / iterations as f64, 
        time_sum / iterations as f64,
        (nodes_sum as f64 / time_sum) / 1_000_000.0
    );

    // run_sts("/home/tien/code/chess_engine/src/sts/STS1.epd");
}
