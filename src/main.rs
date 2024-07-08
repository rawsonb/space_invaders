use crossterm::{cursor::position, event::KeyCode};
use engine::{Update, World};

mod engine;
mod graphics;

const MAP_HEIGHT: u16 = 15;
const MAP_WIDTH: u16 = 25;
const BULLET_SPEED: f64 = 3.5;
const PLAYER_SPEED: f64 = 4.5;

fn main() {
    let mut world = World::new(MAP_WIDTH as usize, MAP_HEIGHT as usize);
    world.add_entity(Ship {
        position: (1.0, 13.0),
        velocity: 0.0,
    });
    world.add_entity(Border {});
    let _ = world.init();
}

struct Ship {
    position: (f64, f64),
    velocity: f64,
}

impl Update for Ship {
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {
        world.debug_draw(format!("{:?}", self.velocity).as_str());
        world.debug_draw(format!("\n{:?}", self.position.0).as_str());
        world.debug_draw(format!("\n\n{:?}", world.ui.last_input).as_str());
        world
            .debug_draw(format!("\n\n\n{:?}", world.ui.current_input).as_str());
        self.position.0 += self.velocity * delta;
        self.position.0 =
            f64::clamp(self.position.0, 1.0, MAP_WIDTH as f64 - 2.0);

        match world.ui.current_input {
            Some(KeyCode::Left) => {
                if world.ui.last_input.is_some_and(|x| x == KeyCode::Left) {
                    self.velocity = 0.0;
                } else {
                    self.velocity = -PLAYER_SPEED;
                }
            }
            Some(KeyCode::Right) => {
                if world.ui.last_input.is_some_and(|x| x == KeyCode::Right) {
                    self.velocity = 0.0;
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
                world.ui.current_input = None;
            }
            _ => {}
        }
        world.draw(
            '^',
            (
                self.position.0.round() as u16,
                self.position.1.round() as u16,
            ),
            crossterm::style::Color::Green,
        );
    }
}

struct Border {}

impl Update for Border {
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {
        for r in 0..MAP_WIDTH {
            for c in 0..MAP_HEIGHT {
                if r == 0 || c == 0 || r == MAP_WIDTH - 1 || c == MAP_HEIGHT - 1
                {
                    world.draw('#', (r, c), crossterm::style::Color::Yellow);
                }
            }
        }
    }
}

struct Bullet {
    position: (f64, f64),
}

impl Update for Bullet {
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {
        self.position =
            (self.position.0, self.position.1 - delta * BULLET_SPEED);
        let target_pos =
            (self.position.0, self.position.1 - delta * BULLET_SPEED);
        if target_pos.1 < 1.0 {
            world.remove_entity(id);
        } else {
            world.draw(
                '*',
                (
                    self.position.0.round() as u16,
                    self.position.1.round() as u16,
                ),
                crossterm::style::Color::Red,
            );
        }
    }
}

struct Blocker {}
