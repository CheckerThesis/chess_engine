

#[cfg(test)]
mod tests {
    use std::sync::{Arc, atomic::Ordering};

    use super::*;
    use crate::{board::{self, MAX_DEPTH}, defs::Color, fn_name, search::{self, Search, evaluate::{PIECE_VALUE}, search::{MATE_SCORE, MATE_THRESHOLD}}, squares::squares::{A1, A2, A3, A5, B2, B3, D5, E4}, transposition_table::{self, EXACT_FLAG, TranspositionData, TranspositionTable}};
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
        let transposition_table = Arc::new(TranspositionTable::new(20));
        let mut search = Search::new(transposition_table, 0);
        let mut board = Board::new("6k1/5ppp/8/8/8/8/8/R6K w - - 0 1");

        let best_move = board.iterative_deepen(&search, 2, false);

        assert!(best_move.is_some());
        let mv = best_move.unwrap();
        assert_eq!(mv.to_string(), "a1a8");
    }

    #[test]
    fn test_tt_pruning_effectiveness() {
        let transposition_table = Arc::new(TranspositionTable::new(20));
        let mut search = Search::new(transposition_table, 0);
        let mut board = Board::new(FEN_START);
        
        // 1. First Run: Cold Start (TT is empty)
        search.reset_stats();
        board.iterative_deepen(&search, 5, false); 
        let nodes_cold = search.nodes_visited.load(Ordering::Relaxed);
        
        // 2. Second Run: Warm Start (TT is full from previous run)
        search.reset_stats();
        board.iterative_deepen(&search, 5, false);
        let nodes_warm = search.nodes_visited.load(Ordering::Relaxed);
        
        println!("Cold nodes: {}, Warm nodes: {}", nodes_cold, nodes_warm);

        // 3. Assertion: Warm run should visit DRASTICALLY fewer nodes.
        // Usually, it's just the PV nodes, so < 1% of the cold run.
        assert!(nodes_warm < nodes_cold / 10, "TT did not prune effectively!");
    }

    #[test]
    fn test_tt_storage_flags() {
        let transposition_table = Arc::new(TranspositionTable::new(20));
        let search = Search::new(transposition_table, 0); // 1 MB
        let mut board = Board::new(FEN_START); // Start position
        let root_key = board.position_key;

        // Run a shallow search
        board.alpha_beta(&search, -30000, 30000, 1, false);

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
        let transposition_table = Arc::new(TranspositionTable::new(20));
        let mut search = Search::new(transposition_table, 0);
        // A position where white can mate in 1 (Ra8#) or mate in 2 (others)
        // 7k/R7/8/8/8/8/8/7K w - - 0 1
        let mut board = Board::new("7k/Q7/8/1R6/8/8/8/K7 w - - 0 1"); 

        // Search Depth 3 to allow seeing the "longer" mate path too
        let best_move = board.iterative_deepen(&search, 3, false);
        
        assert!(best_move.is_some());
        // Engine MUST pick the immediate mate, not delay it
        assert_eq!(best_move.unwrap().to_string(), "b5b8"); 
    }

    #[test]
    fn recognizes_stalemate() {
        let transposition_table = Arc::new(TranspositionTable::new(20));
        let mut search = Search::new(transposition_table, 0);
        // Black king at h8, White Queen at f7. Black has no moves, but is not in check.
        let mut board = Board::new("7k/Q7/8/6R1/8/8/8/K7 b - - 0 1");

        // We expect the score to be EXACTLY 0.
        let score = board.alpha_beta(&search, -30000, 30000, 1, false);

        assert_eq!(score, 0, "Stalemate should evaluate to exactly 0");
    }

    #[test]
    fn tt_stores_best_move_at_root() {
        let transposition_table = Arc::new(TranspositionTable::new(20));
        let search = Search::new(transposition_table, 0);
        let mut board = Board::new(FEN_START);
        
        board.alpha_beta(&search, -30000, 30000, 4, false);
        
        // Manually verify the TT has a move for the root
        let entry = search.transposition_table.probe(board.position_key);
        assert!(entry.is_some());
        assert!(entry.unwrap().get_move() != Move::default());
    }

    #[test]
    fn test_tt_mate_score_normalization() {
        fn normalize_score_for_store(raw_score: i32, ply: i32) -> i16 {
        let mut score = raw_score;
            if score >= MATE_THRESHOLD {
                score += ply;
            } else if score <= -MATE_THRESHOLD {
                score -= ply;
            }
            score as i16
        }

        fn unnormalize_score_from_probe(tt_score: i16, ply: i32) -> i32 {
            let mut score = tt_score as i32;
            if score >= MATE_THRESHOLD {
                score -= ply;
            } else if score <= -MATE_THRESHOLD {
                score += ply;
            }
            score
        }
        
        let tt = TranspositionTable::new(20); // Size 1MB
        let position_key: u64 = 123456789;

        // SCENARIO: Mate in 1 found at Ply 5
        // Search score: 30000 - (ply 5 + 1 dist) = 29994
        let ply_5 = 5;
        let search_score_at_ply_5 = MATE_SCORE - (ply_5 + 1); 
        
        // 1. Emulate STORE at Ply 5
        let stored_score = normalize_score_for_store(search_score_at_ply_5, ply_5);
        
        let mut data = TranspositionData(0);
        data.set_score(stored_score);
        data.set_depth(10); // Arbitrary
        data.set_age(1);    // Arbitrary
        tt.store(position_key, data);

        // 2. Emulate PROBE at Ply 20 (The "Time Travel" Check)
        // We reached the same position, but much deeper in the game tree.
        // It is still "Mate in 1", so score should be: 30000 - (ply 20 + 1 dist) = 29979
        let ply_20 = 20;
        let entry = tt.probe(position_key).expect("Should find the entry we just stored");
        
        let retrieved_raw = entry.get_score();
        let final_score = unnormalize_score_from_probe(retrieved_raw, ply_20);

        let expected_score_at_ply_20 = MATE_SCORE - (ply_20 + 1);

        assert_eq!(final_score, expected_score_at_ply_20, 
            "The score retrieved at Ply 20 did not match the expected relative mate score.");
            
        println!("Stored Raw (Ply 5 context removed): {}", stored_score); // Should be 29999
        println!("Retrieved (Ply 20 context added):   {}", final_score);  // Should be 29979
    }

    #[test]
    fn test_tt_replacement_policy_age_and_depth() {
        // 1. Create a small table (2^4 = 16 entries) so collisions are easy to force
        let tt_pow2 = 4;
        let tt_size = 1 << tt_pow2; 
        let tt = TranspositionTable::new(tt_pow2);
        
        // 2. Define two keys that map to the SAME index (Collision)
        // index = key & (size - 1)
        let key_a: u64 = 5;              // index 5
        let key_b: u64 = 5 + tt_size as u64; // index 5 (5 + 16 = 21; 21 & 15 = 5)

        assert_ne!(key_a, key_b); // Ensure they are different keys

        // Helper to create data
        let make_data = |depth: u8, age: u8, score: i16| -> TranspositionData {
            let mut d = TranspositionData(0);
            d.set_depth(depth);
            d.set_age(age);
            d.set_score(score);
            d
        };

        // --- CASE 1: Same Position (Match) ---
        // Store deep data
        tt.store(key_a, make_data(10, 10, 100));
        
        // Try to overwrite with shallow data for the SAME position (newer age)
        tt.store(key_a, make_data(2, 11, 200));
        
        let entry = tt.probe(key_a).unwrap();
        assert_eq!(entry.get_depth(), 10, "SAME POS: Should KEEP depth 10 (High Quality), ignoring depth 2 (Low Quality)");
        assert_eq!(entry.get_age(), 10, "SAME POS: Should keep the age of the high-quality entry");

        // --- CASE 2: Different Position (Collision) ---
        // Now we store Key B. It maps to the same slot as Key A.
        // The existing entry (Key A) is Age 10. Our new entry (Key B) is Age 11.
        
        // Store Key B (Age 11, Depth 2)
        tt.store(key_b, make_data(2, 11, 300));
        
        let entry_b = tt.probe(key_b).expect("Should find entry for Key B");
        
        // HERE is where Age logic kicks in. 
        // Even though Depth 2 (New) < Depth 10 (Old), the Ages differ (11 vs 10) AND Keys differ.
        // The old entry is "stale" (from a previous search), so we trash it.
        assert_eq!(entry_b.get_score(), 300, "COLLISION: Should replace because old entry was Stale (Age 10 vs 11)");
        assert_eq!(entry_b.get_depth(), 2);
        
        // Verify Key A is effectively gone (probe returns None or Key B data which fails checksum)
        assert!(tt.probe(key_a).is_none(), "Key A should have been overwritten by Key B");
    }

    #[test]
    fn quiescence_poison() {
        let mut position = Board::new("5k2/8/2p5/3p4/3Q4/8/8/7K w - - 0 1");
        let transposition_table = Arc::new(TranspositionTable::new(20));
        let mut search = Search::new(transposition_table, 0);
        let score = position.iterative_deepen(&search, 1, true).unwrap();
        println!("{}", score);
        // edit the return to quiesence in alpha-beta to evaluate
    }

    #[test]
    fn test_see_hanging_piece() {
        // Scenario: White Rook (A1) captures hanging Black Pawn (A5)
        // Expected: +100 (Gain Pawn)
        let mut board = Board::new("8/8/8/p7/8/8/8/R7 w - - 0 1");
        
        let score = board.static_exchange_evaluation(
            A1,        // To: A5
            A5,    // Target: Pawn
            PieceType::PAWN,        // From: A1
            PieceType::ROOK     // Actor: Rook
        );

        assert_eq!(score, PIECE_VALUE[PieceType::PAWN.index()], "Capturing undefended pawn should return Pawn Value");
    }

    #[test]
    fn test_see_bad_capture() {
        // Scenario: White Rook (A1) captures Black Pawn (A5) defended by Black Rook (A8)
        // Exchange: RxP (+100) -> RxR (-500)
        // Result: White loses 400.
        let mut board = Board::new("r7/8/8/p7/8/8/8/R7 w - - 0 1");

        let score = board.static_exchange_evaluation(
            A1,
            A5,
            PieceType::PAWN,
            PieceType::ROOK
        );

        // Score should be negative (Value of Pawn - Value of Rook)
        assert_eq!(score, PIECE_VALUE[PieceType::PAWN.index()] - PIECE_VALUE[PieceType::ROOK.index()], "Trading Rook for Pawn should be negative");
    }

    #[test]
    fn xray_exchange() {
        let mut board = Board::new("r7/8/8/p7/8/8/Q7/R7 w - - 0 1");
        let score = board.static_exchange_evaluation(
            A1, 
            A5, 
            PieceType::PAWN, 
            PieceType::ROOK
        );

        assert_eq!(score, PIECE_VALUE[PieceType::PAWN.index()], "Battery should make capture safe (+100)");
    }

    #[test]
    fn complex_exchange() {
        let mut board = Board::new("3q4/8/4p3/3p4/4B3/2N5/8/8 w - - 0 1");
        let score = board.static_exchange_evaluation(
            E4, 
            D5, 
            PieceType::PAWN, 
            PieceType::KNIGHT
        );

        assert_eq!(score, PIECE_VALUE[PieceType::PAWN.index()] - PIECE_VALUE[PieceType::KNIGHT.index()], "Result should be losing the Knight for the Pawn (-220)");
    }

    #[test]
    fn test_killer_moves_logic() {
        pub fn store_killer(position: &mut Board, mv: Move) {
            if mv.captured() != 0 { return; } // Only quiet moves
            
            let ply = position.ply as usize;
            if ply >= MAX_DEPTH { return; }

            // Prevent duplication
            if position.killers[ply][0] != Some(mv) {
                position.killers[ply][1] = position.killers[ply][0];
                position.killers[ply][0] = Some(mv);
            }
        }
        let mut position = Board::new(FEN_START);
        
        let move_a = Move::new(A2, A3, 0, 0, 0);
        let move_b = Move::new(B2, B3, 0, 0, 0);
        
        let ply = 5; 
        position.ply = ply as u8;

        // --- TEST 1: Store First Killer ---
        // Simulate beta cutoff with Move A
        store_killer(&mut position, move_a); 

        assert_eq!(position.killers[ply][0], Some(move_a), "Move A should be in Slot 0");
        assert_eq!(position.killers[ply][1], None, "Slot 1 should still be empty");

        // --- TEST 2: Store Second Killer (Shift) ---
        // Simulate beta cutoff with Move B
        store_killer(&mut position, move_b); 

        assert_eq!(position.killers[ply][0], Some(move_b), "Move B should be new Slot 0");
        assert_eq!(position.killers[ply][1], Some(move_a), "Move A should have shifted to Slot 1");

        // --- TEST 3: Duplicate Prevention ---
        // Simulate beta cutoff with Move B AGAIN
        store_killer(&mut position, move_b); 

        assert_eq!(position.killers[ply][0], Some(move_b), "Slot 0 should stay Move B");
        assert_eq!(position.killers[ply][1], Some(move_a), "Slot 1 should NOT become Move B");
    }
}