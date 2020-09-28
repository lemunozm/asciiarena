pub mod terminal;

pub use super::actions::{ActionManager, Dispatcher};
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
    fn open(&mut self);
    fn close(&mut self);
    fn create_renderer(&mut self) -> Self::Renderer;
}

pub trait Frontend {
    type Input: Input;
    type Viewport: Viewport;
}
