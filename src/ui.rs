use crate::app::{App, AppState};
use crate::grid::Grid;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    widgets::{Block, Paragraph, Wrap},
};

pub fn render(f: &mut Frame, app: &App) {
    let [top_chunk, bottom_chunk] = Layout::vertical([Constraint::Length(7), Constraint::Min(10)])
        .margin(2)
        .areas(f.area());

    match &app.state {
        AppState::PromptPath { input, error } => {
            let mut text = format!(
                "Welcome to the Sudoku Solver!\n\nPlease enter the path to the Sudoku .txt file:\n> {}",
                input
            );
            if let Some(err) = error {
                text.push_str(&format!("\n\nError: {}", err));
            }
            let p = Paragraph::new(text).block(Block::bordered().title(" Grid File "));
            f.render_widget(p, top_chunk);
        }
        AppState::PromptMode { grid } => {
            let text = "Grid loaded successfully!\n\nPress '1' for ANY solution (stop at first match)\nPress '2' for ALL solutions\n\n(Press Ctrl+N at any time to load a new grid)";
            let p = Paragraph::new(text).block(Block::bordered().title(" Solving Mode "));
            f.render_widget(p, top_chunk);
            draw_single_grid(f, grid, " Loaded Grid ", bottom_chunk);
        }
        AppState::PromptStart { grid, all } => {
            let text = if *all {
                "Ready to solve! \n\nMode: ALL. \n\nPress ENTER to start the engine.".to_string()
            } else {
                "Ready to solve! \n\nMode: ANY. \n\nPress ENTER to start the engine.".to_string()
            };
            let p = Paragraph::new(text).block(Block::bordered().title(" Ready to Solve "));
            f.render_widget(p, top_chunk);
            draw_single_grid(f, grid, " Loaded Grid ", bottom_chunk);
        }
        AppState::Solved {
            grid,
            solutions,
            active_index,
            duration,
        } => {
            let total = solutions.len();
            let text = if total == 0 {
                format!(
                    "Solved in {} ms! No solutions found.\n\nPress ESC to quit, or Ctrl+N to load another grid.",
                    duration.as_millis()
                )
            } else {
                format!(
                    "Solved in {} ms! Found {} solution(s).\n\nShowing solution {} of {}.\nUse [LEFT] and [RIGHT] arrows to navigate. Press ESC to quit, or Ctrl+N to load another grid.",
                    duration.as_millis(),
                    total,
                    active_index + 1,
                    total
                )
            };

            let p = Paragraph::new(text)
                .block(Block::bordered().title(" Results "))
                .wrap(Wrap { trim: true });
            f.render_widget(p, top_chunk);

            if total == 0 {
                draw_single_grid(f, grid, " Initial Grid ", bottom_chunk);
            } else {
                let [left_pane, right_pane] =
                    Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .areas(bottom_chunk);

                draw_single_grid(
                    f,
                    &solutions[*active_index],
                    " Active Solution ",
                    right_pane,
                );
                draw_single_grid(f, grid, " Initial Grid ", left_pane);
            }
        }
    }
}

fn draw_single_grid(f: &mut Frame, grid: &Grid, title: &str, area: Rect) {
    let mut s = String::new();
    for r in 0..9 {
        if r > 0 && r % 3 == 0 {
            s.push_str("------+-------+------\n");
        }
        for c in 0..9 {
            if c > 0 && c % 3 == 0 {
                s.push_str(" | ");
            } else if c > 0 {
                s.push(' ');
            }
            match grid[r * 9 + c] {
                Some(val) => s.push_str(&val.to_string()),
                None => s.push('.'),
            }
        }
        s.push('\n');
    }

    let p = Paragraph::new(s).block(Block::bordered().title(title));
    f.render_widget(p, area);
}
