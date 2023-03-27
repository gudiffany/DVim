use crossterm::event::{read, Event::*, KeyEvent};
use diffany::*;

pub struct Keyboard {}
impl Keyboard {
    pub fn read_key(&self) -> StdResult<KeyEvent, EditorResult> {
        loop {
            if let Ok(event) = read() {
                if let Key(key_event) = event {
                    return Ok(key_event);
                }
            } else {
                return Err(EditorResult::KeyRradFailed);
            }
        }
    }
}