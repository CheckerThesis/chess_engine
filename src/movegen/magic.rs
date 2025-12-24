use std::{cell::OnceCell, sync::{LazyLock, OnceLock}};

use crate::movegen::{attacks::{get_demand_moves, get_down_moves, get_left_moves, get_right_moves, get_supply_moves, get_up_moves}, bitboards::{DOWN_RAYS, LEFT_RAYS, RIGHT_RAYS, UP_RAYS}};

const MAGIC_ROOK_SHIFT: u8 = 12;
const MAGIC_BISHOP_SHIFT: u8 = 9;
const ATTACK_TABLE_SIZE: usize = 88507;

const ROOK_MAGICS: [(u64, u32); 64] = [
    (0x80280013FF84FFFF, 10890), (0x5FFBFEFDFEF67FFF, 50579), (0xFFEFFAFFEFFDFFFF, 62020),
    (0x003000900300008A, 67322), (0x0050028010500023, 80251), (0x0020012120A00020, 58503),
    (0x0030006000C00030, 51175), (0x0058005806B00002, 83130), (0x7FBFF7FBFBEAFFFC, 50430),
    (0x0000140081050002, 21613), (0x0000180043800048, 72625), (0x7FFFE800021FFFB8, 80755),
    (0xFFFFCFFE7FCFFFAF, 69753), (0x00001800C0180060, 26973), (0x4F8018005FD00018, 84972),
    (0x0000180030620018, 31958), (0x00300018010C0003, 69272), (0x0003000C0085FFFF, 48372),
    (0xFFFDFFF7FBFEFFF7, 65477), (0x7FC1FFDFFC001FFF, 43972), (0xFFFEFFDFFDFFDFFF, 57154),
    (0x7C108007BEFFF81F, 53521), (0x20408007BFE00810, 30534), (0x0400800558604100, 16548),
    (0x0040200010080008, 46407), (0x0010020008040004, 11841), (0xFFFDFEFFF7FBFFF7, 21112),
    (0xFEBF7DFFF8FEFFF9, 44214), (0xC00000FFE001FFE0, 57925), (0x4AF01F00078007C3, 29574),
    (0xBFFBFAFFFB683F7F, 17309), (0x0807F67FFA102040, 40143), (0x200008E800300030, 64659),
    (0x0000008780180018, 70469), (0x0000010300180018, 62917), (0x4000008180180018, 60997),
    (0x008080310005FFFA, 18554), (0x4000188100060006, 14385), (0xFFFFFF7FFFBFBFFF,     0),
    (0x0000802000200040, 38091), (0x20000202EC002800, 25122), (0xFFFFF9FF7CFFF3FF, 60083),
    (0x000000404B801800, 72209), (0x2000002FE03FD000, 67875), (0xFFFFFF6FFE7FCFFD, 56290),
    (0xBFF7EFFFBFC00FFF, 43807), (0x000000100800A804, 73365), (0x6054000A58005805, 76398),
    (0x0829000101150028, 20024), (0x00000085008A0014,  9513), (0x8000002B00408028, 24324),
    (0x4000002040790028, 22996), (0x7800002010288028, 23213), (0x0000001800E08018, 56002),
    (0xA3A80003F3A40048, 22809), (0x2003D80000500028, 44545), (0xFFFFF37EEFEFDFBE, 36072),
    (0x40000280090013C1,  4750), (0xBF7FFEFFBFFAF71F,  6014), (0xFFFDFFFF777B7D6E, 36054),
    (0x48300007E8080C02, 78538), (0xAFE0000FFF780402, 28745), (0xEE73FFFBFFBB77FE,  8555),
    (0x0002000308482882,  1009)
];

