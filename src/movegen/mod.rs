pub mod attacks;
pub mod attack_table;
pub mod generate;
pub mod bitboards;
pub mod test;
pub mod magic;
pub mod mvv_lva;

use core::fmt;

use crate::{board::Board, defs::{Color, FILES_BOARD, Piece, PieceType, RANKS_BOARD, Ranks}};

pub const MOVE_FLAG_NONE: usize = 0;
pub const MOVE_FLAG_EN_PASSANT: usize = 1 << 0;
pub const MOVE_FLAG_PAWN_START: usize = 1 << 1;
pub const MOVE_FLAG_CASTLE: usize = 1 << 2;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Move(pub u32);
impl Move {
    pub fn new(from: usize, to: usize, capture: usize, flags: usize, promote: usize) -> Self { 
        Self (from as u32 | 
        ((to as u32) << 6) | 
        ((capture as u32) << 12) | 
        ((flags as u32) << 17) | 
        ((promote as u32) << 20))
    }

 /* 0000 0000 0000 0000 0000 0000 0011 1111 -> From
    0000 0000 0000 0000 0000 1111 1100 0000 -> To
    0000 0000 0000 0001 1111 0000 0000 0000 -> Captured
    0000 0000 0000 0010 0000 0000 0000 0000 -> Is enpassant
    0000 0000 0000 0100 0000 0000 0000 0000 -> Is pawn start
    0000 0000 0000 1000 0000 0000 0000 0000 -> Is castle
    0000 0001 1111 0000 0000 0000 0000 0000 -> Promoted piece */
    pub fn from_square(&self) -> usize { (self.0 & 0x3F) as usize }
    pub fn to_square(&self) -> usize { (self.0 >> 6 & 0x3F) as usize }
    pub fn captured(&self) -> usize { (self.0 >> 12 & 0x1F) as usize }
    pub fn promoted(&self) -> usize { (self.0 >> 20 & 0x1F) as usize }

    pub fn is_en_passant(&self) -> bool { (self.0 & (1 << 17)) != 0 }
    pub fn is_double_push(&self) -> bool { (self.0 & (1 << 18)) != 0 }
    pub fn is_castling(&self) -> bool { (self.0 & (1 << 19)) != 0 }
}
impl Default for Move {
    fn default() -> Self { Self(0) }
}
impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let from = SQUARE_TO_STRING[self.from_square()];
        let to = SQUARE_TO_STRING[self.to_square()];

        if self.promoted() != 0 {
            let promo = get_promo_char(self.promoted());
            write!(f, "{}{}{}", from, to, promo)
        } else {
            write!(f, "{}{}", from, to)
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct ScoredMove {
    pub mv: Move,
    pub score: i16, 
}
impl ScoredMove {
    pub fn new(mv: Move, score: i16) -> Self { Self { mv, score } }

    pub fn default() -> Self { Self { mv: Move::default(), score: 0 } } 
}

pub struct MoveList {
    pub moves: [ScoredMove; 256],
    pub count: usize
}
impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [ScoredMove::new(Move::new(0, 0, 0, 0, 0), 0); 256],
            count: 0,
        }
    }

    fn add(&mut self, mv: ScoredMove) {
        self.moves[self.count] = mv;
        self.count += 1;
    }

    pub fn len(&self) -> usize { self.count }

    pub fn iter(&self) -> std::slice::Iter<'_, ScoredMove> { self.moves[..self.count].iter() }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, ScoredMove> { self.moves[..self.count].iter_mut() }

    pub fn sort(&mut self) {
        self.moves[..self.count].sort_unstable_by(|a, b| b.score.cmp(&a.score));
    }
}

fn get_promo_char(promo_index: usize) -> &'static str {
    match promo_index {
        // Knights (White=8, Black=9)
        8 | 9 => "n",
        // Bishops (White=12, Black=13)
        12 | 13 => "b",
        // Rooks (White=16, Black=17)
        16 | 17 => "r",
        // Queens (White=20, Black=21)
        20 | 21 => "q",
        // Dragons (White=28, Black=29) - Optional, included if your variant needs it
        28 | 29 => "d", 
        // Default
        _ => "",
    }
}

pub const SQUARE_TO_STRING: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

impl fmt::Display for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Move List ({} moves found):", self.count)?;

        for (i, scored_move) in self.iter().enumerate() {
            let mv = scored_move.mv;
            writeln!(f, "  {:>2}: {}{}{} (Score: {}, Raw: {:?})",
                     i + 1,
                     SQUARE_TO_STRING[mv.from_square()],
                     SQUARE_TO_STRING[mv.to_square()],
                     get_promo_char(mv.promoted()),
                     scored_move.score,
                     scored_move.mv)?;
        }
        
        Ok(())
    }
}
