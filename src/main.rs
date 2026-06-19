use std::{
    env,
    fs::{self, File},
    io::{
        ErrorKind::{self, AlreadyExists},
        Read, Write,
    },
    path::PathBuf,
    time::{Duration, SystemTime},
};

use chrono::{DateTime, Local};
use crossterm::event::{
    Event,
    KeyCode::{Char, Down, Enter, Esc, Left, Right, Up},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{
        Constraint::{Fill, Length},
        Layout,
    },
    style::{Color, Style, Stylize},
    symbols::border,
    widgets::{Block, Borders, List, ListState, Padding, Paragraph, StatefulWidget, Widget},
};
use ratatui_textarea::{Input, TextArea, WrapMode};

#[derive(PartialEq, Clone)]
enum Mode {
    List(u8),
    Edit(EditState),
}

#[derive(PartialEq, Clone, Copy)]
pub enum EditMode {
    Insert,
    Normal,
}

#[derive(PartialEq, Clone)]
pub struct EditState {
    begin_time: String,
    mode: EditMode,
}

pub struct App {
    raw_path: PathBuf,
    raw_list: Vec<String>,
    list: List<'static>,
    list_state: ListState,
    textarea: TextArea<'static>,
    mode: Mode,
    now_time: DateTime<Local>,
}

fn main() -> anyhow::Result<()> {
    let mut root = env::home_dir().expect("获取家目录失败");
    root.push(".log");
    if let Err(e) = fs::create_dir(&root)
        && e.kind() != ErrorKind::AlreadyExists
    {
        panic!("{e}")
    }
    let mut terminal = ratatui::init();
    let restore = app(&mut terminal, root);
    ratatui::restore();
    restore
}

