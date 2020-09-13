use rand::seq::SliceRandom;

use std::collections::{HashMap};

pub type EntityId = usize;

pub struct Arena {
    id: usize,
    players: HashMap<String, EntityId>,
    ranking: Vec<String>
}

impl Arena {
    pub fn new(id: usize, players_it: impl IntoIterator<Item = String>) -> Arena {
        Arena {
            id,
            players: players_it.into_iter().enumerate().map(|(index, player)| (player, index + 1)).collect(),
            ranking: Vec::new(),
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn step(&mut self) {
        self.ranking = self.players.keys().map(|player| player.into()).collect();
        self.ranking.shuffle(&mut rand::thread_rng());
    }

    pub fn has_finished(&self) -> bool {
        self.ranking.len() == self.players.len()
    }

    pub fn ranking(&self) -> &Vec<String> {
        &self.ranking
    }
}
