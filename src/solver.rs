use crate::grid::Grid;
use anyhow::{Result, anyhow};

const NMAX_SOL: usize = 250;
const ALL_CANDIDATES: u32 = 0b111111111;

// Lookup tables for quick identification of row, column, and block.
pub const ROW: [usize; 81] = {
    let mut arr = [0; 81];
    let mut i = 0;
    while i < 81 {
        arr[i] = i / 9;
        i += 1;
    }
    arr
};
pub const COL: [usize; 81] = {
    let mut arr = [0; 81];
    let mut i = 0;
    while i < 81 {
        arr[i] = i % 9;
        i += 1;
    }
    arr
};
pub const BLK: [usize; 81] = {
    let mut arr = [0; 81];
    let mut i = 0;
    while i < 81 {
        let r = i / 9;
        let c = i % 9;
        arr[i] = (r / 3) * 3 + (c / 3);
        i += 1;
    }
    arr
};

pub fn solve(grid: Grid, find_all: bool) -> Result<Vec<Grid>> {
    let mut solver = Solver::new(grid, find_all)?;
    solver.solve_recursive()?;
    Ok(solver.solutions)
}

// The core state maintains flat arrays for bitmasks.
// This allows the CPU to evaluate constraints using single clock-cycle bitwise operations.
#[derive(Clone)]
struct Solver {
    grid: Grid,
    rows: [u32; 9],
    cols: [u32; 9],
    blocks: [u32; 9],
    empty_cells: Vec<usize>,
    solutions: Vec<Grid>,
    find_all: bool,
}

impl Solver {
    fn new(initial_grid: Grid, find_all: bool) -> Result<Self> {
        // We use a flat array representation for the masks.
        // A set bit at index 'v' means the value 'v + 1' is already placed in that unit.
        let mut solver = Self {
            grid: initial_grid,
            rows: [0; 9],   // Bitmask of values placed in each row
            cols: [0; 9],   // Bitmask of values placed in each column
            blocks: [0; 9], // Bitmask of values placed in each block
            empty_cells: Vec::with_capacity(81),
            solutions: Vec::with_capacity(if find_all { NMAX_SOL } else { 1 }),
            find_all,
        };

        // Initialize the masks and track which cells still need to be solved.
        // Operating on a pre-filtered list of empty cells prevents iterating over the
        // entire 81-cell board during every backtrace step.
        for i in 0..81 {
            match initial_grid[i] {
                Some(val) => {
                    let bit = 1 << (val - 1);
                    let r = i / 9;
                    let c = i % 9;
                    let b = (r / 3) * 3 + (c / 3);

                    // Guard against invalid initial grids where a constraint is already violated.
                    if (solver.rows[r] | solver.cols[c] | solver.blocks[b]) & bit != 0 {
                        return Err(anyhow!(
                            "Corrupted state: initial grid violates constraints at index {}",
                            i
                        ));
                    }

                    solver.rows[r] |= bit;
                    solver.cols[c] |= bit;
                    solver.blocks[b] |= bit;
                }
                None => {
                    solver.empty_cells.push(i);
                }
            }
        }

        Ok(solver)
    }

    // Single-threaded backtracking loop.
    // Threading overhead far outweighs the cost of these lightweight bitwise operations.
    // Keeping this on a single thread ensures the L1 cache stays hot.
    fn solve_recursive(&mut self) -> Result<bool> {
        // If no empty cells remain, the grid is successfully solved.
        if self.empty_cells.is_empty() {
            self.solutions.push(self.grid);

            // Short-circuit the recursion entirely if we only need one solution
            // or if we have hit the maximum requested solutions.
            let should_stop = !self.find_all || self.solutions.len() >= NMAX_SOL;
            return Ok(should_stop);
        }

        // Always branch on the most constrained cell.
        // Picking the cell with the fewest candidates minimizes the branching factor
        // of the search tree, drastically reducing total execution time.
        let best_idx = self.find_most_constrained_cell();

        let cell = self.empty_cells[best_idx];
        let r = ROW[cell];
        let c = COL[cell];
        let b = BLK[cell];

        // Propagate constraint
        let taken = self.rows[r] | self.cols[c] | self.blocks[b]; // // Bitmask of already taken values
        let mut available = !taken & ALL_CANDIDATES; // Mask out the unused upper bits of the 16-bit integer

        // If a cell has no available candidates but the grid is not full,
        // this branch is a dead end. We return false to backtrack.
        if available == 0 {
            return Ok(false);
        }

        // Temporarily remove the cell from the empty list to step deeper into the recursion.
        // Swap-remove is $O(1)$ and avoids shifting array elements in memory.
        self.empty_cells.swap_remove(best_idx);

        // Iterate through all valid candidate bits.
        // Using trailing zeros quickly identifies the exact value of the lowest set bit,
        // allowing us to skip checking zeroes entirely.
        while available != 0 {
            let bit_idx = available.trailing_zeros();
            let bit = 1 << bit_idx;

            available &= !bit; // Clear the bit we are currently testing

            self.grid[cell] = Some((bit_idx + 1) as u32);
            self.rows[r] |= bit;
            self.cols[c] |= bit;
            self.blocks[b] |= bit;

            let should_stop = self.solve_recursive()?;
            if should_stop {
                return Ok(true);
            }

            // Backtrack by unsetting the bits.
            // This restores the state cleanly without requiring deep copies of the solver struct.
            self.grid[cell] = None;
            self.rows[r] &= !bit;
            self.cols[c] &= !bit;
            self.blocks[b] &= !bit;
        }

        // Restore the cell to the empty list before returning to the parent caller.
        self.empty_cells.push(cell);

        // Swap it back to its original position to maintain deterministic behavior
        // and correct index tracking across recursive bounds.
        let last_idx = self.empty_cells.len() - 1;
        self.empty_cells.swap(best_idx, last_idx);

        Ok(false)
    }

    // Scans the remaining empty cells to find the one with the fewest possible candidates.
    fn find_most_constrained_cell(&self) -> usize {
        let mut min_candidates = 10;
        let mut best_idx = 0;

        for (idx, &cell) in self.empty_cells.iter().enumerate() {
            let r = ROW[cell];
            let c = COL[cell];
            let b = BLK[cell];

            let taken = self.rows[r] | self.cols[c] | self.blocks[b]; // // Bitmask of already taken values
            let available = !taken & ALL_CANDIDATES; // Mask out the unused upper bits of the 16-bit integer
            let candidates_count = available.count_ones();

            if candidates_count < min_candidates {
                min_candidates = candidates_count;
                best_idx = idx;

                // Absolute minimum candidates possible for an unsolved cell is zero or one.
                // If we hit this, we can stop searching immediately as it cannot be beaten.
                if min_candidates <= 1 {
                    break;
                }
            }
        }

        best_idx
    }
}
