#![allow(warnings)]
use crate::{board::{Board, print_bitboard}, defs::{Castling, Color, FILES_BOARD, Piece, PieceType, RANKS_BOARD, Ranks}, fens::{FEN_1, FEN_ENPASSANT, FEN_PROMOTION_BLACK, FEN_PROMOTION_WHITE, FEN_SQUARE_ATTACKED, FEN_START, KIWIPETE}, movegen::{MoveFlag, MoveList}};

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

TODO Move the LazyLock to a build.rs file that generates the random at compile time

*/

fn main() {
    pub const KING_INTO_CHECK: &str = "k7/8/8/8/8/8/4r3/4K3 w - - 0 1";
    let e1: usize = 4; // King
    let f2: usize = 13; // Square attacked by Rook on e2

    let mut position: Board = Board::new(KING_INTO_CHECK);
    println!("{position}");
    let old_side = position.side;

    // Move King E1 -> F2 (Illegal because e2 Rook attacks rank 2)
    let mv = MoveList::move_builder(
        e1, f2, 0, 0, 0
    );

    // Expect make_move to return false
    assert!(!position.make_move(mv));
}
