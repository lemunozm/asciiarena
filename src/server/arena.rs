use rand::seq::SliceRandom;

use std::collections::{HashMap};

pub type EntityId = usize;

pub struct Arena {
    number: u8,
    players: HashMap<char, EntityId>,
    ranking: Vec<char>
}

impl Arena {
    pub fn new(number: u8, players_it: impl IntoIterator<Item = char>) -> Arena {
        Arena {
            number,
            players: players_it
                .into_iter()
                .enumerate()
                .map(|(index, player)| (player, index + 1))
                .collect(),
            ranking: Vec::new(),
        }
    }

    pub fn number(&self) -> u8 {
        self.number
    }

    pub fn step(&mut self) {
        self.ranking = self.players.keys().map(|player| *player).collect();
        self.ranking.shuffle(&mut rand::thread_rng());
    }

    pub fn has_finished(&self) -> bool {
        self.ranking.len() == self.players.len()
    }

    pub fn ranking(&self) -> &Vec<char> {
        &self.ranking
    }
}
