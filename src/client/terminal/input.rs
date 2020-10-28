use super::events::{TerminalEventCollector};

use crossterm::event::{Event as TermEvent, KeyEvent};

#[derive(Debug)]
pub enum InputEvent {
    KeyPressed(KeyEvent),
    ResizeDisplay(usize, usize),
}

pub struct InputReceiver {
    _event_collector: TerminalEventCollector, // Kept because we need its internal thread running
}

impl InputReceiver {
    pub fn new(event_callback: impl Fn(InputEvent) + Send + Sync + 'static) -> InputReceiver {
        let _event_collector = TerminalEventCollector::new(move |event| {
            Self::process_event(event, &event_callback);
        });

        InputReceiver { _event_collector, }
    }

    fn process_event(event: TermEvent, event_callback: &impl Fn(InputEvent)) {
        match event {
            TermEvent::Key(key_event) => {
                event_callback(InputEvent::KeyPressed(key_event));
            }
            TermEvent::Resize(width, height) => {
                event_callback(InputEvent::ResizeDisplay(width as usize, height as usize));
            }
            _ => (),
        };
    }
}
