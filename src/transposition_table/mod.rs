/*
 u32                 u16        u8
0000 0000 0000 0000 0000 0000 0011 1111 -> From
0000 0000 0000 0000 0000 1111 1100 0000 -> To
0000 0000 0000 0001 1111 0000 0000 0000 -> Captured
0000 0000 0000 0010 0000 0000 0000 0000 -> Is enpassant
0000 0000 0000 0100 0000 0000 0000 0000 -> Is pawn start
0000 0000 0000 1000 0000 0000 0000 0000 -> Is castle
0000 0001 1111 0000 0000 0000 0000 0000 -> Promoted piece

 u64         --- Split ---          
0000 0000 0000 0000 0000 0000 0011 1111 -> Depth (for replacement policy)
0000 0000 0000 0000 0011 1111 1100 0000 -> Age (for replacement policy)
0000 0000 0000 0000 1100 0000 0000 0000 -> Alpha beta flags
1111 1111 1111 1111 0000 0000 0000 0000 -> Score (32000 max)
*/

use std::sync::atomic::{AtomicU64, Ordering};

use crate::movegen::Move;

pub const ALPHA_FLAG: u8 = 0b0001;
pub const BETA_FLAG:  u8 = 0b0010;
pub const EXACT_FLAG: u8 = 0b0011;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TranspositionData(pub u64);
impl TranspositionData {
    pub fn new(mv: Move, depth: u8, age: u8, flag: u8, score: i16) -> Self {
        let mut data = TranspositionData(0);
        data.set_move(mv);
        data.set_depth(depth);
        data.set_flag(flag);
        data.set_score(score);

        data
    }

    pub fn get_move(&self) -> Move { Move((self.0 & 0xFFFF_FFFF) as u32) }
    pub fn set_move(&mut self, mv: Move) { self.0 = (self.0 & !0xFFFF_FFFF) | (mv.0 as u64); }

    pub fn get_depth(&self) -> u8 { ((self.0 >> 32) & 0x3F) as u8 }
    pub fn set_depth(&mut self, depth: u8) {
        let d = (depth as u64) & 0x3F;
        self.0 = (self.0 & !(0x3F << 32)) | (d << 32);
    }

    pub fn get_age(&self) -> u8 { ((self.0 >> 38) & 0xFF) as u8 }
    pub fn set_age(&mut self, age: u8) {
        let a = (age as u64) & 0xFF;
        self.0 = (self.0 & !(0xFF << 38)) | (a << 38);
    }

    pub fn get_flag(&self) -> u8 { ((self.0 >> 46) & 0x03) as u8 }
    pub fn set_flag(&mut self, flag: u8) {
        let f = (flag as u64) & 0x03;
        self.0 = (self.0 & !(0x03 << 46)) | (f << 46);
    }

    pub fn get_score(&self) -> i16 { (self.0 >> 48) as i16 }
    pub fn set_score(&mut self, score: i16) {
        let s = (score as u16 as u64) & 0xFFFF;
        self.0 = (self.0 & !(0xFFFF << 48)) | (s << 48);
    }
}

pub struct TranspositionEntry {
    encoded_key: AtomicU64,
    data: AtomicU64
}

pub struct TranspositionTable {
    entries: Vec<TranspositionEntry>,
    mask: usize
}
impl TranspositionTable {
    pub fn new(length_by_pow2: usize) -> Self {
        let size = 1 << length_by_pow2;
        let entries = (0..size)
            .map(|_| TranspositionEntry {
                encoded_key: AtomicU64::new(0),
                data: AtomicU64::new(0),
            })
            .collect();
        Self {
            entries: entries,
            mask: size - 1
        }
    }

    fn index(&self, position_key: u64) -> usize { position_key as usize & self.mask }

    pub fn probe(&self, position_key: u64) -> Option<TranspositionData> {
        let entry = &self.entries[self.index(position_key)];
        // Acquire: Don't do any loads until after the key has been acquired: key is acquired before data is loaded
        let checksum = entry.encoded_key.load(Ordering::Acquire);
        let data = entry.data.load(Ordering::Relaxed);

        if (checksum ^ data) == position_key { 
            Some(TranspositionData(data)) 
        }
        else { None }
    }

    pub fn store(&self, position_key: u64, data: TranspositionData) {
        let index = self.index(position_key);
        let entry = &self.entries[index];

        let old_checksum = entry.encoded_key.load(Ordering::Relaxed);
        let old_data_raw = entry.data.load(Ordering::Relaxed);
        let old_data = TranspositionData(old_data_raw);

        let is_same_pos = (old_checksum ^ old_data_raw) == position_key;
        let replace = if is_same_pos {
            data.get_depth() >= old_data.get_depth()
        }
        else {
            if old_data.get_age() != data.get_age() { true }
            else { data.get_depth() >= old_data.get_depth() }
        };

        if replace {
            entry.data.store(data.0, Ordering::Relaxed);
            let checksum = position_key ^ data.0;
            // Release: Only store when all previous writes are done: data must be stored before key is
            entry.encoded_key.store(checksum, Ordering::Release);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::Move;

    #[test]
    fn transposition_data_packs_and_unpacks_correctly() {
        // Pick values that exercise the bitfields.
        let mv = Move::new(
            12,                 // from
            44,                 // to
            17,                 // captured
            0b101,              // flags: enpassant + castle (bits 17 and 19)
            9,                  // promote
        );

        let depth: u8 = 55;      // fits in 6 bits (0..63)
        let age: u8 = 201;
        let flag: u8 = EXACT_FLAG; // fits in 2 bits (0..3)
        let score: i16 = -1234;

        let mut td = TranspositionData(0);
        td.set_move(mv);
        td.set_depth(depth);
        td.set_age(age);
        td.set_flag(flag);
        td.set_score(score);

        // TranspositionData fields
        assert_eq!(td.get_move(), mv);
        assert_eq!(td.get_depth(), depth);
        assert_eq!(td.get_age(), age);
        assert_eq!(td.get_flag(), flag);
        assert_eq!(td.get_score(), score);

        // Also verify the embedded Move bit layout survived packing.
        let mv2 = td.get_move();
        assert_eq!(mv2.from_square(), 12);
        assert_eq!(mv2.to_square(), 44);
        assert_eq!(mv2.captured(), 17);
        assert_eq!(mv2.promoted(), 9);
        assert!(mv2.is_en_passant());
        assert!(!mv2.is_double_push());
        assert!(mv2.is_castling());
    }
}
