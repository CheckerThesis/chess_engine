#![allow(warnings)]
use crate::{board::{Board, print_bitboard}, defs::{Color, FILES_BOARD, RANKS_BOARD, Ranks}, fens::{FEN_1, FEN_ENPASSANT, FEN_PROMOTION_BLACK, FEN_PROMOTION_WHITE, FEN_SQUARE_ATTACKED, FEN_START, KIWIPETE}, movegen::MoveList};

mod defs;
mod board;
mod fens;
mod io;
mod movegen;
mod transposition_table;
/*
TODO
- Determine what struct to use to handle movelist
    - Determine what struct to use to handle move
- Test difference between packing bits for moves and a full struct
    - Change the move bit flags to bools


*/

fn main() {
    pub const POSITION3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let position = Board::new(POSITION3);
    position.check_board(fn_name!());
    let mut move_list: MoveList = MoveList::new();
    move_list.generate_all_moves(&position);
    println!("{position}");
    println!("{move_list}");
}
