use crate::app::stateful_list::{Direction as ListDirection, StatefulList};
use chrono::{DateTime, Duration, Local};
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};

use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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

    fn started(&self) -> bool {
        self.start_at.is_some()
    }

    fn done(&self) -> bool {
        self.end_at.is_some()
    }
}

#[derive(Clone)]
pub enum Input {
    Titel,
    Desc,
}

impl Default for Input {
    fn default() -> Self {
        Input::Titel
    }
}
#[derive(Clone)]
pub enum DialogState {
    New,
    Edit,
    Hide,
}
#[derive(Clone)]
pub struct Dialog {
    pub input: Item,
    pub selected_input: Input,
    pub state: DialogState,
}

impl Default for Dialog {
    fn default() -> Self {
        Dialog {
            input: Item::default(),
            selected_input: Input::Titel,
            state: DialogState::Hide,
        }
    }
}

impl<'a> Dialog {
    pub fn process_input(&mut self, key: KeyCode, modi: KeyModifiers) {
        match (key, modi) {
            (KeyCode::Esc, _) => {
                self.close_dialog();
            }
            (KeyCode::Tab, _) => match self.selected_input {
                Input::Titel => self.selected_input = Input::Desc,
                Input::Desc => self.selected_input = Input::Titel,
            },
            (KeyCode::Char(x), _) => match self.selected_input {
                Input::Titel => self.input.title.push(x),
                Input::Desc => self.input.desc.push(x),
            },
            (KeyCode::Backspace, _) => {
                match self.selected_input {
                    Input::Titel => {
                        self.input.title.pop();
                    }
                    Input::Desc => {
                        self.input.desc.pop();
                    }
                };
            }
            _ => {}
        }
    }

    pub fn close_dialog(&mut self) {
        self.state = DialogState::Hide;
        self.input = Item::default();
        self.selected_input = Input::Titel;
    }

    pub fn displayed(&self) -> bool {
        !matches!(self.state, DialogState::Hide)
    }

    pub fn display(&mut self, state: DialogState) {
        self.state = state;
    }

    pub fn editing(&self) -> bool {
        matches!(self.state, DialogState::Edit)
    }
}

#[derive(Serialize, Deserialize)]
pub struct App {
    pub name: String,
    pub group_list: StatefulList<GroupList<Item>>,
    #[serde(skip)]
    pub active_list: Option<usize>,
    #[serde(skip)]
    pub dialog: Dialog,
}

