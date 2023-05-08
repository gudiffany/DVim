use crossterm::{Result};
mod editor;
use editor::Editor;
mod screen;
mod keyboard;
fn main() -> Result<()> {
    let mut editor = Editor::new("input.txt")?;
    editor.start()?;
    Ok(())
}
