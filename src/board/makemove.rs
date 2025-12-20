use colored::Colorize;

use crate::{board::{Board, Undo, clear_bit, position_keys::{self, CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::{Color, Piece, PieceType}, fn_name, movegen::{Move, attacks::square_attacked}, squares::squares::{A1, A8, C1, C8, D1, D8, F1, F8, G1, G8, H1, H8}};

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
    pub fn make_move(&mut self, mv: Move) -> bool {    
        #[cfg(debug_assertions)] { self.check_board(fn_name!()); }

        let from = mv.from_square();
        let to = mv.to_square();
        let side = self.side;

        self.history[self.history_ply].position_key = self.position_key;

        if mv.is_en_passant() {
            // `to` is diagonal, +-8 gets the square behind/infront
            if side == Color::WHITE { self.clear_piece(to - 8); }
            else { self.clear_piece(to + 8); }

        } else if mv.is_castling() {
            if square_attacked(from, side, self) ||
            square_attacked(to, side, self) { return false }

            let crossing = match to {
                6 => 5,   // White Kingside (g1 needs f1)
                2 => 3,   // White Queenside (c1 needs d1)
                62 => 61, // Black Kingside (g8 needs f8)
                58 => 59, // Black Queenside (c8 needs d8)
                _ => 0,
            };

            if square_attacked(crossing, side, self) { return false }

            if      to == C1 { self.move_piece(A1, D1); }
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

        let captured = mv.captured();
        if captured != 0 {
            self.clear_piece(to);
            self.fifty_move = 0;
        }

        self.history_ply += 1;
        self.ply += 1;

        if self.pieces[from].piece_type() == PieceType::PAWN {
            self.fifty_move = 0;

            if mv.is_double_push() {
                if side == Color::WHITE { self.en_passant = Some((from + 8) as u8); }
                else { self.en_passant = Some((from - 8) as u8); }
            }
            self.hash_en_passant();
        }

        self.move_piece(from, to);

        let promote_piece = mv.promoted();
        if promote_piece != 0 {
            self.clear_piece(to);
            let piece = Piece(promote_piece as u8);
            self.add_piece(to, piece);
        }

        self.side = self.side.opposite();
        self.hash_side();

        #[cfg(debug_assertions)] { self.check_board(fn_name!()); }

        let king_square = self.bitboards[PieceType::KING.bb_index(side)].trailing_zeros() as usize;
        if square_attacked(king_square, side, self) {
            self.take_move();
            debug_assert_eq!(self.position_key, self.history[self.history_ply].position_key, 
                            "Position key mismatch after take_move!");
            return false
        }

        true
    }

    pub fn take_move(&mut self) {
        #[cfg(debug_assertions)] { self.check_board(fn_name!()); }

        self.history_ply -= 1;
        self.ply -= 1;

        let mv = self.history[self.history_ply].mv;
        let from = mv.from_square();
        let to = mv.to_square();

        if self.en_passant.is_some() { self.hash_en_passant(); }
        self.hash_castle();

        self.castle_permission = self.history[self.history_ply].castle_permission;
        self.fifty_move =  self.history[self.history_ply].fifty_move;
        self.en_passant =  self.history[self.history_ply].en_passant;

        if self.en_passant.is_some() { self.hash_en_passant(); }
        self.hash_castle();

        self.side = self.side.opposite();
        self.hash_side();

        if mv.is_en_passant() {
            // `to` is diagonal, +-8 gets the square behind/infront
            if self.side == Color::WHITE { self.add_piece(to - 8, Piece::make(PieceType::PAWN, Color::BLACK)); }
            else { self.add_piece(to + 8, Piece::make(PieceType::PAWN, Color::WHITE)); }

        } else if mv.is_castling() {
            if to == C1      { self.move_piece(D1, A1); }
            else if to == C8 { self.move_piece(D8, A8); }
            else if to == G1 { self.move_piece(F1, H1); }
            else if to == G8 { self.move_piece(F8, H8); }
            else { eprintln!("{}", "take_move: castle problem".red()); }
        }

        self.move_piece(to, from);

        let captured = mv.captured();
        if captured != 0 && !mv.is_en_passant() {
            let piece = Piece(captured as u8);
            self.add_piece(to, piece);
        }

        if mv.promoted() != 0 {
            self.clear_piece(from);  // Remove the promoted piece that move_piece put there
            let piece = Piece::make(PieceType::PAWN, self.side);
            self.add_piece(from, piece);
        }

        #[cfg(debug_assertions)] { self.check_board(fn_name!()); }
    }
}
