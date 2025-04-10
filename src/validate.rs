use crate::defs::{Pieces::{Empty, BlackKing, WhitePawn}, Squares::OffBoard, BLACK, FILES_BOARD, WHITE};

// error checking

pub fn square_on_board(square: usize) -> bool { FILES_BOARD[square] != OffBoard as u8 }

pub fn side_valid(side: usize) -> bool { side == WHITE || side == BLACK }

// pub fn file_rank_valid(file_rank: usize) -> bool { file_rank >=  Rank1 as usize && file_rank <= Rank7 as usize }

pub fn piece_valid_empty(piece: usize) -> bool { piece >= Empty as usize && piece <= BlackKing as usize }

pub fn piece_valid(piece: usize) -> bool { piece >= WhitePawn as usize && piece <= BlackKing as usize }
