pub mod position_keys;
pub mod makemove;
pub mod makemove_helpers;

use std::{collections::HashMap, fmt, ops::Index};

use colored::Colorize;

use crate::defs::{fr2sq, Castling, Color, Files, Piece, PieceType, Ranks};

pub const MAX_GAME_MOVES: usize = 2048;
pub const MAX_DEPTH: usize = 32;

pub fn set_bit(bb: &mut u64, square: usize) { *bb |= 1u64 << square; }
pub fn clear_bit(bb: &mut u64, square: usize) { *bb &= !(1u64 << square); }

#[macro_export]
macro_rules! fn_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        name.strip_suffix("::f").unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or(name)
    }};
}

#[derive(Default, Copy, Clone)]
pub struct Undo {
    pub mv: u32,
    pub castle_permission: u8,
    pub en_passant: usize,
    pub fifty_move: usize,
    pub position_key: u64,
}
pub struct Board {
    pub bitboards: [u64; PieceType::COUNT],
    pub pieces: [Option<Piece>; 64],
    pub side: Color,

    pub en_passant: Option<usize>,
    pub fifty_move: usize,
    pub ply: usize,
    pub history_ply: usize,
    pub castle_permission: u8,
    pub position_key: u64,

    pub history: [Undo; MAX_GAME_MOVES],

    // principal variation is the best sequence of moves,
    // ie. the best according to the engine
    // ie. expected moves played
    pub pv_array: [u32; MAX_DEPTH],

    // for move ordering, rough way to record non-capture moves that are good enough to cause beta cut-off or good alpha
    pub search_history: [[u32; 64]; 13], // stores when a score has beaten alpha, history heuristic
    pub search_killers: [[u32; MAX_DEPTH]; 2], // stores when a score has beaten beta but is not a capture, killer moves
}
impl Board {
    pub fn new(fen: &str) -> Self {
        let mut board = Board::default();
        board.parse_fen(fen);
        board
    }

    pub fn occupancies(&self, side: Color) -> u64 {        
        let mut occupancy: u64 = 0;
        let pieces = match side {
            Color::White => Piece::WHITE,
            Color::Black => Piece::BLACK,
            _ => {
                eprintln!("{}", "occupancies: incorrect side in FEN".red());
                Piece::WHITE
            },
        };

        for piece in pieces { occupancy |= self.bitboards[piece.bb_index()]; }

        occupancy
    }

    #[cfg(debug_assertions)]
    pub fn check_board(&self, location_called: &str) {
        let mut bb_from_pieces: [u64; PieceType::COUNT] = [0; PieceType::COUNT];

        // Bb matches pieces
        for (square, piece_option) in self.pieces.iter().enumerate() {
            if let Some(piece) = piece_option {
                set_bit(&mut bb_from_pieces[piece.bb_index()], square);
            }
        }
        for i in 0..PieceType::COUNT {
            if bb_from_pieces[i] != self.bitboards[i] {
                eprintln!("{}", format!("check_board ({}): bitboard[{}] and pieces array not synced!", location_called, i).red());
            }
        }
        
        // Each bb has correct piece_type
        for (bb_index, &bb) in self.bitboards.iter().enumerate() {
            let mut bb_copy = bb;
            while bb_copy != 0 {
                let square = bb_copy.trailing_zeros() as usize;
                if let Some(piece) = self.pieces[square] {
                    if piece.bb_index() != bb_index {
                        eprintln!("{}", format!("check_board ({}): piece at square {} has wrong bitboard index", location_called, square).red());
                    }
                } else {
                    eprintln!("{}", format!("check_board ({}): bitboard[{}] has bit set at square {} but no piece exists", location_called, bb_index, square).red());
                }
                
                bb_copy &= bb_copy - 1;
            }
        }

        // Occupancy matches bb
        let white_occupancy = self.occupancies(Color::White);
        let black_occupancy = self.occupancies(Color::Black);
        let mut computed_white_occupancy = 0u64;
        let mut computed_black_occupancy = 0u64;
        for piece in Piece::WHITE { computed_white_occupancy |= self.bitboards[piece.bb_index()]; }
        for piece in Piece::BLACK { computed_black_occupancy |= self.bitboards[piece.bb_index()]; }
        if white_occupancy != computed_white_occupancy { eprintln!("{}", format!("check_board ({}): white occupancy mismatch", location_called).red()); }
        if black_occupancy != computed_black_occupancy { eprintln!("{}", format!("check_board ({}): black occupancy mismatch", location_called).red()); }
        if (white_occupancy & black_occupancy) != 0 { eprintln!("{}", format!("check_board ({}): white and black occupancies overlap", location_called).red()); }

        // Enpassant valid
        if let Some(ep_square) = self.en_passant {
            if ep_square >= 64 {
                eprintln!("{}", format!("check_board ({}): en passant square out of bounds", location_called).red());
            } else {
                let rank = ep_square / 8;
                if (self.side == Color::White && rank != 5) || (self.side == Color::Black && rank != 2) {
                    eprintln!("{}", format!("check_board ({}): en passant square on wrong rank for current side", location_called).red());
                }
            }
        }
        
        // King exists
        let white_king_bb = self.bitboards[Piece::WHITE[PieceType::King as usize].bb_index()];
        let black_king_bb = self.bitboards[Piece::BLACK[PieceType::King as usize].bb_index()];
        let white_king_count = white_king_bb.count_ones();
        let black_king_count = black_king_bb.count_ones();
        if white_king_count != 1 { eprintln!("{}", format!("check_board ({}): expected 1 white king, found {}", location_called, white_king_count).red()); }
        if black_king_count != 1 { eprintln!("{}", format!("check_board ({}): expected 1 black king, found {}", location_called, black_king_count).red()); }

        if self.generate_position_key() != self.position_key { eprintln!("{}", format!("check_board ({}): position key wrong", location_called)); }
    }

