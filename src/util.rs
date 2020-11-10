use crate::vec2::Vec2;

use serde::{Serialize, Deserialize};

use rand::{distributions::{Distribution, Standard}, Rng};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub enum Direction {
    Up, Down, Left, Right,
}

impl Direction {
    pub fn vec2(&self) -> Vec2 {
        match *self {
            Direction::Up => Vec2::y(-1.0),
            Direction::Right => Vec2::x(1.0),
            Direction::Down => Vec2::y(1.0),
            Direction::Left => Vec2::x(-1.0),
        }
    }

    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }

    pub fn turn_left(&mut self) {
        match *self {
            Direction::Up => *self = Direction::Right,
            Direction::Right => *self = Direction::Down,
            Direction::Down => *self = Direction::Left,
            Direction::Left => *self = Direction::Up,
        }
    }

    pub fn turn_right(&mut self) {
        match *self {
            Direction::Up => *self = Direction::Left,
            Direction::Right => *self = Direction::Up,
            Direction::Down => *self = Direction::Right,
            Direction::Left => *self = Direction::Down,
        }
    }
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        match rng.gen_range(0, 4) {
            0 => Direction::Up,
            1 => Direction::Right,
            2 => Direction::Down,
            3 => Direction::Left,
            _ => unreachable!(),
        }
    }
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

    pub fn character_list(list: impl IntoIterator<Item = impl Borrow<char>>) -> String {
        let mut formatted = String::new();
        let mut it = list.into_iter();
        if let Some(character) = it.next() {
            formatted.push(*character.borrow());
            for character in it {
                formatted.push_str(&format!(", {}", *character.borrow()));
            }
        }
        formatted
    }

    pub fn character_points_list(list: impl IntoIterator<Item = (char, usize)>) -> String {
        let mut formatted = String::new();
        let mut it = list.into_iter();
        if let Some((character, points)) = it.next() {
            formatted.push_str(&format!("{}: {}", character, points));
            for (character, points) in it {
                formatted.push_str(&format!(", {}: {}", character, points));
            }
        }
        formatted
    }
}


