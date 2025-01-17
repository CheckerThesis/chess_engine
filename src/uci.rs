use crate::{board::{parse_fen, print_board}, defs::{Board, HashTable, SearchInfo, BLACK, ENGINE_OPTIONS, HASH_TABLE, MAX_DEPTH, WHITE}, io::parse_move, makemove::make_move, polybook::{self, POLY_BOOK}, search::search_position, FEN_START};
use std::{io::{self, BufRead}, time::{Duration, Instant}};

// go depth 6 wtime 1000 btime 1000 binc 1000 winc 1000 movetime 1000 movestogo 40
pub fn parse_go(input: &String, info: &mut SearchInfo, position: &mut Board, hash_table: &mut HashTable) {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut i = 0;

    let mut depth = None;
    let mut moves_to_go = 30;
    let mut move_time = None;
    let mut time = None;
    let mut increment = Duration::from_millis(0);
    let mut infinite = false;

    info.time_set = false;
    // info.stopped = false;

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

    if infinite {
        info.time_set = false;
        // info.stop_time = Instant::now() + Duration::from_secs(u64::MAX);
    } else {
        if let Some(_m) = move_time {
            time = move_time;
            moves_to_go = 1;
        }

        info.start_time = Instant::now();

        if let Some(t) = time {
            info.time_set = true;
            let mut time_final = t / moves_to_go as u32;
            time_final -= Duration::from_millis(50);
            time = Some(time_final);
            info.stop_time = info.start_time + time.unwrap() + increment;
        }
    }

    if let Some(d) = depth {
        info.depth = d;
    } else {
        info.depth = MAX_DEPTH as i32;
    }

    println!("time: {:?}    start: {:?}    stop: {:?}    depth: {}\ntimeset: {}    info.stopped: {}",
    time, info.start_time, info.stop_time, info.depth, info.time_set, info.stopped);
    search_position(position, info, hash_table);
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
        let mut the_move: u64;
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

pub fn uci_loop() {
    let name = "Vault";
    let z = true;

    let mut user_input = String::new();
    println!("id name {}", name.to_string());
    println!("id author Tein Cow");
    println!("uciok");

    let position: &mut Board = &mut Board::default();
    let info: &mut SearchInfo = &mut SearchInfo::default();

    let mut test_i = 0;

    loop {
        user_input.clear();

        if z {
            test_i += 1;
            if test_i == 1 {
                user_input = "position startpos".to_string();
                // user_input = "position fen r3kb1r/3n1pp1/p6p/2pPp2q/Pp2N3/3B2PP/1PQ2P2/R3K2R w KQkq - 0 1".to_string();
            } else if test_i == 2 {
                user_input = "setoption name Book value false".to_string();
            } else if test_i == 3 {
                user_input = "go depth 8".to_string();
            } else if test_i == 4 {
                // user_input = "go depth 3".to_string();
            } else if test_i == 5{
                // user_input = "go depth 9".to_string();
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
            parse_position(&"position startpos".to_string(), position);
        } else if user_input.contains("go") {
            // where the global HASH_TABLE gets dropped in
            parse_go(&user_input, info, position, &mut HASH_TABLE.lock().unwrap());
        } else if user_input == "quit" {
            info.quit = true;
            break;
        } else if user_input == "uci" {
            println!("id name {}", name.to_string());
            println!("id author Tien Cow");
            println!("uciok");
        } else if user_input.contains("setoption name Book value ") {
            // let init: &Vec<polybook::PolyBookEntry> = &*POLY_BOOK;

            let mut options = ENGINE_OPTIONS.lock().unwrap();

            if user_input.contains("true") {
                options.book = true;
            } else {
                options.book = false;
            }
            println!("options.book: {}", options.book);
        }
        if info.quit { break; }
    }
}
