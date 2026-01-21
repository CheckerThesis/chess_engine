use std::{sync::atomic::{AtomicU64, Ordering}, time::Instant};

use crate::{board::{Board, print_bitboard}, defs::{Color, Piece, PieceType}, movegen::{bitboards::{BLACK_PAWN_ATTACKS, KING_RAYS, KNIGHT_RAYS, WHITE_PAWN_ATTACKS}, magic::{get_bishop_attacks, get_rook_attacks}}, squares::squares::A1};

pub const PIECE_VALUE: [i32; 8] = [
    0,     // none
    100,   // pawn
    325,   // knight
    325,   // bishop
    550,   // rook
    1000,  // queen
    32767, // king,
    1300,  // dragon
];

pub const MIRROR64: [usize; 64] = [
    56, 57, 58, 59, 60, 61, 62, 63,
    48, 49, 50, 51, 52, 53, 54, 55,
    40, 41, 42, 43, 44, 45, 46, 47,
    32, 33, 34, 35, 36, 37, 38, 39,
    24, 25, 26, 27, 28, 29, 30, 31,
    16, 17, 18, 19, 20, 21, 22, 23,
    8,  9,  10, 11, 12, 13, 14, 15,
    0,  1,  2,  3,  4,  5,  6,  7
];
pub const IDENTITY64: [usize; 64] = [
    0, 1, 2, 3, 4, 5, 6, 7,
    8, 9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 26, 27, 28, 29, 30, 31,
    32, 33, 34, 35, 36, 37, 38, 39,
    40, 41, 42, 43, 44, 45, 46, 47,
    48, 49, 50, 51, 52, 53, 54, 55,
    56, 57, 58, 59, 60, 61, 62, 63
];

pub const OPENING_PIECE_SQUARE: [[i32; 64]; 8] = [
    [0; 64], // none
    [ // pawn
        0,  0,  0,  0,  0,  0,  0,  0,
        10, 10, 0,  -10, -10, 0,  10, 10,
        5,  0,  0,  5,  5,  0,  0,  5,
        0,  0,  10, 20, 20, 10, 0,  0,
        5,  5,  5,  10, 10, 5,  5,  5,
        10, 10, 10, 20, 20, 10, 10, 10,
        20, 20, 20, 30, 30, 20, 20, 20,
        0,  0,  0,  0,  0,  0,  0,  0
    ],
    [ // knight
        0,  -10, 0,  0,  0,  0,  -10, 0,
        0,  0,  0,  5,  5,  0,  0,  0,
        0,  0,  10, 10, 10, 10, 0,  0,
        0,  0,  10, 20, 20, 10, 5,  0,
        5,  10, 15, 20, 20, 15, 10, 5,
        5,  10, 10, 20, 20, 10, 10, 5,
        0,  0,  5,  10, 10, 5,  0,  0,
        0,  0,  0,  0,  0,  0,  0,  0
    ],
    [ // bishop
        0,  0,  -10, 0,  0,  -10, 0,  0,
        0,  0,  0,  10, 10, 0,  0,  0,
        0,  0,  10, 15, 15, 10, 0,  0,
        0,  10, 15, 20, 20, 15, 10, 0,
        0,  10, 15, 20, 20, 15, 10, 0,
        0,  0,  10, 15, 15, 10, 0,  0,
        0,  0,  0,  10, 10, 0,  0,  0,
        0,  0,  0,  0,  0,  0,  0,  0
    ],
    [ // rook
        0,  0,  5,  10, 10, 5,  0,  0,
        0,  0,  5,  10, 10, 5,  0,  0,
        0,  0,  5,  10, 10, 5,  0,  0,
        0,  0,  5,  10, 10, 5,  0,  0,
        0,  0,  5,  10, 10, 5,  0,  0,
        0,  0,  5,  10, 10, 5,  0,  0,
        25, 25, 25, 25, 25, 25, 25, 25,
        0,  0,  5,  10, 10, 5,  0,  0
    ],
    [ // queen
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ],
    [ // king
        0,  5,  5,  -10, -10, 0,  10, 5,
        -30, -30, -30, -30, -30, -30, -30, -30,
        -50, -50, -50, -50, -50, -50, -50, -50,
        -70, -70, -70, -70, -70, -70, -70, -70,
        -70, -70, -70, -70, -70, -70, -70, -70,
        -70, -70, -70, -70, -70, -70, -70, -70,
        -70, -70, -70, -70, -70, -70, -70, -70,
        -70, -70, -70, -70, -70, -70, -70, -70
    ],
    [ // dragon
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ],
];

