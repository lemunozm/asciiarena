mod player;
mod arena;

use player::{Player};
use arena::{Arena};

use crate::character::{Character, CharacterId, CharacterBuilder};

use std::collections::{HashMap, BTreeMap, BTreeSet};

use std::rc::{Rc};

pub struct Game {
    map_size: usize,
    winner_points: usize,

    arena_number: usize,
    arena: Option<Arena>,

    characters: HashMap<CharacterId, Rc<Character>>,

    players: BTreeMap<char, Player>,
}

impl Game {
    pub fn new(
        map_size: usize,
        winner_points: usize,
        player_characters: impl Iterator<Item = char>
    ) -> Game {
        let characters = player_characters
            .map(|symbol| {
                let character = CharacterBuilder::default()
                    .id(CharacterId::Player(symbol))
                    .symbol(symbol)
                    .max_health(Player::MAX_LIFE)
                    .max_energy(Player::MAX_ENERGY)
                    .speed_base(Player::SPEED_BASE)
                    .build()
                    .unwrap();

                (character.id(), Rc::new(character))
            })
            .collect::<HashMap<_, _>>();

        let players = characters
            .values()
            .map(|character|{
                (character.symbol(), Player::new(character.clone()))
            })
            .collect();

        Game {
            map_size,
            winner_points,
            arena_number: 0,
            arena: None,
            players,
            characters,
        }
    }

    pub fn arena(&self) -> Option<&Arena> {
        self.arena.as_ref()
    }

    pub fn characters(&self) -> &HashMap<CharacterId, Rc<Character>> {
        &self.characters
    }

    pub fn player_mut(&mut self, character_symbol: char) -> Option<&mut Player> {
        self.players.get_mut(&character_symbol)
    }

    pub fn players(&self) -> &BTreeMap<char, Player> {
        &self.players
    }

    pub fn arena_number(&self) -> usize {
        self.arena_number
    }

    pub fn pole(&self) -> Vec<&Player> {
        let mut sorted_players = self.players.values().collect::<Vec<_>>();

        sorted_players.sort_by(|a, b| b.total_points().partial_cmp(&a.total_points()).unwrap());
        sorted_players
    }

    pub fn create_new_arena(&mut self) -> &Arena {
        let mut arena = Arena::new(self.map_size, self.players.len());

        for (index, player) in self.players.values_mut().enumerate() {
            let position = arena.map().initial_position(index);
            let character = player.character().clone();
            let control = arena.create_entity(character, position);
            player.set_control(control.clone());
            player.reset_partial_points();
        }

        self.arena = Some(arena);
        self.arena_number += 1;
        self.arena.as_ref().unwrap()
    }

    pub fn step(&mut self) {
        let living_players_before = self.living_players();

        self.arena.as_mut().unwrap().update();

        let living_players_after = self.living_players();
        let death_players = living_players_before.difference(&living_players_after);

        let player_number = self.players.len();
        for symbol in death_players {
            let player = self.players.get_mut(symbol).unwrap();
            player.remove_control();
            player.update_points(player_number - living_players_before.len());
        }
    }

    pub fn living_players(&self) -> BTreeSet<char> {
        self.players
            .values()
            .filter(|player| player.is_alive())
            .map(|player| player.character().symbol())
            .collect()
    }

    pub fn has_finished(&self) -> bool {
        self.players
            .values()
            .find(|&player| player.total_points() >= self.winner_points)
            .is_some()
    }
}
