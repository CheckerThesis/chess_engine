use colored::Colorize;

use crate::{defs::{Board, DEBUG, MAX_DEPTH}, makemove::{make_move, take_move}, movegen::move_exists};


pub fn get_pv_line(depth: u8, position: &mut Board) -> usize{
    if DEBUG && depth > MAX_DEPTH as u8 { eprintln!("{}", "get_pv_line: [depth] greater than MAX_DEPTH".red()); }

    let mut count: usize = 0;

    while let Some(the_move) = position.probe_pv_table(position.position_key) {
        if DEBUG && count > MAX_DEPTH { eprintln!("{}", "get_pv_line: [count] greater than MAX_DEPTH".red()); }

        if move_exists(position, the_move) {
            make_move(position, the_move);
            position.pv_array[count] = the_move;
            count += 1;
        } else {
            break;
        }
    }

    while position.ply > 0 { take_move(position); }

    count
}
