pub mod attacks;
pub mod generate;
pub mod bitboards;

use core::fmt;
use std::sync::LazyLock;

use crate::{board::Board, defs::{Color, Piece, PieceType, Ranks, RANKS_BOARD}};

pub struct MoveList {
    moves: [u64; 256],
    pub count: usize
}
impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [0; 256],
            count: 0,
        }
    }

    fn add(&mut self, mv: u64) {
        self.moves[self.count] = mv;
        self.count += 1;
    }

    pub fn len(&self) -> usize { self.count }

    pub fn iter(&self) -> std::slice::Iter<'_, u64> { self.moves[..self.count].iter() }
}

const SQUARE_TO_STRING: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

/// Returns the promotion piece character based on the piece index.
///
/// This function makes assumptions based on your `add_..._pawn_move` functions:
/// 1. `promote: 0` is used for no-promotion.
/// 2. The piece indices for Q, R, B, N are passed for promotion.
/// 3. We assume a common encoding like:
///    W_Q=5, W_R=4, W_B=3, W_N=2
///    B_Q=11, B_R=10, B_B=9, B_N=8
///    (This order matches your `queen`, `rook`, `bishop`, `knight` calls)
fn get_promo_char(promo_index: u32) -> &'static str {
    match promo_index {
        // White pieces (assuming N=2, B=3, R=4, Q=5)
        2 => "n",
        3 => "b",
        4 => "r",
        5 => "q",
        // Black pieces (assuming n=8, b=9, r=10, q=11)
        8 => "n",
        9 => "b",
        10 => "r",
        11 => "q",
        // Default: no promotion (index 0) or invalid (King/Pawn)
        _ => "",
    }
}

impl fmt::Display for MoveList {
    /// Formats the move list for printing.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write a header
        writeln!(f, "Move List ({} moves found):", self.count)?;
        
        // Iterate over all moves in the list
        for i in 0..self.count {
            // Get the 64-bit move entry
            let move_with_score = self.moves[i];

            // Extract the 32-bit move data (lower 32 bits)
            let move_data = move_with_score as u32;

            // Extract the score (upper bits)
            let score = move_with_score >> 47;

            // --- Decode the 32-bit move data ---
            // Based on: move_builder(from, to, capture, flags, promote)
            
            // from: bits 0-5 (mask 0x3F)
            let from = (move_data & 0x3F) as usize;
            
            // to: bits 6-11 (mask 0x3F)
            let to = ((move_data >> 6) & 0x3F) as usize;
            
            // promote: bits 20-24 (mask 0x1F)
            let promote = (move_data >> 20) & 0x1F;

            // Convert square indices to algebraic notation
            let from_sq_str = SQUARE_TO_STRING[from];
            let to_sq_str = SQUARE_TO_STRING[to];
            
            // Get the promotion character (e.g., "q", "r", or "")
            let promo_str = get_promo_char(promote as u32);

            // Write the formatted move string
            // Example: "  1: e2e4 (Score: 0)"
            // Example: " 12: e7e8q (Score: 10000)"
            writeln!(f, "  {:>2}: {}{}{} (Score: {})",
                     i + 1,
                     from_sq_str,
                     to_sq_str,
                     promo_str,
                     score)?;
        }
        
        Ok(())
    }
}

/*
0000 0000 0000 0000 0000 0000 0011 1111 -> From
0000 0000 0000 0000 0000 1111 1100 0000 -> To
0000 0000 0000 0001 1111 0000 0000 0000 -> Captured
0000 0000 0000 0010 0000 0000 0000 0000 -> Is enpassant
0000 0000 0000 0100 0000 0000 0000 0000 -> Is pawn start
0000 0000 0000 1000 0000 0000 0000 0000 -> Is castle
0000 0001 1111 0000 0000 0000 0000 0000 -> Promoted piece
*/
pub fn from_square(mv: u32) -> usize { (mv & 0x3F) as usize }
pub fn to_square(mv: u32) -> usize { (mv >> 6 & 0x3F) as usize }
pub fn captured(mv: u32) -> usize { (mv >> 12 & 0x1F) as usize }
pub fn promoted(mv: u32) -> usize { (mv >> 20 & 0x1F) as usize }

pub fn is_en_passant(mv: u32) -> bool { (mv & (1 << 17)) != 0 }
pub fn is_double_push(mv: u32) -> bool { (mv & (1 << 18)) != 0 }
pub fn is_castling(mv: u32) -> bool { (mv & (1 << 19)) != 0 }

pub mod MoveFlag {
    pub const NONE: usize       = 0;
    pub const EN_PASSANT: usize = 0b0001; // Will become 1 << 17
    pub const PAWN_START: usize = 0b0010; // Will become 1 << 18
    pub const CASTLE: usize     = 0b0100; // Will become 1 << 19
}