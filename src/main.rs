use std::{env, fs, io::{self, ErrorKind}, path::PathBuf};

use crossterm::event::{Event, KeyCode::Esc};
use ratatui::{DefaultTerminal, Frame, style::Style, widgets::{Block, List}};
use ratatui_textarea::{Input, TextArea};

fn main() -> io::Result<()> {
    let mut root = env::home_dir().expect("获取家目录失败");
    root.push(".log");
    if let Err(e) = fs::create_dir(&root) && e.kind() != ErrorKind::AlreadyExists {
        panic!("{e}")
    }
    let mut terminal = ratatui::init();
    let restore = app(&mut terminal, root);
    ratatui::restore();
    restore
}

fn app(terminal: &mut DefaultTerminal, root: PathBuf) -> std::io::Result<()> {
    let list = root.read_dir()?.fold(vec![], |mut acc, dir| {
        if let Ok(dir) = dir && let Some((name, _)) = dir.file_name().to_string_lossy().to_string().rsplit_once(".") {
            acc.push(name.to_string());
        }
        acc
    });
    let mut app = App{
        list: List::new(list)
        .block(Block::bordered().title("List"))
        .style(Style::new().white())
        .highlight_style(Style::new().italic())
        .highlight_symbol(">>")
        .repeat_highlight_symbol(true),
        textarea: TextArea::default(),
    };
    app.run(terminal)
}

pub enum State {

}

pub struct App {
    list: List<'static>,
    textarea: TextArea<'static>,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Ok(Event::Key(key)) = crossterm::event::read() {
                if key.code == Esc {
                    break;
                }
                self.textarea.input(Input::from(key));
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self.list.clone(), frame.area());
    }
}

pub fn open_vim() {

}