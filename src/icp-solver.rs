use anyhow::{Result, anyhow};

use rayon::prelude::*;
use std::sync::mpsc::{Sender, channel};

use crate::grid::Grid;

const NMAX_SOL: usize = 250;
pub type Messages = [u16; 81];

enum Status {
    Stall,
    Solved,
    Unsolvable,
}

fn init_msgs_from(grid: Grid) -> Messages {
    grid.map(|c| match c {
        Some(v) => 1 << (v - 1),
        None => 0b111111111,
    })
}

/// Solve the grid, returning at most NMAX_SOL
pub fn solve(grid: Grid, all: bool) -> Result<Vec<Grid>> {
    let msgs = init_msgs_from(grid);

    // Creates a new asynchronous channel, with transmitter and receiver
    let (tx, rx) = channel::<Result<Grid>>();

    rayon::spawn(move || {
        // Catch any error happening in the internal solve and send it to the channel
        if let Err(e) = solve_internal(msgs, tx.clone()) {
            let _ = tx.send(Err(e));
        }
    });

    let nmax = if all { NMAX_SOL } else { 1 };
    let mut solutions = Vec::with_capacity(nmax);

    // Process the stream, short-circuiting the main thread if an error is received
    for msg in rx {
        match msg {
            Ok(sol) => {
                solutions.push(sol);
                if solutions.len() >= nmax {
                    return Ok(solutions);
                }
            }
            Err(e) => return Err(e),
        }
    }

    Ok(solutions)
}

/// Given a global state over all grid messages, solve it.
/// 1. Run iterative constraint propagation on the current messages until convergence
/// 2. Process the different convergence status
///     * if unsolvable, returns an empty iterator
///     * if solved, returns an iterator over this single solution
///     * if stall, guess and returns an iterator over the subsequent solving
fn solve_internal(mut msgs: Messages, sender: Sender<Result<Grid>>) -> Result<()> {
    let status = icp(&mut msgs);

    match status {
        Status::Unsolvable => Ok(()),
        Status::Solved => {
            let mut sol = [None; 81];
            for (idx, &msg) in msgs.iter().enumerate() {
                // 4. Safely handle errors instead of unwrapping
                let b = (0..9)
                    .find(|&b| msg & (1 << b) != 0)
                    .ok_or_else(|| anyhow!("Corrupted state: no candidate at index {}", idx))?;
                sol[idx] = Some(b as u16 + 1);
            }

            // Send the Ok solution. Ignore send errors (happens if receiver dropped).
            let _ = sender.send(Ok(sol));
            Ok(())
        }
        Status::Stall => {
            if let Some((i, imsg)) = find_srd(&msgs) {
                (0..9)
                    .into_par_iter()
                    .filter(move |&b| (imsg >> b) & 1 != 0)
                    .try_for_each_with(sender, |s, b| -> Result<()> {
                        let mut next_msgs = msgs;
                        next_msgs[i] = 1 << b;

                        // Recursive call bubbles errors up to try_for_each_with
                        solve_internal(next_msgs, s.clone())
                    })?;
            }
            Ok(())
        }
    }
}

/// Returns the index of the uncertain cell with the least candidate values.
/// Returns `None` if every cell has at most one candidate value.
fn find_srd(msgs: &[u16; 81]) -> Option<(usize, u16)> {
    msgs.iter()
        .enumerate()
        .filter_map(|(i, &msg)| (msg.count_ones() > 1).then_some((i, msg)))
        .min_by_key(|&(_, msg)| msg.count_ones())
}

// Note: basic scheduling seems to be sufficient.
fn icp(msgs: &mut [u16; 81]) -> Status {
    // Allocate a workspace to check local constraints
    let mut others_in_row = [0b11111111; 8];
    let mut others_in_col = [0b11111111; 8];
    let mut others_in_blk = [0b11111111; 8];

    let mut has_changed = true;
    while has_changed {
        has_changed = false;
        for i in 0..81 {
            let row = i / 9;
            let col = i % 9;

            load_row_msgs(row, col, &msgs, &mut others_in_row);
            load_col_msgs(row, col, &msgs, &mut others_in_col);
            load_blk_msgs(row, col, &msgs, &mut others_in_blk);

            for b in 0..9 {
                // Branchless programming is usually only faster when branches are highly unpredictable
                // Here, mutations are relatively rare compared to checks so if-statements are more efficient.
                if (msgs[i] >> b) & 1 != 0
                    && (!is_valid(1 << b, &others_in_row)
                        || !is_valid(1 << b, &others_in_col)
                        || !is_valid(1 << b, &others_in_blk))
                {
                    msgs[i] &= !(1 << b); // unset the bit
                    has_changed = true
                }
            }
        }
    }

    get_status(&msgs)
}

fn get_status(msgs: &Messages) -> Status {
    if msgs.iter().any(|&msg| msg.count_ones() == 0) {
        return Status::Unsolvable;
    }
    if msgs.iter().all(|&msg| msg.count_ones() == 1) {
        return Status::Solved;
    }
    return Status::Stall;
}

/// Recursive function to determine the validity of a state given a slice of msgs
/// involved in the same constraint.
fn is_valid(state: u16, msgs: &[u16]) -> bool {
    state == 0b111111111
        || (0..9).any(|b| {
            ((msgs[0] >> b) & 1 != 0)
                && (state >> b) & 1 == 0
                && is_valid(state | 1 << b, &msgs[1..])
        })
}

fn load_row_msgs(row: usize, col: usize, msgs: &Messages, out: &mut [u16]) {
    for (i, c) in ((0..col).chain(col + 1..9)).enumerate() {
        out[i] = msgs[row * 9 + c];
    }
}

fn load_col_msgs(row: usize, col: usize, msgs: &Messages, out: &mut [u16]) {
    for (i, r) in ((0..row).chain(row + 1..9)).enumerate() {
        out[i] = msgs[r * 9 + col];
    }
}

fn load_blk_msgs(row: usize, col: usize, msgs: &Messages, out: &mut [u16]) {
    let block_row = row / 3;
    let block_col = col / 3;
    let pos = (row % 3) * 3 + (col % 3);
    for (i, p) in ((0..pos).chain(pos + 1..9)).enumerate() {
        out[i] = msgs[(block_row * 3 + p / 3) * 9 + (block_col * 3 + p % 3)];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid() {
        let mut msgs = [0b111111111; 8];
        assert_eq!(is_valid(0b000010000, &msgs), true);

        msgs[7] = 0b000110000;
        assert_eq!(is_valid(0b000010000, &msgs), true);

        msgs[7] = 0b000010000;
        assert_eq!(is_valid(0b000010000, &msgs), false);
    }
}
