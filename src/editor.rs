use crate::keyboard::*;
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
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
    keymap: HashMap<char, EditorKey>,
    rows: Vec<String>,
    rowsoff: u16,
    coloff: u16,
}

impl Editor {
    pub fn with_file<P: AsRef<Path>>(filename: P) -> Result<Self> {
        let file_line = std::fs::read_to_string(filename)
            .expect("unable to open file")
            .split('\n')
            .map(|x| x.into())
            .collect::<Vec<String>>();
        Editor::build(&file_line)
    }
    pub fn new() -> Result<Self> {
        Editor::build(&[])
    }

    pub fn build(data: &[String]) -> Result<Self> {
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
                Vec::from(data)
            },
            rowsoff: 0,
            coloff: 0,
        })
    }
    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?; //原始模式
        loop {
            if self.refresh_screen().is_err() {
                self.die("unable to refresh screen");
            }
            self.screen
                .move_to(&self.cursor, self.rowsoff, self.coloff)?;
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
                    KeyCode::End => self.cursor.x = self.screen.bound().x - 1,
                    KeyCode::Up => self.move_cursor(EditorKey::Up),
                    KeyCode::Down => self.move_cursor(EditorKey::Down),
                    KeyCode::Right => self.move_cursor(EditorKey::Right),
                    KeyCode::Left => self.move_cursor(EditorKey::Left),
                    KeyCode::PageDown | KeyCode::PageUp => {
                        let bounds = self.screen.bound();
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
        self.screen.draw_rows(&self.rows, self.rowsoff, self.coloff)
    }

    pub fn die<S: Into<String>>(&mut self, message: S) {
        let _ = self.screen.clear_screen();
        let _ = terminal::disable_raw_mode();
        eprintln!("{} : {}", message.into(), errno());
        std::process::exit(1);
    }

    fn move_cursor(&mut self, key: EditorKey) {
        use EditorKey::*;

        let row_idx = if self.cursor.y as usize >= self.rows.len() {
            None
        } else {
            Some(self.cursor.y as usize)
        };
        match key {
            Left => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            }
            Right => {
                if let Some(idx) = row_idx {
                    if (self.rows[idx].len() as u16) > self.cursor.x {
                        self.cursor.x += 1;
                    }
                }
            }
            Up => {
                self.cursor.y = self.cursor.y.saturating_sub(1);
            }
            Down if self.cursor.y < (self.rows.len()) as u16 => self.cursor.y += 1,
            _ => {}
        }
    }

    fn scroll(&mut self) {
        let bounds = self.screen.bound();
        if (self.cursor.y) < self.rowsoff {
            self.rowsoff = self.cursor.y;
        }
        if (self.cursor.y) >= (self.rowsoff + (bounds.y)) {
            self.rowsoff = self.cursor.y - bounds.y + 1;
        }
        if self.cursor.x < self.coloff {
            self.coloff = self.cursor.x;
        }
        if (self.cursor.x) >= (self.coloff + (bounds.x)) {
            self.coloff = self.cursor.x - bounds.x + 1;
        }
    }
}
