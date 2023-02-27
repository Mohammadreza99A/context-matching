use crate::geometry::Point;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParticleContextType {
    GoFishing,
    Fishing,
    GoToPort,
}

impl FromStr for ParticleContextType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GoFishing" => Ok(ParticleContextType::GoFishing),
            "Fishing" => Ok(ParticleContextType::Fishing),
            "GoToPort" => Ok(ParticleContextType::GoToPort),
            _ => Err(format!("Invalid particle context type: {}", s)),
        }
    }
}

impl fmt::Display for ParticleContextType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParticleContextType::GoFishing => write!(f, "GoFishing"),
            ParticleContextType::Fishing => write!(f, "Fishing"),
            ParticleContextType::GoToPort => write!(f, "GoToPort"),
        }
    }
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
