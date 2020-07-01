pub mod vec2;
mod util;
mod collision;
pub mod message;
pub mod network;
mod server;
pub mod events;
pub mod network_manager;
pub mod server_manager;
pub mod client_manager;

use collision::Rectangle;
use vec2::Vec2;

use std::rc::Rc;

pub struct Skill {
    name: String,
    id: usize,
    cost: i32,
    cooldown: i32,
}

pub struct Spell {
    skill: Rc<Skill>,
    rectangle: Rectangle
}

pub struct Character {
    name: String,
    skills: Vec<Rc<Skill>>,
}


pub struct Entity {
    character: Rc<Character>,
    rectangle: Rectangle,
    live: i32,
    max_live: i32,
    energy: i32,
    max_energy: i32,
}

pub struct Player {
    id: usize,
    entity: Option<Rc<Entity>>,
    points: usize,
}

pub struct Wall {
    rectangles: Vec<Rectangle>,
}

pub struct Map {
    dimension: Vec2,
    obstacles: Vec<Wall>,
}

pub struct Scene {
    entities: Vec<Rc<Entity>>,
    spells: Vec<Spell>,
    map: Map,
}

pub struct Arena {
    scene: Scene,
}

pub struct Game {
    players: Vec<Player>,
    arena: Arena,
    win_points: usize,
}


