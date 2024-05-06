use std::process::Command;

fn main() {
    let _ = Command::new("cargo")
        .args(["run", "--target-dir", "../temp_target"])
        .current_dir("./tables_gen")
        .output()
        .unwrap();

    println!("cargo:rerun-if-changed=tables_gen");
    std::fs::remove_dir_all("./temp_target").unwrap();
}
