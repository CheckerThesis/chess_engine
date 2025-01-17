use colored::Colorize;

use crate::{defs::{Board, HashFlag, HashTable, DEBUG, INF_BOUND, IS_MATE, MAX_DEPTH, NO_MOVE}, makemove::{make_move, take_move}, movegen::move_exists};

pub fn get_pv_line(depth: u8, position: &mut Board, hash_table: &mut HashTable) -> usize{
    if DEBUG && (depth > MAX_DEPTH as u8 || depth < 1) { eprintln!("{}", "get_pv_line: [depth] greater than MAX_DEPTH or less than 1".red()); }

    let mut count: usize = 0;
    let mut the_move = probe_pv_move(&position, hash_table);

    while the_move != NO_MOVE && count < depth as usize {
        if DEBUG && count > MAX_DEPTH { eprintln!("{}", "get_pv_line: [count] greater than MAX_DEPTH".red()); }

        if move_exists(position, the_move) {
            make_move(position, the_move);
            position.pv_array[count] = the_move;
            count += 1;
        } else {
            break;
        }

        the_move = probe_pv_move(&position, hash_table);
    }

    while position.ply > 0 { take_move(position); }

    count
}

pub fn clear_hash_table(hash_table: &mut HashTable) {
    hash_table.pv_table.clear();
    hash_table.new_write = 0;
}

// checks if table has an entry that matches the current position, if found set the_move equal to the stored move in the hash
// if the score is within proper bounds of alpha-beta, set score and prune in the alpha-beta function
pub fn probe_hash_table(position: &mut Board, hash_table: &mut HashTable, the_move: &mut u64, score: &mut i32, alpha: i32, beta: i32, depth: i32) -> bool {
    let i = position.position_key as usize % hash_table.pv_table.capacity();

    if DEBUG {
        if depth > MAX_DEPTH as i32 || depth < 1 { eprintln!("{}", "probe_hash_table: [depth] out of bounds".red()); }
        if alpha >= beta { eprintln!("{}", "probe_hash_table: [alpha] greater than beta".red()); }
        if alpha > INF_BOUND || alpha < -INF_BOUND { eprintln!("{}", "probe_hash_table: [alpha] out of bounds".red()); }
        if beta > INF_BOUND || beta < -INF_BOUND { eprintln!("{}", "probe_hash_table: [beta] out of bounds".red()); }
    }

    if hash_table.pv_table[i].position_key == position.position_key {
        *the_move = hash_table.pv_table[i].the_move;

        if hash_table.pv_table[i].depth >= depth {
            hash_table.hit += 1;

            // set mate score for alpha beta
            *score = hash_table.pv_table[i].score;
            if *score > IS_MATE {
                *score -= position.ply as i32;
            } else if *score < -IS_MATE {
                *score += position.ply as i32;
            }

            // if it is a cutoff
            match hash_table.pv_table[i].flags {
                x if x == HashFlag::HashFlagAlpha as u8 => {
                    if *score <= alpha {
                        *score = alpha;
                        return true
                    }
                }
                x if x == HashFlag::HashFlagBeta as u8 => {
                    if *score >= beta {
                        *score = beta;
                        return true
                    }
                }
                x if x == HashFlag::HashFlagExact as u8 => {
                    return true
                }
                _ => return false
            }
        }
    }

    false
}

pub fn store_hash_entry(position: &mut Board, hash_table: &mut HashTable, the_move: u64, score: &mut i32, flags: u8, depth: i32) {
    let i = position.position_key as usize % hash_table.pv_table.capacity();

    if DEBUG {
        // if i < 0 || i > position.hash_table.pv_table.capacity() - 1 { eprintln!("{}", "store_hash_entry: [i] out of bounds".red()); }
        if depth > MAX_DEPTH as i32 || depth < 1 { eprintln!("{}", "store_hash_entry: [depth] out of bounds".red()); }
        // if position.ply < 0 || position.ply >= MAX_DEPTH as u8 { eprintln!("{}", "store_hash_entry: [ply] out of bounds".red()); }
    }

    if hash_table.pv_table[i].position_key == 0 {
        hash_table.new_write += 1;
    } else {
        hash_table.over_write += 1;
    }

    // reset mate score back to infinite
    if *score > IS_MATE {
        *score += position.ply as i32
    } else if *score < -IS_MATE {
        *score -= position.ply as i32;
    }

    hash_table.pv_table[i].the_move = the_move;
    hash_table.pv_table[i].position_key = position.position_key;
    hash_table.pv_table[i].flags = flags;
    hash_table.pv_table[i].score = *score;
    hash_table.pv_table[i].depth = depth;
}

pub fn probe_pv_move(position: &Board, hash_table: &mut HashTable) -> u64 {
    let i = position.position_key as usize % hash_table.pv_table.capacity();

    // if DEBUG && i < 0 || i > position.hash_table.pv_table.capacity() - 1 { eprintln!("{}", "probe_pv_move: [i] out of bounds".red()); }

    if hash_table.pv_table[i].position_key == position.position_key { return hash_table.pv_table[i].the_move }
    return NO_MOVE
}
