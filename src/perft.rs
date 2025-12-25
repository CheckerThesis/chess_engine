use crate::{board::Board, fens::FEN_START, movegen::{MoveList, SQUARE_TO_STRING}};

pub fn perft(position: &mut Board, depth: usize) -> usize {
    if depth == 0 { return 1 }

    let mut move_list = MoveList::new();
    let mut nodes = 0;

    move_list.generate_all_moves(&position);

    for i in 0..move_list.count {
        let mv = move_list.moves[i].mv;

        #[cfg(debug_assertions)]
        let snapshot = (
            position.position_key,
            position.en_passant,
            position.castle_permission,
            position.fifty_move,
            position.side
        );

        if !position.make_move(mv) { continue; }
        nodes += perft(position, depth - 1);
        position.take_move();

        #[cfg(debug_assertions)] {
            let (key, ep, castle, fifty, side) = snapshot;

            if position.position_key != key { panic!("Zobrist key mismatch after unmake_move"); }
            if position.en_passant != ep { panic!("EP square mismatch after unmake_move"); }
            if position.castle_permission != castle { panic!("Castling rights mismatch after unmake_move"); }
            if position.fifty_move != fifty { panic!("Fifty-move counter mismatch after unmake_move"); }
            if position.side != side { panic!("Side-to-move mismatch after unmake_move"); }

            position.check_board("perft");

        }
    }

    return nodes
}

pub fn perft_divide(position: &mut Board, depth: usize) {
    let mut move_list = MoveList::new();
    let mut total = 0;

    move_list.generate_all_moves(&position);

    for i in 0..move_list.count {
        let mv = move_list.moves[i].mv;
        let from = mv.from_square();
        let to = mv.to_square();
   
        if !position.make_move(mv) { continue;  }

        let nodes = if depth == 1 { 1 } else { perft(position, depth - 1) };
        total += nodes;

        let mv = move_list.moves[i].mv;
        let from = mv.from_square();
        let to = mv.to_square();
        println!("{}{}: {}", 
            SQUARE_TO_STRING[from], 
            SQUARE_TO_STRING[to], 
            nodes
        );

        position.take_move();
    }
    println!("\nTotal: {}", total);
}

#[cfg(test)]
mod tests {
    use crate::fens::{KIWIPETE, POSITION3, POSITION4, POSITION4_FLIPPED, POSITION5, POSITION6};

    use super::*;

    #[test]
    fn mm_perft_depth_1() {
        let mut position = Board::new(FEN_START);
        let mut perft_value = perft(&mut position, 1);
        assert_eq!(perft_value, 20);

        position = Board::new(KIWIPETE);
        perft_value = perft(&mut position, 1);
        assert_eq!(perft_value, 48);

        position = Board::new(POSITION3);
        perft_value = perft(&mut position, 1);
        assert_eq!(perft_value, 14);

        position = Board::new(POSITION4);
        perft_value = perft(&mut position, 1);
        assert_eq!(perft_value, 6);

        position = Board::new(POSITION4_FLIPPED);
        perft_value = perft(&mut position, 1);
        assert_eq!(perft_value, 6);

        position = Board::new(POSITION5);
        perft_value = perft(&mut position, 1);
        assert_eq!(perft_value, 44);

        position = Board::new(POSITION6);
        perft_value = perft(&mut position, 1);
        assert_eq!(perft_value, 46);
    }

    #[test]
    fn mm_perft_depth_2() {
        let mut position = Board::new(FEN_START);
        let mut perft_value = perft(&mut position, 2);
        assert_eq!(perft_value, 400);

        position = Board::new(KIWIPETE);
        perft_value = perft(&mut position, 2);
        assert_eq!(perft_value, 2039);

        position = Board::new(POSITION3);
        perft_value = perft(&mut position, 2);
        assert_eq!(perft_value, 191);

        position = Board::new(POSITION4);
        perft_value = perft(&mut position, 2);
        assert_eq!(perft_value, 264);

        position = Board::new(POSITION4_FLIPPED);
        perft_value = perft(&mut position, 2);
        assert_eq!(perft_value, 264);

        position = Board::new(POSITION5);
        perft_value = perft(&mut position, 2);
        assert_eq!(perft_value, 1486);

        position = Board::new(POSITION6);
        perft_value = perft(&mut position, 2);
        assert_eq!(perft_value, 2079);
    }

