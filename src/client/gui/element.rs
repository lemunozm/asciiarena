use super::input::{InputEvent};

use crate::client::state::{State};
use crate::client::store::{Store};

use tui::backend::{CrosstermBackend};
use tui::{Frame};
use tui::layout::{Rect};

use std::io::{Stdout};

pub struct Context<'a, 'b> {
    pub state: &'a State,
    pub frame: &'a mut Frame<'b, CrosstermBackend<Stdout>>,
}

impl<'a, 'b> Context<'a, 'b> {
    pub fn new(
        state: &'a State,
        frame: &'a mut Frame<'b, CrosstermBackend<Stdout>>
    ) -> Context<'a, 'b> {
        Context { state: state, frame }
    }
}

pub trait GuiElement {
    fn process_event(&mut self, store: &mut Store, event: InputEvent);
    fn update(&mut self, state: &State);
    fn render(&self, ctx: &mut Context, rect: Rect);
}


// ELEMENT TEMPLATE
/*

use crate::client::state::{State};
use crate::client::store::{Store};
use crate::client::gui::input::{InputEvent};
use crate::client::gui::element::{Context, GuiElement};

use tui::layout::{Rect};

pub struct NewElement {}

impl GuiElement for NewElement {
    fn process_event(&mut self, store: &mut Store, event: InputEvent) {
        //TODO
    }

    fn update(&mut self, state: &State) {
        //TODO
    }

    fn render(&self, ctx: &mut Context, space: Rect) {
        //TODO
    }
}

*/
