use std::collections::{VecDeque};

pub struct Control<I, A> {
    id: I,
    pending_actions: VecDeque<A>,
}

impl<I: Copy, A> Control<I, A> {
    pub fn new(id: I) -> Control<I, A> {
        Control {
            id,
            pending_actions: VecDeque::new(),
        }
    }

    pub fn id(&self) -> I {
        self.id
    }

    pub fn push_action(&mut self, action: A) {
        self.pending_actions.push_back(action);
    }

    pub fn pop_action(&mut self) -> Option<A> {
        self.pending_actions.pop_front()
    }

    pub fn has_actions(&self) -> bool {
        !self.pending_actions.is_empty()
    }
}
