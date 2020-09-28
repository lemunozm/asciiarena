pub mod terminal;

pub use super::actions::{Dispatcher};
pub use super::state::{State};
pub use super::util::store::{StateManager};

pub trait Input {
    fn new(actions: impl Dispatcher + 'static) -> Self;
}

pub trait Renderer {
    fn new() -> Self;
    fn render(&mut self, state: &StateManager<State>);
}

pub trait Viewport {
    type Renderer: Renderer;

    fn new_full_screen() -> Self;
    fn open(&mut self) -> Self::Renderer;
    fn close(&mut self);
}

pub trait Frontend {
    type Input: Input;
    type Viewport: Viewport;
}
