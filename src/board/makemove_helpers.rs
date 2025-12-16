use crate::{board::{Board, Undo, clear_bit, position_keys::{self, CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::{Color, Piece, PieceType}, fn_name, movegen::{attacks::square_attacked, captured, from_square, is_castling, is_double_push, is_en_passant, promoted, to_square}};

impl Board {
    fn hash_piece(&mut self, piece: Piece, square: usize) { self.position_key ^= PIECE_KEYS[piece.bb_index()][square]; }
    pub(crate) fn hash_castle(&mut self) { self.position_key ^= CASTLE_KEYS[self.castle_permission as usize]; }
    pub(crate) fn hash_side(&mut self) { self.position_key ^= *SIDE_KEY; }
    pub(crate) fn hash_en_passant(&mut self) { 
        if let Some(en_passant) = self.en_passant {
            self.position_key ^=  EN_PASSANT_KEYS[en_passant]; 
        }
    }

    pub fn clear_piece(&mut self, square: usize) {
        if let Some(piece) = self.pieces[square] {
            self.hash_piece(piece, square);
            clear_bit(&mut self.bitboards[piece.bb_index()], square);
            self.pieces[square] = None;
            // position.materal
            // position big/major/minor piece
        }
    }

    pub(crate) fn add_piece(&mut self, square: usize, piece: Piece) {
        self.hash_piece(piece, square);
        set_bit(&mut self.bitboards[piece.bb_index()], square);
        self.pieces[square] = Some(piece);
    }

    pub(crate) fn move_piece(&mut self, from: usize, to: usize) {
        if let Some(piece) = self.pieces[from] {
            self.hash_piece(piece, from);
            clear_bit(&mut self.bitboards[piece.bb_index()], from);
            self.pieces[from] = None;

            if let Some(captured_piece) = self.pieces[to] {
                self.hash_piece(captured_piece, to);
                clear_bit(&mut self.bitboards[captured_piece.bb_index()], to);
            }

            self.hash_piece(piece, to);
            set_bit(&mut self.bitboards[piece.bb_index()], to);
            self.pieces[to] = Some(piece);
        }
    }
}