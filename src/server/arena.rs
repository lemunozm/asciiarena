use crate::server::entity::{Entity};
use crate::server::map::{Map};

use rand::seq::SliceRandom;

use std::collections::{HashMap};

pub type EntityId = usize;

pub struct Arena {
    ranking: Vec<char>,
    map: Map,
    players: HashMap<char, EntityId>,
    entities: HashMap<EntityId, Entity>,
    next_entity_id: EntityId,
}

impl Arena {
    pub fn new(map_size: usize, players_it: impl Iterator<Item = char> + Clone) -> Arena {
        let mut arena = Arena {
            ranking: Vec::new(),
            map: Map::new(map_size, players_it.clone().count()),
            players: HashMap::new(),
            entities: HashMap::new(),
            next_entity_id: 0,
        };

        for (index, player) in players_it.enumerate() {
            let entity = Entity::new(player, arena.map.initial_position(index).unwrap());
            let id = arena.add_entity(entity);
            arena.players.insert(player, id);
        }

        arena
    }

    pub fn step(&mut self) {
        self.ranking = self.players.keys().map(|player| *player).collect();
        self.ranking.shuffle(&mut rand::thread_rng());
    }

    pub fn add_entity(&mut self, entity: Entity) -> EntityId {
        self.entities.insert(self.next_entity_id, entity);
        self.next_entity_id += 1;
        self.next_entity_id
    }

    pub fn has_finished(&self) -> bool {
        self.ranking.len() == self.players.len()
    }

    pub fn ranking(&self) -> &Vec<char> {
        &self.ranking
    }
}