const BISHOP_MAGICS: [(u64, u32); 64] = [
    (0xA7020080601803D8, 60984), (0x13802040400801F1, 66046), (0x0A0080181001F60C, 32910),
    (0x1840802004238008, 16369), (0xC03FE00100000000, 42115), (0x24C00BFFFF400000,   835),
    (0x0808101F40007F04, 18910), (0x100808201EC00080, 25911), (0xFFA2FEFFBFEFB7FF, 63301),
    (0x083E3EE040080801, 16063), (0xC0800080181001F8, 17481), (0x0440007FE0031000, 59361),
    (0x2010007FFC000000, 18735), (0x1079FFE000FF8000, 61249), (0x3C0708101F400080, 68938),
    (0x080614080FA00040, 61791), (0x7FFE7FFF817FCFF9, 21893), (0x7FFEBFFFA01027FD, 62068),
    (0x53018080C00F4001, 19829), (0x407E0001000FFB8A, 26091), (0x201FE000FFF80010, 15815),
    (0xFFDFEFFFDE39FFEF, 16419), (0xCC8808000FBF8002, 59777), (0x7FF7FBFFF8203FFF, 16288),
    (0x8800013E8300C030, 33235), (0x0420009701806018, 15459), (0x7FFEFF7F7F01F7FD, 15863),
    (0x8700303010C0C006, 75555), (0xC800181810606000, 79445), (0x20002038001C8010, 15917),
    (0x087FF038000FC001,  8512), (0x00080C0C00083007, 73069), (0x00000080FC82C040, 16078),
    (0x000000407E416020, 19168), (0x00600203F8008020, 11056), (0xD003FEFE04404080, 62544),
    (0xA00020C018003088, 80477), (0x7FBFFE700BFFE800, 75049), (0x107FF00FE4000F90, 32947),
    (0x7F8FFFCFF1D007F8, 59172), (0x0000004100F88080, 55845), (0x00000020807C4040, 61806),
    (0x00000041018700C0, 73601), (0x0010000080FC4080, 15546), (0x1000003C80180030, 45243),
    (0xC10000DF80280050, 20333), (0xFFFFFFBFEFF80FDC, 33402), (0x000000101003F812, 25917),
    (0x0800001F40808200, 32875), (0x084000101F3FD208,  4639), (0x080000000F808081, 17077),
    (0x0004000008003F80, 62324), (0x08000001001FE040, 18159), (0x72DD000040900A00, 61436),
    (0xFFFFFEFFBFEFF81D, 57073), (0xCD8000200FEBF209, 61025), (0x100000101EC10082, 81259),
    (0x7FBAFFFFEFE0C02F, 64083), (0x7F83FFFFFFF07F7F, 56114), (0xFFF1FFFFFFF7FFC1, 57058),
    (0x0878040000FFE01F, 58912), (0x945E388000801012, 22194), (0x0840800080200FDA, 70880),
    (0x100000C05F582008, 11140)
];

fn get_rook_mask(square: usize) -> u64 {
    let r = square / 8;
    let f = square % 8;
    let mut mask = 0u64;

    for r_i in (r + 1)..7 { mask |= 1 << (r_i * 8 + f); } 
    for r_i in 1..r       { mask |= 1 << (r_i * 8 + f); } 
    for f_i in (f + 1)..7 { mask |= 1 << (r * 8 + f_i); } 
    for f_i in 1..f       { mask |= 1 << (r * 8 + f_i); } 
    
    mask
}
fn get_bishop_mask(square: usize) -> u64 {
    let r = (square / 8) as i8;
    let f = (square % 8) as i8;
    let mut mask = 0u64;

    for (dr, df) in [(1, 1), (-1, 1), (-1, -1), (1, -1)] {
        let mut tr = r + dr;
        let mut tf = f + df;
        
        while tr > 0 && tr < 7 && tf > 0 && tf < 7 {
            mask |= 1 << (tr * 8 + tf);
            tr += dr;
            tf += df;
        }
    }
    mask
}

// Inited from the above functions
const ROOK_MASKS: [u64; 64] = [
    282578800148862,565157600297596,1130315200595066,2260630401190006,
    4521260802379886,9042521604759646,18085043209519166,36170086419038334,
    282578800180736,565157600328704,1130315200625152,2260630401218048,
    4521260802403840,9042521604775424,18085043209518592,36170086419037696,
    282578808340736,565157608292864,1130315208328192,2260630408398848,
    4521260808540160,9042521608822784,18085043209388032,36170086418907136,
    282580897300736,565159647117824,1130317180306432,2260632246683648,
    4521262379438080,9042522644946944,18085043175964672,36170086385483776,
    283115671060736,565681586307584,1130822006735872,2261102847592448,
    4521664529305600,9042787892731904,18085034619584512,36170077829103616,
    420017753620736,699298018886144,1260057572672512,2381576680245248,
    4624614895390720,9110691325681664,18082844186263552,36167887395782656,
    35466950888980736,34905104758997504,34344362452452352,33222877839362048,
    30979908613181440,26493970160820224,17522093256097792,35607136465616896,
    9079539427579068672,8935706818303361536,8792156787827803136,8505056726876686336,
    7930856604974452736,6782456361169985536,4485655873561051136,9115426935197958144,
];
const BISHOP_MASKS: [u64; 64] = [
    18049651735527936,70506452091904,275415828992,1075975168,
    38021120,8657588224,2216338399232,567382630219776,
    9024825867763712,18049651735527424,70506452221952,275449643008,
    9733406720,2216342585344,567382630203392,1134765260406784,
    4512412933816832,9024825867633664,18049651768822272,70515108615168,
    2491752130560,567383701868544,1134765256220672,2269530512441344,
    2256206450263040,4512412900526080,9024834391117824,18051867805491712,
    637888545440768,1135039602493440,2269529440784384,4539058881568768,
    1128098963916800,2256197927833600,4514594912477184,9592139778506752,
    19184279556981248,2339762086609920,4538784537380864,9077569074761728,
    562958610993152,1125917221986304,2814792987328512,5629586008178688,
    11259172008099840,22518341868716544,9007336962655232,18014673925310464,
    2216338399232,4432676798464,11064376819712,22137335185408,
    44272556441600,87995357200384,35253226045952,70506452091904,
    567382630219776,1134765260406784,2832480465846272,5667157807464448,
    11333774449049600,22526811443298304,9024825867763712,18049651735527936,
];

