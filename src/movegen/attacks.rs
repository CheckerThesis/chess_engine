use crate::{board::{Board, print_bitboard}, defs::{Color, Piece, PieceType, RANKS_BOARD, Ranks}, movegen::{MoveList, bitboards::{BLACK_PAWN_ATTACKS, KING_RAYS, KNIGHT_RAYS, WHITE_PAWN_ATTACKS}, magic::{get_bishop_attacks, get_rook_attacks}}};

#[inline(always)]
pub fn square_attacked(square: usize, side: Color, position: &Board) -> bool {
    debug_assert!(square < 64);

    let enemy_side = side.opposite();
    
    // Pawns
    let enemy_pawns = position.bitboards[PieceType::PAWN.bb_index(enemy_side)];
    
    let capturing_offsets = 
        if side == Color::WHITE { WHITE_PAWN_ATTACKS[square] } 
        else { BLACK_PAWN_ATTACKS[square] };

    if (capturing_offsets & enemy_pawns) != 0 { return true }

    // Knights
    let enemy_knights = position.bitboards[PieceType::KNIGHT.bb_index(enemy_side)];
    if (KNIGHT_RAYS[square] & enemy_knights) != 0 { return true; }

    let our_occupancy = position.occupancies[side.index()];
    let their_occupancy = position.occupancies[enemy_side.index()];
    
    // Diagonal
    let enemy_bishops = position.bitboards[PieceType::BISHOP.bb_index(enemy_side)];
    let enemy_queens = position.bitboards[PieceType::QUEEN.bb_index(enemy_side)];
    let diagonal_attackers = enemy_bishops | enemy_queens;

    let bishop_attacks = get_bishop_attacks(square, our_occupancy | their_occupancy) & !our_occupancy;
    if (bishop_attacks & diagonal_attackers) != 0 { return true; }

    // Orthogonal
    let enemy_rooks = position.bitboards[PieceType::ROOK.bb_index(enemy_side)];
    let orthogonal_attackers = enemy_rooks | enemy_queens;

    let rook_attacks = get_rook_attacks(square, our_occupancy | their_occupancy) & !our_occupancy;
    if (rook_attacks & orthogonal_attackers) != 0 { return true; }

    // Kings
    let enemy_kings = position.bitboards[PieceType::KING.bb_index(enemy_side)];
    if (KING_RAYS[square] & enemy_kings) != 0 { return true; }

    false
}

impl Board {
    pub fn get_smallest_attacker(&self, square: usize, side: Color, occupancy: u64) -> Option<(PieceType, u64)> {
        let pawns = self.bitboards[PieceType::PAWN.bb_index(side)] & occupancy;
        if pawns != 0 {
            let attack_mask = 
                if side == Color::WHITE { BLACK_PAWN_ATTACKS[square] }
                else { WHITE_PAWN_ATTACKS[square] };
            let attackers = attack_mask & pawns;
            if attackers != 0 { return Some((PieceType::PAWN, attackers & attackers.wrapping_neg())) }
        }

        let knights = self.bitboards[PieceType::KNIGHT.bb_index(side)] & occupancy;
        if knights != 0 {
            let attackers = KNIGHT_RAYS[square] & knights;
            if attackers != 0 {
                return Some((PieceType::KNIGHT, attackers & attackers.wrapping_neg()));
            }
        }

        let bishops = self.bitboards[PieceType::BISHOP.bb_index(side)] & occupancy;
        let queens = self.bitboards[PieceType::QUEEN.bb_index(side)] & occupancy;
        if bishops | queens != 0 {
            let diagonal_attacks = get_bishop_attacks(square, occupancy);
            let bishop_attackers = diagonal_attacks & bishops;
            if bishop_attackers != 0 { return Some((PieceType::BISHOP, bishop_attackers & bishop_attackers.wrapping_neg())); }
        }

        let rooks = self.bitboards[PieceType::ROOK.bb_index(side)] & occupancy;
        if rooks | queens != 0 {
            let rook_attacks = get_rook_attacks(square, occupancy);
            let rook_attackers = rook_attacks & rooks;
            if rook_attackers != 0 { return Some((PieceType::ROOK, rook_attackers & rook_attackers.wrapping_neg())); }
        
            let mut queen_attackers = 0;
            if queens != 0 {
                let all_attacks = get_bishop_attacks(square, occupancy) | get_rook_attacks(square, occupancy);
                queen_attackers = all_attacks & queens;
                if queen_attackers != 0 { return Some((PieceType::QUEEN, queen_attackers & queen_attackers.wrapping_neg())); }
            }
        }

        let king = self.bitboards[PieceType::KING.bb_index(side)] & occupancy;
        if (KING_RAYS[square] & king) != 0 {
            return Some((PieceType::KING, king));
        }

        None
    }
}
