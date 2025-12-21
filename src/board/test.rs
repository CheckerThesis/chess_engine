use crate::{board::{Board, Undo, clear_bit, position_keys::{self, CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::{Color, Piece, PieceType}, fn_name, movegen::{attacks::square_attacked}};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::print_bitboard;
    use crate::movegen::{MOVE_FLAG_CASTLE, MOVE_FLAG_EN_PASSANT, MOVE_FLAG_PAWN_START, Move, MoveList};
    use crate::defs::{BLACK_KING_CASTLE, BLACK_QUEEN_CASTLE, Color, Piece, PieceType, WHITE_KING_CASTLE, WHITE_QUEEN_CASTLE};
    
    #[test]
    fn mm_make_move_white_castle_kingside() {
        const WHITE_KINGSIDE: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let e1: usize = 4;
        let g1: usize = 6;
        let h1: usize = 7;
        let f1: usize = 5;

        let mut position: Board = Board::new(WHITE_KINGSIDE);
        let old_key = position.position_key;

        let mv = Move::new(e1, g1, 0, MOVE_FLAG_CASTLE, 0);

        assert!(position.make_move(mv));

        // King moved
        assert!(position.pieces[e1] == Piece::NONE);
        assert_eq!(position.pieces[g1], Piece::WHITE_KING);
        
        // Rook moved automatically
        assert!(position.pieces[h1] == Piece::NONE);
        assert_eq!(position.pieces[f1], Piece::WHITE_ROOK);

        assert_eq!(position.side, Color::BLACK);
        assert_ne!(position.position_key, old_key);
        
        let white_rights_mask = WHITE_KING_CASTLE | WHITE_QUEEN_CASTLE;
        assert_eq!(position.castle_permission & white_rights_mask, 0); 
        
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_black_castle_kingside() {
        const BLACK_KINGSIDE: &str = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
        let e8: usize = 60;
        let g8: usize = 62;
        let h8: usize = 63;
        let f8: usize = 61;

        let mut position: Board = Board::new(BLACK_KINGSIDE);
        let old_key = position.position_key;

        let mv = Move::new(e8, g8, 0, MOVE_FLAG_CASTLE, 0);

        assert!(position.make_move(mv));

        // King moved
        assert!(position.pieces[e8] == Piece::NONE);
        assert_eq!(position.pieces[g8], Piece::BLACK_KING);
        
        // Rook moved automatically
        assert!(position.pieces[h8] == Piece::NONE);
        assert_eq!(position.pieces[f8], Piece::BLACK_ROOK);

        assert_eq!(position.side, Color::WHITE);
        assert_ne!(position.position_key, old_key);

        let black_rights_mask = BLACK_KING_CASTLE | BLACK_QUEEN_CASTLE;
        assert_eq!(position.castle_permission & black_rights_mask, 0);

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_in_check_castle() {
        const WHITE_IN_CHECK_CASTLE: &str = "4k2r/8/8/8/8/8/8/Rr2K2R w KQ - 0 1";
        let e1: usize = 4;
        let g1: usize = 6;
        let h1: usize = 7;
        let f1: usize = 5;

        let mut position: Board = Board::new(WHITE_IN_CHECK_CASTLE);
        let old_key = position.position_key;
        let old_side = position.side;

        let mv = Move::new(e1, g1, 0, MOVE_FLAG_CASTLE, 0);

        assert_eq!(position.make_move(mv), false);

        assert_eq!(position.side, old_side);
        assert_eq!(position.position_key, old_key);

        // King is back at E1
        assert_eq!(position.pieces[e1], Piece::WHITE_KING);
        assert!(position.pieces[g1] == Piece::NONE);
        
        // Rook is back at H1
        assert_eq!(position.pieces[h1], Piece::WHITE_ROOK);
        assert!(position.pieces[f1] == Piece::NONE);
        
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }
    
    #[test]
    fn mm_make_move_white_en_passant() {
        const EN_PASSANT_W: &str = "7k/8/8/3pP3/8/8/8/K7 w - d6 0 1";
        let e5: usize = 36;
        let d6: usize = 43; // Target
        let d5: usize = 35; // Victim pawn location

        let mut position: Board = Board::new(EN_PASSANT_W);
        let old_key = position.position_key;

        // Capture bits are usually 0 for EP in move_builder because the target square is empty
        // The flag tells make_move to handle the special capture logic.
        let mv = Move::new(
            e5, d6, 0, MOVE_FLAG_EN_PASSANT, 0
        );

        assert!(position.make_move(mv));

        // Topology
        assert!(position.pieces[e5] == Piece::NONE); // Start empty
        assert_eq!(position.pieces[d6], Piece::WHITE_PAWN); // End occupied
        assert!(position.pieces[d5] == Piece::NONE); // Victim captured!

        assert_eq!(position.side, Color::BLACK);
        assert_ne!(position.position_key, old_key);

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_black_en_passant() {
        const EN_PASSANT_B: &str = "7k/8/8/8/3pP3/8/8/K7 b - e3 0 1";
        let e4: usize = 28; // Black Pawn
        let e3: usize = 20; // Target
        let d4: usize = 27; // Victim (White Pawn)

        let mut position: Board = Board::new(EN_PASSANT_B);

        let mv = Move::new(
            d4, e3, 0, MOVE_FLAG_EN_PASSANT, 0
        );

        assert!(position.make_move(mv));

        assert!(position.pieces[e4] == Piece::NONE);
        assert_eq!(position.pieces[e3], Piece::BLACK_PAWN);
        assert!(position.pieces[d4] == Piece::NONE); // White pawn gone

        assert_eq!(position.side, Color::WHITE);

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_white_promotion() {
        const PROMOTION_W: &str = "7k/4P3/8/8/8/8/8/K7 w - - 0 1";
        let e7: usize = 52;
        let e8: usize = 60;

        let mut position: Board = Board::new(PROMOTION_W);

        // Promote to Queen (Index 5 based on your logic)
        let mv = Move::new(
            e7, e8, 0, 0, Piece::WHITE_QUEEN.index()
        );

        assert!(position.make_move(mv));

        assert!(position.pieces[e7] == Piece::NONE);
        assert_eq!(position.pieces[e8], Piece::WHITE_QUEEN);
        
        assert_eq!(position.side, Color::BLACK);

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_black_promotion() {
        const PROMOTION_B: &str = "7k/8/8/8/8/8/4p3/K7 b - - 0 1";
        let e2: usize = 12;
        let e1: usize = 4;

        let mut position: Board = Board::new(PROMOTION_B);

        // Promote to Black Queen (Index 11 based on your logic)
        let mv = Move::new(
            e2, e1, 0, 0, Piece::BLACK_QUEEN.index()
        );

        assert!(position.make_move(mv));

        assert!(position.pieces[e2] == Piece::NONE);
        assert_eq!(position.pieces[e1], Piece::BLACK_QUEEN);

        assert_eq!(position.side, Color::WHITE);

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_fail_illegal() {
        pub const KING_INTO_CHECK: &str = "k7/8/8/8/8/8/4r3/4K3 w - - 0 1";
        let e1: usize = 4; // King
        let f2: usize = 13; // Square attacked by Rook on e2

        let mut position: Board = Board::new(KING_INTO_CHECK);
        let old_side = position.side;

        // Move King E1 -> F2 (Illegal because e2 Rook attacks rank 2)
        let mv = Move::new(
            e1, f2, 0, 0, 0
        );

        // Expect make_move to return false
        assert_eq!(position.make_move(mv), false);

        // --- Verify Reversal ---
        
        // Side restored
        assert_eq!(position.side, old_side);
        
        // Pieces didn't move
        assert_eq!(position.pieces[e1], Piece::WHITE_KING);
        assert!(position.pieces[f2] == Piece::NONE);
        
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_fail_pinned_piece() {
        // Setup: White King on e1, White Rook on e2, Black Rook on e8.
        // The White Rook is pinned to the King. Moving it exposes the King.
        const PINNED_ROOK: &str = "1k2r3/8/8/8/8/8/4R3/4K3 w - - 0 1";
        
        let e2: usize = 12; // White Rook
        let h2: usize = 15; // Target square
        
        let mut position: Board = Board::new(PINNED_ROOK);
        let old_side = position.side;
        
        // Move Rook e2 -> h2 (Illegal: Exposes King to e8 Rook)
        let mv = Move::new(
            e2, h2, 0, 0, 0
        );

        assert_eq!(position.make_move(mv), false, "Should return false when moving a pinned piece");

        // --- Verify Reversal ---

        // Side restored
        assert_eq!(position.side, old_side);

        // Rook is still at e2
        assert_eq!(position.pieces[e2], Piece::WHITE_ROOK);
        // Target square empty
        assert!(position.pieces[h2] == Piece::NONE);

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_fail_en_passant_discovered_check() {
        const EP_DISCOVERED_CHECK: &str = "8/8/8/K2pP2r/8/8/8/k7 w - d6 0 1";

        let e5: usize = 36;
        let d6: usize = 43; // Target
        let d5: usize = 35; // Victim

        let mut position: Board = Board::new(EP_DISCOVERED_CHECK);
        let old_side = position.side;

        // Move e5xd6 (En Passant)
        // If this moves happens, the 5th rank clears (e5 and d5 gone), exposing Ka5 to Rh5.
        let mv = Move::new(
            e5, d6, 0, MOVE_FLAG_EN_PASSANT, 0
        );

        assert_eq!(position.make_move(mv), false, "Should return false if EP capture reveals check");

        assert_eq!(position.side, old_side);
        assert_eq!(position.pieces[e5], Piece::WHITE_PAWN); // white Pawn back at start
        assert_eq!(position.pieces[d5], Piece::BLACK_PAWN); // victim (Black Pawn) restored at d5
        assert!(position.pieces[d6] == Piece::NONE); // target square empty

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_white_capture_promotion() {
        // White Pawn on b7, Black Rook on a8.
        // Move: b7xa8 (Capture Rook) -> Promote to Queen
        const CAPTURE_PROMO_W: &str = "r3k3/1P6/8/8/8/8/8/4K3 w - - 0 1";
        
        let b7: usize = 49;
        let a8: usize = 56;

        let mut position: Board = Board::new(CAPTURE_PROMO_W);
        let old_key = position.position_key;

        // Move builder usually takes: from, to, captured_piece, flag, promoted_piece
        let mv = Move::new(
            b7, a8, Piece::BLACK_ROOK.index(), 0, Piece::WHITE_QUEEN.index()
        );

        assert!(position.make_move(mv));

        // 1. Check Source is empty
        assert!(position.pieces[b7] == Piece::NONE);

        // 2. Check Target has Promoted Piece (White Queen), not the captured Rook
        assert_eq!(position.pieces[a8], Piece::WHITE_QUEEN);
        
        // 3. Check Side Flipped
        assert_eq!(position.side, Color::BLACK);
        assert_ne!(position.position_key, old_key);

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_black_capture_promotion() {
        // Black Pawn on g2, White Rook on h1.
        // Move: g2xh1 (Capture Rook) -> Promote to Knight (Underpromotion)
        const CAPTURE_PROMO_B: &str = "4k3/8/8/8/8/8/6p1/4K2R b K - 0 1";

        let g2: usize = 14;
        let h1: usize = 7;

        let mut position: Board = Board::new(CAPTURE_PROMO_B);
        let old_key = position.position_key;

        let mv = Move::new(
            g2, h1, Piece::WHITE_ROOK.index(), 0, Piece::BLACK_KNIGHT.index()
        );

        assert!(position.make_move(mv));

        // 1. Check Source is empty
        assert!(position.pieces[g2] == Piece::NONE);

        // 2. Check Target has Promoted Piece (Black Knight)
        assert_eq!(position.pieces[h1], Piece::BLACK_KNIGHT);

        // 3. Check Side Flipped
        assert_eq!(position.side, Color::WHITE);
        assert_ne!(position.position_key, old_key);

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }

    #[test]
    fn mm_make_move_double_push_sets_ep() {
        // Standard start position where e2 is a white pawn
        const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        let mut position = Board::new(START_POS);

        let e2: usize = 12;
        let e4: usize = 28;
        let e3: usize = 20; // The resulting en passant target square (behind the pawn)

        // Pre-check: EP should be None at start
        assert_eq!(position.en_passant, None);

        // Move: e2 -> e4 (Double Push)
        let mv = Move::new(
            e2, e4, 0, MOVE_FLAG_PAWN_START, 0
        );

        assert!(position.make_move(mv));

        // 1. Verify Topology
        assert!(position.pieces[e2] == Piece::NONE);
        assert_eq!(position.pieces[e4], Piece::WHITE_PAWN);

        // 2. Verify En Passant Square is set to e3
        assert_eq!(position.en_passant, Some(e3 as u8));

        // 3. Verify Zobrist Key changed
        // (Optional: You could also verify the key includes the specific EP hash if you have helpers for that)
        
        // --- Verify Reversal (take_move) ---
        position.take_move();

        // 4. Verify EP is reset to None
        assert_eq!(position.en_passant, None);
        
        // 5. Verify Pawn is back
        assert_eq!(position.pieces[e2], Piece::WHITE_PAWN);
        assert!(position.pieces[e4] == Piece::NONE);

        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
    }
}