use std::{sync::{Arc, atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering}}, time::Instant};

use crate::{board::Board, defs::{Color, Piece, PieceType}, movegen::Move, transposition_table::{self, TranspositionTable}};

pub mod search;
pub mod test;
pub mod evaluate;

pub struct Search {
    pub transposition_table: Arc<TranspositionTable>,
    pub nodes_visited: AtomicUsize,
    pub age: AtomicU8,

    pub stop_flag: AtomicBool,
    pub start_time: Instant,
    pub time_limit_ms: u128,

    pub first_cutoffs: AtomicUsize,
    pub total_cutoffs: AtomicUsize,
}
impl Search {
    pub fn new(transposition_table: Arc<TranspositionTable>, time_limit_ms: u128) -> Self {
        Self { 
            transposition_table: transposition_table,
            nodes_visited: AtomicUsize::new(0),
            age: AtomicU8::new(0),

            stop_flag: AtomicBool::new(false),
            start_time: Instant::now(),
            time_limit_ms: time_limit_ms,

            first_cutoffs: AtomicUsize::new(0),
            total_cutoffs: AtomicUsize::new(0),
        }
    }

    pub fn reset_stats(&self) {
        self.nodes_visited.store(0, Ordering::Relaxed);
        self.first_cutoffs.store(0, Ordering::Relaxed);
        self.total_cutoffs.store(0, Ordering::Relaxed);
    }

    pub fn reset(&self) {
        self.reset_stats();
        self.age.store(0, Ordering::Relaxed);
    }

    pub fn should_stop(&self) -> bool {
        if self.time_limit_ms == 0 { return false }
        if self.stop_flag.load(Ordering::Relaxed) { return true; }

        let elapsed = self.start_time.elapsed().as_millis();
        if elapsed >= self.time_limit_ms {
            self.stop_flag.store(true, Ordering::Relaxed);
            return true
        }

        false
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

    pub fn get_pv_line(&mut self, search: &Search, depth: u8) -> Vec<Move> {
        let mut pv_moves = Vec::new();
        let mut moves_made = 0;

        for _ in 0..depth {
            if let Some(entry) = search.transposition_table.probe(self.position_key) {
                let mv = entry.get_move();
                if mv == Move::default() { break }

                if self.make_move(mv) {
                    pv_moves.push(mv);
                    moves_made += 1;
                    if self.is_repetition() || self.fifty_move >= 100 { break; }
                }
                else { break }
            }
            else { break }
        }

        for _ in 0..moves_made { self.take_move(); }

        pv_moves
    }

    pub fn update_history(&mut self, mv: Move, depth: u8, is_good: bool) {
        let mut bonus = (depth * depth) as i16;
        bonus = if is_good { bonus } else { -bonus };

        let to = mv.to_square();
        let piece_index = self.pieces[mv.from_square()].index();
        
        self.history_heuristic[piece_index][to] = (self.history_heuristic[piece_index][to] + bonus).min(8999);
    }

    // With complex gravity, trying to reach 0 from both sides (when malus happens) asymptotically
    // pub fn update_history(&mut self, mv: Move, depth: u8, is_good: bool) {
    //     let bonus = if is_good { (depth * depth) as i16 } else { -((depth * depth) as i16) };

    //     let to = mv.to_square();
    //     let piece_index = self.pieces[mv.from_square()].index();

    //     let current = self.history_heuristic[piece_index][to];
    //     let max_history_score: i16 = 8000;

    //     let gravity = current * bonus.abs() / max_history_score;
    //     self.history_heuristic[piece_index][to] = 
    //         (current + bonus - gravity.signum() * gravity.abs()).clamp(-max_history_score, max_history_score);
    // }

    pub fn get_previous_move(&self) -> Option<Move> {
        if self.history_ply > 0 { Some(self.history[self.history_ply - 1].mv) } 
        else { None }
    }

    pub fn has_non_pawn_material(&self) -> bool {
        let side = self.side;

        let knights = self.bitboards[Piece::make(PieceType::KNIGHT, side).index()];
        let bishops = self.bitboards[Piece::make(PieceType::BISHOP, side).index()];
        let rooks   = self.bitboards[Piece::make(PieceType::ROOK, side).index()];
        let queens  = self.bitboards[Piece::make(PieceType::QUEEN, side).index()];
        let dragons = self.bitboards[Piece::make(PieceType::DRAGON, side).index()];

        (knights | bishops | rooks | queens | dragons) != 0
    }
}