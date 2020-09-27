pub mod state;
pub mod actions;

use std::rc::{Rc};
use std::cell::{RefCell, Ref};

pub trait Actionable {
    type State;
    type Action;
    fn dispatch(&mut self, state: &mut Self::State, action: Self::Action);
}

#[derive(Clone)]
pub struct Store<M> where M: Actionable {
    state: Rc<RefCell<M::State>>,
    action_manager: Rc<RefCell<M>>,
    mutation_count: Rc<RefCell<usize>>,
}

impl<M: Actionable> Store<M> {
    pub fn new(state: M::State, action_manager: M) -> Store<M> {
        Store {
            state: Rc::new(RefCell::new(state)),
            action_manager : Rc::new(RefCell::new(action_manager)),
            mutation_count: Rc::new(RefCell::new(0)),
        }
    }

    pub fn dispatch(&mut self, action: M::Action) {
        self.action_manager.borrow_mut().dispatch(&mut self.state.borrow_mut(), action);
        *self.mutation_count.borrow_mut() += 1;
    }

    pub fn state(&self) -> Ref<'_, M::State> {
        self.state.borrow()
    }

    pub fn mutation_count(&self) -> usize {
        *self.mutation_count.borrow()
    }
}
