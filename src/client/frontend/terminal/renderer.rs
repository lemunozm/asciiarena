use crate::client::frontend::{Renderer};
use crate::client::state::{State};
use crate::client::util::store::{StateManager};

pub struct TerminalRenderer {
}

impl TerminalRenderer {
    pub fn new() -> TerminalRenderer {
        TerminalRenderer {}
    }
}

impl Renderer for TerminalRenderer {
    fn render(&mut self, state: &StateManager<State>) {
        println!("Attempt to render");
    }
}

impl Drop for TerminalRenderer {
    fn drop(&mut self) {
        //TODO
    }
}
