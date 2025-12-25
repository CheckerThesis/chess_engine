use std::cmp::max;

use crate::board::{Board};
use crate::fn_name;
use crate::movegen::{Move, MoveList};

impl Board {
    pub fn alpha_beta(&mut self, mut alpha: i32, mut beta: i32, depth: u8) -> i32 {
        #[cfg(debug_assertions)] { self.check_board(fn_name!()); }

        if depth <= 0 { return self.evaluate() }

        // If draw
        if (self.is_repetition() || self.fifty_move >= 100) && self.ply == 1 { return 0 }

        let mut movelist = MoveList::new();
        movelist.generate_all_moves(self);

        for scored_move in movelist.iter() {
            let mv = scored_move.mv;

            if !self.make_move(mv) { continue; }
            let evaluation = -self.alpha_beta(-beta, -alpha, depth - 1);
            self.take_move();

            alpha = max(alpha, evaluation);
            if alpha >= beta { break } // prune/cutoff
        }

        alpha
    }

    pub fn iterative_deepen(&mut self, depth: u8) -> Option<Move> {
        let mut best_move = None;
        let mut best_evaluation: i32;
        
        for current_depth in 1..=depth {
            best_evaluation = self.alpha_beta(-30000, 30000, current_depth);
        }

        best_move
    }
}