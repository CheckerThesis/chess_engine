use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use crate::{board::Board, defs::{Color, Piece}, transposition_table::TranspositionTable};

pub mod search;
pub mod test;

pub struct Search {
    pub transposition_table: Arc<TranspositionTable>,
    pub nodes_visited: AtomicUsize,
}
impl Search {
    pub fn new(length_by_pow2: usize) -> Self {
        Self { 
            transposition_table: TranspositionTable::new(length_by_pow2).into(),
            nodes_visited: AtomicUsize::new(0)
        }
    }

    pub fn reset_stats(&self) {
        self.nodes_visited.store(0, Ordering::Relaxed);
    }
}

impl Board {
    pub fn is_repetition(&self) -> bool {
        if self.history_ply <= 1 { return false }
        for i in (self.history_ply - self.fifty_move as usize)..self.history_ply - 1 {
            if self.position_key == self.history[i].position_key { return true }
        }

        false
    }

    pub fn evaluate(&self) -> i32 {
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
        const PIECE_SQUARE: [[i32; 64]; 8] = [
            [0; 64], // none
            [ // pawn
                0	,	0	,	0	,	0	,	0	,	0	,	0	,	0	,
                10	,	10	,	0	,	-10	,	-10	,	0	,	10	,	10	,
                5	,	0	,	0	,	5	,	5	,	0	,	0	,	5	,
                0	,	0	,	10	,	20	,	20	,	10	,	0	,	0	,
                5	,	5	,	5	,	10	,	10	,	5	,	5	,	5	,
                10	,	10	,	10	,	20	,	20	,	10	,	10	,	10	,
                20	,	20	,	20	,	30	,	30	,	20	,	20	,	20	,
                0	,	0	,	0	,	0	,	0	,	0	,	0	,	0
            ],
            [ // knight
                0	,	-10	,	0	,	0	,	0	,	0	,	-10	,	0	,
                0	,	0	,	0	,	5	,	5	,	0	,	0	,	0	,
                0	,	0	,	10	,	10	,	10	,	10	,	0	,	0	,
                0	,	0	,	10	,	20	,	20	,	10	,	5	,	0	,
                5	,	10	,	15	,	20	,	20	,	15	,	10	,	5	,
                5	,	10	,	10	,	20	,	20	,	10	,	10	,	5	,
                0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
                0	,	0	,	0	,	0	,	0	,	0	,	0	,	0
            ],
            [ // bishop
                0	,	0	,	-10	,	0	,	0	,	-10	,	0	,	0	,
                0	,	0	,	0	,	10	,	10	,	0	,	0	,	0	,
                0	,	0	,	10	,	15	,	15	,	10	,	0	,	0	,
                0	,	10	,	15	,	20	,	20	,	15	,	10	,	0	,
                0	,	10	,	15	,	20	,	20	,	15	,	10	,	0	,
                0	,	0	,	10	,	15	,	15	,	10	,	0	,	0	,
                0	,	0	,	0	,	10	,	10	,	0	,	0	,	0	,
                0	,	0	,	0	,	0	,	0	,	0	,	0	,	0
            ],
            [ // rook
                0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
                0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
                0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
                0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
                0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
                0	,	0	,	5	,	10	,	10	,	5	,	0	,	0	,
                25	,	25	,	25	,	25	,	25	,	25	,	25	,	25	,
                0	,	0	,	5	,	10	,	10	,	5	,	0	,	0
            ],
            [ // queen
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
            ],
            [ // king
                0	,	5	,	5	,	-10	,	-10	,	0	,	10	,	5	,
                -30	,	-30	,	-30	,	-30	,	-30	,	-30	,	-30	,	-30	,
                -50	,	-50	,	-50	,	-50	,	-50	,	-50	,	-50	,	-50	,
                -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
                -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
                -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
                -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,
                -70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70	,	-70
            ],
            [ // dragon
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
                0	,	0	,	0	,	0 	,	0	,	0	,	0	,	0	,
            ],
        ];

        const PIECE_VALUE: [i32; 8] = [
            0,     // none
            100,   // pawn
            325,   // knight
            325,   // bishop
            550,   // rook
            1000,  // queen
            32767, // king,
            1300,  // dragon
        ];

        let mut score: i32 = 0;

        for &piece in Piece::WHITE_PIECES {
            let mut bitboard = self.bitboards[piece.index()];
            while bitboard != 0 {
                let square = bitboard.trailing_zeros() as usize;
                bitboard &= bitboard - 1;
                score += PIECE_VALUE[piece.piece_type().index()];
                score += PIECE_SQUARE[piece.piece_type().index()][square];
            }
        }

        for &piece in Piece::BLACK_PIECES {
            let mut bitboard = self.bitboards[piece.index()];
            while bitboard != 0 {
                let square = bitboard.trailing_zeros() as usize;
                bitboard &= bitboard - 1;
                score -= PIECE_VALUE[piece.piece_type().index()];
                score -= PIECE_SQUARE[piece.piece_type().index()][MIRROR64[square]];
            }
        }
        
        if self.side == Color::WHITE { score }
        else { -score }
    }
}
