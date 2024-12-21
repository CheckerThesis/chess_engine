use crate::defs::{Pieces::{Empty, BlackKing, WhitePawn}, Ranks::{Rank1, Rank7}, Squares::OffBoard, BLACK, FILES_BOARD, WHITE};

// error checking

pub fn square_on_board(square: usize) -> bool { FILES_BOARD[square] != OffBoard as u8 }
// if !square_on_board(square as usize) { println!("Square not on board") }

pub fn side_valid(side: usize) -> bool { side == WHITE || side == BLACK }
// if !side_valid(side as usize) { println!("Side not valid") }

pub fn file_rank_valid(file_rank: usize) -> bool { file_rank >=  Rank1 as usize && file_rank <= Rank7 as usize }
// if !file_rank_valid(square as usize) { println!("File/rank not valid") }

pub fn piece_valid_empty(piece: usize) -> bool { piece >= Empty as usize && piece <= BlackKing as usize }
// if !piece_valid_empty(square as usize) { println!("Piece empty not valid") }

pub fn piece_valid(piece: usize) -> bool { piece >= WhitePawn as usize && piece <= BlackKing as usize }
// if !piece_valid(square as usize) { println!("Piece not valid") }
