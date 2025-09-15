use crate::{board::Board, defs::{Color, Piece, PieceType}, movegen::generate::{get_demand_moves, get_down_moves, get_left_moves, get_right_moves, get_supply_moves, get_up_moves}};

pub fn square_attacked(square: usize, side: Color, position: &Board) -> bool {
    let occupied_bb = [
        position.occupancies(Color::White), 
        position.occupancies(Color::Black), 
        position.occupancies(Color::White) | position.occupancies(Color::Black)
    ];

    false
}