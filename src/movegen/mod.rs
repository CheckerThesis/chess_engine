pub mod attacks;
pub mod generate;

use std::sync::LazyLock;

use crate::board::Board;

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

    pub fn move_builder(
        from: usize, 
        to: usize, 
        capture: usize, 
        flags: usize, 
        promote: usize
    ) -> u32 { 
        from as u32 | 
        ((to as u32) << 6) | 
        ((capture as u32) << 12) | 
        ((flags as u32) << 17) | 
        ((promote as u32) << 20)
    }

    fn set_score(mv: u32, score: u64) -> u64 { return (mv as u64) | (score << 47) } 

    pub fn add_quiet_move(&mut self, position: &Board, mv: u32) {
        // TODO set score killer move
        self.add(MoveList::set_score(mv, 0));
    }

    pub fn add_capture_move(&mut self, position: &Board, mv: u32) {
        // TODO set score MVV_LVA
        self.add(MoveList::set_score(mv, 10000));
    }

    pub fn len(&self) -> usize { self.count }

    pub fn iter(&self) -> std::slice::Iter<'_, u64> { self.moves[..self.count].iter() }
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
pub fn from_square(the_move: u32) -> u8 { (the_move & 0x3F) as u8 }
pub fn to_square(the_move: u32) -> u8 { (the_move >> 6 & 0x3F) as u8 }
pub fn captured(the_move: u32) -> u8 { (the_move >> 12 & 0x1F) as u8 }
pub fn promoted(the_move: u32) -> u8 { (the_move >> 20 & 0x1F) as u8 }

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