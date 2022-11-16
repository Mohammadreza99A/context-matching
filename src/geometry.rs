use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
