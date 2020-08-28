use crate::app::stateful_list::{Direction as ListDirection, StatefulList};
use chrono::{DateTime, Duration, Local};
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct GroupList<T> {
    pub name: String,
    pub list: StatefulList<T>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Item {
    pub title: String,
    pub desc: String,
    pub start_at: Option<DateTime<Local>>,
    pub end_at: Option<DateTime<Local>>,
    pub duration: i64,
    pub paused: bool,
}

impl Item {
    pub fn formatted_duration(&self) -> String {
        let mut output = "Duration:".to_string();
        let mut duration = Duration::milliseconds(self.duration);

        if duration.num_weeks() > 0 {
            output.push_str(format!(" {}w", duration.num_weeks()).as_str());
            if let Some(dur) = duration.checked_sub(&Duration::weeks(duration.num_weeks())) {
                duration = dur;
            }
        }

        if duration.num_hours() > 0 {
            output.push_str(format!(" {}h", duration.num_hours()).as_str());
            if let Some(dur) = duration.checked_sub(&Duration::hours(duration.num_hours())) {
                duration = dur;
            }
        }

        if duration.num_minutes() > 0 {
            output.push_str(format!(" {}m", duration.num_minutes()).as_str());
            if let Some(dur) = duration.checked_sub(&Duration::minutes(duration.num_minutes())) {
                duration = dur;
            }
        }

        output.push_str(format!(" {}s", duration.num_seconds()).as_str());
        output
    }
}

#[derive(Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub group_list: StatefulList<GroupList<Item>>,
    #[serde(skip)]
    pub curr_size: Rect,
    #[serde(skip)]
    pub active_list: Option<usize>,
    pub dialog_input: Item,
    #[serde(skip)]
    pub open_dialog: bool,
}

