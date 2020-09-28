pub mod terminal;

pub use super::actions::{ActionManager};
pub use super::state::{State};
pub use super::util::store::{Store, StateManager};

use message_io::events::{Senderable};

pub trait Input {
    type InputEvent;

    fn new<S>(store: Store<ActionManager>, input_sender: S) -> Self
    where S: Senderable<Self::InputEvent> + Send + 'static + Clone;
    fn process_event(&mut self, event: Self::InputEvent);
}

pub trait Renderer {
    fn new() -> Self;
    fn render(&mut self, state: &StateManager<State>);
}

pub trait Viewport {
    type Renderer;

    fn new_full_screen() -> Self;
    fn open(&mut self);
    fn close(&mut self);
    fn create_renderer(&mut self) -> Self::Renderer;
}

pub trait Frontend {
    type Input;
    type Viewport;
}