pub const ENDING_PIECE_SQUARE: [[i32; 64]; 8] = [
    [0; 64], // none
    [ // pawn
        0,  0,  0,  0,  0,  0,  0,  0,
        0,  0,  0,  0,  0,  0,  0,  0, 
        5,  5,  5,  5,  5,  5,  5,  5,
        10, 10, 10, 20, 20, 10, 10, 10, 
        20, 20, 20, 30, 30, 20, 20, 20,
        40, 40, 40, 50, 50, 40, 40, 40,
        80, 80, 80, 90, 90, 80, 80, 80,
        0,  0,  0,  0,  0,  0,  0,  0  
    ],
    // [0; 64], // pawn (rely on passed pawn logic)
    [ // knight
        -10, -5,  -5,  -5,  -5,  -5,  -5, -10,
        -5,  0,   0,   0,   0,   0,   0,  -5,
        -5,  0,   10,  10,  10,  10,  0,  -5,
        -5,  0,   15,  20,  20,  15,  0,  -5,
        -5,  0,   15,  20,  20,  15,  0,  -5,
        -5,  0,   10,  10,  10,  10,  0,  -5,
        -5,  0,   0,   0,   0,   0,   0,  -5,
        -10, -5,  -5,  -5,  -5,  -5,  -5, -10
    ],
    [ // bishop
        -10, -5,  -5,  -5,  -5,  -5,  -5, -10,
        -5,  0,   0,   0,   0,   0,   0,  -5,
        -5,  0,   5,   10,  10,  5,   0,  -5,
        -5,  5,   10,  20,  20,  10,  5,  -5,
        -5,  5,   10,  20,  20,  10,  5,  -5,
        -5,  0,   5,   10,  10,  5,   0,  -5,
        -5,  0,   0,   0,   0,   0,   0,  -5,
        -10, -5,  -5,  -5,  -5,  -5,  -5, -10
    ],
    [ // rook
        0, 0, 0, 5, 5, 0, 0, 0,
        0, 0, 0, 5, 5, 0, 0, 0,
        0, 0, 0, 5, 5, 0, 0, 0,
        0, 0, 0, 5, 5, 0, 0, 0,
        0, 0, 0, 5, 5, 0, 0, 0,
        0, 0, 0, 5, 5, 0, 0, 0,
        0, 0, 0, 5, 5, 0, 0, 0,
        0, 0, 0, 5, 5, 0, 0, 0,
    ],
    [ // queen
        -10, -5,  -5,  -5,  -5,  -5,  -5, -10,
        -5,  0,   0,   0,   0,   0,   0,  -5,
        -5,  0,   5,   5,   5,   5,   0,  -5,
        -5,  0,   10,  20,  20,  10,  0,  -5,
        -5,  0,   10,  20,  20,  10,  0,  -5,
        -5,  0,   5,   5,   5,   5,   0,  -5,
        -5,  0,   0,   0,   0,   0,   0,  -5,
        -10, -5,  -5,  -5,  -5,  -5,  -5, -10
    ],
    [ // king
        -50	,	-10	,	0	,	0	,	0	,	0	,	-10	,	-50	,
        -10,	0	,	10	,	10	,	10	,	10	,	0	,	-10	,
        0	,	10	,	20	,	20	,	20	,	20	,	10	,	0	,
        0	,	10	,	20	,	40	,	40	,	20	,	10	,	0	,
        0	,	10	,	20	,	40	,	40	,	20	,	10	,	0	,
        0	,	10	,	20	,	20	,	20	,	20	,	10	,	0	,
        -10,	0	,	10	,	10	,	10	,	10	,	0	,	-10	,
        -50	,	-10	,	0	,	0	,	0	,	0	,	-10	,	-50
    ],
    [ // dragon
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0,
    ],
];

