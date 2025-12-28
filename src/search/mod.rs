use std::sync::{Arc, atomic::{AtomicU8, AtomicUsize, Ordering}};

use crate::{board::Board, defs::{Color, Piece}, transposition_table::TranspositionTable};

pub mod search;
pub mod test;
pub mod evaluate;

pub struct Search {
    pub transposition_table: Arc<TranspositionTable>,
    pub nodes_visited: AtomicUsize,
    pub age: AtomicU8
}
impl Search {
    pub fn new(length_by_pow2: usize) -> Self {
        Self { 
            transposition_table: TranspositionTable::new(length_by_pow2).into(),
            nodes_visited: AtomicUsize::new(0),
            age: AtomicU8::new(0)
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
}