impl<'a> App {
    pub fn new(name: String) -> App {
        App {
            name,
            group_list: StatefulList::new(),
            active_list: None,
            dialog: Dialog::default(),
        }
    }

    pub fn add_time(&mut self, duration: std::time::Duration) {
        for list in &mut self.group_list.items {
            for item in &mut list.list.items {
                if item.start_at.is_some() && item.end_at.is_none() && !item.paused {
                    if let Ok(time) = Duration::from_std(duration) {
                        item.duration += time.num_milliseconds();
                    }
                }
            }
        }
    }

    fn selected_item(&self) -> Option<(usize, usize)> {
        if let Some(list_index) = self.active_list {
            if let Some(list) = self.group_list.items.get(list_index) {
                if let Some(index) = list.list.state.selected() {
                    if list.list.items.get(index).is_some() {
                        return Some((list_index, index));
                    }
                }
            }
        }
        None
    }

    fn get_item(&mut self, list_index: usize, index: usize) -> Option<&mut Item> {
        if let Some(list) = self.group_list.items.get_mut(list_index) {
            list.list.items.get_mut(index)
        } else {
            None
        }
    }

    fn get_selected_item(&mut self) -> Option<&mut Item> {
        if let Some((list_index, index)) = self.selected_item() {
            self.get_item(list_index, index)
        } else {
            None
        }
    }

    fn show_dialog<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let size = frame.size();
        let dialog_title = if self.active_list.is_some() {
            " New Item "
        } else {
            " New List "
        };

        let dialog_block = Block::default()
            .title(dialog_title)
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Blue));

        let dialog_size = Rect::new(
            size.x + size.width / 3,
            size.y + size.height / 3,
            size.width / 3,
            size.height / 3,
        );

        let (titel_input_style, desc_input_style) = match self.dialog.selected_input {
            Input::Titel => (
                Style::default().fg(Color::Black).bg(Color::LightCyan),
                Style::default().fg(Color::White).bg(Color::Black),
            ),
            Input::Desc => (
                Style::default().fg(Color::White).bg(Color::Black),
                Style::default().fg(Color::Black).bg(Color::LightCyan),
            ),
        };

        let dialog_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Ratio(1, 1),
            ])
            .split(dialog_size.inner(&Margin {
                vertical: 1,
                horizontal: 1,
            }));

        let title_label = Paragraph::new(Text::from("Title"))
            .style(Style::default().fg(Color::White).bg(Color::Blue))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        let title = Paragraph::new(Span::raw(self.dialog.input.title.clone()))
            .style(titel_input_style)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        frame.render_widget(Clear, dialog_size);
        frame.render_widget(dialog_block, dialog_size);
        frame.render_widget(title_label, dialog_layout[0]);
        frame.render_widget(title, dialog_layout[1]);

        if self.active_list.is_some() {
            let desc_label = Paragraph::new(Text::from("Description"))
                .style(Style::default().fg(Color::White).bg(Color::Blue))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });

            let desc = Paragraph::new(Span::raw(self.dialog.input.desc.clone()))
                .style(desc_input_style)
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });

            frame.render_widget(desc_label, dialog_layout[2]);
            frame.render_widget(desc, dialog_layout[3]);
        }
    }

    pub fn event(&mut self, key: KeyCode, modi: KeyModifiers) {
        if self.dialog.displayed() && key != KeyCode::Enter {
            self.dialog.process_input(key, modi);
        } else {
            match (key, modi) {
                (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                    if !self.dialog.displayed() {
                        self.dialog.display(DialogState::New);
                    }
                }
                (KeyCode::Char('e'), KeyModifiers::CONTROL) => {
                    if !self.dialog.displayed() {
                        if let Some(item) = self.get_selected_item() {
                            self.dialog.input = item.clone();
                            self.dialog.display(DialogState::Edit);
                        }
                    }
                }
                (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
                    if let Some(index) = self.active_list {
                        let list = &mut self.group_list.items.get_mut(index).unwrap().list;
                        if let Some(index) = list.state.selected() {
                            list.items.remove(index);
                        }
                    } else if let Some(index) = self.group_list.state.selected() {
                        self.group_list.items.remove(index);
                        self.group_list.state.select(None);
                    }
                }
                (KeyCode::Char('s'), KeyModifiers::ALT) => {
                    if let Some(item) = self.get_selected_item() {
                        if item.start_at.is_some() {
                            item.start_at = None;
                            item.end_at = None;
                            item.duration = 0;
                        } else {
                            item.start_at = Some(Local::now());
                        }
                    }
                }
                (KeyCode::Char('d'), KeyModifiers::ALT) => {
                    if let Some(item) = self.get_selected_item() {
                        if item.end_at.is_some() {
                            item.end_at = None;
                        } else {
                            item.end_at = Some(Local::now());
                        }
                    }
                }
                (KeyCode::Char('p'), KeyModifiers::ALT) => {
                    if let Some(item) = self.get_selected_item() {
                        item.paused = !item.paused
                    }
                }
                (KeyCode::Enter, _) => {
                    if self.dialog.displayed() {
                        if self.dialog.editing() {
                            let title = self.dialog.input.title.clone();
                            let desc = self.dialog.input.desc.clone();
                            if let Some(item) = self.get_selected_item() {
                                item.title = title;
                                item.desc = desc;
                            }
                        } else if let Some(index) = self.active_list {
                            let list = &mut self.group_list.items.get_mut(index).unwrap().list;
                            list.add(self.dialog.input.clone());
                        } else {
                            self.group_list.add(GroupList {
                                name: self.dialog.input.title.to_string(),
                                list: StatefulList::new(),
                            });
                        }
                    }
                    self.dialog.close_dialog();
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
    }

    pub fn draw<B: Backend>(&mut self, frame: &mut Frame<B>) {
        let size = frame.size();

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
                        .map(|item| {
                            let style = if item.done() {
                                Style::default().fg(Color::Green)
                            } else if item.paused {
                                Style::default().fg(Color::Blue)
                            } else if item.started() {
                                Style::default().fg(Color::Yellow)
                            } else {
                                Style::default().fg(Color::White)
                            };

                            ListItem::new(Span::styled(item.title, style))
                        })
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
                            .style(Style::default().bg(Color::Black));

                        let para_box = item_list_layout[1].inner(&Margin {
                            vertical: 1,
                            horizontal: 1,
                        });

                        let card_layout = Layout::default()
                            .direction(Direction::Vertical)
                            .constraints([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)])
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
                        } else if item.start_at.is_some() && item.end_at.is_none() {
                            "In progress"
                        } else {
                            ""
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
        if self.dialog.displayed() {
            self.show_dialog(frame);
        }
    }
}
