use colored::Colorize;

use crate::{board::{check_board, reset_board, update_lists_material}, data::{MIRROR64, PIECE_VALUE}, defs::{sq120, sq64, Board, Castling::*, Pieces::*, Squares::*, BLACK, BLACK_PASSED_MASK, BOTH, DEBUG, FILES_BOARD, FILE_BB_MASK, ISOLATED_MASK, RANKS_BOARD, WHITE, WHITE_PASSED_MASK}, hashkeys::generate_position_key, validate::square_on_board};

const PAWN_ISOLATED: i32 = -10;
const PAWN_PASSED : [i32; 8] = [0 , 5, 10, 20, 35, 60, 100, 200];
const ROOK_OPEN_FILE: i32 = 10;
const ROOK_SEMI_OPEN_FILE: i32 = 5;
const QUEEN_OPEN_FILE: i32 = 5;
const QUEEN_SEMI_OPEN_FILE: i32 = 3;
const BISHOP_PAIR: i32 = 30;

const PAWN_TABLE: [i32; 64] = [
    0	,	0	,	0	,	0	,	0	,	0	,	0	,	0	,
    10	,	10	,	0	,	-10	,	-10	,	0	,	10	,	10	,
    5	,	0	,	0	,	5	,	5	,	0	,	0	,	5	,
    0	,	0	,	10	,	20	,	20	,	10	,	0	,	0	,
    5	,	5	,	5	,	10	,	10	,	5	,	5	,	5	,
    10	,	10	,	10	,	20	,	20	,	10	,	10	,	10	,
    20	,	20	,	20	,	30	,	30	,	20	,	20	,	20	,
    0	,	0	,	0	,	0	,	0	,	0	,	0	,	0
];
const KNIGHT_TABLE: [i32; 64] = [
    0	,	-10	,	0	,	0	,	0	,	0	,	-10	,	0	,
    0	,	0	,	0	,	5	,	5	,	0	,	0	,	0	,
    0	,	0	,	10	,	10	,	10	,	10	,	0	,	0	,
    0	,	0	,	10	,	20	,	20	,	10	,	5	,	0	,
    5	,	10	,	15	,	20	,	20	,	15	,	10	,	5	,
    5	,	10	,	10	,	20	,	20	,	10	,	10	,	5	,
    0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
    0	,	0	,	0	,	0	,	0	,	0	,	0	,	0
];
const BISHOP_TABLE: [i32; 64] = [
    0	,	0	,	-10	,	0	,	0	,	-10	,	0	,	0	,
    0	,	0	,	0	,	10	,	10	,	0	,	0	,	0	,
    0	,	0	,	10	,	15	,	15	,	10	,	0	,	0	,
    0	,	10	,	15	,	20	,	20	,	15	,	10	,	0	,
    0	,	10	,	15	,	20	,	20	,	15	,	10	,	0	,
    0	,	0	,	10	,	15	,	15	,	10	,	0	,	0	,
    0	,	0	,	0	,	10	,	10	,	0	,	0	,	0	,
    0	,	0	,	0	,	0	,	0	,	0	,	0	,	0
];
const ROOK_TABLE: [i32; 64] = [
    0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
    0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
    0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
    0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
    0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
    0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
    25	,	25	,	25	,	25	,	25	,	25	,	25	,	25	,
    0	,	0	,	5	,	10	,	10	,	5	,	0	,	0
];
const KING_END_TABLE: [i32; 64] = [
    -50	,	-10	,	0	,	0	,	0	,	0	,	-10	,	-50	,
    -10,	0	,	10	,	10	,	10	,	10	,	0	,	-10	,
    0	,	10	,	20	,	20	,	20	,	20	,	10	,	0	,
    0	,	10	,	20	,	40	,	40	,	20	,	10	,	0	,
    0	,	10	,	20	,	40	,	40	,	20	,	10	,	0	,
    0	,	10	,	20	,	20	,	20	,	20	,	10	,	0	,
    -10,	0	,	10	,	10	,	10	,	10	,	0	,	-10	,
    -50	,	-10	,	0	,	0	,	0	,	0	,	-10	,	-50
];
const KING_OPEN_TABLE: [i32; 64] = [
    0	,	5	,	5	,	-10	,	-10	,	0	,	10	,	5	,
    -30	,	-30	,	-30	,	-30	,	-30	,	-30	,	-30	,	-30	,
    -50	,	-50	,	-50	,	-50	,	-50	,	-50	,	-50	,	-50	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70
];

