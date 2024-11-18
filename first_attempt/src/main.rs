#![allow(unused)]

mod defs;
mod bitboards;
mod hashkeys;
mod boards;
mod data;

use boards::print_board;
use defs::START_FEN;

use crate::{defs::{Squares::*, BoardState}};
use crate::{bitboards::{print_bitboard, count_bits, pop_bit}};
use crate::hashkeys::generate_position_key;
use crate::boards::{reset_board, parse_fen};

fn main() {
    // let mut play_bit_board: u64 = 0;

    // play_bit_board |= (1u64 << SQ64!(D2));
    // play_bit_board |= (1u64 << SQ64!(D3));
    // play_bit_board |= (1u64 << SQ64!(D4));
    // print_bitboard(play_bit_board);

    // print!("\n");

    // let mut count = COUNT!(play_bit_board);
    // println!("Count: {count}");

    // let mut index = POP!(&mut play_bit_board);
    // println!("Index: {index}");

    // print_bitboard(play_bit_board);

    // count = COUNT!(play_bit_board);
    // println!("\nCount: {count}");
    // index = POP!(&mut play_bit_board);
    // println!("Index: {index}");

    // print_bitboard(play_bit_board);

    // count = COUNT!(play_bit_board);
    // println!("\nCount: {count}");
    // index = POP!(&mut play_bit_board);
    // println!("Index: {index}");

    // SETBIT!(play_bit_board, 63);
    // print_bitboard(play_bit_board);

    // CLEARBIT!(play_bit_board, 27);
    // print_bitboard(play_bit_board);

    // println!("\n\n");

    let my_board: &mut BoardState = &mut BoardState::default();
    parse_fen(START_FEN, my_board);
    print_board(my_board);

    // generate_position_key(play_bit_board);

    // print!("{}", SQ64_TO_SQ120[27]);

    // for i in 0..BOARD_SQUARE_NUMBER {
    //     if i % 10 == 0 {
    //         println!();
    //     }
    //     print!("{}   ", SQ120_TO_SQ64[i]);
    // }

    // println!();
    // println!();

    // for i in 0..64 {
    //     if i % 8 == 0 {
    //         println!();
    //     }
        // print!("{}   ", SQ64_TO_SQ120[i])
    // }
}
