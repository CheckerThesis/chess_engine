use std::sync::LazyLock;

use rand::{rng, RngCore};

use crate::{board::Board, defs::{Color, Piece}};

pub static PIECE_KEYS: LazyLock<[[u64; 64]; Piece::COUNT]> = LazyLock::new(|| {
    let mut rng = rng();
    let mut keys = [[0u64; 64]; Piece::COUNT];
    for piece in 0..Piece::COUNT {
        for square in 0..64 { keys[piece][square] = rng.next_u64(); }
    }
    keys
});
pub static SIDE_KEY: LazyLock<u64> = LazyLock::new(|| {
    let mut rng = rng();
    rng.next_u64()
});
pub static CASTLE_KEYS: LazyLock<[u64; 16]> = LazyLock::new(|| {
    let mut rng = rng();
    let mut keys = [0u64; 16];
    for i in 0..16 { keys[i] = rng.next_u64(); }
    keys
});
pub static EN_PASSANT_KEYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut rng = rng();
    let mut keys = [0u64; 64];
    for i in 0..64 { keys[i] = rng.next_u64(); }
    keys
});

impl Board {    
    pub fn generate_position_key(&self) -> u64 {
        let mut final_key = 0u64;

        for square in 0..64 {
            let piece = self.pieces[square];
            if piece != Piece::NONE { final_key ^= PIECE_KEYS[piece.index()][square]; }
        }
        if self.side == Color::WHITE { final_key ^= *SIDE_KEY; }
        if let Some(ep_square) = self.en_passant { final_key ^= EN_PASSANT_KEYS[ep_square as usize]; }
        final_key ^= CASTLE_KEYS[self.castle_permission as usize];
        final_key
    }
}