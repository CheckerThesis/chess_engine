use colored::Colorize;

use crate::{board::{Board, Undo, clear_bit, position_keys::{self, CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::{Color, Piece, PieceType}, fn_name, movegen::{attacks::square_attacked, captured, from_square, is_castling, is_double_push, is_en_passant, promoted, to_square}};

pub const CASTLE_PERMISSION: [u8; 64] = [
    13, 15, 15, 15, 12, 15, 15, 14, 
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    7,  15, 15, 15, 3,  15, 15, 11
];

impl Board {
    pub fn make_move(&mut self, mv: u32) -> bool {
        fn get_promoted_piece(piece_index: usize, side: Color) -> Piece {
            let piece_type = match piece_index {
                2 | 8 => PieceType::Knight,
                3 | 9 => PieceType::Bishop,
                4 | 10 => PieceType::Rook,
                5 | 11 => PieceType::Queen,
                _ => panic!("Invalid promotion index"),
            };
            
            Piece { piece_type: piece_type, color: side }
        }

        if cfg!(debug_assertions) { self.check_board(fn_name!()); }

        let from = from_square(mv);
        let to = to_square(mv);
        let side = self.side;

        // Handle castle
        if is_castling(mv) {
            if square_attacked(from, side, self) || square_attacked(to, side, self) { return false }

            let crossing = match to {
                6 => 5,   // White Kingside (g1 needs f1)
                2 => 3,   // White Queenside (c1 needs d1)
                62 => 61, // Black Kingside (g8 needs f8)
                58 => 59, // Black Queenside (c8 needs d8)
                _ => 0,
            };

            if square_attacked(crossing, side, self) { return false }

            match to {
                6 => self.move_piece(7, 5), // White Kingside => Rook H1 to F1
                2 => self.move_piece(0, 3), // White Queenside => Rook A1 to D1
                62 => self.move_piece(63, 61), // Black Kingside => Rook H8 to F8
                58 => self.move_piece(56, 59), // Black Queenside => Rook A8 to D8
                _ => eprintln!("{}", "make_move: Invalid castle move".red()),
            }
        }

        // Store history before modifying
        self.history[self.history_ply] = Undo {
            mv: mv,
            castle_permission: self.castle_permission,
            en_passant: self.en_passant.unwrap_or(64),
            fifty_move: self.fifty_move,
            position_key: self.position_key,
        };

        // Handle enpassant
        if is_en_passant(mv) {
            // `to` is diagonal, +-8 gets the square behind/infront
            if side == Color::White { self.clear_piece(to - 8); }
            else { self.clear_piece(to + 8); }
        }

        // Update hashes
        self.hash_en_passant();
        self.en_passant = None;

        let mask = CASTLE_PERMISSION[from] & CASTLE_PERMISSION[to];
        let old_permission = self.castle_permission;
        let new_permission = old_permission & mask;

        if old_permission != new_permission {
            let xor_diff = CASTLE_KEYS[old_permission as usize] ^ CASTLE_KEYS[new_permission as usize];
            self.position_key ^= xor_diff;
            self.castle_permission = new_permission;
        }

        self.fifty_move += 1;

        // Move piece
        let captured = captured(mv);
        if captured != 0 {
            self.clear_piece(to);
            self.fifty_move = 0;
        }

        self.history_ply += 1;
        self.ply += 1;

        if let Some(pawn @ Piece { piece_type: PieceType::Pawn, .. }) = self.pieces[from] {
            self.fifty_move = 0;

            if is_double_push(mv) {
                match side {
                    Color::White => self.en_passant = Some(to - 8),
                    Color::Black => self.en_passant = Some(to + 8),
                    _ => eprintln!("{}", "make_move: Invalid color in double push".red()),
                }
                self.hash_en_passant();
            }
        }

        let promoted = promoted(mv);
        if promoted != 0 {
            self.clear_piece(from);
            self.add_piece(to, get_promoted_piece(promoted, side));
        } 
        else { self.move_piece(from, to); }

        self.side = self.side.opposite();
        self.hash_side();

        if cfg!(debug_assertions) { self.check_board(fn_name!()); }

        // let king_side = self.side.opposite();
        // let king_square = self.bitboards[PieceType::King.bb_index(king_side)].trailing_zeros() as usize;
        // if square_attacked(king_square, king_side, self) {
        //     // take_move
        //     return false;
        // }

        if let Some(Piece { piece_type: PieceType::King, .. }) = self.pieces[to] {
            if square_attacked(to, self.side.opposite(), self) {
                // take_move
                return false
            }
        }

        true
    }

    // fn take_move(&mut self) {
    //     if cfg!(debug_assertions) { self.check_board(fn_name!()); }

    //     self.history_ply -= 1;
    //     self.ply -= 1;

    //     let mv = self.history[self.history_ply].mv;
    //     let from = from_square(mv);
    //     let to = to_square(mv);

    //     if let Some(en_passant_square) = self.en_passant { self.hash_en_passant(); }
    //     self.hash_castle();

    //     self.castle_permission = self.history[self.history_ply].castle_permission;
    //     self.fifty_move = self.history[self.history_ply].fifty_move;
    //     self.en_passant = Some(self.history[self.history_ply].en_passant);


    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::{MoveList, MoveFlag};
    use crate::defs::{Castling, Color, Piece, PieceType};

    pub const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    
    // Castling
    pub const WHITE_KINGSIDE: &str = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    pub const BLACK_KINGSIDE: &str = "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1";
    pub const WHITE_IN_CHECK_CASTLE: &str = "4k2r/8/8/8/8/8/8/Rr2K2R w KQ - 0 1";

    // En Passant
    pub const EN_PASSANT_W: &str = "8/8/8/3pP3/8/8/8/8 w - d6 0 1";
    pub const EN_PASSANT_B: &str = "8/8/8/8/3pP3/8/8/8 b - e3 0 1";

    // Promotion
    pub const PROMOTION_W: &str = "8/4P3/8/8/8/8/8/8 w - - 0 1";
    pub const PROMOTION_B: &str = "8/8/8/8/8/8/4p3/8 b - - 0 1";

    // Check
    pub const KING_INTO_CHECK: &str = "k7/8/8/8/8/8/4r3/4K3 w - - 0 1";

    #[test]
    fn test_make_move_white_castle_kingside() {
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
        let e1: usize = 4;
        let g1: usize = 6;
        let h1: usize = 7;
        let f1: usize = 5;

        let mut position: Board = Board::new(WHITE_IN_CHECK_CASTLE);
        let old_key = position.position_key;

        let mv = MoveList::move_builder(e1, g1, 0, MoveFlag::CASTLE, 0);

        assert_eq!(position.make_move(mv), false);

        // King didn't move
        // assert!(position.pieces[e1] == );
        // assert_eq!(position.pieces[g1], Some(Piece { piece_type: PieceType::King, color: Color::White }));
        
        // // Rook moved automatically
        // assert!(position.pieces[h1].is_none());
        // assert_eq!(position.pieces[f1], Some(Piece { piece_type: PieceType::Rook, color: Color::White }));

        // assert_eq!(position.side, Color::Black);
        // assert_ne!(position.position_key, old_key);
        
        // let white_rights_mask = (Castling::WhiteKingCastle as u8) | (Castling::WhiteQueenCastle as u8);
        // assert_eq!(position.castle_permission & white_rights_mask, 0); 
        
        position.check_board(fn_name!());
    }
    
    #[test]
    fn test_make_move_white_en_passant() {
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
        let e4: usize = 28; // Black Pawn
        let d3: usize = 19; // Target
        let d4: usize = 27; // Victim (White Pawn)

        let mut position: Board = Board::new(EN_PASSANT_B);

        // e4 -> d3
        let mv = MoveList::move_builder(
            e4, d3, 0, MoveFlag::EN_PASSANT, 0
        );

        assert!(position.make_move(mv));

        assert!(position.pieces[e4].is_none());
        assert_eq!(position.pieces[d3], Some(Piece { piece_type: PieceType::Pawn, color: Color::Black }));
        assert!(position.pieces[d4].is_none()); // White pawn gone

        assert_eq!(position.side, Color::White);

        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_white_promotion() {
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
        let e2: usize = 12;
        let e1: usize = 4;

        let mut position: Board = Board::new(PROMOTION_B);

        // Promote to Black Queen (Index 11 based on your logic)
        let mv = MoveList::move_builder(
            e2, e1, 0, 0, 11
        );

        assert!(position.make_move(mv));

        assert!(position.pieces[e2].is_none());
        assert_eq!(position.pieces[e1], Some(Piece { piece_type: PieceType::Queen, color: Color::Black }));

        assert_eq!(position.side, Color::White);

        position.check_board(fn_name!());
    }

    #[test]
    fn test_make_move_fail_illegal() {
        let e1: usize = 4; // King
        let f2: usize = 13; // Square attacked by Rook on e2

        let mut position: Board = Board::new(KING_INTO_CHECK);
        let old_side = position.side;
        println!("{position}");

        // Move King E1 -> F2 (Illegal because e2 Rook attacks rank 2)
        let mv = MoveList::move_builder(
            e1, f2, 0, 0, 0
        );

        // Expect make_move to return false
        assert!(!position.make_move(mv));

        // Ensure state reverted (or stayed same)
        assert_eq!(position.side, old_side);
        
        // Ensure pieces didn't move
        assert_eq!(position.pieces[e1], Some(Piece { piece_type: PieceType::King, color: Color::White }));
        assert!(position.pieces[f2].is_none());
        
        position.check_board(fn_name!());
    }
}