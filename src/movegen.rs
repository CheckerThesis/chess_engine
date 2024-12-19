
/*
fn move_gen(board, moveList)
    loop all pieces
        if slider then loop each dir and add move
            Add move moveList->moves[moveList->moves] = move
            moveList->count++
*/

use crate::{board::check_board, data::PIECE_COLOR, defs::{Board, MoveList, Pieces::*, Ranks::*, Squares::{OffBoard, NoSq}, FILES_BOARD, MOVE_FLAG_EN_PASSENT, MOVE_FLAG_PAWN_START, RANKS_BOARD}, validate::square_on_board, BLACK, WHITE};

pub fn move_builder(from: u64, to: u64, capture: u64, promote: u64, flag: u64) -> u64 { from | (to << 7) | (capture << 14) | (promote << 20) | flag }

// add move to array and increment
pub fn add_quiet_move(position: &mut Board, the_move: u64, move_list: &mut MoveList) {
    move_list.moves[move_list.count].el_move = the_move;
    move_list.moves[move_list.count].score = 0;
    move_list.count += 1;
}

pub fn add_capture_move(position: &mut Board, the_move: u64, move_list: &mut MoveList) {
    move_list.moves[move_list.count].el_move = the_move;
    move_list.moves[move_list.count].score = 0;
    move_list.count += 1;
}

pub fn add_en_passent_move(position: &mut Board, the_move: u64, move_list: &mut MoveList) {
    move_list.moves[move_list.count].el_move = the_move;
    move_list.moves[move_list.count].score = 0;
    move_list.count += 1;
}

pub fn add_white_pawn_capture_move(position: &mut Board, from: usize, to: usize, capture: usize, move_list: &mut MoveList) {
    // if able to promote on capture
    if RANKS_BOARD[from as usize] == Rank7 as u8 {
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, WhiteQueen as u64, 0), move_list);
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, WhiteRook as u64, 0), move_list);
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, WhiteBishop as u64, 0), move_list);
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, WhiteKnight as u64, 0), move_list);
    } else {
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, Empty as u64, 0), move_list);
    }
}

pub fn add_white_pawn_move(position: &mut Board, from: usize, to: usize, move_list: &mut MoveList) {
    if RANKS_BOARD[from as usize] == Rank7 as u8 {
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, WhiteQueen as u64, 0), move_list);
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, WhiteRook as u64, 0), move_list);
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, WhiteBishop as u64, 0), move_list);
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, WhiteKnight as u64, 0), move_list);
    } else {
        add_capture_move(position, move_builder(from as u64, to as u64, Empty as u64, Empty as u64, 0), move_list);
    }
}

pub fn generate_all_moves(position: &mut Board, move_list: &mut MoveList) {
    match check_board(position) {
        Ok(_) => print!(""),
        Err(e) => println!("{}", e),
    }

    move_list.count = 0;
    let side = position.side;
    let piece = Empty;
    let temp_square = 0;

    if side == WHITE as u8 {
        // loop through each WhitePawn
        for piece_number in 0..position.piece_number[WhitePawn as usize] {
            let square: usize = position.piece_list[WhitePawn as usize][piece_number as usize] as usize;
            if !square_on_board(square) { println!("Square not on board"); }

            // non-capture moves
            // if square in front is empty
            if position.pieces[square + 10] == Empty as u8 {
                add_white_pawn_move(position, square, square + 10, move_list);

                // if 2 squares in front is empty
                if RANKS_BOARD[square] == Rank2 as u8 && position.pieces[square + 20] == Empty as u8 {
                    add_quiet_move(position, move_builder(square as u64,
                        (square + 20) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_PAWN_START),
                        move_list
                    );
                }
            }

            // capture moves
            // if top left
            if FILES_BOARD[square + 9] != OffBoard as u8 && PIECE_COLOR[position.pieces[square + 9] as usize] == BLACK as u8 {
                add_white_pawn_capture_move(position, square, square + 9, position.pieces[square + 9] as usize, move_list);
            }
            // if top right
            if FILES_BOARD[square + 11] != OffBoard as u8 && PIECE_COLOR[position.pieces[square + 11] as usize] == BLACK as u8 {
                add_white_pawn_capture_move(position, square, square + 11, position.pieces[square + 11] as usize, move_list);
            }

            if position.en_passent != NoSq as u8 {
                if square + 9 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square + 9) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
                if square + 11 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square + 11) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
            }
        }
    } else {

    }

}
