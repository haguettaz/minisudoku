# Propadoku: A Sudoku Solver

A high-performance Sudoku solver implemented in Rust, utilizing **iterative constraint propagation** to efficiently navigate the search space.
This solver is designed to handle grids of any difficulty, including ill-posed puzzles with multiple solutions.

---

## 🦀 Features

* **Constraint Propagation:** Employs message-passing algorithms to prune the search space before resorting to backtracking.
* **Exhaustive Search:** Capable of identifying and returning all possible valid solutions for a given grid.
* **Rust-Powered:** Leverages Rust's memory safety and speed for rapid execution.
* **Simple Interface:** Clean CLI for processing puzzle files.

---

## ✏️ Mathematical Foundation

The solver models Sudoku as a **Constraint Satisfaction Problem (CSP)** over a grid of variables $x_{i,j}$ with current domains $`X_{i,j} \subseteq \{1, \dots, 9\}`$.

The global configuration is valid if it satisfies a collection of local constraints, each represented by an indicator factor $\phi$. 
For each row, column, and $3 \times 3$ block, let $\mathbf{z}$ denote the 9-tuple of variables associated with that unit. 
The constraint is satisfied if:
$$\phi(\mathbf{z}) = 1$$
where the factor $\phi$ is the indicator function of the permutation set:
```math
\phi(z_1, \dots, z_9) =
\begin{cases}
1 & \text{if } \{ z_1, \dots, z_9 \} = \{ 1, \dots, 9 \} \\
0 & \text{otherwise}
\end{cases}
```

Iterative constraint propagation reduces the candidate domains $X_{i,j}$ by enforcing local consistency across these factors until a fixed point is reached.

---

## 🧩 Getting Started

### Prerequisites

Ensure you have the Rust toolchain installed. 
If not, you can get it at [rustup.rs](https://rustup.rs/).

### Installation

Clone the repository and navigate into the directory:

```bash
git clone https://github.com/haguettaz/icp-sudoku-rust.git
cd sudoku-solver
```

### Usage

The solver reads puzzles from `.txt` files.
The grid should be represented by numbers $1-9$, using `_` for empty cells.

To solve a puzzle, run:

```bash
cargo run -- path/to/your_puzzle.txt
```

Several example grids are provided in the `examples/` directory to get you started.

---

## ⚙️ Implementation Details

The algorithm follows these primary steps:

1. **Initialization:** Load the grid and assign initial domains to all cells.
2. **Constraint Propagation:** Reduce domains by message-passing over local (row, column, and block) constraints.
3. **Search (if required):** If propagation does not result in a full solution, the solver branches on the cell with the smallest remaining domain to find all valid completions.
