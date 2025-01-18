use std::{sync::{LazyLock, Mutex}, time::Instant};

use colored::Colorize;
use rand::{thread_rng, Rng};

pub const BOARD_SQUARE_NUMBER: usize = 120;
pub const MAX_GAME_MOVES: usize = 2048;
pub const MAX_POSITION_MOVES: usize = 256;

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
pub const BOTH: usize = 2;

pub const DEBUG: bool = false;

pub const NO_MOVE: u64 = 0;

pub const MAX_DEPTH: usize = 64;
pub const INF_BOUND: i32 = 30000;
pub const IS_MATE: i32 = INF_BOUND - MAX_DEPTH as i32;

// .lock() is specific to mutex, careful with .unwrap() consider using something more robust
pub static ENGINE_OPTIONS: LazyLock<Mutex<EngineOptions>> = LazyLock::new(|| Mutex::new(EngineOptions::default()));
pub static HASH_TABLE: LazyLock<Mutex<HashTable>> = LazyLock::new(|| Mutex::new(HashTable::default()));

/*
A8 B8 C8 D8 E8 F8 G8 H8
A7 B7 C7 D7 E7 F7 G7 H7
A6 B6 C6 D6 E6 F6 G6 H6
A5 B5 C5 D5 E5 F5 G5 H5
A4 B4 C4 D4 E4 F4 G4 H4
A3 B3 C3 D3 E3 F3 G3 H3
A2 B2 C2 D2 E2 F2 G2 H2
A1 B1 C1 D1 E1 F1 G1 H1
*/

#[repr(u8)]
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
#[repr(u8)]
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
#[repr(u8)]
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

#[repr(u8)]
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

pub enum Castling {
    WhiteKingCastle = 1,
    WhiteQueenCastle = 2,
    BlackKingCastle = 4,
    BlackQueenCastle = 8,
}

#[derive(Copy, Clone, Default)]
pub struct Undo {
    pub the_move: u64,
    pub castle_permission: u8,
    pub en_passent: u8,
    pub fifty_move: u8,
    pub position_key: u64,
}

#[derive(Copy, Clone, Default)]
pub struct Move {
    pub el_move: u64,
    pub score: u32,
}

