use super::arena::entity::{EntityAction, EntityBehaviour, Entity};
use super::arena::map::{Map};

use crate::character::{Character};
use crate::direction::{Direction};
use crate::ids::{SkillId, EntityId};

use std::rc::{Rc};
use std::time::{Instant};
use std::cell::{RefCell};
use std::collections::{HashMap};

pub struct Player {
    character: Rc<Character>,
    entity_handler: Rc<RefCell<EntityHandler>>,
    points: usize,
}

impl Player {
    pub const MAX_LIFE: usize = 100;
    pub const MAX_ENERGY: usize = 100;
    pub const SPEED_BASE: f32 = 8.0;

    pub fn new(character: Rc<Character>) -> Player {
        Player {
            character,
            entity_handler: Rc::new(RefCell::new(EntityHandler::default())),
            points: 0,
        }
    }

    pub fn character(&self) -> &Rc<Character> {
        &self.character
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_handler.borrow().entity_id
    }

    pub fn points(&self) -> usize {
        self.points
    }

    pub fn is_alive(&self) -> bool {
        self.entity_handler.borrow().entity_id != EntityId::NONE
    }

    pub fn walk(&mut self, direction: Direction) {
        self.entity_handler.borrow_mut().actions.push(EntityAction::Walk(direction))
    }

    pub fn cast(&mut self, direction: Direction, id: SkillId) {
        self.entity_handler.borrow_mut().actions.push(EntityAction::Cast(direction, id))
    }

    pub fn add_points(&mut self, points: usize) {
        self.points += points;
    }

    pub fn create_entity_behaviour(&mut self, entity_id: EntityId) -> Box<PlayerBehaviour> {
        self.entity_handler.borrow_mut().entity_id = entity_id;
        Box::new(PlayerBehaviour {entity_handler: self.entity_handler.clone()})
    }
}

#[derive(Default)]
pub struct EntityHandler {
    entity_id: EntityId,
    actions: Vec<EntityAction>,
}

pub struct PlayerBehaviour {
    entity_handler: Rc<RefCell<EntityHandler>>
}

impl EntityBehaviour for PlayerBehaviour {
    fn destroyed(&mut self) -> Vec<EntityAction> {
        self.entity_handler.borrow_mut().entity_id = EntityId::NONE;
        vec![]
    }

    fn update(
        &mut self,
        _time: Instant,
        _entity: &Entity,
        _map: &Map,
        _entities: &HashMap<EntityId, Entity>
    ) -> Vec<EntityAction> {
        let actions = &mut self.entity_handler.borrow_mut().actions;
        let returned = actions.clone();
        actions.clear();
        returned
    }
}
