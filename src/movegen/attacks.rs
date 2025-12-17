use crate::{board::{Board, print_bitboard}, defs::{Color, Piece, PieceType, RANKS_BOARD, Ranks}, movegen::{MoveList, bitboards::{BLACK_PAWN_ATTACKS, DEMAND_DIAGONAL_RAYS, DOWN_RAYS, KING_RAYS, KNIGHT_RAYS, LEFT_RAYS, RANK_BB_MASK, RIGHT_RAYS, SUPPLY_DIAGONAL_RAYS, UP_RAYS, WHITE_PAWN_ATTACKS}}};

pub static mut hi: usize = 0; 

pub fn get_up_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
    let ray = UP_RAYS[square];
    let blockers = ray & (our_occupancy | their_occupancy);

    if blockers == 0 { return ray; }
    let mask_to_blocker = blockers ^ (blockers - 1);
    let attack_mask = ray & mask_to_blocker;

    attack_mask & !our_occupancy
}
pub fn get_down_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
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
pub fn get_left_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
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
pub fn get_right_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
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

pub fn get_supply_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
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
pub fn get_demand_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
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

pub fn square_attacked(square: usize, side: Color, position: &Board) -> bool {
    let enemy_side = side.opposite();
    
    // Pawns
    let enemy_pawns = position.bitboards[PieceType::Pawn.bb_index(enemy_side)];
    
    let capturing_offsets = if side == Color::White { &WHITE_PAWN_ATTACKS[square] } 
    else { &BLACK_PAWN_ATTACKS[square] };

    if (capturing_offsets & enemy_pawns) != 0 { return true }

    // Knights
    let enemy_knights = position.bitboards[PieceType::Knight.bb_index(enemy_side)];
    if (KNIGHT_RAYS[square] & enemy_knights) != 0 { return true; }

    let our_occupancy = position.occupancies(side);
    // print_bitboard(our_occupancy);
    let their_occupancy = position.occupancies(enemy_side);
    
    // Diagonal
    let enemy_bishops = position.bitboards[PieceType::Bishop.bb_index(enemy_side)];
    let enemy_queens = position.bitboards[PieceType::Queen.bb_index(enemy_side)];
    let diagonal_attackers = enemy_bishops | enemy_queens;

    let bishop_attacks = 
        get_demand_moves(square, our_occupancy, their_occupancy) |
        get_supply_moves(square, our_occupancy, their_occupancy);
    // println!("------");
    // print_bitboard(our_occupancy);
    // print_bitboard(their_occupancy);
    // print_bitboard(bishop_attacks);
    if (bishop_attacks & diagonal_attackers) != 0 { return true; }

    // Orthogonal
    let enemy_rooks = position.bitboards[PieceType::Rook.bb_index(enemy_side)];
    let orthogonal_attackers = enemy_rooks | enemy_queens;

    let rook_attacks = 
        get_up_moves(square, our_occupancy, their_occupancy) |
        get_down_moves(square, our_occupancy, their_occupancy) |
        get_left_moves(square, our_occupancy, their_occupancy) |
        get_right_moves(square, our_occupancy, their_occupancy);
    if (rook_attacks & orthogonal_attackers) != 0 { return true; }

    // Kings
    let enemy_kings = position.bitboards[PieceType::King.bb_index(enemy_side)];
    if (KING_RAYS[square] & enemy_kings) != 0 { return true; }

    false
}