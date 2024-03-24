use std::process::Command;

fn main() {
    let _ = Command::new("cargo")
        .args(["run", "--target-dir", "./target"])
        .current_dir("./tables_gen")
        .output()
        .unwrap();
}
