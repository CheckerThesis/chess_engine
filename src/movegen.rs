
/*
fn move_gen(board, moveList)
    loop all pieces
        if slider then loop each dir and add move
            Add move moveList->moves[moveList->moves] = move
            moveList->count++
*/

use colored::Colorize;

use crate::{attack::square_attacked, board::check_board, data::{PIECE_CHAR, PIECE_COLOR}, defs::{captured, from_square, to_square, Board, Castling::*, MoveList, Pieces::{self, *}, Ranks::*, Squares::*, DEBUG, FILES_BOARD, MOVE_FLAG_CASTLE, MOVE_FLAG_EN_PASSENT, MOVE_FLAG_PAWN_START, MVV_LVA_SCORES, RANKS_BOARD}, io::print_square, makemove::{make_move, take_move}, validate::{piece_valid, piece_valid_empty, square_on_board}, BLACK, WHITE};

pub fn move_builder(from: u64, to: u64, capture: u64, promote: u64, flag: u64) -> u64 { from | (to << 7) | (capture << 14) | (promote << 20) | flag }

pub fn move_exists(position: &mut Board, the_move: u64) -> bool {
    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    for move_number in 0..move_list.count {
        if !make_move(position, move_list.moves[move_number].el_move) {
            continue;
        }

        take_move(position);

        if move_list.moves[move_number].el_move == the_move {
            return true
        }
    }

    return false
}

// add move to array and increment
pub fn add_quiet_move(position: &mut Board, the_move: u64, move_list: &mut MoveList) {
    if DEBUG {
        if !square_on_board(from_square(the_move) as usize) { eprintln!("{}", "add_quiet_move: [from] square not on board".red()); }
        if !square_on_board(to_square(the_move) as usize) { eprintln!("{}", "add_quiet_move: [to] square not on board".red()); }
    }

    move_list.moves[move_list.count].el_move = the_move;

    // update non-capture moves that give beta cut-offs
    if position.search_killers[0][position.ply as usize] == the_move {
        move_list.moves[move_list.count].score = 900000;
    } else if position.search_killers[1][position.ply as usize] == the_move {
        move_list.moves[move_list.count].score = 800000;
    } else {
        move_list.moves[move_list.count].score = position.search_history[position.pieces[from_square(the_move) as usize] as usize][to_square(the_move) as usize];
    }

    move_list.count += 1;
}

pub fn add_capture_move(position: &mut Board, the_move: u64, move_list: &mut MoveList) {
    if DEBUG {
        if !piece_valid_empty(captured(the_move) as usize) { eprintln!("{}", "add_capture_move: [capture] piece isn't empty or a piece".red()); }
        if !square_on_board(from_square(the_move) as usize) { eprintln!("{}", "add_capture_move: [from] square not on board".red()); }
        if !square_on_board(to_square(the_move) as usize) { eprintln!("{}", "add_capture_move: [to] square not on board".red()); }
    }

    move_list.moves[move_list.count].el_move = the_move;
    // add 1 million so they are above killer moves (8/900000), history heuristics, pv_moves for move ordering
    move_list.moves[move_list.count].score = MVV_LVA_SCORES[captured(the_move) as usize][position.pieces[from_square(the_move) as usize] as usize] + 1000000;
    move_list.count += 1;
}

pub fn add_en_passent_move(_position: &mut Board, the_move: u64, move_list: &mut MoveList) {
    if DEBUG {
        if !piece_valid_empty(captured(the_move) as usize) { eprintln!("{}", "add_en_passent_move: [capture] piece isn't empty or a piece".red()); }
        if !square_on_board(from_square(the_move) as usize) { eprintln!("{}", "add_en_passent_move: [from] square not on board".red()); }
        if !square_on_board(to_square(the_move) as usize) { eprintln!("{}", "add_en_passent_move: [to] square not on board".red()); }
    }

    move_list.moves[move_list.count].el_move = the_move;
    // add 1 million so they are above killer moves and history heuristics and pv_moves for move ordering
    move_list.moves[move_list.count].score = 105 + 1000000;
    move_list.count += 1;
}

