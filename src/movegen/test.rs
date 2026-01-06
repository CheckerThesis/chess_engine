use crate::{board::Board, movegen::MoveList};

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
        assert_eq!(move_list.count, 16, "The POSITION3 should have 16 moves"); // 14 with fully legal movegen

        position = Board::new(POSITION4);
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
        move_list.generate_all_moves(&position);
        assert_eq!(move_list.count, 38, "The POSITION4 should have 38 moves"); // 6 with fully legal movegen

        position = Board::new(POSITION4_MIRRORED);
        #[cfg(debug_assertions)] { position.check_board(fn_name!()); }
        move_list.generate_all_moves(&position);
        assert_eq!(move_list.count, 38, "The POSITION4_MIRRORED should have 38 moves"); // 6 with fully legal movegen
    }

    #[test]
    fn test_promotion() {
        
    }
}