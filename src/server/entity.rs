use crate::vec2::Vec2;

pub struct Entity {
    symbol: char,
    position: Vec2,
}

impl Entity {
    pub fn new(symbol: char, position: Vec2) -> Entity {
        Entity {
            symbol,
            position,
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
}

