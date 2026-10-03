mod app;
mod grid;
mod solver;
mod ui;

use app::{App, AppState};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
};
use std::io;
use std::time::Instant;

fn run_app(terminal: &mut DefaultTerminal, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.code == KeyCode::Esc {
                return Ok(());
            }
            if key.code == KeyCode::Char('n')
                && key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::META)
            {
                app.state = AppState::PromptPath {
                    input: String::new(),
                    error: None,
                };
                continue;
            }

            match &mut app.state {
                AppState::PromptPath { input, error } => match key.code {
                    KeyCode::Char(c) if key.modifiers.is_empty() => input.push(c),
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        // Call the solver module directly
                        match grid::load(input.trim()) {
                            Ok(grid) => app.state = AppState::PromptMode { grid },
                            Err(e) => *error = Some(e),
                        }
                    }
                    _ => {}
                },
                AppState::PromptMode { grid } => match key.code {
                    KeyCode::Char('1') => {
                        app.state = AppState::PromptStart {
                            grid: *grid,
                            all: false,
                        }
                    }
                    KeyCode::Char('2') => {
                        app.state = AppState::PromptStart {
                            grid: *grid,
                            all: true,
                        }
                    }
                    _ => {}
                },
                AppState::PromptStart { grid, all } => {
                    if key.code == KeyCode::Enter {
                        // Call the solver module directly and time the solve
                        let now = Instant::now();
                        let result = solver::solve(*grid, *all);
                        let time = now.elapsed();
                        match result {
                            Ok(solutions) => {
                                app.state = AppState::Solved {
                                    grid: *grid,
                                    solutions,
                                    duration: time,
                                    active_index: 0,
                                }
                            }
                            Err(_) => {
                                return Err(io::Error::new(
                                    io::ErrorKind::Other,
                                    "Solver encountered an error",
                                ));
                            }
                        }
                    }
                }
                AppState::Solved {
                    solutions,
                    active_index,
                    ..
                } => match key.code {
                    KeyCode::Left => *active_index = active_index.saturating_sub(1),
                    KeyCode::Right => {
                        if !solutions.is_empty() && *active_index < solutions.len() - 1 {
                            *active_index += 1;
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let app = App::new();
    let app_result = run_app(&mut terminal, app);

    ratatui::restore();

    app_result
}
