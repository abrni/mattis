use crate::moves::Move16;

#[repr(packed)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpEntry {
    m: Move16,
    key: [u8; 5],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpTable {
    vec: Vec<TpEntry>,
    collisions: usize,
    filled: usize,
}

impl TpTable {
    pub fn new() -> Self {
        let size_b = 112 << 20; // size in bytes
        let len = size_b / std::mem::size_of::<TpEntry>();

        let vec = vec![
            TpEntry {
                m: Move16::build().finish(),
                key: [0; 5],
            };
            len
        ];

        assert!(len <= (1 << 24));
        assert_eq!(vec.len(), vec.capacity());

        Self {
            vec,
            collisions: 0,
            filled: 0,
        }
    }

    pub fn insert(&mut self, key: u64, m: Move16) {
        let idx = (key & 0xFFFFFF) as usize;
        let entry = unsafe { self.vec.get_unchecked_mut(idx) };
        let bytes = (key >> 24).to_le_bytes();

        if entry.key == [0, 0, 0, 0, 0] {
            self.filled += 1;
        } else if entry.key != bytes[0..5] {
            self.collisions += 1;
        }

        entry.key.copy_from_slice(&bytes[0..5]);
        entry.m = m;
    }

    pub fn get(&self, key: u64) -> Option<Move16> {
        let idx = (key & 0xFFFFFF) as usize;
        let entry = unsafe { self.vec.get_unchecked(idx) };
        let bytes = &(key >> 24).to_le_bytes()[0..5];

        if bytes == entry.key {
            Some(entry.m)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.filled
    }

    pub fn is_empty(&self) -> bool {
        self.filled == 0
    }

    pub fn capacity(&self) -> usize {
        self.vec.len()
    }

    pub fn collisions(&self) -> usize {
        self.collisions
    }

    pub fn fill_level(&self) -> f32 {
        self.len() as f32 / self.capacity() as f32
    }
}

impl Default for TpTable {
    fn default() -> Self {
        Self::new()
    }
}
