pub mod attacks;
pub mod generate;

use core::fmt;
use std::sync::LazyLock;

use crate::{board::Board, defs::{Color, Piece, PieceType, Ranks, RANKS_BOARD}};

pub struct MoveList {
    moves: [u64; 256],
    count: usize
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

pub static RANK_BB_MASK: LazyLock<[u64; 9]> = LazyLock::new(|| {
    let mut rank_bb_mask: [u64; 9] = [0; 9];
    const RANK_1: u64 = 0x00000000000000FF;

    for i in 1..9 {
        rank_bb_mask[i] = RANK_1 << ((i - 1) * 8);
    }

    rank_bb_mask
});

fn get_rank(square: u64) -> usize {
    for i in 0..8 {
        if square & RANK_BB_MASK[i] != 0 { return i }
    }
    100
}

const A_FILE_MASK: u64 = !0xFEFEFEFEFEFEFEFE;
const B_FILE_MASK: u64 = A_FILE_MASK << 1;
const G_FILE_MASK: u64 = A_FILE_MASK << 6;
const H_FILE_MASK: u64 = !0x7F7F7F7F7F7F7F7F;

pub static WHITE_PAWN_ATTACKS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut attacks: [u64; 64] = [0; 64];

    for i in 0..64 {
        let sq: u64 = 1 << i;
        let capture;

        if A_FILE_MASK & sq != 0 { capture = sq << 9; } 
        else if H_FILE_MASK & sq != 0 { capture = sq << 7; } 
        else { capture = (sq << 7) | (sq << 9); }

        attacks[i] = capture;
    }

    attacks
});

pub static BLACK_PAWN_ATTACKS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut attacks: [u64; 64] = [0; 64];

    for i in (0..64).rev() {
        let sq: u64 = 1 << i as u64;
        let capture;
        
        if A_FILE_MASK & sq != 0 { capture = sq >> 7; } 
        else if H_FILE_MASK & sq != 0 { capture = sq >> 9; } 
        else { capture = (sq >> 7) | (sq >> 9); }

        attacks[i] = capture;
    }

    attacks
});

pub static UP_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut up_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let mut ray_bb: u64 = 0;
        let mut ray_square= square as u64;

        while get_rank(ray_square) < 7 {
            ray_square += 8;
            if ray_square < 64 { ray_bb |= 1 << ray_square; } 
            else { break; }
        }
        up_rays[square] = ray_bb;
    }

    up_rays[0] = up_rays[1] >> 1;
    up_rays
});
pub static DOWN_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut down_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let mut ray_bb: u64 = 0;
        let mut ray_square= square as u64;

        while get_rank(ray_square) < 7 {
            if ray_square >= 8 { 
                ray_square -= 8;
                ray_bb |= 1 << ray_square; 
            }
            else { break; }
        }
        down_rays[square] = ray_bb;
    }

    down_rays
});
pub static LEFT_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut left_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let mut ray_bb: u64 = 0;
        let mut ray_square = square as u64;

        while ray_square % 8 > 0 {
            ray_square -= 1;
            ray_bb |= 1 << ray_square;
        }
        left_rays[square] = ray_bb;
    }

    left_rays
});
pub static RIGHT_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut right_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let mut ray_bb: u64 = 0;
        let mut ray_square = square as u64;

        while ray_square % 8 < 7 {
            ray_square += 1;
            ray_bb |= 1 << ray_square;
        }
        right_rays[square] = ray_bb;
    }

    right_rays
});

pub static SUPPLY_DIAGONAL_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut anti_diagonal_rays: [u64; 64] = [0; 64];

    for square in 0..64 {
        let mut ray_bb: u64 = 0;

        let mut ray_square_up_right = square as u64;
        // Rank is (square / 8), File is (square % 8)
        while (ray_square_up_right / 8) < 7 && (ray_square_up_right % 8) < 7 {
            ray_square_up_right += 9;
            ray_bb |= 1 << ray_square_up_right;
        }

        let mut ray_square_down_light = square as u64;
        while (ray_square_down_light / 8) > 0 && (ray_square_down_light % 8) > 0 {
            ray_square_down_light -= 9;
            ray_bb |= 1 << ray_square_down_light;
        }

        anti_diagonal_rays[square] = ray_bb;
    }

    anti_diagonal_rays
});
pub static DEMAND_DIAGONAL_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut main_diagonal_rays: [u64; 64] = [0; 64];

    for square in 0..64 {
        let mut ray_bb: u64 = 0;

        // --- Calculate Up-Left ray (+7) ---
        let mut ray_square_up_left = square as u64;
        while (ray_square_up_left / 8) < 7 && (ray_square_up_left % 8) > 0 {
            ray_square_up_left += 7; // Move one square up-left
            ray_bb |= 1 << ray_square_up_left; // Add this square to the bitboard
        }

        // --- Calculate Down-Right ray (-7) ---
        let mut current_square_dr = square as u64;
        // Loop while the current square is not on the 1st rank (rank 0)
        // and not on the H file (file 7).
        // Rank is (square / 8), File is (square % 8)
        while (current_square_dr / 8) > 0 && (current_square_dr % 8) < 7 {
            current_square_dr -= 7; // Move one square down-right
            ray_bb |= 1 << current_square_dr; // Add this square to the bitboard
        }

        main_diagonal_rays[square] = ray_bb;
    }

    main_diagonal_rays
});

pub static KNIGHT_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    const NOT_A_FILE: u64 = 0xfefefefefefefefe; 
    const NOT_AB_FILE: u64 = 0xfcfcfcfcfcfcfcfc;
    const NOT_GH_FILE: u64 = 0x3f3f3f3f3f3f3f3f;
    const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

    let mut knight_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let square_bb: u64 = 1 << square;
        let mut attacks: u64 = 0;

        attacks |= (square_bb & NOT_H_FILE) << 17;
        attacks |= (square_bb & NOT_GH_FILE) << 10;
        attacks |= (square_bb & NOT_GH_FILE) >> 6;
        attacks |= (square_bb & NOT_H_FILE) >> 15;

        attacks |= (square_bb & NOT_A_FILE) << 15;
        attacks |= (square_bb & NOT_AB_FILE) << 6;
        attacks |= (square_bb & NOT_AB_FILE) >> 10;
        attacks |= (square_bb & NOT_A_FILE) >> 17;

        knight_rays[square] = attacks;
    }

    knight_rays
});

pub static KING_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    const NOT_A_FILE: u64 = 0xfefefefefefefefe; 
    const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

    let mut king_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let square_bb: u64 = 1 << square;
        let mut attacks: u64 = 0;

        attacks |= square_bb << 8;  // up
        attacks |= square_bb >> 8;  // down
        
        // Horizontal moves (left and right)
        attacks |= (square_bb & NOT_H_FILE) << 1;  // right
        attacks |= (square_bb & NOT_A_FILE) >> 1;  // left
        
        // Diagonal moves
        attacks |= (square_bb & NOT_H_FILE) << 9;  // up-right
        attacks |= (square_bb & NOT_A_FILE) << 7;  // up-left
        attacks |= (square_bb & NOT_H_FILE) >> 7;  // down-right
        attacks |= (square_bb & NOT_A_FILE) >> 9;  // down-left

        king_rays[square] = attacks;
    }

    king_rays
});