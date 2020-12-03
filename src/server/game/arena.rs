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
    last_entity_id: EntityId,
}

impl Arena {
    pub fn new(map_size: usize, players_number: usize) -> Arena {
        Arena {
            map: Map::new(map_size, players_number),
            entities: HashMap::new(),
            entity_controls: Vec::new(),
            last_entity_id: EntityId::NO_ENTITY,
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
        self.last_entity_id = self.last_entity_id.next();
        let entity = Entity::new(self.last_entity_id, character, position);
        self.entities.insert(self.last_entity_id, entity);
        self.entities.get_mut(&self.last_entity_id).unwrap()
    }

    pub fn attach_entity_control(&mut self, control: Rc<RefCell<EntityControl>>) {
        assert!(control.borrow().entity_id().is_valid());
        assert!(self.entities.get(&control.borrow().entity_id()).is_some());
        self.entity_controls.push(control);
    }

    pub fn update(&mut self) {
        let current_time = Instant::now();

        assert!(self.entities.iter().all(|(_, entity)| entity.is_alive()));
        assert!(self.entity_controls.iter().all(|control| control.borrow().entity_id().is_valid()));

        //Should it be processed randomly?
        for control in &mut self.entity_controls {
            let mut control = control.borrow_mut();
            let entity_id = control.entity_id();
            while let Some(action) = control.pop_action() {
                match action {
                    EntityAction::Walk(direction) => {
                        let entity = &self.entities[&entity_id];
                        let next_position = entity.position() + direction.to_vec2();
                        if self.map.contains(next_position) {
                            let occupied_position = self.entities
                                .values()
                                .find(|player| player.position() == next_position)
                                .is_some();

                            if !occupied_position {
                                let entity = self.entities.get_mut(&entity_id).unwrap();
                                entity.walk(direction, current_time);
                            }
                        }
                    }
                    EntityAction::Cast(_skill) => {
                        /*
                        let spell_spec = SpellSpec {
                            id: 1,
                            damage: 5,
                            behaviour_builder: FireballBehaviourBuilder,
                        };
                        let spell = Spell::new(spell_spec, );
                        //TODO
                        */
                    }
                }
            }
        }

        for control in &mut self.entity_controls {
            let mut control = control.borrow_mut();
            let entity = &self.entities[&control.entity_id()];
            if !entity.is_alive() {
                control.detach_entity();
            }
        }

        self.entities.retain(|_, entity| entity.is_alive());
        self.entity_controls.retain(|control| control.borrow().entity_id().is_valid());
    }
}
