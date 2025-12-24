#![allow(warnings)]
use std::{collections::HashMap, hint::black_box, time::{Duration, Instant}};

use vault::{
    board::{Board, print_bitboard, set_bit},
    defs::{Color, FILES_BOARD, Piece, PieceType, RANKS_BOARD, Ranks},
    fens::{FEN_1, FEN_ENPASSANT, FEN_PROMOTION_BLACK, FEN_PROMOTION_WHITE, FEN_SQUARE_ATTACKED, FEN_START, KIWIPETE},
    movegen::{MoveList, attacks::{get_demand_moves, get_down_moves, get_left_moves, get_right_moves, get_supply_moves, get_up_moves}, bitboards::RANK_BB_MASK, magic::{BISHOP_MAGIC_BB, MagicTable, ROOK_MAGIC_BB}},
    perft::{perft, perft_divide}, squares::squares::{A6, B2, B3, B5, B7, C3, C4, D3, D5, E2, E3, E4, E5, E6, E7, F3, F5, G3, G5}
};
/*
lto = "fat"
codegen-units = 1
for release giga speed

NEXT: PERFT

TODO Test difference between packing bits for moves and a full struct
TODO Move the LazyLock to a build.rs file that generates the random at compile time
TODO Optimize make_move function
*/

// cargo test mm_ && cargo test _gener && cargo test perft_depth_4 -r  && cargo r -r
fn test() {
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

fn main() {
    // let position = Board::new(KIWIPETE);
    // println!("{position}");
    // let square = F3;
    // print_bitboard(
    //     (ROOK_MAGIC_BB.get_attacks(square, position.occupancies[Color::BOTH.index()]) |
    //     BISHOP_MAGIC_BB.get_attacks(square, position.occupancies[Color::BOTH.index()])) & !position.occupancies[Color::WHITE.index()]
    // );

    test();
    // let mut position = Board::new(KIWIPETE);
    // let start = Instant::now();
    // println!("{}", perft(&mut position, 5));
    // let total_duration = start.elapsed();
    // println!("Time: {:?}", total_duration);
}
