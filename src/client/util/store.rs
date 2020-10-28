pub trait Actionable {
    type State;
    type Action;
    fn dispatch(&mut self, state_manager: &mut Self::State, action: Self::Action);
}

pub struct Store<A> where A: Actionable {
    state: A::State,
    action_manager: A,
}

impl<A: Actionable> Store<A> {
    pub fn new(state: A::State, action_manager: A) -> Store<A> {
        Store {
            state,
            action_manager : action_manager,
        }
    }

    pub fn dispatch(&mut self, action: A::Action) {
        self.action_manager.dispatch(&mut self.state, action);
    }

    pub fn state(&self) -> &A::State {
        &self.state
    }
}
