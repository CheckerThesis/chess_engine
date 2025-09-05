use crate::defs::{FILES_BOARD, RANKS_BOARD};


pub fn square_string(square: usize) -> String {
    let file = ('a' as u8 + FILES_BOARD[square] as u8) as char;
    let rank = ('1' as u8 + RANKS_BOARD[square] as u8) as char;
    format!("{}{}", file, rank)
}

pub fn move_string(mv: u32) -> String {
    "".to_string()
}