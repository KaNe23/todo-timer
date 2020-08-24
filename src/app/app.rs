use crate::app::stateful_list::StatefulList;
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
#[derive(Clone)]
pub struct GroupList<T> {
    pub name: String,
    pub list: StatefulList<T>,
}

#[derive(Clone)]
pub struct Item {
    pub title: String,
    pub desc: String,
}

pub struct App<'a> {
    pub curr_size: Rect,
    pub name: String,
    pub group_list: StatefulList<GroupList<Item>>,
    pub active_list: Option<usize>,
    pub dialog: Option<(Block<'a>, Rect, Paragraph<'a>, Rect)>,
    pub dialog_input: String,
}

impl<'a> App<'a> {
    pub fn new(name: String, size: Rect) -> App<'a> {
        App {
            curr_size: size,
            name,
            group_list: StatefulList::new(),
            active_list: None,
            dialog: None,
            dialog_input: "".to_string(),
        }
    }

    fn create_dialog(&mut self, size: Rect) -> Option<(Block<'a>, Rect, Paragraph<'a>, Rect)> {
        let dialog_title = if let Some(_) = self.active_list {
            "New Item"
        } else {
            "New List"
        };

        let dialog_block = Block::default()
            .title(dialog_title)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::LightBlue));

        let dialog_size = Rect::new(
            size.x + size.width / 3,
            size.y + size.height / 3,
            size.width / 3,
            size.height / 3,
        );

        let para = Paragraph::new(Span::raw(self.dialog_input.clone()))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        let mut para_box = dialog_size.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });
        para_box.height = 1;
        Some((dialog_block, dialog_size, para, para_box))
    }

    pub fn event(&mut self, key: KeyCode, modi: KeyModifiers) {
        match (key, modi) {
            (KeyCode::Esc, _) => {
                if let Some(_) = self.dialog {
                    self.dialog = None;
                }
            }
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                self.dialog = self.create_dialog(self.curr_size);
            }
            (KeyCode::Char(x), KeyModifiers::NONE) => {
                if let Some(_) = &self.dialog {
                    self.dialog_input = format!("{}{}", self.dialog_input, x);
                    self.dialog = self.create_dialog(self.curr_size);
                }
            }
            (KeyCode::Char(x), KeyModifiers::SHIFT) => {
                if let Some(_) = &self.dialog {
                    self.dialog_input = format!("{}{}", self.dialog_input, x);
                    self.dialog = self.create_dialog(self.curr_size);
                }
            }
            (KeyCode::Backspace, _) => {
                if let Some(_) = &self.dialog {
                    let _ = self.dialog_input.pop();
                    self.dialog = self.create_dialog(self.curr_size);
                }
            }
            (KeyCode::Enter, _) => {
                if self.dialog.is_some() {
                    if let Some(index) = self.active_list {
                        let list = &mut self.group_list.items.get_mut(index).unwrap().list;
                        list.add(Item {
                            title: self.dialog_input.clone(),
                            desc: "".to_string(),
                        });
                    } else {
                        self.group_list.add(GroupList {
                            name: self.dialog_input.to_string(),
                            list: StatefulList::new(),
                        });
                    }

                    self.dialog_input = "".to_string();
                    self.dialog = None;
                }
            }
            (KeyCode::Up, _) => {
                if let Some(pos) = self.active_list {
                    self.group_list.items[pos].list.previous();
                } else {
                    self.group_list.previous();
                }
            }
            (KeyCode::Down, _) => {
                if let Some(pos) = self.active_list {
                    self.group_list.items[pos].list.next();
                } else {
                    self.group_list.next();
                }
            }
            (KeyCode::Right, _) => {
                if self.active_list.is_none() {
                    self.active_list = self.group_list.state.selected();
                }
            }
            (KeyCode::Left, _) => {
                if let Some(index) = self.active_list {
                    let list = self.group_list.items.get_mut(index).unwrap();
                    list.list.state.select(None);
                    self.active_list = None;
                }
            }
            _ => {}
        }
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let size = frame.size();
        self.curr_size = size;

        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(size);

        if let Some(index) = self.group_list.state.selected() {
            let group_list = self.group_list.items.get_mut(index).unwrap();
            let list = List::new(
                group_list
                    .list
                    .items
                    .clone()
                    .into_iter()
                    .map(|gl| ListItem::new(Span::raw(gl.title)))
                    .collect::<Vec<_>>(),
            );

            let block = Block::default()
                .title(format!(" {} ", group_list.name.clone()))
                .borders(Borders::ALL)
                .style(Style::default().bg(Color::Black));

            let style = if self.active_list.is_some() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let list = list
                .block(block)
                .style(style)
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, layout[1], &mut group_list.list.state);
        }

        let list = List::new(
            self.group_list
                .items
                .clone()
                .into_iter()
                .map(|gl| ListItem::new(Span::raw(gl.name)))
                .collect::<Vec<_>>(),
        );

        let block = Block::default()
            .title(" Todo-Timer ")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Black));

        let style = if self.active_list.is_some() {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let list = list
            .block(block)
            .style(style)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol("> ");

        frame.render_stateful_widget(list, layout[0], &mut self.group_list.state);

        // render dialog
        if let Some((dialog_block, dialog_size, para, para_box)) = &self.dialog {
            frame.render_widget((*dialog_block).clone(), *dialog_size);
            frame.render_widget((*para).clone(), *para_box);
        };
    }
}
