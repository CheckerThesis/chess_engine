use colored::Colorize;

use crate::{board::check_board, data::{IS_BISHOP_QUEEN, IS_KING, IS_KNIGHT, IS_ROOK_QUEEN, PIECE_COLOR}, defs::{fr2sq, Board, Files::*, Pieces::*, Ranks::*, Squares::OffBoard, DEBUG, WHITE}, validate::{side_valid, square_on_board}};

const KNIGHT_DIRECTION: [i8; 8] = [-8, -19, -21, -12, 8, 19, 21, 12];
const ROOK_DIRECTION: [i8; 4] = [-1, -10, 1, 10];
const BISHOP_DIRECTION: [i8; 4] = [-9, -11, 9, 11];
const KING_DIRECTION: [i8; 8] = [-1, -10, 1, 10, -9, -11, 9, 11];

// square is the square we're interested in attacking
pub fn square_attacked(square: usize, side: usize, position: &mut Board) -> bool {
    let squarei8 = square as i8;

    if DEBUG {
        if !square_on_board(square) { eprintln!("{}", "square_attacked: [square] square not on board".red()); }
        if !side_valid(side) { eprintln!("{}", "square_attacked: [side] side not valid".red()); }
        check_board(position);
    }

    // if white, check bottom left and bottom right square if there is WhitePawn
    if side == WHITE {
        if position.pieces[square - 11] == WhitePawn as u8 || position.pieces[square - 9] == WhitePawn as u8 {
            return true
        }
    // else black, check top left and top right square if there is BlackPawn
    } else {
        if position.pieces[square + 11] == BlackPawn as u8 || position.pieces[square + 9] == BlackPawn as u8 {
            return true
        }
    }

    // check knight attacks in range
    for i in 0..8 {
        // CARE for i8 cast
        let piece = position.pieces[(squarei8 + KNIGHT_DIRECTION[i]) as usize] as usize;

        if piece < 13 && IS_KNIGHT[piece] && PIECE_COLOR[piece] == side as u8 {
            return true
        }
    }

    // rooks and queens
    for i in 0..4 {
        let direction = ROOK_DIRECTION[i];
        let mut temp_square = squarei8 + direction;
        let mut piece = position.pieces[temp_square as usize];

        while piece != OffBoard as u8 {
            if piece != Empty as u8 {
                if IS_ROOK_QUEEN[piece as usize] && PIECE_COLOR[piece as usize] == side as u8 {
                    return true
                }

                break;
            }

            temp_square += direction;
            piece = position.pieces[temp_square as usize];
        }
    }

    // bishops and queens
    for i in 0..4 {
        let direction = BISHOP_DIRECTION[i];
        let mut temp_square = squarei8 + direction;
        let mut piece = position.pieces[temp_square as usize];

        while piece != OffBoard as u8 {
            if piece != Empty as u8 {
                if IS_BISHOP_QUEEN[piece as usize] && PIECE_COLOR[piece as usize] == side as u8 {
                    return true
                }

                break;
            }

            temp_square += direction;
            piece = position.pieces[temp_square as usize];
        }
    }

    // king
    for i in 0..8 {
        // CARE for i8 cast
        let piece = position.pieces[(squarei8 + KING_DIRECTION[i]) as usize] as usize;

        if piece < 13 && IS_KING[piece] && PIECE_COLOR[piece] == side as u8 {
            return true
        }
    }

    return false
}

// shows attacked squares

pub fn test_square_attacked(side: usize, position: &mut Board) {
    println!("{}", side);
    for rank in (Rank1 as u8..=Rank8 as u8).rev() {
        for file in FileA as u8..=FileH as u8 {
            let square = fr2sq(file, rank);
            if square_attacked(square.into(), side, position) {
                print!("X");
            } else {
                print!("-");
            }
        }
        println!();
    }
}
