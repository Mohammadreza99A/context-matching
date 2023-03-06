use crate::geometry::Point;
use crate::observation::Observation;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

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

#[derive(Debug, Serialize, Clone)]
pub struct ParticleHistory {
    pub particles: Vec<Particle>,
}

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

impl fmt::Display for ParticleHistory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for particle in &self.particles {
            write!(
                f,
                "{},{},{:.2},{:.2},{:.2},{}\n",
                particle.pos,
                particle.direction,
                particle.heading,
                particle.speed,
                particle.weight,
                particle.context
            )?;
        }
        Ok(())
    }
}
