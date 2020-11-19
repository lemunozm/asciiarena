use super::arena::{Arena};

use std::collections::{HashMap};

pub struct Game {
    arena_number: usize,
    arena: Option<Arena>,
    player_points: HashMap<char, usize>,
    winner_points: usize,
    map_size: usize,
}

impl Game {
    pub fn new(players_it: impl IntoIterator<Item = char>, winner_points: usize, map_size: usize) -> Game {
        Game {
            arena_number: 0,
            arena: None,
            player_points: players_it.into_iter().map(|player|(player, 0)).collect(),
            winner_points,
            map_size,
        }
    }

    pub fn arena(&self) -> Option<&Arena> {
        self.arena.as_ref()
    }

    pub fn arena_number(&self) -> usize {
        self.arena_number
    }

    pub fn pole(&self) -> Vec<(char, usize)> {
        let mut sorted_players = self.player_points
            .clone()
            .into_iter()
            .collect::<Vec<_>>();

        sorted_players.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap());
        sorted_players
    }

    pub fn create_new_arena(&mut self) -> &Arena {
        self.arena_number += 1;

        let players = self.player_points.keys().map(|player| *player);
        self.arena = Some(Arena::new(self.map_size, players));
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
