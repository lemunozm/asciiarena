use serde::{Serialize, Deserialize};

pub type CharacterId = usize;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Builder)]
pub struct Character {
    id: CharacterId,
    symbol: char,
    max_live: usize,
    max_energy: usize,
    speed_base: f32,
}

impl Character {
    pub fn id(&self) -> CharacterId {
        self.id
    }

    pub fn symbol(&self) -> char {
        self.symbol
    }

    pub fn max_live(&self) -> usize {
        self.max_live
    }

    pub fn max_energy(&self) -> usize {
        self.max_energy
    }

    pub fn speed_base(&self) -> f32 {
        self.speed_base
    }
}

