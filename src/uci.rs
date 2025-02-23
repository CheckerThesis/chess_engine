use colored::Colorize;

use crate::{board::{parse_fen, print_board}, defs::{Board, SearchInfo, BLACK, ENGINE_OPTIONS, HASH_TABLE, MAX_DEPTH, WHITE}, io::parse_move, makemove::make_move, perft::perft_test, pvtable::clear_hash_table, search::search_position, FEN_START};
use std::{io::{self, BufRead}, sync::{atomic::Ordering, Arc}, thread, time::{Duration, Instant}};

// go depth 6 wtime 1000 btime 1000 binc 1000 winc 1000 movetime 1000 movestogo 40
pub fn parse_go(input: &String, info: Arc<SearchInfo>, position: &mut Board) {
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

        println!("time: {:?}    start: {:?}    stop: {:?}    depth: {}\ntimeset: {}    info.stopped: {}",
        time, protected.start_time, protected.stop_time, info.depth.load(Ordering::Relaxed), info.time_set.load(Ordering::Relaxed), info.stopped.load(Ordering::Relaxed));
    }

    let mut position_clone = position.clone();
    let search_info = Arc::clone(&info);
    let table = Arc::clone(&HASH_TABLE);

    thread::spawn(move || {
        if let Ok(mut hash_table) = table.lock() {
            search_position(&mut position_clone, search_info, &mut hash_table);
        } else {
            eprintln!("{}", "parse_go: failed to acquire lock on HASH_TABLE".red());
        }
    });
}

// position fen fenstr
// position startpos
// .. moves e2e4 e7e5 ..
pub fn parse_position(input: &String, position: &mut Board) {
    let mut moves_index: usize = input.len();
    if let Some(move_index) = input.find("moves") { moves_index = move_index; }

    if input.contains("startpos") {
        parse_fen(FEN_START, position);
    } else if let Some(fen_index) = input.find("fen") {
        parse_fen(&input[fen_index + 4..moves_index - 1], position);
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

    print_board(position);
}

// TODO if stop, bring back search thread maybe need to make a gamestate struct that stores
// the handle for the thread
pub fn uci_loop() {
    let name = "Vault";
    let mut testing = true;

    let mut user_input = String::new();
    println!("id name {}", name.to_string());
    println!("id author Tein Cow");
    println!("uciok");

    let position: &mut Board = &mut Board::default();
    let info = SearchInfo::new();

    let mut i = 0;

    loop {
        user_input.clear();

        if testing {
            i += 1;
            if i == 1 {
                user_input = "position fen rn1qkb1r/pp2pppp/5n2/3p1b2/3P4/2N1P3/PP3PPP/R1BQKBNR w KQkq - 0 1".to_string();
                // user_input = "quit".to_string();
                // user_input = "position startpos".to_string();
                // user_input = "quit".to_string();
            } else if i == 2 {
                // user_input = "setoption name Book value false".to_string();
                user_input = "go depth 10".to_string();
            // }
            // else if i == 3 {
            //     user_input = "position startpos moves e2e4".to_string();
            // }
            // else if i == 4 {
            //     user_input = "go depth 7".to_string();
            } else {
                io::stdin().lock().read_line(&mut user_input).unwrap();
            }
        } else {
            io::stdin().lock().read_line(&mut user_input).unwrap();
        }
        user_input = user_input.trim().to_string();

        if user_input == "" { continue; }

        if user_input == "isready" {
            println!("readyok");
            continue;

        } else if user_input.contains("position") {
            parse_position(&user_input, position);

        } else if user_input == "ucinewgame" {
            // HASH_TABLE.clear();
            clear_hash_table(&mut HASH_TABLE.lock().unwrap());
            parse_position(&"position startpos".to_string(), position);

        } else if user_input.contains("go") {
            parse_go(&user_input, Arc::clone(&info), position);

        } else if user_input == "stop" {
            info.stopped.store(true, Ordering::Relaxed);
        } else if user_input == "quit" {
            break;

        } else if user_input == "uci" {
            println!("id name {}", name.to_string());
            println!("id author Tien Cow");
            println!("uciok");

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
        }
    }
}
