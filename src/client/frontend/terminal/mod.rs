pub mod viewport;
pub mod renderer;
pub mod input;
pub mod events;

use crate::client::frontend::{Frontend};

pub struct Terminal;

impl Frontend for Terminal {
    type Input = input::TerminalInput;
    type Viewport = viewport::Viewport;
}
