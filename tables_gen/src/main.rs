const OUT_DIR: &str = "../_static_tables";

macro_rules! run_gen {
    ($name:ident) => {{
        let result = $name();
        let bytes = bytemuck::bytes_of(&result);
        std::fs::write(format!("{OUT_DIR}/{}", stringify!($name)), bytes).unwrap();
    }};
}

fn main() {
    let dir_path = std::path::Path::new(OUT_DIR);
    if !dir_path.exists() {
        std::fs::create_dir(dir_path).unwrap();
    }

    {
        use tables_gen::*;
        run_gen!(file_bitboards);
        run_gen!(not_file_bitboards);
        run_gen!(rank_bitboards);
        run_gen!(not_rank_bitboards);
        run_gen!(border);
        run_gen!(white_pawn_passed_masks);
        run_gen!(black_pawn_passed_masks);
        run_gen!(isolated_pawn_masks);
        run_gen!(knight_move_patterns);
        run_gen!(king_move_patterns);
        run_gen!(rook_move_patterns);
        run_gen!(bishop_move_patterns);
    }
}
