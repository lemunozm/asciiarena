pub mod entity;
pub mod map;
pub mod control;
pub mod spell;

use map::{Map};
use entity::{Entity, EntityControl, EntityAction};
use spell::{Spell, SpellAction};

use crate::character::{Character};
use crate::ids::{SpellId, EntityId, SpellSpecId};
use crate::vec2::Vec2;

use std::collections::{HashMap, VecDeque};

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

    pub fn create_spell(&mut self, spec_id: SpellSpecId, entity_id: EntityId) {
        let id = SpellId::next(self.last_spell_id);
        let entity = &self.entities[&entity_id];
        let spell = Spell::new(id, spec_id, &entity);
        self.last_spell_id = id;
        self.spells.insert(id, spell);
    }

    pub fn update(&mut self) {
        assert!(self.entities.iter().all(|(_, entity)| entity.is_alive()));

        self.spells.retain(|_, spell| !spell.is_destroyed());

        let current_time = Instant::now();

        //for each entity: on_update()

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
                                entity.set_direction(direction);
                                entity.walk(current_time);
                            }
                            else {
                                //on_entity_collision()
                            }
                        }
                        else {
                            //on_wall_collision()
                        }
                    }
                    EntityAction::Cast(_skill) => {
                        self.create_spell(SpellSpecId(1), entity_id);
                    }
                }
            }
        }

        for (_, spell) in &mut self.spells {
            let actions = spell
                .behaviour()
                .on_update(current_time, &spell, &self.map, &self.entities);

            let mut spell_actions = VecDeque::from(actions);
            while let Some(action) = spell_actions.pop_front() {
                match action {
                    SpellAction::Move => {
                        spell.move_step(current_time);
                        if self.map.contains(spell.position()) {
                            let entity_position = self.entities
                                .values_mut()
                                .find(|entity| entity.position() == spell.position());

                            if let Some(entity) = entity_position {
                                let (actions, affect) = spell
                                    .behaviour()
                                    .on_entity_collision(&entity);

                                if affect {
                                    entity.add_health(-spell.damage());
                                }

                                spell_actions.extend(actions);
                            }
                        }
                        else {
                            spell.destroy();
                            let actions = spell.behaviour().on_destroy_by_wall_collision(&spell);
                            spell_actions.extend(actions);
                        }
                    }
                    SpellAction::SetSpeed(speed) => spell.set_speed(speed),
                    SpellAction::SetDirection(direction) => spell.set_direction(direction),
                    SpellAction::Cast(_spells) => todo!(),
                    SpellAction::Create(_entities) => todo!(),
                    SpellAction::Destroy => spell.destroy(),
                }
            }
        }

        self.entities.retain(|_, entity| entity.is_alive());
    }
}
