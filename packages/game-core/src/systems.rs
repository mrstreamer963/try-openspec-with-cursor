use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;

use crate::components::{
    BedOccupancy, BerrySupply, BuildingType, Colonist, ColonistId, ColonistName,
    ConstructionSite, COLONIST_NAME_POOL, NeedKind, Needs, Path, Position, Task, TaskKind,
    BUILD_WORK_PER_TICK,
};
use crate::pathfinding::{best_adjacent_stand, find_path};
use crate::world::{
    WorldGrid, BERRIES_PER_BUSH, FOOD_DECAY_PER_SEC, MOVE_SPEED, NEED_RESTORE, NEED_THRESHOLD,
    SLEEP_DECAY_PER_SEC, WANDER_MIN_RADIUS, WANDER_PICK_ATTEMPTS, WANDER_RADIUS, WORLD_SIZE,
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

fn colonist_occupancy_map(world: &mut World) -> HashMap<(i32, i32), Entity> {
    let mut map = HashMap::new();
    let mut q = world.query::<(Entity, &Position, &Colonist)>();
    for (entity, pos, _) in q.iter(world) {
        map.insert(pos.grid_cell(), entity);
    }
    map
}

fn is_cell_free(
    map: &HashMap<(i32, i32), Entity>,
    cell: (i32, i32),
    self_entity: Entity,
) -> bool {
    match map.get(&cell) {
        None => true,
        Some(&occupant) => occupant == self_entity,
    }
}

fn stand_available_for_eat(
    stand: (i32, i32),
    occupancy: &HashMap<(i32, i32), Entity>,
    reserved_stands: &HashSet<(i32, i32)>,
) -> bool {
    !reserved_stands.contains(&stand) && !occupancy.contains_key(&stand)
}

fn pick_wander_target(
    grid: &WorldGrid,
    from: (i32, i32),
    occupied: &HashMap<(i32, i32), Entity>,
    self_entity: Entity,
) -> Option<Vec<(i32, i32)>> {
    let mut candidates = Vec::new();
    for dy in -WANDER_RADIUS..=WANDER_RADIUS {
        for dx in -WANDER_RADIUS..=WANDER_RADIUS {
            let manhattan = dx.abs() + dy.abs();
            if manhattan < WANDER_MIN_RADIUS || manhattan > WANDER_RADIUS {
                continue;
            }
            let cell = (from.0 + dx, from.1 + dy);
            if !grid.is_walkable(cell.0, cell.1) {
                continue;
            }
            if let Some(&occupant) = occupied.get(&cell) {
                if occupant != self_entity {
                    continue;
                }
            }
            candidates.push(cell);
        }
    }

    if candidates.is_empty() {
        return None;
    }

    let attempts = WANDER_PICK_ATTEMPTS.min(candidates.len());
    for i in shuffled_indices(candidates.len(), attempts) {
        let target = candidates[i];
        if let Some(waypoints) = find_path(grid, from, target) {
            return Some(if waypoints.len() > 1 {
                waypoints[1..].to_vec()
            } else {
                vec![target]
            });
        }
    }
    None
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

pub fn construction_site_at(world: &mut World, x: i32, y: i32) -> Option<Entity> {
    let mut q = world.query::<(Entity, &Position, &ConstructionSite)>();
    q.iter(world)
        .find(|(_, pos, _)| pos.grid_cell() == (x, y))
        .map(|(entity, _, _)| entity)
}

pub fn is_valid_build_tile(world: &mut World, grid: &WorldGrid, x: i32, y: i32) -> bool {
    if !grid
        .terrain_at(x, y)
        .map(|t| t.walkable())
        .unwrap_or(false)
    {
        return false;
    }
    if grid.building_at(x, y).is_some() {
        return false;
    }
    if construction_site_at(world, x, y).is_some() {
        return false;
    }
    true
}

pub fn complete_construction(
    world: &mut World,
    grid: &mut WorldGrid,
    site_entity: Entity,
    site: &ConstructionSite,
    x: i32,
    y: i32,
) {
    let building = site.building_type;
    if !grid.place_building(x, y, building) {
        return;
    }
    let position = Position {
        x: x as f32,
        y: y as f32,
    };
    if building == BuildingType::BerryBush {
        world.spawn((
            crate::components::Building,
            position,
            building,
            BerrySupply::new(BERRIES_PER_BUSH),
        ));
    } else if building == BuildingType::Bed {
        world.spawn((
            crate::components::Building,
            position,
            building,
            BedOccupancy::default(),
        ));
    } else {
        world.spawn((crate::components::Building, position, building));
    }
    let _ = world.despawn(site_entity);
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
    site_entity: Option<Entity>,
}

pub fn auto_assign_tasks(world: &mut World, grid: &WorldGrid) {
    preempt_build_for_critical_needs(world);
    release_stuck_tasks(world);

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

    let open_sites: Vec<(Entity, i32, i32)> = {
        let mut q = world.query::<(Entity, &Position, &ConstructionSite)>();
        q.iter(world)
            .filter_map(|(entity, pos, site)| {
                if site.reserved_by.is_some() {
                    return None;
                }
                let (gx, gy) = pos.grid_cell();
                Some((entity, gx, gy))
            })
            .collect()
    };

    let mut reserved_beds: HashSet<Entity> = HashSet::new();
    let mut reserved_sites: HashSet<Entity> = HashSet::new();
    let mut reserved_stands: HashSet<(i32, i32)> = HashSet::new();
    let occupancy = colonist_occupancy_map(world);
    let mut pending: Vec<PendingAssignment> = Vec::new();
    let mut pending_wander: Vec<(Entity, Vec<(i32, i32)>)> = Vec::new();

    let mut colonists = world.query::<(Entity, &Position, &Needs, &Task, &Path)>();
    for (entity, pos, needs, task, path) in colonists.iter(world) {
        if !matches!(task.kind, TaskKind::Idle) {
            continue;
        }

        let (gx, gy) = pos.grid_cell();

        let need = if needs.food < NEED_THRESHOLD {
            Some(NeedKind::Food)
        } else if needs.sleep < NEED_THRESHOLD {
            Some(NeedKind::Sleep)
        } else {
            None
        };

        if let Some(need_kind) = need {
            let assignment = match need_kind {
                NeedKind::Food => nearest_eat_assignment(
                    grid,
                    (gx, gy),
                    &berry_bushes,
                    &occupancy,
                    &reserved_stands,
                )
                    .map(|((bx, by), (sx, sy))| PendingAssignment {
                        entity,
                        task_kind: TaskKind::Eat,
                        building_x: bx,
                        building_y: by,
                        target_x: sx,
                        target_y: sy,
                        waypoints: Vec::new(),
                        bed_entity: None,
                        site_entity: None,
                    }),
                NeedKind::Sleep => nearest_free_bed((gx, gy), &free_beds, &reserved_beds).map(
                    |(bed_entity, bx, by)| PendingAssignment {
                        entity,
                        task_kind: TaskKind::Sleep,
                        building_x: bx,
                        building_y: by,
                        target_x: bx,
                        target_y: by,
                        waypoints: Vec::new(),
                        bed_entity: Some(bed_entity),
                        site_entity: None,
                    },
                ),
            };

            let Some(mut assignment) = assignment else {
                continue;
            };

            if let Some(waypoints) = find_path(grid, (gx, gy), (assignment.target_x, assignment.target_y)) {
                assignment.waypoints = if waypoints.len() > 1 {
                    waypoints[1..].to_vec()
                } else {
                    vec![(assignment.target_x, assignment.target_y)]
                };

                if let Some(bed_entity) = assignment.bed_entity {
                    reserved_beds.insert(bed_entity);
                }
                if assignment.task_kind == TaskKind::Eat {
                    reserved_stands.insert((assignment.target_x, assignment.target_y));
                }

                pending.push(assignment);
            }
            continue;
        }

        if let Some((site_entity, (bx, by), (sx, sy))) =
            nearest_build_assignment(grid, (gx, gy), &open_sites, &reserved_sites)
        {
            if let Some(waypoints) = find_path(grid, (gx, gy), (sx, sy)) {
                let path_waypoints = if waypoints.len() > 1 {
                    waypoints[1..].to_vec()
                } else {
                    vec![(sx, sy)]
                };

                reserved_sites.insert(site_entity);

                pending.push(PendingAssignment {
                    entity,
                    task_kind: TaskKind::Build,
                    building_x: bx,
                    building_y: by,
                    target_x: sx,
                    target_y: sy,
                    waypoints: path_waypoints,
                    bed_entity: None,
                    site_entity: Some(site_entity),
                });
            }
            continue;
        }

        if path.index >= path.waypoints.len() {
            if let Some(waypoints) = pick_wander_target(grid, (gx, gy), &occupancy, entity) {
                pending_wander.push((entity, waypoints));
            }
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
        if let Some(site_entity) = assignment.site_entity {
            if let Some(mut site) = world.get_mut::<ConstructionSite>(site_entity) {
                site.reserved_by = Some(assignment.entity);
            }
        }
    }

    for (entity, waypoints) in pending_wander {
        if let Some(mut path) = world.get_mut::<Path>(entity) {
            path.waypoints = waypoints;
            path.index = 0;
        }
    }
}

fn preempt_build_for_critical_needs(world: &mut World) {
    let preemptions: Vec<Entity> = {
        let mut q = world.query::<(Entity, &Needs, &Task)>();
        q.iter(world)
            .filter_map(|(entity, needs, task)| {
                if !matches!(task.kind, TaskKind::Build) {
                    return None;
                }
                if needs.food < NEED_THRESHOLD || needs.sleep < NEED_THRESHOLD {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect()
    };

    for entity in preemptions {
        release_construction_reservation(world, entity);
        clear_task(world, entity);
    }
}

fn release_stuck_tasks(world: &mut World) {
    let stuck: Vec<(Entity, TaskKind)> = {
        let mut q = world.query::<(Entity, &Position, &Task, &Path)>();
        q.iter(world)
            .filter_map(|(entity, pos, task, path)| {
                if !matches!(task.kind, TaskKind::Build | TaskKind::Eat | TaskKind::Sleep) {
                    return None;
                }
                let (gx, gy) = pos.grid_cell();
                if gx == task.target_x && gy == task.target_y {
                    return None;
                }
                if path.index < path.waypoints.len() {
                    return None;
                }
                Some((entity, task.kind))
            })
            .collect()
    };

    for (entity, kind) in stuck {
        if matches!(kind, TaskKind::Build) {
            release_construction_reservation(world, entity);
        }
        clear_task(world, entity);
    }
}

fn nearest_build_assignment(
    grid: &WorldGrid,
    from: (i32, i32),
    sites: &[(Entity, i32, i32)],
    reserved: &HashSet<Entity>,
) -> Option<(Entity, (i32, i32), (i32, i32))> {
    let mut candidates: Vec<((Entity, i32, i32), i32)> = sites
        .iter()
        .filter(|(entity, _, _)| !reserved.contains(entity))
        .map(|&(entity, sx, sy)| ((entity, sx, sy), (sx - from.0).abs() + (sy - from.1).abs()))
        .collect();
    candidates.sort_by_key(|(_, dist)| *dist);

    for ((entity, sx, sy), _) in candidates {
        if let Some(stand) = best_adjacent_stand(grid, (sx, sy), from) {
            return Some((entity, (sx, sy), stand));
        }
    }
    None
}

fn nearest_eat_assignment(
    grid: &WorldGrid,
    from: (i32, i32),
    bushes: &[(i32, i32)],
    occupancy: &HashMap<(i32, i32), Entity>,
    reserved_stands: &HashSet<(i32, i32)>,
) -> Option<((i32, i32), (i32, i32))> {
    let mut candidates: Vec<((i32, i32), i32)> = bushes
        .iter()
        .map(|&(bx, by)| ((bx, by), (bx - from.0).abs() + (by - from.1).abs()))
        .collect();
    candidates.sort_by_key(|(_, dist)| *dist);

    for ((bx, by), _) in candidates {
        if let Some(stand) = crate::pathfinding::best_adjacent_stand_filtered(
            grid,
            (bx, by),
            from,
            |stand| stand_available_for_eat(stand, occupancy, reserved_stands),
        ) {
            return Some(((bx, by), stand));
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

struct SnapIntent {
    entity: Entity,
    target: (i32, i32),
    target_x: f32,
    target_y: f32,
}

pub fn colonist_movement(world: &mut World, dt: f32) {
    let step = MOVE_SPEED * dt;
    let mut occupancy = colonist_occupancy_map(world);

    let mut snap_intents: Vec<SnapIntent> = Vec::new();
    let mut partial_moves: Vec<(Entity, f32, f32)> = Vec::new();

    {
        let mut colonists = world.query::<(Entity, &Position, &Path)>();
        for (entity, pos, path) in colonists.iter(world) {
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
                snap_intents.push(SnapIntent {
                    entity,
                    target: (tx, ty),
                    target_x,
                    target_y,
                });
            } else {
                partial_moves.push((entity, (dx / dist) * step, (dy / dist) * step));
            }
        }
    }

    for (entity, dx, dy) in partial_moves {
        if let Some(mut pos) = world.get_mut::<Position>(entity) {
            pos.x += dx;
            pos.y += dy;
        }
    }

    snap_intents.sort_by_key(|intent| intent.entity);

    for intent in snap_intents {
        if !is_cell_free(&occupancy, intent.target, intent.entity) {
            continue;
        }

        occupancy.retain(|_, occupant| *occupant != intent.entity);
        occupancy.insert(intent.target, intent.entity);

        if let Some(mut pos) = world.get_mut::<Position>(intent.entity) {
            pos.x = intent.target_x;
            pos.y = intent.target_y;
        }
        if let Some(mut path) = world.get_mut::<Path>(intent.entity) {
            path.index += 1;
        }
    }
}

pub fn task_execution(world: &mut World, grid: &mut WorldGrid) {
    apply_build_work(world, grid);

    let completions: Vec<(Entity, TaskKind, i32, i32, i32, i32)> = {
        let mut colonists = world.query::<(Entity, &Position, &Task, &Path)>();
        colonists
            .iter(world)
            .filter_map(|(entity, pos, task, path)| {
                if matches!(task.kind, TaskKind::Idle | TaskKind::Build) {
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
                let adjacent =
                    (gx - building_x).abs() + (gy - building_y).abs() == 1;
                if adjacent
                    && grid.building_at(building_x, building_y) == Some(BuildingType::BerryBush)
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
            TaskKind::Idle | TaskKind::Build => {}
        }

        clear_task(world, colonist_entity);
    }

    for (entity, gx, gy) in to_despawn {
        grid.remove_building(gx, gy);
        let _ = world.despawn(entity);
    }
}

fn apply_build_work(world: &mut World, grid: &mut WorldGrid) {
    let workers: Vec<(Entity, i32, i32, i32, i32)> = {
        let mut colonists = world.query::<(Entity, &Position, &Task, &Path)>();
        colonists
            .iter(world)
            .filter_map(|(entity, pos, task, path)| {
                if !matches!(task.kind, TaskKind::Build) {
                    return None;
                }
                if path.index < path.waypoints.len() {
                    return None;
                }
                let (gx, gy) = pos.grid_cell();
                if gx != task.target_x || gy != task.target_y {
                    return None;
                }
                let adjacent =
                    (gx - task.building_x).abs() + (gy - task.building_y).abs() == 1;
                if !adjacent {
                    return None;
                }
                Some((entity, gx, gy, task.building_x, task.building_y))
            })
            .collect()
    };

    for (colonist_entity, _gx, _gy, building_x, building_y) in workers {
        let site_entity = construction_site_at(world, building_x, building_y);
        let Some(site_entity) = site_entity else {
            release_construction_reservation(world, colonist_entity);
            clear_task(world, colonist_entity);
            continue;
        };

        let should_complete = {
            let mut site = match world.get_mut::<ConstructionSite>(site_entity) {
                Some(s) => s,
                None => continue,
            };
            site.work_remaining -= BUILD_WORK_PER_TICK;
            site.work_remaining <= 0.0
        };

        if should_complete {
            if let Some(site) = world.get::<ConstructionSite>(site_entity).copied() {
                complete_construction(
                    world,
                    grid,
                    site_entity,
                    &site,
                    building_x,
                    building_y,
                );
            }
            clear_task(world, colonist_entity);
        }
    }
}

fn release_construction_reservation(world: &mut World, colonist: Entity) {
    let mut sites = world.query::<&mut ConstructionSite>();
    for mut site in sites.iter_mut(world) {
        if site.reserved_by == Some(colonist) {
            site.reserved_by = None;
        }
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
    if let Some(task) = world.get::<Task>(colonist) {
        match task.kind {
            TaskKind::Build => release_construction_reservation(world, colonist),
            TaskKind::Sleep => release_bed_reservation(world, colonist),
            _ => {}
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{Building, TerrainType};
    use crate::pathfinding::best_adjacent_stand;
    use crate::world::{WorldGrid, NEED_THRESHOLD, WORLD_SIZE};

    fn grass_grid() -> WorldGrid {
        let len = (WORLD_SIZE * WORLD_SIZE) as usize;
        WorldGrid {
            terrain: vec![TerrainType::Grass; len],
            buildings: vec![None; len],
            seed: 0,
        }
    }

    #[test]
    fn eat_task_targets_adjacent_stand_not_bush_tile() {
        let mut grid = grass_grid();
        assert!(grid.place_building(10, 10, BuildingType::BerryBush));

        let mut world = World::new();
        world.spawn((
            Building,
            Position {
                x: 10.0,
                y: 10.0,
            },
            BuildingType::BerryBush,
            BerrySupply::new(3),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs {
                    food: NEED_THRESHOLD - 1.0,
                    sleep: 100.0,
                },
                Task::default(),
                Path::default(),
            ))
            .id();

        auto_assign_tasks(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
        assert_eq!((task.building_x, task.building_y), (10, 10));
        assert_ne!((task.target_x, task.target_y), (10, 10));
        assert_eq!(
            (task.target_x - task.building_x).abs() + (task.target_y - task.building_y).abs(),
            1
        );
    }

    #[test]
    fn eat_execution_requires_adjacency_not_bush_tile() {
        let mut grid = grass_grid();
        assert!(grid.place_building(10, 10, BuildingType::BerryBush));

        let mut world = World::new();
        world.spawn((
            Building,
            Position {
                x: 10.0,
                y: 10.0,
            },
            BuildingType::BerryBush,
            BerrySupply::new(3),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs {
                    food: NEED_THRESHOLD - 1.0,
                    sleep: 100.0,
                },
                Task {
                    kind: TaskKind::Eat,
                    building_x: 10,
                    building_y: 10,
                    target_x: 10,
                    target_y: 10,
                },
                Path::default(),
            ))
            .id();

        task_execution(&mut world, &mut grid);

        let needs = world.get::<Needs>(colonist).unwrap();
        assert!(
            needs.food < NEED_THRESHOLD,
            "standing on bush must not eat"
        );
    }

    #[test]
    fn only_one_colonist_reserves_a_bed() {
        let mut grid = grass_grid();
        assert!(grid.place_building(12, 12, BuildingType::Bed));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position {
                    x: 12.0,
                    y: 12.0,
                },
                BuildingType::Bed,
                BedOccupancy::default(),
            ))
            .id();

        let sleepy = |world: &mut World, x: f32, y: f32| {
            world
                .spawn((
                    Colonist,
                    Position { x, y },
                    Needs {
                        food: 100.0,
                        sleep: NEED_THRESHOLD - 1.0,
                    },
                    Task::default(),
                    Path::default(),
                ))
                .id()
        };

        let c1 = sleepy(&mut world, 5.0, 12.0);
        let c2 = sleepy(&mut world, 6.0, 12.0);

        auto_assign_tasks(&mut world, &grid);

        let assigned: Vec<_> = [c1, c2]
            .iter()
            .filter(|&&e| world.get::<Task>(e).unwrap().kind == TaskKind::Sleep)
            .copied()
            .collect();
        assert_eq!(assigned.len(), 1);

        let occ = world.get::<BedOccupancy>(bed).unwrap();
        assert_eq!(occ.reserved_by, Some(assigned[0]));
    }

    #[test]
    fn bed_reservation_released_when_sleep_path_fails() {
        let mut grid = grass_grid();
        assert!(grid.place_building(12, 12, BuildingType::Bed));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position {
                    x: 12.0,
                    y: 12.0,
                },
                BuildingType::Bed,
                BedOccupancy::default(),
            ))
            .id();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 12.0 },
                Needs {
                    food: 100.0,
                    sleep: 100.0,
                },
                Task {
                    kind: TaskKind::Sleep,
                    building_x: 12,
                    building_y: 12,
                    target_x: 12,
                    target_y: 12,
                },
                Path::default(),
            ))
            .id();

        if let Some(mut occ) = world.get_mut::<BedOccupancy>(bed) {
            occ.reserved_by = Some(colonist);
        }

        auto_assign_tasks(&mut world, &grid);

        let occ = world.get::<BedOccupancy>(bed).unwrap();
        assert!(occ.reserved_by.is_none());
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
    }

    #[test]
    fn build_task_targets_adjacent_stand_not_site_tile() {
        let grid = grass_grid();
        let mut world = World::new();
        world.spawn((
            ConstructionSite {
                building_type: BuildingType::Wall,
                work_remaining: 30.0,
                reserved_by: None,
            },
            Position {
                x: 10.0,
                y: 10.0,
            },
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::new_full(),
                Task::default(),
                Path::default(),
            ))
            .id();

        auto_assign_tasks(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Build);
        assert_eq!((task.building_x, task.building_y), (10, 10));
        assert_ne!((task.target_x, task.target_y), (10, 10));
        assert_eq!(
            (task.target_x - task.building_x).abs() + (task.target_y - task.building_y).abs(),
            1
        );
    }

    #[test]
    fn best_adjacent_stand_skips_unwalkable_neighbors() {
        let mut grid = grass_grid();
        grid.terrain[WorldGrid::index(5, 4).unwrap()] = TerrainType::Water;
        grid.terrain[WorldGrid::index(5, 6).unwrap()] = TerrainType::Water;
        grid.terrain[WorldGrid::index(4, 5).unwrap()] = TerrainType::Water;

        let stand = best_adjacent_stand(&grid, (5, 5), (0, 5)).unwrap();
        assert_eq!(stand, (6, 5));
    }

    #[test]
    fn second_colonist_waits_when_snap_target_occupied() {
        let _grid = grass_grid();
        let mut world = World::new();

        let blocker = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Path {
                    waypoints: vec![(10, 10)],
                    index: 0,
                },
            ))
            .id();

        let waiter = world
            .spawn((
                Colonist,
                Position { x: 9.0, y: 10.0 },
                Path {
                    waypoints: vec![(10, 10)],
                    index: 0,
                },
            ))
            .id();

        colonist_movement(&mut world, 0.25);

        let blocker_cell = world.get::<Position>(blocker).unwrap().grid_cell();
        let waiter_cell = world.get::<Position>(waiter).unwrap().grid_cell();
        assert_eq!(blocker_cell, (10, 10));
        assert_ne!(waiter_cell, (10, 10));
        assert_eq!(world.get::<Path>(waiter).unwrap().index, 0);
    }

    #[test]
    fn colonists_partial_step_without_snap_blocking() {
        let _grid = grass_grid();
        let mut world = World::new();

        world.spawn((
            Colonist,
            Position { x: 9.0, y: 10.0 },
            Path {
                waypoints: vec![(10, 10)],
                index: 0,
            },
        ));

        world.spawn((
            Colonist,
            Position { x: 11.0, y: 10.0 },
            Path {
                waypoints: vec![(10, 10)],
                index: 0,
            },
        ));

        colonist_movement(&mut world, 0.01);

        let indices: Vec<_> = world
            .query::<(&Path, &Colonist)>()
            .iter(&world)
            .map(|(path, _)| path.index)
            .collect();
        assert_eq!(indices, vec![0, 0]);

        let positions: Vec<_> = world
            .query::<(&Position, &Colonist)>()
            .iter(&world)
            .map(|(pos, _)| (pos.x, pos.y))
            .collect();
        assert!(positions.iter().all(|&(x, y)| x != 10.0 || y != 10.0));
    }

    #[test]
    fn eat_assignment_skips_stand_occupied_by_colonist() {
        let mut grid = grass_grid();
        assert!(grid.place_building(10, 10, BuildingType::BerryBush));

        let mut world = World::new();
        world.spawn((
            Building,
            Position {
                x: 10.0,
                y: 10.0,
            },
            BuildingType::BerryBush,
            BerrySupply::new(3),
        ));

        world.spawn((
            Colonist,
            Position { x: 9.0, y: 10.0 },
            Needs::new_full(),
            Task::default(),
            Path::default(),
        ));

        let hungry = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs {
                    food: NEED_THRESHOLD - 1.0,
                    sleep: 100.0,
                },
                Task::default(),
                Path::default(),
            ))
            .id();

        auto_assign_tasks(&mut world, &grid);

        let task = world.get::<Task>(hungry).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
        assert_ne!((task.target_x, task.target_y), (9, 10));
    }

    #[test]
    fn only_one_colonist_assigned_single_bush_stand() {
        let mut grid = grass_grid();
        grid.terrain[WorldGrid::index(10, 9).unwrap()] = TerrainType::Water;
        grid.terrain[WorldGrid::index(10, 11).unwrap()] = TerrainType::Water;
        grid.terrain[WorldGrid::index(9, 10).unwrap()] = TerrainType::Water;
        assert!(grid.place_building(10, 10, BuildingType::BerryBush));

        let mut world = World::new();
        world.spawn((
            Building,
            Position {
                x: 10.0,
                y: 10.0,
            },
            BuildingType::BerryBush,
            BerrySupply::new(3),
        ));

        let hungry = |world: &mut World, x: f32, y: f32| {
            world
                .spawn((
                    Colonist,
                    Position { x, y },
                    Needs {
                        food: NEED_THRESHOLD - 1.0,
                        sleep: 100.0,
                    },
                    Task::default(),
                    Path::default(),
                ))
                .id()
        };

        let c1 = hungry(&mut world, 5.0, 10.0);
        let c2 = hungry(&mut world, 6.0, 10.0);

        auto_assign_tasks(&mut world, &grid);

        let assigned: Vec<_> = [c1, c2]
            .iter()
            .filter(|&&e| world.get::<Task>(e).unwrap().kind == TaskKind::Eat)
            .filter(|&&e| (world.get::<Task>(e).unwrap().target_x, world.get::<Task>(e).unwrap().target_y) == (11, 10))
            .copied()
            .collect();
        assert_eq!(assigned.len(), 1);
    }

    #[test]
    fn idle_colonist_gets_wander_path_when_no_needs_or_build() {
        let grid = grass_grid();
        let mut world = World::new();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs::new_full(),
                Task::default(),
                Path::default(),
            ))
            .id();

        auto_assign_tasks(&mut world, &grid);

        let path = world.get::<Path>(colonist).unwrap();
        assert!(
            !path.waypoints.is_empty(),
            "idle colonist should receive a wander path"
        );
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
    }

    #[test]
    fn wander_path_replaced_when_critical_need_triggers_eat() {
        let mut grid = grass_grid();
        assert!(grid.place_building(10, 10, BuildingType::BerryBush));

        let mut world = World::new();
        world.spawn((
            Building,
            Position {
                x: 10.0,
                y: 10.0,
            },
            BuildingType::BerryBush,
            BerrySupply::new(3),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::new_full(),
                Task::default(),
                Path {
                    waypoints: vec![(6, 10), (7, 10)],
                    index: 0,
                },
            ))
            .id();

        if let Some(mut needs) = world.get_mut::<Needs>(colonist) {
            needs.food = NEED_THRESHOLD - 1.0;
        }

        auto_assign_tasks(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
        let path = world.get::<Path>(colonist).unwrap();
        assert_ne!(path.waypoints, vec![(6, 10), (7, 10)]);
        assert!(!path.waypoints.is_empty());
    }

    #[test]
    fn wander_target_excludes_current_and_occupied_cells() {
        let grid = grass_grid();
        let mut world = World::new();

        let occupant = world
            .spawn((
                Colonist,
                Position { x: 11.0, y: 10.0 },
                Path::default(),
            ))
            .id();

        let wanderer = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Path::default(),
            ))
            .id();

        let occupancy = colonist_occupancy_map(&mut world);
        let from = (10, 10);

        for _ in 0..20 {
            let Some(waypoints) = pick_wander_target(&grid, from, &occupancy, wanderer) else {
                continue;
            };
            let target = waypoints.last().copied().unwrap();
            let dist = (target.0 - from.0).abs() + (target.1 - from.1).abs();
            assert!(
                dist >= WANDER_MIN_RADIUS,
                "wander must be at least {WANDER_MIN_RADIUS} cells away"
            );
            assert_ne!(
                target,
                (11, 10),
                "wander must not target occupied cell"
            );
            assert_ne!(
                occupancy.get(&target),
                Some(&occupant),
                "wander must not target another colonist's cell"
            );
        }
    }

    #[test]
    fn wander_keeps_task_kind_idle() {
        let grid = grass_grid();
        let mut world = World::new();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs::new_full(),
                Task::default(),
                Path::default(),
            ))
            .id();

        auto_assign_tasks(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        let path = world.get::<Path>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Idle);
        assert!(!path.waypoints.is_empty());
    }
}