pub struct MoveList {
    pub moves: [Move; MAX_POSITION_MOVES],
    pub count: usize,
}
impl Default for MoveList {
    fn default() -> Self {
        MoveList {
            moves: [Move::default(); MAX_POSITION_MOVES],
            count: 0,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct HashEntry {
    pub position_key: u64,
    pub the_move: u64,
    pub score: i32,
    pub depth: i32,
    pub flags: u8,
    pub age: u8,
}

pub struct HashTable {
    pub pv_table: Vec<HashEntry>,
    pub new_write: u64,
    pub over_write: u64,
    pub hit: u64,
    pub cut: u64,
    pub current_age: u8,
}
impl Default for HashTable {
    fn default() -> Self {
        HashTable {
            pv_table: vec![HashEntry::default(); 131070],
            new_write: 0,
            over_write: 0,
            hit: 0,
            cut: 0,
            current_age: 0,
        }
    }
}

#[repr(u8)]
pub enum HashFlag {
    HashFlagNone,
    HashFlagAlpha,
    HashFlagBeta,
    HashFlagExact,
}

pub struct Board {
    pub pieces: [u8; BOARD_SQUARE_NUMBER],
    // pawn bitboard
    pub pawns: [u64; 3],

    pub king_square: [u8; 2],

    pub side: u8,
    pub en_passent: u8,
    pub fifty_move: u8,

    pub ply: u8,
    pub history_ply: usize,

    pub castle_permission: u8,

    pub position_key: u64,

    // number of pieces for each piece type
    pub piece_number: [u8; 13],
    // anything that's not a pawn
    pub big_piece: [u8; 2],
    // rooks and queens
    pub major_piece: [u8; 2],
    // bishops and knights
    pub minor_piece: [u8; 2],
    // value of each side
    pub material: [i32; 2],

    pub history: [Undo; MAX_GAME_MOVES],

    // piece_list [WhiteKnight][0] = E1 | for looping through only pieces for move generation
    pub piece_list: [[u8; 10]; 13],

    // principal variation is the best sequence of moves,
    // ie. the best according to the engine
    // ie. expected moves played
    pub pv_array: [u64; MAX_DEPTH],

    // for move ordering, rough way to record non-capture moves that are good enough to cause beta cut-off or good alpha
    pub search_history: [[u32; BOARD_SQUARE_NUMBER]; 13], // stores when a score has beaten alpha
    pub search_killers: [[u64; MAX_DEPTH]; 2], // stores when a score has beaten beta but is not a capture
}
impl Default for Board {
    fn default() -> Self {
        Board {
            pieces: [Squares::OffBoard as u8; BOARD_SQUARE_NUMBER],
            pawns: [0; 3],
            king_square: [Squares::NoSq as u8; 2],
            side: BOTH as u8,
            en_passent: Squares::NoSq as u8,
            fifty_move: 0,
            ply: 0,
            history_ply: 0,
            castle_permission: 0,
            position_key: 0,
            piece_number: [0; 13],
            big_piece: [0; 2],
            major_piece: [0; 2],
            minor_piece: [0; 2],
            material: [0; 2],
            history: [Undo::default(); MAX_GAME_MOVES],
            piece_list: [[0; 10]; 13],
            pv_array: [0; MAX_DEPTH],
            search_history: [[0; BOARD_SQUARE_NUMBER]; 13],
            search_killers: [[0; MAX_DEPTH]; 2],
        }
    }
}

#[derive(Copy, Clone)]
pub struct SearchInfo {
    pub start_time: Instant,
    pub stop_time: Instant,
    pub depth: i32,
    pub time_set: bool,

    pub moves_to_go: u8,

    pub nodes: u64,

    pub quit: bool,
    pub stopped: bool,

    // gives an idea of how good move ordering is, should be greater than 90%
    pub fail_high: f32, // number of times alpha > beta on the first move
    pub fail_high_first: f32, // number of times alpha > beta total
    pub null_cut: u32,
}
impl Default for SearchInfo {
    fn default() -> Self {
        SearchInfo {
            start_time: Instant::now(),
            stop_time: Instant::now(),
            depth: 0,
            time_set: false,
            moves_to_go: 0,
            nodes: 0,
            quit: false,
            stopped: false,
            fail_high: 0.0,
            fail_high_first: 0.0,
            null_cut: 0,
        }
    }
}

pub struct EngineOptions {
    pub book: bool,
}
impl Default for EngineOptions {
    fn default() -> Self {
        EngineOptions {
            book: false,
        }
    }
}

// small board to big board
pub fn fr2sq(file: u8, rank: u8) -> u8 { 21 + file + rank * 10 }
pub fn sq64(sq120: u8) -> u8 { SQ120_TO_SQ64[sq120 as usize] }
pub fn sq120(sq64: u8) -> u8 { SQ64_TO_SQ120[sq64 as usize] }

pub fn clear_bit(bitboard: &mut u64, square: u8) { *bitboard &= CLEAR_MASK[sq64(square) as usize]; }
pub fn set_bit(bitboard: &mut u64, square: u8) { *bitboard |= SET_MASK[sq64(square) as usize]; }

/*
One block of bits = F
1111 = F = 15
1000 = 8
1000 1111 = 8F

0001 = 1
0010 = 2
0100 = 4

Lowest square a piece will be on is 21, highest is 98
0000 0000 0000 0000 0000 0111 1111 -> From -> 0x7F
0000 0000 0000 0011 1111 1000 0000 -> To >> 7 0x7F (shift right by 7 bits)
0000 0000 0011 1100 0000 0000 0000 -> Captured (go up to 12) >> 14 0xF (F because it is 4 digits)
0000 0000 0100 0000 0000 0000 0000 -> En-passent capture (piece to promoted to) -> 0x40000
0000 0000 1000 0000 0000 0000 0000 -> Pawn start -> 0x80000
0000 1111 0000 0000 0000 0000 0000 -> Promoted piece >> 20 0xF
0001 0000 0000 0000 0000 0000 0000 -> Castle -> 0x1000000
So essentially, each hexidecimal digit represents each 4 digits
            4    8    9    7    F  -> 4897F
0000 0000 0100 1000 1001 0111 1111
*/
pub fn print_binary(the_move: u64) {
    println!("As binary: ");

    for i in (0..=27).rev() {
        if 1 << i & the_move == 0 {
            print!("0");
        } else {
            print!("1");
        }
        if i % 4 == 0 { print!(" "); }
    }
    println!();
}

// the_move >> x , x is how much the shift is
// the_move >> x & y, y is the amount of digits (7 for 0x3F)
pub fn from_square(the_move: u64) -> u64 { the_move & 0x7F }
pub fn to_square(the_move: u64) -> u64 { the_move >> 7 & 0x7F }
pub fn captured(the_move: u64) -> u64 { the_move >> 14 & 0xF }
pub fn promoted(the_move: u64) -> u64 { the_move >> 20 & 0xF }

// beginning number is hex to decimal, 0's is empty 4-digits
pub const MOVE_FLAG_EN_PASSENT: u64 = 0x40000; // 0000 0000 0100 0000 0000 0000 0000
pub const MOVE_FLAG_PAWN_START: u64 = 0x80000; // 0000 0000 1000 0000 0000 0000 0000
pub const MOVE_FLAG_CASTLE: u64 = 0x1000000;   // 0001 0000 0000 0000 0000 0000 0000
pub const MOVE_FLAG_CAPTURE: u64 = 0x7C000;    // 0000 0000 0011 1100 0000 0000 0000
pub const MOVE_FLAG_PROMOTE: u64 = 0xF00000;   // 0000 1111 0000 0000 0000 0000 0000

/*
    println!("SQ120-SQ64");
    for i in 0..BOARD_SQUARE_NUMBER {
        if i % 10 == 0 {
            println!();
        }
        print!("{:>4}", SQ120_TO_SQ64[i]);
    }
    println!("\n");
    println!("SQ64-SQ120");
    for i in 0..64 {
        if i % 8 == 0 {
            println!();
        }
        print!("{:>4}", SQ64_TO_SQ120[i])
    }
    println!("\n");
    */
pub static SQ120_TO_SQ64: LazyLock<[u8; BOARD_SQUARE_NUMBER]> = LazyLock::new(|| {
    let mut sq120_to_sq64 = [65; BOARD_SQUARE_NUMBER];
        let mut square64: u8 = 0;

        for rank in Ranks::Rank1 as u8..=Ranks::Rank8 as u8 {
            for file in Files::FileA as u8..=Files::FileH as u8 {
                let square = fr2sq(file, rank);
                sq120_to_sq64[square as usize] = square64;
                square64 += 1;
            }
        }
        sq120_to_sq64
});
pub static SQ64_TO_SQ120: LazyLock<[u8; 64]> = LazyLock::new(|| {
    let mut sq64_to_sq120 = [120; 64];
    let mut square64: u8 = 0;

    for rank in Ranks::Rank1 as u8..=Ranks::Rank8 as u8 {
        for file in Files::FileA as u8..=Files::FileH as u8 {
            let square = fr2sq(file, rank);
            sq64_to_sq120[square64 as usize] = square;
            square64 += 1;
        }
    }
    sq64_to_sq120
});

// array of integers, each is 8x8 u64 all 0 except for one set to 1
pub static SET_MASK: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut mask = [0; 64];
    for i in 0..64 {
        mask[i] = 1 << i;
    }
    mask
});
pub static CLEAR_MASK: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut mask = [0; 64];
    for i in 0..64 {
        mask[i] = !(1 << i);
    }
    mask
});

