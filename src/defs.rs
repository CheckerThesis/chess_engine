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

pub const WHITE_KING_CASTLE:  u8 = 0b0001;
pub const WHITE_QUEEN_CASTLE: u8 = 0b0010;
pub const BLACK_KING_CASTLE:  u8 = 0b0100;
pub const BLACK_QUEEN_CASTLE: u8 = 0b1000;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Color(pub u8);
impl Color {
    pub const WHITE:   Self = Self(0); // 00
    pub const BLACK:   Self = Self(1); // 01
    pub const NEITHER: Self = Self(2); // 10
    pub const BOTH:    Self = Self(3); // 11

    pub fn opposite(&self) -> Color { Self(self.0 ^ 1) }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct PieceType(pub u8);
impl PieceType {
    pub const NONE:   Self = Self(0);
    pub const PAWN:   Self = Self(1);
    pub const KNIGHT: Self = Self(2);
    pub const BISHOP: Self = Self(3);
    pub const ROOK:   Self = Self(4);
    pub const QUEEN:  Self = Self(5);
    pub const KING:   Self = Self(6);
    pub const DRAGON: Self = Self(7);

    #[inline(always)] pub fn index(&self) -> usize { self.0 as usize }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Piece(pub u8);
impl Piece { //                           PieceType Color
    pub const NONE:         Self = Self(0);  // 000 00
    pub const WHITE_PAWN:   Self = Self(4);  // 001 00
    pub const BLACK_PAWN:   Self = Self(5);  // 001 01
    pub const WHITE_KNIGHT: Self = Self(8);  // 010 00
    pub const BLACK_KNIGHT: Self = Self(9);  // 010 01
    pub const WHITE_BISHOP: Self = Self(12); // 011 00
    pub const BLACK_BISHOP: Self = Self(13); // 011 01
    pub const WHITE_ROOK:   Self = Self(16); // 100 00 
    pub const BLACK_ROOK:   Self = Self(17); // 100 01
    pub const WHITE_QUEEN:  Self = Self(20); // 101 00
    pub const BLACK_QUEEN:  Self = Self(21); // 101 01
    pub const WHITE_KING:   Self = Self(24); // 110 00
    pub const BLACK_KING:   Self = Self(25); // 110 01
    pub const WHITE_DRAGON: Self = Self(28); // 111 00
    pub const BLACK_DRAGON: Self = Self(29); // 111 01

    pub const COUNT: usize = 32;

    #[inline(always)] pub fn index(&self) -> usize { self.0 as usize }
    #[inline(always)] pub fn piece_type(&self) -> PieceType { PieceType(self.0 >> 2) }
    #[inline(always)] pub fn color(&self) -> Color { Color(self.0 & 0b11) }
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