use std::{
    io::{self, Stdout},
    sync::mpsc::{self, Receiver},
    thread,
};

use crossterm::{
    cursor,
    event::{read, Event, KeyCode},
    style::{self, Color, Stylize},
    QueueableCommand,
};

pub struct UI {
    pub stdout: Stdout,
    pub current_input: Option<KeyCode>,
    input_reciever: Receiver<Option<KeyCode>>,
}

impl UI {
    pub fn new() -> UI {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || loop {
            tx.send(read_inputs()).unwrap();
        });
        UI {
            stdout: io::stdout(),
            current_input: None,
            input_reciever: rx,
        }
    }
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
    pub fn debug_draw(&mut self, text: &str, line: u16) -> io::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, line))?
            .queue(style::PrintStyledContent((text).with(Color::Red)))?;
        Ok(())
    }
    pub fn update_input(&mut self) {
        self.current_input = match self.input_reciever.try_recv() {
            Ok(ko) => match ko {
                Some(k) => Some(k),
                None => self.current_input,
            },
            Err(_) => self.current_input,
        };
    }
}

fn read_inputs() -> Option<KeyCode> {
    match read() {
        Ok(event) => match event {
            Event::Key(event) => Some(event.code),
            _ => None,
        },
        _ => None,
    }
}
