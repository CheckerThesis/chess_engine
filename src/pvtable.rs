
use colored::Colorize;
use rand::{thread_rng, Rng};

use crate::{board::parse_fen, defs::{extract_movelist_move, Board, HashTable, MoveList, AB_BOUND, DEBUG, INF_BOUND, MAX_DEPTH, NO_MOVE}, io::print_move, makemove::{make_move, take_move}, movegen::{generate_all_moves, move_exists}};

#[inline(always)]
pub fn extract_score(data: u64) -> i32 { (data & 0xFFFF) as i32 - INF_BOUND as i32 }
#[inline(always)]
pub fn extract_depth(data: u64) -> u64 { (data >> 16) & 0x3F }
#[inline(always)]
pub fn extract_flags(data: u64) -> u64 { (data >> 23) & 0x3 }
#[inline(always)]
pub fn extract_move(data: u64) -> u64 { data >> 25 }

#[inline(always)]
pub fn fold_data(score: i32, depth: u64, flags: u64, the_move: u32) -> u64 {
    (score + INF_BOUND as i32) as u64 |
    (depth << 16) |
    (flags << 23) |
    ((the_move as u64) << 25)
}

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

pub fn get_pv_line(depth: u8, position: &mut Board, hash_table: &HashTable) -> usize{
    if DEBUG && (depth > MAX_DEPTH as u8 || depth < 1) { eprintln!("{}", "get_pv_line: [depth] greater than MAX_DEPTH or less than 1".red()); }

    let mut count: usize = 0;
    let mut the_move = hash_table.probe_pv_move(&position);

    while the_move != NO_MOVE && count < depth as usize {
        if DEBUG && count > MAX_DEPTH { eprintln!("{}", "get_pv_line: [count] greater than MAX_DEPTH".red()); }

        if move_exists(position, the_move) {
            make_move(position, the_move);
            position.pv_array[count] = the_move;
            count += 1;
        } else {
            break;
        }

        the_move = hash_table.probe_pv_move(&position);
    }

    while position.ply > 0 { take_move(position); }

    count
}

pub fn clear_hash_table(hash_table: &HashTable) {
    hash_table.clear();
    hash_table.set_new_write(0);
    hash_table.set_current_age(0);
}

// checks if table has an entry that matches the current position, if found set the_move equal to the stored move in the hash
// if the score is within proper bounds of alpha-beta, set score and prune in the alpha-beta function
// pub fn probe_hash_table(position: &mut Board, hash_table: &mut HashTable, the_move: &mut u32, score: &mut i32, alpha: i32, beta: i32, depth: i32) -> bool {
//     let i = position.position_key as usize % hash_table.pv_table.capacity();

//     if DEBUG {
//         if depth > MAX_DEPTH as i32 || depth < 1 { eprintln!("{}", "probe_hash_table: [depth] out of bounds".red()); }
//         if alpha >= beta { eprintln!("{}", "probe_hash_table: [alpha] greater than beta".red()); }
//         if alpha > AB_BOUND || alpha < -AB_BOUND { eprintln!("{}", "probe_hash_table: [alpha] out of bounds".red()); }
//         if beta > AB_BOUND || beta < -AB_BOUND { eprintln!("{}", "probe_hash_table: [beta] out of bounds".red()); }
//     }

//     if hash_table.pv_table[i].position_key == position.position_key {
//         *the_move = hash_table.pv_table[i].the_move;

//         if hash_table.pv_table[i].depth >= depth {
//             hash_table.hit += 1;

//             // set mate score for alpha beta
//             *score = hash_table.pv_table[i].score;
//             if *score > IS_MATE {
//                 *score -= position.ply as i32;
//             } else if *score < -IS_MATE {
//                 *score += position.ply as i32;
//             }

//             // if it is a cutoff
//             match hash_table.pv_table[i].flags {
//                 x if x == HashFlag::HashFlagAlpha as u8 => {
//                     if *score <= alpha {
//                         *score = alpha;
//                         return true
//                     }
//                 }
//                 x if x == HashFlag::HashFlagBeta as u8 => {
//                     if *score >= beta {
//                         *score = beta;
//                         return true
//                     }
//                 }
//                 x if x == HashFlag::HashFlagExact as u8 => {
//                     return true
//                 }
//                 _ => return false
//             }
//         }
//     }

//     false
// }

// pub fn store_hash_entry(position: &mut Board, hash_table: &mut HashTable, the_move: u32, score: &mut i32, flags: u8, depth: i32) {
//     let i = position.position_key as usize % hash_table.pv_table.capacity();
//     // if (print_move(the_move) == "f8e7") {
//     //     println!("f8e7");
//     // }

//     if DEBUG {
//         if i > hash_table.pv_table.capacity() - 1 { eprintln!("{}", "store_hash_entry: [i] out of bounds".red()); }
//         if depth > MAX_DEPTH as i32 || depth < 1 { eprintln!("{}", "store_hash_entry: [depth] out of bounds".red()); }
//         if position.ply >= MAX_DEPTH as u8 { eprintln!("{}", "store_hash_entry: [ply] out of bounds".red()); }
//     }

//     let mut replace = false;

//     if hash_table.pv_table[i].position_key == 0 {
//         hash_table.new_write += 1;
//         replace = true;
//     } else {
//         if hash_table.pv_table[i].age < hash_table.current_age || hash_table.pv_table[i].depth < depth {
//             replace = true
//         }
//     }

//     if replace == false { return; }

//     // reset mate score back to infinite
//     if *score > IS_MATE {
//         *score += position.ply as i32
//     } else if *score < -IS_MATE {
//         *score -= position.ply as i32;
//     }

//     hash_table.pv_table[i].the_move = the_move;
//     hash_table.pv_table[i].position_key = position.position_key;
//     hash_table.pv_table[i].flags = flags;
//     hash_table.pv_table[i].score = *score;
//     hash_table.pv_table[i].depth = depth;
//     hash_table.pv_table[i].age = hash_table.current_age;

//     // if (print_move(the_move) == "f8e7") {
//     //     println!("mv: {}   poskey: {}   score: {}   depth: {}   age: {}",
//     //         print_move(hash_table.pv_table[i].the_move),
//     //         hash_table.pv_table[i].position_key,
//     //         hash_table.pv_table[i].score,
//     //         hash_table.pv_table[i].depth,
//     //         hash_table.pv_table[i].age
//     //     );
//     // }
// }

// #[inline(always)]
// pub fn probe_pv_move(position: &Board, hash_table: &mut HashTable) -> u32 {
//     let i = position.position_key as usize % hash_table.pv_table.capacity();

//     // if DEBUG && i < 0 || i > position.hash_table.pv_table.capacity() - 1 { eprintln!("{}", "probe_pv_move: [i] out of bounds".red()); }

//     if hash_table.pv_table[i].position_key == position.position_key {
//         // println!("   PROBED {}", print_move(hash_table.pv_table[i].the_move));
//         return hash_table.pv_table[i].the_move }
//     return NO_MOVE
// }
