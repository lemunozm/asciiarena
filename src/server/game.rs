use std::time::{Instant};

pub struct Game {
    prepare_arena_timestamp: Option<Instant>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            prepare_arena_timestamp: None,
        }
    }

    pub fn init(&mut self) {

    }

    pub fn process_event(&mut self) {

    }

    pub fn prepare_arena_timestamp(&self) -> Option<Instant> {
        self.prepare_arena_timestamp
    }
}
