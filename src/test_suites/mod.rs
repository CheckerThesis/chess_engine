use std::{fs::File, io::{self, BufRead, BufReader}, sync::Arc};

use crate::{board::Board, defs::PieceType, fens::FEN_START, movegen::{Move, MoveList, SQUARE_TO_STRING}, search::{self, Search}, transposition_table::TranspositionTable};

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

pub fn move_to_san(position: &Board, mv: Move) -> String {
    // Handle castling
    if mv.is_castling() {
        return if mv.to_square() > mv.from_square() {
            "O-O".to_string()
        } else {
            "O-O-O".to_string()
        };
    }

    let mut san = String::new();
    let piece = position.pieces[mv.from_square()];
    let piece_type = piece.piece_type();

    // Piece letter (not for pawns)
    if piece_type != PieceType::PAWN {
        let piece_char = match piece_type {
            PieceType::KNIGHT => 'N',
            PieceType::BISHOP => 'B',
            PieceType::ROOK => 'R',
            PieceType::QUEEN => 'Q',
            PieceType::KING => 'K',
            _ => '?',
        };
        san.push(piece_char);
    }

    // Check disambiguation - are there other pieces of same type that can reach same square?
    let mut move_list = MoveList::new();
    move_list.generate_all_moves(position);

    let mut needs_file = false;
    let mut needs_rank = false;

    let from_file = (mv.from_square() % 8) as u8;
    let from_rank = (mv.from_square() / 8) as u8;

    for scored in move_list.iter() {
        let m = scored.mv;
        if m.to_square() == mv.to_square() 
            && m.from_square() != mv.from_square()
            && position.pieces[m.from_square()].piece_type() == piece_type 
        {
            let other_file = (m.from_square() % 8) as u8;
            let other_rank = (m.from_square() / 8) as u8;

            if other_file == from_file {
                needs_rank = true;
            } else {
                needs_file = true;
            }
        }
    }

    // Pawn captures always show file
    if piece_type == PieceType::PAWN && mv.captured() != 0 {
        needs_file = true;
    }

    if needs_file {
        san.push((b'a' + from_file) as char);
    }
    if needs_rank {
        san.push((b'1' + from_rank) as char);
    }

    // Capture symbol
    if mv.captured() != 0 {
        san.push('x');
    }

    // Destination square
    let to_file = (mv.to_square() % 8) as u8;
    let to_rank = (mv.to_square() / 8) as u8;
    san.push((b'a' + to_file) as char);
    san.push((b'1' + to_rank) as char);

    // Promotion
    if mv.promoted() != 0 {
        san.push('=');
        let promo_char = match mv.promoted() {
            2 => 'N',
            3 => 'B',
            4 => 'R',
            5 => 'Q',
            _ => '?',
        };
        san.push(promo_char);
    }

    san
}

pub fn run_suites(path: &str) {
    let file = File::open(path).expect("Could not open EPD file");
    let lines = BufReader::new(file).lines();

    let mut solved = 0;
    let mut total = 0;
    let mut avoided_count = 0;

    let mut position = Board::new(FEN_START);
    let transposition_table = Arc::new(TranspositionTable::new(20));

    for line in lines {
        if let Ok(epd) = line {
            if !epd.contains(" bm ") { continue; }

            total += 1;

            let fen_end = epd.find(" am ")
                .or_else(|| epd.find(" bm "))
                .unwrap();
            let fen = &epd[..fen_end];

            position.parse_fen(fen);

            // Parse best moves (bm)
            let mut expected_moves: Vec<Move> = Vec::new();
            if let Some(bm_start) = epd.find(" bm ") {
                let bm_section = &epd[bm_start + 4..];
                let bm_end = bm_section.find(';').unwrap_or(bm_section.len());
                let best_moves_str = bm_section[..bm_end].trim();

                for san in best_moves_str.split_whitespace() {
                    if let Some(mv) = parse_san(&position, san) {
                        expected_moves.push(mv);
                    } else {
                        println!("Warning: Could not parse bm '{}' in test {}", san, total);
                    }
                }
            }

            // Parse avoid moves (am) - if present
            let mut avoid_moves: Vec<Move> = Vec::new();
            if let Some(am_start) = epd.find(" am ") {
                let am_section = &epd[am_start + 4..];
                let am_end = am_section.find(';').unwrap_or(am_section.len());
                let avoid_moves_str = am_section[..am_end].trim();

                for san in avoid_moves_str.split_whitespace() {
                    if let Some(mv) = parse_san(&position, san) {
                        avoid_moves.push(mv);
                    } else {
                        println!("Warning: Could not parse am '{}' in test {}", san, total);
                    }
                }
            }

            // Get best moves string for output
            let bm_start = epd.find(" bm ").unwrap();
            let bm_section = &epd[bm_start + 4..];
            let bm_end = bm_section.find(';').unwrap_or(bm_section.len());
            let best_moves_str = bm_section[..bm_end].trim();

            // Run search
            let best_move: Option<Move>;
            {
                let search = Arc::new(Search::new(transposition_table.clone(), 0));
                best_move = position.iterative_deepen(&search, 8, false);
            }

            if let Some(engine_move) = best_move {
                let engine_san = move_to_san(&position, engine_move);

                if expected_moves.contains(&engine_move) {
                    solved += 1;
                    // println!("Test {}: PASSED (Found {})", total, engine_san);
                } else if avoid_moves.contains(&engine_move) {
                    avoided_count += 1;
                    // println!("Test {}: FAILED - Avoided move! (Expected {}, Got {})", 
                    //          total, best_moves_str, engine_san);
                } else {
                    // println!("Test {}: FAILED (Expected {}, Got {})", 
                    //          total, best_moves_str, engine_san);
                }
            } else {
                // println!("Test {}: FAILED (Engine returned No Move)", total);
            }
        }
    }

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Final Score: {}/{} ({:.1}%)", solved, total, 100.0 * solved as f64 / total as f64);
    if avoided_count > 0 {
        println!("⚠️  Found avoided moves {} times", avoided_count);
    }
}