use crate::{board::{parse_fen, print_board}, defs::{Board, SearchInfo, BLACK, MAX_DEPTH, NO_MOVE, WHITE}, io::parse_move, makemove::make_move, search::search_position, FEN_START};
use std::{io::{self, stdin, BufRead}, time::{Duration, Instant}};

// go depth 6 wtime 1000 btime 1000 binc 1000 winc 1000 movetime 1000 movestogo 40
pub fn parse_go(input: &String, info: &mut SearchInfo, position: &mut Board) {
    let tokens: Vec<&str> = input.split_whitespace().collect();
    let mut i = 0;

    let mut depth = None;
    let mut moves_to_go = 30;
    let mut move_time = None;
    let mut time = None;
    let mut increment = Duration::from_millis(0);

    info.time_set = false;

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
                depth = Some(tokens[i + 1].parse::<u8>().unwrap());
                i += 2;
            }
            "movetime" => {
                move_time = Some(Duration::from_millis(tokens[i + 1].parse::<u64>().unwrap()));
                i += 2;
            }
            _ => i += 1,
        }
    }

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

    if let Some(d) = depth {
        info.depth = d;
    } else {
        info.depth = MAX_DEPTH as u8;
    }

    // TODO go movetime not working

    println!("time: {:?}    start: {:?}    stop: {:?}    depth: {}    timeset: {}", time, info.start_time, info.stop_time, info.depth, info.time_set);
    search_position(position, info);
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
    let name = "Tien Cao";

    // set in and out buffer to 0?

    let mut user_input = String::new();
    println!("id name {}", name.to_string());
    println!("id author Tein Cow");
    println!("uciok");

    let position: &mut Board = &mut Board::default();
    let info: &mut SearchInfo = &mut SearchInfo::default();

    let mut test_i = 0;

    loop {
        test_i += 1;

        user_input.clear();

        if test_i == 1 {
            user_input = "position startpos".to_string();
        } else if test_i == 2 {
            user_input = "go wtime 180000 btime 180000".to_string();
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
            parse_go(&user_input, info, position);
        } else if user_input == "quit" {
            info.quit = true;
            break;
        } else if user_input == "uci" {
            println!("id name {}", name.to_string());
            println!("id author Tien Cow");
            println!("uciok");
        }
        if info.quit { break; }
    }
}
