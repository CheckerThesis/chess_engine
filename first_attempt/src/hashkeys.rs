use crate::defs::{BoardState, Pieces::Empty, Sides::White, Squares::NoSq, BOARD_SQUARE_NUMBER, PIECE_KEYS, SIDE_KEY, CASTLE_KEYS};

pub fn generate_position_key(position: &BoardState) -> u64 {
    let mut final_key: u64 = 0;

    for square in 0..BOARD_SQUARE_NUMBER {
        let piece = position.pieces[square];
        if (piece != NoSq as u64 && piece != Empty as u64) {
            // must be an actual piece (between WhitePawn & BlackKing)
            final_key ^= PIECE_KEYS[piece as usize][square];
        }
    }
    if (position.side == White as u64) {
        final_key ^= *SIDE_KEY;
    }

    if (position.en_passent != NoSq as u64) {
        final_key ^= PIECE_KEYS[Empty as usize][position.en_passent as usize];
    }

    final_key ^= CASTLE_KEYS[position.castle_permission as usize];

    final_key
}

// use crate::defs::{BoardState, Pieces, Sides, Squares, BOARD_SQUARE_NUMBER};
// use crate::defs::{PIECE_KEYS, SIDE_KEY, CASTLE_KEYS};

// pub fn generate_position_key(position: &BoardState) -> u64 {
//     let mut final_key: u64 = 0;

//     for square in 0..BOARD_SQUARE_NUMBER {
//         let piece = position.pieces[square];
//         if piece != Pieces::Empty {
//             // must be an actual piece (between WhitePawn & BlackKing)
//             final_key ^= PIECE_KEYS[square][piece as usize];
//         }
//     }

//     if position.side == Sides::WHITE as u64 {
//         final_key ^= *SIDE_KEY;
//     }

//     if position.en_passent != Squares::NoSq as u64 {
//         final_key ^= PIECE_KEYS[position.en_passent as usize][Pieces::Empty as usize];
//     }

//     final_key ^= CASTLE_KEYS[position.castle_permission as usize];

//     final_key
// }