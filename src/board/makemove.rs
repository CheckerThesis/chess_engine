use crate::{board::{Board, clear_bit, position_keys::{CASTLE_KEYS, EN_PASSANT_KEYS, PIECE_KEYS, SIDE_KEY}, set_bit}, defs::Piece};

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