use std::io::{self, Stdout};

use crossterm::{
    cursor,
    style::{self, Color, Stylize},
    QueueableCommand,
};

pub struct GUI {
    pub stdout: Stdout,
}

impl Default for GUI {
    fn default() -> Self {
        GUI {
            stdout: io::stdout(),
        }
    }
}

impl GUI {
    pub fn terminal_draw(
        character: char,
        position: (u16, u16),
        color: Color,
    ) -> io::Result<()> {
        io::stdout()
            .queue(cursor::MoveTo(position.0, position.1))?
            .queue(style::PrintStyledContent((character).with(color)))?;
        Ok(())
    }
    pub fn debug_draw(text: &str, line: u16) -> io::Result<()> {
        io::stdout()
            .queue(cursor::MoveTo(0, line))?
            .queue(style::PrintStyledContent((text).with(Color::Red)))?;
        Ok(())
    }
}
