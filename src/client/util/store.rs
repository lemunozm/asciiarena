use std::rc::{Rc};
use std::cell::{RefCell, Ref};

pub trait Actionable {
    type State;
    type Action;
    fn dispatch(&mut self, state_manager: &mut StateManager<Self::State>, action: Self::Action);
}

pub struct Store<A> where A: Actionable {
    state: Rc<RefCell<A::State>>,
    action_manager: Rc<RefCell<A>>,
    mutator: Rc<RefCell<StateManager<A::State>>>,
}

impl<A: Actionable> Store<A> {
    pub fn new(state: A::State, action_manager: A) -> Store<A> {
        let state = Rc::new(RefCell::new(state));
        Store {
            state: state.clone(),
            action_manager : Rc::new(RefCell::new(action_manager)),
            mutator: Rc::new(RefCell::new(StateManager::new(state))),
        }
    }

    pub fn dispatch(&mut self, action: A::Action) {
        self.action_manager
            .borrow_mut()
            .dispatch(&mut self.mutator.borrow_mut(), action);
    }

    pub fn state(&self) -> Ref<'_, A::State> {
        self.state.borrow()
    }

    pub fn mutation_count(&self) -> usize {
        self.mutator.borrow().mutation_count()
    }
}

impl<A: Actionable> Clone for Store<A> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            action_manager: self.action_manager.clone(),
            mutator: self.mutator.clone(),
        }
    }
}

pub struct StateManager<S> {
    state: Rc<RefCell<S>>,
    mutation_count: usize,
}

impl<S> StateManager<S> {
    fn new(state: Rc<RefCell<S>>) -> StateManager<S> {
        StateManager {
            state,
            mutation_count: 0,
        }
    }

    pub fn mutate(&mut self, mutation: impl FnOnce(&mut S)) {
        mutation(&mut self.state.borrow_mut());
        self.mutation_count += 1;
    }

    pub fn get(&self) -> Ref<'_, S> {
        self.state.borrow()
    }

    pub fn mutation_count(&self) -> usize {
        self.mutation_count
    }
}
