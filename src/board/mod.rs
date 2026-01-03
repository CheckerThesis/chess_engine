pub mod position_keys;
pub mod makemove;
pub mod makemove_helpers;
pub mod test;

use std::{collections::HashMap, fmt};

use colored::Colorize;

use crate::{defs::{BLACK_KING_CASTLE, BLACK_QUEEN_CASTLE, Color, Files, Piece, PieceType, Ranks, WHITE_KING_CASTLE, WHITE_QUEEN_CASTLE, fr2sq}, movegen::Move};

pub const MAX_GAME_MOVES: usize = 2048;
pub const MAX_DEPTH: usize = 64;

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

#[derive(Default, Copy, Clone, PartialEq, Debug)]
pub struct Undo {
    pub mv: Move,
    pub castle_permission: u8,
    pub en_passant: Option<u8>,
    pub fifty_move: u8,
    pub position_key: u64,
}
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Board {
    pub occupancies: [u64; 3],
    pub bitboards: [u64; Piece::COUNT],
    pub king_square: [usize; 2],
    pub pieces: [Piece; 64],
    pub side: Color,

    pub en_passant: Option<u8>,
    pub fifty_move: u8,
    pub ply: u8,
    pub history_ply: usize,
    pub castle_permission: u8,
    pub position_key: u64,

    pub history: [Undo; MAX_GAME_MOVES],

    // for move ordering, rough way to record non-capture moves that are good enough to cause beta cut-off or good alpha
    pub history_heuristic: [[i16; 64]; Piece::COUNT], // stores when a score has beaten alpha, history heuristic
    pub killers: [[Option<Move>; 2]; MAX_DEPTH], // stores when a score has beaten beta but is not a capture, killer moves
}
impl Board {
    pub fn new(fen: &str) -> Self {
        let mut board = Board::default();
        board.parse_fen(fen);
        board
    }

    #[cfg(debug_assertions)]
    pub fn check_board(&self, location_called: &str) {
        let mut bb_from_pieces: [u64; Piece::COUNT] = [0; Piece::COUNT];

        // Bb matches pieces
        for (square, piece) in self.pieces.iter().enumerate() {
            if *piece != Piece::NONE {
                set_bit(&mut bb_from_pieces[piece.index()], square);
            }
        }
        for i in 0..Piece::COUNT {
            if bb_from_pieces[i] != self.bitboards[i] {
                eprintln!("{}", format!("check_board ({}): bitboard[{}] and pieces array not synced!", location_called, i).red());
                panic!();
            }
        }
        
        // Each bb has correct piece_type
        for (bb_index, &bb) in self.bitboards.iter().enumerate() {
            let mut bb_copy = bb;
            while bb_copy != 0 {
                let square = bb_copy.trailing_zeros() as usize;
                let piece = self.pieces[square];
                if piece != Piece::NONE {
                    if piece.index() != bb_index {
                        eprintln!("{}", format!("check_board ({}): piece at square {} has wrong bitboard index", location_called, square).red());
                        panic!();
                    }
                } else {
                    eprintln!("{}", format!("check_board ({}): bitboard[{}] has bit set at square {} but no piece exists", location_called, bb_index, square).red());
                    panic!();
                }
                
                bb_copy &= bb_copy - 1;
            }
        }

        // Occupancy matches bb
        let white_occupancy = self.occupancies[Color::WHITE.index()];
        let black_occupancy = self.occupancies[Color::BLACK.index()];
        let mut computed_white_occupancy = 0u64;
        let mut computed_black_occupancy = 0u64;
        for piece in Piece::WHITE_PIECES { computed_white_occupancy |= self.bitboards[piece.index()]; }
        for piece in Piece::BLACK_PIECES { computed_black_occupancy |= self.bitboards[piece.index()]; }
        if white_occupancy != computed_white_occupancy { 
            eprintln!("{}", format!("check_board ({}): white occupancy mismatch", location_called).red()); 
            println!("white_occupancy");
            print_bitboard(white_occupancy);
            
            println!("computed_white_occupancy");
            print_bitboard(computed_white_occupancy);

            println!("{self}");

            panic!();
        }
        if black_occupancy != computed_black_occupancy { 
            eprintln!("{}", format!("check_board ({}): black occupancy mismatch", location_called).red()); 
            panic!();
        }
        if (white_occupancy & black_occupancy) != 0 { 
            eprintln!("{}", format!("check_board ({}): white and black occupancies overlap", location_called).red());
            panic!();
        }

        // Enpassant valid
        if let Some(ep_square) = self.en_passant {
            if ep_square >= 64 {
                eprintln!("{}", format!("check_board ({}): en passant square out of bounds", location_called).red());
                panic!();
            } else {
                let rank = ep_square / 8;
                if (self.side == Color::WHITE && rank != 5) || (self.side == Color::BLACK && rank != 2) {
                    eprintln!("{}", format!("check_board ({}): en passant square on wrong rank for current side", location_called).red());
                    panic!();
                }
            }
        }
        
        // King exists
        let white_king_bb = self.bitboards[Piece::WHITE_KING.index()];
        let black_king_bb = self.bitboards[Piece::BLACK_KING.index()];
        let white_king_count = white_king_bb.count_ones();
        let black_king_count = black_king_bb.count_ones();
        if white_king_count != 1 { 
            eprintln!("{}", format!("check_board ({}): expected 1 white king, found {}", location_called, white_king_count).red()); 
            panic!();
        }
        if black_king_count != 1 { 
            eprintln!("{}", format!("check_board ({}): expected 1 black king, found {}", location_called, black_king_count).red()); 
            panic!();
        }

        if self.generate_position_key() != self.position_key { 
            eprintln!("{}", format!("check_board ({}): position key wrong", location_called)); 
            panic!();
        }
    }

