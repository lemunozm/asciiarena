//use crate::vec2::Vec2;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Direction {
    Up, Down, Left, Right, None,
}

impl Direction {
    /*
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
    */
}

pub type SessionToken = usize;

pub fn is_valid_player_name(name: &str) -> bool {
    name.len() == 1 && name.chars().all(|c| c.is_ascii_uppercase())
}

pub mod format {
    pub fn player_names<S: AsRef<str> + Ord>(players: impl IntoIterator<Item = S>) -> String {
        let mut formatted = String::new();
        let mut it = players.into_iter();
        if let Some(name) = it.next() {
            formatted.push_str(name.as_ref());
            for name in it {
                formatted.push_str(&format!(", {}", name.as_ref()));
            }
        }
        formatted
    }

    pub fn player_points<S: AsRef<str> + Ord>(player_points: impl IntoIterator<Item = (S, usize)>) -> String {
        let mut formatted = String::new();
        let mut it = player_points.into_iter();
        if let Some((player, points)) = it.next() {
            formatted.push_str(&format!("{}: {}", player.as_ref(), points));
            for (player, points) in it {
                formatted.push_str(&format!(", {}: {}", player.as_ref(), points));
            }
        }
        formatted
    }
}


