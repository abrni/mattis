use crate::chess_move::ChessMove;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SearchKillers(Box<[[ChessMove; 2]]>);

impl SearchKillers {
    pub fn new(size: usize) -> Self {
        Self(vec![Default::default(); size].into_boxed_slice())
    }

    pub fn slot1(&self, ply: usize) -> ChessMove {
        self.0[ply][0]
    }

    pub fn slot2(&self, ply: usize) -> ChessMove {
        self.0[ply][1]
    }

    pub fn store(&mut self, ply: usize, m: ChessMove) {
        self.0[ply][1] = self.0[ply][0];
        self.0[ply][0] = m;
    }
}

impl Default for SearchKillers {
    fn default() -> Self {
        Self::new(1024)
    }
}
