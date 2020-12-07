use crate::vec2::Vec2;

use rand::{distributions::{Distribution, Uniform}};

pub enum Terrain {
    //Floor,
    //Wall,
}

pub struct Map {
    size: usize,
    _ground: Vec<Terrain>,
    initial_positions: Vec<Vec2>,
}

impl Map {
    pub fn new(size: usize, players_number: usize) -> Map {
        Map {
            size,
            _ground: Vec::new(), //TODO
            initial_positions: Self::random_separated_positions(size, players_number),
        }
    }

    fn random_separated_positions(size: usize, count: usize) -> Vec<Vec2> {
        let mut rng = rand::thread_rng();

        (0..count).map(|_| {
            let x_range = Uniform::from(1..size - 1);
            let y_range = Uniform::from(1..size - 1);

            Vec2::xy(x_range.sample(&mut rng) as i32, y_range.sample(&mut rng) as i32)
        }).collect()
    }

    pub fn initial_position(&self, index: usize) -> Vec2 {
        *self.initial_positions.get(index).unwrap()
    }

    pub fn contains(&self, position: Vec2) -> bool {
        position.x >= 1 &&
        position.y >= 1 &&
        position.x < (self.size - 1) as i32 &&
        position.y < (self.size - 1) as i32
    }
}
