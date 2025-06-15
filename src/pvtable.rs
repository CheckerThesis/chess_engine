
use std::sync::{atomic::{AtomicU64, AtomicU8, Ordering}, Arc, LazyLock};

use colored::Colorize;

use crate::{board::Board, defs::{AB_BOUND, DEBUG, INF_BOUND, IS_MATE, MAX_DEPTH, NO_MOVE}, makemove::{make_move, take_move}, movegen::move_exists};

pub const TT_SIZE: i32 = 8388608;
pub static HASH_TABLE: LazyLock<Arc<HashTable>> = LazyLock::new(|| Arc::new(HashTable::default()));

pub fn extract_score(data: u64) -> i32 { (data & 0xFFFF) as i32 - INF_BOUND as i32 }
pub fn extract_depth(data: u64) -> u64 { (data >> 16) & 0x3F }
pub fn extract_flags(data: u64) -> u64 { (data >> 23) & 0x3 }
pub fn extract_move(data: u64) -> u64 { data >> 25 }
pub fn fold_data(score: i32, depth: u64, flags: u64, the_move: u32) -> u64 { (score + INF_BOUND as i32) as u64 | (depth << 16) | (flags << 23) | ((the_move as u64) << 25) }
#[derive(Default)]
pub struct HashEntry {
    position_key: AtomicU64,
    data: AtomicU64,
    age: AtomicU8,
}
/**
Transposition table, stores previously computed positions.

The `HashTable` stores `HashEntry` structs which consist of a position's:
- Position key (for use in alpha-beta, checking if the position key of the current position is already in the `HashTable`).
- Data (score, depth, flags, move).
- Age of the entry.

# Fields
- `pv_table` - A vector of `HashEntry` representing the hash table entries.
- `new_write` - Counter for newly added entries.
- `over_write` - Counter for overwritten entries.
- `hit` - Counter for successful lookups.
- `cut` - Counter for search cutoffs based on stored entries.
- `current_age` - Current age of table to help in replacing old data.

# Usage
The `HashTable` is used to store and retrieve search data for game positions, avoiding redundent computations. In our case, it's used at the beginning of alpha-beta.
*/
pub struct HashTable {
    pv_table: Vec<HashEntry>,
    new_write: AtomicU64,
    over_write: AtomicU64,
    hit: AtomicU64,
    cut: AtomicU64,
    current_age: AtomicU8,
}
impl HashTable {
    fn default() -> Self {
        HashTable {
            pv_table: (0..TT_SIZE).map(|_| HashEntry::default()).collect(),
            new_write: AtomicU64::new(0),
            over_write: AtomicU64::new(0),
            hit: AtomicU64::new(0),
            cut: AtomicU64::new(0),
            current_age: AtomicU8::new(0),
        }
    }

    /**
    Stores new entry in `HashTable`.

    # Parameters
    - `position`: Reference to position in the current alpha-beta.
    - `the_move`: Move from `pick_next_move` function.
    - `score`: Mutable reference to the score for a position.
    - `flags`: Hash flags for this `HashEntry`.
    - `depth`: Depth of current search for given position.

    # Logic
    1. Check if replace is necessary (if index is empty, age is younger than stored, deeper depth than stored).
    2. Adjust score for mate.
    */
    pub fn store_hash_entry(&self, position: &Board, the_move: u32, score: &mut i32, flags: u8, depth: i32) {
        let data = fold_data(*score, depth as u64, flags as u64, the_move);
        let i = position.position_key as usize % self.pv_table.capacity();

        if DEBUG {
            if i > self.pv_table.capacity() - 1 { eprintln!("{}", "store_hash_entry: [i] out of bounds".red()); }
            if depth > MAX_DEPTH as i32 || depth < 1 { eprintln!("{}", "store_hash_entry: [depth] out of bounds".red()); }
            if position.ply >= MAX_DEPTH as u8 { eprintln!("{}", "store_hash_entry: [ply] out of bounds".red()); }
        }

        let mut replace = false;
        if self.get_position_key(i) == 0 {
            self.increment_new_write();
            replace = true;
        } else {
            self.increment_over_write();
            if self.get_age(i) < self.get_current_age() || extract_depth(self.get_data(i)) <= depth as u64 { replace = true }
        }
        // if self.get_position_key(i) == 0 {
        //     self.increment_new_write();
        //     replace = true;
        // } else {
        //     self.increment_over_write();
        //     if extract_depth(self.get_data(i)) < depth as u64 { replace = true }
        //     else if extract_depth(self.get_data(i)) == depth as u64 && self.get_age(i) < self.get_current_age() { replace = true }
        // }

        if replace == false { return; }

        // IS_MATE is mate found for white, -IS_MATE is mate found for black, add or subtract
        // ply to get the exact amount of moves for mate
        if *score > IS_MATE { *score += position.ply as i32 } 
        else if *score < -IS_MATE { *score -= position.ply as i32; }

        self.set_position_key(i, position.position_key);
        self.set_data(i, data);
        self.set_age(i, self.get_current_age());
    }

