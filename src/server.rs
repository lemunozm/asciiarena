use crate::Character;
use crate::Skill;

use std::collections::HashMap;
use std::rc::Rc;

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

impl Server {
    pub fn new() -> Server {
        Server {
            skill: HashMap::new(), //TODO: initialize
            users: HashMap::new(),
        }
    }

}
