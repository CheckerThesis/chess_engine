
use colored::Colorize;

use crate::{defs::{Board, HashTable, DEBUG, MAX_DEPTH, NO_MOVE}, makemove::{make_move, take_move}, movegen::move_exists};
/*
pub fn data_check(the_move: u32) {
    let mut rng = thread_rng();
    let depth = rng.gen_range(0..1000) % MAX_DEPTH;
    let flags = rng.gen_range(0..1000) % 3;
    let score = rng.gen_range(-300..300) % AB_BOUND;

    let data = fold_data(score, depth as u64, flags as u64, the_move);

    println!("Original: move: {}  depth: {}  flags: {}  score: {}", print_move(the_move), depth, flags, score);
    println!("Folded:   move: {}  depth: {}  flags: {}  score: {}\n", print_move(extract_move(data) as u32), extract_depth(data), extract_flags(data), extract_score(data));
}

pub fn hash_test(fen: String) {
    let mut position = Board::default();
    parse_fen(&fen, &mut position);

    let move_list = &mut MoveList::default();
    generate_all_moves(&mut position, move_list);

    for move_number in 0..move_list.count {
        if !make_move(&mut position, extract_movelist_move(move_list.moves[move_number])) { continue; }

        take_move(&mut position);
        data_check(extract_movelist_move(move_list.moves[move_number]));
    }
}
    */

/**
Retrieves the Principal Variation (PV) line from the `HashTable` for the given position.

The PV is the sequence of moves considered to be best for both sides. This function iteratively probes the hash table to reconstruct this line.

# Parameters
- `depth`: The maximum number of moves to retrieve for the PV line.
- `position`: A mutable reference to the `Board` struct. The board state is temporarily modified as moves are made to trace the line.
- `hash_table`: A reference to the `HashTable`, which stores the PV moves found during the search.

# Returns
The number of moves in the PV line that were successfully retrieved and stored in `position.pv_array`.
*/
pub fn get_pv_line(depth: u8, position: &mut Board, hash_table: &HashTable) -> usize {
    if DEBUG && (depth > MAX_DEPTH as u8 || depth < 1) { eprintln!("{}", "get_pv_line: [depth] greater than MAX_DEPTH or less than 1".red()); }

    let mut count: usize = 0;
    let mut the_move = hash_table.probe_pv_move(&position);

    while the_move != NO_MOVE && count < depth as usize {
        if DEBUG && count > MAX_DEPTH { eprintln!("{}", "get_pv_line: [count] greater than MAX_DEPTH".red()); }

        if move_exists(position, the_move) {
            make_move(position, the_move);
            position.pv_array[count] = the_move;
            count += 1;
        } else { break; }

        the_move = hash_table.probe_pv_move(&position);
    }

    while position.ply > 0 { take_move(position); }

    count
}

pub fn clear_hash_table(hash_table: &HashTable) {
    hash_table.clear();
    hash_table.set_new_write(0);
    hash_table.set_over_write(0);
    hash_table.set_current_age(0);
    hash_table.set_cut(0);
}
