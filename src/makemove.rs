use std::mem::take;

use colored::*;

use crate::{attack::square_attacked, board::{check_board, debug_board}, data::{IS_KING, IS_PAWN, PIECE_BIG, PIECE_COLOR, PIECE_MAJOR, PIECE_VALUE}, defs::{captured, clear_bit, from_square, print_binary, promoted, set_bit, to_square, Board, Pieces::*, Ranks::*, Squares::*, BOTH, CASTLE_KEYS, DEBUG, MOVE_FLAG_CASTLE, MOVE_FLAG_EN_PASSENT, MOVE_FLAG_PAWN_START, NO_MOVE, PIECE_KEYS, RANKS_BOARD, SIDE_KEY, SQ120_TO_SQ64, WHITE}, hashkeys::generate_position_key, validate::{piece_valid, side_valid, square_on_board}};

/*
1. make move
2. get from, to, capture
3. store current position in position.history array
4. move current piece from -> to
5. if capture was made, remove captured from the piece list
6. update fifty move rule (if pawn or capture was moved/made, reset fifty move rule)
7. promotions
8. en passant captures
9. set en passant square if pawn start
10. for all pieces added, moved removed, update all position counters and piece lists
11. maintain position key
12. castle permission
13. change side, increment ply and history ply
*/

pub fn hash_piece(position: &mut Board, piece: usize, square: usize) { position.position_key ^= PIECE_KEYS[piece as usize][square as usize]; }
pub fn hash_castle(position: &mut Board) { position.position_key ^= CASTLE_KEYS[position.castle_permission as usize]; }
pub fn hash_side(position: &mut Board) { position.position_key ^= *SIDE_KEY; }
pub fn hash_en_passant(position: &mut Board) { position.position_key ^= PIECE_KEYS[Empty as usize][position.en_passent as usize]; }

// 1111 == 15
// castle_permission &= castle_permission[from]
// castle_permission &= 3 -> 0011
const CASTLE_PERMISSION: [u8; 120] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 13, 15, 15, 15, 12, 15, 15, 14, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15,  7, 15, 15, 15,  3, 15, 15, 11, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15
];

pub fn clear_piece(square: usize, position: &mut Board) {
    if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "clear_piece: [square] square not on board".red()); }
    let piece = position.pieces[square] as usize;
    if DEBUG && !piece_valid(piece as usize) { eprintln!("{}", "clear_piece: [piece] piece not valid".red()); }
    let color = PIECE_COLOR[piece as usize] as usize;

    // hash out
    hash_piece(position, piece, square);

    position.pieces[square] = Empty as u8;
    position.material[color] -= PIECE_VALUE[piece];

    if PIECE_BIG[piece] {
        position.big_piece[color] -= 1;

        if PIECE_MAJOR[piece] {
            position.major_piece[color] -= 1;
        } else {
            position.minor_piece[color] -= 1;
        }
    } else {
        clear_bit(&mut position.pawns[color], square as u8);
        clear_bit(&mut position.pawns[BOTH], square as u8);
    }

    /*
    position.piece_number[WhitePawn] == 5
    position.piece_list[WhitePawn][0] == sq0
    position.piece_list[WhitePawn][1] == sq1
    position.piece_list[WhitePawn][2] == sq2
    position.piece_list[WhitePawn][3] == sq3
    position.piece_list[WhitePawn][4] == sq4

    square == square2, temp_piece_number == 2
     */
    let mut temp_piece_number: isize = -1;
    for i in 0..position.piece_number[piece] {
        if position.piece_list[piece][i as usize] == square as u8 {
            temp_piece_number = i as isize;
            break;
        }
    }

    if DEBUG && temp_piece_number == -1 { eprintln!("{}", "clear_piece: temp_piece_number not defined (cannot find square containing piece)".red()); }

    position.piece_number[piece] -= 1; // position.piece_number[WhitePawn] == 4
    position.piece_list[piece][temp_piece_number as usize] = position.piece_list[piece][position.piece_number[piece] as usize];
    // position.piece_list[WhitePawn][2] == position.piece_list[WhitePawn][4] = sq4
    /*
    position.piece_number[WhitePawn] == 4
    position.piece_list[WhitePawn][0] == sq0
    position.piece_list[WhitePawn][1] == sq1
    position.piece_list[WhitePawn][2] == sq4
    position.piece_list[WhitePawn][3] == sq3
    */
}

pub fn add_piece(square: usize, position: &mut Board, piece: usize) {
    if DEBUG {
        if !square_on_board(square as usize) { eprintln!("{}", "add_piece: [square] square not on board".red()); }
        if !piece_valid(piece as usize) { eprintln!("{}", "add_piece: [piece] piece not valid".red()); }
    }

    let color = PIECE_COLOR[piece as usize] as usize;

    // hash in
    hash_piece(position, piece, square);

    position.pieces[square] = piece as u8;

    if PIECE_BIG[piece] {
        position.big_piece[color] += 1;

        if PIECE_MAJOR[piece] {
            position.major_piece[color] += 1;
        } else {
            position.minor_piece[color] += 1;
        }
    } else {
        set_bit(&mut position.pawns[color], square as u8);
        set_bit(&mut position.pawns[BOTH], square as u8);
    }

    position.material[color] += PIECE_VALUE[piece];
    position.piece_list[piece][position.piece_number[piece] as usize] = square as u8;
    position.piece_number[piece] += 1;
}

