use super::events::{TerminalEventCollector};

use crate::client::actions::{Action, Dispatcher};

use crossterm::event::{Event as TermEvent, KeyEvent, KeyCode, KeyModifiers};

pub struct InputDispatcher {
    _event_collector: TerminalEventCollector, // Kept because we need its internal thread running
}

impl InputDispatcher {
    pub fn new(mut actions: impl Dispatcher + 'static) -> InputDispatcher {
        let _event_collector = TerminalEventCollector::new(move |event| {
            Self::process_event(event, &mut actions);
        });

        InputDispatcher { _event_collector, }
    }

    fn process_event(event: TermEvent, actions: &mut dyn Dispatcher) {
        match event {
            TermEvent::Key(KeyEvent{code, modifiers}) => match code {
                KeyCode::Esc => {
                    actions.dispatch(Action::Close);
                },
                KeyCode::Char(character) => {
                    if character == 'c' && modifiers.contains(KeyModifiers::CONTROL) {
                        actions.dispatch(Action::Close);
                    }
                },
                _ => (),
            }
            TermEvent::Resize(width, height) => {
                actions.dispatch(Action::ResizeWindow(width as usize, height as usize));
            }
            _ => (),
        };
    }
}
