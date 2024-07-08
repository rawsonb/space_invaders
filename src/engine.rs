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

use crate::graphics::UI;

pub trait Update {
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {}
}

struct Entity {
    data: Box<dyn Update>,
    id: i64,
}

pub struct World {
    pub entities: Vec<Entity>,
    map: Vec<Vec<(char, Color)>>,
    pub ui: UI,
}

impl<'a> World {
    pub fn new(map_width: usize, map_height: usize) -> Self {
        World {
            entities: Vec::new(),
            map: vec![vec![('#', Color::Black); map_height]; map_width],
            ui: UI::new(),
        }
    }

    fn clear_map(&mut self) {
        let width = self.map[0].len();
        for row in self.map.iter_mut() {
            row.clear();
            row.append(&mut vec![(' ', Color::Black); width])
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

    pub fn debug_draw(&mut self, text: &str) {
        let _ = self.ui.debug_draw(text, self.map[0].len() as u16);
    }

    fn draw_map(&mut self) {
        for r in 0..self.map.len() {
            for c in 0..self.map[0].len() {
                let _ = self.ui.terminal_draw(
                    self.map[r][c].0,
                    (r as u16, c as u16),
                    self.map[r][c].1,
                );
            }
        }
    }

    pub fn init(&mut self) -> io::Result<()> {
        let _ = terminal::enable_raw_mode();

        self.ui
            .stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(Hide)?;

        let _ = self.game_loop();

        self.ui
            .stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(MoveTo(0, 0))?
            .execute(Show)?;

        let _ = terminal::disable_raw_mode();

        Ok(())
    }

    fn game_loop(&mut self) -> io::Result<()> {
        let mut now = SystemTime::now();
        loop {
            match now.elapsed() {
                Ok(elapsed) => {
                    now = SystemTime::now();
                    self.ui.update_input();
                    if self
                        .ui
                        .current_input
                        .is_some_and(|x| x == KeyCode::Char('q'))
                    {
                        break;
                    }

                    self.update_entities(elapsed.as_secs_f64());
                }
                Err(e) => {
                    println!("Error: {e:?}");
                }
            }
        }

        self.ui
            .stdout
            .execute(terminal::Clear(terminal::ClearType::All))?
            .execute(MoveTo(0, 0))?
            .execute(Show)?;
        let _ = terminal::disable_raw_mode();

        Ok(())
    }

    fn update_entities(&mut self, delta: f64) {
        let mut entity_queue: Vec<Entity> = Vec::new();
        entity_queue.append(&mut self.entities);
        for entity in entity_queue.iter_mut() {
            entity.data.update(delta, self, entity.id);
        }
        self.entities.append(&mut entity_queue);

        self.draw_map();
        _ = self.ui.stdout.flush();
        self.clear_map();
    }
}
