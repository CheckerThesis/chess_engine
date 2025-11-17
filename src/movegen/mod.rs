pub mod attacks;
pub mod generate;

use core::fmt;
use std::sync::LazyLock;

use crate::{board::Board, defs::{Color, Piece, PieceType, Ranks, RANKS_BOARD}};

pub struct MoveList {
    moves: [u64; 256],
    count: usize
}
impl MoveList {
    pub fn new() -> Self {
        Self {
            moves: [0; 256],
            count: 0,
        }
    }

    fn add(&mut self, mv: u64) {
        self.moves[self.count] = mv;
        self.count += 1;
    }

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
        let from_square_index = bishops.trailing_zeros();
        let from_square = 1u64 << from_square_index;
        let mut movement_bb = 
            get_supply_moves(from_square_index, our_occupancy, their_occupancy) |
            get_demand_moves(from_square_index, our_occupancy, their_occupancy);
        while movement_bb != 0 {
            let to_square_index = movement_bb.trailing_zeros() as u64;
            let to_square = 1 << to_square_index;

            if to_square & their_occupancy == 0 {
                self.add_quiet_move(position, MoveList::move_builder(
                    from_square_index as usize, 
                    to_square_index as usize, 
                    0, 
                    0, 
                    0
                ));
            } else {
                add_capture_move(position, move_builder(
                    sq120(from_square_index as u8) as u32, 
                    sq120(to_square_index as u8) as u32, 
                    position.pieces[sq120(to_square_index as u8) as usize] as u32, 
                    0 as u32, 
                    0), 
                    move_list
                );
                self.add_capture_move(position, MoveList::move_builder(
                    from_square_index as usize, 
                    to_square_index as usize, 
                    position.pieces, 
                    0, 
                    0
                ));
            }
            movement_bb &= movement_bb - 1;
        }
        bishops &= bishops - 1;
    }

    let mut rooks = position.bitboards[piece[1]];
    while rooks != 0 {
        let from_square_index = rooks.trailing_zeros() as usize;
        let from_square = 1u64 << from_square_index;
        let mut movement_bb = 
            get_up_moves(from_square_index, &position.occupancies, side) |
            get_down_moves(from_square_index, &position.occupancies, side) |
            get_left_moves(from_square_index, &position.occupancies, side) |
            get_right_moves(from_square_index, &position.occupancies, side);
        while movement_bb != 0 {
            let to_square_index = movement_bb.trailing_zeros() as u64;
            let to_square = 1 << to_square_index;

            if to_square & their_occupancy == 0 {
                add_quiet_move(position, move_builder(
                    sq120(from_square_index as u8) as u32, 
                    sq120(to_square_index as u8) as u32, 
                    Empty as u32, 
                    Empty as u32, 
                    0), 
                    move_list
                );
            } else {
                add_capture_move(position, move_builder(
                    sq120(from_square_index as u8) as u32, 
                    sq120(to_square_index as u8) as u32, 
                    position.pieces[sq120(to_square_index as u8) as usize] as u32, 
                    0 as u32, 
                    0), 
                    move_list
                );
            }
            movement_bb &= movement_bb - 1;
        }
        rooks &= rooks - 1;
    }

    let mut queens = position.bitboards[piece[2]];
    while queens != 0 {
        let from_square_index = queens.trailing_zeros() as usize;
        let from_square = 1u64 << from_square_index;
        let mut movement_bb = 
            get_supply_moves(from_square_index, &position.occupancies, side) |
            get_demand_moves(from_square_index, &position.occupancies, side) |
            get_up_moves(from_square_index, &position.occupancies, side) |
            get_down_moves(from_square_index, &position.occupancies, side) |
            get_left_moves(from_square_index, &position.occupancies, side) |
            get_right_moves(from_square_index, &position.occupancies, side);
        while movement_bb != 0 {
            let to_square_index = movement_bb.trailing_zeros() as u64;
            let to_square = 1 << to_square_index;

            if to_square & their_occupancy == 0 {
                add_quiet_move(position, move_builder(
                    sq120(from_square_index as u8) as u32, 
                    sq120(to_square_index as u8) as u32, 
                    Empty as u32, 
                    Empty as u32, 
                    0), 
                    move_list
                );
            } else {
                add_capture_move(position, move_builder(
                    sq120(from_square_index as u8) as u32, 
                    sq120(to_square_index as u8) as u32, 
                    position.pieces[sq120(to_square_index as u8) as usize] as u32, 
                    0 as u32, 
                    0), 
                    move_list
                );
            }
            movement_bb &= movement_bb - 1;
        }
        queens &= queens - 1;
    }
}


    pub fn len(&self) -> usize { self.count }

    pub fn iter(&self) -> std::slice::Iter<'_, u64> { self.moves[..self.count].iter() }
}

