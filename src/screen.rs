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
    pub fn draw_rows(&mut self, rows: &[Raw], rowsoff: u16, coloff: u16) -> Result<()> {
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        for raw in 0..self.hight {
            let filerow = (raw + rowsoff) as usize;
            if filerow >= rows.len() {
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
                let mut len = rows[filerow].render_len();
                if len < coloff as usize {
                    continue;
                }
                len -= coloff as usize;
                let start = coloff as usize;
                let end = start
                    + if len >= self.width as usize {
                        self.width as usize
                    } else {
                        len
                    };
                self.stdout
                    .queue(cursor::MoveTo(0, raw))?
                    .queue(Print(rows[filerow].render[start..end].to_string()))?;
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

    pub fn move_to(
        &mut self,
        pos: &Position,
        render_x: u16,
        rowsoff: u16,
        coloff: u16,
    ) -> Result<()> {
        self.stdout
            .queue(cursor::MoveTo(render_x - coloff, pos.y - rowsoff))?;
        Ok(())
    }

    pub fn bound(&self) -> Position {
        Position {
            x: self.width,
            y: self.hight,
        }
    }
}