pub fn add_white_pawn_capture_move(position: &mut Board, from: usize, to: usize, capture: usize, move_list: &mut MoveList) {
    if DEBUG {
        if !piece_valid_empty(capture) { eprintln!("{}", "add_white_pawn_capture_move: [capture] piece isn't empty or a piece".red()); }
        if !square_on_board(from as usize) { eprintln!("{}", "add_white_pawn_capture_move: [from] square not on board".red()); }
        if !square_on_board(to as usize) { eprintln!("{}", "add_white_pawn_capture_move: [to] square not on board".red()); }
    }

    // if able to promote on capture
    if RANKS_BOARD[from as usize] == Rank7 as u8 {
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, WhiteQueen as u64, 0), move_list);
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, WhiteRook as u64, 0), move_list);
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, WhiteBishop as u64, 0), move_list);
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, WhiteKnight as u64, 0), move_list);
    } else {
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, Empty as u64, 0), move_list);
    }
}
pub fn add_white_pawn_move(position: &mut Board, from: usize, to: usize, move_list: &mut MoveList) {
    if DEBUG {
        if !square_on_board(from as usize) { eprintln!("{}", "add_white_pawn_move: [from] square not on board".red()); }
        if !square_on_board(to as usize) { eprintln!("{}", "add_white_pawn_move: [to] square not on board".red()); }
    }

    // if WhitePawn is on Rank7, it will promote on capture
    if RANKS_BOARD[from as usize] == Rank7 as u8 {
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, WhiteQueen as u64, 0), move_list);
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, WhiteRook as u64, 0), move_list);
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, WhiteBishop as u64, 0), move_list);
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, WhiteKnight as u64, 0), move_list);
    } else {
        add_capture_move(position, move_builder(from as u64, to as u64, Empty as u64, Empty as u64, 0), move_list);
    }
}

pub fn add_black_pawn_capture_move(position: &mut Board, from: usize, to: usize, capture: usize, move_list: &mut MoveList) {
    if DEBUG {
        if !piece_valid_empty(capture) { eprintln!("{}", "add_black_pawn_capture_move: [capture] piece isn't empty or a piece".red()); }
        if !square_on_board(from as usize) { eprintln!("{}", "add_black_pawn_capture_move: [from] square not on board".red()); }
        if !square_on_board(to as usize) { eprintln!("{}", "add_black_pawn_capture_move: [to] square not on board".red()); }
    }

    // if able to promote on capture
    if RANKS_BOARD[from as usize] == Rank2 as u8 {
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, BlackQueen as u64, 0), move_list);
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, BlackRook as u64, 0), move_list);
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, BlackBishop as u64, 0), move_list);
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, BlackKnight as u64, 0), move_list);
    } else {
        add_capture_move(position, move_builder(from as u64, to as u64, capture as u64, Empty as u64, 0), move_list);
    }
}
pub fn add_black_pawn_move(position: &mut Board, from: usize, to: usize, move_list: &mut MoveList) {
    if DEBUG {
        if !square_on_board(from as usize) { eprintln!("{}", "add_black_pawn_move: [from] square not on board".red()); }
        if !square_on_board(to as usize) { eprintln!("{}", "add_black_pawn_move: [to] square not on board".red()); }
    }

    // if BlackPawn is on Rank2, it will promote on capture
    if RANKS_BOARD[from as usize] == Rank2 as u8 {
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, BlackQueen as u64, 0), move_list);
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, BlackRook as u64, 0), move_list);
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, BlackBishop as u64, 0), move_list);
        add_quiet_move(position, move_builder(from as u64, to as u64, Empty as u64, BlackKnight as u64, 0), move_list);
    } else {
        add_capture_move(position, move_builder(from as u64, to as u64, Empty as u64, Empty as u64, 0), move_list);
    }
}

