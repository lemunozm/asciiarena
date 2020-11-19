use crate::vec2::Vec2;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entity {
    symbol: char,
    position: Vec2,
    max_live: usize,
    live: usize,
    max_energy: usize,
    energy: usize,
}

impl Entity {
    pub fn new(symbol: char, position: Vec2, max_live: usize, max_energy: usize) -> Entity {
        Entity {
            symbol,
            position,
            max_live: 100,
            live: 100,
            max_energy: 100,
            energy: 100,
        }
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    pub fn displace(&mut self, displacement: Vec2) {
        self.position += displacement;
    }

    pub fn symbol(&self) -> char {
        self.symbol
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn max_live(&self) -> usize {
        self.max_live
    }

    pub fn max_energy(&self) -> usize {
        self.max_energy
    }

    pub fn live(&self) -> usize {
        self.live
    }

    pub fn energy(&self) -> usize {
        self.energy
    }
}

