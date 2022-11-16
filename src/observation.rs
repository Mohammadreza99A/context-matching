use crate::context::ContextType;
use crate::geometry::Point;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Observation {
    pub pos: Point,
    pub time: f64,
    pub context: ContextType,
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
                context: if record.label.contains("fishing") {
                    ContextType::FISHING
                } else {
                    ContextType::SAILING
                },
            };
            observations.push(obs);
        }

        Ok(observations)
    }
}