pub static PIECE_KEYS: LazyLock<[[u64; 120]; 13]> = LazyLock::new(|| {
    let mut rng = thread_rng();
    // 0u64 is a u64 integer with 0 as its value
    let mut keys = [[0u64; 120]; 13];

    for piece in 0..13 {
        for square in 0..120 {
            keys[piece][square] = rng.gen();
        }
    }
    keys
});
pub static SIDE_KEY: LazyLock<u64> = LazyLock::new(|| {
    let mut rng = thread_rng();
    rng.gen()
});
pub static CASTLE_KEYS: LazyLock<[u64; 16]> = LazyLock::new(|| {
    let mut rng = thread_rng();
    let mut keys = [0u64; 16];
    for i in 0..16 {
        keys[i] = rng.gen();
    }
    keys
});

/*
println!("Files board");
for i in 0..BOARD_SQUARE_NUMBER {
    if i % 10 == 0 && i != 0 {
        println!();
    }
    print!("{:<4}", FILES_BOARD[i]);
}
println!("\n\nRanks board");
for i in 0..BOARD_SQUARE_NUMBER {
    if i % 10 == 0 && i != 0 {
        println!();
    }
    print!("{:<4}", RANKS_BOARD[i]);
}
*/
pub static FILES_BOARD: LazyLock<[u8; BOARD_SQUARE_NUMBER]> = LazyLock::new(|| {
    let mut files_board: [u8; BOARD_SQUARE_NUMBER] = [BOARD_SQUARE_NUMBER as u8; BOARD_SQUARE_NUMBER];

    for i in 0..BOARD_SQUARE_NUMBER {
        files_board[i] = Squares::OffBoard as u8;
    }

    for rank in Ranks::Rank1 as u8..=Ranks::Rank8 as u8 {
        for file in Files::FileA as u8..=Files::FileH as u8 {
            let square = fr2sq(file, rank) as usize;
            files_board[square] = file;
        }
    }

    files_board
});
pub static RANKS_BOARD: LazyLock<[u8; BOARD_SQUARE_NUMBER]> = LazyLock::new(|| {
    let mut ranks_board: [u8; BOARD_SQUARE_NUMBER] = [BOARD_SQUARE_NUMBER as u8; BOARD_SQUARE_NUMBER];

    for i in 0..BOARD_SQUARE_NUMBER {
        ranks_board[i] = Squares::OffBoard as u8;
    }

    for rank in Ranks::Rank1 as u8..=Ranks::Rank8 as u8 {
        for file in Files::FileA as u8..=Files::FileH as u8 {
            let square = fr2sq(file, rank) as usize;
            ranks_board[square] = rank;
        }
    }

    ranks_board
});

