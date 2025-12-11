use crate::{board::Board, fens::FEN_START, movegen::MoveList};


pub fn perft(position: &mut Board, depth: usize) -> usize {
    let mut move_list = MoveList::new();
    let mut nodes = 0;

    move_list.generate_all_moves(&position);

    if depth == 1 { return move_list.count }

    for i in 0..move_list.count {
        if !position.make_move(move_list.moves[i] as u32) { continue; }
        nodes += perft(position, depth - 1);
        position.take_move();
    }

    return nodes
}