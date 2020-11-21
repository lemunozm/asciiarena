use crate::server::map::{Map};
use crate::entity::{Entity, EntityId};
use crate::character::{Character};
use crate::vec2::Vec2;

use std::collections::{HashMap};

use std::rc::{Rc};

pub struct Arena {
    map: Map,
    entities: HashMap<EntityId, Entity>,
    next_entity_id: EntityId,
}

impl Arena {
    pub fn new(map_size: usize, players_number: usize) -> Arena {
        Arena {
            map: Map::new(map_size, players_number),
            entities: HashMap::new(),
            next_entity_id: 0,
        }
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn entity_mut(&mut self, entity_id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&entity_id)
    }

    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    pub fn create_entity(&mut self, character: Rc<Character>, position: Vec2) -> &mut Entity {
        let id = self.next_entity_id;
        let entity = Entity::new(id, character, position);
        self.next_entity_id += 1;
        self.entities.insert(id, entity);
        self.entities.get_mut(&id).unwrap()
    }

    pub fn update(&mut self) -> Vec<Entity> {
        Vec::new() // removed entities
    }
}
