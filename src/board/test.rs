use crate::{board::{Board, Undo, clear_bit, position_keys::{self, CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::{Color, Piece, PieceType}, fn_name, movegen::{attacks::square_attacked, captured, from_square, is_castling, is_double_push, is_en_passant, promoted, to_square}};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::{MoveList, MoveFlag};
    use crate::defs::{Castling, Color, Piece, PieceType};
    
    #[test]
    fn test_make_move_white_castle_kingside() {
        const WHITE_KINGSIDE: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let e1: usize = 4;
        let g1: usize = 6;
        let h1: usize = 7;
        let f1: usize = 5;

        let mut position: Board = Board::new(WHITE_KINGSIDE);
        let old_key = position.position_key;

        let mv = MoveList::move_builder(e1, g1, 0, MoveFlag::CASTLE, 0);

        assert!(position.make_move(mv));

        // King moved
        assert!(position.pieces[e1].is_none());
        assert_eq!(position.pieces[g1], Some(Piece { piece_type: PieceType::King, color: Color::White }));
        
        // Rook moved automatically
        assert!(position.pieces[h1].is_none());
        assert_eq!(position.pieces[f1], Some(Piece { piece_type: PieceType::Rook, color: Color::White }));

        assert_eq!(position.side, Color::Black);
        assert_ne!(position.position_key, old_key);
        
        let white_rights_mask = (Castling::WhiteKingCastle as u8) | (Castling::WhiteQueenCastle as u8);
        assert_eq!(position.castle_permission & white_rights_mask, 0); 
        
        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_black_castle_kingside() {
        const BLACK_KINGSIDE: &str = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
        let e8: usize = 60;
        let g8: usize = 62;
        let h8: usize = 63;
        let f8: usize = 61;

        let mut position: Board = Board::new(BLACK_KINGSIDE);
        let old_key = position.position_key;

        let mv = MoveList::move_builder(
            e8, g8, 0, MoveFlag::CASTLE, 0
        );

        assert!(position.make_move(mv));

        // King moved
        assert!(position.pieces[e8].is_none());
        assert_eq!(position.pieces[g8], Some(Piece { piece_type: PieceType::King, color: Color::Black }));
        
        // Rook moved automatically
        assert!(position.pieces[h8].is_none());
        assert_eq!(position.pieces[f8], Some(Piece { piece_type: PieceType::Rook, color: Color::Black }));

        assert_eq!(position.side, Color::White);
        assert_ne!(position.position_key, old_key);

        let black_rights_mask = (Castling::BlackKingCastle as u8) | (Castling::BlackQueenCastle as u8);
        assert_eq!(position.castle_permission & black_rights_mask, 0);

        position.check_board(fn_name!());
    }

    #[test]
    fn test_in_check_castle() {
        const WHITE_IN_CHECK_CASTLE: &str = "4k2r/8/8/8/8/8/8/Rr2K2R w KQ - 0 1";
        let e1: usize = 4;
        let g1: usize = 6;
        let h1: usize = 7;
        let f1: usize = 5;

        let mut position: Board = Board::new(WHITE_IN_CHECK_CASTLE);
        let old_key = position.position_key;
        let old_side = position.side;

        let mv = MoveList::move_builder(e1, g1, 0, MoveFlag::CASTLE, 0);

        assert_eq!(position.make_move(mv), false);

        assert_eq!(position.side, old_side);
        assert_eq!(position.position_key, old_key);

        // King is back at E1
        assert_eq!(position.pieces[e1], Some(Piece { piece_type: PieceType::King, color: Color::White }));
        assert!(position.pieces[g1].is_none());
        
        // Rook is back at H1
        assert_eq!(position.pieces[h1], Some(Piece { piece_type: PieceType::Rook, color: Color::White }));
        assert!(position.pieces[f1].is_none());
        
        position.check_board(fn_name!());
    }
    
    #[test]
    fn test_make_move_white_en_passant() {
        const EN_PASSANT_W: &str = "7k/8/8/3pP3/8/8/8/K7 w - d6 0 1";
        let e5: usize = 36;
        let d6: usize = 43; // Target
        let d5: usize = 35; // Victim pawn location

        let mut position: Board = Board::new(EN_PASSANT_W);
        let old_key = position.position_key;

        // Capture bits are usually 0 for EP in move_builder because the target square is empty
        // The flag tells make_move to handle the special capture logic.
        let mv = MoveList::move_builder(
            e5, d6, 0, MoveFlag::EN_PASSANT, 0
        );

        assert!(position.make_move(mv));

        // Topology
        assert!(position.pieces[e5].is_none()); // Start empty
        assert_eq!(position.pieces[d6], Some(Piece { piece_type: PieceType::Pawn, color: Color::White })); // End occupied
        assert!(position.pieces[d5].is_none()); // Victim captured!

        assert_eq!(position.side, Color::Black);
        assert_ne!(position.position_key, old_key);

        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_black_en_passant() {
        const EN_PASSANT_B: &str = "7k/8/8/8/3pP3/8/8/K7 b - e3 0 1";
        let e4: usize = 28; // Black Pawn
        let e3: usize = 20; // Target
        let d4: usize = 27; // Victim (White Pawn)

        let mut position: Board = Board::new(EN_PASSANT_B);

        let mv = MoveList::move_builder(
            d4, e3, 0, MoveFlag::EN_PASSANT, 0
        );

        assert!(position.make_move(mv));

        assert!(position.pieces[e4].is_none());
        assert_eq!(position.pieces[e3], Some(Piece { piece_type: PieceType::Pawn, color: Color::Black }));
        assert!(position.pieces[d4].is_none()); // White pawn gone

        assert_eq!(position.side, Color::White);

        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_white_promotion() {
        const PROMOTION_W: &str = "7k/4P3/8/8/8/8/8/K7 w - - 0 1";
        let e7: usize = 52;
        let e8: usize = 60;

        let mut position: Board = Board::new(PROMOTION_W);

        // Promote to Queen (Index 5 based on your logic)
        let mv = MoveList::move_builder(
            e7, e8, 0, 0, 5 
        );

        assert!(position.make_move(mv));

        assert!(position.pieces[e7].is_none());
        assert_eq!(position.pieces[e8], Some(Piece { piece_type: PieceType::Queen, color: Color::White }));
        
        assert_eq!(position.side, Color::Black);

        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_black_promotion() {
        const PROMOTION_B: &str = "7k/8/8/8/8/8/4p3/K7 b - - 0 1";
        let e2: usize = 12;
        let e1: usize = 4;

        let mut position: Board = Board::new(PROMOTION_B);
        println!("{position}");

        // Promote to Black Queen (Index 11 based on your logic)
        let mv = MoveList::move_builder(
            e2, e1, 0, 0, 11
        );

        assert!(position.make_move(mv));
        println!("{position}");

        assert!(position.pieces[e2].is_none());
        assert_eq!(position.pieces[e1], Some(Piece { piece_type: PieceType::Queen, color: Color::Black }));

        assert_eq!(position.side, Color::White);

        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_fail_illegal() {
        pub const KING_INTO_CHECK: &str = "k7/8/8/8/8/8/4r3/4K3 w - - 0 1";
        let e1: usize = 4; // King
        let f2: usize = 13; // Square attacked by Rook on e2

        let mut position: Board = Board::new(KING_INTO_CHECK);
        let old_side = position.side;

        // Move King E1 -> F2 (Illegal because e2 Rook attacks rank 2)
        let mv = MoveList::move_builder(
            e1, f2, 0, 0, 0
        );

        // Expect make_move to return false
        assert_eq!(position.make_move(mv), false);

        // --- Verify Reversal ---
        
        // Side restored
        assert_eq!(position.side, old_side);
        
        // Pieces didn't move
        assert_eq!(position.pieces[e1], Some(Piece { piece_type: PieceType::King, color: Color::White }));
        assert!(position.pieces[f2].is_none());
        
        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_fail_pinned_piece() {
        // Setup: White King on e1, White Rook on e2, Black Rook on e8.
        // The White Rook is pinned to the King. Moving it exposes the King.
        const PINNED_ROOK: &str = "1k2r3/8/8/8/8/8/4R3/4K3 w - - 0 1";
        
        let e2: usize = 12; // White Rook
        let h2: usize = 15; // Target square
        
        let mut position: Board = Board::new(PINNED_ROOK);
        let old_side = position.side;
        
        // Move Rook e2 -> h2 (Illegal: Exposes King to e8 Rook)
        let mv = MoveList::move_builder(
            e2, h2, 0, 0, 0
        );

        assert_eq!(position.make_move(mv), false, "Should return false when moving a pinned piece");

        // --- Verify Reversal ---

        // Side restored
        assert_eq!(position.side, old_side);

        // Rook is still at e2
        assert_eq!(position.pieces[e2], Some(Piece { piece_type: PieceType::Rook, color: Color::White }));
        // Target square empty
        assert!(position.pieces[h2].is_none());

        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_fail_en_passant_discovered_check() {
        const EP_DISCOVERED_CHECK: &str = "8/8/8/K2pP2r/8/8/8/8 w - d6 0 1";

        let e5: usize = 36;
        let d6: usize = 43; // Target
        let d5: usize = 35; // Victim

        let mut position: Board = Board::new(EP_DISCOVERED_CHECK);
        let old_side = position.side;

        // Move e5xd6 (En Passant)
        // If this moves happens, the 5th rank clears (e5 and d5 gone), exposing Ka5 to Rh5.
        let mv = MoveList::move_builder(
            e5, d6, 0, MoveFlag::EN_PASSANT, 0
        );

        assert_eq!(position.make_move(mv), false, "Should return false if EP capture reveals check");

        // --- Verify Reversal ---

        // Side restored
        assert_eq!(position.side, old_side);

        // White Pawn back at start
        assert_eq!(position.pieces[e5], Some(Piece { piece_type: PieceType::Pawn, color: Color::White }));
        
        // Victim (Black Pawn) restored at d5 (Critical check for EP reversal)
        assert_eq!(position.pieces[d5], Some(Piece { piece_type: PieceType::Pawn, color: Color::Black }));
        
        // Target square empty
        assert!(position.pieces[d6].is_none());

        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_white_capture_promotion() {
        // White Pawn on b7, Black Rook on a8.
        // Move: b7xa8 (Capture Rook) -> Promote to Queen
        const CAPTURE_PROMO_W: &str = "r3k3/1P6/8/8/8/8/8/4K3 w - - 0 1";
        
        let b7: usize = 49;
        let a8: usize = 56;
        let rook_type = 4; // Integer representation for Rook capture
        let promoted_queen = 5; // Integer for White Queen promotion

        let mut position: Board = Board::new(CAPTURE_PROMO_W);
        let old_key = position.position_key;

        // Move builder usually takes: from, to, captured_piece, flag, promoted_piece
        let mv = MoveList::move_builder(
            b7, a8, rook_type, 0, promoted_queen
        );

        assert!(position.make_move(mv));

        // 1. Check Source is empty
        assert!(position.pieces[b7].is_none());

        // 2. Check Target has Promoted Piece (White Queen), not the captured Rook
        assert_eq!(position.pieces[a8], Some(Piece { piece_type: PieceType::Queen, color: Color::White }));
        
        // 3. Check Side Flipped
        assert_eq!(position.side, Color::Black);
        assert_ne!(position.position_key, old_key);

        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_black_capture_promotion() {
        // Black Pawn on g2, White Rook on h1.
        // Move: g2xh1 (Capture Rook) -> Promote to Knight (Underpromotion)
        const CAPTURE_PROMO_B: &str = "4k3/8/8/8/8/8/6p1/4K2R b K - 0 1";

        let g2: usize = 14;
        let h1: usize = 7;
        let rook_type = 4; // Captured White Rook
        let promoted_knight = 8; // Integer for Black Knight promotion (based on your helper logic)

        let mut position: Board = Board::new(CAPTURE_PROMO_B);
        let old_key = position.position_key;

        let mv = MoveList::move_builder(
            g2, h1, rook_type, 0, promoted_knight
        );

        assert!(position.make_move(mv));

        // 1. Check Source is empty
        assert!(position.pieces[g2].is_none());

        // 2. Check Target has Promoted Piece (Black Knight)
        assert_eq!(position.pieces[h1], Some(Piece { piece_type: PieceType::Knight, color: Color::Black }));

        // 3. Check Side Flipped
        assert_eq!(position.side, Color::White);
        assert_ne!(position.position_key, old_key);

        position.check_board(fn_name!());
    }
}