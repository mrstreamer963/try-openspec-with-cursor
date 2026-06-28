use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn grid_cell(self) -> (i32, i32) {
        (self.x.floor() as i32, self.y.floor() as i32)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TerrainType {
    Water,
    Sand,
    Grass,
}

impl TerrainType {
    pub fn walkable(self) -> bool {
        !matches!(self, TerrainType::Water)
    }
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum BuildingType {
    Wall,
    Bed,
    BerryBush,
}

impl BuildingType {
    pub fn blocks_movement(self) -> bool {
        matches!(self, BuildingType::Wall)
    }

    pub fn satisfies_need(self) -> Option<NeedKind> {
        match self {
            BuildingType::Bed => Some(NeedKind::Sleep),
            BuildingType::BerryBush => Some(NeedKind::Food),
            BuildingType::Wall => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NeedKind {
    Food,
    Sleep,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Needs {
    pub food: f32,
    pub sleep: f32,
}

impl Needs {
    pub fn new_full() -> Self {
        Self {
            food: 100.0,
            sleep: 100.0,
        }
    }

    pub fn get(&self, kind: NeedKind) -> f32 {
        match kind {
            NeedKind::Food => self.food,
            NeedKind::Sleep => self.sleep,
        }
    }

    pub fn set(&mut self, kind: NeedKind, value: f32) {
        let v = value.clamp(0.0, 100.0);
        match kind {
            NeedKind::Food => self.food = v,
            NeedKind::Sleep => self.sleep = v,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskKind {
    Idle,
    Eat,
    Sleep,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Task {
    pub kind: TaskKind,
    pub target_x: i32,
    pub target_y: i32,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            kind: TaskKind::Idle,
            target_x: 0,
            target_y: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColonistId(pub u32);

#[derive(Component)]
pub struct Colonist;

#[derive(Component)]
pub struct Building;

#[derive(Component, Clone, Debug, Default)]
pub struct Path {
    pub waypoints: Vec<(i32, i32)>,
    pub index: usize,
}

impl Path {
    pub fn clear(&mut self) {
        self.waypoints.clear();
        self.index = 0;
    }
}
