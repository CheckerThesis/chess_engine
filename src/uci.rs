use colored::Colorize;

use crate::{
    attack::square_attacked, board::parse_fen, defs::{
        extract_movelist_move, Board, MoveList, SearchInfo, UciCommand, BLACK, ENGINE_OPTIONS, HASH_TABLE, MAX_DEPTH, MAX_THREADS, WHITE
    }, io::{parse_move, print_move}, makemove::{make_move, take_move}, movegen::generate_all_moves, perft::perft_test, pvtable::clear_hash_table, search::search_position, FEN_START
};
use std::{
    i32,
    io::{self, BufRead},
    sync::{atomic::Ordering, Arc},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

// go depth 6 wtime 1000 btime 1000 binc 1000 winc 1000 movetime 1000 movestogo 40
fn parse_go(input: &String, info: Arc<SearchInfo>, position: &mut Board) -> Option<JoinHandle<()>> {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut i = 0;

    let mut depth = None;
    let mut moves_to_go = 30;
    let mut move_time = None;
    let mut time = None;
    let mut increment = Duration::from_millis(0);
    let mut infinite = false;

    info.time_set.store(false, Ordering::Relaxed);

    while i < tokens.len() {
        match tokens[i] {
            "wtime" => {
                if position.side == WHITE as u8 {
                    time = Some(Duration::from_millis(tokens[i + 1].parse::<u64>().unwrap()));
                }
                i += 2;
            }
            "btime" => {
                if position.side == BLACK as u8 {
                    time = Some(Duration::from_millis(tokens[i + 1].parse::<u64>().unwrap()));
                }
                i += 2;
            }
            "winc" => {
                if position.side == WHITE as u8 {
                    increment = Duration::from_millis(tokens[i + 1].parse::<u64>().unwrap());
                }
                i += 2;
            }
            "binc" => {
                if position.side == BLACK as u8 {
                    increment = Duration::from_millis(tokens[i + 1].parse::<u64>().unwrap());
                }
                i += 2;
            }
            "movestogo" => {
                moves_to_go = tokens[i + 1].parse::<u8>().unwrap_or(30);
                i += 2;
            }
            "depth" => {
                depth = Some(tokens[i + 1].parse::<i32>().unwrap());
                i += 2;
            }
            "movetime" => {
                move_time = Some(Duration::from_millis(tokens[i + 1].parse::<u64>().unwrap()));
                i += 2;
            }
            "infinite" => {
                infinite = true;
                i += 1;
            }
            _ => i += 1,
        }
    }

    if let Ok(mut protected) = info.protected.write() {
        if infinite {
            info.time_set.store(false, Ordering::Relaxed);
        } else {
            if let Some(_m) = move_time {
                time = move_time;
                moves_to_go = 1;
            }

            protected.start_time = Instant::now();

            if let Some(t) = time {
                info.time_set.store(true, Ordering::Relaxed);
                let mut time_final = t / moves_to_go as u32;
                time_final -= Duration::from_millis(50);
                time = Some(time_final);
                protected.stop_time = protected.start_time + time.unwrap() + increment;
            }
        }

        if let Some(d) = depth {
            info.depth.store(d, Ordering::Relaxed);
        } else {
            info.depth.store(MAX_DEPTH as i32, Ordering::Relaxed);
        }

        println!("time: {:?}    start: {:?}    stop: {:?}    depth: {}\ntimeset: {}    info.stopped: {}    thread count: {}",
        time, protected.start_time, protected.stop_time, info.depth.load(Ordering::Relaxed), info.time_set.load(Ordering::Relaxed), info.stopped.load(Ordering::Relaxed), info.get_thread_num());
    }

    let mut position_clone = position.clone();
    let search_info = Arc::clone(&info);

    let main_search_thread = thread::spawn(move || {
        search_position(&mut position_clone, search_info, Arc::clone(&*HASH_TABLE));
    });

    Some(main_search_thread)
}

fn parse_position(input: &String, position: &mut Board) {
    let mut moves_index: usize = input.len();
    if let Some(move_index) = input.find("moves") {
        moves_index = move_index;
    }

    if input.contains("startpos") {
        parse_fen(FEN_START, position);
    } else if let Some(fen_index) = input.find("fen") {
        parse_fen(&input[fen_index + 4..moves_index - 1], position);
        let side;
        if position.side as usize == WHITE { side = BLACK; }
        else { side = WHITE; }
        if square_attacked(position.king_square[position.side as usize] as usize, side, position) {
            println!("bot in-check");
        }
    }

    if !(moves_index == input.len()) {
        let mut the_move;
        let moves = &input[moves_index + 6..];
        let moves_split: Vec<&str> = moves.split_whitespace().collect();

        for (_i, move_string) in moves_split.iter().enumerate() {
            the_move = parse_move(&move_string.to_string(), position);
            make_move(position, the_move);
            position.ply = 0;
        }
    }

    // print_board(position);
}

pub fn uci_loop() {
    let mut main_search_thread: Option<JoinHandle<()>> = None;
    let position: &mut Board = &mut Board::default();
    let info: Arc<SearchInfo> = SearchInfo::new();
    info.set_thread_num(1);

    print_uci_id();

    while handle_uci_command(String::from(""), &mut main_search_thread, position, &info) {}

    if let Some(thread) = main_search_thread.take() {
        match thread.join() {
            // Ok(_) => println!("Thread joined successfully."),
            Ok(_) => print!(""),
            Err(e) => eprintln!("Error joining thread: {:?}", e),
        }
    }
}

fn handle_uci_command(input: String, main_search_thread: &mut Option<JoinHandle<()>>, position: &mut Board, info: &Arc<SearchInfo>) -> bool {
    let mut user_input = String::new();

    if input == "" { io::stdin().lock().read_line(&mut user_input).unwrap(); }
    else { user_input = input; }

    user_input = user_input.trim().to_string();

    match parse_uci_command(&user_input) {
        UciCommand::Uci => print_uci_id(),
        UciCommand::IsReady => println!("readyok"),
        UciCommand::SetOption(_) => process_setoption(&user_input, info),
        UciCommand::UciNewGame => {
            clear_hash_table(&HASH_TABLE);
            parse_position(&"position startpos".to_string(), position);
        }
        UciCommand::Position(_) => parse_position(&user_input, position),
        UciCommand::Go(_) => *main_search_thread = parse_go(&user_input, Arc::clone(info), position),
        UciCommand::Stop => {
            info.stopped.store(true, Ordering::Relaxed);
            if let Some(thread) = main_search_thread.take() {
                if let Err(e) = thread.join() {
                    eprintln!("Error joining thread: {:?}", e);
                }
            }
        }
        UciCommand::Quit => {
            info.stopped.store(true, Ordering::Relaxed);
            return false;
        }
        UciCommand::Testing(_) => testing(&user_input, main_search_thread, position, info),
        UciCommand::Generate => generate(position),
        _ => {}
    }
    true
}
fn parse_uci_command(input: &str) -> UciCommand {
    let tokens: Vec<&str> = input.trim().split_whitespace().collect();

    if tokens.is_empty() { return UciCommand::Unknown }

    return match tokens[0] {
        "uci" => UciCommand::Uci,
        "isready" => UciCommand::IsReady,
        "setoption" => UciCommand::SetOption(input.to_string()),
        "ucinewgame" => UciCommand::UciNewGame,
        "position" => UciCommand::Position(input.to_string()),
        "go" => UciCommand::Go(input.to_string()),
        "stop" => UciCommand::Stop,
        "quit" => UciCommand::Quit,
        "testing" => UciCommand::Testing(String::from(input)),
        "generate" => UciCommand::Generate,
        _ => UciCommand::Unknown,
    }
}

fn print_uci_id() {
    let name = "Vault";
    let author = "Tein Cow";
    println!("id name {}\nid author {}\nuciok", name, author);
}

// TODO opening book
fn process_setoption(input: &str, info: &SearchInfo) {
    let tokens: Vec<&str> = input.trim().split_whitespace().collect();

    match tokens[2] {
        "Threads" => {
            info.set_thread_num(tokens[4].parse::<u8>().unwrap());
            // println!("threads count {}", info.get_thread_num());
        },
        _ => print!("")
    }
}

fn generate(position: &mut Board) {
    println!("movelist start");
    let mut move_list = MoveList::default();
    generate_all_moves(position, &mut move_list);

    let mut move_list_legal = Vec::with_capacity(move_list.count);
    let mut in_check = false;

    for mv in move_list.moves.iter().take(move_list.count) {
        let za_move = extract_movelist_move(*mv);
        let made_move = make_move(position, za_move);

        if made_move { // add check indicator to movelist output
            move_list_legal.push(print_move(za_move));
            take_move(position);
            // 4375 3095
        } else if square_attacked(position.king_square[position.side as usize] as usize, (position.side ^ 1) as usize, position) {
            // println!("position.king_square: {}    position.side: {}", position.king_square[position.side as usize], position.side);
            // print_board(position);
            // println!("{}    {}", za_move, print_move(za_move));
            in_check = true;
        }
    }

    for mv in move_list_legal {
        println!("{}", mv);
    }
    println!("movelist end {}", in_check);
}

fn testing(input: &str, main_search_thread: &mut Option<JoinHandle<()>>, position: &mut Board, info: &Arc<SearchInfo>) {
    let tokens: Vec<&str> = input.trim().split_whitespace().collect();

    match tokens[1] {
        "perft" => {
            let depth: Vec<&str> = input.split_whitespace().collect();
            perft_test(depth[1].parse::<u8>().unwrap(), position);
        },
        "uci" => uci_test(main_search_thread, position, info),
        _ => print!("")
    }
}
fn uci_test(main_search_thread: &mut Option<JoinHandle<()>>, position: &mut Board, info: &Arc<SearchInfo>) {
    let inputs = [
        String::from("position fen r2qkbnr/1b5p/p1n2p2/1pN1pNp1/1P1pP3/1Q4P1/PBPPBP1P/R3K2R w KQkq g6 0 1"),
        String::from("go depth 5"),
        String::from("go movetime 5000"),
        String::from("position fen r5k1/pppn2p1/3p4/4pq1n/1PPb3K/P2P1r2/8/1N6 w - - 3 56"),
        String::from("go wtime 15000 btime 15000 winc 1000 binc 1000"),
        String::from("go movetime 20000"),
        String::from("stop")
    ];
    let times = [0.5, 5.0, 5.0, 0.5, 4.0, 3.0, 0.0];

    for i in 0..inputs.len() {
        println!("{}", format!("{}", inputs[i].bright_magenta()));
        // println!("Sending: {}", inputs[i]);
        handle_uci_command(inputs[i].clone(), main_search_thread, position, info);

        thread::sleep(Duration::from_secs_f32(times[i]));
    }
}

fn uci_loo2p() {
    let name = "Vault";
    let mut testing = false;

    let mut user_input = String::new();

    let mut main_search_thread: Option<JoinHandle<()>> = None;
    let position: &mut Board = &mut Board::default();
    let info = SearchInfo::new();
    info.set_thread_num(2);
    println!("thread count {}", info.get_thread_num());

    println!("id name {}", name.to_string());
    println!("id author Tein Cow");
    println!("uciok");

    let mut i = 0;

    loop {
        user_input.clear();

        if testing {
            i += 1;
            if i == 1 {
                // user_input = "position fen rn1qkb1r/pp2pppp/5n2/3p1b2/3P4/2N1P3/PP3PPP/R1BQKBNR w KQkq - 0 1".to_string();
                // user_input = "position fen r2q1rk1/2p1bppp/p2p1n2/1p2P3/4P1b1/1nP1BN2/PP3PPP/RN1QR1K1 w - - 1 12".to_string();
                // user_input = "quit".to_string();
                user_input = "position fen r2qkbnr/1b5p/p1n2p2/1pN1pNpB/1P1pP3/1Q4P1/PBPP1P1P/R3K2R b KQkq - 1 2".to_string();
                // user_input = "quit".to_string();
            } else if i == 2 {
                // user_input = "setoption name Book value false".to_string();
                // user_input = "go depth 10".to_string();
                user_input = "go depth 8".to_string();
            } else {
                io::stdin().lock().read_line(&mut user_input).unwrap();
            }
        } else {
            io::stdin().lock().read_line(&mut user_input).unwrap();
        }
        
        user_input = user_input.trim().to_string();

        if user_input == "" {
            continue;
        }

        if user_input == "isready" {
            println!("readyok");
            continue;
        } else if user_input.contains("position") {
            parse_position(&user_input, position);
        } else if user_input == "ucinewgame" {
            clear_hash_table(&HASH_TABLE);
            parse_position(&"position startpos".to_string(), position);
        } else if user_input.contains("go") {
            main_search_thread = parse_go(&user_input, Arc::clone(&info), position);
        } else if user_input == "stop" {
            info.stopped.store(true, Ordering::Relaxed);

            if let Some(thread) = main_search_thread.take() {
                match thread.join() {
                    // Ok(_) => println!("Thread joined successfully."),
                    Ok(_) => print!(""),
                    Err(e) => eprintln!("Error joining thread: {:?}", e),
                }
            } else {
                println!("No thread to join.");
            }
        } else if user_input == "quit" {
            info.stopped.store(true, Ordering::Relaxed);
            break;
        } else if user_input == "uci" {
            println!("id name {}", name.to_string());
            println!("id author Tien Cow");
            println!("uciok");
        } else if user_input.contains("threads") {
            let temp: Vec<&str> = user_input.split_whitespace().collect();
            let temp2 = temp[1].parse::<i32>().unwrap();
            if temp2 > MAX_THREADS as i32 {
                println!("too many threads");
            } else {
                info.thread_num.store(temp2 as u8, Ordering::Relaxed);
                println!("threads count {}", info.get_thread_num());
            }
        } else if user_input.contains("setoption name Book value ") {
            if user_input.contains("true") {
                ENGINE_OPTIONS.lock().unwrap().book = true;
            } else {
                ENGINE_OPTIONS.lock().unwrap().book = false;
            }
        } else if user_input == "testing" {
            testing = true;
        } else if user_input.contains("perft") {
            let depth: Vec<&str> = user_input.split_whitespace().collect();
            perft_test(depth[1].parse::<u8>().expect(""), position);
        } else if user_input.contains("generate") {
            println!("movelist start");
            let mut move_list = MoveList::default();
            generate_all_moves(position, &mut move_list);

            let mut move_list_legal = Vec::with_capacity(move_list.count);
            let mut in_check = false;

            for mv in move_list.moves.iter().take(move_list.count) {
                let za_move = extract_movelist_move(*mv);
                let made_move = make_move(position, za_move);

                if made_move { // add check indicator to movelist output
                    move_list_legal.push(print_move(za_move));
                    take_move(position);
                    // 4375 3095
                } else if square_attacked(position.king_square[position.side as usize] as usize, (position.side ^ 1) as usize, position) {
                    // println!("position.king_square: {}    position.side: {}", position.king_square[position.side as usize], position.side);
                    // print_board(position);
                    // println!("{}    {}", za_move, print_move(za_move));
                    in_check = true;
                }
            }

            for mv in move_list_legal {
                println!("{}", mv);
            }
            println!("movelist end {}", in_check);
        } else if user_input.contains("data") {
            position.print_data();
        }
    }
}