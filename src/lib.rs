pub type StdResult<T, E> = std::result::Result<T, E>;
pub enum EditorResult {
    KeyRradFailed,
}

#[derive(Default, Clone, Copy)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}