    #[test]
    fn mm_perft_depth_3() {
        let mut position = Board::new(FEN_START);
        let mut perft_value = perft(&mut position, 3);
        assert_eq!(perft_value, 8902);

        position = Board::new(KIWIPETE);
        perft_value = perft(&mut position, 3);
        assert_eq!(perft_value, 97862);

        position = Board::new(POSITION3);
        perft_value = perft(&mut position, 3);
        assert_eq!(perft_value, 2812);

        position = Board::new(POSITION4);
        perft_value = perft(&mut position, 3);
        assert_eq!(perft_value, 9467);

        position = Board::new(POSITION4_FLIPPED);
        perft_value = perft(&mut position, 3);
        assert_eq!(perft_value, 9467);

        position = Board::new(POSITION5);
        perft_value = perft(&mut position, 3);
        assert_eq!(perft_value, 62379);

        position = Board::new(POSITION6);
        perft_value = perft(&mut position, 3);
        assert_eq!(perft_value, 89890);
    }

    #[test]
    fn perft_depth_4() {
        let mut position = Board::new(FEN_START);
        let mut perft_value = perft(&mut position, 4);
        assert_eq!(perft_value, 197281);

        position = Board::new(KIWIPETE);
        perft_value = perft(&mut position, 4);
        assert_eq!(perft_value, 4085603);

        position = Board::new(POSITION3);
        perft_value = perft(&mut position, 4);
        assert_eq!(perft_value, 43238);

        position = Board::new(POSITION4);
        perft_value = perft(&mut position, 4);
        assert_eq!(perft_value, 422333);

        position = Board::new(POSITION4_FLIPPED);
        perft_value = perft(&mut position, 4);
        assert_eq!(perft_value, 422333);

        position = Board::new(POSITION5);
        perft_value = perft(&mut position, 4);
        assert_eq!(perft_value, 2103487);

        position = Board::new(POSITION6);
        perft_value = perft(&mut position, 4);
        assert_eq!(perft_value, 3894594);
    }

    #[test]
    fn perft_depth_5() {
        let mut position = Board::new(FEN_START);
        let mut perft_value = perft(&mut position, 5);
        assert_eq!(perft_value, 4865609);

        position = Board::new(KIWIPETE);
        perft_value = perft(&mut position, 5);
        assert_eq!(perft_value, 193690690);

        position = Board::new(POSITION3);
        perft_value = perft(&mut position, 5);
        assert_eq!(perft_value, 674624);

        position = Board::new(POSITION4);
        perft_value = perft(&mut position, 5);
        assert_eq!(perft_value, 15833292);

        position = Board::new(POSITION4_FLIPPED);
        perft_value = perft(&mut position, 5);
        assert_eq!(perft_value, 15833292);

        position = Board::new(POSITION5);
        perft_value = perft(&mut position, 5);
        assert_eq!(perft_value, 89941194);

        position = Board::new(POSITION6);
        perft_value = perft(&mut position, 5);
        assert_eq!(perft_value, 164075551);
    }

    #[test]
    fn perft_depth_6() {
        let mut position = Board::new(FEN_START);
        let mut perft_value = perft(&mut position, 6);
        assert_eq!(perft_value, 119060324);

        position = Board::new(KIWIPETE);
        perft_value = perft(&mut position, 6);
        assert_eq!(perft_value, 8031647685);

        position = Board::new(POSITION3);
        perft_value = perft(&mut position, 6);
        assert_eq!(perft_value, 11030083);

        position = Board::new(POSITION4);
        perft_value = perft(&mut position, 6);
        assert_eq!(perft_value, 706045033);

        position = Board::new(POSITION4_FLIPPED);
        perft_value = perft(&mut position, 6);
        assert_eq!(perft_value, 706045033);

        position = Board::new(POSITION6);
        perft_value = perft(&mut position, 6);
        assert_eq!(perft_value, 6923051137);
    }
}