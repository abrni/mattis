use crate::moves::Move16;

#[repr(packed)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpEntry {
    m: Move16,
    key: [u8; 5],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TpTable {
    pub vec: Vec<TpEntry>,
}

impl TpTable {
    pub fn new() -> Self {
        let size_b = 112 << 20; // size in bytes
        let len = size_b / std::mem::size_of::<TpEntry>();

        assert!(len <= (1 << 24));

        Self {
            vec: vec![
                TpEntry {
                    m: Move16::build().finish(),
                    key: [0; 5]
                };
                len
            ],
        }
    }

    pub fn insert(&mut self, key: u64, m: Move16) {
        let idx = (key & 0xFFFFFF) as usize;
        let entry = unsafe { self.vec.get_unchecked_mut(idx) };
        let bytes = (key >> 24).to_le_bytes();

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
}

impl Default for TpTable {
    fn default() -> Self {
        Self::new()
    }
}
