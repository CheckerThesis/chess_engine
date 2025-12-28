use std::cmp::max;
use std::i32;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::board::{Board};
use crate::movegen::attacks::square_attacked;
use crate::search::Search;
use crate::transposition_table::{ALPHA_FLAG, BETA_FLAG, EXACT_FLAG, TranspositionData};
use crate::{fn_name, transposition_table};
use crate::movegen::{Move, MoveList};

impl Board {
    pub fn alpha_beta(&mut self, search: &Search, mut alpha: i32, mut beta: i32, depth: u8) -> i32 {
        #[cfg(debug_assertions)] { self.check_board(fn_name!()); }

        search.nodes_visited.fetch_add(1, Ordering::Relaxed);

        if depth <= 0 { return self.evaluate() }
        // If draw
        if (self.is_repetition() || self.fifty_move >= 100) && self.ply == 1 { return 0 }

        let position_key = self.position_key;
        let original_alpha = alpha;
        
        // Transposition table
        if let Some(transposition_data) = search.transposition_table.probe(position_key) {
            if transposition_data.get_depth() >= depth {
                let transposition_score = transposition_data.get_score() as i32;
                match transposition_data.get_flag() {
                    EXACT_FLAG => return transposition_score,
                    // Fail low, true score is less than `transposition_score`
                    ALPHA_FLAG => if transposition_score <= alpha { return transposition_score },
                    // Fail high, this move is too good so opponent won't let you get it
                    BETA_FLAG  => if transposition_score >= beta  { return transposition_score },
                    _ => panic!()
                }
            }
        }

        let side = self.side.index();
        let in_check = square_attacked(self.king_square[self.side.index()], self.side, self);

        let mut best_move = Move::default();
        let mut best_score = -i32::MAX;
        let mut found_any_legal_moves = false;

        let mut movelist = MoveList::new();
        movelist.generate_all_moves(self);

        for scored_move in movelist.iter() {
            let mv = scored_move.mv;

            if !self.make_move(mv) { continue; }
            let evaluation = -self.alpha_beta(search, -beta, -alpha, depth - 1);
            self.take_move();

            found_any_legal_moves = true;
            best_score = max(best_score, evaluation);

            if evaluation > alpha {
                alpha = evaluation;
                best_move = mv;
            }

            if beta <= alpha { break } // prune/cutoff because move is too good
        }

        if found_any_legal_moves {
            let flag = 
                if alpha <= original_alpha { ALPHA_FLAG }  // didn't improve alpha
                else if beta <= alpha      { BETA_FLAG }   // prune/cutoff because move is too good
                else                       { EXACT_FLAG }; // alpha < score < beta ; no pruning
            let mut data = TranspositionData(0);
            data.set_move(best_move);
            data.set_depth(depth);
            data.set_flag(flag);
            data.set_score(best_score.clamp(i16::MIN as i32, i16::MAX as i32) as i16);

            search.transposition_table.store(position_key, data);
        } else {
            return
                if in_check { -i32::MAX + self.ply as i32 }
                else { 0 }
        }

        best_score
    }

    pub fn iterative_deepen(&mut self, search: &Search, depth: u8) -> Option<Move> {
        let root_key = self.position_key;
        let mut best_move = None;
        
        for current_depth in 1..=depth {
            let evaluation = self.alpha_beta(search, -30000, 30000, current_depth);

            if let Some(transposition_data) = search.transposition_table.probe(root_key) {
                let mv = transposition_data.get_move();
                if mv != Move::default() { best_move = Some(mv); }
            }

            if let Some(mv) = best_move {
                println!("depth {} score {} best {} raw {:?}", current_depth, evaluation, mv, best_move);
            }
        }

        best_move
    }
}