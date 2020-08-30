use serde::{Deserialize, Serialize};
use tui::widgets::ListState;

pub enum Direction {
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StatefulList<T> {
    #[serde(skip)]
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn new() -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items: Vec::new(),
        }
    }

    pub fn move_selected_item(&mut self, direction: Direction) {
        if let Some(index) = self.state.selected() {
            match direction {
                Direction::Down => {
                    let target = if index == 0 {
                        self.items.len() - 1
                    } else {
                        index - 1
                    };
                    self.items.swap(index, target);
                    self.previous();
                }
                Direction::Up => {
                    let target = if index == self.items.len() - 1 {
                        0
                    } else {
                        index + 1
                    };
                    self.items.swap(index, target);
                    self.next();
                }
            }
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }
}
