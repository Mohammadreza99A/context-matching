use crate::geometry::Point;
use crate::particle::ParticleContextType;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Observation {
    pub pos: Point,
    pub time: f64,
    pub heading: f64,
    pub speed: f64,
    pub context: ParticleContextType,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
pub struct AisRecord {
    pub id: String,
    pub t: String,
    pub longitude: f64,
    pub latitude: f64,
    pub x: f64,
    pub y: f64,
    pub signed_turn: f64,
    pub bearing: f64,
    pub time_gap: f64,
    pub distance_gap: f64,
    pub euc_speed: f64,
    pub distanceToShore: f64,
    pub label: String,
}

impl Observation {
    pub fn from_csv(_filename: &str) -> Result<Vec<Observation>, csv::Error> {
        let mut observations: Vec<Observation> = Vec::new();

        let mut rdr = csv::Reader::from_path(_filename)?;
        for result in rdr.deserialize() {
            let record: AisRecord = result?;
            let obs: Observation = Observation {
                pos: Point {
                    x: record.x,
                    y: record.y,
                },
                time: record.time_gap,
                heading: record.bearing,
                speed: record.euc_speed,
                context: if record.label.contains("fishing") {
                    ParticleContextType::Fishing
                } else if record.label.contains("01-sailing") {
                    ParticleContextType::GoFishing
                } else {
                    ParticleContextType::GoToPort
                },
            };
            observations.push(obs);
        }

        Ok(observations)
    }
}

impl fmt::Display for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "position,time,heading,speed,context\n")?;
        write!(
            f,
            "{},{:.2},{:.2},{:.2},{}\n",
            self.pos, self.time, self.heading, self.speed, self.context
        )?;
        Ok(())
    }
}