/*
When ordering moves, you search for them in this order:
1. PV Move
2. Capture -> most valuable victim, least valuable attacker
3. Killers (beta cutoffs)
4. History score

    P takes Q
    N takes Q
    ..
    P takes R
    N takes R
    ..

    vic Q -> 500, P(505), N(504)
    vic R -> 400
*/
pub static MVV_LVA_SCORES: LazyLock<[[u32; 13]; 13]> = LazyLock::new(|| {
    let victim_score: [u32; 13] = [0, 100, 200, 300, 400, 500, 600, 100, 200 ,300 ,400, 500, 600];
    let mut mvv_lva_scores: [[u32; 13]; 13] = [[0; 13]; 13];

    for attacker in Pieces::WhitePawn as usize..Pieces::BlackKing as usize {
        for victim in Pieces::WhitePawn as usize..Pieces::BlackKing as usize {
            mvv_lva_scores[victim][attacker] = victim_score[victim] + 6 - (victim_score[attacker] / 100);
        }
    }

    mvv_lva_scores
});

pub static FILE_BB_MASK: LazyLock<[u64; 8]> = LazyLock::new(|| {
    let mut file_bb_mask: [u64; 8] = [0; 8];

    for rank in (Ranks::Rank1 as u64..=Ranks::Rank8 as u64).rev() {
        for file in Files::FileA as u64..=Files::FileH as u64 {
            let square = rank * 8 + file;
            file_bb_mask[file as usize] |= 1 << square;
        }
    }

    file_bb_mask
});
pub static RANK_BB_MASK: LazyLock<[u64; 8]> = LazyLock::new(|| {
    let mut rank_bb_mask: [u64; 8] = [0; 8];

    for rank in (Ranks::Rank1 as u64..=Ranks::Rank8 as u64).rev() {
        for file in Files::FileA as u64..=Files::FileH as u64 {
            let square = rank * 8 + file;
            rank_bb_mask[rank as usize] |= 1 << square;
        }
    }

    rank_bb_mask
});

/*
when & if it ends up 0, the pawn will be passed
0 0 0 1 1 1 0 0
0 0 0 1 1 1 0 0
0 0 0 1 1 1 0 0
0 0 0 1 1 1 0 0
0 0 0 1 1 1 0 0
0 0 0 0 x 0 0 0
0 0 0 0 0 0 0 0
0 0 0 0 0 0 0 0

for i in 0..64 {
    print_bitboard(ISOLATED_MASK[i]);
}
*/
pub static ISOLATED_MASK: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut masks = [0u64; 64];
    for sq in 0..64 {
        let file = FILES_BOARD[sq120(sq) as usize];
        if file > Files::FileA as u8 {
            masks[sq as usize] |= FILE_BB_MASK[(file - 1) as usize];
        }
        if file < Files::FileH as u8 {
            masks[sq as usize] |= FILE_BB_MASK[(file + 1) as usize];
        }
    }
    masks
});
// White passed pawn masks
pub static WHITE_PASSED_MASK: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut masks = [0u64; 64];
    for sq in 0..64 {
        let file = FILES_BOARD[sq120(sq) as usize];

        // Forward
        let mut tsq = sq + 8;
        while tsq < 64 {
            masks[sq as usize] |= 1u64 << tsq;
            tsq += 8;
        }

        // Forward-left
        if file > Files::FileA as u8 {
            let mut tsq = sq + 7;
            while tsq < 64 {
                masks[sq as usize] |= 1u64 << tsq;
                tsq += 8;
            }
        }

        // Forward-right
        if file < Files::FileH as u8 {
            let mut tsq = sq + 9;
            while tsq < 64 {
                masks[sq as usize] |= 1u64 << tsq;
                tsq += 8;
            }
        }
    }
    masks
});
// Black passed pawn masks
pub static BLACK_PASSED_MASK: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut masks = [0u64; 64];
    for sq in 0..64 {
        let file = FILES_BOARD[sq120(sq) as usize];

        // Backward
        let mut tsq = sq as i32 - 8;
        while tsq >= 0 {
            masks[sq as usize] |= 1u64 << tsq;
            tsq -= 8;
        }

        // Backward-left
        if file > Files::FileA as u8 {
            let mut tsq = sq as i32 - 9;
            while tsq >= 0 {
                masks[sq as usize] |= 1u64 << tsq;
                tsq -= 8;
            }
        }

        // Backward-right
        if file < Files::FileH as u8 {
            let mut tsq = sq as i32 - 7;
            while tsq >= 0 {
                masks[sq as usize] |= 1u64 << tsq;
                tsq -= 8;
            }
        }
    }
    masks
});
