use crate::{board::Board, defs::{Color, Piece, PieceType, RANKS_BOARD, Ranks}, movegen::{MoveList, attacks::{get_demand_moves, get_down_moves, get_left_moves, get_right_moves, get_supply_moves, get_up_moves, square_attacked}, bitboards::{BLACK_PAWN_ATTACKS, DEMAND_DIAGONAL_RAYS, DOWN_RAYS, KING_RAYS, KNIGHT_RAYS, LEFT_RAYS, RANK_BB_MASK, RIGHT_RAYS, SUPPLY_DIAGONAL_RAYS, UP_RAYS, WHITE_PAWN_ATTACKS}}};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fn_name;

    #[test]
    fn test_generate_all_moves() {        
        pub const FEN_START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        pub const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        pub const POSITION3: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        pub const POSITION4: &str = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        pub const POSITION4_MIRRORED: &str = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1 ";

        let mut move_list: MoveList = MoveList::new();

        let mut position: Board = Board::new(FEN_START);
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
        move_list.generate_all_moves(&position);
        assert_eq!(move_list.count, 20, "The FEN_START should have 20 moves");

        position = Board::new(KIWIPETE);
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
        move_list.generate_all_moves(&position);
        assert_eq!(move_list.count, 48, "The KIWIPETE should have 48 moves");

        position = Board::new(POSITION3);
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
        move_list.generate_all_moves(&position);
        assert_eq!(move_list.count, 15, "The POSITION3 should have 15 moves"); // 14 with fully legal movegen

        position = Board::new(POSITION4);
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
        move_list.generate_all_moves(&position);
        assert_eq!(move_list.count, 37, "The POSITION4 should have 37 moves"); // 6 with fully legal movegen

        position = Board::new(POSITION4_MIRRORED);
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
        move_list.generate_all_moves(&position);
        assert_eq!(move_list.count, 37, "The POSITION4_MIRRORED should have 37 moves"); // 6 with fully legal movegen
    }
}