#![allow(warnings)]
use chess_engine_2::{
    board::{Board, print_bitboard},
    defs::{Color, FILES_BOARD, Piece, PieceType, RANKS_BOARD, Ranks},
    fens::{FEN_1, FEN_ENPASSANT, FEN_PROMOTION_BLACK, FEN_PROMOTION_WHITE, FEN_SQUARE_ATTACKED, FEN_START, KIWIPETE},
    movegen::{MoveList, bitboards::RANK_BB_MASK},
    perft::{perft, perft_divide}
};
/*
NEXT: PERFT

TODO Test difference between packing bits for moves and a full struct
TODO Move the LazyLock to a build.rs file that generates the random at compile time
TODO Optimize make_move function
*/

fn main() {
    use std::time::Instant;
    let now = Instant::now();
    let mut position: Board = Board::new(FEN_START);
    println!("{}", perft(&mut position, 6));
    println!("{:.2?}", now.elapsed());
}
