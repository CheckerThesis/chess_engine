use crate::{data::{IS_BISHOP_QUEEN, IS_KNIGHT, IS_ROOK_QUEEN}, defs::{from_square, promoted, to_square, FILES_BOARD, RANKS_BOARD}};

use once_cell::sync::Lazy;
use std::sync::Mutex;

static SQUARE_STRING: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
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

static MOVE_STRING: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new(String::new()));
pub fn print_move(the_move: u64) -> String {
    let file_from = FILES_BOARD[from_square(the_move) as usize];
    let rank_from = RANKS_BOARD[from_square(the_move) as usize];
    let file_to = FILES_BOARD[to_square(the_move) as usize];
    let rank_to = RANKS_BOARD[to_square(the_move) as usize];

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
