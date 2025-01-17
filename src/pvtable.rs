use colored::Colorize;

use crate::{defs::{Board, HashTable, DEBUG, MAX_DEPTH, NO_MOVE}, makemove::{make_move, take_move}, movegen::move_exists};

pub fn get_pv_line(depth: u8, position: &mut Board) -> usize{
    if DEBUG && (depth > MAX_DEPTH as u8 || depth < 1) { eprintln!("{}", "get_pv_line: [depth] greater than MAX_DEPTH or less than 1".red()); }

    let mut count: usize = 0;
    let mut the_move = position.probe_pv_move();

    while the_move != NO_MOVE && count < depth as usize {
        if DEBUG && count > MAX_DEPTH { eprintln!("{}", "get_pv_line: [count] greater than MAX_DEPTH".red()); }

        if move_exists(position, the_move) {
            make_move(position, the_move);
            position.pv_array[count] = the_move;
            count += 1;
        } else {
            break;
        }

        the_move = position.probe_pv_move();
    }

    while position.ply > 0 { take_move(position); }

    count
}

pub fn clear_hash_table(hash_table: &mut HashTable) {
    hash_table.pv_table.clear();
    hash_table.new_write = 0;
}