pub fn generate_all_moves(position: &mut Board, move_list: &mut MoveList) {
    if DEBUG { check_board(position); }

    move_list.count = 0;
    let side = position.side;

    // pawn movement
    if side == WHITE as u8 {
        // loop through each WhitePawn
        for piece_number in 0..position.piece_number[WhitePawn as usize] {
            let square: usize = position.piece_list[WhitePawn as usize][piece_number as usize] as usize;
            if DEBUG && !square_on_board(square) { eprintln!("{}", "generate_all_moves: [square] square not on board (WhitePawn movement)".red()); }

            // Non-capture moves:
            // if square in front is empty
            if position.pieces[square + 10] == Empty as u8 {
                add_white_pawn_move(position, square, square + 10, move_list);

                // if 2 squares in front is empty
                if RANKS_BOARD[square] == Rank2 as u8 && position.pieces[square + 20] == Empty as u8 {
                    add_quiet_move(position, move_builder(
                        square as u64,
                        (square + 20) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_PAWN_START),
                        move_list
                    );
                }
            }

            // Capture moves:
            // if top left
            if FILES_BOARD[square + 9] != OffBoard as u8 && PIECE_COLOR[position.pieces[square + 9] as usize] == BLACK as u8 {
                add_white_pawn_capture_move(position, square, square + 9, position.pieces[square + 9] as usize, move_list);
            }
            // if top right
            if FILES_BOARD[square + 11] != OffBoard as u8 && PIECE_COLOR[position.pieces[square + 11] as usize] == BLACK as u8 {
                add_white_pawn_capture_move(position, square, square + 11, position.pieces[square + 11] as usize, move_list);
            }

            // if en_passent exists
            if position.en_passent != NoSq as u8 {
                if square + 9 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square + 9) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
                if square + 11 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square + 11) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
            }
        }

        // Castling:
        // if castle_permission bitwise & the bit
        if position.castle_permission as u8 & WhiteKingCastle as u8 != 0 {
            // squares between are empty
            if position.pieces[F1 as usize] == Empty as u8 &&
            position.pieces[G1 as usize] == Empty as u8 {
                // if the king is not being attacked
                if !square_attacked(E1 as usize, BLACK, position) &&
                !square_attacked(F1 as usize, BLACK, position) {
                    // println!("White king castle");
                    add_quiet_move(position, move_builder(
                        E1 as u64,
                        G1 as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_CASTLE),
                        move_list
                    );
                }
            }
        }
        if position.castle_permission as u8 & WhiteQueenCastle as u8 != 0 {
            // squares between are empty
            if position.pieces[D1 as usize] == Empty as u8 &&
            position.pieces[C1 as usize] == Empty as u8 &&
            position.pieces[B1 as usize] == Empty as u8 {
                // if the king is not being attacked
                if !square_attacked(E1 as usize, BLACK, position) &&
                !square_attacked(D1 as usize, BLACK, position) {
                    // println!("White queen castle");
                    add_quiet_move(position, move_builder(
                        E1 as u64,
                        C1 as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_CASTLE),
                        move_list
                    );
                }
            }
        }
    } else {
        // loop through each BlackPawn
        for piece_number in 0..position.piece_number[BlackPawn as usize] {
            let square: usize = position.piece_list[BlackPawn as usize][piece_number as usize] as usize;
            if DEBUG && !square_on_board(square) { eprintln!("{}", "generate_all_moves: [square] square not on board (BlackPawn movement)".red()); }

            // Non-capture moves:
            // if square in front is empty
            if position.pieces[square - 10] == Empty as u8 {
                add_black_pawn_move(position, square, square - 10, move_list);

                // if 2 squares in front is empty
                if RANKS_BOARD[square] == Rank7 as u8 && position.pieces[square - 20] == Empty as u8 {
                    add_quiet_move(position, move_builder(
                        square as u64,
                        (square - 20) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_PAWN_START),
                        move_list
                    );
                }
            }

            // Capture moves:
            // if top left
            if FILES_BOARD[square - 9] != OffBoard as u8 && PIECE_COLOR[position.pieces[square - 9] as usize] == WHITE as u8 {
                add_black_pawn_capture_move(position, square, square - 9, position.pieces[square - 9] as usize, move_list);
            }
            // if top right
            if FILES_BOARD[square - 11] != OffBoard as u8 && PIECE_COLOR[position.pieces[square - 11] as usize] == WHITE as u8 {
                add_black_pawn_capture_move(position, square, square - 11, position.pieces[square - 11] as usize, move_list);
            }

            if position.en_passent != NoSq as u8 {
                if square - 9 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square - 9) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
                if square - 11 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square - 11) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
            }
        }

        // Castling:
        // if castle_permission bitwise & the bit
        if position.castle_permission as u8 & BlackKingCastle as u8 != 0 {
            // squares between are empty
            if position.pieces[F8 as usize] == Empty as u8 &&
            position.pieces[G8 as usize] == Empty as u8 {
                // if the king is not being attacked
                if !square_attacked(E8 as usize, WHITE, position) &&
                !square_attacked(F8 as usize, WHITE, position) {
                    // println!("Black king castle");
                    add_quiet_move(position, move_builder(
                        E8 as u64,
                        G8 as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_CASTLE),
                        move_list
                    );
                }
            }
        }
        if position.castle_permission as u8 & BlackQueenCastle as u8 != 0 {
            if position.pieces[D8 as usize] == Empty as u8 &&
            position.pieces[C8 as usize] == Empty as u8 &&
            position.pieces[B8 as usize] == Empty as u8 {
                // if the king is not being attacked
                if !square_attacked(E8 as usize, WHITE, position) &&
                !square_attacked(D8 as usize, WHITE, position) {
                    // println!("Black queen castle");
                    add_quiet_move(position, move_builder(
                        E8 as u64,
                        C8 as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_CASTLE),
                        move_list
                    );
                }
            }
        }
    }

    // indexes that pieces move (like pawn captures)
    const PIECE_DIRECTION: [[i8; 8]; 13] = [
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [-8, -19, -21, -12, 8, 19, 21, 12],
        [-9, -11, 11, 9, 0, 0, 0, 0],
        [-1, -10, 1, 10, 0, 0, 0, 0],
        [-1, -10, 1, 10, -9, -11, 11, 9],
        [-1, -10, 1, 10, -9, -11, 11, 9],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [-8, -19, -21, -12, 8, 19, 21, 12],
        [-9, -11, 11, 9, 0, 0, 0, 0],
        [-1, -10, 1, 10, 0, 0, 0, 0],
        [-1, -10, 1, 10, -9, -11, 11, 9],
        [-1, -10, 1, 10, -9, -11, 11, 9]
    ];
    // number of directions each piece can move (rook 4, queen 8)
    const NUMBER_DIRECTION: [u8; 13] = [0, 0, 8, 4, 4, 8, 8, 0, 8, 4, 4, 8, 8];

    // Loop for slide pieces:
    const LOOP_SLIDE_PIECES: [u8; 8] = [WhiteBishop as u8, WhiteRook as u8, WhiteQueen as u8, 0, BlackBishop as u8, BlackRook as u8, BlackQueen as u8, 0];
    const LOOP_SLIDE_INDEX: [usize; 2] = [0, 4];

    let mut piece_index = LOOP_SLIDE_INDEX[side as usize]; // to skip the 0 (non-slide piece)
    let mut piece = LOOP_SLIDE_PIECES[piece_index];
    piece_index += 1;

    while piece != 0 {
        if DEBUG && !piece_valid(piece as usize) { eprintln!("{}", "generate_all_moves: [piece] piece not valid (loop for slide pieces)".red()); }
        // println!("sliders piece_index: {}   piece: {}", piece_index, piece);

        // loop through every (slide) piece
        for piece_number in 0..position.piece_number[piece as usize] {
            let square = position.piece_list[piece as usize][piece_number as usize];

            if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "generate_all_moves: [square] square not on board (loop for slide pieces)".red()); }
            // println!("piece: {} on {}", PIECE_CHAR.chars().nth(piece as usize).unwrap_or(' '), print_square(square));

            for i in 0..NUMBER_DIRECTION[piece as usize] {
                // get target_square via piece_direction array
                let direction = PIECE_DIRECTION[piece as usize][i as usize] as i16;
                let mut target_square = square as i16 + direction as i16;

                // while target_square is on board
                while square_on_board(target_square as usize) {
                    if position.pieces[target_square as usize] != Empty as u8 {
                        // BLACK ^ 1 == WHITE   WHITE ^ 1 == BLACK
                        // if target square equals opposite color of our color
                        if PIECE_COLOR[position.pieces[target_square as usize] as usize] == side ^ 1 {
                            // println!("  Capture on {}", print_square(target_square as u8));
                            add_capture_move(position, move_builder(
                                square as u64,
                                target_square as u64,
                                position.pieces[target_square as usize] as u64,
                                Empty as u64,
                                0
                            ),
                            move_list);
                        }
                        break;
                    }

                    // println!("  Normal move on {}", print_square(target_square as u8));
                    add_quiet_move(position, move_builder(
                        square as u64,
                        target_square as u64,
                        Empty as u64,
                        Empty as u64,
                        0
                    ),
                    move_list);

                    // add direction again to continue in that direction
                    target_square += direction;
                }
            }
        }

        piece = LOOP_SLIDE_PIECES[piece_index];
        piece_index += 1;
    }

    // Loop for non-slide:
    const LOOP_NON_SLIDE_PIECES: [u8; 6] = [WhiteKnight as u8, WhiteKing as u8, 0, BlackKnight as u8, BlackKing as u8, 0];
    const LOOP_NON_SLIDE_INDEX: [usize; 2] = [0, 3];

    let mut piece_index = LOOP_NON_SLIDE_INDEX[side as usize];
    let mut piece = LOOP_NON_SLIDE_PIECES[piece_index];
    piece_index += 1;

    while piece != 0 {
        if DEBUG && !piece_valid(piece as usize) { eprintln!("{}", "generate_all_moves: [piece] piece not valid (loop for non-slide pieces)".red()); }
        // println!("non-sliders piece_index: {}   piece: {}", piece_index, piece);

        // loop through every (non-slide) piece
        for piece_number in 0..position.piece_number[piece as usize] {
            let square = position.piece_list[piece as usize][piece_number as usize];

            if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "generate_all_moves: [square] square not on board (loop for non-slide pieces)".red()); }
            // println!("piece: {} on {}", PIECE_CHAR.chars().nth(piece as usize).unwrap_or(' '), print_square(square));

            for i in 0..NUMBER_DIRECTION[piece as usize] {
            // get target_square via piece_direction array
                let direction: i8 = PIECE_DIRECTION[piece as usize][i as usize];
                let target_square = (square as i16 + direction as i16) as usize;

                if !square_on_board(target_square as usize) { continue; }

                if position.pieces[target_square] != Empty as u8 {
                    // BLACK ^ 1 == WHITE   WHITE ^ 1 == BLACK
                    // if target square equals opposite color of our color
                    if PIECE_COLOR[position.pieces[target_square] as usize] == side ^ 1 {
                        // println!("  Capture on {}", print_square(target_square as u8));
                        add_capture_move(position, move_builder(
                            square as u64,
                            target_square as u64,
                            position.pieces[target_square as usize] as u64,
                            Empty as u64,
                            0
                        ),
                        move_list);
                    }
                    continue;
                }

                // println!("  Normal move on {}", print_square(target_square as u8));
                add_quiet_move(position, move_builder(
                    square as u64,
                    target_square as u64,
                    Empty as u64,
                    Empty as u64,
                    0
                ),
                move_list);
            }
        }

        piece = LOOP_NON_SLIDE_PIECES[piece_index];
        piece_index += 1;
    }
}

