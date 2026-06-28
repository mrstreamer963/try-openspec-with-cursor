use bevy_ecs::prelude::*;
use serde_json;
use wasm_bindgen::prelude::*;

use crate::components::{BerrySupply, BuildingType, ColonistId, Needs, Position, Task};
use crate::events::{
    BuildingSnapshot, ColonistSnapshot, IncomingEvent, OutgoingEvent, StateSnapshot, TileSnapshot,
};
use crate::systems::{
    auto_assign_tasks, colonist_movement, needs_decay, spawn_colonists, task_execution,
};
use crate::world::{generate_world, WorldGrid, BERRIES_PER_BUSH, WORLD_SIZE};

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
            Err(e) => format!(r#"{{"error":"{}"}}"#, e),
        }
    }

    pub fn tick(&mut self, dt: f32) -> String {
        if self.paused || dt <= 0.0 {
            return self.snapshot_json();
        }

        let scaled_dt = dt * self.speed;
        self.world.insert_resource(self.grid.clone());

        needs_decay(&mut self.world, scaled_dt);
        auto_assign_tasks(&mut self.world, &self.grid);
        colonist_movement(&mut self.world, scaled_dt);
        task_execution(&mut self.world, &mut self.grid);

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
                if self.grid.place_building(x, y, building) {
                    let position = Position {
                        x: x as f32,
                        y: y as f32,
                    };
                    if building == BuildingType::BerryBush {
                        self.world.spawn((
                            crate::components::Building,
                            position,
                            building,
                            BerrySupply::new(BERRIES_PER_BUSH),
                        ));
                    } else {
                        self.world.spawn((
                            crate::components::Building,
                            position,
                            building,
                        ));
                    }
                }
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
            .query::<(&ColonistId, &Position, &Needs, &Task)>()
            .iter(&self.world)
            .map(|(id, pos, needs, task)| ColonistSnapshot {
                id: id.0,
                x: pos.x,
                y: pos.y,
                food: needs.food,
                sleep: needs.sleep,
                task: task.kind,
            })
            .collect();

        StateSnapshot {
            tiles,
            buildings,
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
