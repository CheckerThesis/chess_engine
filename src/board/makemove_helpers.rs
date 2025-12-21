use colored::Colorize;

use crate::{board::{Board, Undo, clear_bit, position_keys::{self, CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::{Color, Piece, PieceType}, fn_name, movegen::{attacks::square_attacked, bitboards}};

impl Board {
   fn hash_piece(&mut self, piece: Piece, square: usize) { self.position_key ^= PIECE_KEYS[piece.index()][square]; }
    pub(crate) fn hash_castle(&mut self) { self.position_key ^= CASTLE_KEYS[self.castle_permission as usize]; }
    pub(crate) fn hash_side(&mut self) { self.position_key ^= *SIDE_KEY; }
    pub(crate) fn hash_en_passant(&mut self) { 
        if let Some(en_passant) = self.en_passant {
            self.position_key ^=  EN_PASSANT_KEYS[en_passant as usize]; 
        }
    }

    pub fn clear_piece(&mut self, square: usize) {
        let piece = self.pieces[square];
        #[cfg(debug_assertions)] {
            if piece == Piece::NONE {
                eprintln!("{}", format!("clear_piece: Tried to remove NONE on square {}", square).red());
            }
        }
        self.hash_piece(piece, square);
        clear_bit(&mut self.bitboards[piece.index()], square);
        clear_bit(&mut self.occupancies[piece.color().index()], square);
        clear_bit(&mut self.occupancies[Color::BOTH.index()], square);
        self.pieces[square] = Piece::NONE;
        // position.materal
        // position big/major/minor piece
        
    }

    pub(crate) fn add_piece(&mut self, square: usize, piece: Piece) {
        self.hash_piece(piece, square);
        set_bit(&mut self.bitboards[piece.index()], square);
        set_bit(&mut self.occupancies[piece.color().index()], square);
        set_bit(&mut self.occupancies[Color::BOTH.index()], square);
        self.pieces[square] = piece;
    }

    pub(crate) fn move_piece(&mut self, from: usize, to: usize) {
        let piece = self.pieces[from];
        #[cfg(debug_assertions)] {
            if piece == Piece::NONE {
                eprintln!("{}", format!("move_piece: Tried to remove NONE on square {}", from).red());
            }
        }
        
        self.hash_piece(piece, from);
        clear_bit(&mut self.bitboards[piece.index()], from);
        clear_bit(&mut self.occupancies[piece.color().index()], from);
        clear_bit(&mut self.occupancies[Color::BOTH.index()], from);
        self.pieces[from] = Piece::NONE;

        let captured_piece = self.pieces[to];
        if captured_piece != Piece::NONE {
            self.hash_piece(captured_piece, to);
            clear_bit(&mut self.bitboards[captured_piece.index()], to);
            clear_bit(&mut self.occupancies[captured_piece.color().index()], to);
            clear_bit(&mut self.occupancies[Color::BOTH.index()], to);
        }
        
        self.hash_piece(piece, to);
        set_bit(&mut self.bitboards[piece.index()], to);
        set_bit(&mut self.occupancies[piece.color().index()], to);
        set_bit(&mut self.occupancies[Color::BOTH.index()], to);
        self.pieces[to] = piece;
    }
}