use super::entity::{Entity};
use super::map::{Map};

use crate::vec2::{Vec2};
use crate::direction::{Direction};
use crate::ids::{EntityId, SpellId, SpellSpecId};
use crate::specification::spells::{SPELL_SPECIFICATIONS};

use std::time::{Instant, Duration};
use std::collections::{HashMap, HashSet};
use std::cell::{RefCell, RefMut};

pub trait SpellBehaviour: Send + Sync {
    fn entity_collision(&mut self, entity: &Entity) -> (Vec<SpellAction>, bool);
    fn destroyed(&mut self, spell: &Spell) -> Vec<SpellAction>;
    fn update(
        &mut self,
        time: Instant,
        spell: &Spell,
        map: &Map,
        entities: &HashMap<EntityId, Entity>,
    ) -> Vec<SpellAction>;
}

pub enum SpellAction {
    SetSpeed(f32),
    SetDirection(Direction),
    Move,
    Cast(Vec<Spell>),
    Create(Vec<Entity>),
    Destroy,
}

pub struct Spell {
    id: SpellId,
    spec_id: SpellSpecId,
    entity_origin_id: EntityId,
    behaviour: RefCell<Box<dyn SpellBehaviour>>,
    damage: i32,
    //effects
    position: Vec2,
    direction: Direction,
    speed: f32,
    next_move_time: Instant,
    affected_entities: HashSet<EntityId>,
    destroyed: bool,
}

impl Spell {
    pub fn new(id: SpellId, spec_id: SpellSpecId, entity: &Entity) -> Spell {
        let spec = SPELL_SPECIFICATIONS.get(&spec_id).unwrap();
        Spell {
            id,
            spec_id,
            entity_origin_id: entity.id(),
            behaviour: RefCell::new(get_behaviour(spec.behaviour_name)),
            damage: spec.damage, /* Mul to entity effects */
            position: entity.position() + entity.direction().to_vec2(),
            direction: entity.direction(),
            speed: spec.speed,
            next_move_time: Instant::now() + Duration::from_secs_f32(1.0 / spec.speed),
            affected_entities: HashSet::new(),
            destroyed: false,
        }
    }

    pub fn id(&self) -> SpellId {
        self.id
    }

    pub fn spec_id(&self) -> SpellSpecId {
        self.spec_id
    }

    pub fn entity_origin_id(&self) -> EntityId {
        self.entity_origin_id
    }

    pub fn damage(&self) -> i32 {
        self.damage
    }

    pub fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub fn behaviour(&self) -> RefMut<'_, Box<dyn SpellBehaviour>> {
        self.behaviour.borrow_mut()
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn displace(&mut self, displacement: Vec2) {
        self.position += displacement;
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed
    }

    pub fn move_step(&mut self, current: Instant) -> bool {
        if current > self.next_move_time {
            self.position += self.direction.to_vec2();
            self.next_move_time = current + Duration::from_secs_f32(1.0 / self.speed);
            return true
        }
        false
    }

    pub fn add_affected_entity(&mut self, entity_id: EntityId) {
        self.affected_entities.insert(entity_id);
    }

    pub fn is_affected_entity(&self, entity_id: EntityId) -> bool {
        self.affected_entities.contains(&entity_id)
    }

    pub fn destroy(&mut self) {
        self.destroyed = true;
    }
}

//TODO: use std::any::type_name for build the string
fn get_behaviour(name: &'static str) -> Box<dyn SpellBehaviour> {
    match name {
        "Explotable ball" => Box::new(behaviour::ExplotableBall),
        "" => Box::new(behaviour::None),
        _ => panic!("Spell behaviour '{}' not found", name),
    }
}

mod behaviour {
    use super::super::entity::{Entity};
    use super::super::map::{Map};

    use super::{SpellBehaviour, SpellAction, Spell};

    use crate::ids::{EntityId};

    use std::time::{Instant};
    use std::collections::{HashMap};

    pub struct None;
    impl SpellBehaviour for None {
        fn entity_collision(&mut self, _entity: &Entity) -> (Vec<SpellAction>, bool) {
            (vec![], false)
        }

        fn destroyed(&mut self, _spell: &Spell) -> Vec<SpellAction> {
            vec![]
        }

        fn update(
            &mut self,
            _time: Instant,
            _spell: &Spell,
            _map: &Map,
            _entities: &HashMap<EntityId, Entity>,
        ) -> Vec<SpellAction> {
            vec![]
        }
    }

    pub struct ExplotableBall;
    impl SpellBehaviour for ExplotableBall {
        fn entity_collision(&mut self, _entity: &Entity) -> (Vec<SpellAction>, bool) {
            (vec![SpellAction::Destroy], true)
        }

        fn destroyed(&mut self, _spell: &Spell) -> Vec<SpellAction> {
            vec![]
        }

        fn update(
            &mut self,
            _time: Instant,
            _spell: &Spell,
            _map: &Map,
            _entities: &HashMap<EntityId, Entity>,
        ) -> Vec<SpellAction> {
            vec![SpellAction::Move]
        }
    }
}
