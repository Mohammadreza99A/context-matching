use crate::geometry::Point;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ParticleContextType {
    SAILING,
    FISHING,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Particle {
    pub pos: Point,
    pub direction: Point,
    pub heading: f64,
    pub speed: f64,
    pub weight: f64,
    pub context: ParticleContextType,
    pub memory: Vec<ParticleContextType>,
}