const SQUARE_TO_STRING: [&str; 64] = [
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
];

/// Returns the promotion piece character based on the piece index.
///
/// This function makes assumptions based on your `add_..._pawn_move` functions:
/// 1. `promote: 0` is used for no-promotion.
/// 2. The piece indices for Q, R, B, N are passed for promotion.
/// 3. We assume a common encoding like:
///    W_Q=5, W_R=4, W_B=3, W_N=2
///    B_Q=11, B_R=10, B_B=9, B_N=8
///    (This order matches your `queen`, `rook`, `bishop`, `knight` calls)
fn get_promo_char(promo_index: u32) -> &'static str {
    match promo_index {
        // White pieces (assuming N=2, B=3, R=4, Q=5)
        2 => "n",
        3 => "b",
        4 => "r",
        5 => "q",
        // Black pieces (assuming n=8, b=9, r=10, q=11)
        8 => "n",
        9 => "b",
        10 => "r",
        11 => "q",
        // Default: no promotion (index 0) or invalid (King/Pawn)
        _ => "",
    }
}

impl fmt::Display for MoveList {
    /// Formats the move list for printing.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write a header
        writeln!(f, "Move List ({} moves found):", self.count)?;
        
        // Iterate over all moves in the list
        for i in 0..self.count {
            // Get the 64-bit move entry
            let move_with_score = self.moves[i];

            // Extract the 32-bit move data (lower 32 bits)
            let move_data = move_with_score as u32;

            // Extract the score (upper bits)
            let score = move_with_score >> 47;

            // --- Decode the 32-bit move data ---
            // Based on: move_builder(from, to, capture, flags, promote)
            
            // from: bits 0-5 (mask 0x3F)
            let from = (move_data & 0x3F) as usize;
            
            // to: bits 6-11 (mask 0x3F)
            let to = ((move_data >> 6) & 0x3F) as usize;
            
            // promote: bits 20-24 (mask 0x1F)
            let promote = (move_data >> 20) & 0x1F;

            // Convert square indices to algebraic notation
            let from_sq_str = SQUARE_TO_STRING[from];
            let to_sq_str = SQUARE_TO_STRING[to];
            
            // Get the promotion character (e.g., "q", "r", or "")
            let promo_str = get_promo_char(promote as u32);

            // Write the formatted move string
            // Example: "  1: e2e4 (Score: 0)"
            // Example: " 12: e7e8q (Score: 10000)"
            writeln!(f, "  {:>2}: {}{}{} (Score: {})",
                     i + 1,
                     from_sq_str,
                     to_sq_str,
                     promo_str,
                     score)?;
        }
        
        Ok(())
    }
}

