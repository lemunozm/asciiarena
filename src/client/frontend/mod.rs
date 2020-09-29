pub mod terminal;

pub use super::actions::{Dispatcher};
pub use super::state::{State};
pub use super::util::store::{StateManager};

pub trait Renderer: Drop {
    fn render(&mut self, state: &StateManager<State>);
}

pub trait Frontend {
    type Input;
    type Renderer: Renderer;

    fn init_input(actions: impl Dispatcher + 'static) -> Self::Input;
    fn init_renderer() -> Self::Renderer;
}
