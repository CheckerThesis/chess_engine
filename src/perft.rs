use crate::{board::Board, fens::FEN_START, movegen::{MoveList, SQUARE_TO_STRING, captured, from_square, to_square}};


pub fn perft(position: &mut Board, depth: usize) -> usize {
    // TODO in here add similar debugging to perft_divide, 1177
    let mut move_list = MoveList::new();
    let mut nodes = 0;

    println!("AHHHHHHHHHHHHH");
    move_list.generate_all_moves(&position);
    println!("{move_list}");
    println!("{}", move_list.moves[20] as u32);
    println!("{}", move_list.moves[20]);

    if depth == 0 { return 1 }

    for i in 0..move_list.count {
        let mv = move_list.moves[i] as u32;
        let captured = captured(mv);
        let from = from_square(mv);
        let to = to_square(mv);
        println!("Before: {}{}", 
            SQUARE_TO_STRING[from], 
            SQUARE_TO_STRING[to], 
        );
        println!("{position}");
        let test = position.make_move(mv);
        println!("After: {}{}", 
            SQUARE_TO_STRING[from], 
            SQUARE_TO_STRING[to], 
        );
        println!("{position}");

        if !test { continue; }
        nodes += perft(position, depth - 1);
        position.take_move();
        println!("After take_move");
        println!("{position}");
    }

    return nodes
}

pub fn perft_divide(position: &mut Board, depth: usize) {
    let mut move_list = MoveList::new();
    let mut total = 0;

    move_list.generate_all_moves(&position);

    for i in 0..move_list.count {
        let mv = move_list.moves[i] as u32;
        let from = from_square(mv);
        let to = to_square(mv);
        println!("Before: {}{}", 
            SQUARE_TO_STRING[from], 
            SQUARE_TO_STRING[to], 
        );
        println!("{position}");
        let test = position.make_move(mv);
        println!("After: {}{} ({})", 
            SQUARE_TO_STRING[from], 
            SQUARE_TO_STRING[to],
            test 
        );
        println!("{position}");
        if !test {
            continue; 
        }

        let nodes = if depth == 1 { 1 } else { perft(position, depth - 1) };
        println!("{position}");
        total += nodes;

        let mv = move_list.moves[i] as u32;
        let from = from_square(mv);
        let to = to_square(mv);
        // println!("{}{}: {}", 
        //     SQUARE_TO_STRING[from], 
        //     SQUARE_TO_STRING[to], 
        //     nodes
        // );

        println!("Before take_move");
        println!("{position}");
        position.take_move();
        println!("After take_move");
        println!("{position}");
    }
    println!("\nTotal: {}", total);
}
