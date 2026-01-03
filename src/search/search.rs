use std::cmp::max;
use std::i32;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use crate::board::{Board, MAX_DEPTH};
use crate::defs::Piece;
use crate::movegen::attacks::square_attacked;
use crate::movegen::mvv_lva::PIECE_VALUE;
use crate::search::{Search, evaluate};
use crate::transposition_table::{ALPHA_FLAG, BETA_FLAG, EXACT_FLAG, TranspositionData};
use crate::{fn_name, transposition_table};
use crate::movegen::{Move, MoveList};

pub const MATE_SCORE: i32 = 30000;
pub const MATE_THRESHOLD: i32 = MATE_SCORE - 1000;

impl Board {
    fn quiescence(&mut self, search: &Search, mut alpha: i32, mut beta: i32) -> i32 {
        search.nodes_visited.fetch_add(1, Ordering::Relaxed);

        // If draw
        if (self.is_repetition() || self.fifty_move >= 100) && self.ply == 1 { return 0 }

        let evaluate = self.evaluate(); // standing pat
        if evaluate >= beta { return beta }
        if evaluate > alpha { alpha = evaluate; }

        let mut movelist = MoveList::new();
        movelist.generate_all_captures(self);
        movelist.sort();

        for scored_move in movelist.iter() {
            let mv = scored_move.mv;

            if self.static_exchange_evaluation(
                mv.from_square(), 
                mv.to_square(), 
                Piece(mv.captured() as u8).piece_type(), 
                self.pieces[mv.from_square()].piece_type()
            ) < 0 { continue }

            // Delta pruning
            let captured_piece_value = PIECE_VALUE[Piece(mv.captured() as u8).piece_type().index()];
            if mv.promoted() == 0 {
                const DELTA_MARGIN: i32 = 200;
                if evaluate + captured_piece_value as i32 + DELTA_MARGIN < alpha { continue }
            }

            if !self.make_move(mv) { continue; }
            let score = -self.quiescence(search, -beta, -alpha);
            self.take_move();

            if score >= beta { return beta }
            if score > alpha { alpha = score; }
        }

        alpha
    }

    pub fn alpha_beta(&mut self, search: &Search, mut alpha: i32, mut beta: i32, depth: u8) -> i32 {
        #[cfg(debug_assertions)] { self.check_board(fn_name!()); }

        if search.nodes_visited.load(Ordering::Relaxed) & 2048 == 0 && search.should_stop() { return 0 }
        if search.stop_flag.load(Ordering::Relaxed) { return 0 }

        search.nodes_visited.fetch_add(1, Ordering::Relaxed);

        if depth <= 0 { return self.quiescence(search, alpha, beta) }
        // If draw
        if (self.is_repetition() || self.fifty_move >= 100) && self.ply == 1 { return 0 }

        let position_key = self.position_key;
        let original_alpha = alpha;
        
        // Transposition table prune
        let mut transposition_move = Move::default();
        if let Some(transposition_data) = search.transposition_table.probe(position_key) {
            transposition_move = transposition_data.get_move();

            if transposition_data.get_depth() >= depth {
                let mut transposition_score = transposition_data.get_score() as i32;
                if transposition_score >= MATE_THRESHOLD { transposition_score -= self.ply as i32; }
                else if transposition_score <= -MATE_THRESHOLD { transposition_score += self.ply as i32; }

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
        let mut best_score = -MATE_SCORE;
        let mut found_any_legal_moves = false;

        let mut movelist = MoveList::new();
        movelist.generate_all_moves(self);

        // Boost transposition table move
        for mv in movelist.iter_mut() {
            if mv.mv.0 == transposition_move.0 {
                mv.score = 30000;
                break;
            }
        }

        movelist.sort();

        for scored_move in movelist.iter() {
            let mv = scored_move.mv;

            // SEE pruning
            // if depth <= 4 && self.static_exchange_evaluation(
            //     mv.from_square(), 
            //     mv.to_square(), 
            //     Piece(mv.captured() as u8).piece_type(), 
            //     self.pieces[mv.from_square()].piece_type()
            // ) < -50 { continue }

            if !self.make_move(mv) { continue; }
            let evaluation = -self.alpha_beta(search, -beta, -alpha, depth - 1);
            self.take_move();

            found_any_legal_moves = true;
            best_score = max(best_score, evaluation);

            if evaluation > alpha {
                alpha = evaluation;
                best_move = mv;
            }

            if beta <= alpha { // prune/cutoff because move is too good
                if mv.captured() == 0 { 
                    // Killer moves
                    let ply = self.ply as usize;
                    if ply < MAX_DEPTH && self.killers[ply][0] != Some(mv) {
                        self.killers[ply][1] = self.killers[ply][0];
                        self.killers[ply][0] = Some(mv);
                    }

                    self.update_history(mv, depth, true); // history heuristic

                    // Reduce the score for all quiet moves before this one (because they didn't cause a cutoff)
                    for prev in movelist.iter().take_while(|m| m.mv != mv) {
                        if prev.mv.captured() == 0 { self.update_history(prev.mv, depth, false); }
                    }
                }
                break 
            }
        }

        if found_any_legal_moves {
            let flag = 
                if alpha <= original_alpha { ALPHA_FLAG }  // didn't improve alpha
                else if beta <= alpha      { BETA_FLAG }   // prune/cutoff because move is too good
                else                       { EXACT_FLAG }; // alpha < score < beta ; no pruning
            let mut data = TranspositionData(0);
            let mut normalized_score = best_score;
            if normalized_score >= MATE_THRESHOLD { normalized_score += self.ply as i32; }
            else if normalized_score <= -MATE_THRESHOLD { normalized_score -= self.ply as i32; }
            data.set_move(best_move);
            data.set_depth(depth);
            data.set_flag(flag);
            data.set_age(search.age.load(Ordering::Relaxed));
            data.set_score(normalized_score.clamp(i16::MIN as i32, i16::MAX as i32) as i16);

            search.transposition_table.store(position_key, data);
        } else {
            return
                if in_check { -MATE_SCORE + self.ply as i32 }
                else { 0 }
        }

        best_score
    }

    pub fn iterative_deepen(&mut self, search: &Search, depth: u8, yes_print: bool) -> Option<Move> {
        search.age.fetch_add(1, Ordering::Relaxed);

        search.stop_flag.store(false, Ordering::Relaxed);

        let root_key = self.position_key;
        let mut best_move = None;
        
        for current_depth in 1..=depth {
            // Simple gravity, dividing by 2
            for piece_type in 0..Piece::COUNT {
                for i in 0..64 {
                    self.history_heuristic[piece_type][i] >>= 1; 
                }
            }

            let evaluation = self.alpha_beta(search, -30000, 30000, current_depth);

            if search.stop_flag.load(Ordering::Relaxed) { break; }

            if let Some(transposition_data) = search.transposition_table.probe(root_key) {
                let mv = transposition_data.get_move();
                if mv != Move::default() { best_move = Some(mv); }
            }

            if let Some(mv) = best_move {
                if yes_print {
                    let pv_line = self.get_pv_line(search, current_depth);
                    let pv_string = pv_line.iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");

                    let time = search.start_time.elapsed().as_millis();
                    let nodes = search.nodes_visited.load(Ordering::Relaxed);
                    let nps = if time > 0 { (nodes as u128 * 1000) / time } else { 0 };
                    
                    println!(
                        "info depth {} score cp {} nodes {} time {} nps {} pv {}", 
                        current_depth, evaluation, nodes, time, nps, pv_string
                    );
                }
            }
        }

        best_move
    }
}