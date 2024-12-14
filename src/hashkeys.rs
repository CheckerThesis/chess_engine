use crate::defs::{Board, Pieces::Empty, Squares::{NoSq, OffBoard}, WHITE, BOARD_SQUARE_NUMBER, CASTLE_KEYS, PIECE_KEYS, SIDE_KEY};

pub fn generate_position_key(position: &Board) -> u64 {
    let mut final_key: u64 = 0;

    // add pieces (PIECE_KEYS) to the final_key
    for square in 0..BOARD_SQUARE_NUMBER {
        let piece = position.pieces[square];

        if piece != NoSq as u8 && piece != Empty as u8 && piece != OffBoard as u8 {
            final_key ^= PIECE_KEYS[piece as usize][square];
        }
    }

    // add position (SIDE_KEY) to final_key
    if position.side == WHITE as u8 {
        final_key ^= *SIDE_KEY;
    }

    // add en_passent (position.en_passent) to final_key
    if position.en_passent != NoSq as u8 {
        final_key ^= PIECE_KEYS[Empty as usize][position.en_passent as usize];
    }

    // add castling (CASTLE_KEYS) to final_key
    final_key ^= CASTLE_KEYS[position.castle_permission as usize];

    final_key
}