    /**
    Check if table has an entry that matches the current position.

    # Parameters
    - `position`: Used to hash the `position.position_key`.
    - `the_move`: Stores the move stored in the hash table.
    - `score`: Score we change inside the function.
    - `alpha`: Determine whether the position has been hashed inside our table.
    - `beta`: Determine whether the position has been hashed inside our table.
    - `depth`: Check what depth we're looking for.
    ## Returns
    Boolean dependent on if the search-info is stored in the table.

    # Logic
    1.
    */
    pub fn probe_hash_table(&self, position: &Board, the_move: &mut u32, score: &mut i32, alpha: i32, beta: i32, depth: i32) -> bool {
        let i = position.position_key as usize % self.pv_table.capacity();

        if DEBUG {
            if depth > MAX_DEPTH as i32 || depth < 1 { eprintln!("{}", "probe_hash_table: [depth] out of bounds".red()); }
            if alpha >= beta { eprintln!("{}", "probe_hash_table: [alpha] greater than beta".red()); }
            if alpha > AB_BOUND || alpha < -AB_BOUND { eprintln!("{}", "probe_hash_table: [alpha] out of bounds".red()); }
            if beta > AB_BOUND || beta < -AB_BOUND { eprintln!("{}", "probe_hash_table: [beta] out of bounds".red()); }
        }

        // Verify this is the same position
        if self.get_position_key(i) == position.position_key {
            *the_move = extract_move(self.get_data(i)) as u32;

            // If depth is at least as deep as current search
            if extract_depth(self.get_data(i)) >= depth as u64 {
                self.increment_hit();

                // If score represents mate, adjust in terms of how many moves until mate
                *score = extract_score(self.get_data(i));
                if *score > IS_MATE { *score -= position.ply as i32; }
                else if *score < -IS_MATE { *score += position.ply as i32; }

                // if it is a cutoff
                match extract_flags(self.get_data(i)) as u8 {
                    x if x == HashFlag::HashFlagAlpha as u8 => {
                        if *score <= alpha {
                            *score = alpha;
                            return true
                        }
                    },
                    x if x == HashFlag::HashFlagBeta as u8 => {
                        if *score >= beta {
                            *score = beta;
                            return true
                        }
                    },
                    x if x == HashFlag::HashFlagExact as u8 => return true,
                    _ => return false
                }
            }
        }

        false
    }

    pub fn probe_pv_move(&self, position: &Board) -> u32 {
        let i = position.position_key as usize % self.pv_table.capacity();

        // if DEBUG && i < 0 || i > position.hash_table.pv_table.capacity() - 1 { eprintln!("{}", "probe_pv_move: [i] out of bounds".red()); }

        if self.get_position_key(i) == position.position_key { return extract_move(self.get_data(i)) as u32 }
        return NO_MOVE
    }

    pub fn clear(&self) {
        for i in 0..TT_SIZE {
            self.set_position_key(i as usize, 0);
            self.set_data(i as usize, 0);
            self.set_age(i as usize, 0);
        }
    }

    pub fn get_position_key(&self, i: usize) -> u64 { self.pv_table[i].position_key.load(Ordering::Relaxed) }
    pub fn set_position_key(&self, i: usize, x: u64) { self.pv_table[i].position_key.store(x, Ordering::Relaxed); }

    pub fn get_data(&self, i: usize) -> u64 { self.pv_table[i].data.load(Ordering::Relaxed) }
    pub fn set_data(&self, i: usize, x: u64) { self.pv_table[i].data.store(x, Ordering::Relaxed); }

    pub fn get_age(&self, i: usize) -> u8 { self.pv_table[i].age.load(Ordering::Relaxed) }
    pub fn set_age(&self, i: usize, x: u8) { self.pv_table[i].age.store(x, Ordering::Relaxed); }

