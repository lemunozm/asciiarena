pub mod entity;
pub mod map;
pub mod control;
pub mod spell;

use map::{Map};
use entity::{Entity, EntityControl, EntityAction};
use spell::{Spell, Fireball, SpellSpec, SpellAction};

use crate::character::{Character};
use crate::ids::{SpellId, EntityId};
use crate::vec2::Vec2;

use std::collections::{HashMap};

use std::time::{Instant};
use std::rc::{Rc};
use std::cell::{RefCell};

pub struct Arena {
    map: Map,
    entities: HashMap<EntityId, Entity>,
    spells: HashMap<SpellId, Spell>,
    last_entity_id: EntityId,
    last_spell_id: SpellId,
}

impl Arena {
    pub fn new(map_size: usize, players_number: usize) -> Arena {
        Arena {
            map: Map::new(map_size, players_number),
            entities: HashMap::new(),
            spells: HashMap::new(),
            last_entity_id: EntityId::NONE,
            last_spell_id: SpellId::NONE,
        }
    }

    pub fn map(&self) -> &Map {
        &self.map
    }

    pub fn entities(&self) -> &HashMap<EntityId, Entity> {
        &self.entities
    }

    pub fn spells(&self) -> &HashMap<SpellId, Spell> {
        &self.spells
    }

    pub fn create_entity(
        &mut self,
        character: Rc<Character>,
        position: Vec2
    ) -> &Rc<RefCell<EntityControl>> {
        let id = EntityId::next(self.last_entity_id);
        let entity = Entity::new(id, character, position);
        self.last_entity_id = id;
        self.entities.insert(id, entity);
        self.entities[&id].control()
    }

    pub fn create_spell(&mut self, spell_spec: &dyn SpellSpec, entity_id: EntityId) {
        let id = SpellId::next(self.last_spell_id);
        let entity = &self.entities[&entity_id];
        let spell = Spell::new(id, spell_spec, &entity);
        self.last_spell_id = id;
        self.spells.insert(id, spell);
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
                        self.create_spell(&Fireball, entity_id);
                    }
                }
            }
        }

        let spell_controls = self.spells
            .iter()
            .filter_map(|(_, spell)| {
                match spell.control().borrow().has_actions() {
                    true => Some(spell.control().clone()),
                    false => None,
                }
            })
            .collect::<Vec<_>>();

        for control in spell_controls {
            let spell_id = control.borrow().id();
            let spell = self.spells.get_mut(&spell_id).unwrap();
            spell.behaviour_mut().on_update(current_time, &self.map, &self.entities);
            while let Some(action) = control.borrow_mut().pop_action() {
                match action {
                    SpellAction::Move(direction) => {
                        let next_position = spell.position() + direction.to_vec2();
                        if self.map.contains(next_position) {
                            let entity_position = self.entities
                                .values_mut()
                                .find(|entity| entity.position() == next_position);

                            if let Some(entity) = entity_position {
                                spell.behaviour_mut().on_entity_collision(entity);
                                entity.add_health(-spell.damage())
                            }
                            spell.set_position(next_position)
                        }
                        else {
                            spell.behaviour_mut().on_wall_collision(next_position);
                        }
                    }
                    SpellAction::Cast(_skill) => {
                        //TODO
                    }
                    SpellAction::Destroy => spell.destroy()
                }
            }
        }

        self.entities.retain(|_, entity| entity.is_alive());
    }
}
