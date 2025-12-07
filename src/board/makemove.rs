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
        if captured != 0 && !is_en_passant(mv) {
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

        let king_side = self.side.opposite();
        let king_square = self.bitboards[PieceType::King.bb_index(king_side)].trailing_zeros() as usize;
        if square_attacked(king_square, king_side, self) {
            self.take_move();
            return false;
        }

        true
    }

    pub fn take_move(&mut self) {
        if cfg!(debug_assertions) { self.check_board(fn_name!()); }

        self.history_ply -= 1;
        self.ply -= 1;

        let undo = self.history[self.history_ply];
        let mv = undo.mv;
        let from = from_square(mv);
        let to = to_square(mv);

        // 1. Flip side back to the color that made the move
        self.side = self.side.opposite();
        self.hash_side();

        // 2. Handle En Passant
        // We must manually put the captured pawn back. 
        // Note: The captured pawn is NOT at 'to', but at 'to +/- 8'.
        if is_en_passant(mv) {
            self.move_piece(to, from);

            let captured_square = if self.side == Color::White { to - 8 } else { to + 8 };
            self.add_piece(captured_square, Piece { piece_type: PieceType::Pawn, color: self.side.opposite() });
        }
        // 3. Handle Castling
        // Move King back (to -> from) and Rook back to original corner
        else if is_castling(mv) {
            self.move_piece(to, from);

            match to {
                6 => self.move_piece(5, 7),   // White Kingside: Rook F1 -> H1
                2 => self.move_piece(3, 0),   // White Queenside: Rook D1 -> A1
                62 => self.move_piece(61, 63),// Black Kingside: Rook F8 -> H8
                58 => self.move_piece(59, 56),// Black Queenside: Rook D8 -> A8
                _ => eprintln!("{}", "take_move: Invalid castle move".red()),
            }
        }
        // 4. Standard Moves and Promotions
        else {
            let promoted = promoted(mv);
            if promoted != 0 {
                // If it was a promotion, the piece at 'to' is a Queen/Rook/etc.
                // We must remove it and put a Pawn back at 'from'.
                self.clear_piece(to);
                self.add_piece(from, Piece { piece_type: PieceType::Pawn, color: self.side });
            } else {
                // Standard move reversal
                self.move_piece(to, from);
            }

            // 5. Restore Captured Piece
            // If a piece was captured (and not En Passant), put it back at 'to'.
            let captured_val = captured(mv);
            if captured_val != 0 {
                let captured_piece_type = match captured_val {
                    1 => PieceType::Pawn,
                    2 => PieceType::Knight,
                    3 => PieceType::Bishop,
                    4 => PieceType::Rook,
                    5 => PieceType::Queen,
                    6 => PieceType::King,
                    _ => panic!("take_move: Invalid captured piece type"),
                };
                self.add_piece(to, Piece { piece_type: captured_piece_type, color: self.side.opposite() });
            }
        }

        // 6. Restore Board State from History
        self.castle_permission = undo.castle_permission;
        self.fifty_move = undo.fifty_move;
        self.position_key = undo.position_key;

        // Restore En Passant square (64 indicates None)
        self.en_passant = if undo.en_passant == 64 { None } else { Some(undo.en_passant) };

        if cfg!(debug_assertions) { self.check_board(fn_name!()); }
    }
}
