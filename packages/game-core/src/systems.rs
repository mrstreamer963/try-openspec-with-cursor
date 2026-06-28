use bevy_ecs::prelude::*;

use crate::components::{BuildingType, Colonist, ColonistId, NeedKind, Needs, Path, Position, Task, TaskKind};
use crate::pathfinding::find_path;
use crate::world::{
    WorldGrid, FOOD_DECAY_PER_SEC, MOVE_SPEED, NEED_RESTORE, NEED_THRESHOLD, SLEEP_DECAY_PER_SEC,
    WORLD_SIZE,
};

pub fn spawn_colonists(world: &mut World, grid: &WorldGrid) -> u32 {
    let mut next_id = 1u32;
    let mut spawned = 0;
    let center = WORLD_SIZE / 2;

    for y in (center - 10)..=(center + 10) {
        for x in (center - 10)..=(center + 10) {
            if spawned >= 3 {
                return next_id;
            }
            if grid.terrain_at(x, y) == Some(crate::components::TerrainType::Grass)
                && grid.is_walkable(x, y)
            {
                let _ = world
                    .spawn((
                        Colonist,
                        ColonistId(next_id),
                        Position {
                            x: x as f32,
                            y: y as f32,
                        },
                        Needs::new_full(),
                        Task::default(),
                        Path::default(),
                    ));
                next_id += 1;
                spawned += 1;
            }
        }
    }
    next_id
}

pub fn needs_decay(world: &mut World, dt: f32) {
    let mut query = world.query::<&mut Needs>();
    for mut needs in query.iter_mut(world) {
        needs.food = (needs.food - FOOD_DECAY_PER_SEC * dt).max(0.0);
        needs.sleep = (needs.sleep - SLEEP_DECAY_PER_SEC * dt).max(0.0);
    }
}

pub fn auto_assign_tasks(world: &mut World, grid: &WorldGrid) {
    let buildings: Vec<(i32, i32, BuildingType)> = {
        let mut q = world.query::<(&Position, &BuildingType)>();
        q.iter(world)
            .map(|(pos, bt)| {
                let (gx, gy) = pos.grid_cell();
                (gx, gy, *bt)
            })
            .collect()
    };

    let mut colonists = world.query::<(
        Entity,
        &Position,
        &Needs,
        &mut Task,
        &mut Path,
    )>();

    for (entity, pos, needs, mut task, mut path) in colonists.iter_mut(world) {
        if !matches!(task.kind, TaskKind::Idle) {
            continue;
        }

        let need = if needs.food < NEED_THRESHOLD {
            Some(NeedKind::Food)
        } else if needs.sleep < NEED_THRESHOLD {
            Some(NeedKind::Sleep)
        } else {
            None
        };

        let Some(need_kind) = need else {
            continue;
        };

        let (gx, gy) = pos.grid_cell();
        let target = nearest_building_for_need(gx, gy, need_kind, &buildings);
        let Some((tx, ty, _)) = target else {
            continue;
        };

        let task_kind = match need_kind {
            NeedKind::Food => TaskKind::Eat,
            NeedKind::Sleep => TaskKind::Sleep,
        };

        if let Some(waypoints) = find_path(grid, (gx, gy), (tx, ty)) {
            if waypoints.len() > 1 {
                path.waypoints = waypoints[1..].to_vec();
            } else {
                path.waypoints = vec![(tx, ty)];
            }
            path.index = 0;
            task.kind = task_kind;
            task.target_x = tx;
            task.target_y = ty;
        } else {
            path.clear();
            task.kind = TaskKind::Idle;
            let _ = entity;
        }
    }
}

fn nearest_building_for_need(
    x: i32,
    y: i32,
    need: NeedKind,
    buildings: &[(i32, i32, BuildingType)],
) -> Option<(i32, i32, BuildingType)> {
    buildings
        .iter()
        .filter(|(_, _, bt)| bt.satisfies_need() == Some(need))
        .min_by_key(|(bx, by, _)| (bx - x).abs() + (by - y).abs())
        .map(|(bx, by, bt)| (*bx, *by, *bt))
}

pub fn colonist_movement(world: &mut World, dt: f32) {
    let step = MOVE_SPEED * dt;
    let mut colonists = world.query::<(&mut Position, &mut Path)>();

    for (mut pos, mut path) in colonists.iter_mut(world) {
        if path.index >= path.waypoints.len() {
            continue;
        }

        let (tx, ty) = path.waypoints[path.index];
        let target_x = tx as f32;
        let target_y = ty as f32;
        let dx = target_x - pos.x;
        let dy = target_y - pos.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist <= step || dist < 0.001 {
            pos.x = target_x;
            pos.y = target_y;
            path.index += 1;
        } else {
            pos.x += (dx / dist) * step;
            pos.y += (dy / dist) * step;
        }
    }
}

pub fn task_execution(world: &mut World, grid: &WorldGrid) {
    let mut colonists = world.query::<(
        &Position,
        &mut Needs,
        &mut Task,
        &mut Path,
    )>();

    for (pos, mut needs, mut task, mut path) in colonists.iter_mut(world) {
        if matches!(task.kind, TaskKind::Idle) {
            continue;
        }
        if path.index < path.waypoints.len() {
            continue;
        }
        let (gx, gy) = pos.grid_cell();
        if gx != task.target_x || gy != task.target_y {
            continue;
        }

        let building = grid.building_at(gx, gy);
        let satisfied = match task.kind {
            TaskKind::Eat => building == Some(BuildingType::BerryBush),
            TaskKind::Sleep => building == Some(BuildingType::Bed),
            TaskKind::Idle => false,
        };

        if satisfied {
            match task.kind {
                TaskKind::Eat => needs.set(NeedKind::Food, NEED_RESTORE),
                TaskKind::Sleep => needs.set(NeedKind::Sleep, NEED_RESTORE),
                TaskKind::Idle => {}
            }
        }

        task.kind = TaskKind::Idle;
        task.target_x = 0;
        task.target_y = 0;
        path.clear();
    }
}
