use crate::defs::*;

pub const PIECE_CHAR: &str = ".♙♘♗♖♕♔♟♞♝♜♛♚";
pub const SIDE_CHAR: &str = "wb-";
pub const RANK_CHAR: &str = "12345678";
pub const FILE_CHAR: &str = "abcdefgh";
                                // Empty, WPawn, WKnight, WBishop, WRook, WQueen, WKing, BPawn, BKnight, BBishop, BRook, BQueen,BKing
pub const PIECE_BIG: [bool; 13] = [false, false, true, true, true, true, true, false, true, true, true, true, true];
pub const PIECE_MAJOR: [bool; 13] = [false, false, false, false, true, true, true, false, false, false, true, true, true];
pub const PIECE_MINOR: [bool; 13] = [false, false, true, true, false, false, false, false, true, true, false, false, false];
pub const PIECE_VALUE: [i32; 13] = [0, 100, 325, 325, 550, 1000, 50000, 100, 325, 325, 550, 1000, 50000];
pub const PIECE_COLOR: [u8; 13] = [BOTH as u8, WHITE as u8, WHITE as u8, WHITE as u8, WHITE as u8, WHITE as u8, WHITE as u8, BLACK as u8, BLACK as u8, BLACK as u8, BLACK as u8, BLACK as u8, BLACK as u8];

pub const IS_PAWN: [bool; 13] = [false, true, false, false, false, false, false, true, false, false, false, false, false];
pub const IS_KNIGHT: [bool; 13] = [false, false, true, false, false, false, false, false, true, false, false, false, false];
pub const IS_KING: [bool; 13] = [false, false, false, false, false, false, true, false, false, false, false, false, true];
pub const IS_ROOK_QUEEN: [bool; 13] = [false, false, false, false, true, true, false, false, false, false, true, true, false];
pub const IS_BISHOP_QUEEN: [bool; 13] = [false, false, false, true, false, true, false, false, false, true, false, true, false];

pub const MIRROR64: [usize; 64] = [
    56	,	57	,	58	,	59	,	60	,	61	,	62	,	63	,
    48	,	49	,	50	,	51	,	52	,	53	,	54	,	55	,
    40	,	41	,	42	,	43	,	44	,	45	,	46	,	47	,
    32	,	33	,	34	,	35	,	36	,	37	,	38	,	39	,
    24	,	25	,	26	,	27	,	28	,	29	,	30	,	31	,
    16	,	17	,	18	,	19	,	20	,	21	,	22	,	23	,
    8	,	9	,	10	,	11	,	12	,	13	,	14	,	15	,
    0	,	1	,	2	,	3	,	4	,	5	,	6	,	7
];
