#![allow(warnings)]
use std::{hint::black_box, sync::{Arc, atomic::Ordering}, time::{Duration, Instant}};

use vault::{
    board::Board, defs::{Color, Piece, PieceType}, fens::{ENDGAME1, FEN_MATE_IN_4, FEN_START, ITALIAN, KIWIPETE, MIDDLEGAME1, POSITION3, POSITION4, POSITION4_FLIPPED}, movegen::{MOVE_FLAG_NONE, MOVE_FLAG_PAWN_START, Move, MoveList, attacks::square_attacked}, perft::perft, search::{Search, evaluate::{EVAL_CALLS, EVAL_TIME_NS}}, squares::squares::{B1, B8, C3, C6, E2, E4, E5, F7}, test_suites::run_suites, transposition_table::{self, TranspositionTable}, uci::uci_loop
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
    const SAMPLES: usize = 1;
    const ITERATIONS: u32 = 5;
    const DEPTH: u8 = 13;

    let test_positions = [
        ("Tactical", KIWIPETE),
        ("Opening", "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4"),
        ("Quiet Mid", "r1bq1rk1/ppp2ppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQ1RK1 w - - 0 8"),
        ("Endgame", "8/pp3k2/2p2p2/3p1Pp1/3P2P1/2P2K2/PP6/8 w - - 0 1"),
    ];

    let mut samples: Vec<(Duration, u64)> = Vec::with_capacity(SAMPLES);

    for sample_num in 1..=SAMPLES {
        let start = Instant::now();
        let mut total_nodes = 0;
        let mut correctness_ok = true;

        for _ in 0..ITERATIONS {
            for (i, (name, fen)) in test_positions.iter().enumerate() {
                let tt = Arc::new(TranspositionTable::new(20));
                let mut position = Board::new(fen);
                let search = Arc::new(Search::new(tt.clone(), 0));

                let best_move = black_box(position.iterative_deepen(&search, black_box(DEPTH), false));
                total_nodes += search.nodes_visited.load(Ordering::Relaxed);
            }
        }

        let duration = start.elapsed();
        samples.push((duration, total_nodes as u64));
        println!(
            "Sample {}: {:.2}s, {} nodes", 
            sample_num, 
            duration.as_secs_f64(), 
            total_nodes,
        );
    }

    let valid_samples = &mut samples[0..];
    valid_samples.sort_by_key(|(d, _)| *d);

    let median_idx = valid_samples.len() / 2;
    let (median_time, median_nodes) = valid_samples[median_idx];

    let total_iterations = ITERATIONS as u64 * test_positions.len() as u64;
    println!("----------------------------------------");
    println!("    - Median Time: {:?}", median_time);
    println!("    - Median Nodes: {}", median_nodes);
    println!(
        "    - Speed: {:.2} Mnps", 
        median_nodes as f64 / median_time.as_secs_f64() / 1_000_000.0
    );
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

    // let mut nodes_sum: f64 = 0.0;
    // let mut time_sum: f64 = 0.0;
    // let iterations = 3;

    // println!("Countermove");

    // for i in 0..iterations {
    //     let (nodes, time) = test_move_ordering(9);
    //     nodes_sum += nodes;
    //     time_sum += time;
    // }

    // println!("--------");
    // println!("Nodes: {}   Time: {:.2}s   ({:.2} Mnps)", 
    //     nodes_sum / iterations as f64, 
    //     time_sum / iterations as f64,
    //     (nodes_sum as f64 / time_sum) / 1_000_000.0
    // );
}

// TODO Eval king safety
// TODO Pawn structure (doubled, isolated, passed pawns)
// TODO Piece mobility
// TODO Piece coordination

// Future:
// Bucket transposition table
fn main() {
    // test_search_changes();
    // run_suites("/home/tien/code/chess_engine/src/test_suites/wac/wac.epd");

    // let tt = Arc::new(TranspositionTable::new(24));
    // let mut position = Board::new(KIWIPETE);
    // let search = Arc::new(Search::new(tt.clone(), 0));
    // let best_move = position.iterative_deepen(&search, 17, true);
    // let first = search.first_cutoffs.load(Ordering::Relaxed);
    // let total = search.total_cutoffs.load(Ordering::Relaxed);
    // println!("first: {}    total: {}    percent: {:.2}%", 
    //     first,
    //     total,
    //     (first as f64 / total as f64) * 100.0
    // );

    uci_loop();

    // let tt = Arc::new(TranspositionTable::new(20));
    // let mut position = Board::new(FEN_START);
    // let search = Arc::new(Search::new(tt.clone(), 0));
    // let best_move = position.iterative_deepen(&search, 15, true);
    // let mut position = Board::new(KIWIPETE);
    // println!("{}", position.evaluate()); // 90

    // position.parse_fen(POSITION3);
    // println!("{}", position.evaluate()); // -5

    // position.parse_fen(POSITION4);
    // println!("{}", position.evaluate()); // 175

    // position.parse_fen(POSITION4_FLIPPED); // 175
    // println!("{}", position.evaluate());

    // let mut position = Board::new(KIWIPETE);
    // position.evaluate();

    // pub fn print_eval_stats() {
    //     let calls = EVAL_CALLS.load(Ordering::Relaxed);
    //     let total_ns = EVAL_TIME_NS.load(Ordering::Relaxed);
        
    //     println!("Eval calls: {}", calls);
    //     println!("Total eval time: {:.2}s", total_ns as f64 / 1e9);
    //     println!("Avg eval time: {:.1}ns", total_ns as f64 / calls as f64);
    // }

    // print_eval_stats();

    // let mut position = Board::new(FEN_START);
    // println!("START: {}", position.evaluate());
    // position = Board::new(KIWIPETE);
    // println!("KIWIPETE: {}", position.evaluate());
    // position = Board::new(POSITION3);
    // println!("POSITION3: {}", position.evaluate());
    // position = Board::new(POSITION4);
    // println!("POSITION4: {}", position.evaluate());
}
