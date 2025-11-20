#![allow(warnings)]
use crate::{board::{Board, print_bitboard}, defs::{Color, FILES_BOARD, RANKS_BOARD, Ranks}, fens::{FEN_ENPASSANT, FEN_PROMOTION_BLACK, FEN_PROMOTION_WHITE, FEN_TRICKY}, movegen::{MoveList}};

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
    let position: Board = Board::new(FEN_TRICKY);
    let mut move_list: MoveList = MoveList::new();
    println!("{position}");
    position.check_board(fn_name!());
    move_list.generate_king_moves(&position);
    println!("{move_list}");

    // let side = position.side;
    // let our_occupancy = position.occupancies(side);
    // let their_occupancy = position.occupancies(side.opposite());
}
