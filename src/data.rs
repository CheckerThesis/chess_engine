use crate::defs::{self, *};

pub static PIECE_CHAR: &str = ".♙♘♗♖♕♔♟♞♝♜♛♚";
// pub static PIECE_CHAR: &str = ".PNBRQKpnbrqk";
pub static SIDE_CHAR: &str = "wb-";
pub static RANK_CHAR: &str = "12345678";
pub static FILE_CHAR: &str = "abcdefgh";

pub static PIECE_BIG: [bool; 13] = [false, false, true, true, true, true, true, false, true, true, true, true, true];
pub static PIECE_MAJOR: [bool; 13] = [false, false, false, false, true, true, true, false, false, false, true, true, true];
pub static PIECE_MINOR: [bool; 13] = [false, false, true, true, false, false, false, false, true, true, false, false, false];
pub static PIECE_VALUE: [i32; 13] = [0, 100, 325, 325, 550, 1000, 50000, 100, 325, 325, 550, 1000, 50000];
pub static PIECE_COLOR: [u8; 13] = [BOTH as u8, WHITE as u8, WHITE as u8, WHITE as u8, WHITE as u8, WHITE as u8, WHITE as u8, BLACK as u8, BLACK as u8, BLACK as u8, BLACK as u8, BLACK as u8, BLACK as u8];

pub static IS_PAWN: [bool; 13] = [false, true, false, false, false, false, false, true, false, false, false, false, false];
pub static IS_KNIGHT: [bool; 13] = [false, false, true, false, false, false, false, false, true, false, false, false, false];
pub static IS_KING: [bool; 13] = [false, false, false, false, false, false, true, false, false, false, false, false, true];
pub static IS_ROOK_QUEEN: [bool; 13] = [false, false, false, false, true, true, false, false, false, false, true, true, false];
pub static IS_BISHOP_QUEEN: [bool; 13] = [false, false, false, true, false, true, false, false, false, true, false, true, false];

pub static PIECE_SLIDES: [bool; 13] = [false, false, false, true, true, true, false, false, false, true, true, true, false];
