use super::arena::{Arena};

use std::collections::{HashMap};

pub struct Game {
    arena: Option<Arena>,
    player_points: HashMap<String, usize>,
    winner_points: usize,
}

impl Game {
    pub fn new(players_it: impl IntoIterator<Item = String>, winner_points: usize) -> Game {
        Game {
            arena: None,
            player_points: players_it.into_iter().map(|player|(player, 0)).collect(),
            winner_points,
        }
    }

    pub fn arena(&self) -> Option<&Arena> {
        self.arena.as_ref()
    }

    pub fn pole(&self) -> Vec<(String, usize)> {
        let mut sorted_players: Vec<(String, usize)> = self.player_points.clone().into_iter().collect();
        sorted_players.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        sorted_players
    }

    pub fn create_new_arena(&mut self) -> &Arena {
        let new_id = match self.arena.as_ref() {
            Some(arena) => arena.id() + 1,
            None => 1,
        };

        let players = self.player_points.keys().map(|player| player.into());
        self.arena = Some(Arena::new(new_id, players));
        &self.arena.as_ref().unwrap()
    }

    pub fn step(&mut self) {
        let arena = self.arena.as_mut().unwrap();
        if !arena.has_finished() {
            arena.step();
            if arena.has_finished() {
                for (index, player) in arena.ranking().iter().rev().enumerate() {
                    let points = self.player_points.get_mut(player).unwrap();
                    *points += index;
                }
            }
        }
    }


    pub fn has_finished(&self) -> bool {
        self.player_points.values().find(|&&p| p >= self.winner_points).is_some()
    }
}