/*
0000 0000 0000 0000 0000 0000 0011 1111 -> From
0000 0000 0000 0000 0000 1111 1100 0000 -> To
0000 0000 0000 0001 1111 0000 0000 0000 -> Captured
0000 0000 0000 0010 0000 0000 0000 0000 -> Is enpassant
0000 0000 0000 0100 0000 0000 0000 0000 -> Is pawn start
0000 0000 0000 1000 0000 0000 0000 0000 -> Is castle
0000 0001 1111 0000 0000 0000 0000 0000 -> Promoted piece
*/
pub fn from_square(mv: u32) -> usize { (mv & 0x3F) as usize }
pub fn to_square(mv: u32) -> usize { (mv >> 6 & 0x3F) as usize }
pub fn captured(mv: u32) -> usize { (mv >> 12 & 0x1F) as usize }
pub fn promoted(mv: u32) -> usize { (mv >> 20 & 0x1F) as usize }

pub static RANK_BB_MASK: LazyLock<[u64; 9]> = LazyLock::new(|| {
    let mut rank_bb_mask: [u64; 9] = [0; 9];
    const RANK_1: u64 = 0x00000000000000FF;

    for i in 1..9 {
        rank_bb_mask[i] = RANK_1 << ((i - 1) * 8);
    }

    rank_bb_mask
});

fn get_rank(square: u64) -> usize {
    for i in 0..8 {
        if square & RANK_BB_MASK[i] != 0 { return i }
    }
    100
}

const A_FILE_MASK: u64 = !0xFEFEFEFEFEFEFEFE;
const B_FILE_MASK: u64 = A_FILE_MASK << 1;
const G_FILE_MASK: u64 = A_FILE_MASK << 6;
const H_FILE_MASK: u64 = !0x7F7F7F7F7F7F7F7F;

pub static WHITE_PAWN_ATTACKS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut attacks: [u64; 64] = [0; 64];

    for i in 0..64 {
        let sq: u64 = 1 << i;
        let capture;

        if A_FILE_MASK & sq != 0 { capture = sq << 9; } 
        else if H_FILE_MASK & sq != 0 { capture = sq << 7; } 
        else { capture = (sq << 7) | (sq << 9); }

        attacks[i] = capture;
    }

    attacks
});

pub static BLACK_PAWN_ATTACKS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut attacks: [u64; 64] = [0; 64];

    for i in (0..64).rev() {
        let sq: u64 = 1 << i as u64;
        let capture;
        
        if A_FILE_MASK & sq != 0 { capture = sq >> 7; } 
        else if H_FILE_MASK & sq != 0 { capture = sq >> 9; } 
        else { capture = (sq >> 7) | (sq >> 9); }

        attacks[i] = capture;
    }

    attacks
});

pub static UP_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut up_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let mut ray_bb: u64 = 0;
        let mut ray_square= square as u64;

        while get_rank(ray_square) < 7 {
            ray_square += 8;
            if ray_square < 64 { ray_bb |= 1 << ray_square; } 
            else { break; }
        }
        up_rays[square] = ray_bb;
    }

    up_rays[0] = up_rays[1] >> 1;
    up_rays
});
pub static DOWN_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut down_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let mut ray_bb: u64 = 0;
        let mut ray_square= square as u64;

        while get_rank(ray_square) < 7 {
            if ray_square >= 8 { 
                ray_square -= 8;
                ray_bb |= 1 << ray_square; 
            }
            else { break; }
        }
        down_rays[square] = ray_bb;
    }

    down_rays
});
pub static LEFT_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut left_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let mut ray_bb: u64 = 0;
        let mut ray_square = square as u64;

        while ray_square % 8 > 0 {
            ray_square -= 1;
            ray_bb |= 1 << ray_square;
        }
        left_rays[square] = ray_bb;
    }

    left_rays
});
pub static RIGHT_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut right_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let mut ray_bb: u64 = 0;
        let mut ray_square = square as u64;

        while ray_square % 8 < 7 {
            ray_square += 1;
            ray_bb |= 1 << ray_square;
        }
        right_rays[square] = ray_bb;
    }

    right_rays
});

