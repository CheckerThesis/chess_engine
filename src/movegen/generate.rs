use colored::Colorize;

use crate::{board::{Board, MAX_DEPTH, print_bitboard}, defs::{BLACK_KING_CASTLE, BLACK_QUEEN_CASTLE, Color, Piece, PieceType, RANKS_BOARD, Ranks, WHITE_KING_CASTLE, WHITE_QUEEN_CASTLE}, movegen::{MOVE_FLAG_CASTLE, MOVE_FLAG_EN_PASSANT, MOVE_FLAG_NONE, MOVE_FLAG_PAWN_START, Move, MoveList, ScoredMove, bitboards::{BLACK_PAWN_ATTACKS, KING_RAYS, KNIGHT_RAYS, WHITE_PAWN_ATTACKS}, magic::{get_bishop_attacks, get_rook_attacks}, mvv_lva::{MVV_LVA_SCORES, PIECE_VALUE}}};

/*
TT: 30000
Good capture: 20000
Killer1: 10000
Killer2: 9000
History: < 8999
Countermove: 8000
Quiet: 0
Bad capture: -1000
*/

impl MoveList {
    fn add_quiet_move(&mut self, position: &Board, mv: Move) {
        let mut score: i16 = 0;

        // Killer moves
        let ply = position.ply as usize;
        if ply < MAX_DEPTH {
            if let Some(killer) = position.killers[ply][0] {
                if killer.0 == mv.0 { score = 10000; }
            }
            if score == 0 {
                if let Some(killer) = position.killers[ply][1] {
                    if killer.0 == mv.0 { score = 9000; }
                }
            }
        }

        // Counter move
        // if score == 0 {
        //     if let Some(previous_mv) = position.get_previous_move() {
        //         let previous_to = previous_mv.to_square();
        //         let opponent_side = position.side.opposite().index();
        //         let piece_index = position.pieces[previous_to].piece_type().index();
        //         if let Some(countermove) = position.countermoves[opponent_side][piece_index][previous_to] {
        //             if countermove.0 == mv.0 { score = 8000; }
        //         }
        //     }
        // }
        
        // History heuristic
        if score == 0 {
            let piece_index = position.pieces[mv.from_square()].index();
            score = position.history_heuristic[piece_index][mv.to_square()];
        }

        self.add(ScoredMove::new(mv, score));
    }

    fn add_capture_move(&mut self, position: &Board, attacker: PieceType, mv: Move) {
        let see_boost = 
            if 
                PIECE_VALUE[Piece(mv.captured() as u8).piece_type().index()] >= PIECE_VALUE[attacker.index()] || 
                position.static_exchange_evaluation(
                    mv.from_square(), 
                    mv.to_square(), 
                    Piece(mv.captured() as u8).piece_type(), 
                    attacker
                ) >= 0
            { 20000 } 
            else { -1000 };

        let mvv_lva_boost = MVV_LVA_SCORES[Piece(mv.captured() as u8).piece_type().index()][attacker.index()];
        self.add(ScoredMove::new(mv, see_boost + mvv_lva_boost));
    }

    fn add_quiet_pawn_move(&mut self, position: &Board, from: usize, to: usize) {
        let (promotion_rank, queen, rook, bishop, knight) = if position.side == Color::WHITE {
            (
                Ranks::Rank7, 
                Piece::WHITE_QUEEN, 
                Piece::WHITE_ROOK, 
                Piece::WHITE_BISHOP, 
                Piece::WHITE_KNIGHT
            )
        } else {
            (
                Ranks::Rank2, 
                Piece::BLACK_QUEEN, 
                Piece::BLACK_ROOK, 
                Piece::BLACK_BISHOP, 
                Piece::BLACK_KNIGHT
            )
        };

        if RANKS_BOARD[from] == promotion_rank as usize {
            self.add_quiet_move(position, Move::new(
                from, 
                to, 
                0, 
                MOVE_FLAG_NONE,
                queen.index()
            ));
            self.add_quiet_move(position, Move::new(
                from, 
                to, 
                0, 
                MOVE_FLAG_NONE,
                rook.index()
            ));
            self.add_quiet_move(position, Move::new(
                from, 
                to, 
                0, 
                MOVE_FLAG_NONE,
                bishop.index()
            ));
            self.add_quiet_move(position, Move::new(
                from, 
                to, 
                0, 
                MOVE_FLAG_NONE,
                knight.index()
            ));
        } else {
            self.add_quiet_move(position, Move::new(
                from, 
                to, 
                0, 
                MOVE_FLAG_NONE,
                0
            ));
        }
    }

