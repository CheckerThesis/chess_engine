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

        // Castling
        if mv.is_castling() {
            if square_attacked(from, side, self) { return false }

            let crossing = (from + to) / 2; // equivalent to switch statement
            if square_attacked(crossing, side, self) { return false }

            match to {
                C1 => self.move_piece(A1, D1),
                C8 => self.move_piece(A8, D8),
                G1 => self.move_piece(H1, F1),
                G8 => self.move_piece(H8, F8),
                _ => eprintln!("{}", "make_move: castle problem".red())
            }
        }

        self.history[self.history_ply] = Undo {
            mv,
            castle_permission: self.castle_permission,
            en_passant: self.en_passant,
            fifty_move: self.fifty_move,
            position_key: self.position_key,
        };

        // Capture and fifty move logic
        if mv.captured() != 0 {
            if !mv.is_en_passant() { self.clear_piece(to); }
            self.fifty_move = 0;
        } 
        else { 
            self.fifty_move = if self.pieces[from].piece_type() == PieceType::PAWN { 0 } 
            else { self.fifty_move + 1 };
        }

        // Enpassant
        if mv.is_en_passant() {
            if side == Color::WHITE { self.clear_piece(to - 8); }
            else { self.clear_piece(to + 8); }
        } 

        // Hashing
        if self.en_passant.is_some() { self.hash_en_passant(); }
        self.en_passant = None;

        let new_castle = self.castle_permission & CASTLE_PERMISSION[from] & CASTLE_PERMISSION[to];
        if new_castle != self.castle_permission {
            self.position_key ^= CASTLE_KEYS[self.castle_permission as usize] ^ CASTLE_KEYS[new_castle as usize];
            self.castle_permission = new_castle;
        }

        // Double push
        if mv.is_double_push() {
            if side == Color::WHITE { self.en_passant = Some((from + 8) as u8); }
            else { self.en_passant = Some((from - 8) as u8); }
            self.hash_en_passant();
        }
        
        // Promotion and move piece
        let promote_piece = mv.promoted();
        if promote_piece != 0 {
            self.clear_piece(from);
            self.add_piece(to, Piece(promote_piece as u8));
        }
        else { self.move_piece(from, to); }
        
        self.history_ply += 1;
        self.ply += 1;
        self.side = self.side.opposite();
        self.hash_side();

        #[cfg(debug_assertions)] { self.check_board(fn_name!()); }

        let king_square = self.bitboards[PieceType::KING.bb_index(side)].trailing_zeros() as usize;
        if square_attacked(king_square, side, self) {
            self.take_move();
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
            if self.side == Color::WHITE { self.add_piece(to - 8, Piece::make(PieceType::PAWN, Color::BLACK)); }
            else { self.add_piece(to + 8, Piece::make(PieceType::PAWN, Color::WHITE)); }

        } else if mv.is_castling() {
            match to {
                C1 => self.move_piece(D1, A1),
                C8 => self.move_piece(D8, A8),
                G1 => self.move_piece(F1, H1),
                G8 => self.move_piece(F8, H8),
                _ => eprintln!("{}", "take_move: castle problem".red())
            }
        }

        self.move_piece(to, from);

        let captured = mv.captured();
        if captured != 0 && !mv.is_en_passant() {
            let piece = Piece(captured as u8);
            self.add_piece(to, piece);
        }

        if mv.promoted() != 0 {
            self.clear_piece(from);
            let piece = Piece::make(PieceType::PAWN, self.side);
            self.add_piece(from, piece);
        }

        #[cfg(debug_assertions)] { self.check_board(fn_name!()); }
    }
}
