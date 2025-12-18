#![allow(warnings)]
use crate::{board::{Board, print_bitboard}, defs::{Color, FILES_BOARD, Piece, PieceType, RANKS_BOARD, Ranks}, fens::{FEN_1, FEN_ENPASSANT, FEN_PROMOTION_BLACK, FEN_PROMOTION_WHITE, FEN_SQUARE_ATTACKED, FEN_START, KIWIPETE}, movegen::{MoveFlag, MoveList, bitboards::RANK_BB_MASK}, perft::{perft, perft_divide}};

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
    // let mut position: Board = Board::new(FEN_START);
    // println!("{}", perft(&mut position, 2));
    // let mut position = Board::new(FEN_START);
    // println!("{}", perft(&mut position, 3));
    // let mut position = Board::new("rnbqk1nr/pppp1ppp/4p3/8/1b6/PP6/2PPPPPP/RNBQKBNR w KQkq - 0 1");
    // println!("{position}");
    // let mut movelist = MoveList::new();
    // movelist.generate_all_moves(&position);
    // println!("{movelist}");
    // perft_divide(&mut position, 2);
}
