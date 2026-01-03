use std::{fs::File, io::{self, BufRead, BufReader}, sync::Arc};

use crate::{board::Board, defs::PieceType, fens::FEN_START, movegen::{Move, MoveList, SQUARE_TO_STRING}, search::{self, Search}, transposition_table::TranspositionTable};

pub fn run_sts(path: &str) {
    pub fn parse_san(position: &Board, san: &str) -> Option<Move> {
        // 1. Generate all legal moves using your MoveList
        let mut move_list = MoveList::new();
        move_list.generate_all_moves(position);

        // 2. Clean the SAN string (remove +, #, x)
        let clean_san = san.replace("+", "").replace("#", "").replace("x", "");

        // 3. Handle Castling explicitly
        // We assume the generator flags castling moves correctly.
        // White: e1->g1 (O-O), e1->c1 (O-O-O)
        // Black: e8->g8 (O-O), e8->c8 (O-O-O)
        if clean_san == "O-O" {
            return move_list.iter().find_map(|scored| {
                let m = scored.mv; // Assuming ScoredMove has a public .mv field
                if m.is_castling() && m.to_square() > m.from_square() { Some(m) } else { None }
            });
        }
        if clean_san == "O-O-O" {
            return move_list.iter().find_map(|scored| {
                let m = scored.mv; 
                if m.is_castling() && m.to_square() < m.from_square() { Some(m) } else { None }
            });
        }

        // 4. Handle Promotion (e.g., "a8=Q")
        let (target_str, promo_char) = if clean_san.contains('=') {
            let parts: Vec<&str> = clean_san.split('=').collect();
            (parts[0], parts[1].chars().next())
        } else {
            (clean_san.as_str(), None)
        };

        // 5. Parse Destination Square (last 2 chars, e.g., "e4")
        let target_len = target_str.len();
        if target_len < 2 { return None; }
        
        let dest_sq_str = &target_str[target_len - 2..];
        let dest_sq = SQUARE_TO_STRING.iter().position(|&s| s == dest_sq_str)?;

        // 6. Parse Moving Piece Type and Disambiguation
        // If starts with uppercase, it's a piece (N, B, R, Q, K). Otherwise, it's a pawn.
        let first_char = target_str.chars().next().unwrap();
        let (target_piece_type, disambiguation) = if first_char.is_uppercase() {
            let pt = match first_char {
                'N' => PieceType::KNIGHT,
                'B' => PieceType::BISHOP,
                'R' => PieceType::ROOK,
                'Q' => PieceType::QUEEN,
                'K' => PieceType::KING,
                _ => return None,
            };
            // Everything between first char and last 2 chars is disambiguation
            (pt, &target_str[1..target_len - 2])
        } else {
            (PieceType::PAWN, &target_str[0..target_len - 2])
        };

        // 7. Find the matching move
        for scored_mv in move_list.iter() {
            let mv = scored_mv.mv;

            // A. Check Destination
            if mv.to_square() != dest_sq { continue; }

            // B. Check Piece Type
            let moving_piece = position.pieces[mv.from_square()];
            if moving_piece.piece_type() != target_piece_type { continue; }

            // C. Check Promotion (if applicable)
            if let Some(p_char) = promo_char {
                // Note: Adjust these indices if your engine uses different values for promotion
                let expected_promo_idx = match p_char {
                    'n' | 'N' => PieceType::KNIGHT.index(), // 2
                    'b' | 'B' => PieceType::BISHOP.index(), // 3
                    'r' | 'R' => PieceType::ROOK.index(),   // 4
                    'q' | 'Q' => PieceType::QUEEN.index(),  // 5
                    _ => 0,
                };

                // If the move isn't a promotion, or the promoted piece doesn't match
                // We use mv.promoted() assuming it returns the PieceType index (e.g. 5 for Queen)
                if mv.promoted() == 0 || mv.promoted() != expected_promo_idx { 
                    continue; 
                }
            } else {
                // If the SAN string has no '=', the move shouldn't be a promotion
                // (Unless it's implied, but standard SAN is usually explicit)
                if mv.promoted() != 0 { continue; }
            }

            // D. Check Disambiguation (e.g. "Nbd7" -> 'b', "R1e2" -> '1')
            if !disambiguation.is_empty() {
                let from_file = (mv.from_square() % 8) as u8;
                let from_rank = (mv.from_square() / 8) as u8;
                let from_file_char = (b'a' + from_file) as char;
                let from_rank_char = (b'1' + from_rank) as char;

                let mut matches = true;
                for ch in disambiguation.chars() {
                    if ch >= 'a' && ch <= 'h' {
                        if ch != from_file_char { matches = false; }
                    } else if ch >= '1' && ch <= '8' {
                        if ch != from_rank_char { matches = false; }
                    }
                }
                if !matches { continue; }
            }

            // If we passed all checks, this is the move
            return Some(mv);
        }

        None
    }

    let file = File::open(path).expect("Could not open EPD file");
    let lines = BufReader::new(file).lines();

    let mut solved = 0;
    let mut total = 0;

    let mut position = Board::new(FEN_START);
    let transposition_table = Arc::new(TranspositionTable::new(20));

    for line in lines {
        if let Ok(epd) = line {
            total += 1;

            let parts: Vec<&str> = epd.split(';').collect();
            let fen_part = parts[0];
            let fen_best_move: Vec<&str> = fen_part.split(" bm ").collect();
            let fen = fen_best_move[0].trim();
            let best_moves_str = fen_best_move[1].trim();

            position.parse_fen(fen);

            let mut expected_moves: Vec<Move> = Vec::new();     
            for san in best_moves_str.split_whitespace() {
                // calls your local helper function
                if let Some(mv) = parse_san(&position, san) {
                    expected_moves.push(mv);
                } else {
                    println!("Warning: Could not parse SAN move '{}' in position", san);
                }
            }

            let best_move: Option<Move>;
            {
                let search = Arc::new(Search::new(transposition_table.clone(), 0));
                best_move = position.iterative_deepen(&search, 8, false);
            }

            if let Some(engine_move) = best_move {
                if expected_moves.contains(&engine_move) {
                    solved += 1;
                    println!("Test {}: PASSED (Found {})", total, engine_move);
                } else {
                    // Start formatting a failure message
                    println!("Test {}: FAILED (Expected {:?}, Got {})", total, engine_move.to_string(), best_moves_str);
                    // println!("  FEN:      {}", fen);
                    // // Show what the test suite wanted (Strings)
                    // println!("  Expected: {}", best_moves_str); 
                    // // Show what your engine found (String representation of your move)
                    // println!("  Got:      {}", engine_move);
                }
            } else {
                println!("Test {}: FAILED (Engine returned No Move)", total);
            }
        }
    }
    println!("Final Score: {}/{}", solved, total);
}