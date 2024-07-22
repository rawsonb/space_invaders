use std::{os::linux::raw::stat, vec};
mod engine;
use crate::engine::{Entity, World};
use core::mem::discriminant as tag;
use crossterm::{cursor::position, event::KeyCode};
const MAP_HEIGHT: u16 = 15;
const MAP_WIDTH: u16 = 25; // in characters
const BULLET_SPEED: f64 = 5.0;
const PLAYER_SPEED: f64 = 4.5; // characters per second
const PLAYER_RELOAD_TIME: f64 = 0.3;
const PLIBBLE_SPEED: f64 = 2.0;
const PLIBBLER_RELOAD_TIME: f64 = 3.0;
const PLIBBLER_SPEED: f64 = 1.5;
const SHOOTLER_SPEED: f64 = 1.0;
const SHOOTLER_RELOAD_TIME: f64 = 2.0;

fn main() {
    let mut world = World::new(MAP_WIDTH as usize, MAP_HEIGHT as usize);
    world.add_entity(Ship {
        position: (12, 13),
        tilt: (0.0, 0.0),
        target: (0, 0),
        reload: PLAYER_RELOAD_TIME,
    });
    world.add_entity(Plibbler {
        motion: EnemyMotion {
            position: (3, 1),
            tilt: (0.0, 0.0),
            target: (1, 0),
            bounds: (1, 11),
        },
        reload: PLIBBLER_RELOAD_TIME,
    });
    world.add_entity(Plibbler {
        motion: EnemyMotion {
            position: (21, 1),
            tilt: (0.0, 0.0),
            target: (-1, 0),
            bounds: (13, 23),
        },
        reload: PLIBBLER_RELOAD_TIME,
    });
    world.add_entity(Plibble {
        motion: EnemyMotion {
            position: (1, 2),
            tilt: (0.0, 0.0),
            target: (1, 0),
            bounds: (1, 11),
        },
    });
    world.add_entity(Plibble {
        motion: EnemyMotion {
            position: (23, 2),
            tilt: (0.0, 0.0),
            target: (-1, 0),
            bounds: (13, 23),
        },
    });
    world.add_entity(Shootler {
        motion: EnemyMotion {
            position: (23, 2),
            tilt: (0.0, 0.0),
            target: (-1, 0),
            bounds: (13, 23),
        },
        reload: SHOOTLER_RELOAD_TIME,
    });

    build_walls(&mut world);

    world.add_entity(Barrier { position: (4, 12) });
    world.add_entity(Barrier { position: (5, 12) });
    world.add_entity(Barrier { position: (6, 12) });
    world.add_entity(Barrier { position: (11, 12) });
    world.add_entity(Barrier { position: (12, 12) });
    world.add_entity(Barrier { position: (13, 12) });
    world.add_entity(Barrier { position: (18, 12) });
    world.add_entity(Barrier { position: (19, 12) });
    world.add_entity(Barrier { position: (20, 12) });
    world.add_entity(Barrier { position: (5, 11) });
    world.add_entity(Barrier { position: (12, 11) });
    world.add_entity(Barrier { position: (19, 11) });
    let _ = world.init();
}

fn build_walls(world: &mut World) {
    for r in 0..MAP_WIDTH {
        for c in 0..MAP_HEIGHT {
            if r == 0 || c == 0 || r == MAP_WIDTH - 1 || c == MAP_HEIGHT - 1 {
                world.add_entity(Wall { position: (r, c) });
            }
        }
    }
}

struct Health {
    hp: f64,
}

enum Alignment {
    Player = 0,
    Enemy,
}

struct Align {
    alignment: Alignment,
}

struct Ship {
    position: (u16, u16),
    tilt: (f64, f64),
    target: (i8, i8),
    reload: f64,
}

