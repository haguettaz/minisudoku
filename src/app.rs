use std::time;

use crate::grid::Grid;

pub enum AppState {
    PromptPath {
        input: String,
        error: Option<String>,
    },
    PromptMode {
        grid: Grid,
    },
    PromptStart {
        grid: Grid,
        find_all: bool, // Whether to search for all solution or just one
    },
    Solved {
        grid: Grid,
        solutions: Vec<Grid>,
        active_index: usize,
        duration: time::Duration,
    },
}

pub struct App {
    pub state: AppState,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::PromptPath {
                input: String::new(),
                error: None,
            },
        }
    }
}
