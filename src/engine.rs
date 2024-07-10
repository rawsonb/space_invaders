use core::slice;
use std::{
    borrow::BorrowMut,
    clone,
    io::{self, Stdout, Write},
    iter::Map,
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
    fn update(
        &mut self,
        _delta: f64,
        _world: &mut World,
        _id: i64,
    ) -> Option<fn(&mut Vec<EntityData>)> {
        None
    }
}

pub struct EntityData {
    pub entity: Box<dyn Update>,
    pub id: i64,
    pub tags: Vec<String>,
}

pub struct World {
    entities: Vec<EntityData>,
    map: Vec<Vec<(char, Color, Vec<i64>)>>,
    pub ui: UI,
    next_id: i64,
}

impl World {
    pub fn new(map_width: usize, map_height: usize) -> Self {
        World {
            entities: Vec::new(),
            map: vec![
                vec![('#', Color::Black, Vec::new()); map_height];
                map_width
            ],
            ui: UI::new(),
            next_id: 0,
        }
    }

    fn clear_map(&mut self) {
        let width = self.map[0].len();
        for row in self.map.iter_mut() {
            row.clear();
            row.append(&mut vec![(' ', Color::Black, Vec::new()); width])
        }
    }

    pub fn add_entity(&mut self, entity_data: impl Update + 'static) {
        self.entities.push(EntityData {
            entity: Box::new(entity_data),
            id: self.next_id,
            tags: vec![],
        });
        self.next_id += 1;
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
        position: (u16, u16),
        character: char,
        color: Color,
        id: i64,
    ) {
        let mut position = position;
        position.0 = position.0.clamp(0, self.map.len() as u16 - 1);
        position.1 = position.1.clamp(0, self.map[0].len() as u16 - 1);
        let pos = &mut self.map[position.0 as usize][position.1 as usize];
        pos.0 = character;
        pos.1 = color;
        pos.2.push(id);
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
        let entity_count = self.entities.len();
        for i in 0..entity_count {
            let mut current_entity = self.entities.swap_remove(0);
            current_entity.entity.update(delta, self, current_entity.id);
            self.entities.push(current_entity);
        }

        self.draw_map();
        _ = self.ui.stdout.flush();
        self.clear_map();
    }

    pub fn entities(&self) -> &Vec<EntityData> {
        return &self.entities;
    }

    fn map_query(
        &mut self,
        position: (usize, usize),
    ) -> (char, Color, Vec<i64>) {
        return self.map[position.0][position.1].clone();
    }

    pub fn add_tag(&mut self, id: i64, tags: &str) {
        (&mut self.entities)
            .iter_mut()
            .filter(|x| x.id == id)
            .for_each(|x| x.tags.push(tags.to_string().split(" ").collect()));
    }
}
