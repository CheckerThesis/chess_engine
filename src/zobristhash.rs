use std::hash::{BuildHasher, Hasher};

#[derive(Default, Clone)]
pub struct IdentityHasher(u64);
impl Hasher for IdentityHasher {
    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, _bytes: &[u8]) {
        todo!()
    }
}

#[derive(Default, Clone)]
pub struct IdentityBuildHasher;
impl BuildHasher for IdentityBuildHasher {
    type Hasher = IdentityHasher;
    fn build_hasher(&self) -> Self::Hasher {
        IdentityHasher::default()
    }
}
