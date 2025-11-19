use crate::{board::Board, defs::{Color, Piece, PieceType, RANKS_BOARD, Ranks}, movegen::{BLACK_PAWN_ATTACKS, DEMAND_DIAGONAL_RAYS, DOWN_RAYS, LEFT_RAYS, MoveList, RANK_BB_MASK, RIGHT_RAYS, SUPPLY_DIAGONAL_RAYS, UP_RAYS, WHITE_PAWN_ATTACKS}};

impl MoveList {
    pub fn move_builder(
        from: usize, 
        to: usize, 
        capture: usize, 
        flags: usize, 
        promote: usize
    ) -> u32 { 
        from as u32 | 
        ((to as u32) << 6) | 
        ((capture as u32) << 12) | 
        ((flags as u32) << 17) | 
        ((promote as u32) << 20)
    }

    fn set_score(mv: u32, score: u64) -> u64 { return (mv as u64) | (score << 47) } 

    fn add_quiet_move(&mut self, position: &Board, mv: u32) {
        // TODO set score killer move
        self.add(MoveList::set_score(mv, 0));
    }

    fn add_capture_move(&mut self, position: &Board, mv: u32) {
        // TODO set score MVV_LVA
        self.add(MoveList::set_score(mv, 10000));
    }

    fn add_quiet_pawn_move(&mut self, position: &Board, from: usize, to: usize) {
        let (promotion_rank, queen, rook, bishop, knight) = if position.side == Color::White {
            (
                Ranks::Rank7, 
                Piece { piece_type: PieceType::Queen, color: Color::White }, 
                Piece { piece_type: PieceType::Rook, color: Color::White }, 
                Piece { piece_type: PieceType::Bishop, color: Color::White }, 
                Piece { piece_type: PieceType::Knight, color: Color::White }
            )
        } else {
            (
                Ranks::Rank2, 
                Piece { piece_type: PieceType::Queen, color: Color::Black }, 
                Piece { piece_type: PieceType::Rook, color: Color::Black }, 
                Piece { piece_type: PieceType::Bishop, color: Color::Black }, 
                Piece { piece_type: PieceType::Knight, color: Color::Black }
            )
        };

        if RANKS_BOARD[from] == promotion_rank as usize {
            self.add_quiet_move(position, MoveList::move_builder(
                from, 
                to, 
                0, 
                1, /*Flags*/
                Piece::bb_index(&queen)
            ));
            self.add_quiet_move(position, MoveList::move_builder(
                from, 
                to, 
                0, 
                1, /*Flags*/
                Piece::bb_index(&rook)
            ));
            self.add_quiet_move(position, MoveList::move_builder(
                from, 
                to, 
                0, 
                1, /*Flags*/
                Piece::bb_index(&bishop)
            ));
            self.add_quiet_move(position, MoveList::move_builder(
                from, 
                to, 
                0, 
                1, /*Flags*/
                Piece::bb_index(&knight)
            ));
        } else {
            self.add_quiet_move(position, MoveList::move_builder(
                from, 
                to, 
                0, 
                1, /*Flags*/
                0
            ));
        }
    }

    fn add_capture_pawn_move(&mut self, position: &Board, from: usize, to: usize, captured: usize) {
        let (promotion_rank, queen, rook, bishop, knight) = if position.side == Color::White {
            (
                Ranks::Rank7, 
                Piece { piece_type: PieceType::Queen, color: Color::White }, 
                Piece { piece_type: PieceType::Rook, color: Color::White }, 
                Piece { piece_type: PieceType::Bishop, color: Color::White }, 
                Piece { piece_type: PieceType::Knight, color: Color::White }
            )
        } else {
            (
                Ranks::Rank2, 
                Piece { piece_type: PieceType::Queen, color: Color::Black }, 
                Piece { piece_type: PieceType::Rook, color: Color::Black }, 
                Piece { piece_type: PieceType::Bishop, color: Color::Black }, 
                Piece { piece_type: PieceType::Knight, color: Color::Black }
            )
        };

        if RANKS_BOARD[from] == promotion_rank as usize {
            self.add_capture_move(position, MoveList::move_builder(
                from, 
                to, 
                captured, 
                1, /*Flags*/
                Piece::bb_index(&queen)
            ));
            self.add_capture_move(position, MoveList::move_builder(
                from, 
                to, 
                captured, 
                1, /*Flags*/
                Piece::bb_index(&rook)
            ));
            self.add_capture_move(position, MoveList::move_builder(
                from, 
                to, 
                captured, 
                1, /*Flags*/
                Piece::bb_index(&bishop)
            ));
            self.add_capture_move(position, MoveList::move_builder(
                from, 
                to, 
                captured, 
                1, /*Flags*/
                Piece::bb_index(&knight)
            ));
        } else {
            self.add_capture_move(position, MoveList::move_builder(
                from, 
                to, 
                captured, 
                1, /*Flags*/
                0
            ));
        }
    }