impl<'a> App {
    pub fn new(name: String, size: Rect) -> App {
        App {
            curr_size: size,
            name,
            group_list: StatefulList::new(),
            active_list: None,
            dialog_input: Item::default(),
            open_dialog: false,
        }
    }

    pub fn add_time(&mut self, duration: std::time::Duration) {
        for list in &mut self.group_list.items {
            for item in &mut list.list.items {
                if item.start_at.is_some() && item.end_at.is_none() && !item.paused {
                    if let Ok(time) = Duration::from_std(duration) {
                        item.duration = item.duration + time.num_milliseconds();
                    }
                }
            }
        }
    }

    fn show_dialog<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let size = frame.size();
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

        let para = Paragraph::new(Span::raw(self.dialog_input.title.clone()))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        let mut para_box = dialog_size.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });
        para_box.height = 1;
        frame.render_widget(dialog_block, dialog_size);
        frame.render_widget(para, para_box);
    }

    pub fn close_dialog(&mut self) {
        self.open_dialog = false;
        self.dialog_input.title.clear();
        self.dialog_input.desc.clear();
    }

    pub fn event(&mut self, key: KeyCode, modi: KeyModifiers) {
        match (key, modi) {
            (KeyCode::Esc, _) => {
                if self.open_dialog {
                    self.close_dialog();
                }
            }
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                self.open_dialog = true;
            }
            (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                if let Some(index) = self.active_list {
                    let list = &mut self.group_list.items.get_mut(index).unwrap().list;
                    if let Some(index) = list.state.selected() {
                        list.items.remove(index);
                    }
                } else {
                    if let Some(index) = self.group_list.state.selected() {
                        self.group_list.items.remove(index);
                        self.group_list.state.select(None);
                    }
                }
            }
            (KeyCode::Char('s'), KeyModifiers::ALT) => {
                if let Some(index) = self.active_list {
                    if let Some(list) = self.group_list.items.get_mut(index) {
                        if let Some(index) = list.list.state.selected() {
                            if let Some(item) = list.list.items.get_mut(index) {
                                if item.start_at.is_some() {
                                    item.start_at = None;
                                } else {
                                    item.start_at = Some(Local::now());
                                }
                            }
                        }
                    }
                }
            }
            (KeyCode::Char('d'), KeyModifiers::ALT) => {
                if let Some(index) = self.active_list {
                    if let Some(list) = self.group_list.items.get_mut(index) {
                        if let Some(index) = list.list.state.selected() {
                            if let Some(item) = list.list.items.get_mut(index) {
                                if item.end_at.is_some() {
                                    item.end_at = None;
                                } else {
                                    item.end_at = Some(Local::now());
                                }
                            }
                        }
                    }
                }
            }
            (KeyCode::Char('p'), KeyModifiers::ALT) => {
                if let Some(index) = self.active_list {
                    if let Some(list) = self.group_list.items.get_mut(index) {
                        if let Some(index) = list.list.state.selected() {
                            if let Some(item) = list.list.items.get_mut(index) {
                                item.paused = !item.paused
                            }
                        }
                    }
                }
            }
            (KeyCode::Char(x), KeyModifiers::NONE) => {
                if self.open_dialog {
                    self.dialog_input.title = format!("{}{}", self.dialog_input.title, x);
                }
            }
            (KeyCode::Char(x), KeyModifiers::SHIFT) => {
                if self.open_dialog {
                    self.dialog_input.title = format!("{}{}", self.dialog_input.title, x);
                }
            }
            (KeyCode::Backspace, _) => {
                if self.open_dialog {
                    let _ = self.dialog_input.title.pop();
                }
            }
            (KeyCode::Enter, _) => {
                if self.open_dialog {
                    if let Some(index) = self.active_list {
                        let list = &mut self.group_list.items.get_mut(index).unwrap().list;
                        list.add(Item {
                            title: self.dialog_input.title.clone(),
                            desc: "".to_string(),
                            start_at: None,
                            end_at: None,
                            duration: 0,
                            paused: false,
                        });
                    } else {
                        self.group_list.add(GroupList {
                            name: self.dialog_input.title.to_string(),
                            list: StatefulList::new(),
                        });
                    }

                    self.close_dialog();
                }
            }
            (KeyCode::Up, KeyModifiers::CONTROL) => {
                if let Some(index) = self.active_list {
                    let list = &mut self.group_list.items.get_mut(index).unwrap().list;
                    list.move_selected_item(ListDirection::Down);
                } else {
                    self.group_list.move_selected_item(ListDirection::Down);
                }
            }
            (KeyCode::Down, KeyModifiers::CONTROL) => {
                if let Some(index) = self.active_list {
                    let list = &mut self.group_list.items.get_mut(index).unwrap().list;
                    list.move_selected_item(ListDirection::Up);
                } else {
                    self.group_list.move_selected_item(ListDirection::Up);
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
            if let Some(group_list) = self.group_list.items.get_mut(index) {
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

                if let Some(index) = group_list.list.state.selected() {
                    if let Some(item) = group_list.list.items.get(index) {
                        let item_list_layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                            .split(layout[1]);

                        let dialog_block = Block::default()
                            .title(format!(" {} ", item.title.clone()))
                            .borders(Borders::ALL)
                            .style(Style::default());

                        let para_box = item_list_layout[1].inner(&Margin {
                            vertical: 1,
                            horizontal: 1,
                        });

                        let card_layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
                            .split(para_box);

                        let para = Paragraph::new(Span::raw(item.desc.clone()))
                            .style(Style::default().fg(Color::White).bg(Color::Black))
                            .alignment(Alignment::Left)
                            .wrap(Wrap { trim: true });

                        frame.render_widget(para, card_layout[0]);

                        let start_at = if let Some(start_at) = item.start_at {
                            format!("Started: {}", start_at.to_rfc2822())
                        } else {
                            "Started: Not started".to_string()
                        };

                        let end_at = if let Some(end_at) = item.end_at {
                            format!("Ended: {}", end_at.to_rfc2822())
                        } else {
                            "Ended: Not done".to_string()
                        };

                        let paused = if item.paused {
                            "Paused"
                        } else {
                            if item.start_at.is_some() && item.end_at.is_none() {
                                "In progress"
                            } else {
                                ""
                            }
                        };

                        let mut info = Text::default();
                        info.lines.push(Spans::from(vec![Span::raw(start_at)]));
                        info.lines.push(Spans::from(vec![Span::raw(end_at)]));
                        info.lines
                            .push(Spans::from(vec![Span::raw(item.formatted_duration())]));
                        info.lines.push(Spans::from(vec![Span::raw(paused)]));

                        let para = Paragraph::new(info)
                            .style(Style::default().fg(Color::White).bg(Color::Black))
                            .alignment(Alignment::Left)
                            .wrap(Wrap { trim: true });

                        frame.render_widget(para, card_layout[1]);

                        frame.render_widget(dialog_block, item_list_layout[1]);

                        frame.render_stateful_widget(
                            list,
                            item_list_layout[0],
                            &mut group_list.list.state,
                        );
                    } else {
                        frame.render_stateful_widget(list, layout[1], &mut group_list.list.state);
                    }
                } else {
                    frame.render_stateful_widget(list, layout[1], &mut group_list.list.state);
                }
            }
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
        if self.open_dialog {
            self.show_dialog(frame);
        };
    }
}
