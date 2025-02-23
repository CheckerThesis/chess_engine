use std::time::Instant;

use crate::{board::{check_board, print_board}, defs::{extract_movelist_move, Board, MoveList, DEBUG}, io::print_move, makemove::{make_move, take_move}, movegen::generate_all_moves};

pub fn perft(depth: u8, position: &mut Board, leaf_nodes: &mut u64) {
    if DEBUG { check_board(position); }

    if depth == 0 {
        *leaf_nodes += 1;
        return;
    }

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    for move_number in 0..move_list.count {
        if !make_move(position, extract_movelist_move(move_list.moves[move_number])) {
            continue;
        }

        perft(depth - 1, position, leaf_nodes);
        take_move(position);
    }
}

pub fn perft_test(depth: u8, position: &mut Board) {
    if DEBUG { check_board(position); }

    print_board(position);
    println!("Start test to depth: {}", depth);
    let mut leaf_nodes = 0;
    let now = Instant::now();

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    for move_number in 0..move_list.count {
        let the_move = extract_movelist_move(move_list.moves[move_number]);
        if !make_move(position, the_move) {
            continue;
        }
        let old_leaf_nodes = leaf_nodes;
        perft(depth - 1, position, &mut leaf_nodes);
        take_move(position);
        let nodes = leaf_nodes - old_leaf_nodes;
        println!("move {} : {} : {}", move_number + 1, print_move(the_move), nodes);
    }

    println!("Test complete: {} leaf nodes visited. Took {} seconds.", leaf_nodes, now.elapsed().as_secs());
}
