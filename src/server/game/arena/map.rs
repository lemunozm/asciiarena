use crate::vec2::Vec2;
use crate::message::Terrain;

pub struct Map {
    size: usize,
    ground: Vec<Terrain>,
}

impl Map {
    pub fn new(size: usize) -> Map {
        Map {
            size,
            ground: Self::build_ground(size, 0),
        }
    }

    fn build_ground(size: usize, _seed: usize) -> Vec<Terrain> {
        (0..size * size).map(|index|{
            let x = index % size;
            let y = index / size;

            if x == 0 || y == 0 || x == size - 1 || y == size - 1 {
                Terrain::Wall
            }
            else {
                Terrain::Floor
            }

        }).collect()
    }

    pub fn ground(&self) -> &Vec<Terrain> {
        &self.ground
    }

    pub fn get(&self, position: Vec2) -> Terrain {
        assert!(position.x >= 0 && position.x < self.size as i32);
        assert!(position.y >= 0 && position.y < self.size as i32);
        self.ground[position.y as usize* self.size + position.x as usize]
    }

    pub fn position_of(&self, index: usize) -> Vec2 {
        assert!(index < self.size * self.size);
        Vec2::xy((index % self.size) as i32, (index / self.size) as i32)
    }
}

