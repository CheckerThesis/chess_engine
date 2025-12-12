use colored::Colorize;

use crate::{board::{Board, Undo, clear_bit, position_keys::{self, CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::{Color, Piece, PieceType}, fn_name, movegen::{attacks::square_attacked, captured, from_square, is_castling, is_double_push, is_en_passant, promoted, to_square}, squares::squares::{A1, A8, C1, C8, D1, D8, F1, F8, G1, G8, H1, H8}};

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

static mut hi: usize = 0; 

impl Board {
    // pub fn make_move2(&mut self, mv: u32) -> bool {
    //     fn get_promoted_piece(piece_index: usize, side: Color) -> Piece {
    //         let piece_type = match piece_index {
    //             2 | 3 => PieceType::Knight,
    //             4 | 5 => PieceType::Bishop,
    //             6 | 7 => PieceType::Rook,
    //             8 | 9 => PieceType::Queen,
    //             _ => panic!("Invalid promotion index"),
    //         };
            
    //         Piece { piece_type: piece_type, color: side }
    //     }

    //     if cfg!(debug_assertions) { self.check_board(fn_name!()); }

    //     let from = from_square(mv);
    //     let to = to_square(mv);
    //     let side = self.side;

    //     // Store history before modifying
    //     self.history[self.history_ply] = Undo {
    //         mv: mv,
    //         castle_permission: self.castle_permission,
    //         en_passant: self.en_passant.unwrap_or(64),
    //         fifty_move: self.fifty_move,
    //         position_key: self.position_key,
    //     };

    //     self.history_ply += 1;
    //     self.ply += 1;

    //     // Handle castle
    //     if is_castling(mv) {
    //         if square_attacked(from, side, self) || 
    //         square_attacked(to, side, self) { return false }

    //         let crossing = match to {
    //             6 => 5,   // White Kingside (g1 needs f1)
    //             2 => 3,   // White Queenside (c1 needs d1)
    //             62 => 61, // Black Kingside (g8 needs f8)
    //             58 => 59, // Black Queenside (c8 needs d8)
    //             _ => 0,
    //         };

    //         if square_attacked(crossing, side, self) { return false }
        
    //         match to {
    //             6 => self.move_piece(7, 5), // White Kingside => Rook H1 to F1
    //             2 => self.move_piece(0, 3), // White Queenside => Rook A1 to D1
    //             62 => self.move_piece(63, 61), // Black Kingside => Rook H8 to F8
    //             58 => self.move_piece(56, 59), // Black Queenside => Rook A8 to D8
    //             _ => eprintln!("{}", "make_move: Invalid castle move".red()),
    //         }
    //     }

    //     // Handle enpassant
    //     if is_en_passant(mv) {
    //         // `to` is diagonal, +-8 gets the square behind/infront
    //         // if side == Color::White { self.clear_piece(to - 8); }
    //         // else { self.clear_piece(to + 8); }

    //         let capture_sq = if side == Color::White {
    //         if to < 8 { return false; } // Garbage move protection
    //             to - 8 
    //         } else {
    //             to + 8 
    //         };
    //         self.clear_piece(capture_sq);
    //     }

    //     // Update hashes
    //     self.hash_en_passant();
    //     self.en_passant = None;

    //     let mask = CASTLE_PERMISSION[from] & CASTLE_PERMISSION[to];
    //     let old_permission = self.castle_permission;
    //     let new_permission = old_permission & mask;

    //     if old_permission != new_permission {
    //         let xor_diff = CASTLE_KEYS[old_permission as usize] ^ CASTLE_KEYS[new_permission as usize];
    //         self.position_key ^= xor_diff;
    //         self.castle_permission = new_permission;
    //     }

    //     self.fifty_move += 1;

    //     // Move piece
    //     let captured = captured(mv);
    //     if captured != 0 && !is_en_passant(mv) {
    //         self.clear_piece(to);
    //         self.fifty_move = 0;
    //     }

    //     if let Some(pawn @ Piece { piece_type: PieceType::Pawn, .. }) = self.pieces[from] {
    //         self.fifty_move = 0;

    //         if is_double_push(mv) {
    //             match side {
    //                 Color::White => self.en_passant = Some(to - 8),
    //                 Color::Black => self.en_passant = Some(to + 8),
    //                 _ => eprintln!("{}", "make_move: Invalid color in double push".red()),
    //             }
    //             self.hash_en_passant();
    //         }
    //     }

    //     let promoted = promoted(mv);
    //     if promoted != 0 {
    //         self.clear_piece(from);
    //         self.add_piece(to, get_promoted_piece(promoted, side));
    //     } 
    //     else { self.move_piece(from, to); }

    //     self.side = self.side.opposite();
    //     self.hash_side();

    //     if cfg!(debug_assertions) { self.check_board(fn_name!()); }

    //     let king_side = self.side.opposite();
    //     let king_square = self.bitboards[PieceType::King.bb_index(king_side)].trailing_zeros() as usize;
    //     if square_attacked(king_square, king_side, self) {
    //         self.take_move();
    //         return false;
    //     }

    //     true
    // }
 
    pub fn make_move(&mut self, mv: u32) -> bool {
        fn get_promoted_piece(piece_index: usize, side: Color) -> Piece {
            let piece_type = match piece_index {
                2 | 3 => PieceType::Knight,
                4 | 5 => PieceType::Bishop,
                6 | 7 => PieceType::Rook,
                8 | 9 => PieceType::Queen,
                _ => panic!("Invalid promotion index"),
            };
            
            Piece { piece_type: piece_type, color: side }
        }
        let bean;
        unsafe { 
            hi += 1;
            bean = hi; 
        }
        if cfg!(debug_assertions) { 
            println!("{bean}");
            self.check_board(fn_name!()); 
        }

        if (bean == 11173) {
            let fen = self.get_fen();
            println!("{fen}");
            println!("{self}");
            self.print_bitboards(Some(&[
                Piece { piece_type: PieceType::Pawn, color: Color::White }, 
                Piece { piece_type: PieceType::Bishop, color: Color::White },
                Piece { piece_type: PieceType::Pawn, color: Color::Black },
            ]));
        }

        let from = from_square(mv);
        let to = to_square(mv);
        let side = self.side;

        self.history[self.history_ply].position_key = self.position_key;

        if is_en_passant(mv) {
            // `to` is diagonal, +-8 gets the square behind/infront
            if side == Color::White { self.clear_piece(to - 8); }
            else { self.clear_piece(to + 8); }

        } else if is_castling(mv) {
            if to == C1      { self.move_piece(A1, D1); }
            else if to == C8 { self.move_piece(A8, D8); }
            else if to == G1 { self.move_piece(H1, F1); }
            else if to == G8 { self.move_piece(H8, F8); }
            else { eprintln!("{}", "make_move: castle problem".red()); }
        }

        if self.en_passant.is_some() { self.hash_en_passant(); }
        self.hash_castle();

        self.history[self.history_ply].mv = mv;
        self.history[self.history_ply].fifty_move = self.fifty_move;
        self.history[self.history_ply].en_passant = self.en_passant;
        self.history[self.history_ply].castle_permission = self.castle_permission;

        self.castle_permission &= CASTLE_PERMISSION[from];
        self.castle_permission &= CASTLE_PERMISSION[to];
        self.en_passant = None;

        self.hash_castle();

        self.fifty_move += 1;

        let captured = captured(mv);
        if captured != 0 {
            self.clear_piece(to);
            self.fifty_move = 0;
        }

        self.history_ply += 1;
        self.ply += 1;

        if let Some(Piece { piece_type: PieceType::Pawn, .. }) = self.pieces[from] {
            self.fifty_move = 0;

            if is_double_push(mv) {
                if side == Color::White { self.en_passant = Some(from + 8); }
                else { self.en_passant = Some(from - 8); }
            }
            self.hash_en_passant();
        }

        self.move_piece(from, to);
        if (bean == 11173) {
            println!("{self}");
            self.print_bitboards(Some(&[
                Piece { piece_type: PieceType::Pawn, color: Color::White }, 
                Piece { piece_type: PieceType::Bishop, color: Color::White },
                Piece { piece_type: PieceType::Pawn, color: Color::Black },
            ]));
        }

        let promote_piece = promoted(mv);
        if promote_piece != 0 {
            self.clear_piece(to);
            let piece = Piece::MV_TO_PIECE[promote_piece];
            self.add_piece(to, piece);
        }

        self.side = self.side.opposite();
        self.hash_side();

        if cfg!(debug_assertions) { self.check_board(fn_name!()); }

        let king_square = self.bitboards[PieceType::King.bb_index(self.side)].trailing_zeros() as usize;
        if square_attacked(king_square, self.side, self) {
            self.take_move();
            return false
        }


        true
    }

    // pub fn take_move2(&mut self) {
    //     if cfg!(debug_assertions) { self.check_board(fn_name!()); }

    //     self.history_ply -= 1;
    //     self.ply -= 1;

    //     let undo = self.history[self.history_ply];
    //     let mv = undo.mv;
    //     let from = from_square(mv);
    //     let to = to_square(mv);

    //     // 1. Flip side back to the color that made the move
    //     self.side = self.side.opposite();
    //     self.hash_side();

    //     // 2. Handle En Passant
    //     // We must manually put the captured pawn back. 
    //     // Note: The captured pawn is NOT at 'to', but at 'to +/- 8'.
    //     if is_en_passant(mv) {
    //         self.move_piece(to, from);

    //         let captured_square = if self.side == Color::White { to.wrapping_sub(8) } else { to + 8 };
    //         self.add_piece(captured_square, Piece { piece_type: PieceType::Pawn, color: self.side.opposite() });
    //     }
    //     // 3. Handle Castling
    //     // Move King back (to -> from) and Rook back to original corner
    //     else if is_castling(mv) {
    //         self.move_piece(to, from);

    //         match to {
    //             6 => self.move_piece(5, 7),   // White Kingside: Rook F1 -> H1
    //             2 => self.move_piece(3, 0),   // White Queenside: Rook D1 -> A1
    //             62 => self.move_piece(61, 63),// Black Kingside: Rook F8 -> H8
    //             58 => self.move_piece(59, 56),// Black Queenside: Rook D8 -> A8
    //             _ => eprintln!("{}", "take_move: Invalid castle move".red()),
    //         }
    //     }
    //     // 4. Standard Moves and Promotions
    //     else {
    //         let promoted = promoted(mv);
    //         if promoted != 0 {
    //             // If it was a promotion, the piece at 'to' is a Queen/Rook/etc.
    //             // We must remove it and put a Pawn back at 'from'.
    //             self.clear_piece(to);
    //             self.add_piece(from, Piece { piece_type: PieceType::Pawn, color: self.side });
    //         } else {
    //             // Standard move reversal
    //             self.move_piece(to, from);
    //         }

    //         // 5. Restore Captured Piece
    //         // If a piece was captured (and not En Passant), put it back at 'to'.
    //         let captured_val = captured(mv);
    //         if captured_val != 0 {
    //             let captured_piece_type = match captured_val / 2 {
    //                 0 => PieceType::Pawn,
    //                 1 => PieceType::Knight,
    //                 2 => PieceType::Bishop,
    //                 3 => PieceType::Rook,
    //                 4 => PieceType::Queen,
    //                 5 => PieceType::King,
    //                 _ => panic!("take_move: Invalid captured piece type"),
    //             };
    //             self.add_piece(to, Piece { piece_type: captured_piece_type, color: self.side.opposite() });
    //         }
    //     }

    //     // 6. Restore Board State from History
    //     self.castle_permission = undo.castle_permission;
    //     self.fifty_move = undo.fifty_move;
    //     self.position_key = undo.position_key;

    //     // Restore En Passant square (64 indicates None)
    //     self.en_passant = if undo.en_passant == 64 { None } else { Some(undo.en_passant) };

    //     if cfg!(debug_assertions) { self.check_board(fn_name!()); }
    // }

    pub fn take_move(&mut self) {
        if cfg!(debug_assertions) { self.check_board(fn_name!()); }

        let bean;
        unsafe { 
            bean = hi; 
        }

        if (bean == 11173) {
            println!("{self}");
            self.print_bitboards(Some(&[
                Piece { piece_type: PieceType::Pawn, color: Color::White }, 
                Piece { piece_type: PieceType::Bishop, color: Color::White },
                Piece { piece_type: PieceType::Pawn, color: Color::Black },
            ]));
        }

        self.history_ply -= 1;
        self.ply -= 1;

        let mv = self.history[self.history_ply].mv;
        let from = from_square(mv);
        let to = to_square(mv);

        if self.en_passant.is_some() { self.hash_en_passant(); }
        self.hash_castle();

        self.castle_permission = self.history[self.history_ply].castle_permission;
        self.fifty_move =  self.history[self.history_ply].fifty_move;
        self.en_passant =  self.history[self.history_ply].en_passant;

        if self.en_passant.is_some() { self.hash_en_passant(); }
        self.hash_castle();

        self.side = self.side.opposite();
        self.hash_side();

        if is_en_passant(mv) {
            // `to` is diagonal, +-8 gets the square behind/infront
            if self.side == Color::White { self.add_piece(to - 8, Piece { piece_type: PieceType::Pawn, color: Color::Black }); }
            else { self.add_piece(to + 8, Piece { piece_type: PieceType::Pawn, color: Color::Black }); }

        } else if is_castling(mv) {
            if to == C1      { self.move_piece(A1, D1); }
            else if to == C8 { self.move_piece(A8, D8); }
            else if to == G1 { self.move_piece(H1, F1); }
            else if to == G8 { self.move_piece(H8, F8); }
            else { eprintln!("{}", "take_move: castle problem".red()); }
        }

        self.move_piece(to, from);
        if (bean == 11172) {
            println!("{self}");
            self.print_bitboards(Some(&[
                Piece { piece_type: PieceType::Pawn, color: Color::White }, 
                Piece { piece_type: PieceType::Bishop, color: Color::White },
                Piece { piece_type: PieceType::Pawn, color: Color::Black },
            ]));
        }

        let captured = captured(mv);
        if captured != 0 {
            let piece = Piece::MV_TO_PIECE[captured];
            self.add_piece(to, piece);
        }

        let promote_piece = promoted(mv);
        let mut pawn_type: Option<Piece> = None;
        if promote_piece != 0 {
            let piece = Piece::MV_TO_PIECE[promote_piece];
            if piece.color == Color::White { pawn_type = Some(Piece { piece_type: PieceType::Pawn, color: Color::White }); }
            else { pawn_type = Some(Piece { piece_type: PieceType::Pawn, color: Color::Black }); }
            self.add_piece(to, piece);
        }

        if promote_piece != 0 {
            self.clear_piece(from);
            self.add_piece(from, pawn_type.unwrap());
        }

        if cfg!(debug_assertions) { self.check_board(fn_name!()); }
    }
}
