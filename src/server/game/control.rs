use super::entity::{EntityId};

use crate::direction::{Direction};

pub enum EntityAction {
    Walk(Direction),
    Cast(usize /*Skill*/),
}

#[derive(Default)]
pub struct EntityControl {
    entity_id: Option<EntityId>,
    pending_actions: Vec<EntityAction>,
}

impl EntityControl {
    pub fn push_action(&mut self, action: EntityAction) {
        if self.entity_id.is_some() {
            self.pending_actions.push(action);
        }
    }

    pub fn attach_entity(&mut self, id: EntityId) {
        self.entity_id = Some(id);
    }

    pub fn detach_entity(&mut self) {
        self.entity_id = None;
    }

    pub fn entity_id(&self) -> Option<EntityId> {
        self.entity_id
    }

    pub fn actions(&mut self) -> &[EntityAction] {
        self.pending_actions.as_slice()
    }

    pub fn reset_actions(&mut self) {
        self.pending_actions.clear()
    }
}