pub static SUPPLY_DIAGONAL_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut anti_diagonal_rays: [u64; 64] = [0; 64];

    for square in 0..64 {
        let mut ray_bb: u64 = 0;

        let mut ray_square_up_right = square as u64;
        // Rank is (square / 8), File is (square % 8)
        while (ray_square_up_right / 8) < 7 && (ray_square_up_right % 8) < 7 {
            ray_square_up_right += 9;
            ray_bb |= 1 << ray_square_up_right;
        }

        let mut ray_square_down_light = square as u64;
        while (ray_square_down_light / 8) > 0 && (ray_square_down_light % 8) > 0 {
            ray_square_down_light -= 9;
            ray_bb |= 1 << ray_square_down_light;
        }

        anti_diagonal_rays[square] = ray_bb;
    }

    anti_diagonal_rays
});
pub static DEMAND_DIAGONAL_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    let mut main_diagonal_rays: [u64; 64] = [0; 64];

    for square in 0..64 {
        let mut ray_bb: u64 = 0;

        // --- Calculate Up-Left ray (+7) ---
        let mut ray_square_up_left = square as u64;
        while (ray_square_up_left / 8) < 7 && (ray_square_up_left % 8) > 0 {
            ray_square_up_left += 7; // Move one square up-left
            ray_bb |= 1 << ray_square_up_left; // Add this square to the bitboard
        }

        // --- Calculate Down-Right ray (-7) ---
        let mut current_square_dr = square as u64;
        // Loop while the current square is not on the 1st rank (rank 0)
        // and not on the H file (file 7).
        // Rank is (square / 8), File is (square % 8)
        while (current_square_dr / 8) > 0 && (current_square_dr % 8) < 7 {
            current_square_dr -= 7; // Move one square down-right
            ray_bb |= 1 << current_square_dr; // Add this square to the bitboard
        }

        main_diagonal_rays[square] = ray_bb;
    }

    main_diagonal_rays
});

pub static KNIGHT_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    const NOT_A_FILE: u64 = 0xfefefefefefefefe; 
    const NOT_AB_FILE: u64 = 0xfcfcfcfcfcfcfcfc;
    const NOT_GH_FILE: u64 = 0x3f3f3f3f3f3f3f3f;
    const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

    let mut knight_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let square_bb: u64 = 1 << square;
        let mut attacks: u64 = 0;

        attacks |= (square_bb & NOT_H_FILE) << 17;
        attacks |= (square_bb & NOT_GH_FILE) << 10;
        attacks |= (square_bb & NOT_GH_FILE) >> 6;
        attacks |= (square_bb & NOT_H_FILE) >> 15;

        attacks |= (square_bb & NOT_A_FILE) << 15;
        attacks |= (square_bb & NOT_AB_FILE) << 6;
        attacks |= (square_bb & NOT_AB_FILE) >> 10;
        attacks |= (square_bb & NOT_A_FILE) >> 17;

        knight_rays[square] = attacks;
    }

    knight_rays
});

pub static KING_RAYS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    const NOT_A_FILE: u64 = 0xfefefefefefefefe; 
    const NOT_H_FILE: u64 = 0x7f7f7f7f7f7f7f7f;

    let mut king_rays: [u64; 64] = [0; 64];

    for square in 0..64 as usize {
        let square_bb: u64 = 1 << square;
        let mut attacks: u64 = 0;

        attacks |= square_bb << 8;  // up
        attacks |= square_bb >> 8;  // down
        
        // Horizontal moves (left and right)
        attacks |= (square_bb & NOT_H_FILE) << 1;  // right
        attacks |= (square_bb & NOT_A_FILE) >> 1;  // left
        
        // Diagonal moves
        attacks |= (square_bb & NOT_H_FILE) << 9;  // up-right
        attacks |= (square_bb & NOT_A_FILE) << 7;  // up-left
        attacks |= (square_bb & NOT_H_FILE) >> 7;  // down-right
        attacks |= (square_bb & NOT_A_FILE) >> 9;  // down-left

        king_rays[square] = attacks;
    }

    king_rays
});