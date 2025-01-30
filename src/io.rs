use crate::{data::{IS_BISHOP_QUEEN, IS_KNIGHT, IS_ROOK_QUEEN}, defs::{Pieces::Empty, fr2sq, from_square, promoted, to_square, Board, MoveList, DEBUG, FILES_BOARD, NO_MOVE, RANKS_BOARD}, movegen::generate_all_moves, validate::square_on_board};

use colored::Colorize;
use std::sync::{LazyLock, Mutex};

static SQUARE_STRING: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

pub fn print_square(square: u8) -> String {
    let file = ('a' as u8 + FILES_BOARD[square as usize]) as char;
    let rank = ('1' as u8 + RANKS_BOARD[square as usize]) as char;

    let square_string = format!("{}{}", file, rank);

    // access SQUARE_STRING, .lock() gives mutable access to String via MutexGuard
    let mut global_square_string = SQUARE_STRING.lock().unwrap();
    // * dereferences MutexGuard giving access to String inside Mutex
    *global_square_string = square_string;
    global_square_string.clone()
}

static MOVE_STRING: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new(String::new()));

pub fn print_move(the_move: u32) -> String {
    let file_from = FILES_BOARD[from_square(the_move) as usize];
    let rank_from = RANKS_BOARD[from_square(the_move) as usize];
    let file_to = FILES_BOARD[to_square(the_move) as usize];
    let rank_to = RANKS_BOARD[to_square(the_move) as usize];

    // get the promoted bit
    let promoted = promoted(the_move);

    let move_string: String;
    if promoted != 0 {
        let mut promoted_char = 'q';
        if IS_KNIGHT[promoted as usize] {
            promoted_char = 'n';
        } else if IS_ROOK_QUEEN[promoted as usize] && !IS_BISHOP_QUEEN[promoted as usize] {
            promoted_char = 'r';
        } else if !IS_ROOK_QUEEN[promoted as usize] && IS_BISHOP_QUEEN[promoted as usize] {
            promoted_char = 'b';
        }
        move_string = format!("{}{}{}{}{}",
            ('a' as u8 + file_from) as char,
            ('1' as u8 + rank_from) as char,
            ('a' as u8 + file_to) as char,
            ('1' as u8 + rank_to) as char,
            promoted_char
        );
    } else {
        move_string = format!("{}{}{}{}",
        ('a' as u8 + file_from) as char,
        ('1' as u8 + rank_from) as char,
        ('a' as u8 + file_to) as char,
        ('1' as u8 + rank_to) as char
        );
    }

    let mut global_move_string = MOVE_STRING.lock().unwrap();
    *global_move_string = move_string;
    global_move_string.clone()
}

// a2a4, find this move and return as integer, match from and to squares to user input
pub fn parse_move(s: &String, position: &mut Board) -> u32 {
    let char_vec: Vec<char> = s.chars().collect();
    // println!("0: {}    1: {}    2: {}    3: {}    4: {}", char_vec[0], char_vec[1], char_vec[2], char_vec[3], char_vec[4]);

    if char_vec[1] > '8' || char_vec[1] < '1' { return NO_MOVE }
    if char_vec[3] > '8' || char_vec[3] < '1' { return NO_MOVE }
    if char_vec[0] > 'h' || char_vec[0] < 'a' { return NO_MOVE }
    if char_vec[2] > 'h' || char_vec[2] < 'a' { return NO_MOVE }

    let from = fr2sq(char_vec[0] as u8 - 'a' as u8, char_vec[1] as u8 - '1' as u8);
    let to = fr2sq(char_vec[2] as u8 - 'a' as u8, char_vec[3] as u8 - '1' as u8);
    // println!("char_vec: {}    from: {}    to: {}", s, from, to);

    if DEBUG && !square_on_board(from as usize) && !square_on_board(to as usize) { eprintln!("{}", "parse_move: [from/to] square not on board".red()); }

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    for move_number in 0..move_list.count {
        let the_move = move_list.moves[move_number].el_move;

        if from_square(the_move) == from && to_square(the_move) == to {
            let promotion_piece = promoted(the_move);

            if promotion_piece != Empty as u8 {
                if IS_ROOK_QUEEN[promotion_piece as usize] && !IS_BISHOP_QUEEN[promotion_piece as usize] && char_vec[4] == 'r' {
                    return the_move
                } else if !IS_ROOK_QUEEN[promotion_piece as usize] && IS_BISHOP_QUEEN[promotion_piece as usize] && char_vec[4] == 'b' {
                    return the_move
                } else if IS_ROOK_QUEEN[promotion_piece as usize] && IS_BISHOP_QUEEN[promotion_piece as usize] && char_vec[4] == 'q' {
                    return the_move
                } else if IS_KNIGHT[promotion_piece as usize] && char_vec[4] == 'n' {
                    return the_move
                }
                continue;
            }
            return the_move
        }
    }

    return NO_MOVE
}

pub fn print_move_list(move_list: &mut MoveList) {
    println!("Move list:");

    for i in 0..move_list.count {
        let the_move = move_list.moves[i].el_move;
        let score = move_list.moves[i].score;

        println!("Move: {} > {} (score: {})", i + 1, print_move(the_move), score);
    }
    println!("Move list total moves: {}", move_list.count);
}