pub fn mirror_board(position: &mut Board) {
    let mut temp_pieces: [u8; 64] = [0; 64];
    let swap_piece: [u8; 13] = [Empty as u8, BlackPawn as u8, BlackKnight as u8, BlackBishop as u8, BlackRook as u8, BlackQueen as u8, BlackKing as u8,
    WhitePawn as u8, WhiteKnight as u8, WhiteBishop as u8, WhiteRook as u8, WhiteQueen as u8, WhiteKing as u8];
    let temp_side = position.side ^ 1;
    let mut temp_castle_permission = 0;
    let mut temp_en_passant = NoSq as u8;

    if position.castle_permission & WhiteKingCastle as u8 != 0 { temp_castle_permission |= BlackKingCastle as u8; }
    if position.castle_permission & WhiteQueenCastle as u8 != 0 { temp_castle_permission |= BlackQueenCastle as u8; }
    if position.castle_permission & BlackKingCastle as u8 != 0 { temp_castle_permission |= WhiteKingCastle as u8; }
    if position.castle_permission & BlackQueenCastle as u8 != 0 { temp_castle_permission |= WhiteQueenCastle as u8; }

    if position.en_passent != NoSq as u8 { temp_en_passant = sq120(MIRROR64[sq64(position.en_passent) as usize] as u8); }

    for square in 0..64 { temp_pieces[square] = position.pieces[sq120(MIRROR64[square] as u8) as usize]; }

    reset_board(position);

    for square in 0..64 {
        let temp = swap_piece[temp_pieces[square] as usize];
        position.pieces[sq120(square as u8) as usize] = temp;
    }

    position.side = temp_side;
    position.castle_permission = temp_castle_permission;
    position.en_passent = temp_en_passant;

    position.position_key = generate_position_key(position);

    update_lists_material(position);

    if DEBUG { check_board(position); }
}

pub fn is_draw(position: &Board) -> bool {
    if position.piece_number[WhiteRook as usize] == 0
        && position.piece_number[BlackRook as usize] == 0
        && position.piece_number[WhiteQueen as usize] == 0
        && position.piece_number[BlackQueen as usize] == 0
    {
        if position.piece_number[BlackBishop as usize] == 0 && position.piece_number[WhiteBishop as usize] == 0 {
            if position.piece_number[WhiteKnight as usize] < 3 && position.piece_number[BlackKnight as usize] < 3
            { return true; }
        } else if position.piece_number[WhiteKnight as usize] == 0 && position.piece_number[BlackKnight as usize] == 0 {
            if (position.piece_number[WhiteBishop as usize] as i32 - position.piece_number[BlackBishop as usize] as i32).abs() < 2
            { return true; }
        } else if (position.piece_number[WhiteKnight as usize] < 3 && position.piece_number[WhiteBishop as usize] == 0)
            || (position.piece_number[WhiteBishop as usize] == 1 && position.piece_number[WhiteKnight as usize] == 0)
        {
            if (position.piece_number[BlackKnight as usize] < 3 && position.piece_number[BlackBishop as usize] == 0)
                || (position.piece_number[BlackBishop as usize] == 1 && position.piece_number[BlackKnight as usize] == 0)
            { return true; }
        }
    } else if position.piece_number[WhiteQueen as usize] == 0 && position.piece_number[BlackQueen as usize] == 0 {
        if position.piece_number[WhiteRook as usize] == 1 && position.piece_number[BlackRook as usize] == 1 {
            if (position.piece_number[WhiteKnight as usize] + position.piece_number[WhiteBishop as usize]) < 2
                && (position.piece_number[BlackKnight as usize] + position.piece_number[BlackBishop as usize]) < 2
            { return true; }
        } else if position.piece_number[WhiteRook as usize] == 1 && position.piece_number[BlackRook as usize] == 0 {
            if position.piece_number[WhiteKnight as usize] + position.piece_number[WhiteBishop as usize] == 0
                && ((position.piece_number[BlackKnight as usize] + position.piece_number[BlackBishop as usize]) == 1
                    || (position.piece_number[BlackKnight as usize] + position.piece_number[BlackBishop as usize]) == 2)
            { return true; }
        } else if position.piece_number[BlackRook as usize] == 1 && position.piece_number[WhiteRook as usize] == 0 {
            if position.piece_number[BlackKnight as usize] + position.piece_number[BlackBishop as usize] == 0
                && ((position.piece_number[WhiteKnight as usize] + position.piece_number[WhiteBishop as usize]) == 1
                    || (position.piece_number[WhiteKnight as usize] + position.piece_number[WhiteBishop as usize]) == 2)
            { return true; }
        }
    }

    false
}

pub fn is_endgame(side_material: i32) -> bool { return side_material <= PIECE_VALUE[WhiteRook as usize] + 2 * PIECE_VALUE[WhiteKnight as usize] + 2 * PIECE_VALUE[WhitePawn as usize] + PIECE_VALUE[WhiteKing as usize] }

