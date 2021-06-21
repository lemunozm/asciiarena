use super::util::{self};
use super::menu::{Menu, MenuWidget};
use super::arena::{Arena, ArenaWidget};

use crate::client::configuration::{Config};
use crate::client::store::{Store, Action};
use crate::client::state::{State};

use crate::client::terminal::input::{InputEvent};
use crate::client::terminal::renderer::{Cursor};

use tui::buffer::{Buffer};
use tui::widgets::{Widget, StatefulWidget};
use tui::layout::{Rect};

use crossterm::event::{KeyCode, KeyModifiers};

enum View {
    Menu,
    Arena,
}

pub struct Gui {
    menu: Menu,
    arena: Arena,
}

impl Gui {
    pub fn new(config: &Config) -> Gui {
        Gui { menu: Menu::new(config), arena: Arena::new(config) }
    }

    fn view(&self, state: &State) -> View {
        match state.server.game.arena {
            Some(_) => View::Arena,
            None => View::Menu,
        }
    }

    pub fn process_event(&mut self, store: &mut Store, event: InputEvent) {
        match event {
            InputEvent::KeyPressed(key_event) => {
                if let KeyCode::Char(character) = key_event.code {
                    if character == 'c' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        return store.dispatch(Action::CloseApp)
                    }
                }
            }
            InputEvent::ResizeDisplay(_, _) => {}
        }

        match self.view(store.state()) {
            View::Menu => self.menu.process_event(store, event),
            View::Arena => self.arena.process_event(store, event),
        }
    }

    pub fn update(&mut self, state: &State) {
        match self.view(state) {
            View::Menu => self.menu.update(state),
            View::Arena => self.arena.update(state),
        }
    }
}

#[derive(derive_new::new)]
pub struct GuiWidget<'a> {
    state: &'a State,
    gui: &'a Gui,
}

impl StatefulWidget for GuiWidget<'_> {
    type State = Cursor;
    fn render(self, area: Rect, buffer: &mut Buffer, cursor: &mut Cursor) {
        match self.gui.view(self.state) {
            View::Menu => {
                let area = util::centered_area(area, MenuWidget::dimension());
                MenuWidget::new(self.state, &self.gui.menu).render(area, buffer, cursor)
            }
            View::Arena => {
                let dimension = ArenaWidget::dimension(self.state);
                let area = util::centered_area(area, dimension);
                ArenaWidget::new(self.state, &self.gui.arena).render(area, buffer)
            }
        }
    }
}
