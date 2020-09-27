use crate::version::{Compatibility};
use super::{Actionable, state::State};

pub enum Action {
    SetVersionInfo(String, Compatibility), //server_version, compatibility
}

pub struct ActionManager {
    event_sender: usize,
    server_api: usize,
}

impl ActionManager {
    pub fn new(event_sender: usize, server_api: usize) -> ActionManager {
        ActionManager {
            event_sender: event_sender,
            server_api: server_api,
        }
    }
}

impl Actionable for ActionManager {
    type State = State;
    type Action = Action;

    fn dispatch(&mut self, state: &mut State, action: Action) {
        match action {
            Action::SetVersionInfo(server_version, compatibility) => {
                state.server_mut().set_version_info(server_version, compatibility)
            }
        }
    }
}
