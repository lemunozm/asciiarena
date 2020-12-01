use super::map::{Map};
use super::entity::{Entity};
use super::control::{EntityControl, EntityAction};

use crate::character::{Character};
use crate::message::{EntityId};
use crate::vec2::Vec2;

use std::collections::{HashMap};

use std::time::{Instant};
use std::rc::{Rc};
use std::cell::{RefCell};

pub struct Arena {
    map: Map,
    entities: HashMap<EntityId, Entity>,
    entity_controls: Vec<Rc<RefCell<EntityControl>>>,
    next_entity_id: EntityId,
}

impl Arena {
    pub fn new(map_size: usize, players_number: usize) -> Arena {
        Arena {
            map: Map::new(map_size, players_number),
            entities: HashMap::new(),
            entity_controls: Vec::new(),
            next_entity_id: 0,
        }
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn entities(&self) -> &HashMap<EntityId, Entity> {
        &self.entities
    }

    pub fn create_entity(
        &mut self,
        character: Rc<Character>,
        position: Vec2
        ) -> &mut Entity
    {
        let id = self.next_entity_id;
        let entity = Entity::new(id, character, position);
        self.next_entity_id += 1;
        self.entities.insert(id, entity);
        self.entities.get_mut(&id).unwrap()
    }

    pub fn attach_entity_control(&mut self, control: Rc<RefCell<EntityControl>>) {
        assert!(self.entities.get_mut(&control.borrow().entity_id().unwrap()).is_some());
        self.entity_controls.push(control);
    }

    pub fn update(&mut self) {
        let current_time = Instant::now();

        //Should it be processed randomly?
        for control in &mut self.entity_controls {
            let mut control = control.borrow_mut();
            let entity_id = control.entity_id().expect("Exists");
            for action in control.actions() {
                match action {
                    EntityAction::Walk(direction) => {
                        let entity = self.entities.get(&entity_id).expect("Exists");
                        let next_position = entity.position() + direction.to_vec2();
                        if self.map.contains(next_position) {
                            let occupied_position = self.entities
                                .values()
                                .find(|player| player.position() == next_position)
                                .is_some();

                            if !occupied_position {
                                let entity = self.entities.get_mut(&entity_id).expect("Exists");
                                entity.walk(*direction, current_time);
                            }
                        }
                    }
                    EntityAction::Cast(_skill) => {
                        //TODO
                    }
                }
            }
            control.reset_actions();
        }

        for control in &mut self.entity_controls {
            let mut control = control.borrow_mut();
            let entity_id = control.entity_id().expect("Exists");
            let entity = self.entities.get_mut(&entity_id).expect("Exists");
            if !entity.is_alive() {
                control.detach_entity();
            }
        }

        self.entities.retain(|_, entity| entity.is_alive());
    }
}
