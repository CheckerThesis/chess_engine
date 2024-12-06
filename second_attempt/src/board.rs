use std::collections::HashMap;
use crate::{data::{PIECE_BIG, PIECE_COLOR, PIECE_MAJOR, PIECE_MINOR, PIECE_VALUE}, defs::{fr2sq, set_bit, sq120, sq64, Board, Castling::{*}, Castling::{*}, Files::*, Pieces::{self, *}, Ranks::*, Sides::*, Squares::{NoSq, OffBoard}, BOARD_SQUARE_NUMBER, SQ64_TO_SQ120}, hashkeys::generate_position_key};
use crate::data::{PIECE_CHAR, SIDE_CHAR, RANK_CHAR, FILE_CHAR};

pub fn update_lists_material(position: &mut Board) {
    for square in 0..BOARD_SQUARE_NUMBER {
        let piece = position.pieces[square] as usize;

        // if piece
        if piece != OffBoard as usize && piece != Empty as usize {
            let color = PIECE_COLOR[piece] as usize;

            // if data array true, increment the corresponding color in the board struct
            if PIECE_BIG[piece] {
                position.big_piece[color] += 1;
            }
            if PIECE_MINOR[piece] {
                position.minor_piece[color] += 1;
            }
            if PIECE_MAJOR[piece] {
                position.major_piece[color] += 1;
            }
            position.material[color] += PIECE_VALUE[piece];

            // first white pawn: piece_list[WhitePawn][0] = A1
            // second white pawn: piece_list[WhitePawn][1] = A2
            position.piece_list[piece][position.piece_number[piece] as usize] = square as u8;
            position.piece_number[piece] += 1;

            if piece == WhiteKing as usize {
                position.king_square[color] = square as u8;
            }
            if piece == BlackKing as usize {
                position.king_square[color] = square as u8;
            }

            // if pawn, set the bit (in the pawn bitboard) corresponding to the square to 1
            if piece == WhitePawn as usize {
                set_bit(&mut position.pawns[White as usize], square as u8);
                set_bit(&mut position.pawns[Both as usize], square as u8);
            } else if piece == BlackPawn as usize {
                set_bit(&mut position.pawns[Black as usize], square as u8);
                set_bit(&mut position.pawns[Both as usize], square as u8);
            }
        }
    }
}

// Result<(), &'static str> means returns nothing: (), or error in string: &'static str
pub fn parse_fen(fen: &str, position: &mut Board) -> Result<(), &'static str> {
    reset_board(position);

    let piece_map: HashMap<char, Pieces> = HashMap::from([
        ('p', BlackPawn), ('r', BlackRook), ('n', BlackKnight),
        ('b', BlackBishop), ('k', BlackKing), ('q', BlackQueen),
        ('P', WhitePawn), ('R', WhiteRook), ('N', WhiteKnight),
        ('B', WhiteBishop), ('K', WhiteKing), ('Q', WhiteQueen),
    ]);

    // vector is a linkedlist but back-to-back (essentially an expandable array)
    let fen_split: Vec<&str> = fen.split_whitespace().collect();
    if fen_split.len() < 4 {
        // return Err("Invalid FEN string")
    }

    let mut rank = Rank8 as usize;
    let mut file = FileA as usize;

    // assign board all fen attributes
    for c in fen_split[0].chars() {
        // piece_map.get(&c) will return either Some(value) (if key exists) or None (if key doesn't exist)
        // if piece_map.get(&c) does not equal None, it will be assigned to piece and excecute the block
        // else it will skip it
        // if piece/char
        if let Some(piece) = piece_map.get(&c) {
            let sq64 = fr2sq(file as u8, rank as u8);

            position.pieces[sq64 as usize] = *piece as u8;
            file += 1;
        // .to_digit turns c into base 10
        // if empty/integer
        } else if let Some(empty_squares) = c.to_digit(10) {
            file += empty_squares as usize;
        // if new rank/'/'
        } else if c == '/' {
            if file != 8 {
                println!("Ello");
                return Err("Incorrect amount of char in file");
            }
            // .checked_sub(1) checks if the subtraction (sub) of 1 is ok on rank,
            // if so it assigns rank - 1 to rank else it throws
            // ? unwraps (it was an Option(usize, err)) if not error, else propogates error
            rank = rank.checked_sub(1).ok_or("Rank underflow")?;
            file = FileA as usize;
        } else {
            return Err("Invalid character in FEN");
        }
    }

    position.side = match fen_split[1] {
        "w" => White as u8,
        "b" => Black as u8,
        _ => return Err("Invalid FEN part 2")
    };

    // 00000000 |= 4 == 00000100
    for i in fen_split[2].chars() {
        match i {
            'K' => position.castle_permission |= WhiteKingCastle as u8,
            'Q' => position.castle_permission |= WhiteQueenCastle as u8,
            'k' => position.castle_permission |= BlackKingCastle as u8,
            'q' => position.castle_permission |= BlackQueenCastle as u8,
            '-' => break,
            _ => return Err("Invalid castling permission in FEN"),
        }
    }

    if fen_split[3] != "-" {
        // .chars() converts String into char iterator
        // .next() gets the next (first in this case) element in the char iterator
        // .unwrap() unwraps Option<char>
        // subtracts result (lowercase letter a-h) by ascii 'a' (97)
        let file = fen_split[3].chars().next().unwrap() as usize - 'a' as usize;
        let rank = fen_split[3].chars().nth(1).unwrap().to_digit(10).unwrap() as usize - 1;
        position.en_passent = fr2sq(file as u8, rank as u8);
    }

    position.position_key = generate_position_key(position);
    update_lists_material(position);
    Ok(())
}

