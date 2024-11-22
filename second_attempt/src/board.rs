use std::collections::HashMap;
use crate::defs::{Board, Files::FileA, Pieces::{self, *}, Ranks::Rank8, Sides::*, Squares::{NoSq, OffBoard}, BOARD_SQUARE_NUMBER, SQ64_TO_SQ120};

pub fn parse_fen(fen: &str, position: &mut Board) {
    reset_board(position);

    let piece_map: HashMap<char, Pieces> = HashMap::from([
        ('p', BlackPawn), ('r', BlackRook), ('n', BlackKnight),
        ('b', BlackBishop), ('k', BlackKing), ('q', BlackQueen),
        ('P', WhitePawn), ('R', WhiteRook), ('N', WhiteKnight),
        ('B', WhiteBishop), ('K', WhiteKing), ('Q', WhiteQueen),
    ]);

    // vector is a linkedlist but back-to-back (essentially an expandable array)
    let fen_split: Vec<&str> = fen.split_whitespace().collect();
    if fen_split.len() < 4 {
        // return Err("Invalid FEN string")
    }

    let mut rank = Rank8 as usize;
    let mut file = FileA as usize;
    // : Option<&Pieces>
    for c in fen_split[0].chars() {
        // piece_map.get(&c) will return either Some(value) (if key exists) or None (if key doesn't exist)
        // if piece_map.get(&c) does not equal None, it will be assigned to piece and excecute the block
        // else it will skip it
        if let Some(piece) = piece_map.get(&c) {
        }
    }

    // for c in fen_split[0].chars() {
    //     if let Some(piece) = piece_map.get(&c) {
    //         let sq64 = FR2SQ!(file as u64, rank as u64);
    //         let sq120 = SQ64!(sq64);
    //         position.pieces[sq120 as usize] = *piece as u64;
    //         file += 1;
    //     } else if let Some(empty_squares) = c.to_digit(10) {
    //         file += empty_squares as usize;
    //     } else if c == '/' {
    //         rank = rank.checked_sub(1).ok_or("Rank underflow")?;
    //         file = FileA as usize;
    //     } else {
    //         return Err("Invalid character in FEN");
    //     }

    //     if file >= 8 {
    //         rank = rank.checked_sub(1).ok_or("Rank underflow")?;
    //         file = FileA as usize;
    //     }

    //     if rank < Rank1 as usize {
    //         break;
    //     }
    // }
}

pub fn reset_board(position: &mut Board) {
    // set all squares to off board
    for i in 0..BOARD_SQUARE_NUMBER {
        position.pieces[i] = OffBoard as u8;
    }

    // set small board squares to empty
    for i in 0..64 {
        position.pieces[SQ64_TO_SQ120[i] as usize] = Empty as u8;
    }

    for i in 0..3 {
        position.big_piece[i] = 0;
        position.major_piece[i] = 0;
        position.minor_piece[i] = 0;
        position.pawns[i] = 0;
    }

    for i in 0..13 {
        position.piece_number[i] = 0;
    }

    position.king_square[White as usize] = NoSq as u8;
    position.king_square[Black as usize] = NoSq as u8;

    position.side = Both as u8;
    position.en_passent = NoSq as u8;
    position.fifty_move = 0;

    position.ply = 0;
    position.history_ply = 0;

    position.castle_permission = 0;

    position.position_key = 0;
}
