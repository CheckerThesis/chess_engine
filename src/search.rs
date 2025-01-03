use std::time::{Duration, Instant};

use colored::Colorize;

use crate::{attack::square_attacked, board::check_board, defs::{from_square, to_square, Board, MoveList, SearchInfo, BOARD_SQUARE_NUMBER, DEBUG, INFINITE, IS_MATE, MAX_DEPTH, MAX_GAME_MOVES, MOVE_FLAG_CAPTURE, NO_MOVE}, evaluate::evaluate_position, io::print_move, makemove::{make_move, take_move}, movegen::{generate_all_capture_moves, generate_all_moves}, pvtable::get_pv_line};

// check if time up, or interrupt from GUI
pub fn check_up(info: &mut SearchInfo) { if info.time_set && info.time.elapsed().as_secs() > info.stop_time  { info.stopped = true; } }

// from move_number through the remaining moves, find the best score and put it in front
pub fn pick_next_move(move_number: usize, move_list: &mut MoveList) {
    let mut best_score = 0;
    let mut best_number = move_number;
    for i in move_number as usize..move_list.count {
        // if move is greater than current best_score, save the index
        if move_list.moves[i].score > best_score {
            best_score = move_list.moves[i].score;
            best_number = i;
        }
    }

    let temp = move_list.moves[move_number];
    move_list.moves[move_number] = move_list.moves[best_number];
    move_list.moves[best_number] = temp;
}

//position.history_ply - position.fifty_move as usize
pub fn is_repetition(position: &Board) -> bool {
    if position.history_ply <= 1 { return false }

    let start = position.history_ply.saturating_sub(position.fifty_move as usize);

    // println!(
    //     "history_ply: {}, fifty_move: {}, start: {}",
    //     position.history_ply,
    //     position.fifty_move,
    //     position.history_ply.saturating_sub(position.fifty_move as usize)
    // );

    for i in start..position.history_ply - 1 {
        if DEBUG && i > MAX_GAME_MOVES { eprintln!("{}", "is_repetition: [i] is greater than MAX_GAME_MOVES ".red()) }

        if position.position_key == position.history[i].position_key { return true; }
    }

    false
}

pub fn clear_for_search(position: &mut Board, info: &mut SearchInfo) {
    for i in 0..13 {
        for j in 0..BOARD_SQUARE_NUMBER {
            position.search_history[i][j] = 0;
        }
    }

    for i in 0..2 {
        for j in 0..MAX_DEPTH {
            position.search_killers[i][j] = 0;
        }
    }

    position.pv_table.clear();

    position.ply = 0; // half moves for current search

    info.time = Instant::now();
    info.nodes = 0;
    info.fail_high = 0.0;
    info.fail_high_first = 0.0;
}

// resolves all captures so if a queen takes a defended pawn, the engine doesn't think it is +1 pawn, but realizes
// opponent can take the queen as well (horizon effect)
pub fn quiescence(alpha: i32, beta: i32, position: &mut Board, info: &mut SearchInfo) -> i32 {
    if DEBUG { check_board(position); }

    if info.nodes & 2047 == 0 { check_up(info); }

    info.nodes += 1;
    if is_repetition(position) || position.fifty_move >= 100 { return 0 } // position is draw
    if position.ply > MAX_DEPTH as u8 - 1 { return evaluate_position(position) } // too deep

    let move_list = &mut MoveList::default();
    generate_all_capture_moves(position, move_list);

    let mut legal = 0;
    let mut score = evaluate_position(position);
    let mut internal_alpha = alpha;
    let mut best_move = NO_MOVE;

    if score >= beta { return beta }

    if score > internal_alpha { internal_alpha = score; }

    // loop through moves
    for move_number in 0..move_list.count {
        pick_next_move(move_number, move_list);

        // if not legal move
        if !make_move(position, move_list.moves[move_number].el_move) { continue; }

        legal += 1;
        score = -quiescence(-beta, -internal_alpha, position, info); // negamax
        take_move(position);

        if score > internal_alpha {
            // beta cutoff
            if score >= beta {
                // if searched the best move first
                if legal == 1 {
                    info.fail_high_first += 1.0;
                }
                info.fail_high += 1.0;

                return beta
            }
            internal_alpha = score;
            best_move = move_list.moves[move_number].el_move;
        }
    }

    if internal_alpha != alpha { position.store_pv_move(best_move); }

    internal_alpha
}

