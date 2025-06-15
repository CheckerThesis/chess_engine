use std::time::Instant;

use colored::Colorize;

use crate::{board::{check_board, parse_fen, Board}, defs::DEBUG, makemove::{make_move, take_move}, movegen::{extract_movelist_move, generate_all_moves, MoveList}};

fn perft(depth: u8, position: &mut Board, leaf_nodes: &mut u64) {
    if DEBUG { check_board(position); }

    if depth == 0 {
        *leaf_nodes += 1;
        return;
    }

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    for move_number in 0..move_list.count {
        if !make_move(position, extract_movelist_move(move_list.moves[move_number])) { continue; }

        perft(depth - 1, position, leaf_nodes);
        take_move(position);
    }
}

pub fn perft_test(depth: u8, position: &mut Board) -> u64 {
    if DEBUG { check_board(position); }

    // print_board(position);
    println!("Start test to depth: {}", depth);
    let mut leaf_nodes = 0;
    let now = Instant::now();

    let move_list = &mut MoveList::default();
    generate_all_moves(position, move_list);

    for move_number in 0..move_list.count {
        let the_move = extract_movelist_move(move_list.moves[move_number]);
        if !make_move(position, the_move) { continue; }
        let old_leaf_nodes = leaf_nodes;
        perft(depth - 1, position, &mut leaf_nodes);
        take_move(position);
        let _nodes = leaf_nodes - old_leaf_nodes;
        // println!("move {} : {} : {}", move_number + 1, print_move(the_move), nodes);
    }

    println!("Test complete: {} leaf nodes visited. Took {} seconds.", leaf_nodes, now.elapsed().as_secs());
    leaf_nodes
}

pub fn automatic_perft_test(position: &mut Board) {
    let perft_outputs: [u64; 4] = [119060324, 8031647685, 11030083, 706045033];
    let fens = [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1 "
    ];

    for i in 0..4 {
        parse_fen(fens[i], position);
        if perft_test(6, position) == perft_outputs[i] { println!("position {} correct!", format!("{}", i).green()) }

    }
}