    pub fn parse_fen(&mut self, fen: &str) {
        fn add_piece(position: &mut Board, square: usize, piece: Piece) {
            set_bit(&mut position.bitboards[piece.bb_index()], square);
            position.pieces[square] = Some(piece);
        }

        fn reset_board(position: &mut Board) {
            for square in 0..64 { position.pieces[square] = None; }
            position.bitboards.fill(0);
            position.en_passant = None;
            position.fifty_move = 0;
            position.ply = 0;
            position.history_ply = 0;
            position.castle_permission = 0;
            position.position_key = 0;
        }

        reset_board(self);
        
        let piece_map: HashMap<char, Piece> = HashMap::from([
            ('p', Piece { piece_type: PieceType::Pawn, color: Color::Black}), 
            ('r', Piece { piece_type: PieceType::Rook, color: Color::Black}), 
            ('n', Piece { piece_type: PieceType::Knight, color: Color::Black}),
            ('b', Piece { piece_type: PieceType::Bishop, color: Color::Black}), 
            ('k', Piece { piece_type: PieceType::King, color: Color::Black}), 
            ('q', Piece { piece_type: PieceType::Queen, color: Color::Black}),
            ('P', Piece { piece_type: PieceType::Pawn, color: Color::White}), 
            ('R', Piece { piece_type: PieceType::Rook, color: Color::White}), 
            ('N', Piece { piece_type: PieceType::Knight, color: Color::White}),
            ('B', Piece { piece_type: PieceType::Bishop, color: Color::White}), 
            ('K', Piece { piece_type: PieceType::King, color: Color::White}), 
            ('Q', Piece { piece_type: PieceType::Queen, color: Color::White}),

            ('D', Piece { piece_type: PieceType::King, color: Color::White}), 
            ('d', Piece { piece_type: PieceType::Queen, color: Color::Black}),
        ]);

        let fen_split: Vec<&str> = fen.split_whitespace().collect();
        if fen_split.len() < 4 { eprintln!("{}", "parse_fen: FEN string is invalid".red()); }

        let mut rank = Ranks::Rank8 as isize;
        let mut file = Files::FileA as usize;
        for character in fen_split[0].chars() {
            if let Some(&piece) = piece_map.get(&character) {
                add_piece(self, fr2sq(file, rank as usize), piece);
                file += 1;
            } else if let Some(empty_squares) = character.to_digit(10) {
                file += empty_squares as usize;
            } else if character == '/' {
                if file != 8 { eprintln!("{}", "parse_fen: incorrect amount of char in file".red()); }
                rank -= 1;
                if rank < 0 { eprintln!("{}", "parse_fen: Rank underflow".red()); }
                file = Files::FileA as usize;
            } else {
                eprintln!("{}", "parse_fen: incorrect char in FEN".red());
            }
        }
        
        self.side = match fen_split[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _ => {
                eprintln!("{}", "parse_fen: incorrect side in FEN".red());
                Color::Neither
            },
        };

        for i in fen_split[2].chars() {
            match i {
                'K' => self.castle_permission |= Castling::WhiteKingCastle as u8,
                'Q' => self.castle_permission |= Castling::WhiteQueenCastle as u8,
                'k' => self.castle_permission |= Castling::BlackKingCastle as u8,
                'q' => self.castle_permission |= Castling::BlackQueenCastle as u8,
                '-' => break,
                _ => eprintln!("{}", "parse_fen: invalid castling permission".red()),
            }
        }

        if fen_split[3] != "-" {
            let file = fen_split[3].chars().next().unwrap() as usize - 'a' as usize;
            let rank = fen_split[3].chars().nth(1).unwrap().to_digit(10).unwrap() as usize - 1;
            self.en_passant = Some(fr2sq(file, rank));
        }

        self.position_key = self.generate_position_key();
    }

