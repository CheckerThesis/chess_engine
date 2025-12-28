

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::*;
    use crate::{board, fn_name, search::{self, Search}, transposition_table::EXACT_FLAG};
    use crate::{board::Board, defs::PieceType, fens::FEN_START, movegen::{MOVE_FLAG_NONE, Move}, squares::squares::{B1, B8, C3, C6}};

    #[test]
    fn is_repetition() {
        let mut position = Board::new(FEN_START);
        
        let b1c3 = Move::new(B1, C3, PieceType::NONE.index(), MOVE_FLAG_NONE, PieceType::NONE.index());
        position.make_move(b1c3);
        if position.is_repetition() { panic!(); }
        
        let b8c6 = Move::new(B8, C6, PieceType::NONE.index(), MOVE_FLAG_NONE, PieceType::NONE.index());
        position.make_move(b8c6);
        if position.is_repetition() { panic!(); }

        let c3b1 = Move::new(C3, B1, PieceType::NONE.index(), MOVE_FLAG_NONE, PieceType::NONE.index());
        position.make_move(c3b1);
        if position.is_repetition() { panic!(); }

        let c6b8 = Move::new(C6, B8, PieceType::NONE.index(), MOVE_FLAG_NONE, PieceType::NONE.index());
        position.make_move(c6b8);
        if !position.is_repetition() { panic!(); }
    }

    #[test]
    fn finds_mate_in_one() {
        let mut search = Search::new(20);
        let mut board = Board::new("6k1/5ppp/8/8/8/8/8/R6K w - - 0 1");

        let best_move = board.iterative_deepen(&search, 2);

        assert!(best_move.is_some());
        let mv = best_move.unwrap();
        assert_eq!(mv.to_string(), "a1a8");
    }

    #[test]
    fn test_tt_pruning_effectiveness() {
        let mut search = Search::new(20);
        let mut board = Board::new(FEN_START);
        
        // 1. First Run: Cold Start (TT is empty)
        search.reset_stats();
        board.iterative_deepen(&search, 5); 
        let nodes_cold = search.nodes_visited.load(Ordering::Relaxed);
        
        // 2. Second Run: Warm Start (TT is full from previous run)
        search.reset_stats();
        board.iterative_deepen(&search, 5);
        let nodes_warm = search.nodes_visited.load(Ordering::Relaxed);
        
        println!("Cold nodes: {}, Warm nodes: {}", nodes_cold, nodes_warm);

        // 3. Assertion: Warm run should visit DRASTICALLY fewer nodes.
        // Usually, it's just the PV nodes, so < 1% of the cold run.
        assert!(nodes_warm < nodes_cold / 10, "TT did not prune effectively!");
    }

    #[test]
    fn test_tt_storage_flags() {
        let search = Search::new(20); // 1 MB
        let mut board = Board::new(FEN_START); // Start position
        let root_key = board.position_key;

        // Run a shallow search
        board.alpha_beta(&search, -30000, 30000, 1);

        // Manually probe the table using the Key
        let entry = search.transposition_table.probe(root_key);

        // 1. Verify we got a hit
        assert!(entry.is_some(), "Root position was not stored in TT");
        let data = entry.unwrap();

        // 2. Verify Data Integrity
        assert_eq!(data.get_depth(), 1, "Stored depth matches search depth");
        
        // Since we ran a full window search (-30000, 30000) on start pos,
        // we expect an EXACT flag (unless start pos is a checkmate/draw).
        assert_eq!(data.get_flag(), EXACT_FLAG, "Start pos should be Exact score inside window");
        
        // 3. Verify Score is sensible (Start pos is usually around +0.2 to +0.5 or 0)
        let score = data.get_score();
        assert!(score > -100 && score < 100, "Start pos score out of expected bounds");
    }

    #[test]
    fn prefers_shorter_mate() {
        let mut search = Search::new(20);
        // A position where white can mate in 1 (Ra8#) or mate in 2 (others)
        // 7k/R7/8/8/8/8/8/7K w - - 0 1
        let mut board = Board::new("7k/Q7/8/1R6/8/8/8/K7 w - - 0 1"); 

        // Search Depth 3 to allow seeing the "longer" mate path too
        let best_move = board.iterative_deepen(&search, 3);
        
        assert!(best_move.is_some());
        // Engine MUST pick the immediate mate, not delay it
        assert_eq!(best_move.unwrap().to_string(), "b5b8"); 
    }

    #[test]
    fn recognizes_stalemate() {
        let mut search = Search::new(20);
        // Black king at h8, White Queen at f7. Black has no moves, but is not in check.
        let mut board = Board::new("7k/Q7/8/6R1/8/8/8/K7 b - - 0 1");

        // We expect the score to be EXACTLY 0.
        let score = board.alpha_beta(&search, -30000, 30000, 1);

        assert_eq!(score, 0, "Stalemate should evaluate to exactly 0");
    }

    #[test]
    fn tt_stores_best_move_at_root() {
        let search = Search::new(20);
        let mut board = Board::new(FEN_START);
        
        board.alpha_beta(&search, -30000, 30000, 4);
        
        // Manually verify the TT has a move for the root
        let entry = search.transposition_table.probe(board.position_key);
        assert!(entry.is_some());
        assert!(entry.unwrap().get_move() != Move::default());
    }
}