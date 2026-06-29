use bevy_ecs::prelude::*;
use serde_json;
use wasm_bindgen::prelude::*;

use crate::components::{
    ActiveStatuses, BedOccupancy, BerrySupply, Building, BuildingKind, Colonist, ColonistId,
    ColonistName, ConstructionSite, DeconstructionSite, Needs, Path, Position, Task, TaskKind,
};
use crate::content::ContentRegistry;
use crate::events::{
    BuildingSnapshot, ColonistSnapshot, ConstructionSiteSnapshot, DeconstructionSiteSnapshot,
    IncomingEvent, OutgoingEvent, StateSnapshot, TileSnapshot,
};
use crate::systems::{
    auto_assign_tasks, colonist_at_task_stand, colonist_movement, construction_site_at,
    deconstruction_site_at, is_valid_build_tile, needs_decay, spawn_colonists, sync_statuses,
    task_execution,
};
use crate::world::{generate_world, WorldGrid, WORLD_SIZE};

#[wasm_bindgen]
pub struct Game {
    world: World,
    grid: WorldGrid,
    content: ContentRegistry,
    paused: bool,
    speed: f32,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new(content_json: &str) -> Result<Game, JsValue> {
        let content = ContentRegistry::from_json(content_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let grid = generate_world(42, &content);
        let mut world = World::new();
        world.insert_resource(content.clone());
        world.insert_resource(grid.clone());
        let _next_id = spawn_colonists(&mut world, &grid, &content);

        Ok(Game {
            world,
            grid,
            content,
            paused: false,
            speed: 1.0,
        })
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
        self.world.insert_resource(self.content.clone());

        needs_decay(&mut self.world, &self.content, dt);
        sync_statuses(&mut self.world, &self.content);
        auto_assign_tasks(&mut self.world, &self.grid, &self.content);
        colonist_movement(&mut self.world, &self.grid, &self.content, dt);
        task_execution(&mut self.world, &mut self.grid, &self.content, dt);

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
                let building_id = self.content.building_id(&building)?;
                if !is_valid_build_tile(&mut self.world, &self.grid, &self.content, x, y) {
                    return None;
                }
                self.world.spawn((
                    ConstructionSite {
                        building_id,
                        work_remaining: self.content.work_required(building_id),
                        reserved_by: None,
                    },
                    Position {
                        x: x as f32,
                        y: y as f32,
                    },
                ));
                None
            }
            IncomingEvent::Deconstruct { x, y } => {
                self.handle_deconstruct(x, y);
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
        let mut terrain = vec![self.content.grass_terrain; len];
        let mut buildings = vec![None; len];

        for tile in &snapshot.tiles {
            if let Some(terrain_id) = self.content.terrain_id(&tile.terrain) {
                if let Some(i) = WorldGrid::index(tile.x, tile.y) {
                    terrain[i] = terrain_id;
                }
            }
        }

        for building in &snapshot.buildings {
            if let Some(building_id) = self.content.building_id(&building.building) {
                if let Some(i) = WorldGrid::index(building.x, building.y) {
                    buildings[i] = Some(building_id);
                }
            }
        }

        self.grid = WorldGrid {
            terrain,
            buildings,
            seed: self.grid.seed,
        };
        self.world.insert_resource(self.grid.clone());

        for building in &snapshot.buildings {
            let Some(building_id) = self.content.building_id(&building.building) else {
                continue;
            };
            let position = Position {
                x: building.x as f32,
                y: building.y as f32,
            };
            if let Some(amount) = self.content.supply_amount_on_complete(building_id) {
                let berries = building.berries.unwrap_or(amount);
                self.world.spawn((
                    Building,
                    position,
                    BuildingKind(building_id),
                    BerrySupply::new(berries),
                ));
            } else if self.content.has_reservation_on_complete(building_id) {
                self.world.spawn((
                    Building,
                    position,
                    BuildingKind(building_id),
                    BedOccupancy::default(),
                ));
            } else {
                self.world.spawn((Building, position, BuildingKind(building_id)));
            }
        }

        for site in &snapshot.construction_sites {
            let Some(building_id) = self.content.building_id(&site.building) else {
                continue;
            };
            let total = self.content.work_required(building_id);
            let work_remaining = total * (1.0 - site.progress.clamp(0.0, 1.0));
            self.world.spawn((
                ConstructionSite {
                    building_id,
                    work_remaining,
                    reserved_by: None,
                },
                Position {
                    x: site.x as f32,
                    y: site.y as f32,
                },
            ));
        }

        for site in &snapshot.deconstruction_sites {
            let Some(building_id) = self.content.building_id(&site.building) else {
                continue;
            };
            let total = self.content.work_to_deconstruct(building_id);
            let work_remaining = total * (1.0 - site.progress.clamp(0.0, 1.0));
            self.world.spawn((
                DeconstructionSite {
                    building_id,
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
            let mut needs = Needs::new_full(&self.content);
            needs.set(self.content.food_need, colonist.food);
            needs.set(self.content.sleep_need, colonist.sleep);

            let mut entity = self.world.spawn((
                Colonist,
                ColonistId(colonist.id),
                ColonistName(colonist.name.clone()),
                Position {
                    x: colonist.x,
                    y: colonist.y,
                },
                needs,
                Task {
                    kind: colonist.task,
                    ..Task::default()
                },
                Path::default(),
                ActiveStatuses::default(),
            ));

            if colonist.hungry || colonist.wants_sleep {
                let mut statuses = ActiveStatuses::default();
                if colonist.hungry {
                    statuses.0.insert(self.content.hungry_status);
                }
                if colonist.wants_sleep {
                    statuses.0.insert(self.content.wants_sleep_status);
                }
                entity.insert(statuses);
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

        let decon_sites: Vec<Entity> = {
            let mut q = self.world.query::<(Entity, &DeconstructionSite)>();
            q.iter(&self.world).map(|(e, _)| e).collect()
        };

        for entity in colonists
            .into_iter()
            .chain(buildings)
            .chain(sites)
            .chain(decon_sites)
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
                    tiles.push(TileSnapshot {
                        x,
                        y,
                        terrain: self.content.terrain_str(terrain).to_string(),
                    });
                }
            }
        }

        let buildings: Vec<BuildingSnapshot> = self
            .world
            .query::<(&Position, &BuildingKind, Option<&BerrySupply>)>()
            .iter(&self.world)
            .map(|(pos, kind, supply)| BuildingSnapshot {
                x: pos.x as i32,
                y: pos.y as i32,
                building: self.content.building_str(kind.0).to_string(),
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
                &Path,
                &ActiveStatuses,
            )>()
            .iter(&self.world)
            .map(|(id, name, pos, needs, task, path, statuses)| ColonistSnapshot {
                id: id.0,
                name: name.0.clone(),
                x: pos.x,
                y: pos.y,
                food: needs.get(self.content.food_need),
                sleep: needs.get(self.content.sleep_need),
                hungry: statuses.has(self.content.hungry_status),
                wants_sleep: statuses.has(self.content.wants_sleep_status),
                task: task.kind,
                at_task_stand: colonist_at_task_stand(pos, task, path),
            })
            .collect();

        let construction_sites: Vec<ConstructionSiteSnapshot> = self
            .world
            .query::<(&Position, &ConstructionSite)>()
            .iter(&self.world)
            .map(|(pos, site)| {
                let total = self.content.work_required(site.building_id);
                let progress = if total > 0.0 {
                    1.0 - (site.work_remaining / total).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                ConstructionSiteSnapshot {
                    x: pos.x as i32,
                    y: pos.y as i32,
                    building: self.content.building_str(site.building_id).to_string(),
                    progress,
                }
            })
            .collect();

        let deconstruction_sites: Vec<DeconstructionSiteSnapshot> = self
            .world
            .query::<(&Position, &DeconstructionSite)>()
            .iter(&self.world)
            .map(|(pos, site)| {
                let total = self.content.work_to_deconstruct(site.building_id);
                let progress = if total > 0.0 {
                    1.0 - (site.work_remaining / total).clamp(0.0, 1.0)
                } else {
                    1.0
                };
                DeconstructionSiteSnapshot {
                    x: pos.x as i32,
                    y: pos.y as i32,
                    building: self.content.building_str(site.building_id).to_string(),
                    progress,
                }
            })
            .collect();

        StateSnapshot {
            tiles,
            buildings,
            construction_sites,
            deconstruction_sites,
            colonists,
            paused: self.paused,
            speed: self.speed,
        }
    }
}

impl Game {
    fn handle_deconstruct(&mut self, x: i32, y: i32) {
        if deconstruction_site_at(&mut self.world, x, y).is_some() {
            return;
        }

        if let Some(site_entity) = construction_site_at(&mut self.world, x, y) {
            let site = match self.world.get::<ConstructionSite>(site_entity) {
                Some(s) => *s,
                None => return,
            };
            let total = self.content.work_required(site.building_id);
            if site.work_remaining >= total {
                if let Some(builder) = site.reserved_by {
                    if let Some(mut task) = self.world.get_mut::<Task>(builder) {
                        task.kind = TaskKind::Idle;
                        task.building_x = 0;
                        task.building_y = 0;
                        task.target_x = 0;
                        task.target_y = 0;
                    }
                    if let Some(mut path) = self.world.get_mut::<Path>(builder) {
                        path.clear();
                    }
                }
                let _ = self.world.despawn(site_entity);
                return;
            }

            if let Some(builder) = site.reserved_by {
                if let Some(mut task) = self.world.get_mut::<Task>(builder) {
                    task.kind = TaskKind::Idle;
                    task.building_x = 0;
                    task.building_y = 0;
                    task.target_x = 0;
                    task.target_y = 0;
                }
                if let Some(mut path) = self.world.get_mut::<Path>(builder) {
                    path.clear();
                }
            }
            let building_id = site.building_id;
            let _ = self.world.despawn(site_entity);
            self.world.spawn((
                DeconstructionSite {
                    building_id,
                    work_remaining: self.content.work_to_deconstruct(building_id),
                    reserved_by: None,
                },
                Position {
                    x: x as f32,
                    y: y as f32,
                },
            ));
            return;
        }

        let Some(building_id) = self.grid.building_at(x, y) else {
            return;
        };

        self.world.spawn((
            DeconstructionSite {
                building_id,
                work_remaining: self.content.work_to_deconstruct(building_id),
                reserved_by: None,
            },
            Position {
                x: x as f32,
                y: y as f32,
            },
        ));
    }

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
    use crate::content::base_content_json;
    use crate::systems::sync_statuses;

    fn test_game() -> Game {
        Game::new(base_content_json()).expect("base content")
    }

    #[test]
    fn snapshot_flags_match_need_buffs_below_threshold() {
        let mut game = test_game();
        let threshold = game
            .content
            .need_def(game.content.food_need)
            .critical_threshold;

        {
            let mut q = game.world.query::<&mut Needs>();
            for mut needs in q.iter_mut(&mut game.world) {
                needs.set(game.content.food_need, threshold - 1.0);
                needs.set(game.content.sleep_need, threshold - 1.0);
            }
        }

        sync_statuses(&mut game.world, &game.content);

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
        let mut game = test_game();
        let threshold = game
            .content
            .need_def(game.content.food_need)
            .critical_threshold;

        {
            let mut q = game.world.query::<&mut Needs>();
            for mut needs in q.iter_mut(&mut game.world) {
                needs.set(game.content.food_need, threshold);
                needs.set(game.content.sleep_need, threshold);
            }
        }

        sync_statuses(&mut game.world, &game.content);

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
        let mut game = test_game();
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
        assert_eq!(
            restored.deconstruction_sites.len(),
            original.deconstruction_sites.len()
        );
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
        let mut game = test_game();

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

        let json2 = game.get_snapshot();
        let event2: OutgoingEvent = serde_json::from_str(&json2).unwrap();
        let OutgoingEvent::StateSnapshot(unchanged) = event2 else {
            panic!("expected state snapshot");
        };
        assert_eq!(unchanged.tiles.len(), 2500);
        assert!(!unchanged.paused);
    }

    #[test]
    fn deconstruct_event_instant_cancel_at_zero_progress() {
        let mut game = test_game();
        game.handle_event(r#"{"type":"build","building":"wall","x":10,"y":10}"#);
        game.handle_event(r#"{"type":"deconstruct","x":10,"y":10}"#);

        let json = game.get_snapshot();
        let event: OutgoingEvent = serde_json::from_str(&json).unwrap();
        let OutgoingEvent::StateSnapshot(snapshot) = event else {
            panic!("expected state snapshot");
        };

        assert!(snapshot.construction_sites.is_empty());
        assert!(snapshot.deconstruction_sites.is_empty());
    }

    #[test]
    fn deconstruction_sites_survive_save_load_round_trip() {
        let mut game = test_game();

        let json = game.get_snapshot();
        let event: OutgoingEvent = serde_json::from_str(&json).unwrap();
        let OutgoingEvent::StateSnapshot(mut original) = event else {
            panic!("expected state snapshot");
        };

        original.buildings.push(BuildingSnapshot {
            x: 10,
            y: 10,
            building: "wall".to_string(),
            berries: None,
        });
        original.deconstruction_sites.push(DeconstructionSiteSnapshot {
            x: 10,
            y: 10,
            building: "wall".to_string(),
            progress: 0.5,
        });

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

        assert_eq!(restored.deconstruction_sites.len(), 1);
        assert_eq!(restored.deconstruction_sites[0].x, 10);
        assert_eq!(restored.deconstruction_sites[0].y, 10);
        assert!((restored.deconstruction_sites[0].progress - 0.5).abs() < 0.01);
    }
}
