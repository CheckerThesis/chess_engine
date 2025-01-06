use std::hash::{BuildHasher, Hasher};

#[derive(Default)]
pub struct ZobristHasher { hash: u64, }

impl Hasher for ZobristHasher {
    fn write(&mut self, bytes: &[u8]) {
        // combine bytes into a u64 (not relevant for Zobrist keys)
        for &byte in bytes {
            self.hash = self.hash.wrapping_add(byte as u64);
        }
    }

    // directly use the Zobrist key
    fn write_u64(&mut self, value: u64) { self.hash = value; }

    fn finish(&self) -> u64 { self.hash }
}

pub struct ZobristHasherBuilder;

impl BuildHasher for ZobristHasherBuilder {
    type Hasher = ZobristHasher;

    fn build_hasher(&self) -> Self::Hasher {
        ZobristHasher::default()
    }
}
// TODO make manual hash table
