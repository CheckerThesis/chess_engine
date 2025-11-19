use crate::{board::Board, defs::{Color, Piece, PieceType}};

pub fn square_attacked(square: usize, side: Color, position: &Board) -> bool {
    let occupied_bb = [
        position.occupancies(Color::White), 
        position.occupancies(Color::Black), 
        position.occupancies(Color::White) | position.occupancies(Color::Black)
    ];

    false
}