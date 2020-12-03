use super::map::{Map};
use super::entity::{Entity, EntityControl, EntityAction};

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
    last_entity_id: EntityId,
}

impl Arena {
    pub fn new(map_size: usize, players_number: usize) -> Arena {
        Arena {
            map: Map::new(map_size, players_number),
            entities: HashMap::new(),
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
    ) -> &Rc<RefCell<EntityControl>> {
        let id = self.last_entity_id.next();
        let entity = Entity::new(id, character, position);
        self.last_entity_id = id;
        self.entities.insert(id, entity);
        self.entities[&id].control()
    }

    pub fn update(&mut self) {
        assert!(self.entities.iter().all(|(_, entity)| entity.is_alive()));

        let current_time = Instant::now();
        let entity_controls = self.entities
            .iter()
            .filter_map(|(_, entity)| {
                match entity.control().borrow().has_actions() {
                    true => Some(entity.control().clone()),
                    false => None,
                }
            })
            .collect::<Vec<_>>();

        //Should it be processed randomly?
        for control in entity_controls {
            let entity_id = control.borrow().id();
            while let Some(action) = control.borrow_mut().pop_action() {
                match action {
                    EntityAction::Walk(direction) => {
                        let entity = &self.entities[&entity_id];
                        let next_position = entity.position() + direction.to_vec2();
                        if self.map.contains(next_position) {
                            let occupied_position = self.entities
                                .values()
                                .find(|entity| entity.position() == next_position)
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

        self.entities.retain(|_, entity| entity.is_alive());
    }
}
