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
    pub const WHITE_KINGSIDE: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    let e1: usize = 4;
    let g1: usize = 6;
    let h1: usize = 7;
    let f1: usize = 5;

    let mut position: Board = Board::new(WHITE_KINGSIDE);
    let old_key = position.position_key;

    let mv = MoveList::move_builder(e1, g1, 0, MoveFlag::CASTLE, 0);

    println!("{position}");

    assert!(position.make_move(mv));

    // King moved
    assert!(position.pieces[e1].is_none());
    assert_eq!(position.pieces[g1], Some(Piece { piece_type: PieceType::King, color: Color::White }));
    
    // Rook moved automatically
    assert!(position.pieces[h1].is_none());
    assert_eq!(position.pieces[f1], Some(Piece { piece_type: PieceType::Rook, color: Color::White }));

    assert_eq!(position.side, Color::Black);
    assert_ne!(position.position_key, old_key);
    
    let white_rights_mask = (Castling::WhiteKingCastle as u8) | (Castling::WhiteQueenCastle as u8);
    assert_eq!(position.castle_permission & white_rights_mask, 0); 
    
    position.check_board(fn_name!());
}