    fn add_capture_pawn_move(&mut self, position: &Board, from: usize, to: usize, captured: usize) {
        let (promotion_rank, queen, rook, bishop, knight) = if position.side == Color::WHITE {
            (
                Ranks::Rank7, 
                Piece::WHITE_QUEEN, 
                Piece::WHITE_ROOK, 
                Piece::WHITE_BISHOP, 
                Piece::WHITE_KNIGHT
            )
        } else {
            (
                Ranks::Rank2, 
                Piece::BLACK_QUEEN, 
                Piece::BLACK_ROOK, 
                Piece::BLACK_BISHOP, 
                Piece::BLACK_KNIGHT
            )
        };

        if RANKS_BOARD[from] == promotion_rank as usize {
            self.add_capture_move(position, PieceType::PAWN, Move::new(
                from, 
                to, 
                captured, 
                MOVE_FLAG_NONE,
                queen.index()
            ));
            self.add_capture_move(position, PieceType::PAWN, Move::new(
                from, 
                to, 
                captured, 
                MOVE_FLAG_NONE,
                rook.index()
            ));
            self.add_capture_move(position, PieceType::PAWN, Move::new(
                from, 
                to, 
                captured, 
                MOVE_FLAG_NONE,
                bishop.index()
            ));
            self.add_capture_move(position, PieceType::PAWN, Move::new(
                from, 
                to, 
                captured, 
                MOVE_FLAG_NONE,
                knight.index()
            ));
        } else {
            self.add_capture_move(position, PieceType::PAWN, Move::new(
                from, 
                to, 
                captured, 
                MOVE_FLAG_NONE,
                0
            ));
        }
    }

    fn add_enpassant_move(&mut self, position: &Board, from: usize, to: usize) {        
        self.add_quiet_move(position, Move::new(
            from, 
            to, 
            0, 
            MOVE_FLAG_EN_PASSANT, 
            0
        ));
    }

    #[inline(always)]
    fn serialize_moves(&mut self, position: &Board, from_square: usize, attacks: u64, captures_only: bool) {
        let side = position.side;
        let their_occupancy = position.occupancies[side.opposite().index()];
        let our_occupancy = position.occupancies[side.index()];
        
        let valid_moves = attacks & !our_occupancy;
        let mut captures = valid_moves & their_occupancy;
        let mut quiets = valid_moves & !their_occupancy;

        let attacker = position.pieces[from_square].piece_type();

        while captures != 0 {
            let to_square = captures.trailing_zeros() as usize;
            let captured_piece = position.pieces[to_square];
    
            self.add_capture_move(position, attacker, Move::new(
                from_square, to_square, captured_piece.index(), MOVE_FLAG_NONE, 0
            ));
            
            captures &= captures - 1;
        }
        if !captures_only {
            while quiets != 0 {
                let to_square = quiets.trailing_zeros() as usize;
                
                self.add_quiet_move(position, Move::new(
                    from_square, to_square, 0, MOVE_FLAG_NONE, 0
                ));
                
                quiets &= quiets - 1;
            }
        }
    }

    fn generate_pawn_moves(&mut self, position: &Board, captures_only: bool) {
        #[cfg(debug_assertions)] fn gen_rank_bb_mask() {
            let mut rank_bb_mask: [u64; 9] = [0; 9];
            const RANK_1: u64 = 0x00000000000000FF;

            for i in 1..9 {
                rank_bb_mask[i] = RANK_1 << ((i - 1) * 8);
            }
        }
        // Inited from above
        const RANK_BB_MASK: [u64; 9] = [
            0,
            255,
            65280,
            16711680,
            4278190080,
            1095216660480,
            280375465082880,
            71776119061217280,
            18374686479671623680
        ];

