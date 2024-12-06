use crate::defs::Sides::{self, *};

pub static PIECE_CHAR: &str = ".♙♘♗♖♕♔♟♞♝♜♛♚";
// pub static PIECE_CHAR: &str = ".PNBRQKpnbrqk";
pub static SIDE_CHAR: &str = "wb-";
pub static RANK_CHAR: &str = "12345678";
pub static FILE_CHAR: &str = "abcdefgh";

pub static PIECE_BIG: [bool; 13] = [false, false, true, true, true, true, true, false, true, true, true, true, true];
pub static PIECE_MAJOR: [bool; 13] = [false, false, false, false, true, true, true, false, false, false, true, true, true];
pub static PIECE_MINOR: [bool; 13] = [false, false, true, true, false, false, false, false, true, true, false, false, false];
pub static PIECE_VALUE: [u16; 13] = [0, 100, 325, 325, 550, 1000, 50000, 100, 325, 325, 550, 1000, 50000];
pub static PIECE_COLOR: [u8; 13] = [Both as u8, White as u8, White as u8, White as u8, White as u8, White as u8, White as u8, Black as u8, Black as u8, Black as u8, Black as u8, Black as u8, Black as u8];
