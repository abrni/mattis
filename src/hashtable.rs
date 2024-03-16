use crate::{board::Board, chess_move::ChessMove, eval::Eval};
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HEKind {
    #[default]
    Exact,
    Alpha,
    Beta,
}

pub enum Probe {
    NoHit,                   // We have no hit in the table
    PV(ChessMove, Eval),     // We do have a hit in the table, but it is not exact and does not cause a branch cutoff
    CutOff(ChessMove, Eval), // We have a successful hit, that was exact or causes a branch cutoff
}

#[derive(Debug, Default)]
struct Entry {
    key: AtomicU64,
    data: AtomicU64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Data {
    pub score: Eval,
    pub cmove: ChessMove,
    pub depth: u16,
    pub kind: HEKind,
    pub age: u8,
}

impl Entry {
    fn store(&self, key: u64, data: Data) {
        let data: u64 = unsafe { std::mem::transmute(data) };
        let key = data ^ key;

        self.key.store(key, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }

    fn load(&self, key: u64) -> Option<Data> {
        let encoded_key = self.key.load(Ordering::Relaxed);
        let data = self.data.load(Ordering::Relaxed);

        if encoded_key ^ data == key {
            let data: Data = unsafe { std::mem::transmute(data) };
            Some(data)
        } else {
            None
        }
    }
}

pub struct TranspositionTable {
    data: Box<[Entry]>,
    capacity: usize,
    shift: u32,
    current_age: AtomicU8,
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        assert!(size_mb != 0, "Cannot create a zero sized hashtable");

        let size_mb = size_mb.next_power_of_two();
        let byte_size = size_mb * 1024 * 1024;
        let entry_size = std::mem::size_of::<Entry>();
        let capacity = byte_size / entry_size;
        let shift = 64 - capacity.trailing_zeros();

        let mut data = Vec::with_capacity(capacity);
        data.resize_with(capacity, Default::default);
        let data = data.into_boxed_slice();

        Self {
            data,
            capacity,
            shift,
            current_age: AtomicU8::new(0),
        }
    }

    pub fn reset(&self) {
        self.current_age.store(0, Ordering::Relaxed);
        for entry in self.data.iter() {
            entry.key.store(0, Ordering::Relaxed);
            entry.data.store(0, Ordering::Relaxed);
        }
    }

    fn index(&self, key: u64) -> usize {
        (key >> self.shift) as usize
    }

    pub fn store(&self, position_key: u64, score: Eval, m: ChessMove, depth: u16, kind: HEKind) {
        let current_table_age = self.current_age.load(Ordering::Relaxed);
        let index = self.index(position_key);
        debug_assert!(index < self.capacity);

        // TODO: check if mate??? siehe VICE

        // SAFETY: Our index is always in range.
        let table_entry = unsafe { self.data.get_unchecked(index) };
        let entry_key = table_entry.key.load(Ordering::Relaxed);
        let entry_data = table_entry.data.load(Ordering::Relaxed);
        let entry_data: Data = unsafe { std::mem::transmute(entry_data) };

        // Check if it makes sense to store the move.
        // The old entry can be replaced if any of the following conditions is true:
        // - the old entry has never been written to
        // - TODO: the old entry is corrupted by a data race
        // - the old entry is not from the current age
        // - the old entry has a lower depth than we are trying to write
        let replace = entry_key == 0 || entry_data.age != current_table_age || entry_data.depth <= depth;

        if !replace {
            return;
        }

        let new_data = Data {
            score,
            cmove: m,
            depth,
            kind,
            age: current_table_age,
        };

        table_entry.store(position_key, new_data);
    }

    pub fn load(&self, key: u64) -> Option<Data> {
        let index = self.index(key);
        let entry = unsafe { self.data.get_unchecked(index) };
        entry.load(key)
    }

    pub fn get(&self, key: u64) -> Option<ChessMove> {
        self.load(key).map(|data| data.cmove)
    }

    pub fn probe(&self, board: &Board, alpha: Eval, beta: Eval, depth: u16) -> Probe {
        let Some(data) = self.load(board.position_key) else { return Probe::NoHit };

        let cmove = data.cmove;
        let score = data.score;

        if data.depth < depth {
            return Probe::PV(cmove, score);
        }

        debug_assert!(data.depth >= 1);

        match data.kind {
            HEKind::Alpha if score <= alpha => Probe::CutOff(cmove, alpha),
            HEKind::Beta if score >= beta => Probe::CutOff(cmove, beta),
            HEKind::Exact => Probe::CutOff(cmove, score),
            _ => Probe::PV(cmove, score),
        }
    }

    pub fn next_age(&self) {
        self.current_age.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod test {
    use super::HEKind;
    use crate::{
        chess_move::ChessMove,
        eval::Eval,
        hashtable::{Data, Entry, TranspositionTable},
    };

    #[test]
    fn size_of_entry() {
        let entry = Entry::default();
        let data = Data::default();

        assert_eq!(std::mem::size_of_val(&data), 8);
        assert_eq!(std::mem::size_of_val(&entry), 16);
        // assert_eq!(std::mem::align_of_val(&entry), 8);
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
    #[should_panic = "Cannot create a zero sized hashtable"]
    fn try_create_zero_sized() {
        TranspositionTable::new(0);
    }

    #[test]
    fn store_any_key() {
        for size_mb in [2, 4, 8, 16, 32] {
            let table = TranspositionTable::new(size_mb);

            for _ in 0..table.capacity {
                table.store(
                    rand::random(),
                    Eval::default(),
                    ChessMove::default(),
                    u16::default(),
                    HEKind::default(),
                );
            }
        }
    }

    #[test]
    fn encode_decode_entry() {
        let key: u64 = rand::random();
        let data = Data {
            score: rand::random(),
            cmove: ChessMove::default(),
            depth: rand::random(),
            kind: HEKind::Alpha,
            age: rand::random(),
        };

        let entry = Entry::default();
        entry.store(key, data);
        let Some(loaded_data) = entry.load(key) else { panic!("Could not load data.") };
        assert_eq!(data, loaded_data);
    }

    #[test]
    fn decode_entry_with_different_key() {
        let key1: u64 = rand::random();
        let key2: u64 = rand::random();

        let data = Data {
            score: rand::random(),
            cmove: ChessMove::default(),
            depth: rand::random(),
            kind: HEKind::Alpha,
            age: rand::random(),
        };

        let entry = Entry::default();
        entry.store(key1, data);
        assert_eq!(entry.load(key2), None);
    }
}
