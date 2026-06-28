use serde::{Deserialize, Serialize};

use crate::components::{BuildingType, TaskKind, TerrainType};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IncomingEvent {
    SetPaused { paused: bool },
    SetSpeed { multiplier: f32 },
    Build {
        building: BuildingType,
        x: i32,
        y: i32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileSnapshot {
    pub x: i32,
    pub y: i32,
    pub terrain: TerrainType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BuildingSnapshot {
    pub x: i32,
    pub y: i32,
    pub building: BuildingType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub berries: Option<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ColonistSnapshot {
    pub id: u32,
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub food: f32,
    pub sleep: f32,
    pub task: TaskKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstructionSiteSnapshot {
    pub x: i32,
    pub y: i32,
    pub building: BuildingType,
    pub progress: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tiles: Vec<TileSnapshot>,
    pub buildings: Vec<BuildingSnapshot>,
    pub construction_sites: Vec<ConstructionSiteSnapshot>,
    pub colonists: Vec<ColonistSnapshot>,
    pub paused: bool,
    pub speed: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutgoingEvent {
    StateSnapshot(StateSnapshot),
    Error { message: String },
}
