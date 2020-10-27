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

pub fn is_valid_character_name(name: &str) -> bool {
    name.len() == 1 && name.chars().all(|c| c.is_ascii_uppercase())
}

pub fn is_valid_character(character: char) -> bool {
    character.is_ascii_uppercase()
}

pub mod format {
    use std::borrow::{Borrow};

    pub fn character_list(characters: impl IntoIterator<Item = impl Borrow<char>>) -> String {
        let mut formatted = String::new();
        let mut it = characters.into_iter();
        if let Some(character) = it.next() {
            formatted.push(*character.borrow());
            for character in it {
                formatted.push_str(&format!(", {}", *character.borrow()));
            }
        }
        formatted
    }

    pub fn character_points_list(character_points: impl IntoIterator<Item = (char, usize)>) -> String {
        let mut formatted = String::new();
        let mut it = character_points.into_iter();
        if let Some((character, points)) = it.next() {
            formatted.push_str(&format!("{}: {}", character, points));
            for (character, points) in it {
                formatted.push_str(&format!(", {}: {}", character, points));
            }
        }
        formatted
    }
}


