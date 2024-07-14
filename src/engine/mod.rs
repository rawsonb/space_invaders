use core::slice;
use crossterm::{
    cursor::{self, Hide, MoveTo, Show},
    event::{read, Event, KeyCode, KeyEvent},
    queue,
    style::{self, Color, Stylize},
    terminal, ExecutableCommand, QueueableCommand,
};
use graphics::UI;
//use space_invaders_macros::Component;
use std::{
    any::{Any, TypeId},
    borrow::BorrowMut,
    clone,
    collections::HashMap,
    io::{self, Stdout, Write},
    iter::Map as IterMap,
    path::Component,
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant, SystemTime},
};
pub mod graphics;
// Drawing too fast causes flickering
const MIN_FRAME_TIME: f64 = 0.04;
pub trait Update {
    fn update(&mut self, delta: f64, world: &mut World, id: i64);
}

pub struct EntityData {
    pub entity: Box<dyn Update>,
    pub id: i64,
    components: HashMap<TypeId, Box<dyn Any>>,
}

pub struct World {
    pub entities: Vec<EntityData>,
    removal_queue: Vec<i64>,
    pub map: Map,
    pub ui: UI,
    next_id: i64,
}

impl World {
    pub fn new(map_width: usize, map_height: usize) -> Self {
        World {
            entities: Vec::new(),
            map: Map::new(map_width, map_height),
            ui: UI::new(),
            next_id: 0,
            removal_queue: vec![],
        }
    }

    pub fn add_entity(&mut self, entity_data: impl Update + 'static) {
        self.entities.push(EntityData {
            entity: Box::new(entity_data),
            id: self.next_id,
            components: HashMap::new(),
        });
        self.next_id += 1;
    }

    pub fn remove_entity(&mut self, id: i64) {
        self.removal_queue.push(id);
    }

    fn draw(&mut self) {
        let map = &self.map;
        for c in 0..map.width {
            for r in 0..map.height {
                if !map.tiles[c][r].current_contents.is_empty()
                    || !map.tiles[c][r].previous_contents.is_empty()
                {
                    let _ = self.ui.terminal_draw(
                        (c as u16, r as u16),
                        map.tiles[c][r].display_character,
                        map.tiles[c][r].color,
                    );
                }
            }
        }
    }

    pub fn query_map(&mut self, position: (u16, u16)) -> Vec<&mut EntityData> {
        let mut world_entities = Vec::new();
        for entity in self.entities.iter_mut() {
            if self.map.tiles[position.0 as usize][position.1 as usize]
                .previous_contents
                .contains(&entity.id)
            {
                world_entities.push(entity);
            }
        }
        return world_entities;
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

        self.draw();
        _ = self.ui.stdout.flush();
        self.map.clear();
    }
    // NEED TO CLEAN THIS UP BUT IM TIRED
    pub fn get_component<T: 'static>(&mut self, id: i64) -> Option<&mut T> {
        match self.entities.iter_mut().find(|x| x.id == id) {
            Some(x) => {
                let type_id = TypeId::of::<T>();
                let component = x.components.get_mut(&type_id);
                match component {
                    Some(cb) => cb.downcast_mut::<T>(),
                    None => None,
                }
            }
            None => None,
        }
    }

    pub fn add_component<T: 'static>(&mut self, id: i64, component: T) {
        match self.entities.iter_mut().find(|x| x.id == id) {
            Some(x) => {
                x.components.insert(TypeId::of::<T>(), Box::new(component));
            }
            None => {}
        }
    }
}

pub struct Map {
    width: usize,
    height: usize,
    tiles: Vec<Vec<MapTile>>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Map {
            width,
            height,
            tiles: vec![
                vec![
                    MapTile {
                        display_character: '#',
                        color: Color::Black,
                        current_contents: Vec::new(),
                        previous_contents: Vec::new()
                    };
                    height
                ];
                width
            ],
        }
    }

    pub fn clear(&mut self) {
        for col in self.tiles.iter_mut() {
            for tile in col.iter_mut() {
                tile.display_character = ' ';
                tile.color = crossterm::style::Color::Black;
                tile.previous_contents.clear();
                tile.previous_contents.append(&mut tile.current_contents);
            }
        }
    }

    pub fn write(
        &mut self,
        position: (u16, u16),
        character: char,
        color: Color,
        id: i64,
    ) {
        let mut position = position;
        position.0 = position.0.clamp(0, self.width as u16 - 1);
        position.1 = position.1.clamp(0, self.height as u16 - 1);
        let pos = &mut self.tiles[position.0 as usize][position.1 as usize];
        pos.display_character = character;
        pos.color = color;
        pos.current_contents.push(id);
    }
}

#[derive(Clone)]
pub struct MapTile {
    display_character: char,
    color: Color,
    current_contents: Vec<i64>, // by ids
    previous_contents: Vec<i64>,
}
