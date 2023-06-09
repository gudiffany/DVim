use crate::keyboard::*;
use crate::raw::*;
use crate::screen::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::{terminal, Result};
use diffany::*;
use errno::errno;
use std::path::Path;
use std::time::Duration;
use std::time::Instant;

const QUIT_TIME: usize = 3;
#[derive(Clone, Copy)]
enum EditorKey {
    Up,
    Right,
    Left,
    Down,
}

enum PromptKey {
    Enter,
    Escape,
    Char(char),
    Pre,
    Next,
}

enum SearchDirection {
    Backword,
    Forword,
}

pub struct Editor {
    filename: String,
    screen: Screen,
    status_msg: String,
    status_time: Instant,
    keyboard: Keyboard,
    cursor: Position,
    render_x: u16,
    rows: Vec<Raw>,
    rowsoff: u16,
    coloff: u16,
    dirty: usize,
    quit_time: usize,
    last_match: Option<usize>,
    direction: SearchDirection,
    save_hl: Option<usize>,
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
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            status_msg: String::from("HELP: Ctrl+q = quit && Ctrl+s = save && Ctrl+f = search"),
            status_time: Instant::now(),
            cursor: Position::default(),
            rows: if data.is_empty() {
                Vec::new()
            } else {
                let v = Vec::from(data);
                let mut rows = Vec::new();
                for raw in v {
                    rows.push(Raw::new(raw))
                }
                if rows.last().unwrap().len() == 0 {
                    rows.pop();
                }
                rows
            },
            rowsoff: 0,
            coloff: 0,
            render_x: 0,
            filename: filename.into(),
            dirty: 0,
            quit_time: QUIT_TIME,
            last_match: None,
            direction: SearchDirection::Forword,
            save_hl: None,
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
                } => {
                    if self.dirty > 0 && self.quit_time > 0 {
                        self.set_status_msg(format!(
                            "warning!!! filename unsaved changes.Press Ctrl + q {} more time to quit",
                            self.quit_time,
                        ));
                        self.quit_time -= 1;
                        return Ok(false);
                    } else {
                        return Ok(true);
                    }
                }
                KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.save(),

                KeyEvent {
                    code: KeyCode::Char('f'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => self.find(),

                KeyEvent {
                    code: KeyCode::Char('l'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Esc, ..
                } => {}

                KeyEvent {
                    code: KeyCode::Char('h'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Delete,
                    ..
                } => {
                    if let KeyEvent {
                        code: KeyCode::Delete,
                        ..
                    } = c
                    {
                        self.move_cursor(EditorKey::Right);
                    }
                    self.del_char();
                } // TODO

                KeyEvent {
                    code: KeyCode::Char(key),
                    modifiers: KeyModifiers::NONE,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char(key),
                    modifiers: KeyModifiers::SHIFT,
                    ..
                } => self.insert_char(key),

                KeyEvent { code, .. } => match code {
                    KeyCode::Enter => {
                        self.insert_newlines();
                    } // TODO
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
        self.quit_time = QUIT_TIME;
        Ok(false)
    }

    pub fn refresh_screen(&mut self) -> Result<()> {
        self.scroll();
        self.screen.clear_screen()?;
        self.screen
            .draw_rows(&self.rows, self.rowsoff, self.coloff)?;

        if !self.status_msg.is_empty() && self.status_time.elapsed() > Duration::from_secs(5) {
            self.status_msg.clear();
        }

        self.screen.draw_bar(
            format!(
                "{:20} - {} lines {} {}",
                if self.filename.is_empty() {
                    "[No None]"
                } else {
                    &self.filename
                },
                self.rows.len(),
                self.dirty,
                if self.dirty > 0 { "(modified)" } else { "" }
            ),
            format!("{}/{}", self.cursor.x, self.cursor.y),
            &self.status_msg,
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

    fn insert_char(&mut self, c: char) {
        if !self.cursor.above(self.rows.len()) {
            self.insert_row(self.rows.len(), String::new());
        }
        self.rows[self.cursor.y as usize].insert_char(self.cursor.x as usize, c);
        self.cursor.x += 1;
        self.dirty += 1;
    }

    fn del_char(&mut self) {
        if !self.cursor.above(self.rows.len()) {
            return;
        }
        if self.cursor.x == 0 && self.cursor.y == 0 {
            return;
        }
        let cur_row = self.cursor.y as usize;
        if self.cursor.x > 0 {
            if self.rows[cur_row].del_char(self.cursor.x as usize - 1) {
                self.dirty += 1;
                self.cursor.x -= 1;
            }
        } else {
            self.cursor.x = self.rows[cur_row - 1].len() as u16;
            if let Some(row) = self.del_row(cur_row) {
                self.rows[cur_row - 1].append_string(&row);
                self.cursor.y -= 1;
                self.dirty += 1;
            }
        }
    }

    fn insert_newlines(&mut self) {
        let row = self.cursor.y as usize;

        if self.cursor.x == 0 {
            self.insert_row(row, String::from(""))
        } else {
            let new_row = self.rows[row].split(self.cursor.x as usize);
            self.insert_row(row + 1, new_row);
        }
        self.cursor.y += 1;
        self.cursor.x = 0;
    }

    fn insert_row(&mut self, at: usize, s: String) {
        if at > self.rows.len() {
            return;
        }

        self.rows.insert(at, Raw::new(s));
        self.dirty += 1;
    }

    fn del_row(&mut self, at: usize) -> Option<String> {
        if at >= self.rows.len() {
            None
        } else {
            self.dirty += 1;
            Some(self.rows.remove(at).chars)
        }
    }

    fn rows_to_string(&self) -> String {
        let mut buf = String::new();
        for r in &self.rows {
            buf.push_str(r.chars.as_str());
            buf.push('\n');
        }
        buf
    }

    fn save(&mut self) {
        if self.filename.is_empty() {
            if let Some(filename) = self.prompt("save as", None) {
                self.filename = filename;
            } else {
                self.set_status_msg("save aborted");
                return;
            }
        }

        let buf = self.rows_to_string();
        let len = buf.as_bytes().len();
        if std::fs::write(&self.filename, &buf).is_ok() {
            self.dirty = 0;
            self.set_status_msg(&format!("{len} bytes written to disk"));
        } else {
            self.set_status_msg(&format!("can't save I/O error: {}", errno()));
        }
    }

    fn prompt(
        &mut self,
        prompt: &str,
        callback: Option<fn(&mut Editor, &str, PromptKey)>,
    ) -> Option<String> {
        let mut buf = String::from("");
        loop {
            self.set_status_msg(&format!("{}:{}", prompt, buf));
            let _ = self.refresh_screen();
            let _ = self.screen.flush();
            if let Ok(c) = self.keyboard.read_key() {
                let mut prompt_key: Option<PromptKey> = None;

                match c {
                    KeyEvent {
                        code: KeyCode::Enter,
                        ..
                    } => {
                        if let Some(callback) = callback {
                            callback(self, &buf, PromptKey::Enter);
                        }
                        self.set_status_msg("");
                        return Some(buf);
                    }
                    KeyEvent {
                        code: KeyCode::Esc, ..
                    } => {
                        if let Some(callback) = callback {
                            callback(self, &buf, PromptKey::Escape);
                        }
                        self.set_status_msg("");
                        return None;
                    }

                    KeyEvent {
                        code: KeyCode::Char('h'),
                        modifiers: KeyModifiers::CONTROL,
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Backspace,
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Delete,
                        ..
                    } => {
                        buf.pop();
                    }

                    KeyEvent {
                        code: KeyCode::Up, ..
                    }
                    | KeyEvent {
                        code: KeyCode::Left,
                        ..
                    } => {
                        if let Some(callback) = callback {
                            callback(self, &buf, PromptKey::Pre);
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Down,
                        ..
                    }
                    | KeyEvent {
                        code: KeyCode::Right,
                        ..
                    } => {
                        if let Some(callback) = callback {
                            callback(self, &buf, PromptKey::Next);
                        }
                    }

                    KeyEvent {
                        code: KeyCode::Char(ch),
                        modifiers: modif,
                        ..
                    } => {
                        if matches!(modif, KeyModifiers::NONE | KeyModifiers::SHIFT) {
                            prompt_key = Some(PromptKey::Char(ch));
                            buf.push(ch);
                        }
                    }
                    _ => {}
                }
                if let Some(callback) = callback {
                    if let Some(key) = prompt_key {
                        callback(self, &buf, key);
                    }
                }
            }
        }
    }

    fn find(&mut self) {
        let (saved_position, saved_coloff, saved_rowsoff) =
            (self.cursor, self.coloff, self.rowsoff);

        if self
            .prompt("Search (ESC to cancel)", Some(Editor::find_callback))
            .is_none()
        {
            self.cursor = saved_position;
            self.coloff = saved_coloff;
            self.rowsoff = saved_rowsoff;
        }
    }

    fn find_callback(&mut self, query: &str, event: PromptKey) {
        if let Some(save_hl) = self.save_hl {
            self.rows[save_hl].reset_hightlight();
            self.save_hl = None;
        }
        match event {
            PromptKey::Enter | PromptKey::Escape => {
                self.last_match = None;
                self.direction = SearchDirection::Forword;
            }
            PromptKey::Next => self.direction = SearchDirection::Forword,
            PromptKey::Pre => self.direction = SearchDirection::Backword,
            _ => {
                self.last_match = None;
                self.direction = SearchDirection::Forword;
            }
        }
        let mut current = if let Some(line) = self.last_match {
            line
        } else {
            self.direction = SearchDirection::Forword;
            self.rows.len()
        };

        for _ in 0..self.rows.len() {
            match self.direction {
                SearchDirection::Forword => {
                    current += 1;
                    if current >= self.rows.len() {
                        current = 0;
                    }
                }
                SearchDirection::Backword => {
                    if current == 0 {
                        current = self.rows.len() - 1;
                    } else {
                        current -= 1;
                    }
                }
            }
            if let Some(m) = self.rows[current]
                .render
                .match_indices(query)
                .take(1)
                .next()
            {
                let start = m.0;
                drop(m);
                self.last_match = Some(current);
                self.cursor.y = current as u16;
                self.cursor.x = self.rows[current].rx_to_cx(start);
                self.rowsoff = self.rows.len() as u16;
                self.rows[current].hightlight_match(start, query.len());
                self.save_hl = Some(current);
                break;
            }
        }
    }

    pub fn set_status_msg<T: Into<String>>(&mut self, message: T) {
        self.status_time = Instant::now();
        self.status_msg = message.into();
    }
}
