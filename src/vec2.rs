use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign, SubAssign, MulAssign, DivAssign};

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default, Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: i32,
    pub y: i32,
}

impl Vec2 {
    pub fn zero() -> Vec2 {
        Vec2 {x: 0, y: 0}
    }

    pub fn xy(x: i32, y: i32) -> Vec2 {
        Vec2 {x, y}
    }

    pub fn x(x: i32) -> Vec2 {
        Vec2 {x, y: 0}
    }

    pub fn y(y: i32) -> Vec2 {
        Vec2 {x: 0, y}
    }

    pub fn clear(&mut self) {
        self.x = 0;
        self.y = 0;
    }

    pub fn length(&self) -> f32 {
        (self.square_length() as f32).sqrt()
    }

    pub fn square_length(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
}

impl Add for Vec2 {
    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul for Vec2 {
    type Output = i32;

    fn mul(self, other: Vec2) -> i32 {
        self.x * other.x + self.y * other.y
    }
}

impl Mul<i32> for Vec2 {
    type Output = Vec2;

    fn mul(self, scalar: i32) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<i32> for Vec2 {
    type Output = Vec2;

    fn div(self, scalar: i32) -> Vec2 {
        Vec2 {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Vec2 {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Vec2) {
        *self = *self + other
    }
}

impl SubAssign for Vec2 {
    fn sub_assign(&mut self, other: Vec2) {
        *self = *self - other
    }
}

impl MulAssign<i32> for Vec2 {
    fn mul_assign(&mut self, scalar: i32) {
        *self = *self * scalar
    }
}

impl DivAssign<i32> for Vec2 {
    fn div_assign(&mut self, scalar: i32) {
        *self = *self / scalar
    }
}
