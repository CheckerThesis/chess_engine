use colored::Colorize;

use crate::{defs::{sq64, Board, Pieces::*, BLACK, DEBUG, SQ120_TO_SQ64, WHITE}, validate::square_on_board};

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
const KING_E_TABLE: [i32; 64] = [
    -50	,	-10	,	0	,	0	,	0	,	0	,	-10	,	-50	,
    -10,	0	,	10	,	10	,	10	,	10	,	0	,	-10	,
    0	,	10	,	20	,	20	,	20	,	20	,	10	,	0	,
    0	,	10	,	20	,	40	,	40	,	20	,	10	,	0	,
    0	,	10	,	20	,	40	,	40	,	20	,	10	,	0	,
    0	,	10	,	20	,	20	,	20	,	20	,	10	,	0	,
    -10,	0	,	10	,	10	,	10	,	10	,	0	,	-10	,
    -50	,	-10	,	0	,	0	,	0	,	0	,	-10	,	-50
];
const KING_O_TABLE: [i32; 64] = [
    0	,	5	,	5	,	-10	,	-10	,	0	,	10	,	5	,
    -30	,	-30	,	-30	,	-30	,	-30	,	-30	,	-30	,	-30	,
    -50	,	-50	,	-50	,	-50	,	-50	,	-50	,	-50	,	-50	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
    -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70
];
const MIRROR64: [usize; 64] = [
    56	,	57	,	58	,	59	,	60	,	61	,	62	,	63	,
    48	,	49	,	50	,	51	,	52	,	53	,	54	,	55	,
    40	,	41	,	42	,	43	,	44	,	45	,	46	,	47	,
    32	,	33	,	34	,	35	,	36	,	37	,	38	,	39	,
    24	,	25	,	26	,	27	,	28	,	29	,	30	,	31	,
    16	,	17	,	18	,	19	,	20	,	21	,	22	,	23	,
    8	,	9	,	10	,	11	,	12	,	13	,	14	,	15	,
    0	,	1	,	2	,	3	,	4	,	5	,	6	,	7
];

pub fn evaluate_position(position: &Board) -> i32 {
    let mut score: i32 = position.material[WHITE] - position.material[BLACK];

    let mut piece = WhitePawn as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for WhitePawns".red()); }

        score += PAWN_TABLE[sq64(square) as usize];
    }
    // println!("WhitePawn: {}", score);
    piece = BlackPawn as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for BlackPawn".red()); }

        score -= PAWN_TABLE[MIRROR64[sq64(square) as usize]];
    }
    // println!("BlackPawn: {}", score);

    piece = WhiteKnight as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for WhiteKnight".red()); }

        score += KNIGHT_TABLE[sq64(square) as usize];
    }
    // println!("WhiteKnight: {}", score);
    piece = BlackKnight as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for BlackKnight".red()); }

        score -= KNIGHT_TABLE[MIRROR64[sq64(square) as usize]];
    }
    // println!("BlackKnight: {}", score);
    piece = WhiteBishop as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for WhiteBishop".red()); }

        score += BISHOP_TABLE[sq64(square) as usize];
    }
    piece = BlackBishop as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for BlackBishop".red()); }

        score -= BISHOP_TABLE[MIRROR64[sq64(square) as usize]];
    }

    piece = WhiteRook as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for WhiteRook".red()); }

        score += ROOK_TABLE[sq64(square) as usize];
    }
    piece = BlackRook as usize;
    for piece_number in 0..position.piece_number[piece] {
        let square = position.piece_list[piece][piece_number as usize];

        if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "evaluate: [square] is off board for BlackRook".red()); }

        score -= ROOK_TABLE[MIRROR64[sq64(square) as usize]];
    }

    if position.side == WHITE as u8 {
        return score
    }

    -score
}
