use crate::vec2::Vec2;

pub struct Rectangle {
    center: Vec2,
    half_size: Vec2,
}

impl Rectangle {
    pub fn new(position: Vec2, dimension: Vec2) -> Rectangle {
        let half_size = dimension * 0.5;
        Rectangle {
            center: position + half_size,
            half_size: half_size,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.center + self.half_size
    }

    pub fn dimension(&self) -> Vec2 {
        self.half_size * 2.0
    }

    pub fn center(&self) -> Vec2 {
        self.center
    }

    pub fn half_size(&self) -> Vec2 {
        self.half_size
    }

    pub fn set_position(&mut self, position: Vec2) {
        self.center = position + self.half_size;
    }

    pub fn set_dimension(&mut self, dimension: Vec2) {
        self.half_size = dimension * 0.5;
    }
}
