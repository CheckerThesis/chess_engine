use std::{io::{self, BufRead, stdin}, sync::{Arc, atomic::Ordering}, thread::{JoinHandle, spawn}};

use crate::{board::{Board, position_keys}, defs::{Color, Piece, PieceType}, fens::FEN_START, movegen::{MOVE_FLAG_CASTLE, MOVE_FLAG_EN_PASSANT, MOVE_FLAG_NONE, Move}, search::Search, squares::move_to_indices, transposition_table::TranspositionTable};

pub enum UciCommand {
    Quit,
    Uci,
    SetOption(String),
    Position(String),
    UciNewGame,
    IsReady,
    Go(String),
    Stop,
    // ponder
    Unknown,

    Testing(String),
    Generate
}

pub fn uci_loop() {
    fn print_uci_id() {
        let name = "Vault";
        let author = "Tein Cow";
        println!("id name {}\nid author {}\nuciok", name, author);
    }

    fn parse_position(tokens: &[&str]) -> Board {
        fn parse_uci_move(position: &mut Board, move_string: &str) {
            let (from, to) = move_to_indices(move_string).unwrap();
            let captured = position.pieces[to];
            let moving_piece = position.pieces[from].piece_type();

            let mut flags = MOVE_FLAG_NONE;
            let mut promote = PieceType::NONE.index();

            if move_string.len() == 5 {
                promote = match move_string.as_bytes()[4] {
                    b'n' => PieceType::KNIGHT.index(),
                    b'b' => PieceType::BISHOP.index(),
                    b'r' => PieceType::ROOK.index(),
                    b'q' => PieceType::QUEEN.index(),
                    _ => PieceType::NONE.index(),
                }
            }

            if moving_piece == PieceType::KING {
                let diff = (to as i32 - from as i32).abs();
                if diff == 2 { flags = MOVE_FLAG_CASTLE; }
            }

            if moving_piece == PieceType::PAWN && captured == Piece::NONE {
                let file_diff = (to % 8) as i32 - (from % 8) as i32;
                if file_diff.abs() == 1 { flags = MOVE_FLAG_EN_PASSANT; }
            }

            if !position.make_move(Move::new(
                from, 
                to, 
                captured.index(), 
                flags, 
                promote
            )) { panic!() }
        }

        let moves_index = tokens.iter().position(|&t| t == "moves");
        let (pos_tokens, move_tokens) = match moves_index {
            Some(i) => (&tokens[..i], &tokens[i + 1..]),
            None => (tokens, &[][..]),
        };

        let mut board = match pos_tokens.get(0).copied() {
            Some("startpos") => Board::new(FEN_START),
            Some("fen") => {
                let fen = pos_tokens[1..].join(" ");
                Board::new(&fen)
            }
            _ => panic!("invalid position command"),
        };

        for mv in move_tokens {
            parse_uci_move(&mut board, mv);
        }

        board
    }

    fn parse_go(
        tokens: &[&str], 
        position: &Board, 
        tt: Arc<TranspositionTable>
    ) -> (Option<JoinHandle<()>>, Arc<Search>) {
        let mut depth = 64; // Default to max depth
        let mut wtime: Option<u64> = None;
        let mut btime: Option<u64> = None;
        let mut winc: u64 = 0;
        let mut binc: u64 = 0;
        let mut movestogo: u64 = 30;
        let mut movetime: Option<u64> = None;
        let mut infinite = false;

        // Parse tokens
        let mut i = 0;
        while i < tokens.len() {
            match tokens[i] {
                "wtime" => { wtime = tokens.get(i+1).and_then(|t| t.parse().ok()); i += 2; }
                "btime" => { btime = tokens.get(i+1).and_then(|t| t.parse().ok()); i += 2; }
                "winc" => { winc = tokens.get(i+1).and_then(|t| t.parse().ok()).unwrap_or(0); i += 2; }
                "binc" => { binc = tokens.get(i+1).and_then(|t| t.parse().ok()).unwrap_or(0); i += 2; }
                "movestogo" => { movestogo = tokens.get(i+1).and_then(|t| t.parse().ok()).unwrap_or(30); i += 2; }
                "depth" => { depth = tokens.get(i+1).and_then(|t| t.parse().ok()).unwrap_or(64); i += 2; }
                "movetime" => { movetime = tokens.get(i+1).and_then(|t| t.parse().ok()); i += 2; }
                "infinite" => { infinite = true; i += 1; }
                _ => i += 1,
            }
        }

        // Calculate time limit
        let time_limit = if infinite { 0 } 
        else if let Some(mt) = movetime { mt as u128 } 
        else {            
            let (time_left, inc) = 
                if position.side == Color::WHITE { (wtime, winc) } 
                else { (btime, binc) };

            if let Some(t) = time_left {
                let mut alloc = t / movestogo;
                
                if alloc > 50 { alloc -= 50; } 
                
                (alloc + inc) as u128
            } 
            else { 0 }
        };

        let search = Arc::new(Search::new(tt, time_limit));

        let search_thread = search.clone();
        let mut position_thread = position.clone();
        let depth_thread = depth as u8;

        let handle = spawn(move || {
            position_thread.iterative_deepen(&search_thread, depth_thread, true);
        });

        (Some(handle), search)
    }

    let stdin = stdin();
    let mut main_search_thread: Option<JoinHandle<()>> = None;
    let mut position = Board::new(FEN_START);
    let tt = Arc::new(TranspositionTable::new(24));
    let mut search = Arc::new(Search::new(tt.clone(), 0));

    print_uci_id();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l.trim().to_string(),
            Err(_) => continue,
        };
        if line.is_empty() { continue; }

        let tokens: Vec<&str> = line.split_whitespace().collect();
        match tokens.get(0).map(|s| *s) {
            Some("quit") => {
                if let Some(handle) = main_search_thread.take() {
                    search.stop_flag.store(true, Ordering::Relaxed);
                    handle.join().unwrap();
                }
                break;
            },

            Some("uci") => print_uci_id(),

            Some("setoption") => {},

            Some("position") => {
                position = parse_position(&tokens[1..]);
                println!("{}", position);
            },

            Some("ucinewgame") => {
                position = Board::new(FEN_START);
            },

            Some("isready") => println!("readyok"),

            Some("go") => {
                if let Some(handle) = main_search_thread.take() {
                    search.stop_flag.store(true, Ordering::Relaxed);
                    handle.join().unwrap();
                }

                let (handle, new_search) = parse_go(&tokens[1..], &position, tt.clone());
                main_search_thread = handle;
                search = new_search;
            },

            Some("stop") => {
                if let Some(handle) = main_search_thread.take() {
                    search.stop_flag.store(true, Ordering::Relaxed);
                    handle.join().unwrap();
                }
            }
            
            _ => {}
        }
    }
}