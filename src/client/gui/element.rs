use super::input::{InputEvent};

use crate::client::state::{State};
use crate::client::actions::{ActionManager};
use crate::client::util::store::{Store};

use tui::backend::{CrosstermBackend};
use tui::{Frame};
use tui::layout::{Rect};

use std::io::{Stdout};

pub struct Context<'a, 'b> {
    pub state: &'a State,
    pub frame: &'a mut Frame<'b, CrosstermBackend<Stdout>>,
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn new(state: &'a State, frame: &'a mut Frame<'b, CrosstermBackend<Stdout>>) -> Context<'a, 'b> {
        Context { state: state, frame }
    }
}

pub trait GuiElement {
    fn process_event(&mut self, store: &mut Store<ActionManager>, event: InputEvent) {}
    fn update(&mut self, state: &State) {}
    fn render(&self, ctx: &mut Context, rect: Rect) {}
}

