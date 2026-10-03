use std::fs::File;
use std::io::{BufRead, BufReader, Lines, Result as IOResult};
use std::path::Path;

pub type Cell = Option<u16>;
pub type Grid = [Cell; 81];

pub fn load<P>(filename: P) -> Result<Grid, String>
where
    P: AsRef<Path>,
{
    let mut grid = Vec::new();

    // 1. Call read_lines and map the std::io::Error to a String if it fails
    let lines = read_lines(filename).map_err(|e| format!("Failed to read file: {}", e))?;

    // Consumes the iterator, ignoring lines that fail to read
    for line in lines.map_while(Result::ok) {
        grid.extend(line.chars().filter_map(|c| match c {
            '1'..='9' => Some(Some(c.to_digit(10).unwrap() as u16)),
            '_' | '.' | '0' => Some(None),
            _ => None,
        }));
    }

    // Return an Err(String) instead of panicking if the grid doesn't have exactly 81 items
    grid.try_into().map_err(|v: Vec<Cell>| {
        format!(
            "Invalid grid size: expected exactly 81 cells, found {}",
            v.len()
        )
    })
}

// Specify std::io::Result to resolve the missing error type
fn read_lines<P>(filename: P) -> IOResult<Lines<BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines())
}
