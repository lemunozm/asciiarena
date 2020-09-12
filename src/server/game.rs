use super::arena::{Arena};

pub struct Game {
    arena: Option<Arena>,
    points: Vec<usize>,
    winner_points: usize,
}

impl Game {
    pub fn new(winner_points: usize) -> Game {
        Game {
            arena: None,
            points: Vec::new(),
            winner_points
        }
    }

    pub fn create_new_arena(&mut self) -> &Arena {
        let new_id = match self.arena.as_ref() {
            Some(arena) => arena.id() + 1,
            None => 1,
        };
        self.arena = Some(Arena::new(new_id));
        &self.arena.as_ref().unwrap()
    }

    pub fn step(&mut self) {
        self.arena.as_mut().unwrap().step();
        //TODO
    }

    pub fn arena(&self) -> Option<&Arena> {
        self.arena.as_ref()
    }

    pub fn has_finished(&self) -> bool {
        self.points.iter().find(|&&p| p > self.winner_points).is_some()
    }
}
