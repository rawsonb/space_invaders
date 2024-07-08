use std::{
    io::{self, Stdout, Write},
    sync::mpsc::{self, Receiver},
    thread,
    time::SystemTime,
};

use crossterm::{
    cursor::{self, Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};

pub trait Update {
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {}
}

struct Entity {
    data: Box<dyn Update>,
    id: i64,
}

pub struct World {
    entities: Vec<Entity>,
    map: Vec<Vec<(char, Color)>>,
    pub current_input: Option<KeyCode>,
    gui: GUI,
}

impl<'a> World {
    pub fn new(map_width: usize, map_height: usize) -> Self {
        World {
            entities: Vec::new(),
            map: vec![vec![(' ', Color::Black); map_height]; map_width],
            current_input: None,
            gui: GUI::default(),
        }
    }

    pub fn add_entity(&'_ mut self, entity: impl Update + 'static) {
        self.entities.push(Entity {
            data: Box::new(entity),
            id: self.entities.len() as i64 + 1,
        })
    }

    pub fn remove_entity(&mut self, id: i64) {
        for i in 0..self.entities.len() {
            if self.entities[i].id == id {
                self.entities.swap_remove(i);
                return;
            }
        }
    }

    pub fn draw(
        &mut self,
        character: char,
        position: (u16, u16),
        color: Color,
    ) {
        let mut position = position;
        position.0 = position.0.clamp(0, self.map.len() as u16 - 1);
        position.1 = position.1.clamp(0, self.map[0].len() as u16 - 1);
        self.map[position.0 as usize][position.1 as usize] = (character, color);
    }

    pub fn debug_draw(&mut self, text: &str) ->{
        self.gui.debug_draw(text, self.map[0].len());
    }

    fn draw_map(&mut self) {
        for r in 0..self.map.len() {
            for c in 0..self.map[0].len() {
                let _ = GUI::terminal_draw(
                    self.map[r][c].0,
                    (r as u16, c as u16),
                    self.map[r][c].1,
                );
            }
        }
    }

    fn clear_map(&mut self) {
        let width = self.map[0].len();
        for row in self.map.iter_mut() {
            row.clear();
            row.append(&mut vec![(' ', Color::Black); width])
        }
    }

    pub fn init(&mut self) -> io::Result<()> {
        let _ = terminal::enable_raw_mode();

        self.gui
            .stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(Hide)?;

        let (tx, rx) = mpsc::channel();

        thread::spawn(move || loop {
            tx.send(read_inputs()).unwrap();
        });

        self.game_loop(rx);

        self.gui
            .stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(MoveTo(0, 0))?
            .execute(Show)?;

        let _ = terminal::disable_raw_mode();

        Ok(())
    }

    fn game_loop(&mut self, rx: Receiver<Option<KeyCode>>) -> io::Result<()> {
        let mut now = SystemTime::now();
        loop {
            match now.elapsed() {
                Ok(elapsed) => {
                    now = SystemTime::now();

                    self.current_input = match rx.try_recv() {
                        Ok(ko) => match ko {
                            Some(k) => Some(k),
                            None => self.current_input,
                        },
                        Err(_) => self.current_input,
                    };

                    if self
                        .current_input
                        .is_some_and(|x| x == KeyCode::Char('q'))
                    {
                        break;
                    }

                    self.update(elapsed.as_secs_f64());
                }
                Err(e) => {
                    println!("Error: {e:?}");
                }
            }
        }

        self.gui
            .stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(MoveTo(0, 0))?
            .execute(Show)?;
        let _ = terminal::disable_raw_mode();

        Ok(())
    }

    fn update(&mut self, delta: f64) {
        let mut entity_queue: Vec<Entity> = Vec::new();
        entity_queue.append(&mut self.entities);
        for entity in entity_queue.iter_mut() {
            entity.data.update(delta, self, entity.id);
        }
        self.entities.append(&mut entity_queue);

        self.draw_map();
        _ = self.gui.stdout.flush();
        self.clear_map();
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

pub struct GUI {
    stdout: Stdout,
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
    pub fn debug_draw(&mut self, text: &str, line: u16) -> io::Result<()> {
        self.stdout
            .queue(cursor::MoveTo(0, line))?
            .queue(style::PrintStyledContent((text).with(Color::Red)))?;
        Ok(())
    }
}
