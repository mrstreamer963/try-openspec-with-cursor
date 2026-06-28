use std::collections::HashSet;

use bevy_ecs::prelude::*;

use crate::components::{
    BedOccupancy, BerrySupply, BuildingType, Colonist, ColonistId, ColonistName,
    COLONIST_NAME_POOL, NeedKind, Needs, Path, Position, Task, TaskKind,
};
use crate::pathfinding::find_path;
use crate::world::{
    WorldGrid, FOOD_DECAY_PER_SEC, MOVE_SPEED, NEED_RESTORE, NEED_THRESHOLD, SLEEP_DECAY_PER_SEC,
    WORLD_SIZE,
};

fn shuffled_indices(len: usize, count: usize) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..len).collect();
    for i in 0..len {
        let range = len - i;
        let j = i + random_usize(range);
        indices.swap(i, j);
    }
    indices.truncate(count);
    indices
}

fn random_usize(upper: usize) -> usize {
    if upper <= 1 {
        return 0;
    }
    let mut buf = [0u8; 8];
    getrandom::fill(&mut buf).expect("failed to get random bytes");
    (u64::from_le_bytes(buf) % upper as u64) as usize
}

pub fn spawn_colonists(world: &mut World, grid: &WorldGrid) -> u32 {
    let mut next_id = 1u32;
    let mut spawned = 0;
    let center = WORLD_SIZE / 2;
    let mut names = shuffled_indices(COLONIST_NAME_POOL.len(), 3)
        .into_iter()
        .map(|i| COLONIST_NAME_POOL[i].to_string());

    for y in (center - 10)..=(center + 10) {
        for x in (center - 10)..=(center + 10) {
            if spawned >= 3 {
                return next_id;
            }
            if grid.terrain_at(x, y) == Some(crate::components::TerrainType::Grass)
                && grid.is_walkable(x, y)
            {
                let name = names.next().expect("name pool exhausted");
                let _ = world.spawn((
                    Colonist,
                    ColonistId(next_id),
                    ColonistName(name),
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

struct PendingAssignment {
    entity: Entity,
    task_kind: TaskKind,
    building_x: i32,
    building_y: i32,
    target_x: i32,
    target_y: i32,
    waypoints: Vec<(i32, i32)>,
    bed_entity: Option<Entity>,
}

pub fn auto_assign_tasks(world: &mut World, grid: &WorldGrid) {
    let berry_bushes: Vec<(i32, i32)> = {
        let mut q = world.query::<(&Position, &BuildingType, Option<&BerrySupply>)>();
        q.iter(world)
            .filter_map(|(pos, bt, supply)| {
                if *bt != BuildingType::BerryBush {
                    return None;
                }
                let (gx, gy) = pos.grid_cell();
                let available = supply.map(|s| s.remaining > 0).unwrap_or(false);
                if available {
                    Some((gx, gy))
                } else {
                    None
                }
            })
            .collect()
    };

    let free_beds: Vec<(Entity, i32, i32)> = {
        let mut q = world.query::<(Entity, &Position, &BuildingType, &BedOccupancy)>();
        q.iter(world)
            .filter_map(|(entity, pos, bt, occ)| {
                if *bt != BuildingType::Bed || occ.reserved_by.is_some() {
                    return None;
                }
                let (gx, gy) = pos.grid_cell();
                Some((entity, gx, gy))
            })
            .collect()
    };

    let mut reserved_beds: HashSet<Entity> = HashSet::new();
    let mut pending: Vec<PendingAssignment> = Vec::new();

    let mut colonists = world.query::<(Entity, &Position, &Needs, &Task)>();
    for (entity, pos, needs, task) in colonists.iter(world) {
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

        let assignment = match need_kind {
            NeedKind::Food => nearest_eat_assignment(grid, (gx, gy), &berry_bushes).map(
                |(bx, by)| (TaskKind::Eat, bx, by, bx, by, None),
            ),
            NeedKind::Sleep => nearest_free_bed((gx, gy), &free_beds, &reserved_beds).map(
                |(bed_entity, bx, by)| (TaskKind::Sleep, bx, by, bx, by, Some(bed_entity)),
            ),
        };

        let Some((task_kind, building_x, building_y, target_x, target_y, bed_entity)) =
            assignment
        else {
            continue;
        };

        if let Some(waypoints) = find_path(grid, (gx, gy), (target_x, target_y)) {
            let path_waypoints = if waypoints.len() > 1 {
                waypoints[1..].to_vec()
            } else {
                vec![(target_x, target_y)]
            };

            if let Some(bed_entity) = bed_entity {
                reserved_beds.insert(bed_entity);
            }

            pending.push(PendingAssignment {
                entity,
                task_kind,
                building_x,
                building_y,
                target_x,
                target_y,
                waypoints: path_waypoints,
                bed_entity,
            });
        }
    }

    for assignment in pending {
        if let Some(mut task) = world.get_mut::<Task>(assignment.entity) {
            task.kind = assignment.task_kind;
            task.building_x = assignment.building_x;
            task.building_y = assignment.building_y;
            task.target_x = assignment.target_x;
            task.target_y = assignment.target_y;
        }
        if let Some(mut path) = world.get_mut::<Path>(assignment.entity) {
            path.waypoints = assignment.waypoints;
            path.index = 0;
        }
        if let Some(bed_entity) = assignment.bed_entity {
            if let Some(mut occ) = world.get_mut::<BedOccupancy>(bed_entity) {
                occ.reserved_by = Some(assignment.entity);
            }
        }
    }
}

fn nearest_eat_assignment(
    grid: &WorldGrid,
    from: (i32, i32),
    bushes: &[(i32, i32)],
) -> Option<(i32, i32)> {
    let mut candidates: Vec<((i32, i32), i32)> = bushes
        .iter()
        .map(|&(bx, by)| ((bx, by), (bx - from.0).abs() + (by - from.1).abs()))
        .collect();
    candidates.sort_by_key(|(_, dist)| *dist);

    for ((bx, by), _) in candidates {
        if find_path(grid, from, (bx, by)).is_some() {
            return Some((bx, by));
        }
    }
    None
}

fn nearest_free_bed(
    from: (i32, i32),
    beds: &[(Entity, i32, i32)],
    reserved: &HashSet<Entity>,
) -> Option<(Entity, i32, i32)> {
    beds.iter()
        .filter(|(entity, _, _)| !reserved.contains(entity))
        .min_by_key(|(_, bx, by)| (bx - from.0).abs() + (by - from.1).abs())
        .map(|(entity, bx, by)| (*entity, *bx, *by))
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

pub fn task_execution(world: &mut World, grid: &mut WorldGrid) {
    let completions: Vec<(Entity, TaskKind, i32, i32, i32, i32)> = {
        let mut colonists = world.query::<(Entity, &Position, &Task, &Path)>();
        colonists
            .iter(world)
            .filter_map(|(entity, pos, task, path)| {
                if matches!(task.kind, TaskKind::Idle) {
                    return None;
                }
                if path.index < path.waypoints.len() {
                    return None;
                }
                let (gx, gy) = pos.grid_cell();
                if gx != task.target_x || gy != task.target_y {
                    return None;
                }
                Some((
                    entity,
                    task.kind,
                    task.building_x,
                    task.building_y,
                    gx,
                    gy,
                ))
            })
            .collect()
    };

    let mut to_despawn: Vec<(Entity, i32, i32)> = Vec::new();

    for (colonist_entity, kind, building_x, building_y, gx, gy) in completions {
        match kind {
            TaskKind::Eat => {
                if gx == building_x
                    && gy == building_y
                    && grid.building_at(gx, gy) == Some(BuildingType::BerryBush)
                {
                    let mut ate = false;
                    let mut depleted = false;
                    let mut building_entity = None;

                    if let Some(be) = building_entity_at(world, building_x, building_y) {
                        if let Some(mut supply) = world.get_mut::<BerrySupply>(be) {
                            if supply.remaining > 0 {
                                supply.remaining -= 1;
                                ate = true;
                                depleted = supply.remaining == 0;
                                building_entity = Some(be);
                            }
                        }
                    }

                    if ate {
                        if let Some(mut needs) = world.get_mut::<Needs>(colonist_entity) {
                            needs.set(NeedKind::Food, NEED_RESTORE);
                        }
                        if depleted {
                            if let Some(be) = building_entity {
                                to_despawn.push((be, building_x, building_y));
                            }
                        }
                    }
                }
            }
            TaskKind::Sleep => {
                if gx == building_x
                    && gy == building_y
                    && grid.building_at(gx, gy) == Some(BuildingType::Bed)
                {
                    let reserved = building_entity_at(world, building_x, building_y).and_then(
                        |be| world.get::<BedOccupancy>(be).and_then(|o| o.reserved_by),
                    );
                    if reserved == Some(colonist_entity) {
                        if let Some(mut needs) = world.get_mut::<Needs>(colonist_entity) {
                            needs.set(NeedKind::Sleep, NEED_RESTORE);
                        }
                    }
                }
            }
            TaskKind::Idle => {}
        }

        release_bed_reservation(world, colonist_entity);
        clear_task(world, colonist_entity);
    }

    for (entity, gx, gy) in to_despawn {
        grid.remove_building(gx, gy);
        let _ = world.despawn(entity);
    }
}

fn release_bed_reservation(world: &mut World, colonist: Entity) {
    let mut beds = world.query::<&mut BedOccupancy>();
    for mut occ in beds.iter_mut(world) {
        if occ.reserved_by == Some(colonist) {
            occ.reserved_by = None;
        }
    }
}

fn clear_task(world: &mut World, colonist: Entity) {
    if let Some(mut task) = world.get_mut::<Task>(colonist) {
        task.kind = TaskKind::Idle;
        task.building_x = 0;
        task.building_y = 0;
        task.target_x = 0;
        task.target_y = 0;
    }
    if let Some(mut path) = world.get_mut::<Path>(colonist) {
        path.clear();
    }
}

fn building_entity_at(world: &mut World, x: i32, y: i32) -> Option<Entity> {
    let mut q = world.query::<(Entity, &Position, &BuildingType)>();
    q.iter(world)
        .find(|(_, pos, _)| pos.grid_cell() == (x, y))
        .map(|(entity, _, _)| entity)
}