fn app(terminal: &mut DefaultTerminal, root: PathBuf) -> anyhow::Result<()> {
    let mut list = root.read_dir()?.fold(vec![], |mut acc, dir| {
        if let Ok(dir) = dir
            && let Some((name, _)) = dir
                .file_name()
                .to_string_lossy()
                .to_string()
                .rsplit_once(".")
        {
            acc.push(name.to_string());
        }
        acc
    });
    list.sort();
    list.reverse();
    let mut app = App {
        raw_path: root,
        raw_list: list.clone(),
        list: raw_to_list(list),
        list_state: ListState::default(),
        textarea: TextArea::default(),
        mode: Mode::List(1),
        now_time: SystemTime::now().into(),
    };
    app.run(terminal)
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        self.textarea.set_hard_tab_indent(true);
        self.textarea.set_line_number_style(Style::default().fg(Color::DarkGray));
        self.textarea.set_cursor_line_style(Style::default());
        self.textarea
            .set_block(Block::default().borders(Borders::ALL).title("Edit"));
        self.textarea.set_wrap_mode(WrapMode::Glyph);
        loop {
            self.now_time = SystemTime::now().into();
            terminal.draw(|frame| self.draw(frame))?;
            if crossterm::event::poll(Duration::from_secs(1))? && let Ok(Event::Key(key)) = crossterm::event::read() {
                match self.mode.clone() {
                    Mode::Edit(state) => {
                        if key.code == Esc {
                            self.raw_path.push(format!("{}.txt", state.begin_time));
                            let file = File::create_new(&self.raw_path);
                            let linetext = self.textarea.lines().iter().fold(
                                String::new(),
                                |mut cc, str| {
                                    cc += str;
                                    cc += "\n";
                                    cc
                                }
                            );
                            match file {
                                Ok(mut file) => {
                                    let _ = file.write(
                                        format!(
                                            "log #{}\nbegin time: {}\nend time: {}\n\n{}",
                                            self.raw_list.len(),
                                            state.begin_time,
                                            self.now_time.format("%Y-%m-%d %H:%M:%S"),
                                            linetext
                                        )
                                        .as_bytes(),
                                    );
                                    let mut temp = vec![self.raw_path
                                            .file_name()
                                            .unwrap()
                                            .to_string_lossy()
                                            .to_string()
                                            .rsplit_once(".")
                                            .unwrap()
                                            .0
                                            .to_string()];
                                        temp.extend(self.raw_list.clone());
                                    self.raw_list = temp;
                                    self.list = raw_to_list(self.raw_list.clone());
                                }
                                Err(err) => {
                                    if err.kind() == AlreadyExists {
                                        let mut file = File::create(&self.raw_path)?;
                                        let _ = file.write(
                                            linetext
                                            .as_bytes(),
                                        );
                                    }
                                }
                            }
                            self.textarea.clear();
                            self.raw_path.pop();
                            self.mode = Mode::List(1);
                        } else {
                            self.textarea.input(Input::from(key));
                        }
                    }
                    Mode::List(index) => match key.code {
                        Esc => {
                            break;
                        }
                        Char('q') => {
                            break;
                        }
                        Up => {
                            self.list_state.select_previous();
                        }
                        Down => {
                            self.list_state.select_next();
                        }
                        Left => {
                            if index <= 1 {
                                continue;
                            }
                            self.mode = Mode::List(index - 1);
                        }
                        Right => {
                            if index >= 3 {
                                continue;
                            }
                            self.mode = Mode::List(index + 1);
                        }
                        Char('w') => {
                            self.list_state.select_previous();
                        }
                        Char('s') => {
                            self.list_state.select_next();
                        }
                        Char('a') => {
                            if index <= 1 {
                                continue;
                            }
                            self.mode = Mode::List(index - 1);
                        }
                        Char('d') => {
                            if index >= 3 {
                                continue;
                            }
                            self.mode = Mode::List(index + 1);
                        }

                        Enter => {
                            if index == 2 {
                                self.mode = Mode::Edit(EditState {
                                    begin_time: self
                                        .now_time
                                        .format("%Y-%m-%d %H:%M:%S")
                                        .to_string(),
                                    mode: EditMode::Normal,
                                })
                            } else {
                                if let Some(list_index) = self.list_state.selected() {
                                    if index == 1 {
                                        self.raw_path
                                            .push(format!("{}.txt", self.raw_list[list_index]));
                                        let mut file = File::open(&self.raw_path)?;
                                        self.raw_path.pop();
                                        let mut buf = String::new();
                                        file.read_to_string(&mut buf)?;
                                        buf.pop();
                                        self.textarea.insert_str(buf);
                                        self.mode = Mode::Edit(EditState {
                                            begin_time: self.raw_list[list_index].clone(),
                                            mode: EditMode::Normal,
                                        });
                                    } else if index == 3 {
                                        self.raw_path
                                            .push(format!("{}.txt", self.raw_list[list_index]));
                                        fs::remove_file(&self.raw_path)?;
                                        self.raw_path.pop();
                                        self.raw_list.remove(list_index);
                                        self.list = raw_to_list(self.raw_list.clone());
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buffer = frame.buffer_mut();
        match self.mode {
            Mode::List(index) => {
                let [top, list] = Layout::vertical([Length(3), Fill(1)]).areas(area);
                let [_new, _read, _delete, time] =
                    Layout::horizontal([Fill(1), Fill(1), Fill(1), Fill(1)]).areas(top);

                let mut read = Paragraph::new("Read").block(normal_block());
                let mut new = Paragraph::new("New").block(normal_block());
                let mut delete = Paragraph::new("Delete").block(normal_block());
                if index == 1 {
                    read = read.block(normal_block().bg(Color::White).fg(Color::Black));
                } else if index == 2 {
                    new = new.block(normal_block().bg(Color::White).fg(Color::Black));
                } else if index == 3 {
                    delete = delete.block(normal_block().bg(Color::White).fg(Color::Black));
                }
                read.render(_new, buffer);
                new.render(_read, buffer);
                delete.render(_delete, buffer);
                Paragraph::new(format!("{}", self.now_time.format("%Y-%m-%d %H:%M:%S")))
                    .block(normal_block())
                    .render(time, buffer);

                StatefulWidget::render(self.list.clone(), list, buffer, &mut self.list_state);
            }
            Mode::Edit(_) => {
                let [top, _list] = Layout::vertical([Fill(1), Length(3)]).areas(area);
                self.textarea.clone().render(top, buffer);
            }
        }
    }
}

pub fn raw_to_list(list: Vec<String>) -> List<'static> {

    List::new(list)
        .block(Block::bordered().title("Logs").padding(Padding::left(2)))
        .style(Style::new().white())
        .highlight_style(
            Style::new()
                .bg(ratatui::style::Color::White)
                .fg(ratatui::style::Color::Black),
        )
        .highlight_symbol(">>")
}

pub fn normal_block() -> Block<'static> {
    Block::bordered()
        .border_style(Style::default().fg(ratatui::style::Color::White))
        .border_set(border::THICK)
}
