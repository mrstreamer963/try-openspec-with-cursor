use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use crate::content::{BuildingId, ContentRegistry, NeedId, StatusId};

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

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct Needs(pub HashMap<NeedId, f32>);

impl Needs {
    pub fn new_full(content: &ContentRegistry) -> Self {
        let mut map = HashMap::new();
        for (idx, need) in content.needs.iter().enumerate() {
            map.insert(NeedId(idx as u8), need.max);
        }
        Needs(map)
    }

    pub fn with_values(content: &ContentRegistry, food: f32, sleep: f32) -> Self {
        let mut needs = Self::new_full(content);
        needs.set(content.food_need, food);
        needs.set(content.sleep_need, sleep);
        needs
    }

    pub fn get(&self, id: NeedId) -> f32 {
        self.0.get(&id).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, id: NeedId, value: f32) {
        let max = 100.0;
        self.0.insert(id, value.clamp(0.0, max));
    }
}

#[derive(Component, Clone, Debug, Default, Serialize, Deserialize)]
pub struct ActiveStatuses(pub HashSet<StatusId>);

impl ActiveStatuses {
    pub fn has(&self, id: StatusId) -> bool {
        self.0.contains(&id)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskKind {
    Idle,
    Eat,
    Sleep,
    Build,
}

pub const BUILD_WORK_PER_TICK: f32 = 1.0;

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Task {
    pub kind: TaskKind,
    pub building_x: i32,
    pub building_y: i32,
    pub target_x: i32,
    pub target_y: i32,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            kind: TaskKind::Idle,
            building_x: 0,
            building_y: 0,
            target_x: 0,
            target_y: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ColonistId(pub u32);

pub const COLONIST_NAME_POOL: &[&str] = &[
    "Alex", "Mira", "Finn", "Luna", "Kai", "Nora", "Eli", "Zoe", "Sam", "Ivy", "Leo", "Noa",
];

#[derive(Component, Clone, Debug)]
pub struct ColonistName(pub String);

#[derive(Component)]
pub struct Colonist;

#[derive(Component)]
pub struct Building;

#[derive(Component, Clone, Copy, Debug)]
pub struct ConstructionSite {
    pub building_id: BuildingId,
    pub work_remaining: f32,
    pub reserved_by: Option<Entity>,
}

#[derive(Component, Default)]
pub struct BedOccupancy {
    pub reserved_by: Option<Entity>,
}

/// Countdown while a colonist rests on a bed tile before sleep completes.
#[derive(Component, Clone, Copy, Debug)]
pub struct SleepingOnBed {
    pub remaining: f32,
}

#[derive(Component, Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BerrySupply {
    pub remaining: u8,
}

impl BerrySupply {
    pub fn new(remaining: u8) -> Self {
        Self { remaining }
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct BuildingKind(pub BuildingId);

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
