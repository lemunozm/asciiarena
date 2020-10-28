use super::menu::{self, Menu};
use super::arena::{Arena};

use crate::client::configuration::{Config};
use crate::client::store::{Store, Action};
use crate::client::state::{State, GameStatus};

use crate::client::gui::input::{InputEvent};
use crate::client::gui::element::{GuiElement, Context};
use crate::client::gui::elements::util::{self};

use tui::layout::{Rect};

use crossterm::event::{KeyCode, KeyModifiers};

enum Selected {
    Menu,
    Arena,
}

pub struct Gui {
    menu: Menu,
    arena: Arena,
}

impl Gui {
    pub fn new(config: &Config) -> Gui {
        Gui {
            menu: Menu::new(config),
            arena: Arena::new(config),
        }
    }

    fn selected(&self, state: &State) -> Selected {
        match state.server.game.status {
            GameStatus::NotStarted => Selected::Menu,
            GameStatus::Started => Selected::Arena,
                //ArenaStatus::Preparing && arena_num = 1
            GameStatus::Finished => Selected::Arena,
        }
    }
}

impl GuiElement for Gui {
    fn process_event(&mut self, store: &mut Store, event: InputEvent) {
        match event {
            InputEvent::KeyPressed(key_event) => match key_event.code {
                KeyCode::Char(character) => {
                    if character == 'c' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                        return store.dispatch(Action::Close);
                    }
                },
                _ => (),
            }
            InputEvent::ResizeDisplay(_, _) => {},
        }

        match self.selected(store.state()) {
            Selected::Menu => self.menu.process_event(store, event),
            Selected::Arena => self.arena.process_event(store, event),
        }
    }

    fn update(&mut self, state: &State) {
        match self.selected(state) {
            Selected::Menu => self.menu.update(state),
            Selected::Arena => self.arena.update(state),
        }
    }

    fn render(&self, ctx: &mut Context, space: Rect) {
        match self.selected(ctx.state) {
            Selected::Menu => {
                let space = util::centered_space(space, menu::DIMENSION);
                self.menu.render(ctx, space);
            },
            Selected::Arena => {
                let dimension = self.arena.required_dimension(ctx.state);
                let space = util::centered_space(space, dimension);
                self.arena.render(ctx, space);
            }
        }
    }
}