pub fn move_piece(from: u8, to: u8, position: &mut Board) {
    if DEBUG {
        if !square_on_board(from as usize) { eprintln!("{}", "move_piece: [from] square not on board".red()); }
        if !square_on_board(to as usize) { eprintln!("{}", "move_piece: [to] square not on board".red()); }
    }

    let piece = position.pieces[from as usize] as usize;
    let color = PIECE_COLOR[piece] as usize;

    // hash out
    hash_piece(position, piece, from as usize);
    position.pieces[from as usize] = Empty as u8;

    // hash in
    hash_piece(position, piece, to as usize);
    position.pieces[to as usize] = piece as u8;

    // if pawn
    if !PIECE_BIG[piece] {
        clear_bit(&mut position.pawns[color], from);
        clear_bit(&mut position.pawns[BOTH], from);
        set_bit(&mut position.pawns[color], to);
        set_bit(&mut position.pawns[BOTH], to);
    }

    for i in 0..position.piece_number[piece] {
        if position.piece_list[piece][i as usize] == from {
            position.piece_list[piece][i as usize] = to;
            break;
        }
    }
}

pub fn make_move(position: &mut Board, the_move: u64) -> bool {
    if DEBUG { check_board(position); }

    let from = from_square(the_move);
    let to = to_square(the_move);
    let side = position.side;

    if DEBUG {
        if !square_on_board(from as usize) { eprintln!("{}", "make_move: [from] square not on board".red()); }
        if !square_on_board(to as usize) { eprintln!("{}", "make_move: [to] square not on board".red()); }
        if !side_valid(side as usize) { eprintln!("{}", "make_move: [side] side not valid".red()); }
        if !piece_valid(position.pieces[from as usize] as usize) { eprintln!("{}", "make_move: [position.pieces[from]] piece not valid".red()); }
    }

    // store hashkey
    position.history[position.history_ply].position_key = position.position_key;

    // if en passant
    if the_move & MOVE_FLAG_EN_PASSENT != 0 {
        if side == WHITE as u8 {
            // to square is diagonal, +-10 gets the square behind/infront of it
            clear_piece(to as usize - 10, position);
        } else {
            clear_piece(to as usize + 10, position);
        }
    // if castle
    } else if the_move & MOVE_FLAG_CASTLE != 0 {
        if to == C1 as u64 {
            move_piece(A1 as u8, D1 as u8, position);
        } else if to == C8 as u64 {
            move_piece(A8 as u8, D8 as u8, position);
        } else if to == G1 as u64 {
            move_piece(H1 as u8, F1 as u8, position);
        } else if to == G8 as u64 {
            move_piece(H8 as u8, F8 as u8, position);
        } else {
            eprintln!("{}", "make_move: castle problem".red());
        }
    }

    // if en passent is set, then hash out
    if position.en_passent != NoSq as u8 { hash_en_passant(position); }
    // hash out current castle
    hash_castle(position);

    position.history[position.history_ply].the_move = the_move;
    position.history[position.history_ply].fifty_move = position.fifty_move;
    position.history[position.history_ply].en_passent = position.en_passent;
    position.history[position.history_ply].castle_permission = position.castle_permission;

    position.castle_permission &= CASTLE_PERMISSION[from as usize];
    position.castle_permission &= CASTLE_PERMISSION[to as usize];
    position.en_passent = NoSq as u8;

    hash_castle(position);

    position.fifty_move += 1;

    let captured = captured(the_move);
    if captured != Empty as u64 {
        if DEBUG && !piece_valid(captured as usize) { eprintln!("make_move: [captured] piece not valid"); }

        clear_piece(to as usize, position);
        position.fifty_move = 0;
    }

    position.history_ply += 1;
    position.ply += 1;

    if IS_PAWN[position.pieces[from as usize] as usize] {
        position.fifty_move = 0;
        // if pawn start, set en passant square
        if the_move & MOVE_FLAG_PAWN_START != 0 {
            if side == WHITE as u8 {
                position.en_passent = (from + 10) as u8;
                if DEBUG && RANKS_BOARD[position.en_passent as usize] != Rank3 as u8 { eprintln!("{}", "make_move: white en passant not on proper rank".red()); }
            } else {
                position.en_passent = (from - 10) as u8;
                if DEBUG && RANKS_BOARD[position.en_passent as usize] != Rank6 as u8 { eprintln!("{}", "make_move: black en passant not on proper rank".red()); }
            }
            hash_en_passant(position);
        }
    }

    move_piece(from as u8, to as u8, position);

    // promotion
    let promote_piece = promoted(the_move);
    if promote_piece != Empty as u64 {
        if DEBUG && !piece_valid(promote_piece as usize) { eprintln!("{}", "make_move: [promote_piece] piece not valid".red()); }
        clear_piece(to as usize, position);
        add_piece(to as usize, position, promote_piece as usize);
    }

    // king square
    if IS_KING[position.pieces[to as usize] as usize] { position.king_square[position.side as usize] = to as u8; }

    position.side ^= 1;
    hash_side(position);

    if DEBUG { check_board(position); }

    if square_attacked(position.king_square[side as usize] as usize, position.side as usize, position) {
        take_move(position);
        return false;
    }

    true
}

