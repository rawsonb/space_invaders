use core::slice;
use std::{
    borrow::BorrowMut,
    clone,
    io::{self, Stdout, Write},
    iter::Map,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant, SystemTime},
};

use crossterm::{
    cursor::{self, Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    queue,
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};

use crate::graphics::UI;

// Drawing too fast causes flickering
const MIN_FRAME_TIME: f64 = 0.04;

pub trait Update {
    fn update(&mut self, _delta: f64, _world: &mut World, _id: i64) {}
}

pub struct EntityData {
    pub entity: Box<dyn Update>,
    pub id: i64,
    pub tags: Vec<String>,
}

pub struct World {
    pub entities: Vec<EntityData>,
    removal_queue: Vec<i64>,
    // TODO This needs to be a struct oh my god.
    //The first vec are things just draw, the second
    //vec are things drawn last frame (returned when
    //querying map)
    map: Vec<Vec<(char, Color, Vec<i64>, Vec<i64>)>>,
    pub ui: UI,
    next_id: i64,
}

impl World {
    pub fn new(map_width: usize, map_height: usize) -> Self {
        World {
            entities: Vec::new(),
            map: vec![
                vec![
                    ('#', Color::Black, Vec::new(), Vec::new());
                    map_height
                ];
                map_width
            ],
            ui: UI::new(),
            next_id: 0,
            removal_queue: vec![],
        }
    }

    fn clear_map(&mut self) {
        let width = self.map[0].len();
        for row in self.map.iter_mut() {
            for col in row.iter_mut() {
                col.0 = ' ';
                col.1 = crossterm::style::Color::Black;
                col.3.clear();
                col.3.append(&mut col.2);
            }
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
        self.removal_queue.push(id);
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

    pub fn debug_draw(&mut self, line: u16, text: &str) -> io::Result<()> {
        let write_line = self.map[0].len() as u16 + line;
        self.ui
            .stdout
            .queue(MoveTo(0, write_line))?
            .queue(terminal::Clear(terminal::ClearType::CurrentLine))?;
        let _ = self.ui.debug_draw(text, write_line);
        Ok(())
    }

    fn draw_map(&mut self) {
        for r in 0..self.map.len() {
            for c in 0..self.map[0].len() {
                if !self.map[r][c].2.is_empty() || !self.map[r][c].3.is_empty()
                {
                    let _ = self.ui.terminal_draw(
                        self.map[r][c].0,
                        (r as u16, c as u16),
                        self.map[r][c].1,
                    );
                }
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
        let mut now = Instant::now();
        let mut delta: f64;
        loop {
            delta = now.elapsed().as_secs_f64();
            if delta < MIN_FRAME_TIME {
                thread::sleep(Duration::from_secs_f64(MIN_FRAME_TIME - delta));
                delta = now.elapsed().as_secs_f64();
            }
            now = Instant::now();
            self.ui.update_input();
            if self
                .ui
                .current_input
                .is_some_and(|x| x == KeyCode::Char('q'))
            {
                break;
            }
            self.update_entities(delta);
            self.ui.current_input = None;
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
        if !self.removal_queue.is_empty() {
            self.entities
                .retain(|x| !self.removal_queue.contains(&x.id));
            self.removal_queue.clear();
        }
        let entity_count = self.entities.len();
        let mut current_entity;
        for _i in 0..entity_count {
            current_entity = self.entities.remove(0);
            current_entity.entity.update(delta, self, current_entity.id);
            self.entities.push(current_entity);
        }

        self.draw_map();
        _ = self.ui.stdout.flush();
        self.clear_map();
    }

    pub fn query_map(&self, position: (usize, usize)) -> &Vec<i64> {
        return &self.map[position.0][position.1].3;
    }

    pub fn add_tag(&mut self, id: i64, tags: &str) {
        (&mut self.entities)
            .iter_mut()
            .filter(|x| x.id == id)
            .for_each(|x| x.tags.push(tags.to_string().split(" ").collect()));
    }
}
