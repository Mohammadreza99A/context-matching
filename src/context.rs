use crate::geometry::Point;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum ContextType {
    SAILING,
    FISHING,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ContextState {
    pub pos: Point,
    pub direction: Point,
    pub heading: f64,
    pub speed: f64,
    pub context: ContextType,
}