        let occupied_bb = [position.occupancies[Color::WHITE.index()], position.occupancies[Color::BLACK.index()]];
        let empty_squares: u64 = !(occupied_bb[Color::WHITE.index()] | occupied_bb[Color::BLACK.index()]);
        let color = position.side;
        let (
            our_pawns,
            their_pieces,
            push_offset,
            double_push_rank,
            pawn_attacks,
            en_passant_attackers_table
        ) = if color == Color::WHITE {
            (
                position.bitboards[Piece::WHITE_PAWN.index()],
                occupied_bb[Color::BLACK.index()],
                8,
                RANK_BB_MASK[3],
                WHITE_PAWN_ATTACKS,
                BLACK_PAWN_ATTACKS
            )
        } else {
            (
                position.bitboards[Piece::BLACK_PAWN.index()],
                occupied_bb[Color::WHITE.index()],
                -8,
                RANK_BB_MASK[6],
                BLACK_PAWN_ATTACKS,
                WHITE_PAWN_ATTACKS
            )
        };


        if !captures_only {            
            let mut single_pushes = if color == Color::WHITE {
                our_pawns.wrapping_shl(push_offset as u32) & empty_squares
            } else {
                our_pawns.wrapping_shr((-push_offset) as u32) & empty_squares
            };

            let mut double_pushes = if color == Color::WHITE {
                (single_pushes & double_push_rank).wrapping_shl(push_offset as u32) & empty_squares
            } else {
                (single_pushes & double_push_rank).wrapping_shr((-push_offset) as u32) & empty_squares
            };

            while single_pushes != 0 {
                let to_square = single_pushes.trailing_zeros() as usize;
                let from_square = (to_square as i32 - push_offset) as usize;
                self.add_quiet_pawn_move(&position, from_square, to_square);

                single_pushes &= single_pushes - 1;
            }

            while double_pushes != 0 {
                let to_square = double_pushes.trailing_zeros() as usize;
                let from_square = (to_square as i32 - (push_offset * 2)) as usize;
                self.add_quiet_move(&position, Move::new(
                    from_square, 
                    to_square, 
                    0, 
                    MOVE_FLAG_PAWN_START, 
                    0
                ));

                double_pushes &= double_pushes - 1;
            }
        }

        let mut pawns = our_pawns;
        while pawns != 0 {
            let from_square = pawns.trailing_zeros() as usize;
            let mut valid_captures = pawn_attacks[from_square] & their_pieces;

            while valid_captures != 0 {
                let to_square = valid_captures.trailing_zeros() as usize;
                let captured_piece = position.pieces[to_square];

                self.add_capture_pawn_move(&position, from_square, to_square, captured_piece.index());

                valid_captures &= valid_captures - 1;
            }
            pawns &= pawns - 1;
        }

