use std::collections::HashMap;

use crate::keyboard::*;
use crate::screen::*;
use crossterm::event::MediaKeyCode;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::{terminal, Result};
use diffany::*;
use errno::errno;
#[derive(Clone, Copy)]
enum EditorKey {
    ArrowUp,
    ArrowRight,
    ArrowLeft,
    ArrowDown,
    Pageup,
    PageDown,
}

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
    keymap: HashMap<char, EditorKey>,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let mut keymap = HashMap::new();
        keymap.insert('w', EditorKey::ArrowUp);
        keymap.insert('a', EditorKey::ArrowLeft);
        keymap.insert('s', EditorKey::ArrowDown);
        keymap.insert('d', EditorKey::ArrowRight);
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Position::default(),
            keymap,
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
                    KeyCode::Up => self.move_cursor(EditorKey::ArrowUp),
                    KeyCode::Down => self.move_cursor(EditorKey::ArrowDown),
                    KeyCode::Right => self.move_cursor(EditorKey::ArrowRight),
                    KeyCode::Left => self.move_cursor(EditorKey::ArrowLeft),
                    KeyCode::PageDown | KeyCode::PageUp => {
                        let bounds = self.screen.bound();
                        for _ in 0..bounds.y {
                            self.move_cursor(if code == KeyCode::PageUp {
                                EditorKey::ArrowUp
                            } else {
                                EditorKey::ArrowDown
                            })
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            self.die("a error");
        }
        Ok(false)
    }

    pub fn refresh_screen(&mut self) -> Result<()> {
        self.screen.clear_screen()?;
        self.screen.draw_rows()
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
            ArrowLeft => {
                self.cursor.x = self.cursor.x.saturating_sub(1);
            }
            ArrowRight if self.cursor.x <= bounds.x => self.cursor.x += 1,
            ArrowUp => {
                self.cursor.y = self.cursor.y.saturating_sub(1);
            }
            ArrowDown if self.cursor.y <= bounds.y => self.cursor.y += 1,
            _ => {}
        }
    }
}
