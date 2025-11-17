#![allow(warnings)]
use crate::{board::{Board, print_bitboard}, defs::{Color, FILES_BOARD, RANKS_BOARD, Ranks}, fens::{FEN_ENPASSANT, FEN_PROMOTION_BLACK, FEN_PROMOTION_WHITE, FEN_TRICKY}, movegen::{BLACK_PAWN_ATTACKS, MoveList, RANK_BB_MASK, SUPPLY_DIAGONAL_RAYS, WHITE_PAWN_ATTACKS}};

mod defs;
mod board;
mod fens;
mod io;
mod movegen;
mod transposition_table;
/*
TODO
- Determine what struct to use to handle movelist
    - Determine what struct to use to handle move
- Test difference between packing bits for moves and a full struct
    - Change the move bit flags to bools


*/
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
    println!("ray");
    print_bitboard(ray);
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

fn main() {
    let position: Board = Board::new(FEN_TRICKY);
    let mut move_list: MoveList = MoveList::new();
    println!("{position}");
    position.check_board(fn_name!());
    move_list.generate_pawn_moves(&position);
    println!("{move_list}");

    let side = position.side;
    let our_occupancy = position.occupancies(side);
    let their_occupancy = position.occupancies(side.opposite());

    print_bitboard(get_supply_moves(11, our_occupancy, their_occupancy));
}
