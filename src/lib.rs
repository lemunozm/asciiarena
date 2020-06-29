mod vec2;
mod util;
mod collision;
mod message;

use collision::Rectangle;
use vec2::Vec2;

use std::collections::HashMap;
use std::rc::Rc;

struct Skill {
    name: String,
    id: usize,
    cost: i32,
    cooldown: i32,
}

struct Spell {
    skill: Rc<Skill>,
    rectangle: Rectangle
}

struct Character {
    name: String,
    skills: Vec<Rc<Skill>>,
}


struct Entity {
    character: Rc<Character>,
    rectangle: Rectangle,
    live: i32,
    max_live: i32,
    energy: i32,
    max_energy: i32,
}

struct Player {
    id: usize,
    entity: Option<Rc<Entity>>,
    points: usize,
}

struct Wall {
    rectangles: Vec<Rectangle>,
}

struct Map {
    dimension: Vec2,
    obstacles: Vec<Wall>,
}

struct Scene {
    entities: Vec<Rc<Entity>>,
    spells: Vec<Spell>,
    map: Map,
}

struct Arena {
    scene: Scene,
}

struct Game {
    players: Vec<Player>,
    arena: Arena,
    win_points: usize,
}


struct Connection {
}

struct PlayerSession {
    id: usize,
    character: Rc<Character>,
    connection: Connection,
}

struct Server {
    skill: HashMap<String, Rc<Skill>>,
    users: HashMap<usize, PlayerSession>,
}
