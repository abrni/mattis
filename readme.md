# Mattis Chess Engine
Mattis is a UCI (Universal Chess Interface) chess engine written in Rust. 
It is heavily inspired by the [Vice Chess Engine](https://github.com/bluefeversoft/vice).

## Features

- Alpha-Beta Search with Iterative Deepening
- Quiescence Search
- Move Generation using Magic Bitboards
- Null Move Pruning
- Transposition Table
- LazySMP
- MVV/LVA Move Ordering
- Search Killer and Search Histroy Heuristics
- Basic Evaluation using Piece-Square-Tables

You can learn about these features on the [Chess Programming Wiki](https://www.chessprogramming.org)

## Building/Running Mattis
Instructions on how to install Rust can be found [here](https://www.rust-lang.org/tools/install).

Compile and run the engine using Cargo:

```bash
cargo build --release
./target/release/mattis
```
or
```bash
cargo run --release
```

You can interact with the engine using a UCI-compatible chess GUI such as Arena.
