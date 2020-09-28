use crate::client::frontend::{Renderer as RendererBase};
use crate::client::state::{State};
use crate::client::util::store::{StateManager};

pub struct Renderer {
}

impl RendererBase for Renderer {
    fn new() -> Renderer {
        Renderer {}
    }

    fn render(&mut self, state: &StateManager<State>) {
    }
}
