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
}
