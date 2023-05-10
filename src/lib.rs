pub type StdResult<T, E> = std::result::Result<T, E>;
pub enum EditorResult {
    KeyRradFailed,
}

#[derive(Default, Clone, Copy)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn above(&self, raw: usize) -> bool {
        self.y < raw as u16
    }
    pub fn left_off(&self, col: usize) -> bool {
        self.x < col as u16
    }
    pub fn raw(&self) -> usize {
        self.y as usize
    }
}
