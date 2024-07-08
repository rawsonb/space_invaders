use std::io::{self, Stdout};

use crossterm::{
    cursor,
    style::{self, Color, Stylize},
    QueueableCommand,
};

pub struct UI {
    pub stdout: Stdout,
}

impl Default for UI {
    fn default() -> Self {
        UI {
            stdout: io::stdout(),
        }
    }
}

impl UI {
    pub fn terminal_draw(
        &mut self,
        character: char,
        position: (u16, u16),
        color: Color,
    ) -> io::Result<()> {
        self.stdout
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
