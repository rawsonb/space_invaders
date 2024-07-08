use crossterm::{cursor::position, event::KeyCode};
use engine::{Update, World};

mod engine;

const MAP_HEIGHT: u16 = 15;
const MAP_WIDTH: u16 = 25;
const BULLET_SPEED: f64 = 3.5;

struct Ship {
    position: (u16, u16),
}

impl Update for Ship {
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {
        world.debug_draw(format!("{}", delta).as_str());
        match world.current_input {
            Some(KeyCode::Left) => {
                self.position.0 =
                    u16::clamp(self.position.0 - 1, 1, MAP_WIDTH - 2);
                world.current_input = None;
            }
            Some(KeyCode::Right) => {
                self.position.0 =
                    u16::clamp(self.position.0 + 1, 1, MAP_WIDTH - 2);
                world.current_input = None;
            }
            Some(KeyCode::Up) => {
                world.add_entity(Bullet {
                    position: (
                        self.position.0 as f64,
                        self.position.1 as f64 - 1.0,
                    ),
                });
                world.current_input = None;
            }
            _ => {}
        }
        world.draw('^', self.position, crossterm::style::Color::Green);
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

fn main() {
    let mut world = World::new(MAP_WIDTH as usize, MAP_HEIGHT as usize);
    world.add_entity(Ship { position: (1, 13) });
    world.add_entity(Border {});
    world.init();
}
