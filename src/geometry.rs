use serde::{Deserialize, Serialize};

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

    pub fn scaled_weighted_distance_from_line(distance: f64, d0: f64, c: f64) -> f64 {
        1.0f64 - (c * (-distance / d0).exp())
    }
}
