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