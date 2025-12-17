use std::sync::LazyLock;

/*
8  56 57 58 59 60 61 62 63
7  48 49 50 51 52 53 54 55
6  40 41 42 43 44 45 46 47
5  32 33 34 35 36 37 38 39
4  24 25 26 27 28 29 30 31
3  16 17 18 19 20 21 22 23
2  08 09 10 11 12 13 14 15
1  00 01 02 03 04 05 06 07
   A  B  C  D  E  F  G  H
*/
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(usize)]
pub enum Color {
    White = 0,
    Black = 1,
    Both = 2,
    Neither = 3,
}
impl Color {
    pub fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
            _ => Color::Neither,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(usize)]
pub enum PieceType {
    None   = 0,
    Pawn   = 1,
    Knight = 2,
    Bishop = 3,
    Rook   = 4,
    Queen  = 5,
    King   = 6,
    Dragon = 7,
}
impl PieceType {
    pub const COUNT: usize = 8 * 2;

    pub fn bb_index(&self, side: Color) -> usize { (*self as usize * 2) + side as usize }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: Color,
}
impl Piece {
    pub const NONE: Piece = Piece { piece_type: PieceType::None, color: Color::Neither };

    pub const WHITE: [Piece; 7] = [
        Piece { piece_type: PieceType::Pawn, color: Color::White },
        Piece { piece_type: PieceType::Knight, color: Color::White },
        Piece { piece_type: PieceType::Bishop, color: Color::White },
        Piece { piece_type: PieceType::Rook, color: Color::White },
        Piece { piece_type: PieceType::Queen, color: Color::White },
        Piece { piece_type: PieceType::King, color: Color::White },
        Piece { piece_type: PieceType::Dragon, color: Color::White },
    ];
    pub const BLACK: [Piece; 7] = [
        Piece { piece_type: PieceType::Pawn, color: Color::Black },
        Piece { piece_type: PieceType::Knight, color: Color::Black },
        Piece { piece_type: PieceType::Bishop, color: Color::Black },
        Piece { piece_type: PieceType::Rook, color: Color::Black },
        Piece { piece_type: PieceType::Queen, color: Color::Black },
        Piece { piece_type: PieceType::King, color: Color::Black },
        Piece { piece_type: PieceType::Dragon, color: Color::Black },
    ];
    pub const MV_TO_PIECE: [Piece; PieceType::COUNT] = [
        Piece::NONE, Piece::NONE,
        Piece { piece_type: PieceType::Pawn, color: Color::White },
        Piece { piece_type: PieceType::Pawn, color: Color::Black },
        Piece { piece_type: PieceType::Knight, color: Color::White },
        Piece { piece_type: PieceType::Knight, color: Color::Black },
        Piece { piece_type: PieceType::Bishop, color: Color::White },
        Piece { piece_type: PieceType::Bishop, color: Color::Black },
        Piece { piece_type: PieceType::Rook, color: Color::White },
        Piece { piece_type: PieceType::Rook, color: Color::Black },
        Piece { piece_type: PieceType::Queen, color: Color::White },
        Piece { piece_type: PieceType::Queen, color: Color::Black },
        Piece { piece_type: PieceType::King, color: Color::White },
        Piece { piece_type: PieceType::King, color: Color::Black },
        Piece { piece_type: PieceType::Dragon, color: Color::White },
        Piece { piece_type: PieceType::Dragon, color: Color::Black },
    ];

    pub fn bb_index(&self) -> usize { (self.piece_type as usize * 2) + self.color as usize }
}

pub enum Files {
    FileA,
    FileB,
    FileC,
    FileD,
    FileE,
    FileF,
    FileG,
    FileH,   
}

pub enum Ranks {
    Rank1,
    Rank2,
    Rank3,
    Rank4,
    Rank5,
    Rank6,
    Rank7,
    Rank8,
}

pub enum Castling {
    WhiteKingCastle = 1,
    WhiteQueenCastle = 2,
    BlackKingCastle = 4,
    BlackQueenCastle = 8,
}

pub fn fr2sq(file: usize, rank: usize) -> usize { (rank * 8) + file }

pub static FILES_BOARD: LazyLock<[usize; 64]> = LazyLock::new(|| {
    let mut files_board: [usize; 64] = [0; 64];
    for rank in Ranks::Rank1 as usize..=Ranks::Rank8 as usize {
        for file in Files::FileA as usize..=Files::FileH as usize {
            files_board[fr2sq(file, rank)] = file;
        }
    }
    files_board
});
pub static RANKS_BOARD: LazyLock<[usize; 64]> = LazyLock::new(|| {
    let mut ranks_board: [usize; 64] = [0; 64];
    for rank in Ranks::Rank1 as usize..=Ranks::Rank8 as usize {
        for file in Files::FileA as usize..=Files::FileH as usize {
            ranks_board[fr2sq(file, rank)] = rank;
        }
    }
    ranks_board
});