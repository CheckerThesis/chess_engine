use crate::{board::Board, fens::FEN_START, movegen::{MoveList, SQUARE_TO_STRING, from_square, to_square}};


pub fn perft(position: &mut Board, depth: usize) -> usize {
    let mut move_list = MoveList::new();
    let mut nodes = 0;

    move_list.generate_all_moves(&position);

    if depth == 0 { return 1 }

    for i in 0..move_list.count {
        if !position.make_move(move_list.moves[i] as u32) { continue; }
        nodes += perft(position, depth - 1);
        position.take_move();
    }

    return nodes
}

// pub fn perft(position: &mut Board, depth: usize) -> usize {
//     let mut move_list = MoveList::new();
//     let mut nodes = 0;

//     if depth == 0 { return 1 }

//     move_list.generate_all_moves(position);
//     for i in 0..move_list.count {
//         if !position.make_move(move_list.moves[i] as u32) { continue; }
//         nodes += perft(position, depth - 1);
//         position.take_move();
//     }

//     nodes
// }

pub fn perft_divide(position: &mut Board, depth: usize) {
    let mut move_list = MoveList::new();
    let mut total = 0;

    move_list.generate_all_moves(&position);

    for i in 0..move_list.count {
        if move_list.moves[i] == 1357 {
            println!("{}", move_list.moves[i]);
        }
        if !position.make_move(move_list.moves[i] as u32) { 
            continue; 
        }

        let nodes = if depth == 1 { 1 } else { perft(position, depth - 1) };
        total += nodes;

        let mv = move_list.moves[i] as u32;
        let from = from_square(mv);
        let to = to_square(mv);
        println!("{}{}: {}", 
            SQUARE_TO_STRING[from], 
            SQUARE_TO_STRING[to], 
            nodes);

        position.take_move();
    }
    println!("\nTotal: {}", total);
}