fn construct_occupancy(mut index: u64, mut mask: u64) -> u64 {
    let mut result = 0;
    while mask != 0 {
        let lsb = mask & (!mask + 1); 
        mask &= !lsb;

        if (index & 1) != 0 { result |= lsb; }
        index >>= 1;
    }
    result
}

static ATTACKS: LazyLock<Box<[u64]>> = LazyLock::new(|| {
    let mut table = vec![0; ATTACK_TABLE_SIZE];

    for square in 0..64 {
        let (magic, offset) = ROOK_MAGICS[square];
        let mask = get_rook_mask(square);
        let bits =  mask.count_ones();
        let permutations = 1 << bits;

        for i in 0..permutations {
            let occupancy = construct_occupancy(i, mask);
            let index = offset as usize + ((occupancy | !mask).wrapping_mul(magic) >> 52) as usize;
            let attacks = 
                get_up_moves(square, 0, occupancy) |
                get_down_moves(square, 0, occupancy) |
                get_left_moves(square, 0, occupancy) |
                get_right_moves(square, 0, occupancy);
            table[index] = attacks;
        }
    }

    for square in 0..64 {
        let (magic, offset) = BISHOP_MAGICS[square];
        let mask = get_bishop_mask(square);
        let bits = mask.count_ones();
        let permutations = 1 << bits;

        for i in 0..permutations {
            let occupancy = construct_occupancy(i, mask);
            
            // MAGIC FORMULA: Shift 55
            let index = (offset as usize) + 
                ((occupancy | !mask).wrapping_mul(magic) >> 55) as usize;

            // Use your Supply/Demand helpers (passing 0 for our_occupancy to get full rays)
            let attacks = 
                get_supply_moves(square, 0, occupancy) | 
                get_demand_moves(square, 0, occupancy);
                
            table[index] = attacks;
        }
    }

    table.into_boxed_slice()
});

fn init_rook_attack_table() -> Vec<u64> {
    let mut table = vec![0; ATTACK_TABLE_SIZE];

    for square in 0..64 {
        let (magic, offset) = ROOK_MAGICS[square];
        let mask = get_rook_mask(square);
        let bits =  mask.count_ones();
        let permutations = 1 << bits;

        for i in 0..permutations {
            let occupancy = construct_occupancy(i, mask);
            let index = offset as usize + ((occupancy | !mask).wrapping_mul(magic) >> 52) as usize;
            let attacks = 
                get_up_moves(square, 0, occupancy) |
                get_down_moves(square, 0, occupancy) |
                get_left_moves(square, 0, occupancy) |
                get_right_moves(square, 0, occupancy);
            table[index] = attacks;
        }
    }

    table
}
fn init_bishop_attack_table() -> Vec<u64> {
    // Bishop table is small (~5kb to 80kb depending on packing). 
    // Your offsets go up to ~81k, so alloc that much.
    let mut table = vec![0; 82000]; 

    for square in 0..64 {
        let (magic, offset) = BISHOP_MAGICS[square];
        let mask = get_bishop_mask(square);
        let bits = mask.count_ones();
        let permutations = 1 << bits;

        for i in 0..permutations {
            let occupancy = construct_occupancy(i, mask);
            
            // MAGIC FORMULA: Shift 55
            let index = (offset as usize) + 
                ((occupancy | !mask).wrapping_mul(magic) >> 55) as usize;

            // Use your Supply/Demand helpers (passing 0 for our_occupancy to get full rays)
            let attacks = 
                get_supply_moves(square, 0, occupancy) | 
                get_demand_moves(square, 0, occupancy);
                
            table[index] = attacks;
        }
    }
    table
}