    pub fn print_bitboards(&self, pieces_to_print: Option<&[Piece]>) {
        fn index_to_piece_name(index: usize) -> String {
            let color = if index % 2 == 0 { "White" } else { "Black" };
            let piece_type = match index / 2 {
                0 => "Pawn",
                1 => "Knight",
                2 => "Bishop",
                3 => "Rook",
                4 => "Queen",
                5 => "King",
                6 => "Dragon",
                _ => "Unknown",
            };
            format!("{} {}", color, piece_type)
        }
        fn print_single_bitboard(index: usize, bitboard: u64) {
            println!("\nPiece: {}", index_to_piece_name(index));

            for rank in (Ranks::Rank1 as usize..=Ranks::Rank8 as usize).rev() {
                print!("{}  ", rank + 1);
                for file in Files::FileA as usize..=Files::FileH as usize {
                    let square = fr2sq(file, rank);
                    if (bitboard >> square) & 1 == 1 { print!("X "); } 
                    else { print!(". "); }
                }
                println!();
            }
            print!("\n   A B C D E F G H\n");
        }

        println!("\n===================\n     BITBOARDS\n===================");

        match pieces_to_print {
            Some(pieces) => {
                for piece in pieces {
                    let index = piece.bb_index();
                    print_single_bitboard(index, self.bitboards[index]);
                }
            }
            None => {
                for i in 0..PieceType::COUNT {
                    print_single_bitboard(i, self.bitboards[i]);
                }
            }
        }
        println!("===================\n");
    }
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn piece_to_char(piece: Piece) -> char {
            match piece.piece_type {
                PieceType::Pawn => if piece.color == Color::White { '♙' } else { '♟' },
                PieceType::Knight => if piece.color == Color::White { '♘' } else { '♞' },
                PieceType::Bishop => if piece.color == Color::White { '♗' } else { '♝' },
                PieceType::Rook => if piece.color == Color::White { '♖' } else { '♜' },
                PieceType::Queen => if piece.color == Color::White { '♕' } else { '♛' },
                PieceType::King => if piece.color == Color::White { '♔' } else { '♚' },
                PieceType::Dragon => if piece.color == Color::White { 'D' } else { 'D' },
            }
        }

        for rank in (Ranks::Rank1 as usize..=Ranks::Rank8 as usize).rev() {
            write!(f, "{}  ", rank + 1)?;
            for file in Files::FileA as usize..=Files::FileH as usize {
                let square = fr2sq(file, rank);
                let piece_char = match self.pieces[square] {
                    Some(piece) => piece_to_char(piece),
                    None => '.',
                };
                write!(f, "{:>2}", piece_char)?;
            }
            writeln!(f)?;
        }

        write!(f, "\n   ")?;
        for file in 'A'..='H' { write!(f, "{:>2}", file)?; }
        writeln!(f, "\n")?;

        let side_str = if self.side == Color::White { "White" } else { "Black" };
        writeln!(f, "Side to move: {}", side_str)?;

        let en_passant_str = match self.en_passant {
            Some(square) => {
                let file = ((square % 8) as u8 + b'a') as char;
                let rank = ((square / 8) + 1).to_string();
                format!("{}{}", file, rank)
            }
            None => "-".to_string(),
        };
        writeln!(f, "En Passant: {}", en_passant_str)?;

        let wkc = if self.castle_permission & Castling::WhiteKingCastle as u8 != 0 { "K" } else { "-" };
        let wqc = if self.castle_permission & Castling::WhiteQueenCastle as u8 != 0 { "Q" } else { "-" };
        let bkc = if self.castle_permission & Castling::BlackKingCastle as u8 != 0 { "k" } else { "-" };
        let bqc = if self.castle_permission & Castling::BlackQueenCastle as u8 != 0 { "q" } else { "-" };
        writeln!(f, "Castling Rights: {}{}{}{}", wkc, wqc, bkc, bqc)?;

        writeln!(f, "Position Key: {:X}", self.position_key)?;

        Ok(())
    }
}
impl Default for Board {
    fn default() -> Self {
        Board {
            bitboards: [0; PieceType::COUNT],
            pieces: [None; 64],
            side: Color::Neither,
            en_passant: None,
            fifty_move: 0,
            ply: 0,
            history_ply: 0,
            castle_permission: 0,
            position_key: 0,
            history: [Undo::default(); MAX_GAME_MOVES],
            pv_array: [0; MAX_DEPTH],
            search_history: [[0; 64]; 13],
            search_killers: [[0; MAX_DEPTH]; 2]
        }
    }
}

pub fn print_bitboard(bitboard: u64) {
    for rank in (Ranks::Rank1 as usize..=Ranks::Rank8 as usize).rev() {
        for file in Files::FileA as usize..=Files::FileH as usize {
            let square = fr2sq(file, rank);

            if ((1 << square) & bitboard) != 0 { print!("X "); } 
            else { print!("- "); }
        }
        println!();
    }
    println!();
}
