use crate::vec2::Vec2;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Direction {
    Up, Down, Left, Right, None,
}

impl Direction {
    pub fn vec2(&self) -> Vec2 {
        match *self {
            Direction::Up => Vec2::y(-1.0),
            Direction::Down => Vec2::y(1.0),
            Direction::Right => Vec2::x(1.0),
            Direction::Left => Vec2::x(-1.0),
            Direction::None => Vec2::zero(),
        }
    }

    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
            Direction::None => Direction::None,
        }
    }
}

pub type SessionToken = usize;