pub fn bishop_pair_bonus(count: u8) ->i32 { if count >= 2 { return 40 } 0 }

const FILE_MASKS: [u64; 8] = [
    0x0101010101010101, // A file
    0x0202020202020202, // B file
    0x0404040404040404, // C file
    0x0808080808080808, // D file
    0x1010101010101010, // E file
    0x2020202020202020, // F file
    0x4040404040404040, // G file
    0x8080808080808080, // H file
];

pub const ROOK_OPEN_FILE_BONUS: i32 = 30;
pub const ROOK_SEMI_OPEN_FILE_BONUS: i32 = 15;
const QUEEN_OPEN_FILE_BONUS: i32 = 10;
const QUEEN_SEMI_OPEN_FILE_BONUS: i32 = 5;

pub static EVAL_CALLS: AtomicU64 = AtomicU64::new(0);
pub static EVAL_TIME_NS: AtomicU64 = AtomicU64::new(0);

pub const PAWN_PASSED_BONUS: [i32; 8] = [0 , 5, 10, 20, 35, 60, 100, 200];
const fn generate_passed_pawn_scores_white() -> [i32; 64] {
    let mut scores = [0; 64];
    let mut i = 0;
    while i < 64 {
        let rank = i / 8;
        scores[i] = PAWN_PASSED_BONUS[rank];
        i += 1;
    }
    scores
}
const fn generate_passed_pawn_scores_black() -> [i32; 64] {
    let mut scores = [0; 64];
    let mut i = 0;
    while i < 64 {
        let rank = 7 - (i / 8); // Flip rank for black
        scores[i] = PAWN_PASSED_BONUS[rank];
        i += 1;
    }
    scores
}
const fn generate_white_passed_pawn_masks() -> [u64; 64] {
    let mut masks = [0u64; 64];
    let mut sq = 0;
    
    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut mask = 0u64;
        
        // For each rank ahead of the current square
        let mut r = rank + 1;
        while r < 8 {
            // Same file
            mask |= 1u64 << (r * 8 + file);
            
            // Left file (if exists)
            if file > 0 {
                mask |= 1u64 << (r * 8 + file - 1);
            }
            
            // Right file (if exists)
            if file < 7 {
                mask |= 1u64 << (r * 8 + file + 1);
            }
            
            r += 1;
        }
        
        masks[sq] = mask;
        sq += 1;
    }
    
    masks
}
const fn generate_black_passed_pawn_masks() -> [u64; 64] {
    let mut masks = [0u64; 64];
    let mut sq = 0;
    
    while sq < 64 {
        let rank = sq / 8;
        let file = sq % 8;
        let mut mask = 0u64;
        
        // For each rank behind the current square (going down for black)
        let mut r = 0;
        while r < rank {
            // Same file
            mask |= 1u64 << (r * 8 + file);
            
            // Left file (if exists)
            if file > 0 {
                mask |= 1u64 << (r * 8 + file - 1);
            }
            
            // Right file (if exists)
            if file < 7 {
                mask |= 1u64 << (r * 8 + file + 1);
            }
            
            r += 1;
        }
        
        masks[sq] = mask;
        sq += 1;
    }
    
    masks
}
pub const WHITE_PASSED_PAWN_MASKS: [u64; 64] = generate_white_passed_pawn_masks();
pub const BLACK_PASSED_PAWN_MASKS: [u64; 64] = generate_black_passed_pawn_masks();
const PAWN_PASSED_WHITE_BONUS_BOARD: [i32; 64] = generate_passed_pawn_scores_white();
const PAWN_PASSED_BLACK_BONUS_BOARD: [i32; 64] = generate_passed_pawn_scores_black();

