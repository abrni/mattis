use crate::{board::Board, chess_move::ChessMove};
use mattis_types::Eval;
use smallvec::SmallVec;
use std::sync::atomic::{AtomicU64, AtomicU8, Ordering};

pub type PrincipalVariation = SmallVec<[ChessMove; 10]>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntryType {
    #[default]
    Exact,
    Alpha,
    Beta,
}

pub enum Probe {
    NoHit,         // We have no hit in the table
    Pv(ChessMove), // We do have a hit in the table, but it is not exact and does not cause a branch cutoff
    CutOff(Eval),  // We have a successful hit, that was exact or causes a branch cutoff
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
    pub kind: EntryType,
    pub age: u8,
}

impl Entry {
    /// Stores the given data in the the table entry.
    fn store(&self, key: u64, data: Data) {
        // Safety: Transmuting to a u64 is fine, because `Data` has a size of exactly 64 bits.
        let data: u64 = unsafe { std::mem::transmute(data) };
        let key = data ^ key;

        self.key.store(key, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }

    /// Try to load the data from the table entry.
    ///
    /// Loading fails, if the given `key` does not match the stored key.
    /// In that case, the entry is either empty or contains data associated with a different key.
    fn load(&self, key: u64) -> Option<Data> {
        // Load both key and data and decode the key.
        let encoded_key = self.key.load(Ordering::Relaxed);
        let data = self.data.load(Ordering::Relaxed);
        let decoded_key = encoded_key ^ data;

        if decoded_key == key {
            // Safety: We only transmute after checking the decoded key.
            // if the key doesn't match, we do not transmute and return `None` instead.
            let data: Data = unsafe { std::mem::transmute(data) };
            Some(data)
        } else {
            None
        }
    }
}

pub struct TranspositionTable {
    data: Box<[Entry]>,
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
            shift,
            current_age: AtomicU8::new(0),
        }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn reset(&self) {
        self.current_age.store(0, Ordering::Relaxed);
        for entry in self.data.iter() {
            entry.key.store(0, Ordering::Relaxed);
            entry.data.store(0, Ordering::Relaxed);
        }
    }

    #[inline(always)]
    fn index(&self, key: u64) -> usize {
        (key >> self.shift) as usize
    }

    #[inline(always)]
    fn entry(&self, key: u64) -> &Entry {
        let index = self.index(key);

        // Safety: index is always in range
        unsafe { self.data.get_unchecked(index) }
    }

    #[inline(always)]
    pub fn load(&self, key: u64) -> Option<Data> {
        self.entry(key).load(key)
    }

    #[inline(always)]
    pub fn load_move(&self, key: u64) -> Option<ChessMove> {
        self.load(key).map(|data| data.cmove)
    }

    pub fn store(&self, board: &Board, score: Eval, cmove: ChessMove, depth: u16, kind: EntryType) {
        // Load currently stored data
        let table_entry = self.entry(board.position_key);
        let entry_data = table_entry.load(board.position_key);
        let current_table_age = self.current_age.load(Ordering::Relaxed);

        // Its possible, that we encounter hash collisions. We do not override the existing entry if:
        // - the existing entry contains valid data (i.e. it is not corrupted)
        // - and this data is from the current table age
        // - and this data contains a move from a higher search depth than we are trying to store
        //   (i.e. the existing move is more acurate)
        if entry_data.is_some_and(|data| data.age == current_table_age && data.depth > depth) {
            return;
        }

        // Adjust the score, if its a mate score.
        // The mate score is always relative to the root position (i.e. how many moves away from the root).
        // That also means, the current ply does not necesarily match the mate score
        // (e.g. we might store a score of 'mate in 6 ply' at depth 4).
        // When we later access this data, we might
        //     a) have a different root position or
        //     b) be at a different position in the search tree (the same position can be reached at different ply).
        // This means we have to adjust the mate score before storing and after loading it, to ensure accuracy.
        let score = if score.is_mate() {
            score + board.ply as i16 * score.inner().signum()
        } else {
            score
        };

        let new_data = Data {
            score,
            cmove,
            depth,
            kind,
            age: current_table_age,
        };

        table_entry.store(board.position_key, new_data);
    }

    pub fn probe(&self, board: &Board, alpha: Eval, beta: Eval, depth: u16) -> Probe {
        // Try to load data from the table.
        // Loading `None` means, either there is no data or the data has been corrupted.
        // Either way, we just return `NoHit`.
        let Some(data) = self.load(board.position_key) else { return Probe::NoHit };

        // If the stored data is from a lower depth than we are requesting, it cannot be used for a branch-cutoff.
        // Just return the move as a pv move for move ordering.
        if data.depth < depth {
            return Probe::Pv(data.cmove);
        }

        // Adjust the score, if its a mate score.
        // See the corresponding comment in the `store`-function for an explanation.
        let score = if data.score.is_mate() {
            data.score - board.ply as i16 * data.score.inner().signum()
        } else {
            data.score
        };

        // Depending on the entry kind, we return a pv move or a cutoff. Exact entrys can always yield a cutoff.
        // Alpha and beta entries only yield cutoffs, if the score is outside the corresponding bound.
        match data.kind {
            EntryType::Alpha if score <= alpha => Probe::CutOff(alpha),
            EntryType::Beta if score >= beta => Probe::CutOff(beta),
            EntryType::Exact => Probe::CutOff(score),
            _ => Probe::Pv(data.cmove),
        }
    }

    pub fn next_age(&self) {
        self.current_age.fetch_add(1, Ordering::Relaxed);
    }

    pub fn pv(&self, board: &mut Board, depth: usize, first: Option<ChessMove>) -> PrincipalVariation {
        let mut pv = PrincipalVariation::new();

        if depth == 0 {
            return pv;
        }

        if let Some(first) = first {
            pv.push(first);
            assert!(board.make_move(first), "Invalid first move in pv line");
        }

        while pv.len() < depth {
            let Some(m) = self.load_move(board.position_key) else { break };
            board.make_move(m);
            pv.push(m);
        }

        for _ in 0..pv.len() {
            board.take_move();
        }

        pv
    }
}

#[cfg(test)]
mod test {
    use super::EntryType;
    use crate::{
        board::Board,
        chess_move::ChessMove,
        hashtable::{Data, Entry, TranspositionTable},
    };
    use mattis_types::Eval;

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
            assert_eq!(table.len(), table.data.len());
            assert_eq!(table.len() * std::mem::size_of::<Entry>(), byte_size);
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
            let board = Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();

            for _ in 0..table.len() {
                table.store(
                    &board,
                    Eval::default(),
                    ChessMove::default(),
                    u16::default(),
                    EntryType::default(),
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
            kind: EntryType::Alpha,
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
            kind: EntryType::Alpha,
            age: rand::random(),
        };

        let entry = Entry::default();
        entry.store(key1, data);
        assert_eq!(entry.load(key2), None);
    }
}
