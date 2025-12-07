use colored::Colorize;

use crate::{board::Board, defs::{Castling, Color, Piece, PieceType, RANKS_BOARD, Ranks}, movegen::{MoveFlag, MoveList, attacks::{get_demand_moves, get_down_moves, get_left_moves, get_right_moves, get_supply_moves, get_up_moves, square_attacked}, bitboards::{BLACK_PAWN_ATTACKS, DEMAND_DIAGONAL_RAYS, DOWN_RAYS, KING_RAYS, KNIGHT_RAYS, LEFT_RAYS, RANK_BB_MASK, RIGHT_RAYS, SUPPLY_DIAGONAL_RAYS, UP_RAYS, WHITE_PAWN_ATTACKS}}};

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
                MoveFlag::NONE,
                Piece::bb_index(&queen)
            ));
            self.add_quiet_move(position, MoveList::move_builder(
                from, 
                to, 
                0, 
                0,
                Piece::bb_index(&rook)
            ));
            self.add_quiet_move(position, MoveList::move_builder(
                from, 
                to, 
                0, 
                MoveFlag::NONE,
                Piece::bb_index(&bishop)
            ));
            self.add_quiet_move(position, MoveList::move_builder(
                from, 
                to, 
                0, 
                MoveFlag::NONE,
                Piece::bb_index(&knight)
            ));
        } else {
            self.add_quiet_move(position, MoveList::move_builder(
                from, 
                to, 
                0, 
                MoveFlag::NONE,
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
                MoveFlag::NONE,
                Piece::bb_index(&queen)
            ));
            self.add_capture_move(position, MoveList::move_builder(
                from, 
                to, 
                captured, 
                MoveFlag::NONE,
                Piece::bb_index(&rook)
            ));
            self.add_capture_move(position, MoveList::move_builder(
                from, 
                to, 
                captured, 
                MoveFlag::NONE,
                Piece::bb_index(&bishop)
            ));
            self.add_capture_move(position, MoveList::move_builder(
                from, 
                to, 
                captured, 
                MoveFlag::NONE,
                Piece::bb_index(&knight)
            ));
        } else {
            self.add_capture_move(position, MoveList::move_builder(
                from, 
                to, 
                captured, 
                MoveFlag::NONE,
                0
            ));
        }
    }

    fn add_enpassant_move(&mut self, position: &Board, from: usize, to: usize) {
        let enemy_pawn = match position.side {
        Color::White => PieceType::Pawn.bb_index(Color::Black),
        Color::Black => PieceType::Pawn.bb_index(Color::White),
        _ => {
            eprintln!("{}", "add_enpassant_move: Invalid side".red());
            panic!()
        }
    };
        
        self.add_capture_move(position, MoveList::move_builder(
            from, 
            to, 
            enemy_pawn, 
            MoveFlag::EN_PASSANT, 
            0
        ));
    }

    #[inline(always)]
    fn serialize_moves(&mut self, position: &Board, from_square: usize, attacks: u64) {
        let side = position.side;
        let their_occupancy = position.occupancies(side.opposite());
        let our_occupancy = position.occupancies(side);
        
        let valid_moves = attacks & !our_occupancy;
        let mut captures = valid_moves & their_occupancy;
        let mut quiets = valid_moves & !their_occupancy;

        while captures != 0 {
            let to_square = captures.trailing_zeros() as usize;
            let captured_piece = position.pieces[to_square].unwrap();
    
            self.add_capture_move(position, MoveList::move_builder(
                from_square, to_square, captured_piece.bb_index(), MoveFlag::NONE, 0
            ));
            
            captures &= captures - 1;
        }

        while quiets != 0 {
            let to_square = quiets.trailing_zeros() as usize;
            
            self.add_quiet_move(position, MoveList::move_builder(
                from_square, to_square, 0, MoveFlag::NONE, 0
            ));
            
            quiets &= quiets - 1;
        }
    }

    fn generate_pawn_moves(&mut self, position: &Board) {
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
                MoveFlag::PAWN_START, 
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
                self.add_enpassant_move(position, from_square, en_passant_square);
        
                en_passant_attackers &= en_passant_attackers - 1;
            }
        }
    }

    fn generate_knight_moves(&mut self, position: &Board) {
        let side = position.side;
        let our_pieces = position.occupancies(side);
        let their_pieces = position.occupancies(side.opposite());

        let mut knights = position.bitboards[PieceType::Knight.bb_index(side)];
        while knights != 0 {
            let from_square_index = knights.trailing_zeros() as usize;
            self.serialize_moves(position, from_square_index, KNIGHT_RAYS[from_square_index]);
            knights &= knights - 1;
        }
    }

    fn generate_sliding_moves(&mut self, position: &Board) {
        let side = position.side;
        let our_occupancy = position.occupancies(side);
        let their_occupancy = position.occupancies(side.opposite());

        let mut bishops = position.bitboards[PieceType::Bishop.bb_index(side)];
        while bishops != 0 {
            let from_square_index = bishops.trailing_zeros() as usize;
            let movement_bb = 
                get_supply_moves(from_square_index, our_occupancy, their_occupancy) |
                get_demand_moves(from_square_index, our_occupancy, their_occupancy);
            self.serialize_moves(position, from_square_index, movement_bb);
            bishops &= bishops - 1;
        }

        let mut rooks = position.bitboards[PieceType::Rook.bb_index(side)];
        while rooks != 0 {
            let from_square_index = rooks.trailing_zeros() as usize;
            let movement_bb = 
                get_up_moves(from_square_index, our_occupancy, their_occupancy) |
                get_down_moves(from_square_index, our_occupancy, their_occupancy) |
                get_left_moves(from_square_index, our_occupancy, their_occupancy) |
                get_right_moves(from_square_index, our_occupancy, their_occupancy);
            self.serialize_moves(position, from_square_index, movement_bb);
            rooks &= rooks - 1;
        }

        let mut queens = position.bitboards[PieceType::Queen.bb_index(side)];
        while queens != 0 {
            let from_square_index = queens.trailing_zeros() as usize;
            let movement_bb = 
                get_supply_moves(from_square_index, our_occupancy, their_occupancy) |
                get_demand_moves(from_square_index, our_occupancy, their_occupancy) |
                get_up_moves(from_square_index, our_occupancy, their_occupancy) |
                get_down_moves(from_square_index, our_occupancy, their_occupancy) |
                get_left_moves(from_square_index, our_occupancy, their_occupancy) |
                get_right_moves(from_square_index, our_occupancy, their_occupancy);
            self.serialize_moves(position, from_square_index, movement_bb);
            queens &= queens - 1;
        }
    }

    fn generate_king_moves(&mut self, position: &Board) {
        let side = position.side;
        let our_pieces = position.occupancies(side);
        let their_pieces = position.occupancies(side.opposite());

        let mut kings = position.bitboards[PieceType::King.bb_index(side)];
        while kings != 0 {
            let from_sq = kings.trailing_zeros() as usize;
            let mut moves = KING_RAYS[from_sq] & !our_pieces;

            while moves != 0 {
                let to_sq = moves.trailing_zeros() as usize;
                
                if !square_attacked(to_sq, position.side, position) {
                    let to_bb = 1u64 << to_sq;

                    if (to_bb & their_pieces) != 0 {
                        let captured_piece = position.pieces[to_sq].unwrap();
                        self.add_capture_move(position, MoveList::move_builder(
                            from_sq, to_sq, captured_piece.bb_index(), 0, 0
                        ));
                    } else {
                        self.add_quiet_move(position, MoveList::move_builder(
                            from_sq, to_sq, 0, 0, 0
                        ));
                    }
                }

                moves &= moves - 1;
            }
            kings &= kings - 1;
        }
    }

    fn generate_castle_moves(&mut self, position: &Board) {
        let side = position.side;
        let occupancies = position.occupancies(Color::White) | position.occupancies(Color::Black);

        match side {
            Color::White => {
                // King Side (e1 -> g1)
                if (position.castle_permission & Castling::WhiteKingCastle as u8) != 0 {
                    if (occupancies & ((1 << 5) | (1 << 6))) == 0 {
                        // If e1, f1, g1 not under attack
                        if !square_attacked(4, position.side, position) &&
                           !square_attacked(5, position.side, position) &&
                           !square_attacked(6, position.side, position)
                        {
                            self.add_quiet_move(position, MoveList::move_builder(
                                4,
                                6,
                                0,
                                MoveFlag::CASTLE,
                                0
                            ));
                        }
                    }
                }

                // Queen Side (e1 -> c1)
                if (position.castle_permission & Castling::WhiteQueenCastle as u8) != 0 {
                    if (occupancies & ((1 << 1) | (1 << 2) | (1 << 3))) == 0 {
                        // If e1, d1, c1 not under attack
                        if !square_attacked(4, position.side, position) &&
                           !square_attacked(3, position.side, position) && 
                           !square_attacked(2, position.side, position)
                        {
                            self.add_quiet_move(position, MoveList::move_builder(
                                4,
                                2,
                                0,
                                MoveFlag::CASTLE,
                                0
                            ));
                        }
                    }
                }
            },
            Color::Black => {
                // King Side (e8 -> g8)
                if (position.castle_permission & Castling::BlackKingCastle as u8) != 0 {
                    if (occupancies & ((1 << 61) | (1 << 62))) == 0 {
                        // If e8, f8, g8 not under attack
                        if !square_attacked(60, position.side, position) &&
                           !square_attacked(61, position.side, position) &&
                           !square_attacked(62, position.side, position)
                        {
                            self.add_quiet_move(position, MoveList::move_builder(
                                60,
                                62,
                                0,
                                MoveFlag::CASTLE,
                                0
                            ));
                        }
                    }
                }

                // Queen Side (e8 -> c8)
                if (position.castle_permission & Castling::BlackQueenCastle as u8) != 0 {
                    if (occupancies & ((1 << 57) | (1 << 58) | (1 << 59))) == 0 {
                        // If e8, d8, c8 not under attack
                        if !square_attacked(60, position.side, position) &&
                           !square_attacked(59, position.side, position) &&
                           !square_attacked(58, position.side, position)    
                        {
                            self.add_quiet_move(position, MoveList::move_builder(
                                60,
                                58,
                                0,
                                MoveFlag::CASTLE,
                                0
                            ));
                        }
                    }
                }
            },
            _ => {}
        }
    }

    pub fn generate_all_moves(&mut self, position: &Board) {
        self.moves = [0; 256];
        self.count = 0;
        self.generate_pawn_moves(position);
        self.generate_knight_moves(position);
        self.generate_sliding_moves(position);
        self.generate_king_moves(position);
        self.generate_castle_moves(position);
    }
}
