use super::control::{Control};
use super::entity::{Entity};
use super::map::{Map};

use crate::vec2::{Vec2};
use crate::direction::{Direction};
use crate::ids::{EntityId, SpellId, SpellSpecId};

use std::time::{Instant, Duration};

use std::rc::{Rc};
use std::cell::{RefCell};

pub trait SpellSpec {
    fn id(&self) -> SpellSpecId;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn base_damage(&self) -> i32;
    fn create_behaviour(
        &self,
        control: Rc<RefCell<SpellControl>>,
        entity: &Entity
    ) -> Box<dyn SpellBehaviour>;
}

pub trait SpellBehaviour {
    fn on_entity_collision(&mut self, entity: &Entity);
    fn on_wall_collision(&mut self, position: Vec2);
    fn on_update(&mut self, time: Instant, map: &Map, entities: &Vec<Entity>);
}

pub enum SpellAction {
    Move(Direction),
    Cast(Vec<Spell>),
    Destroy,
    //Effect(Vec<Effect>)
}

pub type SpellControl = Control<SpellId, SpellAction>;

pub struct Spell {
    id: SpellId,
    spec_id: SpellSpecId,
    entity_id: EntityId,
    control: Rc<RefCell<SpellControl>>,
    behaviour: Box<dyn SpellBehaviour>,
    damage: i32,
    position: Vec2,
}

impl Spell {
    pub fn new(id: SpellId, spec: &dyn SpellSpec, entity: &Entity) -> Spell {
        let control = Rc::new(RefCell::new(SpellControl::new(id)));
        Spell {
            id,
            spec_id: spec.id(),
            entity_id: entity.id(),
            behaviour: spec.create_behaviour(control.clone(), entity),
            control,
            damage: spec.base_damage(), /* Mul to entity effects */
            position: entity.position() + entity.direction().to_vec2()
        }
    }

    pub fn id(&self) -> SpellId {
        self.id
    }

    pub fn spec_id(&self) -> SpellSpecId {
        self.spec_id
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
    }

    pub fn damage(&self) -> i32 {
        self.damage
    }

    pub fn control(&self) -> &Rc<RefCell<SpellControl>> {
        &self.control
    }

    pub fn behaviour_mut(&mut self) -> &mut dyn SpellBehaviour {
        &mut *self.behaviour
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn displace(&mut self, displacement: Vec2) {
        self.position += displacement;
    }
}

//================================================================
//================================================================

pub struct Fireball;
impl SpellSpec for Fireball {
    fn id(&self) -> SpellSpecId {
        SpellSpecId::next(SpellSpecId::NONE)
    }

    fn name(&self) -> &'static str {
        "Fireball"
    }

    fn description(&self) -> &'static str {
        "Fire attack spell"
    }

    fn base_damage(&self) -> i32 {
        5
    }

    fn create_behaviour(
        &self,
        control: Rc<RefCell<SpellControl>>,
        entity: &Entity
    ) -> Box<dyn SpellBehaviour> {
        Box::new(Behaviour{
            control,
            direction: entity.direction(),
            next_move_time: Instant::now(),
        })
    }
}

struct Behaviour {
    control: Rc<RefCell<SpellControl>>,
    direction: Direction,
    next_move_time: Instant,
}

impl Behaviour {
    const BALL_SPEED: f32 = 15.0;
}

impl SpellBehaviour for Behaviour {
    fn on_entity_collision(&mut self, entity: &Entity) {
        //TODO
    }

    fn on_wall_collision(&mut self, position: Vec2) {
        //TODO
    }

    fn on_update(&mut self, time: Instant, map: &Map, entities: &Vec<Entity>) {
        if time > self.next_move_time {
            self.control.borrow_mut().push_action(SpellAction::Move(self.direction));
            self.next_move_time = time + Duration::from_secs_f32(1.0 / Self::BALL_SPEED);
        }
    }
}