    pub fn parse_fen(&mut self, fen: &str) {
        fn add_piece(position: &mut Board, square: usize, piece: Piece) {
            set_bit(&mut position.bitboards[piece.index()], square);
            set_bit(&mut position.occupancies[piece.color().index()], square);
            set_bit(&mut position.occupancies[Color::BOTH.index()], square);
            position.pieces[square] = piece;
            if piece.piece_type() == PieceType::KING {
                position.king_square[piece.color().index()] = square;
            }
        }

        fn reset_board(position: &mut Board) {
            for square in 0..64 { position.pieces[square] = Piece::NONE; }
            position.bitboards.fill(0);
            position.occupancies.fill(0);
            position.en_passant = None;
            position.fifty_move = 0;
            position.ply = 0;
            position.history_ply = 0;
            position.castle_permission = 0;
            position.position_key = 0;
        }

        reset_board(self);
        
        let piece_map: HashMap<char, Piece> = HashMap::from([
            ('p', Piece::BLACK_PAWN), 
            ('r', Piece::BLACK_ROOK), 
            ('n', Piece::BLACK_KNIGHT),
            ('b', Piece::BLACK_BISHOP), 
            ('k', Piece::BLACK_KING), 
            ('q', Piece::BLACK_QUEEN),
            ('P', Piece::WHITE_PAWN), 
            ('R', Piece::WHITE_ROOK), 
            ('N', Piece::WHITE_KNIGHT),
            ('B', Piece::WHITE_BISHOP), 
            ('K', Piece::WHITE_KING), 
            ('Q', Piece::WHITE_QUEEN),

            ('d', Piece::BLACK_DRAGON), 
            ('D', Piece::WHITE_DRAGON),
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
            "w" => Color::WHITE,
            "b" => Color::BLACK,
            _ => {
                eprintln!("{}", "parse_fen: incorrect side in FEN".red());
                Color::EITHER
            },
        };

        for i in fen_split[2].chars() {
            match i {
                'K' => self.castle_permission |= WHITE_KING_CASTLE,
                'Q' => self.castle_permission |= WHITE_QUEEN_CASTLE,
                'k' => self.castle_permission |= BLACK_KING_CASTLE,
                'q' => self.castle_permission |= BLACK_QUEEN_CASTLE,
                '-' => break,
                _ => eprintln!("{}", "parse_fen: invalid castling permission".red()),
            }
        }

        if fen_split[3] != "-" {
            let file = fen_split[3].chars().next().unwrap() as usize - 'a' as usize;
            let rank = fen_split[3].chars().nth(1).unwrap().to_digit(10).unwrap() as usize - 1;
            self.en_passant = Some(fr2sq(file, rank) as u8);
        }

        self.position_key = self.generate_position_key();
    }

    pub fn get_fen(&self) -> String {
        let mut fen = String::new();

        // 1. Piece Placement
        // Iterate Ranks 8 -> 1
        for rank in (0..8).rev() {
            let mut empty_squares = 0;
            
            // Iterate Files A -> H
            for file in 0..8 {
                let square = rank * 8 + file;
                
                match self.pieces[square] {
                    Piece::NONE => empty_squares += 1,
                    piece => {
                        if empty_squares > 0 {
                            fen.push_str(&empty_squares.to_string());
                            empty_squares = 0;
                        }

                        fen.push(match piece {
                            Piece::WHITE_PAWN => 'P',
                            Piece::BLACK_PAWN => 'p',
                            Piece::WHITE_KNIGHT => 'N',
                            Piece::BLACK_KNIGHT => 'n',
                            Piece::WHITE_BISHOP => 'B',
                            Piece::BLACK_BISHOP => 'b',
                            Piece::WHITE_ROOK => 'R',
                            Piece::BLACK_ROOK => 'r',
                            Piece::WHITE_QUEEN => 'Q',
                            Piece::BLACK_QUEEN => 'q',
                            Piece::WHITE_KING => 'K',
                            Piece::BLACK_KING => 'k',
                            Piece::WHITE_DRAGON => 'D',
                            Piece::BLACK_DRAGON => 'd',
                            _ => '?',
                        });
                    }
                }
            }
            
            if empty_squares > 0 {
                fen.push_str(&empty_squares.to_string());
            }

            if rank > 0 {
                fen.push('/');
            }
        }

        // 2. Active Color
        fen.push(' ');
        fen.push(match self.side {
            Color::WHITE => 'w',
            Color::BLACK => 'b',
            _ => '-',
        });

        // 3. Castling Rights
        fen.push(' ');
        let mut castling = String::new();
        if self.castle_permission & WHITE_KING_CASTLE != 0 { castling.push('K'); }
        if self.castle_permission & WHITE_QUEEN_CASTLE != 0 { castling.push('Q'); }
        if self.castle_permission & BLACK_KING_CASTLE != 0 { castling.push('k'); }
        if self.castle_permission & BLACK_QUEEN_CASTLE != 0 { castling.push('q'); }

        if castling.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&castling);
        }