    pub fn generate_pawn_moves(&mut self, position: &Board) {
        let occupied_bb = [position.occupancies(Color::White), position.occupancies(Color::Black)];
        let empty_squares: u64 = !(occupied_bb[Color::White as usize] | occupied_bb[Color::Black as usize]);
        let color = position.side;
        let (
            our_pawns,
            their_pieces,
            push_offset,
            double_push_rank,
            pawn_attacks,
            en_passant_attackers_table
        ) = if color == Color::White {
            (
                position.bitboards[PieceType::Pawn.bb_index(Color::White)],
                occupied_bb[Color::Black as usize],
                8,
                RANK_BB_MASK[3],
                &WHITE_PAWN_ATTACKS,
                &BLACK_PAWN_ATTACKS
            )
        } else {
            (
                position.bitboards[PieceType::Pawn.bb_index(Color::Black)],
                occupied_bb[Color::White as usize],
                -8,
                RANK_BB_MASK[6],
                &BLACK_PAWN_ATTACKS,
                &WHITE_PAWN_ATTACKS
            )
        };

        let mut single_pushes = if color == Color::White {
            our_pawns.wrapping_shl(push_offset as u32) & empty_squares
        } else {
            our_pawns.wrapping_shr((-push_offset) as u32) & empty_squares
        };

        let mut double_pushes = if color == Color::White {
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
            self.add_quiet_move(&position, MoveList::move_builder(
                from_square, 
                to_square, 
                0, 
                0 /*Pawn start*/, 
                0
            ));

            double_pushes &= double_pushes - 1;
        }

        let mut pawns = our_pawns;
        while pawns != 0 {
            let from_square = pawns.trailing_zeros() as usize;
            let mut valid_captures = pawn_attacks[from_square] & their_pieces;

            while valid_captures != 0 {
                let to_square = valid_captures.trailing_zeros() as usize;
                let captured_piece = position.pieces[to_square];

                if let Some(piece) = captured_piece {
                    self.add_capture_pawn_move(&position, from_square, to_square, piece.bb_index());
                }

                valid_captures &= valid_captures - 1;
            }
            pawns &= pawns - 1;
        }

        if let Some(en_passant_square) = position.en_passant {
            let potential_attackers = en_passant_attackers_table[en_passant_square];
            let mut en_passant_attackers = potential_attackers & our_pawns;

            while en_passant_attackers != 0 {
                let from_square = en_passant_attackers.trailing_zeros() as usize;
                if let Some(piece) = position.pieces[from_square] {
                    self.add_capture_pawn_move(&position, from_square, en_passant_square, piece.bb_index());
                }

                en_passant_attackers &= en_passant_attackers - 1;
            }
        }
    }

