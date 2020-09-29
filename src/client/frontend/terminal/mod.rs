pub mod renderer;
pub mod input;
pub mod events;

use crate::client::frontend::{Frontend};
use crate::client::actions::{Dispatcher};

pub struct Terminal;

impl Frontend for Terminal {
    type Input = input::TerminalInput;
    type Renderer = renderer::TerminalRenderer;

    fn init_input(actions: impl Dispatcher + 'static) -> Self::Input {
        input::TerminalInput::new(actions)
    }

    fn init_renderer() -> Self::Renderer {
        renderer::TerminalRenderer::new()
    }
}
