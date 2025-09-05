use crate::{board::{print_bitboard, Board}, defs::{FILES_BOARD, RANKS_BOARD}, fens::FEN_TRICKY, movegen::WHITE_PAWN_ATTACKS};

mod defs;
mod board;
mod fens;
mod io;
mod movegen;

fn main() {
    hi();
}
fn hi() {
    let mut position = Board::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
    println!("{position}");

    position.check_board(fn_name!());
    for i in 0..64 {
        print_bitboard(WHITE_PAWN_ATTACKS[i]);
    }
}