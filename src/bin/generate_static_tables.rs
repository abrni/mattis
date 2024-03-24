use mattis_bitboard::BitBoard;
use mattis_types::{File, Square};

fn main() {
    generate_file_bitboards();
}

fn generate_file_bitboards() {
    let mut boards = [BitBoard::EMPTY; 8];

    for f in File::iter_all() {
        for r in mattis_types::Rank::iter_all() {
            boards[f].set(Square::from_file_rank(f, r));
        }
    }

    let boards: [u8; std::mem::size_of::<BitBoard>() * 8] = unsafe { std::mem::transmute(boards) };
    std::fs::write("static_tables/file_bitboards", boards).unwrap();
}