pub struct MagicTable {
    movement_bitboards: [u64; 64],
//  (magic number, array offset)
    magics: [(u64, u32); 64],
    shift: u8 // either MAGIC_ROOK_SHIFT or MAGIC_BISHOP_SHIFT
}
impl MagicTable {
    pub fn new(movement_bb: [u64; 64], magics: [(u64, u32); 64], shift: u8) -> Self {
        MagicTable { movement_bitboards: movement_bb, magics: magics, shift: shift }
    }
    pub fn get_attacks(&self, square: usize, occupancy: u64) -> u64 {
        let (magic, offset) = self.magics[square];
        let index = offset as usize + (((occupancy | !self.movement_bitboards[square]).wrapping_mul(magic)) >> (64 - self.shift)) as usize;
        
        ATTACKS[index]
        // unsafe { *ATTACKS.get_unchecked(index) }
    }
}

pub static ROOK_MAGIC_BB: LazyLock<MagicTable> = LazyLock::new(|| {
    MagicTable::new(
        ROOK_MASKS, ROOK_MAGICS, MAGIC_ROOK_SHIFT
    )
});
pub static BISHOP_MAGIC_BB: LazyLock<MagicTable> = LazyLock::new(|| {
    MagicTable::new(
        BISHOP_MASKS, BISHOP_MAGICS, MAGIC_BISHOP_SHIFT
    )
});

#[cfg(test)]
mod tests {
    use crate::{board::Board, defs::{Color, Piece, PieceType}, fens::{FEN_START, KIWIPETE, POSITION3, POSITION4, POSITION4_FLIPPED, POSITION5, POSITION6}, movegen::{attacks::{get_demand_moves, get_down_moves, get_left_moves, get_right_moves, get_supply_moves, get_up_moves}, magic::{BISHOP_MAGICS, MAGIC_BISHOP_SHIFT, MAGIC_ROOK_SHIFT, MagicTable, ROOK_MAGICS, get_bishop_mask, get_rook_mask, init_bishop_attack_table, init_rook_attack_table}}};

    #[test]
    fn test_magic_vs_classic_consistency() {        
        let mut rook_masks = [0; 64];
        let mut bishop_masks = [0; 64];
        for square in 0..64 {
            rook_masks[square] = get_rook_mask(square);
            bishop_masks[square] = get_bishop_mask(square);
        }

        let rook_table = MagicTable::new(
            rook_masks, 
            ROOK_MAGICS, 
            MAGIC_ROOK_SHIFT
        );

        let bishop_table = MagicTable::new(
            bishop_masks, 
            BISHOP_MAGICS, 
            MAGIC_BISHOP_SHIFT
        );

        let fens = [
            FEN_START, KIWIPETE, POSITION3, POSITION4, 
            POSITION4_FLIPPED, POSITION5, POSITION6,
        ];

        for (fen_idx, fen) in fens.iter().enumerate() {
            let position = Board::new(fen);
            let white_occ = position.occupancies[Color::WHITE.index()];
            let black_occ = position.occupancies[Color::BLACK.index()];
            let both_occ  = position.occupancies[Color::BOTH.index()];

            for square in 0..64 {
                let piece = position.pieces[square];

                if piece == Piece::NONE { continue; }

                let color = piece.color();
                let p_type = piece.piece_type();

                let (our_occ, their_occ) = if color == Color::WHITE {
                    (white_occ, black_occ)
                } else {
                    (black_occ, white_occ)
                };

                if p_type == PieceType::ROOK || p_type == PieceType::QUEEN {
                    let classic = 
                        get_up_moves(square, our_occ, their_occ) |
                        get_down_moves(square, our_occ, their_occ) |
                        get_left_moves(square, our_occ, their_occ) |
                        get_right_moves(square, our_occ, their_occ);
                    
                    let magic = rook_table.get_attacks(square, both_occ) & !our_occ;

                    assert_eq!(classic, magic, 
                        "\nFAIL: Orthogonal mismatch at FEN #{}\nFEN: {}\nSquare: {}\nPiece: {}\nClassic: {:064b}\nMagic:   {:064b}\n",
                        fen_idx, fen, square, piece, classic, magic
                    );
                }

                if p_type == PieceType::BISHOP || p_type == PieceType::QUEEN {
                    let classic = get_supply_moves(square, our_occ, their_occ) |
                                get_demand_moves(square, our_occ, their_occ);
                    
                    let magic = bishop_table.get_attacks(square, both_occ) & !our_occ;

                    assert_eq!(classic, magic, 
                        "\nFAIL: Diagonal mismatch at FEN #{}\nFEN: {}\nSquare: {}\nPiece: {}\nClassic: {:064b}\nMagic:   {:064b}\n",
                        fen_idx, fen, square, piece, classic, magic
                    );
                }
            }
        }
    }
}