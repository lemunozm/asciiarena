use crate::vec2::Vec2;

use rand::{distributions::{Distribution, Uniform}};

pub enum Terrain {
    Empty,
    Wall,
}

pub struct Map {
    size: usize,
    ground: Vec<Terrain>,
    initial_positions: Vec<Vec2>,
}

impl Map {
    pub fn new(size: usize, players: usize) -> Map {
        Map {
            size,
            ground: Vec::new(), //TODO
            initial_positions: Self::random_separated_positions(size, players),
        }
    }

    pub fn initial_position(&self, index: usize) -> Option<Vec2> {
        self.initial_positions.get(index).map(|pos| *pos)
    }

    fn random_separated_positions(size: usize, number: usize) -> Vec<Vec2> {
        let mut rng = rand::thread_rng();

        (0..number).map(|_| {
            let x_range = Uniform::from(0..size);
            let y_range = Uniform::from(0..size);

            Vec2::xy(x_range.sample(&mut rng) as i32, y_range.sample(&mut rng) as i32)
        }).collect()
    }
}