const PAWN_ISOLATED: i32 = -10;
const fn generate_isolated_pawn_masks() -> [u64; 64] {
    let mut file_masks = [0; 64];
    let mut square = 0;
    while square < 64 {
        let file = square % 8;
        let mut mask = 0;
        if file > 0 { mask |= file_masks[file - 1]; }
        if file < 7 { mask |= file_masks[file + 1]; }
        file_masks[square] = mask;
        square += 1;
    }

    file_masks
}
const ISOLATED_PAWN_MASK: [u64; 64] = generate_isolated_pawn_masks();

const PHASE_VALUES: [i32; PieceType::COUNT] = [
    0, // none
    0, // pawn
    1, // knight
    1, // bishop
    2, // rook
    4, // queen
    0, // king
    0, // dragon
];
pub const TOTAL_PHASE: i32 = 
    PHASE_VALUES[1] * 16 +
    PHASE_VALUES[2] * 4 +
    PHASE_VALUES[3] * 4 +
    PHASE_VALUES[4] * 4 +
    PHASE_VALUES[5] * 2;

const DOUBLED_PAWN_PENALTY: i32 = -15;

pub const PAWN_SHIELD: [u64; 64] = [
    (1 << 8) | (1 << 9),
    (1 << 8) | (1 << 9) | (1 << 10),
    (1 << 9) | (1 << 10) | (1 << 11),
    0,
    0,
    0,
    (1 << 13) | (1 << 14) | (1 << 15),
    (1 << 14) | (1 << 15),
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 2
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 3
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 4
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 5
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 6
    0, 0, 0, 0, 0, 0, 0, 0, // Rank 7
    (1 << 48) | (1 << 49),
    (1 << 48) | (1 << 49) | (1 << 50),
    (1 << 49) | (1 << 50) | (1 << 51),
    0,
    0,
    0,
    (1 << 53) | (1 << 54) | (1 << 55),
    (1 << 54) | (1 << 55),
];

impl Board {
    pub fn piece_value_at(&self, piece: Piece, square: usize) -> i32 {
        let piece_type = piece.piece_type();
        let color = piece.color();

        let material = PIECE_VALUE[piece_type.index()];
        let piece_square =
            if color == Color::WHITE { OPENING_PIECE_SQUARE[piece_type.index()][square] }
            else { OPENING_PIECE_SQUARE[piece_type.index()][MIRROR64[square]] };

        let value = material + piece_square;
        if color == Color::WHITE { value }
        else { -value }
    }

