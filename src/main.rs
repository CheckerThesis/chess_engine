#![allow(warnings)]
use crate::{board::{Board, print_bitboard}, defs::{Castling, Color, FILES_BOARD, Piece, PieceType, RANKS_BOARD, Ranks}, fens::{FEN_1, FEN_ENPASSANT, FEN_PROMOTION_BLACK, FEN_PROMOTION_WHITE, FEN_SQUARE_ATTACKED, FEN_START, KIWIPETE}, movegen::{MoveFlag, MoveList}, perft::perft};

mod defs;
mod board;
mod fens;
mod io;
mod movegen;
mod transposition_table;
mod perft;
mod squares;
/*
NEXT: PERFT

TODO Test difference between packing bits for moves and a full struct
TODO Move the LazyLock to a build.rs file that generates the random at compile time
TODO Optimize make_move function
*/

fn main() {
    let mut position = Board::new(FEN_START);
    println!("{}", perft(&mut position, 2));
    let mut position = Board::new(FEN_START);
    println!("{}", perft(&mut position, 3));
    let mut position = Board::new(FEN_START);
    println!("{}", perft(&mut position, 4));
    let mut position = Board::new(FEN_START);
    println!("{}", perft(&mut position, 5));
}
