use serde::{Deserialize, Serialize};
use std::cmp::{Eq, Ord, Ordering};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn norm(&self) -> f64 {
        let f = self.x.powf(2.0) + self.y.powf(2.0);
        f.sqrt()
    }

    // pub fn dot(&self, other: &Point) -> f64 {
    //     self.x * other.x + self.y * other.y
    // }

    // pub fn distance(&self, other: &Point) -> f64 {
    //     let dx = (other.x - self.x).powi(2);
    //     let dy = (other.y - self.y).powi(2);
    //     (dx + dy).sqrt()
    // }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f64> for Point {
    type Output = Point;

    fn mul(self, scalar: f64) -> Point {
        Point {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl Div<f64> for Point {
    type Output = Point;

    fn div(self, scalar: f64) -> Point {
        Point {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl Eq for Point {}

impl Ord for Point {
    fn cmp(&self, other: &Point) -> Ordering {
        let self_magnitude = (self.x * self.x + self.y * self.y) as i64;
        let other_magnitude = (other.x * other.x + other.y * other.y) as i64;
        self_magnitude.cmp(&other_magnitude)
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Point) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}
