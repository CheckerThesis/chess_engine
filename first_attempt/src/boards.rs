use std::collections::HashMap;
use crate::{defs::{
    BoardState, Castling::*, Files::*, Pieces::{self, *}, Ranks::*, Sides::{Black, Both, White}, Squares::{self, NoSq, OffBoard}, BOARD_SQUARE_NUMBER, SQ64_TO_SQ120
}, hashkeys::generate_position_key, FR2SQ, SQ120, SQ64};
use crate::data::{PIECE_CHAR, SIDE_CHAR, RANK_CHAR, FILE_CHAR};

pub fn parse_fen(fen: &str, position: &mut BoardState) -> Result<(), &'static str> {
    let mut rank = Rank8 as usize;
    let mut file = FileA as usize;

    reset_board(position);

    let piece_map: HashMap<char, Pieces> = [
        ('p', BlackPawn), ('r', BlackRook), ('n', BlackKnight),
        ('b', BlackBishop), ('k', BlackKing), ('q', BlackQueen),
        ('P', WhitePawn), ('R', WhiteRook), ('N', WhiteKnight),
        ('B', WhiteBishop), ('K', WhiteKing), ('Q', WhiteQueen),
    ].into_iter().collect();

    let fen_split: Vec<&str> = fen.split_whitespace().collect();
    if fen_split.len() < 4 {
        return Err("Invalid FEN string")
    }

    let mut rank = Rank8 as usize;
    let mut file = FileA as usize;

    for c in fen_split[0].chars() {
        if let Some(piece) = piece_map.get(&c) {
            let sq64 = FR2SQ!(file as u64, rank as u64);
            let sq120 = SQ64!(sq64);
            position.pieces[sq120 as usize] = *piece as u64;
            file += 1;
        } else if let Some(empty_squares) = c.to_digit(10) {
            file += empty_squares as usize;
        } else if c == '/' {
            rank = rank.checked_sub(1).ok_or("Rank underflow")?;
            file = FileA as usize;
        } else {
            return Err("Invalid character in FEN");
        }

        if file >= 8 {
            rank = rank.checked_sub(1).ok_or("Rank underflow")?;
            file = FileA as usize;
        }

        if rank < Rank1 as usize {
            break;
        }
    }


    position.side = match fen_split[1] {
        "w" => White as u64,
        "b" => Black as u64,
        _ => return Err("Invalid FEN part 2")
    };

    for i in fen_split[2].chars() {
        match i {
            'K' => position.castle_permission |= WhiteKingCastle as u64,
            'Q' => position.castle_permission |= WhiteQueenCastle as u64,
            'k' => position.castle_permission |= BlackKingCastle as u64,
            'q' => position.castle_permission |= BlackQueenCastle as u64,
            '-' => break,
            _ => return Err("Invalid castling permission in FEN"),
        }
    }

    if fen_split[3] != "-" {
        let file = fen_split[3].chars().next().unwrap() as usize - 'a' as usize;
        let rank = fen_split[3].chars().nth(1).unwrap().to_digit(10).unwrap() as usize - 1;
        position.en_passent = FR2SQ!(file, rank) as u64;
    }

    position.position_key = generate_position_key(position);

    Ok(())
}

pub fn reset_board(position: &mut BoardState) {
    for i in 0..BOARD_SQUARE_NUMBER {
        position.pieces[i] = OffBoard as u64;
    }

    for i in 0..64 {
        position.pieces[SQ64_TO_SQ120[i]] = Empty as u64;
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

    position.king_square[White as usize] = NoSq as u64;
    position.king_square[Black as usize] = NoSq as u64;

    position.side = Both as u64;
    position.en_passent = NoSq as u64;
    position.fifty_move = 0;

    position.ply = 0;
    position.history_ply = 0;

    position.castle_permission = 0;

    position.position_key = 0;
}

pub fn print_board(position: &mut BoardState) {
    println!("Game board:\n");

    for rank in (Rank1 as usize..=Rank8 as usize).rev() {
        print!("{}  ", (rank + 1));
        for file in FileA as usize..=FileH as usize {
            let sq: usize = FR2SQ!(file, rank);
            let piece = position.pieces[sq] as usize;
            print!("{}", PIECE_CHAR.chars().nth(piece).unwrap_or(' '));
        }
        println!();
    }
}
