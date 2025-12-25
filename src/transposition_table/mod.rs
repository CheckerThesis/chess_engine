/*
 u32                 u16        u8
0000 0000 0000 0000 0000 0000 0011 1111 -> From
0000 0000 0000 0000 0000 1111 1100 0000 -> To
0000 0000 0000 0001 1111 0000 0000 0000 -> Captured
0000 0000 0000 0010 0000 0000 0000 0000 -> Is enpassant
0000 0000 0000 0100 0000 0000 0000 0000 -> Is pawn start
0000 0000 0000 1000 0000 0000 0000 0000 -> Is castle
0000 0001 1111 0000 0000 0000 0000 0000 -> Promoted piece

 u64         --- Split ---          
0000 0000 0000 0000 0000 0000 0001 1111 -> Depth (for replacement policy)
0000 0000 0000 0000 0001 1111 1110 0000 -> Age (for replacement policy)
0000 0000 0000 0000 0110 0000 0000 0000 -> Alpha beta flags
0111 1111 1111 1111 1000 0000 0000 0000 -> Score (32000 max)
*/

pub struct HashEntry {
    position_key: u64,
    data: u64 // score, alpha beta flags, replacement policy flags, the move
}

pub struct HashTable {
    entry: Vec<HashEntry>,
    length: usize
}