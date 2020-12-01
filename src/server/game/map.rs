use crate::vec2::Vec2;

use rand::{distributions::{Distribution, Uniform}};

pub enum Terrain {
    //Empty,
    //Wall,
}

pub struct Map {
    _size: usize,
    _ground: Vec<Terrain>,
    initial_positions: Vec<Vec2>,
}

impl Map {
    pub fn new(_size: usize, players_number: usize) -> Map {
        Map {
            _size,
            _ground: Vec::new(), //TODO
            initial_positions: Self::random_separated_positions(_size, players_number),
        }
    }

    fn random_separated_positions(size: usize, count: usize) -> Vec<Vec2> {
        let mut rng = rand::thread_rng();

        (0..count).map(|_| {
            let x_range = Uniform::from(0..size);
            let y_range = Uniform::from(0..size);

            Vec2::xy(x_range.sample(&mut rng) as i32, y_range.sample(&mut rng) as i32)
        }).collect()
    }

    pub fn initial_position(&self, index: usize) -> Vec2 {
        *self.initial_positions.get(index).unwrap()
    }
}
