# 🧩 Minisudoku

A simple but performant Sudoku solver written in pure Rust with a sleek interactive TUI built with [ratatui](https://ratatui.rs).
It can solve puzzles of any difficulty, including ill-posed grids with multiple solutions in milliseconds.

## ⚙️ Installation & Usage

**Prerequisites:** Ensure you have the [Rust toolchain](https://rustup.rs/) installed.

Clone the repository and enter the directory:

```bash
git clone [https://github.com/haguettaz/propadoku.git](https://github.com/haguettaz/propadoku.git)
cd propadoku
```

### Option 1: Run directly with Cargo

For maximum performance, it is highly recommended to run the program in release mode

```bash
cargo run --release
```

### Option 2: Install globally to PATH

Install the binary into Cargo's global binary folder (`~/.cargo/bin`):
```bash
cargo install --path .
```

You can now launch the solver from any terminal directory:
```bash
mini-sudoku
```

### Input Format

The solver reads `.txt` files. 
Represent the grid using numbers 1-9 for known values and empty spaces for unknowns, separated by commas. 
Example grids are included in the `examples/` directory.

## ⚠️ Disclaimer

This project was built primarily for educational purposes rather than to chase raw speed records. 
If you are looking for SOTA performance, check out the [tdoku project and blog post](https://t-dillon.github.io/tdoku/).
