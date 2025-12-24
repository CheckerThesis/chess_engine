use crate::movegen::{attack_table::ATTACKS};

const MAGIC_ROOK_SHIFT: u8 = 12;
const MAGIC_BISHOP_SHIFT: u8 = 9;
const ATTACK_TABLE_SIZE: usize = 88507;

// Gotten from CozyChess
//         (magic number, array offset)
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

#[cfg(debug_assertions)] fn get_rook_mask(square: usize) -> u64 {
    let r = square / 8;
    let f = square % 8;
    let mut mask = 0u64;

    for r_i in (r + 1)..7 { mask |= 1 << (r_i * 8 + f); } 
    for r_i in 1..r       { mask |= 1 << (r_i * 8 + f); } 
    for f_i in (f + 1)..7 { mask |= 1 << (r * 8 + f_i); } 
    for f_i in 1..f       { mask |= 1 << (r * 8 + f_i); } 
    
    mask
}
#[cfg(debug_assertions)] fn get_bishop_mask(square: usize) -> u64 {
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
// Inited from above functions
const ROOK_RAYS: [u64; 64] = [
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
const BISHOP_RAYS: [u64; 64] = [
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

pub fn get_rook_attacks(square: usize, occupancy: u64) -> u64 {
    let (magic, offset) = ROOK_MAGICS[square];
    let index = offset as usize + (((occupancy | !ROOK_RAYS[square]).wrapping_mul(magic)) >> (64 - MAGIC_ROOK_SHIFT)) as usize;    
    // ATTACKS[index]
    unsafe { *ATTACKS.get_unchecked(index) }
}
pub fn get_bishop_attacks(square: usize, occupancy: u64) -> u64 {
    let (magic, offset) = BISHOP_MAGICS[square];
    let index = offset as usize + (((occupancy | !BISHOP_RAYS[square]).wrapping_mul(magic)) >> (64 - MAGIC_BISHOP_SHIFT)) as usize;
    unsafe { *ATTACKS.get_unchecked(index) }

}

#[cfg(test)]
mod tests {
    use crate::{board::Board, defs::{Color, Piece, PieceType}, fens::{FEN_START, KIWIPETE, POSITION3, POSITION4, POSITION4_FLIPPED, POSITION5, POSITION6}, movegen::{magic::{BISHOP_MAGICS, MAGIC_BISHOP_SHIFT, MAGIC_ROOK_SHIFT, ROOK_MAGICS, get_bishop_attacks, get_rook_attacks}}};

    #[test]
    fn test_magic_vs_classic_consistency() {    
        const UP_RAYS: [u64; 64] = [
            72340172838076672,144680345676153344,289360691352306688,578721382704613376,
            1157442765409226752,2314885530818453504,4629771061636907008,9259542123273814016,
            72340172838076416,144680345676152832,289360691352305664,578721382704611328,
            1157442765409222656,2314885530818445312,4629771061636890624,9259542123273781248,
            72340172838010880,144680345676021760,289360691352043520,578721382704087040,
            1157442765408174080,2314885530816348160,4629771061632696320,9259542123265392640,
            72340172821233664,144680345642467328,289360691284934656,578721382569869312,
            1157442765139738624,2314885530279477248,4629771060558954496,9259542121117908992,
            72340168526266368,144680337052532736,289360674105065472,578721348210130944,
            1157442696420261888,2314885392840523776,4629770785681047552,9259541571362095104,
            72339069014638592,144678138029277184,289356276058554368,578712552117108736,
            1157425104234217472,2314850208468434944,4629700416936869888,9259400833873739776,
            72057594037927936,144115188075855872,288230376151711744,576460752303423488,
            1152921504606846976,2305843009213693952,4611686018427387904,9223372036854775808,
            0,0,0,0,
            0,0,0,0,
        ];
        const DOWN_RAYS: [u64; 64] = [
            0,0,0,0,
            0,0,0,0,
            1,2,4,8,
            16,32,64,128,
            257,514,1028,2056,
            4112,8224,16448,32896,
            65793,131586,263172,526344,
            1052688,2105376,4210752,8421504,
            16843009,33686018,67372036,134744072,
            269488144,538976288,1077952576,2155905152,
            4311810305,8623620610,17247241220,34494482440,
            68988964880,137977929760,275955859520,551911719040,
            1103823438081,2207646876162,4415293752324,8830587504648,
            17661175009296,35322350018592,70644700037184,141289400074368,
            282578800148737,565157600297474,1130315200594948,2260630401189896,
            4521260802379792,9042521604759584,18085043209519168,36170086419038336,
        ];
        const LEFT_RAYS: [u64; 64] = [
            0,1,3,7,
            15,31,63,127,
            0,256,768,1792,
            3840,7936,16128,32512,
            0,65536,196608,458752,
            983040,2031616,4128768,8323072,
            0,16777216,50331648,117440512,
            251658240,520093696,1056964608,2130706432,
            0,4294967296,12884901888,30064771072,
            64424509440,133143986176,270582939648,545460846592,
            0,1099511627776,3298534883328,7696581394432,
            16492674416640,34084860461056,69269232549888,139637976727552,
            0,281474976710656,844424930131968,1970324836974592,
            4222124650659840,8725724278030336,17732923532771328,35747322042253312,
            0,72057594037927936,216172782113783808,504403158265495552,
            1080863910568919040,2233785415175766016,4539628424389459968,9151314442816847872,
        ];
        const RIGHT_RAYS: [u64; 64] = [
            254,252,248,240,
            224,192,128,0,
            65024,64512,63488,61440,
            57344,49152,32768,0,
            16646144,16515072,16252928,15728640,
            14680064,12582912,8388608,0,
            4261412864,4227858432,4160749568,4026531840,
            3758096384,3221225472,2147483648,0,
            1090921693184,1082331758592,1065151889408,1030792151040,
            962072674304,824633720832,549755813888,0,
            279275953455104,277076930199552,272678883688448,263882790666240,
            246290604621824,211106232532992,140737488355328,0,
            71494644084506624,70931694131085312,69805794224242688,67553994410557440,
            63050394783186944,54043195528445952,36028797018963968,0,
            18302628885633695744,18158513697557839872,17870283321406128128,17293822569102704640,
            16140901064495857664,13835058055282163712,9223372036854775808,0,
        ];
        const SUPPLY_DIAGONAL_RAYS: [u64; 64] = [
            9241421688590303744,36099303471055872,141012904183808,550831656960,
            2151686144,8404992,32768,0,
            4620710844295151616,9241421688590303233,36099303471054850,141012904181764,
            550831652872,2151677968,8388640,64,
            2310355422147510272,4620710844295020800,9241421688590041601,36099303470531586,
            141012903135236,550829559816,2147491856,16416,
            1155177711056977920,2310355422114021376,4620710844228043008,9241421688456086017,
            36099303202620418,141012367312900,549757915144,4202512,
            577588851233521664,1155177702483820544,2310355404967706624,4620710809935413504,
            9241421619870827009,36099166032102402,140738026276868,1075843080,
            288793326105133056,577586656505233408,1155173313027244032,2310346626054553600,
            4620693252109107456,9241386504218214913,36028934726878210,275415828484,
            144115188075855872,288231475663339520,576462955621646336,1152925911260069888,
            2305851822520205312,4611703645040410880,9223407290080821761,70506452091906,
            0,281474976710656,564049465049088,1128103225065472,
            2256206466908160,4512412933881856,9024825867763968,18049651735527937,
        ];
        const DEMAND_DIAGONAL_RAYS: [u64; 64] = [
            0,256,66048,16909312,
            4328785920,1108169199616,283691315109888,72624976668147712,
            2,65540,16908296,4328783888,
            1108169195552,283691315101760,72624976668131456,145249953336262656,
            516,16778248,4328523792,1108168675360,
            283691314061376,72624976666050688,145249953332101120,290499906664136704,
            132104,4295231504,1108102090784,283691180892224,
            72624976399712384,145249952799424512,290499905598783488,580999811180789760,
            33818640,1099579265056,283674135240768,72624942308409472,
            145249884616818688,290499769233571840,580999538450366464,1161999072605765632,
            8657571872,281492291854400,72620578621636736,145241157243273216,
            290482314486480896,580964628956184576,1161929253617401856,2323857407723175936,
            2216338399296,72062026714726528,144124053429452800,288248106858840064,
            576496213700902912,1152992423106838528,2305983746702049280,4611686018427387904,
            567382630219904,1134765260439552,2269530520813568,4539061024849920,
            9078117754732544,18155135997837312,36028797018963968,0,
        ];
        fn get_up_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            let ray = UP_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            if blockers == 0 { return ray; }
            let mask_to_blocker = blockers ^ (blockers - 1);
            let attack_mask = ray & mask_to_blocker;

            attack_mask & !our_occupancy
        }
        fn get_down_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            let ray = DOWN_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            if blockers == 0 { return ray; }
            else {
                let first_blocker_square = 63 - blockers.leading_zeros();
                let mask_to_blocker = u64::MAX << first_blocker_square;
                let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
                return attack_mask & !our_occupancy;
            }
        }
        fn get_left_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            let ray = LEFT_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            if blockers == 0 { return ray }
            else {
                let first_blocker_square = 63 - blockers.leading_zeros();
                let mask_to_blocker = RIGHT_RAYS[first_blocker_square as usize] & ray;
                let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
                return attack_mask & !our_occupancy;
            }
        }
        fn get_right_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            let ray = RIGHT_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            if blockers == 0 { return ray; }
            else {
                let first_blocker_square = blockers.trailing_zeros();
                let mask_to_blocker = LEFT_RAYS[first_blocker_square as usize] & ray;
                let attack_mask = (ray & mask_to_blocker) | (1 << first_blocker_square);
                return attack_mask & !our_occupancy;
            }
        }
        fn get_supply_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            fn get_supply_positive_ray(square: usize) -> u64 {
                let ray = SUPPLY_DIAGONAL_RAYS[square];
                if square >= 63 {
                    0  // No positive ray from squares 63 and above
                } else {
                    ray & (u64::MAX.wrapping_shl((square + 1) as u32))
                }
            }
            fn get_supply_negative_ray(square: usize) -> u64 {
                let ray = SUPPLY_DIAGONAL_RAYS[square];
                ray & ((1 << square) - 1)
            }
            
            let ray = SUPPLY_DIAGONAL_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            let positive_ray = get_supply_positive_ray(square);
            let negative_ray = get_supply_negative_ray(square);

            // Up right (positive)
            let positive_blockers = blockers & positive_ray;
            let mut positive_attacks = positive_ray;
            if positive_blockers != 0 {
                let first_blocker_square = positive_blockers.trailing_zeros();
                let mask_to_blocker = positive_ray & get_supply_negative_ray(first_blocker_square as usize);
                positive_attacks = (positive_ray & mask_to_blocker) | (1 << first_blocker_square);
            }

            // Down left (negative)
            let negative_blockers = blockers & negative_ray;
            let mut negative_attacks = negative_ray;
            if negative_blockers != 0 {
                let first_blocker_square = 63 - negative_blockers.leading_zeros();
                let mask_to_blocker = negative_ray & get_supply_positive_ray(first_blocker_square as usize);
                negative_attacks = (negative_ray & mask_to_blocker) | (1 << first_blocker_square);
            }

            return (positive_attacks | negative_attacks) & !our_occupancy;
        }
        fn get_demand_moves(square: usize, our_occupancy: u64, their_occupancy: u64) -> u64 {
            fn get_demand_positive_ray(square: usize) -> u64 {
                let ray = DEMAND_DIAGONAL_RAYS[square];
                ray & u64::MAX.checked_shl((square + 1) as u32).unwrap_or(0)
            }
            fn get_demand_negative_ray(square: usize) -> u64 {
                let ray = DEMAND_DIAGONAL_RAYS[square];
                ray & ((1 << square) - 1)
            }

            let ray = DEMAND_DIAGONAL_RAYS[square];
            let blockers = ray & (our_occupancy | their_occupancy);

            let positive_ray = get_demand_positive_ray(square);
            let negative_ray = get_demand_negative_ray(square);
            
            // Up left (positive)
            let positive_blockers = blockers & positive_ray;
            let mut positive_attacks = positive_ray;
            if positive_blockers != 0 {
                let first_blocker_square = positive_blockers.trailing_zeros();
                let mask_to_blocker = positive_ray & get_demand_negative_ray(first_blocker_square as usize);
                positive_attacks = (positive_ray & mask_to_blocker) | (1 << first_blocker_square);
            }

            // Down right (negative)
            let negative_blockers = blockers & negative_ray;
            let mut negative_attacks = negative_ray;
            if negative_blockers != 0 {
                let first_blocker_square = 63 - negative_blockers.leading_zeros();
                let mask_to_blocker = negative_ray & get_demand_positive_ray(first_blocker_square as usize);
                negative_attacks = (negative_ray & mask_to_blocker) | (1 << first_blocker_square);
            }

            return (positive_attacks | negative_attacks) & !our_occupancy;
        }
        
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

        let mut ROOK_RAYS = [0; 64];
        let mut BISHOP_RAYS = [0; 64];
        for square in 0..64 {
            ROOK_RAYS[square] = get_rook_mask(square);
            BISHOP_RAYS[square] = get_bishop_mask(square);
        }

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
                    
                    let magic = get_rook_attacks(square, both_occ) & !our_occ;

                    assert_eq!(classic, magic, 
                        "\nFAIL: Orthogonal mismatch at FEN #{}\nFEN: {}\nSquare: {}\nPiece: {}\nClassic: {:064b}\nMagic:   {:064b}\n",
                        fen_idx, fen, square, piece, classic, magic
                    );
                }

                if p_type == PieceType::BISHOP || p_type == PieceType::QUEEN {
                    let classic = get_supply_moves(square, our_occ, their_occ) |
                                get_demand_moves(square, our_occ, their_occ);
                    
                    let magic = get_bishop_attacks(square, both_occ) & !our_occ;

                    assert_eq!(classic, magic, 
                        "\nFAIL: Diagonal mismatch at FEN #{}\nFEN: {}\nSquare: {}\nPiece: {}\nClassic: {:064b}\nMagic:   {:064b}\n",
                        fen_idx, fen, square, piece, classic, magic
                    );
                }
            }
        }
    }
}