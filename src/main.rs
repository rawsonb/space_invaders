use std::vec;

use crossterm::{cursor::position, event::KeyCode};
use engine::{EntityData, Update, World};

mod engine;
mod graphics;

const MAP_HEIGHT: u16 = 15;
const MAP_WIDTH: u16 = 25; // in characters
const BULLET_SPEED: f64 = 3.5;
const PLAYER_SPEED: f64 = 4.5; // characters per second

fn main() {
    let mut world = World::new(MAP_WIDTH as usize, MAP_HEIGHT as usize);
    world.add_entity(Ship {
        position: (12.0, 13.0),
        velocity: 0.0,
    });
    world.add_entity(Wall {});
    let _ = world.init();
}

struct Ship {
    position: (f64, f64),
    velocity: f64,
}

impl Update for Ship {
    fn update(
        &mut self,
        delta: f64,
        world: &mut World,
        id: i64,
    ) -> Option<fn(&mut Vec<EntityData>)> {
        world.debug_draw(format!("Velocity: {:?}", self.velocity).as_str());
        world.debug_draw(
            format!("\n X_Position: {:?}", self.position.0).as_str(),
        );
        world.debug_draw(
            format!("\n\n Last Input: {:?}", world.ui.last_input).as_str(),
        );
        world.debug_draw(
            format!("\n\n\n Current Input: {:?}", world.ui.current_input)
                .as_str(),
        );
        world.debug_draw(
            format!("\n\n\n\n Num Entities: {:?}", world.entities().len())
                .as_str(),
        );
        self.position.0 += self.velocity * delta;
        self.position.0 =
            f64::clamp(self.position.0, 1.0, MAP_WIDTH as f64 - 2.0);

        match world.ui.current_input {
            Some(KeyCode::Left) => {
                if world.ui.last_input.is_some_and(|x| x == KeyCode::Right) {
                    self.velocity = 0.0;
                    world.ui.current_input = None;
                } else {
                    self.velocity = -PLAYER_SPEED;
                }
            }
            Some(KeyCode::Right) => {
                if world.ui.last_input.is_some_and(|x| x == KeyCode::Left) {
                    self.velocity = 0.0;
                    world.ui.current_input = None;
                } else {
                    self.velocity = PLAYER_SPEED;
                }
            }
            Some(KeyCode::Up) => {
                world.add_entity(Bullet {
                    position: (
                        self.position.0 as f64,
                        self.position.1 as f64 - 1.0,
                    ),
                });
                self.velocity = 0.0;
                world.ui.current_input = None;
            }
            _ => {
                self.velocity = 0.0;
                world.ui.current_input = None;
            }
        }
        world.draw(
            (
                self.position.0.round() as u16,
                self.position.1.round() as u16,
            ),
            '^',
            crossterm::style::Color::Green,
            id,
        );
        None
    }
}

struct Wall {}

impl Update for Wall {
    fn update(
        &mut self,
        _delta: f64,
        world: &mut World,
        id: i64,
    ) -> Option<fn(&mut Vec<EntityData>)> {
        for r in 0..MAP_WIDTH {
            for c in 0..MAP_HEIGHT {
                if r == 0 || c == 0 || r == MAP_WIDTH - 1 || c == MAP_HEIGHT - 1
                {
                    world.draw((r, c), '#', crossterm::style::Color::Grey, id);
                }
            }
        }
        None
    }
}

struct Bullet {
    position: (f64, f64),
}

impl Update for Bullet {
    fn update(
        &mut self,
        delta: f64,
        world: &mut World,
        id: i64,
    ) -> Option<fn(&mut Vec<EntityData>)> {
        self.position =
            (self.position.0, self.position.1 - delta * BULLET_SPEED);
        let target_pos =
            (self.position.0, self.position.1 - delta * BULLET_SPEED);
        if target_pos.1 < 1.0 {
            world.remove_entity(id);
        } else {
            world.draw(
                (
                    self.position.0.round() as u16,
                    self.position.1.round() as u16,
                ),
                '*',
                crossterm::style::Color::Red,
                id,
            );
        }
        None
    }
}

struct Barrier {
    position: (u16, u16),
}

impl Update for Barrier {
    fn update(
        &mut self,
        _delta: f64,
        world: &mut World,
        id: i64,
    ) -> Option<fn(&mut Vec<EntityData>)> {
        world.draw(self.position, '#', crossterm::style::Color::Yellow, id);
        None
    }
}
