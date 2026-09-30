use simple_logger;

use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

use std::time::Instant;

fn init_from_file<P>(filename: P) -> [Option<usize>; 81]
where
    P: AsRef<Path>,
{
    // File must exist in the current path
    let mut grid = Vec::new();
    if let Ok(lines) = read_lines(filename) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.map_while(Result::ok) {
            grid.extend(line.chars().filter_map(|c| match c {
                '1'..='9' => Some(Some(c.to_digit(10).unwrap() as usize)),
                '_' => Some(None),
                _ => None,
            }));
        }
    }
    grid.try_into()
        .unwrap_or_else(|_| panic!("Failed to convert grid to array"))
}

// The output is wrapped in a Result to allow matching on errors.
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn solve(msgs: [usize; 81], sols: &mut Vec<[usize; 81]>) {
    match icp(msgs) {
        Err(_) => (),
        Ok(msgs) => {
            match is_solved(&msgs) {
                true => {
                    let new_sol =
                        msgs.map(|msg| (0..9).find(|&b| msg & (1 << b) != 0).unwrap() + 1);
                    sols.push(new_sol);
                }
                false => {
                    // if not solved, at least one cell has multiple candidates
                    // choose the cell with the smallest remaining domain (SRD) and branch on it
                    let (cur, cur_msg) = find_srd(&msgs).unwrap();
                    for b in 0..9 {
                        if cur_msg & (1 << b) != 0 {
                            let mut new_msgs = msgs.clone();
                            new_msgs[cur] = 1 << b;
                            solve(new_msgs, sols);
                        }
                    }
                }
            }
        }
    }
}

fn find_srd(&msgs: &[usize; 81]) -> Option<(usize, usize)> {
    msgs.iter()
        .enumerate()
        .filter_map(|(i, &msg)| (msg.count_ones() > 1).then_some((i, msg)))
        .min_by_key(|&(_, msg)| msg.count_ones())
}

fn icp(mut msgs: [usize; 81]) -> Result<[usize; 81], String> {
    // Note: basic scheduling seems to be sufficient.
    loop {
        let mut flag = false;
        for cur in 0..81 {
            let row = cur / 9;
            let col = cur % 9;

            log::debug!(
                "Processing cell {} (row {}, col {}) with candidates {:9b}",
                cur,
                row,
                col,
                msgs[cur]
            );

            // row
            let row_msgs = collect_other_row_msgs(row, col, &msgs);
            let col_msgs = collect_other_col_msgs(row, col, &msgs);
            let block_msgs = collect_other_block_msgs(row, col, &msgs);
            for b in 0..9 {
                if (msgs[cur] >> b) & 1 != 0 {
                    if !is_possible(1 << b, &row_msgs)
                        || !is_possible(1 << b, &col_msgs)
                        || !is_possible(1 << b, &block_msgs)
                    {
                        msgs[cur] &= !(1 << b); // unset the bit
                        flag = true
                    }
                }
            }

            // Check unsolvability
            if msgs[cur] == 0 {
                return Err("Puzzle is unsolvable".into());
            }
        }

        // Check convergence
        if !flag {
            return Ok(msgs);
        }
    }
}

fn is_possible(state: usize, msgs: &[usize]) -> bool {
    if state == 0b111111111 {
        true
    } else {
        (0..9).any(|b| {
            if (msgs[0] >> b) & 1 == 0 || (state >> b) & 1 != 0 {
                false
            } else {
                is_possible(state | 1 << b, &msgs[1..])
            }
        })
    }
}

fn collect_other_row_msgs(row: usize, col: usize, msgs: &[usize; 81]) -> Vec<usize> {
    let iter_before = (0..col).map(|c| msgs[row * 9 + c]);
    let iter_after = (col + 1..9).map(|c| msgs[row * 9 + c]);
    iter_before.chain(iter_after).collect()
}

fn collect_other_col_msgs(row: usize, col: usize, msgs: &[usize; 81]) -> Vec<usize> {
    let iter_before = (0..row).map(|r| msgs[r * 9 + col]);
    let iter_after = (row + 1..9).map(|r| msgs[r * 9 + col]);
    iter_before.chain(iter_after).collect()
}

fn collect_other_block_msgs(row: usize, col: usize, msgs: &[usize; 81]) -> Vec<usize> {
    let block_row = row / 3;
    let block_col = col / 3;
    let pos = (row % 3) * 3 + (col % 3);

    let iter_before = (0..pos).map(|i| msgs[(block_row * 3 + i / 3) * 9 + (block_col * 3 + i % 3)]);
    let iter_after =
        (pos + 1..9).map(|i| msgs[(block_row * 3 + i / 3) * 9 + (block_col * 3 + i % 3)]);
    iter_before.chain(iter_after).collect()
}

fn is_solved(msgs: &[usize; 81]) -> bool {
    msgs.iter().all(|&msg| msg.count_ones() == 1)
}

fn main() {
    // Init logger
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let args: Vec<String> = env::args().collect();
    let filename = &args[1];

    // Create the whole grid
    let grid = init_from_file(filename);
    log::debug!("{:?}", grid);

    // Initialize messages from grid
    let msgs = grid.map(|c| match c {
        Some(num) => 1 << (num - 1),
        None => 0b111111111,
    });
    log::info!("Initialization: done!");

    let mut sols: Vec<[usize; 81]> = Vec::new();

    let start_time = Instant::now();
    solve(msgs, &mut sols);
    let duration = start_time.elapsed();
    log::info!(
        "Solving: done! ({} solutions found in {} seconds)",
        sols.len(),
        duration.as_secs_f64()
    );

    for (i, sol) in sols.iter().enumerate() {
        println!("\nSolution {}:", i + 1);
        for row in 0..9 {
            for col in 0..9 {
                print!("{} ", sol[row * 9 + col]);
            }
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_is_possible() {
        let mut msgs = [0b111111111; 8];
        assert_eq!(is_possible(0b000010000, &msgs), true);

        msgs[7] = 0b000110000;
        assert_eq!(is_possible(0b000010000, &msgs), true);

        msgs[7] = 0b000010000;
        assert_eq!(is_possible(0b000010000, &msgs), false);
    }
}
