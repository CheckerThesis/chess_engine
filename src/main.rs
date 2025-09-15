use crate::{board::{print_bitboard, Board}, defs::{FILES_BOARD, RANKS_BOARD}, fens::FEN_TRICKY, movegen::WHITE_PAWN_ATTACKS};

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
    let position: Board = Board::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    println!("{position}");
    position.check_board(fn_name!());
}