impl Entity for Ship {
    fn start(&mut self, world: &mut World, id: i64) {
        world.set_component(id, Health { hp: 10.0 });
        world.set_component(
            id,
            Align {
                alignment: Alignment::Player,
            },
        );
    }
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {
        let _ = world
            .ui
            .debug_draw(15, format!("Tilt: {:?}", self.tilt).as_str());
        let _ = world.ui.debug_draw(
            16,
            format!("X_Position: {:?}", self.position.0).as_str(),
        );
        let _ = world.ui.debug_draw(
            17,
            format!("Last Input: {:?}", world.ui.last_input).as_str(),
        );
        let _ = world
            .ui
            .debug_draw(18, format!("Target: {:?}", self.target).as_str());
        let _ = world
            .ui
            .debug_draw(19, format!("Delta: {:?}", delta).as_str());

        match world.ui.current_input {
            Some(KeyCode::Left) => {
                if self.target.0 == 1 {
                    self.zero_movement()
                } else {
                    self.target = (-1, 0);
                }
            }
            Some(KeyCode::Right) => {
                if self.target.0 == -1 {
                    self.zero_movement();
                } else {
                    self.target = (1, 0);
                }
            }

            Some(KeyCode::Up) => {
                self.shoot(world);
            }
            _ => {}
        }

        match self.target.0 {
            1 => self.tilt.0 += PLAYER_SPEED * delta,
            -1 => self.tilt.0 -= PLAYER_SPEED * delta,
            _ => {}
        }

        if self.reload > 0.0 {
            self.reload -= delta;
        }

        if self.tilt.0 > 1.0 {
            self.position.0 += 1;
            self.tilt.0 -= 1.0;
        } else if self.tilt.0 < -1.0 {
            self.position.0 -= 1;
            self.tilt.0 += 1.0;
        }

        self.position.0 = self.position.0.clamp(1, MAP_WIDTH - 2);
        self.position.1 = self.position.1.clamp(1, MAP_HEIGHT - 2);

        let visual = match self.target.0 {
            -1 => '<',
            1 => '>',
            _ => '^',
        };
        world.map.write(
            self.position,
            visual,
            crossterm::style::Color::Green,
            id,
        );
    }
}

impl Ship {
    fn zero_movement(&mut self) {
        self.tilt = (0.0, 0.0);
        self.target = (0, 0);
    }
    fn shoot(&mut self, world: &mut World) {
        if self.reload <= 0.0 {
            world.add_entity(Bullet {
                position: (self.position.0, self.position.1 - 1),
                tilt: (0.0, 0.0),
                from_player: true,
                color: crossterm::style::Color::DarkGreen,
            });
            self.reload = PLAYER_RELOAD_TIME;
            self.zero_movement();
        }
    }
}

struct Bullet {
    position: (u16, u16),
    tilt: (f64, f64),
    from_player: bool,
    color: crossterm::style::Color,
}

impl Entity for Bullet {
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {
        self.tilt.1 += if self.from_player {
            -delta * BULLET_SPEED
        } else {
            delta * BULLET_SPEED
        };
        if self.tilt.1 <= -1.0 {
            self.position.1 -= 1;
            self.tilt.1 += 1.0;
        } else if self.tilt.1 >= 1.0 {
            self.position.1 += 1;
            self.tilt.1 -= 1.0;
        }
        if self.position.1 <= 0 || self.position.1 >= MAP_HEIGHT - 1 {
            world.remove_entity(id);
        } else {
            let mut other_id = id;
            match world.query_map(self.position).first() {
                Some(x) => {
                    other_id = x.id;
                }
                None => {}
            }
            if other_id == id {
                world.map.write(self.position, '*', self.color, id);
            } else {
                let struck_alignment: Option<&mut Align> =
                    world.get_component(other_id);
                match struck_alignment {
                    Some(x) => {
                        if (tag(&x.alignment) == tag(&Alignment::Enemy)
                            && self.from_player)
                            || (tag(&x.alignment) == tag(&Alignment::Player)
                                && !self.from_player)
                        {
                            world.remove_entity(id);
                            world.remove_entity(other_id);
                        }
                    }
                    None => {
                        world.remove_entity(id);
                        world.remove_entity(other_id);
                    }
                }
            }
        }
    }
}

struct Barrier {
    position: (u16, u16),
}

