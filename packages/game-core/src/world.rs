use bevy_ecs::prelude::*;

use crate::components::{BuildingType, TerrainType};

pub const WORLD_SIZE: i32 = 50;

#[derive(Resource, Clone)]
pub struct WorldGrid {
    pub terrain: Vec<TerrainType>,
    pub buildings: Vec<Option<BuildingType>>,
    pub seed: u32,
}

impl WorldGrid {
    pub fn index(x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x >= WORLD_SIZE || y >= WORLD_SIZE {
            return None;
        }
        Some((y * WORLD_SIZE + x) as usize)
    }

    pub fn terrain_at(&self, x: i32, y: i32) -> Option<TerrainType> {
        Self::index(x, y).map(|i| self.terrain[i])
    }

    pub fn building_at(&self, x: i32, y: i32) -> Option<BuildingType> {
        Self::index(x, y).and_then(|i| self.buildings[i])
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        match self.terrain_at(x, y) {
            Some(t) if t.walkable() => match self.building_at(x, y) {
                Some(b) => !b.blocks_movement(),
                None => true,
            },
            _ => false,
        }
    }

    pub fn remove_building(&mut self, x: i32, y: i32) -> bool {
        if let Some(i) = Self::index(x, y) {
            if self.buildings[i].is_some() {
                self.buildings[i] = None;
                return true;
            }
        }
        false
    }

    pub fn place_building(&mut self, x: i32, y: i32, building: BuildingType) -> bool {
        if !self.is_walkable(x, y) {
            return false;
        }
        if self.building_at(x, y).is_some() {
            return false;
        }
        if let Some(i) = Self::index(x, y) {
            self.buildings[i] = Some(building);
            true
        } else {
            false
        }
    }
}

pub fn generate_world(seed: u32) -> WorldGrid {
    let len = (WORLD_SIZE * WORLD_SIZE) as usize;
    let mut terrain = Vec::with_capacity(len);
    for y in 0..WORLD_SIZE {
        for x in 0..WORLD_SIZE {
            terrain.push(terrain_at(x, y, seed));
        }
    }
    WorldGrid {
        terrain,
        buildings: vec![None; len],
        seed,
    }
}

fn terrain_at(x: i32, y: i32, seed: u32) -> TerrainType {
    let nx = x as f32 / WORLD_SIZE as f32;
    let ny = y as f32 / WORLD_SIZE as f32;
    let h = hash_noise(x, y, seed);
    let dist = ((nx - 0.5).powi(2) + (ny - 0.5).powi(2)).sqrt();

    if dist > 0.42 || h < 0.12 {
        TerrainType::Water
    } else if h < 0.28 {
        TerrainType::Sand
    } else {
        TerrainType::Grass
    }
}

fn hash_noise(x: i32, y: i32, seed: u32) -> f32 {
    let mut n = (x as u32)
        .wrapping_mul(374761393)
        .wrapping_add(y as u32)
        .wrapping_mul(668265263)
        .wrapping_add(seed);
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n & 0xFFFF) as f32 / 65535.0
}

pub const BERRIES_PER_BUSH: u8 = 3;
pub const NEED_THRESHOLD: f32 = 30.0;
pub const NEED_RESTORE: f32 = 100.0;
pub const FOOD_DECAY_PER_SEC: f32 = 2.0;
pub const SLEEP_DECAY_PER_SEC: f32 = 1.5;
pub const MOVE_SPEED: f32 = 4.0;
pub const WANDER_MIN_RADIUS: i32 = 3;
pub const WANDER_RADIUS: i32 = 10;
pub const WANDER_PICK_ATTEMPTS: usize = 8;
/// Max Manhattan ring from a bed when searching for a vacate cell after sleep.
pub const VACATE_SEARCH_RADIUS: i32 = 5;
/// Time spent on the bed tile before sleep need is restored.
pub const SLEEP_ON_BED_SEC: f32 = 2.0;
