#![allow(warnings)]
use std::{hint::black_box, time::{Duration, Instant}};

use vault::{
    board::Board, fens::KIWIPETE, perft::perft
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
    test();
    // let mut position = Board::new(KIWIPETE);
    // let start = Instant::now();
    // println!("{}", perft(&mut position, 5));
    // let total_duration = start.elapsed();
    // println!("Time: {:?}", total_duration);

    // for i in 0..64 {
    //     print!("{},", KING_RAYS[i]);

    //     if (i + 1) % 4 == 0 {
    //         println!();
    //     }
    // }
}