pub fn take_move(position: &mut Board) {
    if DEBUG { check_board(position); }

    position.history_ply -= 1;
    position.ply -= 1;

    let the_move = position.history[position.history_ply].the_move;
    let from = from_square(the_move as u64);
    let to = to_square(the_move as u64);

    if DEBUG {
        if !square_on_board(from as usize) { eprintln!("{}", "take_move: [from] square not on board".red()); }
        if !square_on_board(to as usize) { eprintln!("{}", "take_move: [to] square not on board".red()); }
    }

    if position.en_passent != NoSq as u8 {
        hash_en_passant(position); }
    hash_castle(position);

    position.castle_permission = position.history[position.history_ply].castle_permission;
    position.fifty_move = position.history[position.history_ply].fifty_move;
    position.en_passent = position.history[position.history_ply].en_passent;

    if position.en_passent != NoSq as u8 {
        hash_en_passant(position); }
    hash_castle(position);

    position.side ^= 1;
    hash_side(position);

    // if en passant
    if the_move & MOVE_FLAG_EN_PASSENT != 0 {
        if position.side == WHITE as u8 {
            // to square is diagonal, +-10 gets the square behind/infront of it
            add_piece(to as usize - 10, position, BlackPawn as usize);
        } else {
            add_piece(to as usize + 10, position, WhitePawn as usize);
        }
    // if castle
    } else if the_move & MOVE_FLAG_CASTLE != 0 {
        if to == C1 as u64 {
            move_piece(D1 as u8, A1 as u8, position);
        } else if to == C8 as u64 {
            move_piece(D8 as u8, A8 as u8, position);
        } else if to == G1 as u64 {
            move_piece(F1 as u8, H1 as u8, position);
        } else if to == G8 as u64 {
            move_piece(F8 as u8, H8 as u8, position);
        } else {
            eprintln!("{}", "take_move: castle problem".red());
        }
    }

    move_piece(to as u8, from as u8, position);

    // king square
    if IS_KING[position.pieces[from as usize] as usize] { position.king_square[position.side as usize] = from as u8; }

    let captured = captured(the_move);
    if captured != Empty as u64 {
        if DEBUG && !piece_valid(captured as usize) { eprintln!("take_move: [captured] piece not valid"); }
        add_piece(to as usize, position, captured as usize);
    }

    // promotion
    let promote_piece = promoted(the_move);
    let pawn_type: u8;
    if PIECE_COLOR[promote_piece as usize] == WHITE as u8 {
        pawn_type = WhitePawn as u8;
    } else {
        pawn_type = BlackPawn as u8;
    }
    if promote_piece != Empty as u64 {
        if DEBUG && !piece_valid(promote_piece as usize) && !IS_PAWN[promote_piece as usize] { eprintln!("{}", "take_move: [promote_piece] piece not valid".red()); }
        clear_piece(from as usize, position);
        add_piece(from as usize, position, pawn_type as usize);
    }
    if DEBUG { check_board(position); }
}

pub fn make_null_move(position: &mut Board) {
    if DEBUG { check_board(position); }
    let in_check = square_attacked(position.king_square[position.side as usize] as usize, (position.side ^ 1) as usize, position);
    if in_check { return }

    position.ply += 1;
    position.history[position.history_ply].position_key = position.position_key;

    if position.en_passent != NoSq as u8 { hash_en_passant(position); }

    position.history[position.history_ply].the_move = NO_MOVE;
    position.history[position.history_ply].fifty_move = position.fifty_move;
    position.history[position.history_ply].en_passent = position.en_passent;
    position.history[position.history_ply].castle_permission = position.castle_permission;
    position.en_passent = NoSq as u8;

    position.side ^= 1;
    position.history_ply += 1;
    hash_side(position);

    if DEBUG { check_board(position); }
}

pub fn take_null_move(position: &mut Board) {
    if DEBUG { check_board(position); }

    position.history_ply -= 1;
    position.ply -= 1;

    if position.en_passent != NoSq as u8 { hash_en_passant(position); }

    position.castle_permission = position.history[position.history_ply].castle_permission;
    position.fifty_move = position.history[position.history_ply].fifty_move;
    position.en_passent = position.history[position.history_ply].en_passent;

    if position.en_passent != NoSq as u8 { hash_en_passant(position); }
    position.side ^= 1;
    hash_side(position);

    if DEBUG { check_board(position); }
}