    pub fn get_new_write(&self) -> u64 { self.new_write.load(Ordering::Relaxed) }
    pub fn set_new_write(&self, x: u64) { self.new_write.store(x, Ordering::Relaxed); }
    pub fn increment_new_write(&self) { self.new_write.fetch_add(1, Ordering::Relaxed); }

    pub fn get_over_write(&self) -> u64 { self.over_write.load(Ordering::Relaxed) }
    pub fn set_over_write(&self, x: u64) { self.over_write.store(x, Ordering::Relaxed); }
    pub fn increment_over_write(&self) { self.over_write.fetch_add(1, Ordering::Relaxed); }

    pub fn get_hit(&self) -> u64 { self.hit.load(Ordering::Relaxed) }
    pub fn set_hit(&self, x: u64) { self.hit.store(x, Ordering::Relaxed); }
    pub fn increment_hit(&self) { self.hit.fetch_add(1, Ordering::Relaxed); }

    pub fn get_cut(&self) -> u64 { self.cut.load(Ordering::Relaxed) }
    pub fn set_cut(&self, x: u64) { self.cut.store(x, Ordering::Relaxed); }
    pub fn increment_cut(&self) { self.cut.fetch_add(1, Ordering::Relaxed); }

    pub fn get_current_age(&self) -> u8 { self.current_age.load(Ordering::Relaxed) }
    pub fn set_current_age(&self, x: u8) { self.current_age.store(x,Ordering::Relaxed) }
    pub fn increment_current_age(&self) { self.current_age.fetch_add(1, Ordering::Relaxed); }
}

/**
- HashFlagAlpha:
*/
pub enum HashFlag {
    HashFlagNone,
    HashFlagAlpha,
    HashFlagBeta,
    HashFlagExact,
}

/*
pub fn data_check(the_move: u32) {
    let mut rng = thread_rng();
    let depth = rng.gen_range(0..1000) % MAX_DEPTH;
    let flags = rng.gen_range(0..1000) % 3;
    let score = rng.gen_range(-300..300) % AB_BOUND;

    let data = fold_data(score, depth as u64, flags as u64, the_move);

    println!("Original: move: {}  depth: {}  flags: {}  score: {}", print_move(the_move), depth, flags, score);
    println!("Folded:   move: {}  depth: {}  flags: {}  score: {}\n", print_move(extract_move(data) as u32), extract_depth(data), extract_flags(data), extract_score(data));
}

pub fn hash_test(fen: String) {
    let mut position = Board::default();
    parse_fen(&fen, &mut position);

    let move_list = &mut MoveList::default();
    generate_all_moves(&mut position, move_list);

    for move_number in 0..move_list.count {
        if !make_move(&mut position, extract_movelist_move(move_list.moves[move_number])) { continue; }

        take_move(&mut position);
        data_check(extract_movelist_move(move_list.moves[move_number]));
    }
}
    */

/**
Retrieves the Principal Variation (PV) line from the `HashTable` for the given position.

The PV is the sequence of moves considered to be best for both sides. This function iteratively probes the hash table to reconstruct this line.

# Parameters
- `depth`: The maximum number of moves to retrieve for the PV line.
- `position`: A mutable reference to the `Board` struct. The board state is temporarily modified as moves are made to trace the line.
- `hash_table`: A reference to the `HashTable`, which stores the PV moves found during the search.

# Returns
The number of moves in the PV line that were successfully retrieved and stored in `position.pv_array`.
*/
pub fn get_pv_line(depth: u8, position: &mut Board, hash_table: &HashTable) -> usize {
    if DEBUG && (depth > MAX_DEPTH as u8 || depth < 1) { eprintln!("{}", "get_pv_line: [depth] greater than MAX_DEPTH or less than 1".red()); }

    let mut count: usize = 0;
    let mut the_move = hash_table.probe_pv_move(&position);

    while the_move != NO_MOVE && count < depth as usize {
        if DEBUG && count > MAX_DEPTH { eprintln!("{}", "get_pv_line: [count] greater than MAX_DEPTH".red()); }

        if move_exists(position, the_move) {
            make_move(position, the_move);
            position.pv_array[count] = the_move;
            count += 1;
        } else { break; }

        the_move = hash_table.probe_pv_move(&position);
    }

    while position.ply > 0 { take_move(position); }

    count
}

pub fn clear_hash_table(hash_table: &HashTable) {
    hash_table.clear();
    hash_table.set_new_write(0);
    hash_table.set_over_write(0);
    hash_table.set_current_age(0);
    hash_table.set_cut(0);
}
