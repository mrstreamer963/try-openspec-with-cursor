use bevy_ecs::prelude::*;
use serde_json;
use wasm_bindgen::prelude::*;

use crate::components::{
    BedOccupancy, BerrySupply, Building, BuildingType, Colonist, ColonistId, ColonistName,
    ConstructionSite, Hungry, Needs, Path, Position, Task, WantsSleep, work_required_for,
};
use crate::world::BERRIES_PER_BUSH;
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
                if let Some(err) = self.dispatch(event) {
                    return serde_json::to_string(&err).unwrap_or_else(|_| {
                        r#"{"type":"error","message":"unknown error"}"#.to_string()
                    });
                }
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
        colonist_movement(&mut self.world, &self.grid, dt);
        task_execution(&mut self.world, &mut self.grid, dt);

        self.snapshot_json()
    }

    pub fn get_snapshot(&mut self) -> String {
        self.snapshot_json()
    }
}

impl Game {
    fn dispatch(&mut self, event: IncomingEvent) -> Option<OutgoingEvent> {
        match event {
            IncomingEvent::SetPaused { paused } => {
                self.paused = paused;
                None
            }
            IncomingEvent::SetSpeed { multiplier } => {
                self.speed = multiplier.clamp(0.1, 10.0);
                None
            }
            IncomingEvent::Build { building, x, y } => {
                if !is_valid_build_tile(&mut self.world, &self.grid, x, y) {
                    return None;
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
                None
            }
            IncomingEvent::LoadState { state } => {
                if let Err(message) = self.restore_from_snapshot(state) {
                    Some(OutgoingEvent::Error { message })
                } else {
                    None
                }
            }
        }
    }

    fn restore_from_snapshot(&mut self, snapshot: StateSnapshot) -> Result<(), String> {
        let expected_tiles = (WORLD_SIZE * WORLD_SIZE) as usize;
        if snapshot.tiles.len() != expected_tiles {
            return Err(format!(
                "invalid tile count: expected {}, got {}",
                expected_tiles,
                snapshot.tiles.len()
            ));
        }

        self.despawn_game_entities();

        let len = expected_tiles;
        let mut terrain = vec![crate::components::TerrainType::Grass; len];
        let mut buildings = vec![None; len];

        for tile in &snapshot.tiles {
            if let Some(i) = WorldGrid::index(tile.x, tile.y) {
                terrain[i] = tile.terrain;
            }
        }

        for building in &snapshot.buildings {
            if let Some(i) = WorldGrid::index(building.x, building.y) {
                buildings[i] = Some(building.building);
            }
        }

        self.grid = WorldGrid {
            terrain,
            buildings,
            seed: self.grid.seed,
        };
        self.world.insert_resource(self.grid.clone());

        for building in &snapshot.buildings {
            let position = Position {
                x: building.x as f32,
                y: building.y as f32,
            };
            if building.building == BuildingType::BerryBush {
                let berries = building.berries.unwrap_or(BERRIES_PER_BUSH);
                self.world.spawn((
                    Building,
                    position,
                    building.building,
                    BerrySupply::new(berries),
                ));
            } else if building.building == BuildingType::Bed {
                self.world.spawn((
                    Building,
                    position,
                    building.building,
                    BedOccupancy::default(),
                ));
            } else {
                self.world.spawn((Building, position, building.building));
            }
        }

        for site in &snapshot.construction_sites {
            let total = work_required_for(site.building);
            let work_remaining = total * (1.0 - site.progress.clamp(0.0, 1.0));
            self.world.spawn((
                ConstructionSite {
                    building_type: site.building,
                    work_remaining,
                    reserved_by: None,
                },
                Position {
                    x: site.x as f32,
                    y: site.y as f32,
                },
            ));
        }

        for colonist in &snapshot.colonists {
            let mut entity = self.world.spawn((
                Colonist,
                ColonistId(colonist.id),
                ColonistName(colonist.name.clone()),
                Position {
                    x: colonist.x,
                    y: colonist.y,
                },
                Needs {
                    food: colonist.food,
                    sleep: colonist.sleep,
                },
                Task {
                    kind: colonist.task,
                    ..Task::default()
                },
                Path::default(),
            ));
            if colonist.hungry {
                entity.insert(Hungry);
            }
            if colonist.wants_sleep {
                entity.insert(WantsSleep);
            }
        }

        self.paused = snapshot.paused;
        self.speed = snapshot.speed.clamp(0.1, 10.0);
        Ok(())
    }

    fn despawn_game_entities(&mut self) {
        let colonists: Vec<Entity> = {
            let mut q = self.world.query::<(Entity, &Colonist)>();
            q.iter(&self.world).map(|(e, _)| e).collect()
        };
        let buildings: Vec<Entity> = {
            let mut q = self.world.query::<(Entity, &Building)>();
            q.iter(&self.world).map(|(e, _)| e).collect()
        };
        let sites: Vec<Entity> = {
            let mut q = self.world.query::<(Entity, &ConstructionSite)>();
            q.iter(&self.world).map(|(e, _)| e).collect()
        };

        for entity in colonists
            .into_iter()
            .chain(buildings)
            .chain(sites)
        {
            let _ = self.world.despawn(entity);
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
            .query::<(
                &ColonistId,
                &ColonistName,
                &Position,
                &Needs,
                &Task,
                Option<&Hungry>,
                Option<&WantsSleep>,
            )>()
            .iter(&self.world)
            .map(|(id, name, pos, needs, task, hungry, wants_sleep)| ColonistSnapshot {
                id: id.0,
                name: name.0.clone(),
                x: pos.x,
                y: pos.y,
                food: needs.food,
                sleep: needs.sleep,
                hungry: hungry.is_some(),
                wants_sleep: wants_sleep.is_some(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Needs;
    use crate::systems::update_need_buffs;
    use crate::world::NEED_THRESHOLD;

    #[test]
    fn snapshot_flags_match_need_buffs_below_threshold() {
        let mut game = Game::new();

        {
            let mut q = game.world.query::<&mut Needs>();
            for mut needs in q.iter_mut(&mut game.world) {
                needs.food = NEED_THRESHOLD - 1.0;
                needs.sleep = NEED_THRESHOLD - 1.0;
            }
        }

        update_need_buffs(&mut game.world);

        let json = game.get_snapshot();
        let event: OutgoingEvent = serde_json::from_str(&json).unwrap();
        let OutgoingEvent::StateSnapshot(snapshot) = event else {
            panic!("expected state snapshot");
        };

        for colonist in &snapshot.colonists {
            assert!(colonist.hungry, "colonist {} should be hungry", colonist.id);
            assert!(
                colonist.wants_sleep,
                "colonist {} should want sleep",
                colonist.id
            );
        }

        assert!(json.contains("\"hungry\":true"));
        assert!(json.contains("\"wants_sleep\":true"));
    }

    #[test]
    fn snapshot_flags_false_when_needs_satisfied() {
        let mut game = Game::new();

        {
            let mut q = game.world.query::<&mut Needs>();
            for mut needs in q.iter_mut(&mut game.world) {
                needs.food = NEED_THRESHOLD;
                needs.sleep = NEED_THRESHOLD;
            }
        }

        update_need_buffs(&mut game.world);

        let json = game.get_snapshot();
        let event: OutgoingEvent = serde_json::from_str(&json).unwrap();
        let OutgoingEvent::StateSnapshot(snapshot) = event else {
            panic!("expected state snapshot");
        };

        for colonist in &snapshot.colonists {
            assert!(!colonist.hungry, "colonist {} should not be hungry", colonist.id);
            assert!(
                !colonist.wants_sleep,
                "colonist {} should not want sleep",
                colonist.id
            );
        }
    }

    #[test]
    fn load_state_round_trip_preserves_key_fields() {
        let mut game = Game::new();
        game.handle_event(r#"{"type":"set_paused","paused":true}"#);
        game.handle_event(r#"{"type":"set_speed","multiplier":3}"#);

        let json = game.get_snapshot();
        let event: OutgoingEvent = serde_json::from_str(&json).unwrap();
        let OutgoingEvent::StateSnapshot(original) = event else {
            panic!("expected state snapshot");
        };

        let load_json =
            serde_json::to_string(&IncomingEvent::LoadState {
                state: original.clone(),
            })
            .unwrap();
        let err = game.handle_event(&load_json);
        assert!(err.is_empty(), "load should succeed: {err}");

        let json2 = game.get_snapshot();
        let event2: OutgoingEvent = serde_json::from_str(&json2).unwrap();
        let OutgoingEvent::StateSnapshot(restored) = event2 else {
            panic!("expected state snapshot");
        };

        assert_eq!(restored.paused, original.paused);
        assert_eq!(restored.speed, original.speed);
        assert_eq!(restored.tiles.len(), original.tiles.len());
        assert_eq!(restored.buildings.len(), original.buildings.len());
        assert_eq!(restored.construction_sites.len(), original.construction_sites.len());
        assert_eq!(restored.colonists.len(), original.colonists.len());

        for (a, b) in restored.colonists.iter().zip(original.colonists.iter()) {
            assert_eq!(a.id, b.id);
            assert_eq!(a.name, b.name);
            assert_eq!(a.x, b.x);
            assert_eq!(a.y, b.y);
            assert_eq!(a.food, b.food);
            assert_eq!(a.sleep, b.sleep);
            assert_eq!(a.hungry, b.hungry);
            assert_eq!(a.wants_sleep, b.wants_sleep);
            assert_eq!(a.task, b.task);
        }
    }

    #[test]
    fn load_state_rejects_invalid_tile_count() {
        let mut game = Game::new();

        let json = game.get_snapshot();
        let event: OutgoingEvent = serde_json::from_str(&json).unwrap();
        let OutgoingEvent::StateSnapshot(mut snapshot) = event else {
            panic!("expected state snapshot");
        };
        snapshot.tiles.truncate(100);

        let load_json =
            serde_json::to_string(&IncomingEvent::LoadState { state: snapshot }).unwrap();
        let err = game.handle_event(&load_json);
        assert!(err.contains("error"), "expected error response: {err}");
        assert!(err.contains("invalid tile count"));

        let json2 = game.get_snapshot();
        let event2: OutgoingEvent = serde_json::from_str(&json2).unwrap();
        let OutgoingEvent::StateSnapshot(unchanged) = event2 else {
            panic!("expected state snapshot");
        };
        assert_eq!(unchanged.tiles.len(), 2500);
        assert!(!unchanged.paused);
    }
}
