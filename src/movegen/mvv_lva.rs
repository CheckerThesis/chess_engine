/*
// PieceType indices: 
// 0:None, 1:Pawn, 2:Knight, 3:Bishop, 4:Rook, 5:Queen, 6:King, 7:Dragon
let scores = [0, 100, 200, 300, 400, 500, 600, 780];

println!("// Indexed by [Victim.piece_type()][Attacker.piece_type()]");
println!("pub const MVV_LVA_SCORES: [[u32; 8]; 8] = [");

for victim in 0..8 {
    print!("    [");
    for attacker in 0..8 {
        // Row 0 (None) and Col 0 (None) should be 0
        if victim == 0 || attacker == 0 {
            print!("{:3}", 0);
        } else {
            // Formula: Victim + Offset - (Attacker / 100)
            // Offset is 10 to handle Dragon (780/100 = 7). 
            // Ex: Pawn(100) cap by Dragon(7) = 100 + 10 - 7 = 103.
            let val = scores[victim] + 10 - (scores[attacker] / 100);
            print!("{:3}", val);
        }
        
        if attacker < 7 { print!(", "); }
    }
    println!("],");
}
println!("];");
*/
// Inited from the above
pub const MVV_LVA_SCORES: [[i16; 8]; 8] = [
    [  0,   0,   0,   0,   0,   0,   0,   0], // None
    [  0, 109, 108, 107, 106, 105, 104, 103], // Pawn
    [  0, 209, 208, 207, 206, 205, 204, 203], // Knight
    [  0, 309, 308, 307, 306, 305, 304, 303], // Bishop
    [  0, 409, 408, 407, 406, 405, 404, 403], // Rook
    [  0, 509, 508, 507, 506, 505, 504, 503], // Queen
    [  0, 609, 608, 607, 606, 605, 604, 603], // King
    [  0, 789, 788, 787, 786, 785, 784, 783], // Dragon
];