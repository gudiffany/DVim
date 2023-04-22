use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::{terminal, Result};
use errno::errno;
use diffany::*;
use crate::keyboard::*;
use crate::screen::*;

pub struct Editor {
    screen: Screen,
    keyboard: Keyboard,
    cursor: Position,
}

impl Editor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            keyboard: Keyboard {},
            cursor: Position {x:0 , y:0},
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
        let c = self.keyboard.read_key();

        match c {
            Ok(KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: KeyModifiers::CONTROL,
                kind: _,
                state: _,
            }) => Ok(true),
            Err(EditorResult::KeyRradFailed) => {
                self.die("unable open the keyboard !");
                Ok(false)
            }
            _ => Ok(false),
        }
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
}
