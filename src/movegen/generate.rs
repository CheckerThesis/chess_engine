use crate::{board::{print_bitboard, Board}, defs::{Color, Piece, PieceType}, movegen::{MoveList, BLACK_PAWN_ATTACKS, DEMAND_DIAGONAL_RAYS, DOWN_RAYS, LEFT_RAYS, RANK_BB_MASK, RIGHT_RAYS, SUPPLY_DIAGONAL_RAYS, UP_RAYS, WHITE_PAWN_ATTACKS}};

pub fn get_up_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
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
pub fn get_down_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
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
pub fn get_left_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
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
pub fn get_right_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
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

pub fn get_supply_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
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
pub fn get_demand_moves(square: usize, occupied_bb: &[u64; 3], side: Color) -> u64 {
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

// SLIDING MOVEGEN