pub fn evaluate_position(position: &Board) -> i32 {
    if position.piece_number[WhitePawn as usize] == 0 && position.piece_number[BlackPawn as usize] == 0 && is_draw(position) { return 0 }

    let mut score: i32 = position.material[WHITE] - position.material[BLACK];
    let debug = false;

    let mut piece = WhitePawn as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for WhitePawns".red()); }

        score += PAWN_TABLE[sq64(square) as usize];

        if ISOLATED_MASK[sq64(square) as usize] & position.pawns[WHITE] == 0 {
            // println!("WhitePawn isolated: {}", print_square(square));
            score += PAWN_ISOLATED;
        }

        if WHITE_PASSED_MASK[sq64(square) as usize] & position.pawns[BLACK] == 0 {
            // println!("WhitePawn passed: {}    {}", print_square(square), sq64(square));
            score += PAWN_PASSED[RANKS_BOARD[square as usize] as usize];
        }
    }
    if debug { print!("WhitePawn: {}    ", score); }
    piece = BlackPawn as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for BlackPawn".red()); }

        score -= PAWN_TABLE[MIRROR64[sq64(square) as usize]];

        if ISOLATED_MASK[sq64(square) as usize] & position.pawns[BLACK] == 0 {
            // println!("BlackPawn isolated: {}", print_square(square));
            score -= PAWN_ISOLATED;
        }

        if BLACK_PASSED_MASK[sq64(square) as usize] & position.pawns[WHITE] == 0 {
            // println!("BlackPawn passed: {}", print_square(square));
            score -= PAWN_PASSED[7 - RANKS_BOARD[square as usize] as usize];
        }
    }
    if debug { println!("BlackPawn: {}", score); }

    piece = WhiteKnight as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];
        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for WhiteKnight".red()); }
        score += KNIGHT_TABLE[sq64(square) as usize];
    }
    if debug { print!("WhiteKnight: {}    ", score); }
    piece = BlackKnight as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];
        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for BlackKnight".red()); }
        score -= KNIGHT_TABLE[MIRROR64[sq64(square) as usize]];
    }
    if debug { println!("BlackKnight: {}", score); }

    piece = WhiteBishop as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];
        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for WhiteBishop".red()); }
        score += BISHOP_TABLE[sq64(square) as usize];
    }
    if debug { print!("WhiteBishop: {}    ", score); }
    piece = BlackBishop as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];
        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for BlackBishop".red()); }
        score -= BISHOP_TABLE[MIRROR64[sq64(square) as usize]];
    }
    if debug { println!("BlackBishop: {}", score); }

    piece = WhiteRook as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for WhiteRook".red()); }

        score += ROOK_TABLE[sq64(square) as usize];

        // if no pawns on the file
        if position.pawns[BOTH] & FILE_BB_MASK[FILES_BOARD[square as usize] as usize] == 0 {
            score += ROOK_OPEN_FILE;
        } else if position.pawns[WHITE] & FILE_BB_MASK[FILES_BOARD[square as usize] as usize] == 0 {
            score += ROOK_SEMI_OPEN_FILE;
        }
    }
    if debug { print!("WhiteRook: {}    ", score); }
    piece = BlackRook as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for BlackRook".red()); }

        score -= ROOK_TABLE[MIRROR64[sq64(square) as usize]];

        // if no pawns on the file
        if position.pawns[BOTH] & FILE_BB_MASK[FILES_BOARD[square as usize] as usize] == 0 {
            score -= ROOK_OPEN_FILE;
        } else if position.pawns[BLACK] & FILE_BB_MASK[FILES_BOARD[square as usize] as usize] == 0 {
            score -= ROOK_SEMI_OPEN_FILE;
        }
    }
    if debug { println!("BlackRook: {}", score); }

    piece = WhiteQueen as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for WhiteQueen".red()); }

        // if no pawns on the file
        if position.pawns[BOTH] & FILE_BB_MASK[FILES_BOARD[square as usize] as usize] == 0 {
            score += QUEEN_OPEN_FILE;
        } else if position.pawns[WHITE] & FILE_BB_MASK[FILES_BOARD[square as usize] as usize] == 0 {
            score += QUEEN_SEMI_OPEN_FILE;
        }
    }
    if debug { print!("WhiteQueen: {}    ", score); }
    piece = BlackQueen as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for BlackQueen".red()); }

        // if no pawns on the file
        if position.pawns[BOTH] & FILE_BB_MASK[FILES_BOARD[square as usize] as usize] == 0 {
            score -= QUEEN_OPEN_FILE;
        } else if position.pawns[BLACK] & FILE_BB_MASK[FILES_BOARD[square as usize] as usize] == 0 {
            score -= QUEEN_SEMI_OPEN_FILE;
        }
    }
    if debug { println!("BlackQueen: {}", score); }

    piece = WhiteKing as usize;
    let mut square = position.piece_list[piece][0];
    if is_endgame(position.material[BLACK]) {
        score += KING_END_TABLE[sq64(square) as usize];
    } else {
        score += KING_OPEN_TABLE[sq64(square) as usize];
    }
    if debug { print!("WhiteKing: {}    ", score); }

    piece = BlackKing as usize;
    square = position.piece_list[piece][0];
    if is_endgame(position.material[WHITE]) {
        score -= KING_END_TABLE[MIRROR64[sq64(square) as usize]];
    } else {
        score -= KING_OPEN_TABLE[MIRROR64[sq64(square) as usize]];
    }
    if debug { println!("BlackKing: {}", score); }

    if position.piece_number[WhiteBishop as usize] >= 2 { score += BISHOP_PAIR; }
    if position.piece_number[BlackBishop as usize] >= 2 { score -= BISHOP_PAIR; }

    if debug { println!("-----------"); }
    if position.side == WHITE as u8 { return score }
    -score
}