impl Entity for Barrier {
    fn update(&mut self, _delta: f64, world: &mut World, id: i64) {
        world.map.write(
            self.position,
            '#',
            crossterm::style::Color::Yellow,
            id,
        );
    }
}

struct Wall {
    position: (u16, u16),
}

impl Entity for Wall {
    fn update(&mut self, _delta: f64, world: &mut World, id: i64) {
        world
            .map
            .write(self.position, '#', crossterm::style::Color::White, id);
    }
}

struct EnemyMotion {
    position: (u16, u16),
    tilt: (f64, f64),
    target: (i8, i8),
    bounds: (u16, u16),
}

impl EnemyMotion {
    fn update(&mut self, delta: f64, world: &mut World, id: i64, speed: f64) {
        self.tilt = (
            self.tilt.0 + self.target.0 as f64 * speed * delta,
            self.tilt.1 + self.target.1 as f64 * speed * delta,
        );

        if self.tilt.0 >= 1.0 {
            self.tilt.0 -= 1.0;
            if self.position.0 >= self.bounds.1 {
                self.target.0 = -1;
                self.position.1 += 1;
            } else {
                self.position.0 += 1;
            }
        } else if self.tilt.0 <= -1.0 {
            self.tilt.0 += 1.0;
            if self.position.0 <= self.bounds.0 {
                self.target.0 = 1;
                self.position.1 += 1;
            } else {
                self.position.0 -= 1;
            }
        }

        world
            .map
            .write(self.position, '@', crossterm::style::Color::Red, id);
    }
}

struct Plibble {
    motion: EnemyMotion,
}

impl Entity for Plibble {
    fn start(&mut self, world: &mut World, id: i64) {
        world.set_component(
            id,
            Align {
                alignment: Alignment::Enemy,
            },
        );
    }
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {
        self.motion.update(delta, world, id, PLIBBLE_SPEED);

        world.map.write(
            self.motion.position,
            '@',
            crossterm::style::Color::Red,
            id,
        );
    }
}

struct Plibbler {
    motion: EnemyMotion,
    reload: f64,
}

impl Entity for Plibbler {
    fn start(&mut self, world: &mut World, id: i64) {
        world.set_component(
            id,
            Align {
                alignment: Alignment::Enemy,
            },
        );
    }
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {
        self.motion.update(delta, world, id, PLIBBLER_SPEED);

        if self.reload >= 0.0 {
            self.reload -= delta;
        } else {
            self.reload = PLIBBLER_RELOAD_TIME;
            world.add_entity(Plibble {
                motion: EnemyMotion {
                    position: self.motion.position,
                    tilt: self.motion.tilt,
                    target: self.motion.target,
                    bounds: self.motion.bounds,
                },
            });
            self.motion.tilt.0 -= self.motion.target.0 as f64;
        }

        world.map.write(
            self.motion.position,
            '&',
            crossterm::style::Color::Red,
            id,
        );
    }
}

struct Shootler {
    motion: EnemyMotion,
    reload: f64,
}

impl Entity for Shootler {
    fn start(&mut self, world: &mut World, id: i64) {
        world.set_component(
            id,
            Align {
                alignment: Alignment::Enemy,
            },
        );
    }
    fn update(&mut self, delta: f64, world: &mut World, id: i64) {
        self.motion.update(delta, world, id, SHOOTLER_SPEED);

        if self.reload >= 0.0 {
            self.reload -= delta;
        } else {
            self.reload = SHOOTLER_RELOAD_TIME;
            world.add_entity(Bullet {
                position: self.motion.position,
                tilt: self.motion.tilt,
                from_player: false,
                color: crossterm::style::Color::DarkRed,
            });
            self.motion.tilt.0 -= self.motion.target.0 as f64;
        }
        let mut visual = 'S';
        if self.reload > SHOOTLER_RELOAD_TIME * 0.9 {
            visual = '$';
        }

        world.map.write(
            self.motion.position,
            visual,
            crossterm::style::Color::Red,
            id,
        );
    }
}
