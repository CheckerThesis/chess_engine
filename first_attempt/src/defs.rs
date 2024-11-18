use crate::bitboards::{count_bits, pop_bit};
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};

pub const NAME: &str = "Unknown";
pub const BOARD_SQUARE_NUMBER: usize = 120;
pub const MAX_GAME_MOVES: usize = 2048;
pub const START_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[repr(u64)]
#[derive(Copy, Clone)]
pub enum Pieces {
    Empty,
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}
#[repr(usize)]
pub enum Files {
    FileA,
    FileB,
    FileC,
    FileD,
    FileE,
    FileF,
    FileG,
    FileH,
    FileNone,
}
#[repr(usize)]
pub enum Ranks {
    Rank1,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8,
    RankNone,
}
#[repr(u64)]
pub enum Sides {
    White,
    Black,
    Both,
}
#[repr(u64)]
pub enum Squares {
    A1 = 21, B1, C1, D1, E1, F1, G1, H1,
    A2 = 31, B2, C2, D2, E2, F2, G2, H2,
    A3 = 41, B3, C3, D3, E3, F3, G3, H3,
    A4 = 51, B4, C4, D4, E4, F4, G4, H4,
    A5 = 61, B5, C5, D5, E5, F5, G5, H5,
    A6 = 71, B6, C6, D6, E6, F6, G6, H6,
    A7 = 81, B7, C7, D7, E7, F7, G7, H7,
    A8 = 91, B8, C8, D8, E8, F8, G8, H8, NoSq, OffBoard
}
#[repr(u64)]
pub enum TrueFalse{
    FALSE,
    TRUE,
}

pub enum Castling {
    WhiteKingCastle = 1,
    WhiteQueenCastle = 2,
    BlackKingCastle = 4,
    BlackQueenCastle = 8,
}

#[derive(Copy, Clone)]
pub struct UndoState {
    pub the_move: u64,
    pub castle_permission: u64,
    pub en_passent: u64,
    pub fifty_move: u64,
    pub position_key: u64,
}

impl Default for UndoState {
    fn default() -> Self {
        UndoState {
            the_move: 0,
            castle_permission: 0,
            en_passent: 0,
            fifty_move: 0,
            position_key: 0,
        }
    }
}

pub struct BoardState {
    pub pieces: [u64; BOARD_SQUARE_NUMBER],
    pub pawns: [u64; 3],

    pub king_square: [u64; 2],

    pub side: u64,
    pub en_passent: u64,
    pub fifty_move: u64,

    pub ply: u64,
    pub history_ply: u64,

    pub castle_permission: u64,

    pub position_key: u64,

    pub piece_number: [u64; 13],
    pub big_piece: [u64; 3],
    pub major_piece: [u64; 3],
    pub minor_piece: [u64; 3],

    pub history: [UndoState; MAX_GAME_MOVES],

    pub piece_list: [[i32; 10]; 13],
}

impl Default for BoardState {
    fn default() -> Self {
        BoardState {
            pieces: [Squares::OffBoard as u64; BOARD_SQUARE_NUMBER],
            pawns: [0; 3],
            king_square: [Squares::NoSq as u64; 2],
            side: Sides::Both as u64,
            en_passent: Squares::NoSq as u64,
            fifty_move: 0,
            ply: 0,
            history_ply: 0,
            castle_permission: 0,
            position_key: 0,
            piece_number: [0; 13],
            big_piece: [0; 3],
            major_piece: [0; 3],
            minor_piece: [0; 3],
            history: [UndoState::default(); MAX_GAME_MOVES],
            piece_list: [[0; 10]; 13],
        }
    }
}

// Macros
#[macro_export]
macro_rules! FR2SQ {
    ($file:expr, $rank:expr) => {
        21 + $file + ($rank * 10)
    };
}

#[macro_export]
macro_rules! SQ64 {
  ($sq120:expr) => {
    crate::defs::SQ120_TO_SQ64[$sq120 as usize]
  };
}

#[macro_export]
macro_rules! SQ120 {
  ($sq64:expr) => {
    crate::defs::SQ64_TO_SQ120[$sq64 as usize]
  };
}

#[macro_export]
macro_rules! POP {
    ($bit:expr) => {
        pop_bit($bit)
    };
}

#[macro_export]
macro_rules! COUNT {
    ($bit:expr) => {
        count_bits($bit)
    };
}

#[macro_export]
macro_rules! CLEARBIT {
    ($bitboard:expr, $square:expr) => {
        use crate::defs::CLEAR_MASK;
        unsafe{$bitboard &= CLEAR_MASK[($square)]}
    };
}

#[macro_export]
macro_rules! SETBIT {
    ($bitboard:expr, $square:expr) => {
        use crate::defs::SET_MASK;
        unsafe{$bitboard |= SET_MASK[($square)]}
    };
}

// Globals
lazy_static! {
    pub static ref SQ120_TO_SQ64: [usize; BOARD_SQUARE_NUMBER] = {
        let mut sq120_to_sq64 = [65; BOARD_SQUARE_NUMBER];
        let mut square64: usize = 0;

        for rank in Ranks::Rank1 as u64..=Ranks::Rank8 as u64 {
            for file in Files::FileA as u64..=Files::FileH as u64 {
                let square = FR2SQ!(file, rank) as usize;
                sq120_to_sq64[square] = square64;
                square64 += 1;
            }
        }
        sq120_to_sq64
    };
    pub static ref SQ64_TO_SQ120: [usize; 64] = {
        let mut sq64_to_sq120 = [120; 64];
        let mut square64: usize = 0;

        for rank in Ranks::Rank1 as u64..=Ranks::Rank8 as u64 {
            for file in Files::FileA as u64..=Files::FileH as u64 {
                let square = FR2SQ!(file, rank) as usize;
                sq64_to_sq120[square64] = square;
                square64 += 1;
            }
        }
        sq64_to_sq120
    };
    pub static ref SET_MASK: [u64; 64] = {
        let mut mask = [0u64; 64];
        for i in 0..64 {
            mask[i] = 1 << i;
        }
        mask
    };
    pub static ref CLEAR_MASK: [u64; 64] = {
        let mut mask = [0u64; 64];
        for i in 0..64 {
            mask[i] = !(1 << i);
        }
        mask
    };

    pub static ref PIECE_KEYS: [[u64; 13]; 120] = {
        let mut rng = thread_rng();
        [[rng.gen(); 13]; 120]
    };
    pub static ref SIDE_KEY: u64 = {
        let mut rng = thread_rng();
        rng.gen()
    };
    pub static ref CASTLE_KEYS: [u64; 16] = {
        let mut rng = thread_rng();
        [rng.gen(); 16]
    };
}