        if let Some(en_passant_square) = position.en_passant {
            let potential_attackers = en_passant_attackers_table[en_passant_square as usize];
            let mut en_passant_attackers = potential_attackers & our_pawns;

            while en_passant_attackers != 0 {
                let from_square = en_passant_attackers.trailing_zeros() as usize;
                self.add_enpassant_move(position, from_square, en_passant_square as usize);
        
                en_passant_attackers &= en_passant_attackers - 1;
            }
        }
    }

    fn generate_knight_moves(&mut self, position: &Board, captures_only: bool) {
        let side = position.side;
        let our_pieces = position.occupancies[side.index()];
        let their_pieces = position.occupancies[side.opposite().index()];

        let mut knights = position.bitboards[PieceType::KNIGHT.bb_index(side)];
        while knights != 0 {
            let from_square_index = knights.trailing_zeros() as usize;
            self.serialize_moves(position, from_square_index, KNIGHT_RAYS[from_square_index], captures_only);
            knights &= knights - 1;
        }
    }

    fn generate_sliding_moves(&mut self, position: &Board, captures_only: bool) {
        let side = position.side;
        let our_occupancy = position.occupancies[side.index()];
        let their_occupancy = position.occupancies[side.opposite().index()];

        let mut bishops = position.bitboards[PieceType::BISHOP.bb_index(side)];
        while bishops != 0 {
            let from_square_index = bishops.trailing_zeros() as usize;
            let movement_bb = get_bishop_attacks(from_square_index, our_occupancy | their_occupancy) & !our_occupancy;
            self.serialize_moves(position, from_square_index, movement_bb, captures_only);
            bishops &= bishops - 1;
        }

        let mut rooks = position.bitboards[PieceType::ROOK.bb_index(side)];
        while rooks != 0 {
            let from_square_index = rooks.trailing_zeros() as usize;
            let movement_bb = get_rook_attacks(from_square_index, our_occupancy | their_occupancy) & !our_occupancy;
            self.serialize_moves(position, from_square_index, movement_bb, captures_only);
            rooks &= rooks - 1;
        }

        let mut queens = position.bitboards[PieceType::QUEEN.bb_index(side)];
        while queens != 0 {
            let from_square_index = queens.trailing_zeros() as usize;
            let movement_bb = 
                (get_bishop_attacks(from_square_index, our_occupancy | their_occupancy) & !our_occupancy) |
                (get_rook_attacks(from_square_index, our_occupancy | their_occupancy) & !our_occupancy);
            self.serialize_moves(position, from_square_index, movement_bb, captures_only);
            queens &= queens - 1;
        }
    }

    fn generate_king_moves(&mut self, position: &Board, captures_only: bool) {
        let side = position.side;
        let our_pieces = position.occupancies[side.index()];
        let their_pieces = position.occupancies[side.opposite().index()];

        let mut kings = position.bitboards[PieceType::KING.bb_index(side)];
        while kings != 0 {
            let from_sq = kings.trailing_zeros() as usize;
            let mut moves = KING_RAYS[from_sq] & !our_pieces;

            while moves != 0 {
                let to_sq = moves.trailing_zeros() as usize;

                let to_bb = 1u64 << to_sq;

                if (to_bb & their_pieces) != 0 {
                    let captured_piece = position.pieces[to_sq];
                    self.add_capture_move(position, PieceType::KING, Move::new(
                        from_sq, to_sq, captured_piece.index(), MOVE_FLAG_NONE, 0
                    ));
                } else if !captures_only {
                    self.add_quiet_move(position, Move::new(
                        from_sq, to_sq, 0, MOVE_FLAG_NONE, 0
                    ));
                }

                moves &= moves - 1;
            }
            kings &= kings - 1;
        }
    }

    fn generate_castle_moves(&mut self, position: &Board) {
        let side = position.side;
        let occupancies = position.occupancies[Color::WHITE.index()] | position.occupancies[Color::BLACK.index()];

        match side {
            Color::WHITE => {
                // King Side (e1 -> g1)
                if (position.castle_permission & WHITE_KING_CASTLE) != 0 {
                    if (occupancies & ((1 << 5) | (1 << 6))) == 0 {
                        self.add_quiet_move(position, Move::new(
                            4,
                            6,
                            0,
                            MOVE_FLAG_CASTLE,
                            0
                        ));
                    }
                }

                // Queen Side (e1 -> c1)
                if (position.castle_permission & WHITE_QUEEN_CASTLE as u8) != 0 {
                    if (occupancies & ((1 << 1) | (1 << 2) | (1 << 3))) == 0 {
                        self.add_quiet_move(position, Move::new(
                            4,
                            2,
                            0,
                            MOVE_FLAG_CASTLE,
                            0
                        ));
                    }
                }
            },
            Color::BLACK => {
                // King Side (e8 -> g8)
                if (position.castle_permission & BLACK_KING_CASTLE as u8) != 0 {
                    if (occupancies & ((1 << 61) | (1 << 62))) == 0 {
                        self.add_quiet_move(position, Move::new(
                            60,
                            62,
                            0,
                            MOVE_FLAG_CASTLE,
                            0
                        ));
                    }
                }

                // Queen Side (e8 -> c8)
                if (position.castle_permission & BLACK_QUEEN_CASTLE as u8) != 0 {
                    if (occupancies & ((1 << 57) | (1 << 58) | (1 << 59))) == 0 {
                        self.add_quiet_move(position, Move::new(
                            60,
                            58,
                            0,
                            MOVE_FLAG_CASTLE,
                            0
                        ));
                    }
                }
            },
            _ => {}
        }
    }

    pub fn generate_all_moves(&mut self, position: &Board) {
        self.count = 0;
        self.generate_pawn_moves(position, false);
        self.generate_knight_moves(position, false);
        self.generate_sliding_moves(position, false);
        self.generate_king_moves(position, false);
        self.generate_castle_moves(position);
    }

    pub fn generate_all_captures(&mut self, position: &Board) {
        self.count = 0;
        self.generate_pawn_moves(position, true);
        self.generate_knight_moves(position, true);
        self.generate_sliding_moves(position, true);
        self.generate_king_moves(position, true);
    }
}