pub fn generate_all_capture_moves(position: &mut Board, move_list: &mut MoveList) {
    if DEBUG { check_board(position); }

    move_list.count = 0;
    let side = position.side;

    // pawn movement
    if side == WHITE as u8 {
        // loop through each WhitePawn
        for piece_number in 0..position.piece_number[WhitePawn as usize] {
            let square: usize = position.piece_list[WhitePawn as usize][piece_number as usize] as usize;
            if DEBUG && !square_on_board(square) { eprintln!("{}", "generate_all_moves: [square] square not on board (WhitePawn movement)".red()); }

            // Capture moves:
            // if top left
            if FILES_BOARD[square + 9] != OffBoard as u8 && PIECE_COLOR[position.pieces[square + 9] as usize] == BLACK as u8 {
                add_white_pawn_capture_move(position, square, square + 9, position.pieces[square + 9] as usize, move_list);
            }
            // if top right
            if FILES_BOARD[square + 11] != OffBoard as u8 && PIECE_COLOR[position.pieces[square + 11] as usize] == BLACK as u8 {
                add_white_pawn_capture_move(position, square, square + 11, position.pieces[square + 11] as usize, move_list);
            }

            // if en_passent exists
            if position.en_passent != NoSq as u8 {
                if square + 9 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square + 9) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
                if square + 11 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square + 11) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
            }
        }
    } else {
        // loop through each BlackPawn
        for piece_number in 0..position.piece_number[BlackPawn as usize] {
            let square: usize = position.piece_list[BlackPawn as usize][piece_number as usize] as usize;
            if DEBUG && !square_on_board(square) { eprintln!("{}", "generate_all_moves: [square] square not on board (BlackPawn movement)".red()); }

            // Capture moves:
            // if top left
            if FILES_BOARD[square - 9] != OffBoard as u8 && PIECE_COLOR[position.pieces[square - 9] as usize] == WHITE as u8 {
                add_black_pawn_capture_move(position, square, square - 9, position.pieces[square - 9] as usize, move_list);
            }
            // if top right
            if FILES_BOARD[square - 11] != OffBoard as u8 && PIECE_COLOR[position.pieces[square - 11] as usize] == WHITE as u8 {
                add_black_pawn_capture_move(position, square, square - 11, position.pieces[square - 11] as usize, move_list);
            }

            if position.en_passent != NoSq as u8 {
                if square - 9 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square - 9) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
                if square - 11 == position.en_passent as usize {
                    add_en_passent_move(position, move_builder(
                        square as u64, (square - 11) as u64,
                        Empty as u64,
                        Empty as u64,
                        MOVE_FLAG_EN_PASSENT),
                        move_list
                    );
                }
            }
        }
    }

    // indexes that pieces move (like pawn captures)
    const PIECE_DIRECTION: [[i8; 8]; 13] = [
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [-8, -19, -21, -12, 8, 19, 21, 12],
        [-9, -11, 11, 9, 0, 0, 0, 0],
        [-1, -10, 1, 10, 0, 0, 0, 0],
        [-1, -10, 1, 10, -9, -11, 11, 9],
        [-1, -10, 1, 10, -9, -11, 11, 9],
        [0, 0, 0, 0, 0, 0, 0, 0],
        [-8, -19, -21, -12, 8, 19, 21, 12],
        [-9, -11, 11, 9, 0, 0, 0, 0],
        [-1, -10, 1, 10, 0, 0, 0, 0],
        [-1, -10, 1, 10, -9, -11, 11, 9],
        [-1, -10, 1, 10, -9, -11, 11, 9]
    ];
    // number of directions each piece can move (rook 4, queen 8)
    const NUMBER_DIRECTION: [u8; 13] = [0, 0, 8, 4, 4, 8, 8, 0, 8, 4, 4, 8, 8];

    // Loop for slide pieces:
    const LOOP_SLIDE_PIECES: [u8; 8] = [WhiteBishop as u8, WhiteRook as u8, WhiteQueen as u8, 0, BlackBishop as u8, BlackRook as u8, BlackQueen as u8, 0];
    const LOOP_SLIDE_INDEX: [usize; 2] = [0, 4];

    let mut piece_index = LOOP_SLIDE_INDEX[side as usize]; // to skip the 0 (non-slide piece)
    let mut piece = LOOP_SLIDE_PIECES[piece_index];
    piece_index += 1;

    while piece != 0 {
        if DEBUG && !piece_valid(piece as usize) { eprintln!("{}", "generate_all_moves: [piece] piece not valid (loop for slide pieces)".red()); }
        // println!("sliders piece_index: {}   piece: {}", piece_index, piece);

        // loop through every (slide) piece
        for piece_number in 0..position.piece_number[piece as usize] {
            let square = position.piece_list[piece as usize][piece_number as usize];

            if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "generate_all_moves: [square] square not on board (loop for slide pieces)".red()); }
            // println!("piece: {} on {}", PIECE_CHAR.chars().nth(piece as usize).unwrap_or(' '), print_square(square));

            for i in 0..NUMBER_DIRECTION[piece as usize] {
                // get target_square via piece_direction array
                let direction = PIECE_DIRECTION[piece as usize][i as usize] as i16;
                let mut target_square = square as i16 + direction as i16;

                // while target_square is on board
                while square_on_board(target_square as usize) {
                    if position.pieces[target_square as usize] != Empty as u8 {
                        // BLACK ^ 1 == WHITE   WHITE ^ 1 == BLACK
                        // if target square equals opposite color of our color
                        if PIECE_COLOR[position.pieces[target_square as usize] as usize] == side ^ 1 {
                            // println!("  Capture on {}", print_square(target_square as u8));
                            add_capture_move(position, move_builder(
                                square as u64,
                                target_square as u64,
                                position.pieces[target_square as usize] as u64,
                                Empty as u64,
                                0
                            ),
                            move_list);
                        }
                        break;
                    }
                    // add direction again to continue in that direction
                    target_square += direction;
                }
            }
        }

        piece = LOOP_SLIDE_PIECES[piece_index];
        piece_index += 1;
    }

    // Loop for non-slide:
    const LOOP_NON_SLIDE_PIECES: [u8; 6] = [WhiteKnight as u8, WhiteKing as u8, 0, BlackKnight as u8, BlackKing as u8, 0];
    const LOOP_NON_SLIDE_INDEX: [usize; 2] = [0, 3];

    let mut piece_index = LOOP_NON_SLIDE_INDEX[side as usize];
    let mut piece = LOOP_NON_SLIDE_PIECES[piece_index];
    piece_index += 1;

    while piece != 0 {
        if DEBUG && !piece_valid(piece as usize) { eprintln!("{}", "generate_all_moves: [piece] piece not valid (loop for non-slide pieces)".red()); }
        // println!("non-sliders piece_index: {}   piece: {}", piece_index, piece);

        // loop through every (non-slide) piece
        for piece_number in 0..position.piece_number[piece as usize] {
            let square = position.piece_list[piece as usize][piece_number as usize];

            if DEBUG && !square_on_board(square as usize) { eprintln!("{}", "generate_all_moves: [square] square not on board (loop for non-slide pieces)".red()); }
            // println!("piece: {} on {}", PIECE_CHAR.chars().nth(piece as usize).unwrap_or(' '), print_square(square));

            for i in 0..NUMBER_DIRECTION[piece as usize] {
            // get target_square via piece_direction array
                let direction: i8 = PIECE_DIRECTION[piece as usize][i as usize];
                let target_square = (square as i16 + direction as i16) as usize;

                if !square_on_board(target_square as usize) { continue; }

                if position.pieces[target_square] != Empty as u8 {
                    // BLACK ^ 1 == WHITE   WHITE ^ 1 == BLACK
                    // if target square equals opposite color of our color
                    if PIECE_COLOR[position.pieces[target_square] as usize] == side ^ 1 {
                        // println!("  Capture on {}", print_square(target_square as u8));
                        add_capture_move(position, move_builder(
                            square as u64,
                            target_square as u64,
                            position.pieces[target_square as usize] as u64,
                            Empty as u64,
                            0
                        ),
                        move_list);
                    }
                    continue;
                }
            }
        }

        piece = LOOP_NON_SLIDE_PIECES[piece_index];
        piece_index += 1;
    }
}
