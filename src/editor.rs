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
}

impl Editor {
    pub fn new<P: AsRef<Path>>(filename: P) -> Result<Self> {
        let file_line = std::fs::read_to_string(filename)
            .expect("unable to open file")
            .split('\n')
            .next()
            .unwrap()
            .to_string();

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
            rows: vec![file_line],
        })
    }
    pub fn start(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?; //原始模式
        loop {
            if self.refresh_screen().is_err() {
                self.die("unable to refresh screen");
            }
            self.screen.move_to(self.cursor)?;
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
        self.screen.clear_screen()?;
        self.screen.draw_rows(&self.rows)
    }

    pub fn die<S: Into<String>>(&mut self, message: S) {
        let _ = self.screen.clear_screen();
        let _ = terminal::disable_raw_mode();
        eprintln!("{} : {}", message.into(), errno());
        std::process::exit(1);
    }

    fn move_cursor(&mut self, key: EditorKey) {
        use EditorKey::*;

        let bounds = self.screen.bound();
        match key {
            Left => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            }
            Right if self.cursor.x <= bounds.x => self.cursor.x += 1,
            Up => {
                self.cursor.y = self.cursor.y.saturating_sub(1);
            }
            Down if self.cursor.y <= bounds.y => self.cursor.y += 1,
            _ => {}
        }
    }
}
