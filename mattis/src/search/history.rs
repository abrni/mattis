use mattis_types::{Piece, Square};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SearchHistory([[u64; 64]; 12]);

impl SearchHistory {
    pub fn entry(&self, piece: Piece, square: Square) -> u64 {
        self.0[piece][square]
    }

    pub fn entry_mut(&mut self, piece: Piece, square: Square) -> &mut u64 {
        &mut self.0[piece][square]
    }
}

impl Default for SearchHistory {
    fn default() -> Self {
        Self([[0; 64]; 12])
    }
}
