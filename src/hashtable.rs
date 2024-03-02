use crate::{
    board::Board,
    moves::{Move16, Move32},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HEKind {
    #[default]
    Exact,
    Alpha,
    Beta,
}

pub enum Probe {
    NoHit,               // We have no hit in the table
    PV(Move32, i32),     // We do have a hit in the table, but it is not exact and does not cause a branch cutoff
    CutOff(Move32, i32), // We have a successful hit, that was exact or causes a branch cutoff
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Entry {
    pub position_key: u64,
    pub score: i32,
    pub m16: Move16,
    pub depth: u8,
    pub kind: HEKind,
}

pub struct TranspositionTable {
    data: Box<[Entry]>,
    capacity: usize,
    shift: u32,
    filled: usize,
    overwrites: usize,
    probe_hits: usize,
    probe_successes: usize,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        if size_mb == 0 {
            panic!("Cannot create a zero sized hashtable")
        }

        let size_mb = size_mb.next_power_of_two();
        let size_b = size_mb * 1024 * 1024;
        let entry_size = std::mem::size_of::<Entry>();
        let capacity = size_b / entry_size;
        let shift = 64 - capacity.trailing_zeros();

        let mut data = Vec::new();
        data.reserve_exact(capacity);
        data.resize(capacity, Default::default());

        let data = data.into_boxed_slice();

        Self {
            data,
            capacity,
            shift,
            filled: 0,
            overwrites: 0,
            probe_hits: 0,
            probe_successes: 0,
        }
    }

    fn index(&self, key: u64) -> usize {
        (key >> self.shift) as usize
    }

    pub fn store(&mut self, entry: Entry) {
        let index = self.index(entry.position_key);
        debug_assert!(index < self.capacity);

        // TODO: check if mate??? siehe VICE

        // SAFETY: Our index is always in range.
        let table_entry = unsafe { self.data.get_unchecked_mut(index) };

        if table_entry.position_key == 0 {
            self.filled += 1;
        } else {
            self.overwrites += 1;
        }

        *table_entry = entry;
    }

    pub fn get(&self, key: u64) -> Option<Entry> {
        let index = self.index(key);

        // SAFETY: Our index is always in range.
        let entry = unsafe { *self.data.get_unchecked(index) };

        if entry.position_key == key {
            Some(entry)
        } else {
            None
        }
    }

    pub fn probe(&mut self, board: &Board, alpha: i32, beta: i32, depth: u8) -> Probe {
        let Some(entry) = self.get(board.position_key) else { return Probe::NoHit };

        let m16 = entry.m16;
        let m32 = board.move_16_to_32(m16);
        let score = entry.score;

        if entry.depth < depth {
            return Probe::PV(m32, score);
        }

        debug_assert!(entry.depth >= 1);
        self.probe_hits += 1;

        // TODO: ADJUST SCORE if it indicates a mate
        // if score > ISMATE {score -= board.ply as i32;  }
        // else if score < -ISMATE {score += board.ply as i32; }
        // debug_assert!(score >= -INFINITE && score <= INFINITE);

        let probe_res = match entry.kind {
            HEKind::Alpha if score <= alpha => Probe::CutOff(m32, alpha),
            HEKind::Beta if score >= beta => Probe::CutOff(m32, beta),
            HEKind::Exact => Probe::CutOff(m32, score),
            _ => Probe::PV(m32, score),
        };

        if matches!(probe_res, Probe::CutOff(..)) {
            self.probe_successes += 1;
        }

        probe_res
    }
}

#[cfg(test)]
mod test {
    use crate::hashtable::{Entry, TranspositionTable};

    #[test]
    fn size_of_entry() {
        let entry = Entry::default();

        assert_eq!(std::mem::size_of_val(&entry), 16);
        assert_eq!(std::mem::align_of_val(&entry), 8);
    }

    #[test]
    fn size_of_new_table() {
        for size_mb in [2, 8, 32, 128, 512] {
            let table = TranspositionTable::new(size_mb);
            let byte_size = size_mb * 1024 * 1024;
            let data = &*table.data;
            assert_eq!(std::mem::size_of_val(data), byte_size);
            assert_eq!(table.capacity, table.data.len());
            assert_eq!(table.capacity * std::mem::size_of::<Entry>(), byte_size);
        }
    }

    #[test]
    #[should_panic]
    fn try_create_zero_sized() {
        TranspositionTable::new(0);
    }

    #[test]
    fn store_any_key() {
        for size_mb in [2, 4, 8, 16, 32] {
            let mut table = TranspositionTable::new(size_mb);

            for _ in 0..table.capacity {
                let entry = Entry {
                    position_key: rand::random(),
                    ..Default::default()
                };

                table.store(entry);
            }

            println!(
                "table of size {size_mb}MB: {} capacity, {} entries, {} overwrites",
                table.capacity, table.filled, table.overwrites
            );
        }
    }
}
