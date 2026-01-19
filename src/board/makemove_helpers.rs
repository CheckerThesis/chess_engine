use colored::Colorize;

use crate::{board::{Board, clear_bit, position_keys::{CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::{Color, Piece, PieceType}};

impl Board {
    fn hash_piece(&mut self, piece: Piece, square: usize) { self.position_key ^= PIECE_KEYS[piece.index()][square]; }
    pub(crate) fn hash_castle(&mut self) { self.position_key ^= CASTLE_KEYS[self.castle_permission as usize]; }
    pub(crate) fn hash_side(&mut self) { self.position_key ^= *SIDE_KEY; }
    pub(crate) fn hash_en_passant(&mut self) { 
        if let Some(en_passant) = self.en_passant {
            self.position_key ^=  EN_PASSANT_KEYS[en_passant as usize]; 
        }
    }

    pub(crate) fn clear_piece(&mut self, square: usize) {
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

        if piece.piece_type() == PieceType::KING {
            self.king_square[piece.color().index()] = square;
        }
    }

    pub(crate) fn move_piece(&mut self, from: usize, to: usize) {
        let piece = self.pieces[from];
        #[cfg(debug_assertions)] {
            if piece == Piece::NONE {
                eprintln!("{}", format!("move_piece_quiet: Tried to remove NONE on square {}", from).red());
            }
        }

        self.position_key ^= PIECE_KEYS[piece.index()][from] ^ PIECE_KEYS[piece.index()][to];
        
        let from_and_to_bb = (1 << from) | (1 << to);
        self.bitboards[piece.index()] ^= from_and_to_bb;
        self.occupancies[piece.color().index()] ^= from_and_to_bb;
        self.occupancies[Color::BOTH.index()] ^= from_and_to_bb;

        self.pieces[from] = Piece::NONE;
        self.pieces[to] = piece;

        if piece.piece_type() == PieceType::KING {
            self.king_square[piece.color().index()] = to;
        } 
    }
}