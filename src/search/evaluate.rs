use std::{sync::atomic::{AtomicU64, Ordering}, time::Instant};

use crate::{board::Board, defs::{Color, Piece, PieceType}, movegen::{magic::{get_bishop_attacks, get_rook_attacks}}};

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

        fn evaluate_helper(position: &Board, pieces: &[Piece], color: Color) -> i32 {
            let mut score = 0;
            let mut bishop_count = 0;

            for &piece in pieces {
                let mut bitboard = position.bitboards[piece.index()];
                while bitboard != 0 {
                    let square = bitboard.trailing_zeros() as usize;
                    bitboard &= bitboard - 1;
                    score += PIECE_VALUE[piece.piece_type().index()];
                    score += 
                        if color == Color::WHITE { OPENING_PIECE_SQUARE[piece.piece_type().index()][square] }
                        else { OPENING_PIECE_SQUARE[piece.piece_type().index()][MIRROR64[square]] };

                    let piece_type = piece.piece_type();
                    match piece_type {
                        // Bishop pairs
                        PieceType::BISHOP => bishop_count += 1,
                        
                        // Open files
                        PieceType::ROOK | PieceType::QUEEN => {
                            let bonuses =
                            if piece_type == PieceType::ROOK { [ROOK_OPEN_FILE_BONUS, ROOK_SEMI_OPEN_FILE_BONUS] }
                            else { [QUEEN_OPEN_FILE_BONUS, QUEEN_SEMI_OPEN_FILE_BONUS] };

                            let file = square % 8;
                            let file_mask = FILE_MASKS[file];
                            let white_pawns = position.bitboards[Piece::WHITE_PAWN.index()];
                            let black_pawns = position.bitboards[Piece::BLACK_PAWN.index()];
                        
                            let open_file = file_mask & (white_pawns | black_pawns) == 0;
                            let pawns = 
                                if color == Color::WHITE { white_pawns }
                                else { black_pawns };
                            let semi_open_file = file_mask & pawns == 0;
                            if open_file { score += bonuses[0]; }
                            else if semi_open_file { score += bonuses[1]; }
                        },
                        _ => ()
                    }
                }
            }

            return score + bishop_pair_bonus(bishop_count);
        }

        let score = 
            evaluate_helper(self, Piece::WHITE_PIECES, Color::WHITE) - 
            evaluate_helper(self, Piece::BLACK_PIECES, Color::BLACK); 

        // EVAL_TIME_NS.fetch_add(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
        if self.side == Color::WHITE { score } 
        else { -score }
    }

    fn get_least_valuable_piece(&self, attackers_bb: u64, side: Color) -> Option<(PieceType, u64)> {
        if attackers_bb == 0 { return None; }

        // Pawns
        let subset = attackers_bb & self.bitboards[PieceType::PAWN.bb_index(side)];
        if subset != 0 {
            return Some((PieceType::PAWN, subset & subset.wrapping_neg()));
        }

        // Knights
        let subset = attackers_bb & self.bitboards[PieceType::KNIGHT.bb_index(side)];
        if subset != 0 {
            return Some((PieceType::KNIGHT, subset & subset.wrapping_neg()));
        }

        // Bishops
        let subset = attackers_bb & self.bitboards[PieceType::BISHOP.bb_index(side)];
        if subset != 0 {
            return Some((PieceType::BISHOP, subset & subset.wrapping_neg()));
        }

        // Rooks
        let subset = attackers_bb & self.bitboards[PieceType::ROOK.bb_index(side)];
        if subset != 0 {
            return Some((PieceType::ROOK, subset & subset.wrapping_neg()));
        }

        // Queens
        let subset = attackers_bb & self.bitboards[PieceType::QUEEN.bb_index(side)];
        if subset != 0 {
            return Some((PieceType::QUEEN, subset & subset.wrapping_neg()));
        }

        // Kings
        let subset = attackers_bb & self.bitboards[PieceType::KING.bb_index(side)];
        if subset != 0 {
            return Some((PieceType::KING, subset & subset.wrapping_neg()));
        }

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
