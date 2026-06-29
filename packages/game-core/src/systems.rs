use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;

use crate::components::{
    BedOccupancy, BerrySupply, BuildingType, Colonist, ColonistId, ColonistName,
    ConstructionSite, COLONIST_NAME_POOL, Hungry, NeedKind, Needs, Path, Position, SleepingOnBed,
    Task, TaskKind, WantsSleep, BUILD_WORK_PER_TICK,
};
use crate::pathfinding::{best_adjacent_stand, find_path, find_path_avoiding};
use crate::world::{
    WorldGrid, BERRIES_PER_BUSH, FOOD_DECAY_PER_SEC, MOVE_SPEED, NEED_RESTORE, NEED_THRESHOLD,
    SLEEP_DECAY_PER_SEC, SLEEP_ON_BED_SEC, VACATE_SEARCH_RADIUS, WANDER_MIN_RADIUS,
    WANDER_PICK_ATTEMPTS, WANDER_RADIUS, WORLD_SIZE,
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

fn colonist_blocked_cells(
    occupancy: &HashMap<(i32, i32), Entity>,
    self_entity: Entity,
) -> Vec<(i32, i32)> {
    occupancy
        .iter()
        .filter_map(|(&cell, &occupant)| {
            if occupant == self_entity {
                None
            } else {
                Some(cell)
            }
        })
        .collect()
}

fn find_path_for_colonist(
    grid: &WorldGrid,
    from: (i32, i32),
    goal: (i32, i32),
    occupancy: &HashMap<(i32, i32), Entity>,
    self_entity: Entity,
) -> Option<Vec<(i32, i32)>> {
    let blocked = colonist_blocked_cells(occupancy, self_entity);
    find_path_avoiding(grid, from, goal, &blocked)
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
        if let Some(waypoints) = find_path_for_colonist(grid, from, target, occupied, self_entity) {
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

/// Syncs `Hungry` / `WantsSleep` buffs from raw need values.
/// Must run before [`auto_assign_tasks`], which reads buffs only (not raw need values).
pub fn update_need_buffs(world: &mut World) {
    let mut add_hungry = Vec::new();
    let mut remove_hungry = Vec::new();
    let mut add_wants_sleep = Vec::new();
    let mut remove_wants_sleep = Vec::new();

    let mut q = world.query::<(Entity, &Needs, Option<&Hungry>, Option<&WantsSleep>)>();
    for (entity, needs, hungry, wants_sleep) in q.iter(world) {
        let should_hungry = needs.food < NEED_THRESHOLD;
        let should_wants_sleep = needs.sleep < NEED_THRESHOLD;

        if should_hungry && hungry.is_none() {
            add_hungry.push(entity);
        } else if !should_hungry && hungry.is_some() {
            remove_hungry.push(entity);
        }

        if should_wants_sleep && wants_sleep.is_none() {
            add_wants_sleep.push(entity);
        } else if !should_wants_sleep && wants_sleep.is_some() {
            remove_wants_sleep.push(entity);
        }
    }

    for entity in add_hungry {
        world.entity_mut(entity).insert(Hungry);
    }
    for entity in remove_hungry {
        world.entity_mut(entity).remove::<Hungry>();
    }
    for entity in add_wants_sleep {
        world.entity_mut(entity).insert(WantsSleep);
    }
    for entity in remove_wants_sleep {
        world.entity_mut(entity).remove::<WantsSleep>();
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
    // Expects `update_need_buffs` to have run this tick. Colonists without Hungry/WantsSleep
    // enter idle mode (Build assignment, then wander).
    preempt_build_for_critical_needs(world);
    release_unsatisfiable_eat_tasks(world, grid);

    let occupancy = colonist_occupancy_map(world);
    release_unreachable_eat_tasks(world, grid, &occupancy);
    let prefer_sleep_first = release_stuck_tasks(world);

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
    let mut pending: Vec<PendingAssignment> = Vec::new();
    let mut pending_wander: Vec<(Entity, Vec<(i32, i32)>)> = Vec::new();

    let mut colonists =
        world.query::<(Entity, &Position, &Task, &Path, Option<&Hungry>, Option<&WantsSleep>)>();
    for (entity, pos, task, path, hungry, wants_sleep) in colonists.iter(world) {
        if !matches!(task.kind, TaskKind::Idle) {
            continue;
        }

        let (gx, gy) = pos.grid_cell();

        let has_need_buffs = hungry.is_some() || wants_sleep.is_some();

        if has_need_buffs {
            let mut need_assigned = false;
            let mut critical_needs = Vec::new();
            if prefer_sleep_first.contains(&entity) {
                if wants_sleep.is_some() {
                    critical_needs.push(NeedKind::Sleep);
                }
                if hungry.is_some() {
                    critical_needs.push(NeedKind::Food);
                }
            } else {
                if hungry.is_some() {
                    critical_needs.push(NeedKind::Food);
                }
                if wants_sleep.is_some() {
                    critical_needs.push(NeedKind::Sleep);
                }
            }

            for need_kind in critical_needs {
                let Some(mut assignment) = try_need_assignment(
                    entity,
                    grid,
                    (gx, gy),
                    need_kind,
                    &berry_bushes,
                    &free_beds,
                    &occupancy,
                    &reserved_stands,
                    &reserved_beds,
                ) else {
                    continue;
                };

                if let Some(waypoints) = find_path_for_colonist(
                    grid,
                    (gx, gy),
                    (assignment.target_x, assignment.target_y),
                    &occupancy,
                    entity,
                ) {
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
                    need_assigned = true;
                    break;
                }
            }
            if need_assigned {
                continue;
            }
        }

        // Idle mode: no Hungry / WantsSleep buffs — build orders, then wander.
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
        let mut q = world.query::<(Entity, &Task, Option<&Hungry>, Option<&WantsSleep>)>();
        q.iter(world)
            .filter_map(|(entity, task, hungry, wants_sleep)| {
                if !matches!(task.kind, TaskKind::Build) {
                    return None;
                }
                if hungry.is_some() || wants_sleep.is_some() {
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

fn release_unsatisfiable_eat_tasks(world: &mut World, grid: &WorldGrid) {
    let eat_tasks: Vec<(Entity, i32, i32)> = {
        let mut q = world.query::<(Entity, &Task)>();
        q.iter(world)
            .filter_map(|(entity, task)| {
                if matches!(task.kind, TaskKind::Eat) {
                    Some((entity, task.building_x, task.building_y))
                } else {
                    None
                }
            })
            .collect()
    };

    for (entity, bx, by) in eat_tasks {
        let valid = grid.building_at(bx, by) == Some(BuildingType::BerryBush)
            && building_entity_at(world, bx, by).and_then(|be| {
                world
                    .get::<BerrySupply>(be)
                    .map(|supply| supply.remaining > 0)
            }) == Some(true);
        if !valid {
            clear_task(world, entity);
        }
    }
}

fn release_unreachable_eat_tasks(
    world: &mut World,
    grid: &WorldGrid,
    occupancy: &HashMap<(i32, i32), Entity>,
) {
    let eat_tasks: Vec<(Entity, (i32, i32), (i32, i32))> = {
        let mut q = world.query::<(Entity, &Position, &Task)>();
        q.iter(world)
            .filter_map(|(entity, pos, task)| {
                if !matches!(task.kind, TaskKind::Eat) {
                    return None;
                }
                let (gx, gy) = pos.grid_cell();
                Some((entity, (gx, gy), (task.target_x, task.target_y)))
            })
            .collect()
    };

    for (entity, from, target) in eat_tasks {
        if find_path_for_colonist(grid, from, target, occupancy, entity).is_none() {
            clear_task(world, entity);
        }
    }
}

fn release_stuck_tasks(world: &mut World) -> HashSet<Entity> {
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

    let mut prefer_sleep_first = HashSet::new();

    for (entity, kind) in stuck {
        if matches!(kind, TaskKind::Build) {
            release_construction_reservation(world, entity);
        }
        if matches!(kind, TaskKind::Eat) {
            prefer_sleep_first.insert(entity);
        }
        clear_task(world, entity);
    }

    prefer_sleep_first
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

fn try_need_assignment(
    entity: Entity,
    grid: &WorldGrid,
    from: (i32, i32),
    need_kind: NeedKind,
    berry_bushes: &[(i32, i32)],
    free_beds: &[(Entity, i32, i32)],
    occupancy: &HashMap<(i32, i32), Entity>,
    reserved_stands: &HashSet<(i32, i32)>,
    reserved_beds: &HashSet<Entity>,
) -> Option<PendingAssignment> {
    match need_kind {
        NeedKind::Food => nearest_eat_assignment(
            grid,
            from,
            berry_bushes,
            occupancy,
            reserved_stands,
            entity,
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
        NeedKind::Sleep => nearest_free_bed(from, free_beds, reserved_beds).map(
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
    }
}

fn nearest_eat_assignment(
    grid: &WorldGrid,
    from: (i32, i32),
    bushes: &[(i32, i32)],
    occupancy: &HashMap<(i32, i32), Entity>,
    reserved_stands: &HashSet<(i32, i32)>,
    self_entity: Entity,
) -> Option<((i32, i32), (i32, i32))> {
    let blocked = colonist_blocked_cells(occupancy, self_entity);
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
            if find_path_avoiding(grid, from, stand, &blocked).is_some() {
                return Some(((bx, by), stand));
            }
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

/// Finds the nearest free walkable cell by expanding Manhattan rings from `bed`.
fn find_vacate_cell_after_sleep(
    grid: &WorldGrid,
    bed: (i32, i32),
    from: (i32, i32),
    occupancy: &HashMap<(i32, i32), Entity>,
    self_entity: Entity,
) -> Option<(i32, i32)> {
    for ring in 1..=VACATE_SEARCH_RADIUS {
        let mut best: Option<((i32, i32), usize)> = None;

        for dy in -ring..=ring {
            for dx in -ring..=ring {
                if dx.abs() + dy.abs() != ring {
                    continue;
                }
                let cell = (bed.0 + dx, bed.1 + dy);
                if cell == bed || !grid.is_walkable(cell.0, cell.1) {
                    continue;
                }
                if !is_cell_free(occupancy, cell, self_entity) {
                    continue;
                }
                if let Some(path) = find_path(grid, from, cell) {
                    let len = path.len();
                    if best.map(|(_, best_len)| len < best_len).unwrap_or(true) {
                        best = Some((cell, len));
                    }
                }
            }
        }

        if let Some((cell, _)) = best {
            return Some(cell);
        }
    }

    None
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
            if !is_cell_free(&occupancy, (tx, ty), entity) {
                continue;
            }
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

pub fn task_execution(world: &mut World, grid: &mut WorldGrid, dt: f32) {
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
                        if let Some(mut sleeping) = world.get_mut::<SleepingOnBed>(colonist_entity)
                        {
                            sleeping.remaining -= dt;
                            if sleeping.remaining > 0.0 {
                                continue;
                            }
                            let _ = world.entity_mut(colonist_entity).remove::<SleepingOnBed>();
                        } else {
                            world.entity_mut(colonist_entity).insert(SleepingOnBed {
                                remaining: SLEEP_ON_BED_SEC,
                            });
                            continue;
                        }

                        if let Some(mut needs) = world.get_mut::<Needs>(colonist_entity) {
                            needs.set(NeedKind::Sleep, NEED_RESTORE);
                        }
                        let occupancy = colonist_occupancy_map(world);
                        let vacate = find_vacate_cell_after_sleep(
                            grid,
                            (building_x, building_y),
                            (gx, gy),
                            &occupancy,
                            colonist_entity,
                        );
                        clear_task(world, colonist_entity);
                        if let Some(vacate) = vacate {
                            let blocked = colonist_blocked_cells(&occupancy, colonist_entity);
                            if let Some(waypoints) =
                                find_path_avoiding(grid, (gx, gy), vacate, &blocked)
                            {
                                if let Some(mut path) = world.get_mut::<Path>(colonist_entity) {
                                    path.waypoints = if waypoints.len() > 1 {
                                        waypoints[1..].to_vec()
                                    } else {
                                        vec![vacate]
                                    };
                                    path.index = 0;
                                }
                            }
                        }
                        continue;
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
            TaskKind::Sleep => {
                release_bed_reservation(world, colonist);
                let _ = world.entity_mut(colonist).remove::<SleepingOnBed>();
            }
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
    use crate::world::{
        WorldGrid, NEED_RESTORE, NEED_THRESHOLD, SLEEP_ON_BED_SEC, VACATE_SEARCH_RADIUS, WORLD_SIZE,
    };

    fn grass_grid() -> WorldGrid {
        let len = (WORLD_SIZE * WORLD_SIZE) as usize;
        WorldGrid {
            terrain: vec![TerrainType::Grass; len],
            buildings: vec![None; len],
            seed: 0,
        }
    }

    fn run_auto_assign(world: &mut World, grid: &WorldGrid) {
        update_need_buffs(world);
        auto_assign_tasks(world, grid);
    }

    fn run_sleep_and_vacate_at(
        world: &mut World,
        grid: &mut WorldGrid,
        colonist: Entity,
        bed: (i32, i32),
    ) {
        world.entity_mut(colonist).insert(SleepingOnBed { remaining: 0.0 });
        task_execution(world, grid, SLEEP_ON_BED_SEC);
        for _ in 0..64 {
            if world.get::<Position>(colonist).unwrap().grid_cell() == bed {
                colonist_movement(world, 0.05);
            } else {
                break;
            }
        }
    }

    #[test]
    fn hungry_buff_applied_when_food_below_threshold() {
        let mut world = World::new();
        let colonist = world
            .spawn((
                Colonist,
                Needs {
                    food: NEED_THRESHOLD - 1.0,
                    sleep: 100.0,
                },
            ))
            .id();

        update_need_buffs(&mut world);

        assert!(world.get::<Hungry>(colonist).is_some());
        assert!(world.get::<WantsSleep>(colonist).is_none());
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

        run_auto_assign(&mut world, &grid);

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

        task_execution(&mut world, &mut grid, 0.05);

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

        run_auto_assign(&mut world, &grid);

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

        run_auto_assign(&mut world, &grid);

        let occ = world.get::<BedOccupancy>(bed).unwrap();
        assert!(occ.reserved_by.is_none());
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
    }

    fn spawn_colonist_at(world: &mut World, x: i32, y: i32) -> Entity {
        world
            .spawn((
                Colonist,
                Position {
                    x: x as f32,
                    y: y as f32,
                },
                Needs::new_full(),
                Task::default(),
                Path::default(),
            ))
            .id()
    }

    fn spawn_sleeping_on_bed(world: &mut World, bed: Entity, bed_x: i32, bed_y: i32) -> Entity {
        let colonist = world
            .spawn((
                Colonist,
                Position {
                    x: bed_x as f32,
                    y: bed_y as f32,
                },
                Needs {
                    food: 100.0,
                    sleep: NEED_THRESHOLD - 1.0,
                },
                Task {
                    kind: TaskKind::Sleep,
                    building_x: bed_x,
                    building_y: bed_y,
                    target_x: bed_x,
                    target_y: bed_y,
                },
                Path::default(),
            ))
            .id();

        if let Some(mut occ) = world.get_mut::<BedOccupancy>(bed) {
            occ.reserved_by = Some(colonist);
        }
        colonist
    }

    #[test]
    fn colonist_stays_on_bed_while_sleeping() {
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

        let colonist = spawn_sleeping_on_bed(&mut world, bed, 12, 12);

        task_execution(&mut world, &mut grid, 0.05);

        assert!(world.get::<SleepingOnBed>(colonist).is_some());
        assert_eq!(world.get::<Position>(colonist).unwrap().grid_cell(), (12, 12));
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Sleep);
        assert_eq!(
            world.get::<BedOccupancy>(bed).unwrap().reserved_by,
            Some(colonist)
        );
    }

    #[test]
    fn sleep_vacates_to_adjacent_cell_after_completion() {
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

        let colonist = spawn_sleeping_on_bed(&mut world, bed, 12, 12);
        run_sleep_and_vacate_at(&mut world, &mut grid, colonist, (12, 12));

        let pos = world.get::<Position>(colonist).unwrap();
        let (gx, gy) = pos.grid_cell();
        assert_ne!((gx, gy), (12, 12), "colonist must leave the bed tile");
        assert_eq!(
            (gx - 12).abs() + (gy - 12).abs(),
            1,
            "vacate should prefer ring 1"
        );
        assert_eq!(world.get::<Needs>(colonist).unwrap().sleep, NEED_RESTORE);
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
        assert!(world.get::<BedOccupancy>(bed).unwrap().reserved_by.is_none());
    }

    #[test]
    fn sleep_vacates_to_free_adjacent_when_one_ring_one_cell_open() {
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

        for (x, y) in [(12, 11), (12, 13), (11, 12)] {
            spawn_colonist_at(&mut world, x, y);
        }

        let colonist = spawn_sleeping_on_bed(&mut world, bed, 12, 12);
        run_sleep_and_vacate_at(&mut world, &mut grid, colonist, (12, 12));

        let pos = world.get::<Position>(colonist).unwrap();
        assert_eq!(pos.grid_cell(), (13, 12));
        assert_eq!(world.get::<Needs>(colonist).unwrap().sleep, NEED_RESTORE);
        assert!(world.get::<BedOccupancy>(bed).unwrap().reserved_by.is_none());
    }

    #[test]
    fn sleep_stays_on_bed_when_ring_one_fully_blocked() {
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

        for (x, y) in [(12, 11), (12, 13), (11, 12), (13, 12)] {
            spawn_colonist_at(&mut world, x, y);
        }

        let colonist = spawn_sleeping_on_bed(&mut world, bed, 12, 12);
        world.entity_mut(colonist).insert(SleepingOnBed { remaining: 0.0 });
        task_execution(&mut world, &mut grid, SLEEP_ON_BED_SEC);
        for _ in 0..64 {
            colonist_movement(&mut world, 0.05);
        }

        assert_eq!(world.get::<Position>(colonist).unwrap().grid_cell(), (12, 12));
        assert_eq!(world.get::<Needs>(colonist).unwrap().sleep, NEED_RESTORE);
        assert!(world.get::<BedOccupancy>(bed).unwrap().reserved_by.is_none());
    }

    #[test]
    fn sleep_stays_on_bed_when_no_vacate_cell() {
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

        let bed_cell = (12, 12);
        for dy in -VACATE_SEARCH_RADIUS..=VACATE_SEARCH_RADIUS {
            for dx in -VACATE_SEARCH_RADIUS..=VACATE_SEARCH_RADIUS {
                let dist = dx.abs() + dy.abs();
                if dist == 0 || dist > VACATE_SEARCH_RADIUS {
                    continue;
                }
                let cell = (bed_cell.0 + dx, bed_cell.1 + dy);
                spawn_colonist_at(&mut world, cell.0, cell.1);
            }
        }

        let colonist = spawn_sleeping_on_bed(&mut world, bed, 12, 12);
        world.entity_mut(colonist).insert(SleepingOnBed { remaining: 0.0 });
        task_execution(&mut world, &mut grid, SLEEP_ON_BED_SEC);

        let pos = world.get::<Position>(colonist).unwrap();
        assert_eq!(pos.grid_cell(), bed_cell);
        assert_eq!(world.get::<Needs>(colonist).unwrap().sleep, NEED_RESTORE);
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
        assert!(world.get::<BedOccupancy>(bed).unwrap().reserved_by.is_none());
    }

    #[test]
    fn second_colonist_can_sleep_after_first_vacates() {
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

        let first = spawn_sleeping_on_bed(&mut world, bed, 12, 12);
        run_sleep_and_vacate_at(&mut world, &mut grid, first, (12, 12));
        assert_ne!(world.get::<Position>(first).unwrap().grid_cell(), (12, 12));

        let second = spawn_sleeping_on_bed(&mut world, bed, 12, 12);
        world.entity_mut(second).insert(SleepingOnBed { remaining: 0.0 });
        task_execution(&mut world, &mut grid, SLEEP_ON_BED_SEC);

        assert_eq!(world.get::<Needs>(second).unwrap().sleep, NEED_RESTORE);
        assert!(world.get::<BedOccupancy>(bed).unwrap().reserved_by.is_none());
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

        run_auto_assign(&mut world, &grid);

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

        run_auto_assign(&mut world, &grid);

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

        run_auto_assign(&mut world, &grid);

        let assigned: Vec<_> = [c1, c2]
            .iter()
            .filter(|&&e| world.get::<Task>(e).unwrap().kind == TaskKind::Eat)
            .filter(|&&e| (world.get::<Task>(e).unwrap().target_x, world.get::<Task>(e).unwrap().target_y) == (11, 10))
            .copied()
            .collect();
        assert_eq!(assigned.len(), 1);
    }

    #[test]
    fn idle_mode_assigns_wander_when_no_need_buffs() {
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

        update_need_buffs(&mut world);
        assert!(world.get::<Hungry>(colonist).is_none());
        assert!(world.get::<WantsSleep>(colonist).is_none());

        auto_assign_tasks(&mut world, &grid);

        let path = world.get::<Path>(colonist).unwrap();
        assert!(
            !path.waypoints.is_empty(),
            "idle mode should assign wander when no need buffs"
        );
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
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

        run_auto_assign(&mut world, &grid);

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

        run_auto_assign(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
        let path = world.get::<Path>(colonist).unwrap();
        assert_ne!(path.waypoints, vec![(6, 10), (7, 10)]);
        assert!(!path.waypoints.is_empty());
    }

    #[test]
    fn fallback_to_sleep_when_food_path_blocked_by_colonist() {
        let mut grid = grass_grid();
        grid.terrain[WorldGrid::index(10, 9).unwrap()] = TerrainType::Water;
        grid.terrain[WorldGrid::index(10, 11).unwrap()] = TerrainType::Water;
        grid.terrain[WorldGrid::index(11, 10).unwrap()] = TerrainType::Water;
        assert!(grid.place_building(10, 10, BuildingType::BerryBush));
        assert!(grid.place_building(12, 12, BuildingType::Bed));

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
            Building,
            Position {
                x: 12.0,
                y: 12.0,
            },
            BuildingType::Bed,
            BedOccupancy::default(),
        ));

        // Block the only stand tile (9, 10) next to the bush.
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
                    sleep: NEED_THRESHOLD - 1.0,
                },
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid);

        let task = world.get::<Task>(hungry).unwrap();
        assert_eq!(
            task.kind,
            TaskKind::Sleep,
            "blocked food path should fall back to sleep"
        );
    }

    #[test]
    fn fallback_to_sleep_when_food_unavailable() {
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
                    food: NEED_THRESHOLD - 1.0,
                    sleep: NEED_THRESHOLD - 1.0,
                },
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Sleep);
        assert_eq!((task.building_x, task.building_y), (12, 12));
        let occ = world.get::<BedOccupancy>(bed).unwrap();
        assert_eq!(occ.reserved_by, Some(colonist));
    }

    #[test]
    fn fallback_to_eat_when_sleep_unavailable() {
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
                    sleep: NEED_THRESHOLD - 1.0,
                },
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
        assert_eq!((task.building_x, task.building_y), (10, 10));
    }

    #[test]
    fn food_priority_when_both_needs_critical() {
        let mut grid = grass_grid();
        assert!(grid.place_building(10, 10, BuildingType::BerryBush));
        assert!(grid.place_building(12, 12, BuildingType::Bed));

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
            Building,
            Position {
                x: 12.0,
                y: 12.0,
            },
            BuildingType::Bed,
            BedOccupancy::default(),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs {
                    food: NEED_THRESHOLD - 1.0,
                    sleep: NEED_THRESHOLD - 1.0,
                },
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
    }

    #[test]
    fn stays_idle_when_both_needs_critical_but_nothing_satisfiable() {
        let grid = grass_grid();
        let mut world = World::new();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs {
                    food: NEED_THRESHOLD - 1.0,
                    sleep: NEED_THRESHOLD - 1.0,
                },
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Idle);
        let path = world.get::<Path>(colonist).unwrap();
        assert!(
            !path.waypoints.is_empty(),
            "unsatisfiable critical needs should fall back to wander"
        );
    }

    #[test]
    fn hungry_colonist_wanders_when_no_food_available() {
        let grid = grass_grid();
        let mut world = World::new();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs {
                    food: 0.0,
                    sleep: NEED_RESTORE,
                },
                Task::default(),
                Path::default(),
            ))
            .id();

        update_need_buffs(&mut world);
        assert!(world.get::<Hungry>(colonist).is_some());

        auto_assign_tasks(&mut world, &grid);

        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
        assert!(
            !world.get::<Path>(colonist).unwrap().waypoints.is_empty(),
            "hungry colonist with no food source should wander"
        );
    }

    #[test]
    fn eat_task_released_when_bush_depleted_then_fallback_to_sleep() {
        let mut grid = grass_grid();
        assert!(grid.place_building(10, 10, BuildingType::BerryBush));
        assert!(grid.place_building(12, 12, BuildingType::Bed));

        let mut world = World::new();
        let bush = world
            .spawn((
                Building,
                Position {
                    x: 10.0,
                    y: 10.0,
                },
                BuildingType::BerryBush,
                BerrySupply::new(0),
            ))
            .id();
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
                Position { x: 5.0, y: 10.0 },
                Needs {
                    food: NEED_THRESHOLD - 1.0,
                    sleep: NEED_THRESHOLD - 1.0,
                },
                Task {
                    kind: TaskKind::Eat,
                    building_x: 10,
                    building_y: 10,
                    target_x: 9,
                    target_y: 10,
                },
                Path {
                    waypoints: vec![(9, 10)],
                    index: 0,
                },
            ))
            .id();

        let _ = bush;
        run_auto_assign(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Sleep);
        let occ = world.get::<BedOccupancy>(bed).unwrap();
        assert_eq!(occ.reserved_by, Some(colonist));
    }

    #[test]
    fn wander_path_replaced_when_fallback_sleep_assignment_succeeds() {
        let mut grid = grass_grid();
        assert!(grid.place_building(12, 12, BuildingType::Bed));

        let mut world = World::new();
        world.spawn((
            Building,
            Position {
                x: 12.0,
                y: 12.0,
            },
            BuildingType::Bed,
            BedOccupancy::default(),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 12.0 },
                Needs {
                    food: NEED_THRESHOLD - 1.0,
                    sleep: NEED_THRESHOLD - 1.0,
                },
                Task::default(),
                Path {
                    waypoints: vec![(6, 12), (7, 12)],
                    index: 0,
                },
            ))
            .id();

        run_auto_assign(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Sleep);
        let path = world.get::<Path>(colonist).unwrap();
        assert_ne!(path.waypoints, vec![(6, 12), (7, 12)]);
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

        run_auto_assign(&mut world, &grid);

        let task = world.get::<Task>(colonist).unwrap();
        let path = world.get::<Path>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Idle);
        assert!(!path.waypoints.is_empty());
    }

    #[test]
    fn wander_path_avoids_occupied_cells_on_first_step() {
        use crate::world::generate_world;

        for seed in 0..200u32 {
            let grid = generate_world(seed);
            let mut world = World::new();
            spawn_colonists(&mut world, &grid);

            update_need_buffs(&mut world);
            auto_assign_tasks(&mut world, &grid);
            let occupancy = colonist_occupancy_map(&mut world);

            let mut q = world.query::<(Entity, &Position, &Task, &Path)>();
            for (entity, pos, task, path) in q.iter(&world) {
                if task.kind != TaskKind::Idle || path.waypoints.is_empty() {
                    continue;
                }
                let wp = path.waypoints[0];
                assert!(
                    !occupancy.get(&wp).is_some_and(|&o| o != entity),
                    "seed {seed}: colonist at {:?} has blocked first waypoint {:?}",
                    pos.grid_cell(),
                    wp
                );
            }
        }
    }

    #[test]
    fn spawn_colonists_all_move_after_wander_assignment() {
        use crate::world::generate_world;

        let grid = generate_world(42);
        let mut world = World::new();
        spawn_colonists(&mut world, &grid);

        let mut start_positions: HashMap<Entity, (i32, i32)> = HashMap::new();
        {
            let mut q = world.query::<(Entity, &Position, &Colonist)>();
            for (entity, pos, _) in q.iter(&world) {
                start_positions.insert(entity, pos.grid_cell());
            }
        }
        assert_eq!(start_positions.len(), 3, "expected 3 spawned colonists");

        for _ in 0..200 {
            update_need_buffs(&mut world);
            auto_assign_tasks(&mut world, &grid);
            colonist_movement(&mut world, 0.05);
        }

        let mut stuck = Vec::new();
        let occupancy = colonist_occupancy_map(&mut world);
        {
            let mut q = world.query::<(Entity, &Position, &Task, &Path)>();
            for (entity, pos, task, path) in q.iter(&world) {
                let start = start_positions[&entity];
                let now = pos.grid_cell();
                let has_active_path = path.index < path.waypoints.len();
                if task.kind == TaskKind::Idle && has_active_path && now == start {
                    let wp = path.waypoints[path.index];
                    let blocked_first = occupancy
                        .get(&wp)
                        .filter(|&&e| e != entity)
                        .copied();
                    stuck.push((entity, start, wp, blocked_first));
                }
            }
        }

        assert!(
            stuck.is_empty(),
            "colonists stuck at spawn with active wander path: {:?}",
            stuck
        );
    }
}
