use std::rc::{Rc};
use std::cell::{RefCell, Ref};

pub trait Actionable {
    type State;
    type Action;
    fn dispatch(&mut self, state_manager: &mut Self::State, action: Self::Action);
}

pub struct Store<A> where A: Actionable {
    state: Rc<RefCell<A::State>>,
    action_manager: Rc<RefCell<A>>,
}

impl<A: Actionable> Store<A> {
    pub fn new(state: A::State, action_manager: A) -> Store<A> {
        let state = Rc::new(RefCell::new(state));
        Store {
            state,
            action_manager : Rc::new(RefCell::new(action_manager)),
        }
    }

    pub fn dispatch(&mut self, action: A::Action) {
        self.action_manager
            .borrow_mut()
            .dispatch(&mut self.state.borrow_mut(), action);
    }

    pub fn state(&self) -> Ref<'_, A::State> {
        self.state.borrow()
    }
}

impl<A: Actionable> Clone for Store<A> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            action_manager: self.action_manager.clone(),
        }
    }
}