// TODO more comments here
pub fn alpha_beta(alpha: i32, beta: i32, depth: u8, position: &mut Board, info: &mut SearchInfo, _do_null: bool) -> i32 {
    if DEBUG { check_board(position); }

    if depth <= 0 { return quiescence(alpha, beta, position, info) }

    if info.nodes & 2047 == 0 { check_up(info); }

    info.nodes += 1;

    if is_repetition(position) || position.fifty_move >= 100 { return 0 } // position is draw
    if position.ply > MAX_DEPTH as u8 - 1 { return evaluate_position(position) } // too deep

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    let mut legal = 0;
    let mut internal_alpha = alpha;
    let mut best_move = NO_MOVE;
    let mut score: i32;

    if let Some(pv_move) = position.probe_pv_table(position.position_key) {
        for move_number in 0..move_list.count {
            if move_list.moves[move_number].el_move == pv_move {
                move_list.moves[move_number].score = INFINITE as u32;
                break;
            }
        }
    }

    // loop through moves
    for move_number in 0..move_list.count {
        pick_next_move(move_number, move_list);

        // if not legal move
        if !make_move(position, move_list.moves[move_number].el_move) { continue; }

        legal += 1;
        score = -alpha_beta(-beta, -internal_alpha, depth - 1, position, info, true); // negamax
        take_move(position);

        if info.stopped { return 0 }

        if score > internal_alpha {
            // beta cutoff
            if score >= beta {
                // if searched the best move first
                if legal == 1 {
                    info.fail_high_first += 1.0;
                }
                info.fail_high += 1.0;

                // if not a capture
                if move_list.moves[move_number].el_move & MOVE_FLAG_CAPTURE == 0 {
                    // put the new best killer in index 0 and shuffle the old one into index 1
                    position.search_killers[1][position.ply as usize] = position.search_killers[0][position.ply as usize];
                    position.search_killers[0][position.ply as usize] = move_list.moves[move_number].el_move;
                }

                return beta
            }
            internal_alpha = score;
            best_move = move_list.moves[move_number].el_move;

            if move_list.moves[move_number].el_move & MOVE_FLAG_CAPTURE == 0 {
                position.search_history[position.pieces[from_square(best_move) as usize] as usize][to_square(best_move) as usize] += depth as u32;
            }
        }
    }

    // if we've made 0 legal moves
    if legal == 0 {
        // king_sq attacked by opposite side, and we have no legal moves, we've been checkmated
        if square_attacked(position.king_square[position.side as usize] as usize, position.side as usize ^ 1, position) {
            return -IS_MATE + position.ply as i32;
        } else {
            return 0
        }
    }

    // if this, then we've improved alpha and found the best move so we store it in the pv_array
    if internal_alpha != alpha { position.store_pv_move(best_move); }

    internal_alpha
}

// iterative deepening is more efficient then just going to certain depth because pv_table has the best line
// already so alpha beta will have to go through less nodes
pub fn search_position(position: &mut Board, info: &mut SearchInfo) {
    clear_for_search(position, info);
    let mut best_move = 0;

    // iterative deepening search best move for each depth
    for current_depth in 0..info.depth {
        let best_score = alpha_beta(-INFINITE, INFINITE, current_depth + 1, position, info, true);

        if info.stopped { break; }

        let pv_moves = get_pv_line(current_depth, position);
        best_move = position.pv_array[0];

        print!("info score cp {} depth {} nodes {} time {}",
            best_score, current_depth + 1, info.nodes, info.time.elapsed().as_secs()
        );

        print!(" pv");
        for pv_number in 0..pv_moves { print!(" {}", print_move(position.pv_array[pv_number])); }
        println!();

        println!("Ordering: {}", info.fail_high_first/info.fail_high);
    }

    println!("bestmove {}", print_move(best_move));
}
