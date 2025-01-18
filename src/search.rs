use std::time::Instant;

use colored::Colorize;

use crate::{attack::square_attacked, board::check_board, defs::{from_square, to_square, Board, HashFlag::*, HashTable, MoveList, SearchInfo, BOARD_SQUARE_NUMBER, DEBUG, ENGINE_OPTIONS, INF_BOUND, IS_MATE, MAX_DEPTH, MAX_GAME_MOVES, MOVE_FLAG_CAPTURE, NO_MOVE}, evaluate::evaluate_position, io::print_move, makemove::{make_move, make_null_move, take_move, take_null_move}, movegen::{generate_all_capture_moves, generate_all_moves}, polybook::{self, get_book_move, POLY_BOOK}, pvtable::{get_pv_line, probe_hash_table, store_hash_entry}};

// check if time up, or interrupt from GUI
pub fn check_up(info: &mut SearchInfo) { if info.time_set && (Instant::now() >= info.stop_time) { info.stopped = true; } }

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

pub fn clear_for_search(position: &mut Board, info: &mut SearchInfo, hash_table: &mut HashTable) {
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

    hash_table.over_write = 0;
    hash_table.hit = 0;
    hash_table.cut = 0;
    position.ply = 0; // half moves for current search
    hash_table.current_age += 1;

    info.stopped = false;
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

    let mut score = evaluate_position(position);
    let mut internal_alpha = alpha;

    if score >= beta { return beta }

    if score > internal_alpha { internal_alpha = score; }

    let mut legal = 0;

    let move_list = &mut MoveList::default();
    generate_all_capture_moves(position, move_list);

    // loop through moves
    for move_number in 0..move_list.count {
        pick_next_move(move_number, move_list);

        // if not legal move
        if !make_move(position, move_list.moves[move_number].el_move) { continue; }

        legal += 1;
        score = -quiescence(-beta, -internal_alpha, position, info); // negamax
        take_move(position);

        if info.stopped == true { return 0 }

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
        }
    }

    if DEBUG && internal_alpha < alpha { eprintln!("{}", "quiescence: [alpha] is greater than internal_alpha".red()); }

    internal_alpha
}

pub fn alpha_beta(alpha: &mut i32, beta: &mut i32, mut depth: i32, position: &mut Board, info: &mut SearchInfo, hash_table: &mut HashTable, do_null: bool) -> i32 {
    if DEBUG { check_board(position); }

    if depth <= 0 { return quiescence(*alpha, *beta, position, info) }

    if info.nodes & 2047 == 0 { check_up(info); }

    info.nodes += 1;

    if (is_repetition(position) || position.fifty_move >= 100) && position.ply == 1 { return 0 } // position is draw
    if position.ply > MAX_DEPTH as u8 - 1 { return evaluate_position(position) }

    let in_check = square_attacked(position.king_square[position.side as usize] as usize, (position.side ^ 1) as usize, position);

    if in_check { depth += 1; } // because if one check, likely a sequence of checks into mate, with this

    let mut score: i32 = -INF_BOUND;
    let mut pv_move: u64 = NO_MOVE;
    let mut legal = 0;
    let mut internal_alpha = *alpha;
    let mut best_move = NO_MOVE;
    let mut best_score: i32 = -INF_BOUND;

    // transposition table
    if probe_hash_table(position, hash_table,&mut pv_move, &mut score, *alpha, *beta, depth) {
        hash_table.cut += 1;
        return score
    }

    // null move pruning, give an opponent free move, if still beta cutoff, just return beta (move would be the move at depth 4 in this case)
    if do_null && !in_check && position.ply > 0 && position.big_piece[position.side as usize] > 1 && depth > 4 {
        make_null_move(position);
        score = -alpha_beta(&mut (-*beta), &mut (-*beta + 1), depth - 4, position, info, hash_table,false);
        take_null_move(position);
        if info.stopped { return 0 }
        if score >= *beta && score.abs() < IS_MATE {
            info.null_cut += 1;
            return *beta
        }
    }

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    if pv_move != NO_MOVE {
        for move_number in 0..move_list.count {
            if move_list.moves[move_number].el_move == pv_move {
                move_list.moves[move_number].score = 2000000;
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
        score = -alpha_beta(&mut -*beta, &mut -internal_alpha, depth - 1, position, info, hash_table,false); // negamax
        take_move(position);

        if info.stopped == true { return 0 }

        if score > best_score {
            best_score = score;
            best_move = move_list.moves[move_number].el_move;

            if score > internal_alpha {
                // beta cutoff
                if score >= *beta {
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

                    store_hash_entry(position, hash_table,best_move, &mut *beta, HashFlagBeta as u8, depth);

                    return *beta
                }
                internal_alpha = score;

                // history heuristic
                if move_list.moves[move_number].el_move & MOVE_FLAG_CAPTURE == 0 {
                    position.search_history[position.pieces[from_square(best_move) as usize] as usize][to_square(best_move) as usize] += depth as u32;
                }
            }
        }
    }

    // if we've made 0 legal moves
    if legal == 0 {
        // king_sq attacked by opposite side, and we have no legal moves, we've been checkmated
        if in_check {
            return -IS_MATE + position.ply as i32;
        } else {
            return 0
        }
    }

    // if this, then we've improved alpha and found the best move so we store it in the pv_array
    if internal_alpha != *alpha {
        store_hash_entry(position, hash_table,best_move, &mut best_score, HashFlagExact as u8, depth);
    } else {
        store_hash_entry(position, hash_table,best_move, &mut internal_alpha, HashFlagAlpha as u8, depth);
    }

    internal_alpha
}

pub fn search_position(position: &mut Board, info: &mut SearchInfo, hash_table: &mut HashTable) {
    let mut best_move = NO_MOVE;
    let mut best_score: i32;

    clear_for_search(position, info, hash_table);
    // let init: &Vec<polybook::PolyBookEntry> = &*POLY_BOOK;

    if ENGINE_OPTIONS.lock().unwrap().book { best_move = get_book_move(position); }

    if best_move == NO_MOVE {
        // iterative deepening search best move for each depth
        for current_depth in 0..info.depth {                // infinite
            best_score = alpha_beta(&mut -INF_BOUND, &mut 30000, current_depth + 1, position, info, hash_table, true);

            if info.stopped { break; }

            let pv_moves: usize = get_pv_line(current_depth as u8 + 1, position, hash_table);
            best_move = position.pv_array[0];

            print!("info score cp {} depth {} nodes {} time {}",
                best_score, current_depth + 1, info.nodes, info.start_time.elapsed().as_millis()
            );

            print!(" pv");
            for pv_number in 0..pv_moves { print!(" {}", print_move(position.pv_array[pv_number])); }
            println!();

            // println!("Ordering: {}", info.fail_high_first/info.fail_high);
        }
    }

    println!("bestmove {}", print_move(best_move));
}
