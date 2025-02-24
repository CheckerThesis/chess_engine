use std::{sync::{atomic::{AtomicU32, Ordering}, Arc}, thread::{self, JoinHandle}};

use colored::Colorize;

use crate::{attack::square_attacked, board::check_board, defs::{extract_movelist_move, extract_movelist_score, from_square, store_movelist_score, to_square, Board, HashFlag::*, HashTable, MoveList, SearchInfo, SearchWorkerData, AB_BOUND, BOARD_SQUARE_NUMBER, DEBUG, ENGINE_OPTIONS, IS_MATE, MAX_DEPTH, MAX_GAME_MOVES, MOVE_FLAG_CAPTURE, NO_MOVE}, evaluate::evaluate_position, io::print_move, makemove::{make_move, make_null_move, take_move, take_null_move}, movegen::{generate_all_capture_moves, generate_all_moves}, polybook::get_book_move, pvtable::get_pv_line};

// check if time up, or interrupt from GUI



// from move_number through the remaining moves, find the best score and put it in front
pub fn pick_next_move(move_number: usize, move_list: &mut MoveList) {
    let mut best_score = 0;
    let mut best_number = move_number;
    for i in move_number as usize..move_list.count {
        let score = extract_movelist_score(move_list.moves[i]);
        // if move is greater than current best_score, save the index
        if score > best_score {
            best_score = score;
            best_number = i;
        }
    }

    move_list.moves.swap(move_number, best_number);
}

pub fn is_repetition(position: &Board) -> bool {
    if position.history_ply <= 1 { return false }

    let start = position.history_ply.saturating_sub(position.fifty_move as usize);

    for i in start..position.history_ply - 1 {
        if DEBUG && i > MAX_GAME_MOVES { eprintln!("{}", "is_repetition: [i] is greater than MAX_GAME_MOVES ".red()) }

        if position.position_key == position.history[i].position_key { return true; }
    }

    false
}

pub fn clear_for_search(position: &mut Board, info: &SearchInfo, hash_table: &HashTable) {
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

    hash_table.set_over_write(0);
    hash_table.set_hit(0);
    hash_table.set_cut(0);
    position.ply = 0; // half moves for current search
    hash_table.current_age.fetch_add(1, Ordering::Relaxed);

    info.stopped.store(false, Ordering::Relaxed);
    info.nodes.store(0, Ordering::Relaxed);
    if let Ok(mut protected) = info.protected.write() {
        protected.fail_high = 0.0;
        protected.fail_high_first = 0.0;
    }
}

// resolves all captures so if a queen takes a defended pawn, the engine doesn't think it is +1 pawn, but realizes
// opponent can take the queen as well (horizon effect)
pub fn quiescence(alpha: i32, beta: i32, position: &mut Board, info: &SearchInfo) -> i32 {
    if DEBUG { check_board(position); }

    if info.nodes.load(Ordering::Relaxed) & 2047 == 0 { info.check_up(); }

    info.nodes.fetch_add(1, Ordering::Relaxed);
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
        if !make_move(position, extract_movelist_move(move_list.moves[move_number])) { continue; }

        legal += 1;
        score = -quiescence(-beta, -internal_alpha, position, info); // negamax
        take_move(position);

        if info.is_stopped() == true { return 0 }

        if score > internal_alpha {
            // beta cutoff
            if score >= beta {
                if let Ok(mut protected) = info.protected.write() {
                    // if searched the best move first
                    if legal == 1 {
                        protected.fail_high_first += 1.0;
                    }
                    protected.fail_high += 1.0;
                }

                return beta
            }
            internal_alpha = score;
        }
    }

    if DEBUG && internal_alpha < alpha { eprintln!("{}", "quiescence: [alpha] is greater than internal_alpha".red()); }

    internal_alpha
}