    pub fn evaluate(&self) -> i32 {
        // let start = Instant::now();
        // EVAL_CALLS.fetch_add(1, Ordering::Relaxed);

        let total_material_on_board = 
            (self.bitboards[Piece::WHITE_KNIGHT.index()] | 
            self.bitboards[Piece::BLACK_KNIGHT.index()]).count_ones() as i32 * 
            PHASE_VALUES[PieceType::KNIGHT.index()] +
            (self.bitboards[Piece::WHITE_BISHOP.index()] | 
            self.bitboards[Piece::BLACK_BISHOP.index()]).count_ones() as i32 * 
            PHASE_VALUES[PieceType::BISHOP.index()] +
            (self.bitboards[Piece::WHITE_ROOK.index()] | 
            self.bitboards[Piece::BLACK_ROOK.index()]).count_ones() as i32 * 
            PHASE_VALUES[PieceType::ROOK.index()] +
            (self.bitboards[Piece::WHITE_QUEEN.index()] | 
            self.bitboards[Piece::BLACK_QUEEN.index()]).count_ones() as i32 * 
            PHASE_VALUES[PieceType::QUEEN.index()];

        let mut phase = TOTAL_PHASE - total_material_on_board.min(TOTAL_PHASE);
        phase = (phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE;

        fn evaluate_helper(position: &Board, color: Color) -> (i32, i32) {
            fn get_attacks(piece_type: PieceType, color: Color, square: usize, occupancy: u64) -> u64 {
                match piece_type {
                    PieceType::PAWN => {
                        if color == Color::WHITE { return WHITE_PAWN_ATTACKS[square] }
                        BLACK_PAWN_ATTACKS[square]
                    },
                    PieceType::KNIGHT => KNIGHT_RAYS[square],
                    PieceType::BISHOP => get_bishop_attacks(square, occupancy),
                    PieceType::ROOK => get_rook_attacks(square, occupancy),
                    PieceType::QUEEN => get_bishop_attacks(square, occupancy) | get_rook_attacks(square, occupancy),
                    PieceType::KING => KING_RAYS[square],

                    _ => 0
                }
            }

            let mut open_score = 0;
            let mut end_score = 0;

            let mut bishop_count = 0;

            let (pieces, our_pawns, enemy_pawns, 
                passed_pawn_mask, passed_pawn_bonus, square_map,
                our_occupancy, enemy_pawn_attacks) = 
                if color == Color::WHITE { 
                    (Piece::WHITE_PIECES_EXCLUDE_PAWN,
                    position.bitboards[Piece::WHITE_PAWN.index()], 
                    position.bitboards[Piece::BLACK_PAWN.index()],
                    &WHITE_PASSED_PAWN_MASKS,
                    &PAWN_PASSED_WHITE_BONUS_BOARD,
                    &IDENTITY64,
                    position.occupancies[Color::WHITE.index()],
                    ((position.bitboards[Piece::BLACK_PAWN.index()] >> 7) & !FILE_MASKS[0]) | ((position.bitboards[Piece::BLACK_PAWN.index()] >> 9) & !FILE_MASKS[7]))
                } else {
                    (Piece::BLACK_PIECES_EXCLUDE_PAWN,
                    position.bitboards[Piece::BLACK_PAWN.index()],
                    position.bitboards[Piece::WHITE_PAWN.index()],
                    &BLACK_PASSED_PAWN_MASKS,
                    &PAWN_PASSED_BLACK_BONUS_BOARD,
                    &MIRROR64,
                    position.occupancies[Color::BLACK.index()],
                    ((position.bitboards[Piece::WHITE_PAWN.index()] << 7) & !FILE_MASKS[7]) | ((position.bitboards[Piece::WHITE_PAWN.index()] << 9) & !FILE_MASKS[0]))
                };
            let all_occupancy = position.occupancies[Color::WHITE.index()] | position.occupancies[Color::BLACK.index()];

            const KNIGHT_MOBILITY: [[i32; 9]; 2] = [
                [-20, -15, -5, 0, 5, 10, 15, 20, 25], // opening
                [-20, -10, 0, 5, 10, 15, 20, 25, 30]  // ending
            ];
            const BISHOP_MOBILITY: [[i32; 14]; 2] = [
                [-15, -5, 0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 50],
                [-15, -5, 0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 50]
            ];
            const ROOK_MOBILITY: [[i32; 15]; 2] = [
                [-15, -10, -5, 0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 50],
                [-15, -10, -5, 0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 100]
            ];
            const QUEEN_MOBILITY: [[i32; 28]; 2] = [
                [-10, -5, 0, 5, 10, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50, 50],
                [-10, -5, 0, 5, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100, 100]
            ];

            const PAWN_THREAT_ON_KNIGHT: [i32; 2] = [-20, -10];
            const PAWN_THREAT_ON_BISHOP: [i32; 2] = [-20, -10];
            const PAWN_THREAT_ON_ROOK:   [i32; 2] = [-40, -20];
            const PAWN_THREAT_ON_QUEEN:  [i32; 2] = [-50, -30];

            // Pawns
            let mut bitboard = our_pawns;
            while bitboard != 0 {
                let square = bitboard.trailing_zeros() as usize;
                bitboard &= bitboard - 1;

                open_score += PIECE_VALUE[PieceType::PAWN.index()];
                open_score += OPENING_PIECE_SQUARE[PieceType::PAWN.index()][square_map[square]];
                end_score += PIECE_VALUE[PieceType::PAWN.index()];
                end_score += ENDING_PIECE_SQUARE[PieceType::PAWN.index()][square_map[square]];

                // Passed pawns
                let is_passed = passed_pawn_mask[square] & enemy_pawns == 0;
                if is_passed {
                    let bonus = passed_pawn_bonus[square]; 
                    open_score += bonus / 2;
                    end_score += bonus 
                }
            
                // Isolated pawns
                let is_isolated = (our_pawns & ISOLATED_PAWN_MASK[square]) == 0;
                if is_isolated { 
                    open_score += PAWN_ISOLATED; 
                    end_score += PAWN_ISOLATED;
                }

                // Doubled Pawns
                // let file_mask = FILE_MASKS[square % 8];
                // if (our_pawns & file_mask).count_ones() > 1 {
                //     open_score += DOUBLED_PAWN_PENALTY;
                //     end_score += DOUBLED_PAWN_PENALTY;
                // }
            }

            for &piece in pieces {
                let mut bitboard = position.bitboards[piece.index()];
                while bitboard != 0 {
                    let square = bitboard.trailing_zeros() as usize;
                    bitboard &= bitboard - 1;

                    let piece_type = piece.piece_type();

                    open_score += PIECE_VALUE[piece_type.index()];
                    open_score += OPENING_PIECE_SQUARE[piece_type.index()][square_map[square]];
                    end_score += PIECE_VALUE[piece_type.index()];
                    end_score += ENDING_PIECE_SQUARE[piece_type.index()][square_map[square]];

                    // Mobility
                    let attacks = get_attacks(piece_type, color, square, all_occupancy);
                    let valid_moves = attacks & !our_occupancy;
                    let mobility = valid_moves.count_ones() as usize;
                    let is_attacked_by_pawn = ((1 << square) & enemy_pawn_attacks) != 0;

                    match piece_type {
                        PieceType::KNIGHT => {
                            let index = mobility.min(8);
                            open_score += KNIGHT_MOBILITY[0][index];
                            end_score += KNIGHT_MOBILITY[1][index];
                            if is_attacked_by_pawn {
                                open_score += PAWN_THREAT_ON_KNIGHT[0];
                                end_score += PAWN_THREAT_ON_KNIGHT[1];
                            }
                        },

                        PieceType::BISHOP => {
                            bishop_count += 1; // bishop pairs
                            let index = mobility.min(13);
                            open_score += BISHOP_MOBILITY[0][index];
                            end_score += BISHOP_MOBILITY[1][index];
                            if is_attacked_by_pawn {
                                open_score += PAWN_THREAT_ON_BISHOP[0];
                                end_score += PAWN_THREAT_ON_BISHOP[1];
                            }
                        },
                        
                        PieceType::ROOK | PieceType::QUEEN => {
                            let bonuses;
                            if piece_type == PieceType::ROOK { 
                                bonuses = [ROOK_OPEN_FILE_BONUS, ROOK_SEMI_OPEN_FILE_BONUS];
                                let index = mobility.min(14);
                                open_score += ROOK_MOBILITY[0][index];
                                end_score += ROOK_MOBILITY[1][index];
                                if is_attacked_by_pawn {
                                    open_score += PAWN_THREAT_ON_ROOK[0];
                                    end_score += PAWN_THREAT_ON_ROOK[1];
                                }
                            } else { 
                                bonuses = [QUEEN_OPEN_FILE_BONUS, QUEEN_SEMI_OPEN_FILE_BONUS];
                                let index = mobility.min(27);
                                open_score += QUEEN_MOBILITY[0][index];
                                end_score += QUEEN_MOBILITY[1][index];
                                if is_attacked_by_pawn {
                                    open_score += PAWN_THREAT_ON_QUEEN[0];
                                    end_score += PAWN_THREAT_ON_QUEEN[1];
                                }
                            }

                            // Open files
                            let file = square % 8;
                            let file_mask = FILE_MASKS[file];
                        
                            let open_file = file_mask & (our_pawns | enemy_pawns) == 0;
                            let semi_open_file = file_mask & our_pawns == 0;
                            if open_file { 
                                open_score += bonuses[0]; 
                                end_score += bonuses[0]; 
                            }
                            else if semi_open_file { 
                                open_score += bonuses[1]; 
                                end_score += bonuses[1]; 
                            }
                        },

                        PieceType::KING => {
                            let shield_mask = PAWN_SHIELD[square];
                            if shield_mask != 0 {
                                let shield = shield_mask & our_pawns;
                                let pawn_count = shield.count_ones();
                                let max_pawns = shield_mask.count_ones();
                                let missing = max_pawns - pawn_count;
                                match missing {
                                    0 => (),
                                    1 => open_score -= 10,
                                    2 => open_score -= 15,
                                    3 => open_score -= 25,
                                    _ => ()
                                }
                            }
                        },
        
                        _ => ()
                    }
                }
            }


            let bishop_bonus = bishop_pair_bonus(bishop_count);
            return (open_score + bishop_bonus, end_score + bishop_bonus)
        }

        let (white_open_score, white_end_score) = evaluate_helper(self, Color::WHITE);
        let (black_open_score, black_end_score) = evaluate_helper(self, Color::BLACK);
        
        let open_score = white_open_score - black_open_score;
        let end_score = white_end_score - black_end_score;
        let score = ((open_score * (256 - phase)) + (end_score * phase)) / 256;

        // EVAL_TIME_NS.fetch_add(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
        if self.side == Color::WHITE { score } 
        else { -score }
    }

    fn get_least_valuable_piece(&self, attackers_bb: u64, side: Color) -> Option<(PieceType, u64)> {
        if attackers_bb == 0 { return None; }

        let subset = attackers_bb & self.bitboards[PieceType::PAWN.bb_index(side)];
        if subset != 0 { return Some((PieceType::PAWN, subset & subset.wrapping_neg())); }

        let subset = attackers_bb & self.bitboards[PieceType::KNIGHT.bb_index(side)];
        if subset != 0 { return Some((PieceType::KNIGHT, subset & subset.wrapping_neg())); }

        let subset = attackers_bb & self.bitboards[PieceType::BISHOP.bb_index(side)];
        if subset != 0 { return Some((PieceType::BISHOP, subset & subset.wrapping_neg())); }

        let subset = attackers_bb & self.bitboards[PieceType::ROOK.bb_index(side)];
        if subset != 0 { return Some((PieceType::ROOK, subset & subset.wrapping_neg())); }

        let subset = attackers_bb & self.bitboards[PieceType::QUEEN.bb_index(side)];
        if subset != 0 { return Some((PieceType::QUEEN, subset & subset.wrapping_neg())); }

        let subset = attackers_bb & self.bitboards[PieceType::KING.bb_index(side)];
        if subset != 0 { return Some((PieceType::KING, subset & subset.wrapping_neg())); }

        None
    }

    pub fn static_exchange_evaluation(
        &self, 
        from_square: usize, 
        to_square: usize, 
        target_piece: PieceType, 
        mut capturer: PieceType
    ) -> i32 {
        const MAX_DEPTH: usize = 32;
        let mut gain = [0i32; MAX_DEPTH];
        let mut depth: usize = 0;

        let mut from_bb: u64 = 1 << from_square;
        let mut occupancy = self.occupancies[Color::BOTH.index()];
        let mut side_to_move = self.side.opposite();
        occupancy &= !from_bb;

        gain[depth] = PIECE_VALUE[target_piece.index()];

        loop {
            depth += 1;

            gain[depth] = PIECE_VALUE[capturer.index()] - gain[depth - 1];

            match self.get_smallest_attacker(to_square, side_to_move, occupancy) {
                Some((next_piece, next_bit)) => {
                    capturer = next_piece;
                    from_bb = next_bit;
                    occupancy &= !from_bb;
                    
                    side_to_move = side_to_move.opposite();
                }
                None => break,
            }
        }

        while depth > 1 {
            depth -= 1;
            gain[depth - 1] = -i32::max(-gain[depth - 1], gain[depth]);
        }

        gain[0]
    }
}
