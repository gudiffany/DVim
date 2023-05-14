use crate::keyboard::*;
use crate::raw::*;
use crate::screen::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::{terminal, Result};
use diffany::*;
use errno::errno;
use std::collections::HashMap;
use std::path::Path;
#[derive(Clone, Copy)]
enum EditorKey {
    Up,
    Right,
    Left,
    Down,
}

pub struct Editor {
    filename: String,
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
    render_x: u16,
    keymap: HashMap<char, EditorKey>,
    rows: Vec<Raw>,
    rowsoff: u16,
    coloff: u16,
}

impl Editor {
    pub fn with_file<P: AsRef<Path> + ToString>(filename: P) -> Result<Self> {
        let fn_string = filename.to_string();
        let file_line = std::fs::read_to_string(filename)
            .expect("unable to open file")
            .split('\n')
            .map(|x| x.into())
            .collect::<Vec<String>>();
        Editor::build(&file_line, fn_string)
    }
    pub fn new() -> Result<Self> {
        Editor::build(&[], "")
    }

    pub fn build<T: Into<String>>(data: &[String], filename: T) -> Result<Self> {
        let mut keymap = HashMap::new();
        keymap.insert('w', EditorKey::Up);
        keymap.insert('a', EditorKey::Left);
        keymap.insert('s', EditorKey::Down);
        keymap.insert('d', EditorKey::Right);
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Position::default(),
            keymap,
            rows: if data.is_empty() {
                Vec::new()
            } else {
                let v = Vec::from(data);
                let mut rows = Vec::new();
                for raw in v {
                    rows.push(Raw::new(raw))
                }
                rows
            },
            rowsoff: 0,
            coloff: 0,
            render_x: 0,
            filename: filename.into(),
        })
    }
    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?; //原始模式
        loop {
            if self.refresh_screen().is_err() {
                self.die("unable to refresh screen");
            }
            self.screen
                .move_to(&self.cursor, self.render_x, self.rowsoff, self.coloff)?;
            self.screen.flush()?;
            if self.process_key()? {
                break;
            }
        }
        terminal::disable_raw_mode()
    }
    pub fn process_key(&mut self) -> Result<bool> {
        if let Ok(c) = self.keyboard.read_key() {
            match c {
                KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => return Ok(true),
                KeyEvent {
                    code: KeyCode::Char(key),
                    ..
                } => match key {
                    'w' | 'a' | 's' | 'd' => {
                        let c = *self.keymap.get(&key).unwrap();
                        self.move_cursor(c);
                    }
                    _ => {}
                },
                KeyEvent { code, .. } => match code {
                    KeyCode::Home => self.cursor.x = 0,
                    KeyCode::End => self.cursor.x = self.current_row_len(),
                    KeyCode::Up => self.move_cursor(EditorKey::Up),
                    KeyCode::Down => self.move_cursor(EditorKey::Down),
                    KeyCode::Right => self.move_cursor(EditorKey::Right),
                    KeyCode::Left => self.move_cursor(EditorKey::Left),
                    KeyCode::PageDown | KeyCode::PageUp => {
                        let bounds = self.screen.bound();

                        match code {
                            KeyCode::PageUp => self.cursor.y = self.rowsoff,
                            KeyCode::PageDown => {
                                self.cursor.y =
                                    (self.rowsoff + bounds.y - 1).min(self.rows.len() as u16)
                            }
                            _ => panic!("rust page gg"),
                        }

                        for _ in 0..bounds.y {
                            self.move_cursor(if code == KeyCode::PageUp {
                                EditorKey::Up
                            } else {
                                EditorKey::Down
                            })
                        }
                    }
                    _ => {}
                },
            }
        } else {
            self.die("a error");
        }
        Ok(false)
    }

    pub fn refresh_screen(&mut self) -> Result<()> {
        self.scroll();
        self.screen.clear_screen()?;
        self.screen
            .draw_rows(&self.rows, self.rowsoff, self.coloff)?;
        self.screen.draw_bar(
            format!("{:20} - {} lines", self.filename, self.rows.len()),
            format!("{}/{}", self.cursor.x, self.cursor.y),
        )
    }

    pub fn die<S: Into<String>>(&mut self, message: S) {
        let _ = self.screen.clear_screen();
        let _ = terminal::disable_raw_mode();
        eprintln!("{} : {}", message.into(), errno());
        std::process::exit(1);
    }

    fn move_cursor(&mut self, key: EditorKey) {
        use EditorKey::*;

        match key {
            Left => {
                if self.cursor.x != 0 {
                    self.cursor.x -= 1;
                } else if self.cursor.y > 0 {
                    self.cursor.y -= 1;
                    self.cursor.x = self.rows[self.cursor.raw()].len() as u16;
                }
            }
            Right => {
                if (self.cursor.y as usize) < self.rows.len() {
                    let idx = self.cursor.raw();

                    if self.cursor.left_off(self.rows[idx].len()) {
                        self.cursor.x += 1;
                    } else if self.cursor.above(self.rows.len()) {
                        self.cursor.y += 1;
                        self.cursor.x = 0;
                    }
                }
            }
            Up => {
                self.cursor.y = self.cursor.y.saturating_sub(1);
            }
            Down if self.cursor.y < (self.rows.len()) as u16 => self.cursor.y += 1,
            _ => {}
        }
        let row_len = self.current_row_len();

        self.cursor.x = self.cursor.x.min(row_len);
    }

    fn scroll(&mut self) {
        self.render_x = if self.cursor.above(self.rows.len()) {
            self.rows[self.cursor.y as usize].cx_to_rx(self.cursor.x)
        } else {
            0
        };
        let bounds = self.screen.bound();
        if (self.cursor.y) < self.rowsoff {
            self.rowsoff = self.cursor.y;
        }
        if (self.cursor.y) >= (self.rowsoff + (bounds.y)) {
            self.rowsoff = self.cursor.y - bounds.y + 1;
        }
        if self.render_x < self.coloff {
            self.coloff = self.render_x;
        }
        if (self.render_x) >= (self.coloff + (bounds.x)) {
            self.coloff = self.render_x - bounds.x + 1;
        }
    }
    fn current_row_len(&self) -> u16 {
        if self.cursor.above(self.rows.len()) {
            self.rows[self.cursor.y as usize].len() as u16
        } else {
            0
        }
    }
}
