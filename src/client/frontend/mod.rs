pub mod terminal;

pub use super::actions::{ActionManager};
pub use super::state::{State};
pub use super::util::store::{Store, StateManager};

use message_io::events::{Senderable};

pub trait Input {
    type Event: std::fmt::Debug + Send + 'static;

    fn new<S>(store: Store<ActionManager>, input_sender: S) -> Self
    where S: Senderable<Self::Event> + Send + 'static + Clone;
    fn process_event(&mut self, event: Self::Event);
}

pub trait Renderer {
    fn new() -> Self;
    fn render(&mut self, state: &StateManager<State>);
}

pub trait Viewport {
    type Renderer: Renderer;

    fn new_full_screen() -> Self;
    fn open(&mut self);
    fn close(&mut self);
    fn create_renderer(&mut self) -> Self::Renderer;
}

pub trait Frontend {
    type Input: Input;
    type Viewport: Viewport;
}