pub fn reset_board(position: &mut Board) {
    // set all squares to off board
    for i in 0..BOARD_SQUARE_NUMBER {
        position.pieces[i] = OffBoard as u8;
    }

    // set small board squares to empty
    for i in 0..64 {
        position.pieces[SQ64_TO_SQ120[i] as usize] = Empty as u8;
    }

    for i in 0..2 {
        position.big_piece[i] = 0;
        position.major_piece[i] = 0;
        position.minor_piece[i] = 0;
        position.pawns[i] = 0;
        position.material[i] = 0;
    }
    position.pawns[2] = 0;

    for i in 0..13 {
        position.piece_number[i] = 0;
    }

    position.king_square[White as usize] = NoSq as u8;
    position.king_square[Black as usize] = NoSq as u8;

    position.side = Both as u8;
    position.en_passent = NoSq as u8;
    position.fifty_move = 0;

    position.ply = 0;
    position.history_ply = 0;

    position.castle_permission = 0;

    position.position_key = 0;
}

pub fn debug_board(position: &mut Board) {
    print!("Debug board:");

    for i in 0..BOARD_SQUARE_NUMBER {
        if i % 10 == 0 {
            println!();
        }
        // {:>3} is a format specifier that assuers whatever printed has a width of 3
        // and is right side aligned
        print!("{:>3} ", position.pieces[i]);
    }
    println!();

    let mut count = 0;
    for i in 0..BOARD_SQUARE_NUMBER {
        if i % 10 == 0 {
            println!();
        }
        // {:>3} is a format specifier that assuers whatever printed has a width of 3
        // and is right side aligned
        print!("{:>3} ", count);
        count += 1;
    }
    println!("\n");
}

pub fn print_board(position: &mut Board) {
    println!("Game board:");

    for rank in (Rank1 as u8..=Rank8 as u8).rev() {
        print!("{}  ", (rank + 1));
        for file in FileA as u8..=FileH as u8 {
            let sq: u8 = fr2sq(file, rank);
            let piece = position.pieces[sq as usize];
            print!("{:>2}", PIECE_CHAR.chars().nth(piece as usize).unwrap_or(' '));
        }
        println!();
    }
    println!();
    print!("   ");
    for file in b'A'..=b'H' {
        print!("{:>2}", file as char);
    }
    println!();

    println!("Side: {}\nEnpassent: {}", SIDE_CHAR.chars().nth(position.side as usize).unwrap(), position.en_passent);
    let wkc = if position.castle_permission & WhiteKingCastle as u8 != 0 {"K"} else {"-"};
    let wqc = if position.castle_permission & WhiteQueenCastle as u8 != 0 {"Q"} else {"-"};
    let bkc = if position.castle_permission & BlackKingCastle as u8 != 0 {"k"} else {"-"};
    let bqc = if position.castle_permission & BlackQueenCastle as u8 != 0 {"q"} else {"-"};

    println!("Castle permission: {}{}{}{}\nPosition key: {:X}", wkc, wqc, bkc, bqc, position.position_key);
}
