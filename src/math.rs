use std::ops::{Add, Mul, Sub};

#[derive(Debug)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn rotate(&mut self, radian: f64) {
        let old_x = self.x;
        self.x = self.x * radian.cos() - self.y * radian.sin();
        self.y = old_x * radian.sin() + self.y * radian.cos();
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Add for &Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for &Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs)
    }
}

impl Mul<f64> for &Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f64) -> Self::Output {
        Self::Output::new(self.x * rhs, self.y * rhs)
    }
}
