use super::input::{InputEvent};

use crate::client::state::{State};
use crate::client::store::{Store};

use tui::backend::{CrosstermBackend};
use tui::{Frame};
use tui::layout::{Rect};

use std::io::{Stdout};


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
