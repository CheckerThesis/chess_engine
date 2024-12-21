use std::collections::HashMap;
use crate::{bitboards::{count_bits, pop_bit}, data::{PIECE_BIG, PIECE_COLOR, PIECE_MAJOR, PIECE_MINOR, PIECE_VALUE}, defs::{fr2sq, set_bit, sq120, sq64, Board, Castling::{*}, Castling::{*}, Files::*, Pieces::{self, *}, Ranks::*, Squares::{NoSq, OffBoard}, BLACK, BOARD_SQUARE_NUMBER, BOTH, RANKS_BOARD, SQ64_TO_SQ120, WHITE}, hashkeys::generate_position_key};
use crate::data::{PIECE_CHAR, SIDE_CHAR, RANK_CHAR, FILE_CHAR};

// fill temp variables with current, then compare them with the values filled
pub fn check_board(position: &mut Board) -> Result<(), &'static str> {
    // fill these values with position values, at the end see if they
    let mut temp_piece_number: [u8; 13] = [0; 13];
    let mut temp_big_piece: [u8; 2] = [0; 2];
    let mut temp_major_piece: [u8; 2] = [0; 2];
    let mut temp_minor_piece: [u8; 2] = [0; 2];
    let mut temp_material: [u16; 2] = [0; 2];

    let mut temp_pawns: [u64; 3] = [0; 3];
    temp_pawns[WHITE] = position.pawns[WHITE];
    temp_pawns[BLACK] = position.pawns[BLACK];
    temp_pawns[BOTH] = position.pawns[BOTH];

    // temp_piece = piece type
    for temp_piece in WhitePawn as u8..BlackKing as u8 {
        // temp_piece_num = number of pieces for that type
        for temp_piece_num in 0..position.piece_number[temp_piece as usize] {
            let square120 = position.piece_list[temp_piece as usize][temp_piece_num as usize];
            // if piece at pieces array != temp_piece
            if position.pieces[square120 as usize] != temp_piece {
                return Err("Check piece lists error")
            }
        }
    }

    for square64 in 0..64 {
        let square120 = sq120(square64 as u8);

        let piece: usize = position.pieces[square120 as usize] as usize;
        temp_piece_number[piece] += 1;
        let color: usize = PIECE_COLOR[piece] as usize;

        if PIECE_BIG[piece] {
            temp_big_piece[color] += 1;
        }
        if PIECE_MINOR[piece] {
            temp_minor_piece[color] += 1;
        }
        if PIECE_MAJOR[piece] {
            temp_major_piece[color] += 1;
        }

        if color != BOTH {
            temp_material[color] += PIECE_VALUE[piece];
        }

    }

    for piece in WhitePawn as usize..BlackKing as usize {
        if temp_piece_number[piece] != position.piece_number[piece] {
            return Err("Piece number error")
        }
    }

    let mut pawn_count = count_bits(temp_pawns[WHITE]);
    if pawn_count != position.piece_number[WhitePawn as usize] as u64 {
        return Err("White pawn count error")
    }
    pawn_count = count_bits(temp_pawns[BLACK]);
    if pawn_count != position.piece_number[BlackPawn as usize] as u64 {
        return Err("Black pawn count error")
    }
    pawn_count = count_bits(temp_pawns[BOTH]);
    if pawn_count != (position.piece_number[WhitePawn as usize] + position.piece_number[BlackPawn as usize]) as u64 {
        return Err("Both pawn count error")
    }

    // while bitboard not empty
    // if pieces array at the bitboard position does not have a pawn
    while temp_pawns[WHITE] != 0 {
        let square64 = pop_bit(&mut temp_pawns[WHITE]);

        if position.pieces[sq120(square64 as u8) as usize] != WhitePawn as u8 {
            return Err("WhitePawn bitboard error")
        }
    }
    while temp_pawns[BLACK] != 0 {
        let square64 = pop_bit(&mut temp_pawns[BLACK]);

        if position.pieces[sq120(square64 as u8) as usize] != BlackPawn as u8 {
            return Err("BlackPawn bitboard error")
        }
    }
    while temp_pawns[BOTH] != 0 {
        let square64 = pop_bit(&mut temp_pawns[BOTH]);
        // when square64 = 8, sq120(8) = 31
        // let test = position.pieces[31];

        if (position.pieces[sq120(square64 as u8) as usize] != BlackPawn as u8) &&
        (position.pieces[sq120(square64 as u8) as usize] != WhitePawn as u8) {
            return Err("BothPawn bitboard error")
        }
    }

    if temp_material[WHITE] != position.material[WHITE] && temp_material[BLACK] != position.material[BLACK] {
        return Err("Material value error")
    }
    if temp_major_piece[WHITE] != position.major_piece[WHITE] && temp_major_piece[BLACK] != position.major_piece[BLACK] {
        return Err("Major piece count error")
    }
    if temp_minor_piece[WHITE] != position.minor_piece[WHITE] && temp_minor_piece[BLACK] != position.minor_piece[BLACK] {
        return Err("Minor piece count error")
    }
    if temp_big_piece[WHITE] != position.big_piece[WHITE] && temp_big_piece[BLACK] != position.big_piece[BLACK] {
        return Err("Big piece count error")
    }

    if position.side != WHITE as u8 && position.side != BLACK as u8 {
        return Err("Side error")
    }
    if generate_position_key(position) != position.position_key {
        return Err("Position key error")
    }

    // if en_passent isn't NoSq and a position on Rank6/Rank3 (corresponding to side)
    if position.en_passent != NoSq as u8 &&
    ((RANKS_BOARD[position.en_passent as usize] != Rank6 as u8 && position.side == WHITE as u8) ||
    (RANKS_BOARD[position.en_passent as usize] != Rank3 as u8 && position.side == BLACK as u8)) {
        return Err("En_passent error")
    }

    if position.pieces[position.king_square[WHITE as usize] as usize] != WhiteKing as u8 {
        return Err("White king square error")
    }
    if position.pieces[position.king_square[BLACK as usize] as usize] != BlackKing as u8 {
        return Err("Black king square error")
    }

    Ok(())
}

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
                set_bit(&mut position.pawns[WHITE], square as u8);
                set_bit(&mut position.pawns[BOTH], square as u8);
            } else if piece == BlackPawn as usize {
                set_bit(&mut position.pawns[BLACK], square as u8);
                set_bit(&mut position.pawns[BOTH], square as u8);
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
        "w" => WHITE as u8,
        "b" => BLACK as u8,
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

    position.king_square[WHITE] = NoSq as u8;
    position.king_square[BLACK] = NoSq as u8;

    position.side = BOTH as u8;
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
