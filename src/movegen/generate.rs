use crate::{board::Board, defs::{Color, PieceType}, movegen::{BLACK_PAWN_ATTACKS, DEMAND_DIAGONAL_RAYS, DOWN_RAYS, LEFT_RAYS, RANK_BB_MASK, RIGHT_RAYS, SUPPLY_DIAGONAL_RAYS, UP_RAYS, WHITE_PAWN_ATTACKS}};

fn get_up_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
    let ray = UP_RAYS[square];
    let blockers = ray & (occupied_bb[Color::White as usize] | occupied_bb[Color::Black as usize]);

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
        return attack_mask & !occupied_bb[side as usize];
    }
}
fn get_down_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
    let ray = DOWN_RAYS[square];
    let blockers = ray & (occupied_bb[Color::White as usize] | occupied_bb[Color::Black as usize]);

    if blockers == 0 { return ray; }
    else {
        let first_blocker_square = 63 - blockers.leading_zeros();
        let mask_to_blocker = u64::MAX << first_blocker_square;
        let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
        return attack_mask & !occupied_bb[side as usize];
    }
}
fn get_left_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
    let ray = LEFT_RAYS[square];
    let blockers = ray & (occupied_bb[Color::White as usize] | occupied_bb[Color::Black as usize]);

    if blockers == 0 { return ray }
    else {
        let first_blocker_square = 63 - blockers.leading_zeros();
        let mask_to_blocker = RIGHT_RAYS[first_blocker_square as usize] & ray;
        let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
        return attack_mask & !occupied_bb[side as usize];
    }
}
fn get_right_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
    let ray = RIGHT_RAYS[square];
    let blockers = ray & (occupied_bb[Color::White as usize] | occupied_bb[Color::Black as usize]);

    if blockers == 0 { return ray; }
    else {
        let first_blocker_square = blockers.trailing_zeros();
        let mask_to_blocker = LEFT_RAYS[first_blocker_square as usize] & ray;
        let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
        return attack_mask & !occupied_bb[side as usize];
    }
}

fn get_supply_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
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
    let blockers = ray & (occupied_bb[Color::White as usize] | occupied_bb[Color::Black as usize]);

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

    return (positive_attacks | negative_attacks) & !occupied_bb[side as usize];
}
fn get_demand_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
    fn get_demand_positive_ray(square: usize) -> u64 {
        let ray = DEMAND_DIAGONAL_RAYS[square];
        ray & u64::MAX.checked_shl((square + 1) as u32).unwrap_or(0)
    }
    fn get_demand_negative_ray(square: usize) -> u64 {
        let ray = DEMAND_DIAGONAL_RAYS[square];
        ray & ((1 << square) - 1)
    }

    let ray = DEMAND_DIAGONAL_RAYS[square];
    let all_blockers = ray & (occupied_bb[Color::White as usize] | occupied_bb[Color::Black as usize]);

    let positive_ray = get_demand_positive_ray(square);
    let negative_ray = get_demand_negative_ray(square);
    
    // Up left (positive)
    let positive_blockers = all_blockers & positive_ray;
    let mut positive_attacks = positive_ray;
    if positive_blockers != 0 {
        let first_blocker_square = positive_blockers.trailing_zeros();
        let mask_to_blocker = positive_ray & get_demand_negative_ray(first_blocker_square as usize);
        positive_attacks = (positive_ray & mask_to_blocker) | (1 << first_blocker_square);
    }

    // Down right (negative)
    let negative_blockers = all_blockers & negative_ray;
    let mut negative_attacks = negative_ray;
    if negative_blockers != 0 {
        let first_blocker_square = 63 - negative_blockers.leading_zeros();
        let mask_to_blocker = negative_ray & get_demand_positive_ray(first_blocker_square as usize);
        negative_attacks = (negative_ray & mask_to_blocker) | (1 << first_blocker_square);
    }

    return (positive_attacks | negative_attacks) & !occupied_bb[side as usize];
}

fn get_pawn_moves(position: Board, square: usize, side: Color) -> u64 {
    let occupied_bb = [position.occupancies(Color::White), position.occupancies(Color::Black)];
    let empty_squares = !(occupied_bb[Color::White as usize] | occupied_bb[Color::Black as usize]);
    let color = side;
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
        // add_pawn_move(position, 
        //     sq120(from_square as u8) as usize, 
        //     sq120(to_square as u8) as usize, 
        //     move_list
        // );
        single_pushes &= single_pushes - 1;
    }

    while double_pushes != 0 {
        let to_square = double_pushes.trailing_zeros() as usize;
        let from_square = (to_square as i32 - (push_offset * 2)) as usize;
        // let the_move = move_builder(
        //     sq120(from_square as u8) as u32,
        //     sq120(to_square as u8) as u32,
        //     Empty as u32,
        //     Empty as u32,
        //     MOVE_FLAG_PAWN_START
        // );
        // add_quiet_move(position, the_move, move_list);
        double_pushes &= double_pushes - 1;
    }

    let mut pawns = our_pawns;
    while pawns != 0 {
        let from_square = pawns.trailing_zeros() as usize;
        let mut valid_captures = pawn_attacks[from_square] & their_pieces;

        while valid_captures != 0 {
            let to_square = valid_captures.trailing_zeros() as usize;
            let captured_piece = position.pieces[to_square];
            // add_pawn_capture_move(position, 
            //     sq120(from_square as u8) as usize, 
            //     sq120(to_square as u8) as usize, 
            //     captured_piece as usize, 
            //     move_list
            // );
            valid_captures &= valid_captures - 1;
        }
        pawns &= pawns - 1;
    }

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
        // add_pawn_move(position, 
        //     sq120(from_square as u8) as usize, 
        //     sq120(to_square as u8) as usize, 
        //     move_list
        // );
        single_pushes &= single_pushes - 1;
    }

    while double_pushes != 0 {
        let to_square = double_pushes.trailing_zeros() as usize;
        let from_square = (to_square as i32 - (push_offset * 2)) as usize;
        // let the_move = move_builder(
        //     sq120(from_square as u8) as u32,
        //     sq120(to_square as u8) as u32,
        //     Empty as u32,
        //     Empty as u32,
        //     MOVE_FLAG_PAWN_START
        // );
        // add_quiet_move(position, the_move, move_list);
        double_pushes &= double_pushes - 1;
    }

    let mut pawns = our_pawns;
    while pawns != 0 {
        let from_square = pawns.trailing_zeros() as usize;
        let mut valid_captures = pawn_attacks[from_square] & their_pieces;

        while valid_captures != 0 {
            let to_square = valid_captures.trailing_zeros() as usize;
            let captured_piece = position.pieces[to_square];
            // add_pawn_capture_move(position, 
            //     sq120(from_square as u8) as usize, 
            //     sq120(to_square as u8) as usize, 
            //     captured_piece as usize, 
            //     move_list
            // );
            valid_captures &= valid_captures - 1;
        }
        pawns &= pawns - 1;
    }

    if let Some(en_passant) = position.en_passant {
        let en_passant_square = en_passant;
        let potential_attackers = en_passant_attackers_table[en_passant_square];
        let mut en_passant_attackers = potential_attackers & our_pawns;

        while en_passant_attackers != 0 {
            let from_square = en_passant_attackers.trailing_zeros();
            // let the_move = move_builder(
            //     sq120(from_square as u8) as u32,
            //     position.en_passant as u32,
            //     Empty as u32,
            //     Empty as u32,
            //     MOVE_FLAG_EN_PASSANT
            // );
            // add_en_passant_move(position, the_move, move_list);
            en_passant_attackers &= en_passant_attackers - 1;
        }
    }

    0
}