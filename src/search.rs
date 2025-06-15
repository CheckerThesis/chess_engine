use std::{sync::{atomic::{AtomicU32}, Arc}, thread::{self, JoinHandle}};

use colored::Colorize;

use crate::{attack::square_attacked, board::{check_board, Board}, defs::{from_square, to_square, SearchInfo, SearchWorkerData, AB_BOUND, BOARD_SQUARE_NUMBER, DEBUG, ENGINE_OPTIONS, IS_MATE, MAX_DEPTH, MAX_GAME_MOVES, MOVE_FLAG_CAPTURE, NO_MOVE}, evaluate::evaluate_position, io::print_move, makemove::{make_move, make_null_move, take_move, take_null_move}, movegen::{extract_movelist_move, extract_movelist_score, generate_all_capture_moves, generate_all_moves, store_movelist_score, MoveList}, polybook::get_book_move, pvtable::{get_pv_line, HashFlag, HashTable}};

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
    hash_table.increment_current_age();

    info.set_stopped(false);
    info.set_nodes(0);
    if let Ok(mut protected) = info.protected.write() {
        protected.fail_high = 0.0;
        protected.fail_high_first = 0.0;
    }
}

/**
Resolves all captures to avoid the horizon effect.

The horizon effect from the CPW "Consider the situation where the last move you consider is queen takes pawn. If you stop there and evaluate, you might think that you have won a pawn. But what if you were to search one move deeper and find that the next move is pawn takes queen?"

# Parameters
- `alpha`: Used for alpha-beta pruning, are originally passed in from leaf-nodes of `alpha-beta`.
- `beta`: Used for alpha-beta pruning, are originally passed in from leaf-nodes of `alpha-beta`.
- `position`: Current position being passed in (changes according to generated moves).
- `info`: Reference to info struct to tell how much time is left for search and keep track of how many nodes traversed.
## Returns
Best score found within capture-only search.

# Logic
1. `evaluate_position` current position, if it's score is already greater or equal to beta, return.
2. If that same score is greater than alpha, set alpha equal to it.
3. `generate_all_capture_moves` and alpha-beta through them.
4. Quiescence ends when any of these conditions are met:
    - `position.ply` > `MAX_DEPTH`.
    - When there are no more capture moves to generate (a quiet position has been reached).
    - Fifty move repetition.
    - Time has run out.
*/
pub fn quiescence(alpha: i32, beta: i32, position: &mut Board, info: &SearchInfo) -> i32 {
    if DEBUG { check_board(position); }

    if info.get_nodes() & 2047 == 0 { info.check_up(); }

    info.increment_nodes();
    if is_repetition(position) || position.fifty_move >= 100 { return 0 } // position is draw
    if position.ply > MAX_DEPTH as u8 - 1 { return evaluate_position(position) } // too deep

    let mut score = evaluate_position(position);
    let mut internal_alpha = alpha;

    if score >= beta { return beta }

    if score > internal_alpha { internal_alpha = score; }

    let mut legal = 0;

    let move_list = &mut MoveList::default();
    generate_all_capture_moves(position, move_list);

    for move_number in 0..move_list.count {
        pick_next_move(move_number, move_list);

        if !make_move(position, extract_movelist_move(move_list.moves[move_number])) { continue; } // if not legal move

        legal += 1;
        score = -quiescence(-beta, -internal_alpha, position, info); // negamax
        take_move(position);

        if info.is_stopped() == true { return 0 }

        if score > internal_alpha {
            // beta cutoff
            if score >= beta {
                if let Ok(mut protected) = info.protected.write() {
                    // if searched the best move first
                    if legal == 1 { protected.fail_high_first += 1.0; }
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
/**
Searches through move tree with alpha-beta negamax algorithm.

Alpha-beta allows large branches of the move tree to be cut off; these cut-offs occur more often when the moves are ordered from best to worst. Evaluations are only done on quiet positions (during quiescence).

## Has features:
- Transposition table (`hash_table`): Stores previous searches.
- Null move pruning (`do_null`): When depth is greater than 3, it will cut off the search early if a beta cutoff is caused even after giving the opponent a free move (by performing a null move).
### Move ordering:
- Killer moves - Two quiet (non-capture) moves that cause beta-offs are stored in `position.search_killers`. First slot is given a score of 900000, second 800000.
- History heuristic - Whenever a score has beaten alpha, add the depth to that move.
- MVV LVA - More during move generation, "Most valuable victim, least valuable attacker", pawn takes queen is more valuable than vice versa.

# Parameters
- `alpha`: Used for alpha-beta pruning, are originally passed in from leaf-nodes of `alpha-beta`.
- `beta`: Used for alpha-beta pruning, are originally passed in from leaf-nodes of `alpha-beta`.
- `depth`: Counter variable so search doesn't go too deep.
- `position`: Current position being passed in (changes according to generated moves).
- `info`: Reference to info struct to tell how much time is left for search and keep track of how many nodes traversed.
- `hash_table`: Global transposition table being passed to all threads for LazySMP.
- `do_null`: Boolean to determine if null move pruning should be used.

## Returns
Best move found.

# Logic
1. Check how much time is left with `info.check_up()`.
2. If `in_check` increase the depth by one because there is likely to be a sequence of checks then.
3. Probe `hash_table` with our current `position.position_key`. Return score based on `HashFlag`.
4. Null move prune.
5. Generate all moves then look through the `move_list` to find the `pv_move`, if found boost its score by 2000000.
6. Pick highest scoring move and call `alpha-beta` if negamax.
7. Alpha-beta
    - If score is higher than alpha (alpha-cutoff)
    - If score is higher than beta (beta-cutoff), then if it's not a capture store in the killer moves, always store in the transposition table with `HashFlagBeta` (when probing..), return beta
    - If not capture move, add the current depth to the killer move (outside of the beta if). This is the history heuristic.
8. If `in_check` and `legal` = 0, then player in check and have made 0 legal moves (checkmate).
9. If alpha has changed store it in the `hash_table` with `HashFlagExact` (when probing..), else store it with `HashFlagAlpha` (when probing..).
*/
pub fn alpha_beta(alpha: &mut i32, beta: &mut i32, mut depth: i32, position: &mut Board, info: &SearchInfo, hash_table: &HashTable, do_null: bool) -> i32 {
    if DEBUG { check_board(position); }

    if depth <= 0 { return quiescence(*alpha, *beta, position, info); }

    if info.get_nodes() & 2047 == 0 { info.check_up(); }
    info.increment_nodes();

    if (is_repetition(position) || position.fifty_move >= 100) && position.ply == 1 { return 0; } // draw
    if position.ply > MAX_DEPTH as u8 - 1 { return evaluate_position(position); }

    let in_check = square_attacked(position.king_square[position.side as usize] as usize, (position.side ^ 1) as usize, position);
    if in_check { depth += 1; }

    let mut score: i32 = -AB_BOUND;
    let mut pv_move = NO_MOVE;

    // Probe hashtable for move and set pv_move if exists
    if hash_table.probe_hash_table(position, &mut pv_move, &mut score, *alpha, *beta, depth) {
        hash_table.increment_cut();
        return score;
    }

    // If null-move, prune
    if do_null && !in_check && position.ply > 0 && position.big_piece[position.side as usize] > 0 && depth >= 4 {
        make_null_move(position);
        score = -alpha_beta(&mut (-*beta), &mut -(*beta - 1), depth - 4, position, info, hash_table, false);
        take_null_move(position);
        if info.is_stopped() { return 0; }
        if score >= *beta && score.abs() < IS_MATE {
            info.increment_null_cut();
            return *beta;
        }
    }

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    // if there is a pv_move, boost its score
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
        score = -alpha_beta(&mut -*beta, &mut -internal_alpha, depth - 1, position, info, hash_table, true); // negamax
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
                    hash_table.store_hash_entry(position, best_move, beta, HashFlag::HashFlagBeta as u8, depth);
                    return *beta;
                }
                internal_alpha = score;

                // if not a capture
                if extract_movelist_move(move_list.moves[move_number]) & MOVE_FLAG_CAPTURE == 0 { position.search_history[position.pieces[from_square(best_move) as usize] as usize][to_square(best_move) as usize] += depth as u32; } // history heuristic
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
        hash_table.store_hash_entry(position, best_move, &mut best_score, HashFlag::HashFlagExact as u8, depth);
    } else {
        hash_table.store_hash_entry(position, best_move, &mut internal_alpha, HashFlag::HashFlagAlpha as u8, depth);
    }

    internal_alpha
}

/**
Performs iterative deepening for a single worker thread.

This function serves as the main entry point for each search thread. It repeatedly calls the `alpha_beta` search function, increasing the search depth by one in each iteration, until the maximum depth is reached or a stop signal is received.

# Parameters
- `thread_data`: A mutable instance of `SearchWorkerData`, containing this thread's unique state, such as its board copy, thread number, and a shared pointer to the global `SearchInfo`.
- `hash_table`: A reference to the shared `HashTable` used for collaborative caching between threads.

# Returns
The best move (`u32`) found by this thread's search.

# Logic
1.  Initiates a loop that iterates from a depth of 1 up to the target depth stored in `thread_data.info`.
2.  In each iteration, it first calls `info.check_up()` to see if a stop condition (like a timeout) has been met.
3.  Calls the `alpha_beta` function to perform a full search at the `current_depth`.
4.  If the global `info.is_stopped()` flag becomes true, the loop terminates early.
5.  After a search completes for a given depth, it retrieves the best line of play (the Principal Variation) using `get_pv_line`. The first move of this line is stored as the current best move.
6.  If this is the primary worker thread (`thread_number == 0`), it prints the standard UCI `info` string to the console, detailing the search progress.
7.  Once the loop concludes, it atomically stores the final `best_move` found and returns it.
*/
pub fn iterative_deepen(mut thread_data: SearchWorkerData, hash_table: &HashTable) -> u32 {
    let mut best_move = NO_MOVE;
    let mut best_score: i32;

    for current_depth in 1..=thread_data.info.get_depth() {
        thread_data.info.check_up();
        best_score = alpha_beta(&mut -AB_BOUND, &mut 30000, current_depth, &mut thread_data.position, &thread_data.info, hash_table, true );

        if thread_data.info.is_stopped() { break; }

        // Update best move from PV line
        let pv_moves = get_pv_line(current_depth as u8, &mut thread_data.position, hash_table);
        if pv_moves > 0 { best_move = thread_data.position.pv_array[0]; }

        // Only main thread (thread 0) prints info
        if thread_data.thread_number == 0 {
            if let Ok(protected) = thread_data.info.protected.read() {
                print!("info score cp {} depth {} nodes {} time {} pv", best_score, current_depth, thread_data.info.get_nodes(), protected.start_time.elapsed().as_millis());

                for pv_number in 0..pv_moves { print!(" {}", print_move(thread_data.position.pv_array[pv_number])); }
                println!();
            }
        }
    }

    thread_data.set_best_move(best_move);
    best_move
}

/**
Orchestrates a multi-threaded search for the best move in a given position.

This is the top-level entry point for initiating a search. It handles setting up shared data, spawning worker threads, and synchronizing their results to determine the final best move.

# Parameters
- `position`: A mutable reference to the root `Board` state to be analyzed.
- `info`: An `Arc<SearchInfo>` containing the shared search parameters and control signals (time, depth, stop flags).
- `hash_table`: An `Arc<HashTable>` for the shared transposition table that threads use to collaborate.

# Logic
1.  Prepares the search environment by calling `clear_for_search`.
2.  First, attempts to find a move from an opening book if the option is enabled.
3.  If no book move is available, it proceeds to spawn worker threads.
4.  For each requested thread, it creates a `SearchWorkerData` struct, giving each thread its own clone of the board and shared pointers to the `info` and `hash_table`.
5.  Each worker is spawned into an OS thread, with `iterative_deepen` as its main function.
6.  After spawning all threads, the function waits for each thread to complete its search by calling `.join()` on its handle.
7.  It collects the best move returned by any of the threads.
8.  Finally, it prints the overall best move to the console in standard UCI format.
*/
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
                // depth: info.get_depth() as u8,
                best_move: AtomicU32::new(NO_MOVE),
            };

            let table = Arc::clone(&hash_table);
            worker_threads.push(thread::spawn(move || { iterative_deepen(thread_data, &table) }));
        }

        for handle in worker_threads {
            if let Ok(move_result) = handle.join() {
                if move_result != NO_MOVE { best_move = move_result; }
            }
        }
    }

    println!("bestmove {}", print_move(best_move));
}
