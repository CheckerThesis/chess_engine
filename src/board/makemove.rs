use colored::Colorize;

use crate::{board::{Board, Undo, clear_bit, position_keys::{self, CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::{Color, Piece}, movegen::{from_square, is_castling, is_en_passant, to_square}};

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
    fn hash_piece(&mut self, piece: Piece, square: usize) { self.position_key ^= PIECE_KEYS[piece.bb_index()][square]; }
    fn hash_castle(&mut self) { self.position_key ^= CASTLE_KEYS[self.castle_permission as usize]; }
    fn hash_side(&mut self) { self.position_key ^= *SIDE_KEY; }
    fn hash_en_passant(&mut self) { 
        if let Some(en_passant) = self.en_passant {
            self.position_key ^=  EN_PASSANT_KEYS[en_passant]; 
        }
    }

    pub fn clear_piece(&mut self, square: usize) {
        let piece = self.pieces[square].unwrap();
        self.hash_piece(piece, square);
        clear_bit(&mut self.bitboards[piece.bb_index()], square);
        self.pieces[square] = None;
        // position.materal
        // position big/major/minor piece
    }

    fn add_piece(&mut self, square: usize, piece: Piece) {
        self.hash_piece(piece, square);
        set_bit(&mut self.bitboards[piece.bb_index()], square);
        self.pieces[square] = Some(piece);
    }

    fn move_piece(&mut self, from: usize, to: usize) {
        let piece = self.pieces[from].unwrap();
        self.hash_piece(piece, from);
        self.pieces[from] = None;

        self.hash_piece(piece, to);
        self.pieces[to] = Some(piece);
    }

    fn make_move(&mut self, mv: u32) {
        let from = from_square(mv);
        let to = to_square(mv);
        let side = self.side;

        // Store history before modifying
        self.history[self.history_ply] = Undo {
            mv: mv,
            castle_permission: self.castle_permission,
            en_passant: self.en_passant.unwrap_or(64),
            fifty_move: self.fifty_move,
            position_key: self.position_key,
        };

        // Handle special moves
        if is_en_passant(mv) {
            // `to` is diagonal, +-8 gets the square behind/infront
            if side == Color::White { self.clear_piece(to - 8); }
            else { self.clear_piece(to + 8); }
        } else if is_castling(mv) {
            match to {
                6 => self.move_piece(7, 5), // White Kingside => Rook H1 to F1
                2 => self.move_piece(0, 3), // White Queenside => Rook A1 to D1
                62 => self.move_piece(63, 61), // Black Kingside => Rook H8 to F8
                58 => self.move_piece(56, 59), // Black Queenside => Rook A8 to D8
                _ => eprintln!("{}", "make_move: Invalid castle move".red()),
            }
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
        

    }

    // fn remove_piece(&mut self, square: usize) -> Option<Piece> {
    //     if let Some(piece) = self.pieces[square] {
    //         clear_bit(&mut self.bitboards[piece.bb_index()], square);
    //         self.pieces[square] = None;
    //         return Some(piece)
    //     }
    //     eprintln!("{}", "remove_piece: No piece to remove".red());
    //     None
    // }
    // fn move_piece(&mut self, from: usize, to: usize) {
    //     if let Some(piece) = self.remove_piece(from) { self.add_piece(to, piece); }
    // }
}