pub fn alpha_beta(alpha: &mut i32, beta: &mut i32, mut depth: i32, position: &mut Board, info: &SearchInfo, hash_table: &HashTable, do_null: bool) -> i32 {
    if DEBUG { check_board(position); }

    if depth <= 0 { return quiescence(*alpha, *beta, position, info); }

    if info.nodes.load(Ordering::Relaxed) & 2047 == 0 { info.check_up(); }
    info.nodes.fetch_add(1, Ordering::Relaxed);

    if (is_repetition(position) || position.fifty_move >= 100) && position.ply == 1 { return 0; } // draw
    if position.ply > MAX_DEPTH as u8 - 1 { return evaluate_position(position); }

    let in_check = square_attacked(position.king_square[position.side as usize] as usize, (position.side ^ 1) as usize, position);
    if in_check { depth += 1; }

    let mut score: i32 = -AB_BOUND;
    let mut pv_move = NO_MOVE;

    if hash_table.probe_hash_table(position, &mut pv_move, &mut score, *alpha, *beta, depth) {
        hash_table.cut.fetch_add(1, Ordering::Relaxed);
        return score;
    }

    if do_null && !in_check && position.ply > 0 && position.big_piece[position.side as usize] > 0 && depth >= 4 {
        make_null_move(position);
        score = -alpha_beta(&mut (-*beta), &mut (-*beta + 1), depth - 4, position, info, hash_table, false);
        take_null_move(position);
        if info.is_stopped() { return 0; }
        if score >= *beta && score.abs() < IS_MATE {
            info.null_cut.fetch_add(1, Ordering::Relaxed);
            return *beta;
        }
    }

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    // If there is a PV move, boost its score.
    if pv_move != NO_MOVE {
        for move_number in 0..move_list.count {
            if extract_movelist_move(move_list.moves[move_number]) == pv_move {
                store_movelist_score(&mut move_list.moves[move_number], 2000000);
                break;
            }
        }
    }

    let mut legal = 0;
    let mut internal_alpha = *alpha;
    let mut best_move = NO_MOVE;
    let mut best_score: i32 = -AB_BOUND;

    for move_number in 0..move_list.count {
        pick_next_move(move_number, move_list);

        if !make_move(position, extract_movelist_move(move_list.moves[move_number])) { continue; } // if not legal
        legal += 1;
        score = -alpha_beta(&mut -*beta, &mut -internal_alpha, depth - 1, position, info, hash_table, true);
        take_move(position);
        if info.is_stopped() { return 0; }

        if score > best_score {
            best_score = score;
            best_move = extract_movelist_move(move_list.moves[move_number]);

            if score > internal_alpha {
                // beta cutoff
                if score >= *beta {
                    if let Ok(mut protected) = info.protected.write() {
                        // if searched the best move first
                        if legal == 1 { protected.fail_high_first += 1.0; }
                        protected.fail_high += 1.0;
                    }
                    // if not a capture
                    if extract_movelist_move(move_list.moves[move_number]) & MOVE_FLAG_CAPTURE == 0 {
                        // put the new best killer in index 0 and shuffle the old one into index 1
                        position.search_killers[1][position.ply as usize] = position.search_killers[0][position.ply as usize];
                        position.search_killers[0][position.ply as usize] = extract_movelist_move(move_list.moves[move_number]);
                    }
                    hash_table.store_hash_entry(position, best_move, beta, HashFlagBeta as u8, depth);
                    return *beta;
                }
                internal_alpha = score;
                // history heuristic
                if extract_movelist_move(move_list.moves[move_number]) & MOVE_FLAG_CAPTURE == 0 { position.search_history[position.pieces[from_square(best_move) as usize] as usize][to_square(best_move) as usize] += depth as u32; }
            }
        }
    }

    // if we've made 0 legal moves
    if legal == 0 {
        // king_sq attacked by opposite side, and no legal moves then found checkmate
        return if in_check { -IS_MATE + position.ply as i32 } else { 0 };
    }

    // if this, then we've improved alpha and found the best move so we store it in the pv_array
    if internal_alpha != *alpha {
        hash_table.store_hash_entry(position, best_move, &mut best_score, HashFlagExact as u8, depth);
    } else {
        hash_table.store_hash_entry(position, best_move, &mut internal_alpha, HashFlagAlpha as u8, depth);
    }

    internal_alpha
}

pub fn iterative_deepen(mut thread_data: SearchWorkerData, hash_table: &HashTable) -> u32 {
    let mut best_move = NO_MOVE;
    let mut best_score: i32;

    for current_depth in 1..=thread_data.info.depth.load(Ordering::Relaxed) {
        best_score = alpha_beta(
            &mut -AB_BOUND,
            &mut 30000,
            current_depth,
            &mut thread_data.position,
            &thread_data.info,
            hash_table,
            true
        );

        if thread_data.info.is_stopped() {
            break;
        }

        // Update best move from PV line
        let pv_moves = get_pv_line(current_depth as u8, &mut thread_data.position, hash_table);
        if pv_moves > 0 {
            best_move = thread_data.position.pv_array[0];
        }

        // Only main thread (thread 0) prints info
        if thread_data.thread_number == 0 {
            if let Ok(protected) = thread_data.info.protected.read() {
                print!(
                    "info score cp {} depth {} nodes {} time {} pv",
                    best_score,
                    current_depth,
                    thread_data.info.nodes.load(Ordering::Relaxed),
                    protected.start_time.elapsed().as_millis()
                );

                for pv_number in 0..pv_moves {
                    print!(" {}", print_move(thread_data.position.pv_array[pv_number]));
                }
                println!();
            }
        }
    }

    thread_data.best_move.store(best_move, Ordering::Release);
    best_move
}


pub fn search_position(position: &mut Board, info: Arc<SearchInfo>, hash_table: Arc<HashTable>) {
    let mut best_move = NO_MOVE;
    let mut worker_threads: Vec<JoinHandle<u32>> = Vec::with_capacity(info.get_thread_num() as usize);
    {
        let table = Arc::clone(&hash_table);
        clear_for_search(position, &info, &table);
    }
    if ENGINE_OPTIONS.lock().unwrap().book { best_move = get_book_move(position); }

    if best_move == NO_MOVE {
        for i in 0..info.get_thread_num() {
            let thread_data = SearchWorkerData {
                position: position.clone(),
                info: Arc::clone(&info),
                thread_number: i,
                depth: info.get_depth() as u8,
                best_move: AtomicU32::new(NO_MOVE),
            };

            let table = Arc::clone(&hash_table);
            worker_threads.push(thread::spawn(move || {
                iterative_deepen(thread_data, &table)
            }));
        }

        for handle in worker_threads {
            match handle.join() {
                Ok(move_result) if move_result != NO_MOVE => {
                    best_move = move_result;
                    break;
                }
                _ => continue,
            }
        }
    }

    println!("bestmove {}", print_move(best_move));
}