    pub fn generate_sliding_moves(&mut self, position: &Board) {
        fn get_up_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            let ray = UP_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            if blockers == 0 { return ray }
            else {
                let first_blocker_square = blockers.trailing_zeros();
                // let mask_to_blocker = (1 << (first_blocker_square + 1)) - 1;
                let mask_to_blocker = if first_blocker_square >= 63 {
                    u64::MAX  // All bits set - no upper limit
                } else {
                    (1u64 << (first_blocker_square + 1)) - 1
                };
                let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
                return attack_mask & !our_occupancy;
            }
        }
        fn get_down_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            let ray = DOWN_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            if blockers == 0 { return ray; }
            else {
                let first_blocker_square = 63 - blockers.leading_zeros();
                let mask_to_blocker = u64::MAX << first_blocker_square;
                let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
                return attack_mask & !our_occupancy;
            }
        }
        fn get_left_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            let ray = LEFT_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            if blockers == 0 { return ray }
            else {
                let first_blocker_square = 63 - blockers.leading_zeros();
                let mask_to_blocker = RIGHT_RAYS[first_blocker_square as usize] & ray;
                let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
                return attack_mask & !our_occupancy;
            }
        }
        fn get_right_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            let ray = RIGHT_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            if blockers == 0 { return ray; }
            else {
                let first_blocker_square = blockers.trailing_zeros();
                let mask_to_blocker = LEFT_RAYS[first_blocker_square as usize] & ray;
                let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
                return attack_mask & !our_occupancy;
            }
        }

        fn get_supply_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            fn get_supply_positive_ray(square: usize) -> u64 {
                let ray = SUPPLY_DIAGONAL_RAYS[square];
                if square >= 63 {
                    0  // No positive ray from squares 63 and above
                } else {
                    ray & (u64::MAX.wrapping_shl((square + 1) as u32))
                }
            }
            fn get_supply_negative_ray(square: usize) -> u64 {
                let ray = SUPPLY_DIAGONAL_RAYS[square];
                ray & ((1 << square) - 1)
            }
            
            let ray = SUPPLY_DIAGONAL_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            let positive_ray = get_supply_positive_ray(square);
            let negative_ray = get_supply_negative_ray(square);

            // Up right (positive)
            let positive_blockers = blockers & positive_ray;
            let mut positive_attacks = positive_ray;
            if positive_blockers != 0 {
                let first_blocker_square = positive_blockers.trailing_zeros();
                let mask_to_blocker = positive_ray & get_supply_negative_ray(first_blocker_square as usize);
                positive_attacks = (positive_ray & mask_to_blocker) | (1 << first_blocker_square);
            }

            // Down left (negative)
            let negative_blockers = blockers & negative_ray;
            let mut negative_attacks = negative_ray;
            if negative_blockers != 0 {
                let first_blocker_square = 63 - negative_blockers.leading_zeros();
                let mask_to_blocker = negative_ray & get_supply_positive_ray(first_blocker_square as usize);
                negative_attacks = (negative_ray & mask_to_blocker) | (1 << first_blocker_square);
            }

            return (positive_attacks | negative_attacks) & !our_occupancy;
        }
        fn get_demand_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            fn get_demand_positive_ray(square: usize) -> u64 {
                let ray = DEMAND_DIAGONAL_RAYS[square];
                ray & u64::MAX.checked_shl((square + 1) as u32).unwrap_or(0)
            }
            fn get_demand_negative_ray(square: usize) -> u64 {
                let ray = DEMAND_DIAGONAL_RAYS[square];
                ray & ((1 << square) - 1)
            }

            let ray = DEMAND_DIAGONAL_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            let positive_ray = get_demand_positive_ray(square);
            let negative_ray = get_demand_negative_ray(square);
            
            // Up left (positive)
            let positive_blockers = blockers & positive_ray;
            let mut positive_attacks = positive_ray;
            if positive_blockers != 0 {
                let first_blocker_square = positive_blockers.trailing_zeros();
                let mask_to_blocker = positive_ray & get_demand_negative_ray(first_blocker_square as usize);
                positive_attacks = (positive_ray & mask_to_blocker) | (1 << first_blocker_square);
            }

            // Down right (negative)
            let negative_blockers = blockers & negative_ray;
            let mut negative_attacks = negative_ray;
            if negative_blockers != 0 {
                let first_blocker_square = 63 - negative_blockers.leading_zeros();
                let mask_to_blocker = negative_ray & get_demand_positive_ray(first_blocker_square as usize);
                negative_attacks = (negative_ray & mask_to_blocker) | (1 << first_blocker_square);
            }

            return (positive_attacks | negative_attacks) & !our_occupancy;
        }

        let side = position.side;
        let our_occupancy = position.occupancies(side);
        let their_occupancy = position.occupancies(side.opposite());
        let all_occupancy = our_occupancy | their_occupancy;

        let mut bishops = position.bitboards[PieceType::Bishop.bb_index(side)];
        while bishops != 0 {
            let from_square_index = bishops.trailing_zeros() as usize;
            let from_square = 1u64 << from_square_index;
            let mut movement_bb = 
                get_supply_moves(from_square_index, our_occupancy, their_occupancy) |
                get_demand_moves(from_square_index, our_occupancy, their_occupancy);
            while movement_bb != 0 {
                let to_square_index = movement_bb.trailing_zeros() as usize;
                let to_square = 1 << to_square_index;

                if to_square & their_occupancy == 0 {
                    self.add_quiet_move(position, MoveList::move_builder(
                        from_square_index, 
                        to_square_index, 
                        0, 
                        0, 
                        0
                    ));
                } else {
                    let captured_piece = position.pieces[to_square_index];
                    if let Some(piece) = captured_piece {
                        self.add_capture_move(position, MoveList::move_builder(
                            from_square_index, 
                            to_square_index, 
                            piece.bb_index(), 
                            0, 
                            0
                        ));
                    }
                }
                movement_bb &= movement_bb - 1;
            }
            bishops &= bishops - 1;
        }

        let mut rooks = position.bitboards[PieceType::Rook.bb_index(side)];
        while rooks != 0 {
            let from_square_index = rooks.trailing_zeros() as usize;
            let from_square = 1u64 << from_square_index;
            let mut movement_bb = 
                get_up_moves(from_square_index, our_occupancy, their_occupancy) |
                get_down_moves(from_square_index, our_occupancy, their_occupancy) |
                get_left_moves(from_square_index, our_occupancy, their_occupancy) |
                get_right_moves(from_square_index, our_occupancy, their_occupancy);
            while movement_bb != 0 {
                let to_square_index = movement_bb.trailing_zeros() as usize;
                let to_square = 1 << to_square_index;

                if to_square & their_occupancy == 0 {
                    self.add_quiet_move(position, MoveList::move_builder(
                        from_square_index, 
                        to_square_index, 
                        0, 
                        0, 
                        0
                    ));
                } else {
                    let captured_piece = position.pieces[to_square_index];
                    if let Some(piece) = captured_piece {
                        self.add_capture_move(position, MoveList::move_builder(
                            from_square_index, 
                            to_square_index, 
                            piece.bb_index(), 
                            0, 
                            0
                        ));
                    }
                }
                movement_bb &= movement_bb - 1;
            }
            rooks &= rooks - 1;
        }

        let mut queens = position.bitboards[PieceType::Queen.bb_index(side)];
        while queens != 0 {
            let from_square_index = queens.trailing_zeros() as usize;
            let from_square = 1u64 << from_square_index;
            let mut movement_bb = 
                get_supply_moves(from_square_index, our_occupancy, their_occupancy) |
                get_demand_moves(from_square_index, our_occupancy, their_occupancy) |
                get_up_moves(from_square_index, our_occupancy, their_occupancy) |
                get_down_moves(from_square_index, our_occupancy, their_occupancy) |
                get_left_moves(from_square_index, our_occupancy, their_occupancy) |
                get_right_moves(from_square_index, our_occupancy, their_occupancy);
            while movement_bb != 0 {
                let to_square_index = movement_bb.trailing_zeros() as usize;
                let to_square = 1 << to_square_index;

                if to_square & their_occupancy == 0 {
                    self.add_quiet_move(position, MoveList::move_builder(
                        from_square_index, 
                        to_square_index, 
                        0, 
                        0, 
                        0
                    ));
                } else {
                    let captured_piece = position.pieces[to_square_index];
                    if let Some(piece) = captured_piece {
                        self.add_capture_move(position, MoveList::move_builder(
                            from_square_index, 
                            to_square_index, 
                            piece.bb_index(), 
                            0, 
                            0
                        ));
                    }
                }
                movement_bb &= movement_bb - 1;
            }
            queens &= queens - 1;
        }
    }
}