        // 4. En Passant Target
        fen.push(' ');
        match self.en_passant {
            Some(sq) => {
                let file = (sq % 8) as u8 + b'a';
                let rank = (sq / 8) as u8 + b'1';
                fen.push(file as char);
                fen.push(rank as char);
            }
            None => fen.push('-'),
        }

        // 5. Halfmove Clock (50-move rule counter)
        fen.push(' ');
        fen.push_str(&self.fifty_move.to_string());

        // 6. Fullmove Number
        // Standard calculation: (total_plies / 2) + 1
        fen.push(' ');
        let fullmove = (self.history_ply / 2) + 1;
        fen.push_str(&fullmove.to_string());

        fen
    }

    pub fn print_bitboards(&self) {
        println!("White");
        print_bitboard(self.occupancies[Color::WHITE.index()]);
        println!("Black");
        print_bitboard(self.occupancies[Color::BLACK.index()]);
        println!("Both");
        print_bitboard(self.occupancies[Color::BOTH.index()]);

        for piece in Piece::iter() {
            println!("{}", piece); 
            print_bitboard(self.bitboards[piece.index()]);
        }
    }
}
impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn piece_to_char(piece: Piece) -> char {
            match piece {
                Piece::WHITE_PAWN => '♙',
                Piece::BLACK_PAWN => '♟',
                Piece::WHITE_KNIGHT => '♘',
                Piece::BLACK_KNIGHT => '♞',
                Piece::WHITE_BISHOP => '♗',
                Piece::BLACK_BISHOP => '♝',
                Piece::WHITE_ROOK => '♖',
                Piece::BLACK_ROOK => '♜',
                Piece::WHITE_QUEEN => '♕',
                Piece::BLACK_QUEEN => '♛',
                Piece::WHITE_KING => '♔',
                Piece::BLACK_KING => '♚',
                Piece::WHITE_DRAGON => 'D',
                Piece::BLACK_DRAGON => 'D',
                _ => '.',
            }
        }

        for rank in (Ranks::Rank1 as usize..=Ranks::Rank8 as usize).rev() {
            write!(f, "{}  ", rank + 1)?;
            for file in Files::FileA as usize..=Files::FileH as usize {
                let square = fr2sq(file, rank);
                let piece_char = piece_to_char(self.pieces[square]);
                write!(f, "{:>2}", piece_char)?;
            }
            writeln!(f)?;
        }

        write!(f, "\n   ")?;
        for file in 'A'..='H' { write!(f, "{:>2}", file)?; }
        writeln!(f, "\n")?;

        let side_str = if self.side == Color::WHITE { "White" } else { "Black" };
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

        let wkc = if self.castle_permission & WHITE_KING_CASTLE as u8 != 0 { "K" } else { "-" };
        let wqc = if self.castle_permission & WHITE_QUEEN_CASTLE as u8 != 0 { "Q" } else { "-" };
        let bkc = if self.castle_permission & BLACK_KING_CASTLE as u8 != 0 { "k" } else { "-" };
        let bqc = if self.castle_permission & BLACK_QUEEN_CASTLE as u8 != 0 { "q" } else { "-" };
        writeln!(f, "Castling Rights: {}{}{}{}", wkc, wqc, bkc, bqc)?;

        writeln!(f, "Position Key: {:X}", self.position_key)?;

        Ok(())
    }
}
impl Default for Board {
    fn default() -> Self {
        Board {
            occupancies: [0; 3],
            bitboards: [0; Piece::COUNT],
            king_square: [0; 2],
            pieces: [Piece::NONE; 64],
            side: Color::EITHER,
            en_passant: None,
            fifty_move: 0,
            ply: 0,
            history_ply: 0,
            castle_permission: 0,
            position_key: 0,
            history: [Undo::default(); MAX_GAME_MOVES],
            history_heuristic: [[0; 64]; Piece::COUNT],
            killers: [[None; 2]; MAX_DEPTH]
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
