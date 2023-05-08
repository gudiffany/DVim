use crossterm::{cursor, style::Print, terminal, QueueableCommand, Result};
use diffany::*;
use std::{
    env,
    io::{stdout, Stdout, Write},
};
pub struct Screen {
    stdout: Stdout,
    width: u16,
    hight: u16,
}
impl Screen {
    pub fn new() -> Result<Self> {
        let (conlumns, rows) = crossterm::terminal::size()?;

        Ok(Self {
            width: conlumns,
            hight: rows,
            stdout: stdout(),
        })
    }
    pub fn draw_rows(&mut self, rows: &[String]) -> Result<()> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        for raw in 0..self.hight {
            if raw >= rows.len() as u16 {
                if rows.is_empty() && raw == self.hight / 3 {
                    let mut welcome = format!("diffany --version {VERSION}");
                    welcome.truncate(self.width as usize);
                    if welcome.len() < self.width as usize {
                        let leftmost = ((self.width as usize - welcome.len()) / 2) as u16;
                        self.stdout
                            .queue(cursor::MoveTo(0, raw))?
                            .queue(Print("~".to_string()))?
                            .queue(cursor::MoveTo(leftmost, raw))?
                            .queue(Print(welcome))?;
                    } else {
                        self.stdout
                            .queue(cursor::MoveTo(0, raw))?
                            .queue(Print(welcome))?;
                    }
                } else {
                    self.stdout
                        .queue(cursor::MoveTo(0, raw))?
                        .queue(Print("~".to_string()))?;
                }
            } else {
                let len = rows[0].len().min(self.width as usize);
                self.stdout
                    .queue(cursor::MoveTo(0, raw))?
                    .queue(Print(rows[0][0..len].to_string()))?;
            }
        }
        Ok(())
    }
    pub fn clear_screen(&mut self) -> Result<()> {
        self.stdout
            .queue(terminal::Clear(terminal::ClearType::All))?
            .queue(cursor::MoveTo(0, 0))?;
        Ok(())
    }
    pub fn flush(&mut self) -> Result<()> {
        self.stdout.flush()
    }
    // pub fn cursor_position(&self) -> Result<(u16, u16)> {
    //     cursor::position()
    // }

    pub fn move_to(&mut self, pos: Position) -> Result<()> {
        self.stdout.queue(cursor::MoveTo(pos.x, pos.y))?;
        Ok(())
    }

    pub fn bound(&self) -> Position {
        Position {
            x: self.width,
            y: self.hight,
        }
    }
}
