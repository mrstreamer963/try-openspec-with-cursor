use bevy_ecs::prelude::*;
use serde_json;
use wasm_bindgen::prelude::*;

use crate::components::{
    BerrySupply, BuildingType, ColonistId, ColonistName, ConstructionSite, Needs, Position, Task,
    work_required_for,
};
use crate::events::{
    BuildingSnapshot, ColonistSnapshot, ConstructionSiteSnapshot, IncomingEvent, OutgoingEvent,
    StateSnapshot, TileSnapshot,
};
use crate::systems::{
    auto_assign_tasks, colonist_movement, is_valid_build_tile, needs_decay, spawn_colonists,
    task_execution, update_need_buffs,
};
use crate::world::{generate_world, WorldGrid, WORLD_SIZE};

#[wasm_bindgen]
pub struct Game {
    world: World,
    grid: WorldGrid,
    paused: bool,
    speed: f32,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Game {
        let grid = generate_world(42);
        let mut world = World::new();
        world.insert_resource(grid.clone());
        let _next_id = spawn_colonists(&mut world, &grid);

        Game {
            world,
            grid,
            paused: false,
            speed: 1.0,
        }
    }

    pub fn handle_event(&mut self, json: &str) -> String {
        match serde_json::from_str::<IncomingEvent>(json) {
            Ok(event) => {
                self.dispatch(event);
                String::new()
            }
            Err(e) => {
                let event = OutgoingEvent::Error {
                    message: e.to_string(),
                };
                serde_json::to_string(&event)
                    .unwrap_or_else(|_| r#"{"type":"error","message":"unknown error"}"#.to_string())
            }
        }
    }

    pub fn tick(&mut self, dt: f32) -> String {
        if self.paused || dt <= 0.0 {
            return self.snapshot_json();
        }

        self.world.insert_resource(self.grid.clone());

        needs_decay(&mut self.world, dt);
        update_need_buffs(&mut self.world);
        auto_assign_tasks(&mut self.world, &self.grid);
        colonist_movement(&mut self.world, dt);
        task_execution(&mut self.world, &mut self.grid, dt);

        self.snapshot_json()
    }

    pub fn get_snapshot(&mut self) -> String {
        self.snapshot_json()
    }
}

impl Game {
    fn dispatch(&mut self, event: IncomingEvent) {
        match event {
            IncomingEvent::SetPaused { paused } => self.paused = paused,
            IncomingEvent::SetSpeed { multiplier } => {
                self.speed = multiplier.clamp(0.1, 10.0);
            }
            IncomingEvent::Build { building, x, y } => {
                if !is_valid_build_tile(&mut self.world, &self.grid, x, y) {
                    return;
                }
                self.world.spawn((
                    ConstructionSite {
                        building_type: building,
                        work_remaining: work_required_for(building),
                        reserved_by: None,
                    },
                    Position {
                        x: x as f32,
                        y: y as f32,
                    },
                ));
            }
        }
    }

    fn snapshot_json(&mut self) -> String {
        let snapshot = self.build_snapshot();
        let event = OutgoingEvent::StateSnapshot(snapshot);
        serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string())
    }

    fn build_snapshot(&mut self) -> StateSnapshot {
        let mut tiles = Vec::with_capacity((WORLD_SIZE * WORLD_SIZE) as usize);
        for y in 0..WORLD_SIZE {
            for x in 0..WORLD_SIZE {
                if let Some(terrain) = self.grid.terrain_at(x, y) {
                    tiles.push(TileSnapshot { x, y, terrain });
                }
            }
        }

        let buildings: Vec<BuildingSnapshot> = self
            .world
            .query::<(&Position, &BuildingType, Option<&BerrySupply>)>()
            .iter(&self.world)
            .map(|(pos, bt, supply)| BuildingSnapshot {
                x: pos.x as i32,
                y: pos.y as i32,
                building: *bt,
                berries: supply.map(|s| s.remaining),
            })
            .collect();

        let colonists: Vec<ColonistSnapshot> = self
            .world
            .query::<(&ColonistId, &ColonistName, &Position, &Needs, &Task)>()
            .iter(&self.world)
            .map(|(id, name, pos, needs, task)| ColonistSnapshot {
                id: id.0,
                name: name.0.clone(),
                x: pos.x,
                y: pos.y,
                food: needs.food,
                sleep: needs.sleep,
                task: task.kind,
            })
            .collect();

        let construction_sites: Vec<ConstructionSiteSnapshot> = self
            .world
            .query::<(&Position, &ConstructionSite)>()
            .iter(&self.world)
            .map(|(pos, site)| {
                let total = work_required_for(site.building_type);
                let progress = if total > 0.0 {
                    1.0 - (site.work_remaining / total).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                ConstructionSiteSnapshot {
                    x: pos.x as i32,
                    y: pos.y as i32,
                    building: site.building_type,
                    progress,
                }
            })
            .collect();

        StateSnapshot {
            tiles,
            buildings,
            construction_sites,
            colonists,
            paused: self.paused,
            speed: self.speed,
        }
    }
}

// Re-export for tests
impl Game {
    pub fn grid(&self) -> &WorldGrid {
        &self.grid
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}
