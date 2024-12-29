use crate::defs::Board;

//position.history_ply - position.fifty_move as usize
pub fn is_repetition(position: &Board) -> bool {
    let start = position.history_ply.saturating_sub(position.fifty_move as usize);

    // println!(
    //     "history_ply: {}, fifty_move: {}, start: {}",
    //     position.history_ply,
    //     position.fifty_move,
    //     position.history_ply.saturating_sub(position.fifty_move as usize)
    // );

    for i in start..position.history_ply - 1 {
        if position.position_key == position.history[i].position_key {
            return true;
        }
    }

    false
}

pub fn search_position(position: &mut Board) {

}
