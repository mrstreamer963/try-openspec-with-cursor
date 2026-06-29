use std::collections::{HashMap, HashSet};

use bevy_ecs::prelude::*;

use crate::components::{
    ActiveStatuses, BedOccupancy, BerrySupply, BuildingKind, Colonist, ColonistId, ColonistName,
    ConstructionSite, DeconstructionSite, Needs, Path, Position, SleepingOnBed, Task, TaskKind,
    BUILD_WORK_PER_TICK, COLONIST_NAME_POOL,
};
use crate::content::{
    ApplyCondition, BuildingId, ContentRegistry, InteractionEffect, InteractionMode, NeedId,
    SpawnPrimitive, StatusId, TaskKindRef,
};
use crate::pathfinding::{find_path, find_path_avoiding};
use crate::world::{
    WorldGrid, MOVEMENT_SUBSTEP_DT, MOVE_SPEED, VACATE_SEARCH_RADIUS, WANDER_MIN_RADIUS,
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

fn is_cell_free(map: &HashMap<(i32, i32), Entity>, cell: (i32, i32), self_entity: Entity) -> bool {
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
    content: &ContentRegistry,
    from: (i32, i32),
    goal: (i32, i32),
    occupancy: &HashMap<(i32, i32), Entity>,
    self_entity: Entity,
) -> Option<Vec<(i32, i32)>> {
    let blocked = colonist_blocked_cells(occupancy, self_entity);
    find_path_avoiding(grid, content, from, goal, &blocked)
}

fn waypoints_from_route(route: &[(i32, i32)]) -> Vec<(i32, i32)> {
    if route.len() > 1 {
        route[1..].to_vec()
    } else {
        route.to_vec()
    }
}

fn set_path_from_route(path: &mut Path, route: Vec<(i32, i32)>) {
    path.waypoints = waypoints_from_route(&route);
    path.index = 0;
}

fn is_blocks_settle_cell(grid: &WorldGrid, content: &ContentRegistry, cell: (i32, i32)) -> bool {
    grid.building_at(cell.0, cell.1)
        .map(|b| content.blocks_settle(b))
        .unwrap_or(false)
}

/// Walkable cells where a colonist may snap or idle (excludes blocks_settle building tiles).
fn is_colonist_settle_cell(grid: &WorldGrid, content: &ContentRegistry, cell: (i32, i32)) -> bool {
    grid.is_walkable(content, cell.0, cell.1) && !is_blocks_settle_cell(grid, content, cell)
}

fn colonist_at_task_target(pos: &Position, task: &Task) -> bool {
    let cell = pos.grid_cell();
    cell == (task.target_x, task.target_y)
}

pub fn colonist_at_task_stand(pos: &Position, task: &Task, path: &Path) -> bool {
    match task.kind {
        TaskKind::Build | TaskKind::Deconstruct => colonist_at_labor_site(pos, task),
        TaskKind::Eat | TaskKind::Sleep => {
            if path.index < path.waypoints.len() {
                return false;
            }
            colonist_at_task_target(pos, task)
        }
        TaskKind::Idle => false,
    }
}

/// Build/deconstruct worker is locked in place once adjacent to the site.
fn colonist_at_labor_site(pos: &Position, task: &Task) -> bool {
    if !matches!(task.kind, TaskKind::Build | TaskKind::Deconstruct) {
        return false;
    }
    let cell = pos.grid_cell();
    (cell.0 - task.building_x).abs() + (cell.1 - task.building_y).abs() == 1
}

fn colonist_is_building(pos: &Position, task: &Task) -> bool {
    task.kind == TaskKind::Build && colonist_at_labor_site(pos, task)
}

fn colonist_is_deconstructing(pos: &Position, task: &Task) -> bool {
    task.kind == TaskKind::Deconstruct && colonist_at_labor_site(pos, task)
}

fn colonist_pins_cell(world: &World, entity: Entity, cell: (i32, i32)) -> bool {
    let Some(task) = world.get::<Task>(entity) else {
        return false;
    };
    let Some(pos) = world.get::<Position>(entity) else {
        return false;
    };
    let Some(path) = world.get::<Path>(entity) else {
        return false;
    };

    match task.kind {
        TaskKind::Build | TaskKind::Deconstruct => {
            colonist_at_labor_site(pos, task) && pos.grid_cell() == cell
        }
        TaskKind::Eat | TaskKind::Sleep => {
            colonist_at_task_stand(pos, task, path) && pos.grid_cell() == cell
        }
        TaskKind::Idle => false,
    }
}

fn reserved_stands_for_active_tasks(world: &mut World) -> HashSet<(i32, i32)> {
    let mut reserved = HashSet::new();
    let mut q = world.query::<(&Task, &Position)>();
    for (task, pos) in q.iter(world) {
        match task.kind {
            TaskKind::Eat => {
                reserved.insert((task.target_x, task.target_y));
            }
            TaskKind::Build | TaskKind::Deconstruct => {
                if colonist_at_labor_site(pos, task) {
                    reserved.insert(pos.grid_cell());
                } else {
                    reserved.insert((task.target_x, task.target_y));
                }
            }
            TaskKind::Idle | TaskKind::Sleep => {}
        }
    }
    reserved
}

/// Snap labor workers on adjacent cells and clear any remaining path to the stand.
fn lock_labor_colonists(world: &mut World) {
    let mut q = world.query::<(&mut Position, &Task, &mut Path)>();
    for (mut pos, task, mut path) in q.iter_mut(world) {
        if !colonist_at_labor_site(&*pos, task) {
            continue;
        }
        let cell = pos.grid_cell();
        pos.x = cell.0 as f32;
        pos.y = cell.1 as f32;
        path.waypoints.clear();
        path.index = 0;
    }
}

/// Clears movement for colonists already on their task stand (e.g. eating).
fn stabilize_task_stand_holders(world: &mut World) {
    let mut q = world.query::<(&mut Position, &Task, &mut Path)>();
    for (mut pos, task, mut path) in q.iter_mut(world) {
        if task.kind == TaskKind::Eat && colonist_at_task_target(&*pos, task) {
            pos.x = task.target_x as f32;
            pos.y = task.target_y as f32;
            path.waypoints.clear();
            path.index = 0;
        }
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
    content: &ContentRegistry,
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
            if !is_colonist_settle_cell(grid, content, cell) {
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
        if let Some(waypoints) =
            find_path_for_colonist(grid, content, from, target, occupied, self_entity)
        {
            return Some(if waypoints.len() > 1 {
                waypoints[1..].to_vec()
            } else {
                vec![target]
            });
        }
    }
    None
}

pub fn spawn_colonists(world: &mut World, grid: &WorldGrid, content: &ContentRegistry) -> u32 {
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
            if grid.terrain_at(x, y) == Some(content.grass_terrain)
                && grid.is_walkable(content, x, y)
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
                    Needs::new_full(content),
                    ActiveStatuses::default(),
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

pub fn needs_decay(world: &mut World, content: &ContentRegistry, dt: f32) {
    let mut query = world.query::<&mut Needs>();
    for mut needs in query.iter_mut(world) {
        for (idx, need_def) in content.needs.iter().enumerate() {
            let id = NeedId(idx as u8);
            let current = needs.get(id);
            needs.set(id, (current - need_def.decay_per_sec * dt).max(0.0));
        }
    }
}

/// Syncs active statuses from raw need values.
/// Must run before [`auto_assign_tasks`], which reads statuses only (not raw need values).
pub fn sync_statuses(world: &mut World, content: &ContentRegistry) {
    let mut to_add: Vec<(Entity, StatusId)> = Vec::new();
    let mut to_remove: Vec<(Entity, StatusId)> = Vec::new();

    let mut q = world.query::<(Entity, &Needs, Option<&ActiveStatuses>)>();
    for (entity, needs, statuses) in q.iter(world) {
        for (idx, status_def) in content.statuses.iter().enumerate() {
            let status_id = StatusId(idx as u8);
            let need_value = needs.get(status_def.apply_when.need);
            let threshold = content
                .need_def(status_def.apply_when.need)
                .critical_threshold;
            let should_apply = match status_def.apply_when.condition {
                ApplyCondition::BelowThreshold => need_value < threshold,
            };
            let has_status = statuses.map(|s| s.has(status_id)).unwrap_or(false);
            if should_apply && !has_status {
                to_add.push((entity, status_id));
            } else if !should_apply && has_status {
                to_remove.push((entity, status_id));
            }
        }
    }

    for (entity, status_id) in to_add {
        if world.get::<ActiveStatuses>(entity).is_none() {
            world.entity_mut(entity).insert(ActiveStatuses::default());
        }
        if let Some(mut active) = world.get_mut::<ActiveStatuses>(entity) {
            active.0.insert(status_id);
        }
    }
    for (entity, status_id) in to_remove {
        if let Some(mut active) = world.get_mut::<ActiveStatuses>(entity) {
            active.0.remove(&status_id);
        }
    }
}

pub fn construction_site_at(world: &mut World, x: i32, y: i32) -> Option<Entity> {
    let mut q = world.query::<(Entity, &Position, &ConstructionSite)>();
    q.iter(world)
        .find(|(_, pos, _)| pos.grid_cell() == (x, y))
        .map(|(entity, _, _)| entity)
}

pub fn deconstruction_site_at(world: &mut World, x: i32, y: i32) -> Option<Entity> {
    let mut q = world.query::<(Entity, &Position, &DeconstructionSite)>();
    q.iter(world)
        .find(|(_, pos, _)| pos.grid_cell() == (x, y))
        .map(|(entity, _, _)| entity)
}

pub fn is_valid_build_tile(
    world: &mut World,
    grid: &WorldGrid,
    content: &ContentRegistry,
    x: i32,
    y: i32,
) -> bool {
    if !grid
        .terrain_at(x, y)
        .map(|t| content.terrain_def(t).walkable)
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
    if deconstruction_site_at(world, x, y).is_some() {
        return false;
    }
    true
}

pub fn complete_construction(
    world: &mut World,
    grid: &mut WorldGrid,
    content: &ContentRegistry,
    site_entity: Entity,
    site: &ConstructionSite,
    x: i32,
    y: i32,
) {
    let building_id = site.building_id;
    if !grid.place_building(&content, x, y, building_id) {
        return;
    }
    let position = Position {
        x: x as f32,
        y: y as f32,
    };
    let entity = world
        .spawn((
            crate::components::Building,
            position,
            BuildingKind(building_id),
        ))
        .id();
    for primitive in &content.building_def(building_id).on_complete {
        match primitive {
            SpawnPrimitive::Supply { amount, .. } => {
                world.entity_mut(entity).insert(BerrySupply::new(*amount));
            }
            SpawnPrimitive::Reservation { .. } => {
                world.entity_mut(entity).insert(BedOccupancy::default());
            }
        }
    }
    let _ = world.despawn(site_entity);
}

pub fn complete_deconstruction(
    world: &mut World,
    grid: &mut WorldGrid,
    site_entity: Entity,
    _site: &DeconstructionSite,
    x: i32,
    y: i32,
) {
    if grid.building_at(x, y).is_some() {
        grid.remove_building(x, y);
        if let Some(be) = building_entity_at(world, x, y) {
            let _ = world.despawn(be);
        }
    }
    let _ = world.despawn(site_entity);
}

fn is_deconstruction_assignable(
    world: &mut World,
    content: &ContentRegistry,
    building_id: BuildingId,
    x: i32,
    y: i32,
) -> bool {
    if building_id == content.wall_building {
        return true;
    }

    if let Some(site_entity) = construction_site_at(world, x, y) {
        if let Some(site) = world.get::<ConstructionSite>(site_entity) {
            if site.reserved_by.is_some() {
                return false;
            }
        }
        let mut q = world.query::<&Task>();
        for task in q.iter(world) {
            if task.kind == TaskKind::Build && task.building_x == x && task.building_y == y {
                return false;
            }
        }
    }

    if building_id == content.bed_building {
        if let Some(be) = building_entity_at(world, x, y) {
            if world
                .get::<BedOccupancy>(be)
                .is_some_and(|occ| occ.reserved_by.is_some())
            {
                return false;
            }
        }
        let mut q = world.query::<(&Position, &Task)>();
        for (pos, task) in q.iter(world) {
            if task.kind == TaskKind::Sleep && pos.grid_cell() == (x, y) {
                return false;
            }
        }
    }

    if building_id == content.berry_bush_building {
        let mut q = world.query::<&Task>();
        for task in q.iter(world) {
            if task.kind == TaskKind::Eat && task.building_x == x && task.building_y == y {
                return false;
            }
        }
    }

    true
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

pub fn auto_assign_tasks(world: &mut World, grid: &WorldGrid, content: &ContentRegistry) {
    // Expects `sync_statuses` to have run this tick. Colonists without active need statuses
    // enter idle mode (Build assignment, then wander).
    preempt_labor_for_critical_needs(world, content);
    release_unsatisfiable_eat_tasks(world, grid, content);
    release_unreachable_eat_tasks(world, grid, content);

    let occupancy = colonist_occupancy_map(world);
    let mut prefer_sleep_first = release_stuck_tasks(world);
    prefer_sleep_first.extend(repath_blocked_colonists(world, grid, content));

    let berry_bushes: Vec<(i32, i32)> = {
        let mut q = world.query::<(&Position, &BuildingKind, Option<&BerrySupply>)>();
        q.iter(world)
            .filter_map(|(pos, kind, supply)| {
                if !content.has_supply_on_complete(kind.0) {
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
        let mut q = world.query::<(Entity, &Position, &BuildingKind, &BedOccupancy)>();
        q.iter(world)
            .filter_map(|(entity, pos, kind, occ)| {
                if !content.has_reservation_on_complete(kind.0) || occ.reserved_by.is_some() {
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

    let open_decon_sites: Vec<(Entity, i32, i32)> = {
        let candidates: Vec<(Entity, i32, i32, BuildingId)> = {
            let mut q = world.query::<(Entity, &Position, &DeconstructionSite)>();
            q.iter(world)
                .filter_map(|(entity, pos, site)| {
                    if site.reserved_by.is_some() {
                        return None;
                    }
                    let (gx, gy) = pos.grid_cell();
                    Some((entity, gx, gy, site.building_id))
                })
                .collect()
        };
        candidates
            .into_iter()
            .filter(|(_, gx, gy, building_id)| {
                is_deconstruction_assignable(world, content, *building_id, *gx, *gy)
            })
            .map(|(entity, gx, gy, _)| (entity, gx, gy))
            .collect()
    };

    let mut reserved_beds: HashSet<Entity> = HashSet::new();
    let mut reserved_sites: HashSet<Entity> = HashSet::new();
    let mut reserved_stands = reserved_stands_for_active_tasks(world);
    let mut pending: Vec<PendingAssignment> = Vec::new();
    let mut pending_wander: Vec<(Entity, Vec<(i32, i32)>)> = Vec::new();

    let mut colonists = world.query::<(Entity, &Position, &Task, &Path, Option<&ActiveStatuses>)>();
    for (entity, pos, task, path, statuses) in colonists.iter(world) {
        if !matches!(task.kind, TaskKind::Idle) {
            continue;
        }

        let (gx, gy) = pos.grid_cell();

        let has_need_statuses = statuses
            .map(|s| s.has(content.hungry_status) || s.has(content.wants_sleep_status))
            .unwrap_or(false);

        if has_need_statuses {
            let mut need_assigned = false;
            let mut critical_tasks = Vec::new();
            if prefer_sleep_first.contains(&entity) {
                if statuses
                    .map(|s| s.has(content.wants_sleep_status))
                    .unwrap_or(false)
                {
                    if let Some(task) = content.task_for_status(content.wants_sleep_status) {
                        critical_tasks.push(task);
                    }
                }
                if statuses
                    .map(|s| s.has(content.hungry_status))
                    .unwrap_or(false)
                {
                    if let Some(task) = content.task_for_status(content.hungry_status) {
                        critical_tasks.push(task);
                    }
                }
            } else {
                if statuses
                    .map(|s| s.has(content.hungry_status))
                    .unwrap_or(false)
                {
                    if let Some(task) = content.task_for_status(content.hungry_status) {
                        critical_tasks.push(task);
                    }
                }
                if statuses
                    .map(|s| s.has(content.wants_sleep_status))
                    .unwrap_or(false)
                {
                    if let Some(task) = content.task_for_status(content.wants_sleep_status) {
                        critical_tasks.push(task);
                    }
                }
            }

            for task_ref in critical_tasks {
                let Some(mut assignment) = try_need_assignment(
                    entity,
                    grid,
                    content,
                    (gx, gy),
                    task_ref,
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
                    content,
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
                    if matches!(
                        assignment.task_kind,
                        TaskKind::Build | TaskKind::Deconstruct
                    ) {
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

        // Idle mode: no active need statuses — build/deconstruct orders, then wander.
        if let Some((site_entity, task_kind, (bx, by), (sx, sy))) = nearest_job_assignment(
            grid,
            content,
            (gx, gy),
            &open_sites,
            &open_decon_sites,
            &reserved_sites,
            &occupancy,
            &reserved_stands,
            entity,
        ) {
            if let Some(waypoints) = find_path(grid, content, (gx, gy), (sx, sy)) {
                let path_waypoints = if waypoints.len() > 1 {
                    waypoints[1..].to_vec()
                } else {
                    vec![(sx, sy)]
                };

                reserved_sites.insert(site_entity);

                pending.push(PendingAssignment {
                    entity,
                    task_kind,
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
            if let Some(waypoints) = pick_wander_target(grid, content, (gx, gy), &occupancy, entity)
            {
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
            if assignment.task_kind == TaskKind::Build {
                if let Some(mut site) = world.get_mut::<ConstructionSite>(site_entity) {
                    site.reserved_by = Some(assignment.entity);
                }
            } else if assignment.task_kind == TaskKind::Deconstruct {
                if let Some(mut site) = world.get_mut::<DeconstructionSite>(site_entity) {
                    site.reserved_by = Some(assignment.entity);
                }
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

fn preempt_labor_for_critical_needs(world: &mut World, content: &ContentRegistry) {
    let preemptions: Vec<Entity> = {
        let mut q = world.query::<(Entity, &Task, Option<&ActiveStatuses>)>();
        q.iter(world)
            .filter_map(|(entity, task, statuses)| {
                if !matches!(task.kind, TaskKind::Build | TaskKind::Deconstruct) {
                    return None;
                }
                if statuses
                    .map(|s| s.has(content.hungry_status) || s.has(content.wants_sleep_status))
                    .unwrap_or(false)
                {
                    Some(entity)
                } else {
                    None
                }
            })
            .collect()
    };

    for entity in preemptions {
        release_labor_reservation(world, entity);
        clear_task(world, entity);
    }
}

fn release_unsatisfiable_eat_tasks(world: &mut World, grid: &WorldGrid, content: &ContentRegistry) {
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
        let valid = grid.building_at(bx, by).is_some_and(|id| {
            content.has_supply_on_complete(id)
                && building_entity_at(world, bx, by).and_then(|be| {
                    world
                        .get::<BerrySupply>(be)
                        .map(|supply| supply.remaining > 0)
                }) == Some(true)
        });
        if !valid {
            clear_task(world, entity);
        }
    }
}

fn release_unreachable_eat_tasks(world: &mut World, grid: &WorldGrid, content: &ContentRegistry) {
    let occupancy = colonist_occupancy_map(world);
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
        if find_path_for_colonist(grid, content, from, target, &occupancy, entity).is_none() {
            clear_task(world, entity);
        }
    }
}

fn release_stuck_tasks(world: &mut World) -> HashSet<Entity> {
    let stuck: Vec<(Entity, TaskKind)> = {
        let mut q = world.query::<(Entity, &Position, &Task, &Path)>();
        q.iter(world)
            .filter_map(|(entity, pos, task, path)| {
                if !matches!(
                    task.kind,
                    TaskKind::Build | TaskKind::Deconstruct | TaskKind::Eat | TaskKind::Sleep
                ) {
                    return None;
                }
                if matches!(task.kind, TaskKind::Build | TaskKind::Deconstruct)
                    && colonist_at_labor_site(pos, task)
                {
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
        if matches!(kind, TaskKind::Build | TaskKind::Deconstruct) {
            release_labor_reservation(world, entity);
        }
        if matches!(kind, TaskKind::Eat) {
            prefer_sleep_first.insert(entity);
        }
        clear_task(world, entity);
    }

    prefer_sleep_first
}

/// Recalculates paths for colonists blocked on the next waypoint.
/// Idle colonists get their path cleared for wander reassignment; active tasks repath to target.
fn repath_blocked_colonists(
    world: &mut World,
    grid: &WorldGrid,
    content: &ContentRegistry,
) -> HashSet<Entity> {
    let occupancy = colonist_occupancy_map(world);
    let blocked: Vec<Entity> = {
        let mut q = world.query::<(Entity, &Task, &Path)>();
        q.iter(world)
            .filter_map(|(entity, _task, path)| {
                if path.index >= path.waypoints.len() {
                    return None;
                }
                let (tx, ty) = path.waypoints[path.index];
                if is_cell_free(&occupancy, (tx, ty), entity) {
                    None
                } else {
                    Some(entity)
                }
            })
            .collect()
    };

    let mut prefer_sleep_first = HashSet::new();

    for entity in blocked {
        let task = match world.get::<Task>(entity) {
            Some(t) => *t,
            None => continue,
        };
        let from = match world.get::<Position>(entity) {
            Some(p) => p.grid_cell(),
            None => continue,
        };

        match task.kind {
            TaskKind::Idle => {
                if let Some(mut path) = world.get_mut::<Path>(entity) {
                    path.clear();
                }
            }
            TaskKind::Eat | TaskKind::Build | TaskKind::Deconstruct | TaskKind::Sleep => {
                if matches!(task.kind, TaskKind::Build | TaskKind::Deconstruct) {
                    if let Some(pos) = world.get::<Position>(entity) {
                        if colonist_at_labor_site(pos, &task) {
                            if let Some(mut path) = world.get_mut::<Path>(entity) {
                                path.clear();
                            }
                            if let Some(mut pos) = world.get_mut::<Position>(entity) {
                                let cell = pos.grid_cell();
                                pos.x = cell.0 as f32;
                                pos.y = cell.1 as f32;
                            }
                            continue;
                        }
                    }
                }

                let goal = (task.target_x, task.target_y);
                let blocked_on_goal = world
                    .get::<Path>(entity)
                    .map(|path| path.waypoints[path.index] == goal)
                    .unwrap_or(false);

                if blocked_on_goal {
                    if task.kind == TaskKind::Eat {
                        prefer_sleep_first.insert(entity);
                        clear_task(world, entity);
                    } else if task.kind == TaskKind::Sleep {
                        clear_task(world, entity);
                    }
                    // Build/Deconstruct: wait for the stand to free — do not reassign to another cell.
                    continue;
                }

                match find_path_for_colonist(grid, content, from, goal, &occupancy, entity) {
                    Some(route) => {
                        let new_waypoints = waypoints_from_route(&route);
                        let should_update = world
                            .get::<Path>(entity)
                            .map(|path| new_waypoints.as_slice() != &path.waypoints[path.index..])
                            .unwrap_or(false);
                        if should_update {
                            if let Some(mut path) = world.get_mut::<Path>(entity) {
                                set_path_from_route(&mut path, route);
                            }
                        }
                    }
                    None => {
                        if task.kind == TaskKind::Eat {
                            prefer_sleep_first.insert(entity);
                        }
                        clear_task(world, entity);
                    }
                }
            }
        }
    }

    prefer_sleep_first
}

fn nearest_job_assignment(
    grid: &WorldGrid,
    content: &ContentRegistry,
    from: (i32, i32),
    build_sites: &[(Entity, i32, i32)],
    decon_sites: &[(Entity, i32, i32)],
    reserved: &HashSet<Entity>,
    occupancy: &HashMap<(i32, i32), Entity>,
    reserved_stands: &HashSet<(i32, i32)>,
    self_entity: Entity,
) -> Option<(Entity, TaskKind, (i32, i32), (i32, i32))> {
    let blocked = colonist_blocked_cells(occupancy, self_entity);
    let mut candidates: Vec<(Entity, TaskKind, i32, i32, i32)> = Vec::new();

    for &(entity, sx, sy) in build_sites {
        if reserved.contains(&entity) {
            continue;
        }
        let dist = (sx - from.0).abs() + (sy - from.1).abs();
        candidates.push((entity, TaskKind::Build, sx, sy, dist));
    }
    for &(entity, sx, sy) in decon_sites {
        if reserved.contains(&entity) {
            continue;
        }
        let dist = (sx - from.0).abs() + (sy - from.1).abs();
        candidates.push((entity, TaskKind::Deconstruct, sx, sy, dist));
    }
    candidates.sort_by_key(|(_, _, _, _, dist)| *dist);

    for (entity, task_kind, sx, sy, _) in candidates {
        if let Some(stand) = crate::pathfinding::best_adjacent_stand_filtered(
            grid,
            content,
            (sx, sy),
            from,
            |stand| {
                !reserved_stands.contains(&stand)
                    && !occupancy.contains_key(&stand)
                    && !is_blocks_settle_cell(grid, content, stand)
            },
        ) {
            if find_path_avoiding(grid, content, from, stand, &blocked).is_some() {
                return Some((entity, task_kind, (sx, sy), stand));
            }
        }
    }
    None
}

fn try_need_assignment(
    entity: Entity,
    grid: &WorldGrid,
    content: &ContentRegistry,
    from: (i32, i32),
    task_ref: TaskKindRef,
    berry_bushes: &[(i32, i32)],
    free_beds: &[(Entity, i32, i32)],
    occupancy: &HashMap<(i32, i32), Entity>,
    reserved_stands: &HashSet<(i32, i32)>,
    reserved_beds: &HashSet<Entity>,
) -> Option<PendingAssignment> {
    match task_ref {
        TaskKindRef::Eat => nearest_eat_assignment(
            grid,
            content,
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
        TaskKindRef::Sleep => nearest_free_bed(from, free_beds, reserved_beds, occupancy, entity)
            .map(|(bed_entity, bx, by)| PendingAssignment {
                entity,
                task_kind: TaskKind::Sleep,
                building_x: bx,
                building_y: by,
                target_x: bx,
                target_y: by,
                waypoints: Vec::new(),
                bed_entity: Some(bed_entity),
                site_entity: None,
            }),
        TaskKindRef::Build => None,
    }
}

fn nearest_eat_assignment(
    grid: &WorldGrid,
    content: &ContentRegistry,
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
            content,
            (bx, by),
            from,
            |stand| {
                stand_available_for_eat(stand, occupancy, reserved_stands)
                    && !is_blocks_settle_cell(grid, content, stand)
            },
        ) {
            if find_path_avoiding(grid, content, from, stand, &blocked).is_some() {
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
    occupancy: &HashMap<(i32, i32), Entity>,
    self_entity: Entity,
) -> Option<(Entity, i32, i32)> {
    beds.iter()
        .filter(|(entity, _, _)| !reserved.contains(entity))
        .filter(|(_, bx, by)| is_cell_free(occupancy, (*bx, *by), self_entity))
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
    content: &ContentRegistry,
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
                if cell == bed || !is_colonist_settle_cell(grid, content, cell) {
                    continue;
                }
                if !is_cell_free(occupancy, cell, self_entity) {
                    continue;
                }
                if let Some(path) = find_path(grid, content, from, cell) {
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

/// Separates colonists sharing a grid cell (can happen after partial movement).
fn resolve_colonist_overlaps(world: &mut World, grid: &WorldGrid, content: &ContentRegistry) {
    loop {
        let occupancy = colonist_occupancy_map(world);
        let mut by_cell: HashMap<(i32, i32), Vec<Entity>> = HashMap::new();
        let mut q = world.query::<(Entity, &Position, &Colonist)>();
        for (entity, pos, _) in q.iter(world) {
            by_cell.entry(pos.grid_cell()).or_default().push(entity);
        }

        let mut to_relocate: Option<(Entity, (i32, i32))> = None;
        for (cell, mut entities) in by_cell {
            if entities.len() <= 1 {
                continue;
            }
            entities.sort_by_key(|e| e.to_bits());
            let entity = entities
                .iter()
                .find(|&&e| !colonist_pins_cell(world, e, cell))
                .copied()
                .unwrap_or(entities[1]);
            if let Some(dest) =
                find_vacate_cell_after_sleep(grid, content, cell, cell, &occupancy, entity)
            {
                to_relocate = Some((entity, dest));
                break;
            }
        }

        let Some((entity, dest)) = to_relocate else {
            break;
        };

        if let Some(mut pos) = world.get_mut::<Position>(entity) {
            pos.x = dest.0 as f32;
            pos.y = dest.1 as f32;
        }
        if let Some(mut path) = world.get_mut::<Path>(entity) {
            path.clear();
        }
    }
}

/// Advances past blocks_settle waypoints so colonists pass through without snapping on them.
fn skip_blocks_settle_waypoints(world: &mut World, grid: &WorldGrid, content: &ContentRegistry) {
    let entities: Vec<Entity> = {
        let mut q = world.query::<(Entity, &Path)>();
        q.iter(world).map(|(e, _)| e).collect()
    };
    for entity in entities {
        if let Some(mut path) = world.get_mut::<Path>(entity) {
            while path.index < path.waypoints.len()
                && is_blocks_settle_cell(grid, content, path.waypoints[path.index])
            {
                path.index += 1;
            }
        }
    }
}

/// Moves colonists whose settled cell blocks settle to a nearby settleable tile.
fn eject_colonists_from_blocks_settle(
    world: &mut World,
    grid: &WorldGrid,
    content: &ContentRegistry,
) {
    let occupancy = colonist_occupancy_map(world);
    let on_blocked: Vec<(Entity, (i32, i32))> = {
        let mut q = world.query::<(Entity, &Position, &Colonist)>();
        q.iter(world)
            .filter_map(|(entity, pos, _)| {
                let cell = pos.grid_cell();
                if is_blocks_settle_cell(grid, content, cell) {
                    Some((entity, cell))
                } else {
                    None
                }
            })
            .collect()
    };

    for (entity, cell) in on_blocked {
        if let (Some(task), Some(pos)) = (world.get::<Task>(entity), world.get::<Position>(entity))
        {
            if colonist_at_labor_site(pos, task) {
                continue;
            }
        }
        let Some(dest) =
            find_vacate_cell_after_sleep(grid, content, cell, cell, &occupancy, entity)
        else {
            continue;
        };
        if let Some(mut pos) = world.get_mut::<Position>(entity) {
            pos.x = dest.0 as f32;
            pos.y = dest.1 as f32;
        }
        if let Some(mut path) = world.get_mut::<Path>(entity) {
            path.clear();
        }
    }
}

pub fn colonist_movement(world: &mut World, grid: &WorldGrid, content: &ContentRegistry, dt: f32) {
    lock_labor_colonists(world);
    stabilize_task_stand_holders(world);

    let mut remaining = dt;
    while remaining > 0.0 {
        let sub_dt = remaining.min(MOVEMENT_SUBSTEP_DT);
        colonist_movement_substep(world, grid, content, sub_dt);
        remaining -= sub_dt;
    }

    eject_colonists_from_blocks_settle(world, grid, content);
    resolve_colonist_overlaps(world, grid, content);
    lock_labor_colonists(world);
    stabilize_task_stand_holders(world);
}

fn colonist_movement_substep(
    world: &mut World,
    grid: &WorldGrid,
    content: &ContentRegistry,
    dt: f32,
) {
    skip_blocks_settle_waypoints(world, grid, content);

    let step = MOVE_SPEED * dt;
    let mut occupancy = colonist_occupancy_map(world);

    let mut snap_intents: Vec<SnapIntent> = Vec::new();
    let mut partial_moves: Vec<(Entity, f32, f32)> = Vec::new();
    let mut hold_build_stands: Vec<(Entity, i32, i32)> = Vec::new();

    {
        let mut colonists = world.query::<(Entity, &Position, &Path, Option<&Task>)>();
        for (entity, pos, path, task) in colonists.iter(world) {
            if let Some(task) = task {
                if colonist_at_labor_site(pos, task) {
                    let cell = pos.grid_cell();
                    hold_build_stands.push((entity, cell.0, cell.1));
                    continue;
                }
            }
            if path.index >= path.waypoints.len() {
                continue;
            }

            let (tx, ty) = path.waypoints[path.index];
            if is_blocks_settle_cell(grid, content, (tx, ty)) {
                continue;
            }
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
                let move_dx = (dx / dist) * step;
                let move_dy = (dy / dist) * step;
                let old_cell = pos.grid_cell();
                let new_cell = (
                    (pos.x + move_dx).floor() as i32,
                    (pos.y + move_dy).floor() as i32,
                );
                if new_cell != old_cell && !is_cell_free(&occupancy, new_cell, entity) {
                    continue;
                }
                partial_moves.push((entity, move_dx, move_dy));
            }
        }
    }

    for (entity, tx, ty) in hold_build_stands {
        if let Some(mut pos) = world.get_mut::<Position>(entity) {
            pos.x = tx as f32;
            pos.y = ty as f32;
        }
        if let Some(mut path) = world.get_mut::<Path>(entity) {
            path.clear();
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
        if is_blocks_settle_cell(grid, content, intent.target)
            || !is_cell_free(&occupancy, intent.target, intent.entity)
        {
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

pub fn task_execution(world: &mut World, grid: &mut WorldGrid, content: &ContentRegistry, dt: f32) {
    apply_build_work(world, grid, content);
    apply_deconstruct_work(world, grid, content);

    let completions: Vec<(Entity, TaskKind, i32, i32, i32, i32)> = {
        let mut colonists = world.query::<(Entity, &Position, &Task, &Path)>();
        colonists
            .iter(world)
            .filter_map(|(entity, pos, task, path)| {
                if matches!(
                    task.kind,
                    TaskKind::Idle | TaskKind::Build | TaskKind::Deconstruct
                ) {
                    return None;
                }
                if path.index < path.waypoints.len() {
                    return None;
                }
                let (gx, gy) = pos.grid_cell();
                if gx != task.target_x || gy != task.target_y {
                    return None;
                }
                Some((entity, task.kind, task.building_x, task.building_y, gx, gy))
            })
            .collect()
    };

    let mut to_despawn: Vec<(Entity, i32, i32)> = Vec::new();

    for (colonist_entity, kind, building_x, building_y, gx, gy) in completions {
        match kind {
            TaskKind::Eat => {
                let Some(building_id) = grid.building_at(building_x, building_y) else {
                    continue;
                };
                let Some(interaction) = content.eat_interaction(building_id) else {
                    continue;
                };
                if interaction.mode != InteractionMode::Adjacent {
                    continue;
                }
                let adjacent = (gx - building_x).abs() + (gy - building_y).abs() == 1;
                if !adjacent {
                    continue;
                }

                let mut ate = false;
                let mut depleted = false;
                let mut building_entity = None;

                if let Some(be) = building_entity_at(world, building_x, building_y) {
                    for effect in &interaction.effects {
                        match effect {
                            InteractionEffect::ConsumeSupply { amount, .. } => {
                                if let Some(mut supply) = world.get_mut::<BerrySupply>(be) {
                                    if supply.remaining >= *amount {
                                        supply.remaining -= amount;
                                        ate = true;
                                        depleted = supply.remaining == 0;
                                        building_entity = Some(be);
                                    }
                                }
                            }
                            InteractionEffect::RestoreNeed { need, amount } => {
                                if ate {
                                    if let Some(mut needs) = world.get_mut::<Needs>(colonist_entity)
                                    {
                                        needs.set(*need, *amount);
                                    }
                                }
                            }
                        }
                    }
                }

                if depleted {
                    if let Some(be) = building_entity {
                        to_despawn.push((be, building_x, building_y));
                    }
                    if let Some(decon_entity) =
                        deconstruction_site_at(world, building_x, building_y)
                    {
                        release_deconstruction_site_reservations(world, decon_entity);
                        let _ = world.despawn(decon_entity);
                    }
                }
            }
            TaskKind::Sleep => {
                let Some(building_id) = grid.building_at(gx, gy) else {
                    continue;
                };
                let Some(interaction) = content.sleep_interaction(building_id) else {
                    continue;
                };
                if gx != building_x
                    || gy != building_y
                    || interaction.mode != InteractionMode::OnTile
                {
                    continue;
                }
                let reserved = building_entity_at(world, building_x, building_y)
                    .and_then(|be| world.get::<BedOccupancy>(be).and_then(|o| o.reserved_by));
                if reserved == Some(colonist_entity) {
                    if let Some(mut sleeping) = world.get_mut::<SleepingOnBed>(colonist_entity) {
                        sleeping.remaining -= dt;
                        if sleeping.remaining > 0.0 {
                            continue;
                        }
                        let _ = world.entity_mut(colonist_entity).remove::<SleepingOnBed>();
                    } else {
                        world.entity_mut(colonist_entity).insert(SleepingOnBed {
                            remaining: interaction.duration_sec,
                        });
                        continue;
                    }

                    for effect in &interaction.effects {
                        if let InteractionEffect::RestoreNeed { need, amount } = effect {
                            if let Some(mut needs) = world.get_mut::<Needs>(colonist_entity) {
                                needs.set(*need, *amount);
                            }
                        }
                    }
                    let occupancy = colonist_occupancy_map(world);
                    let vacate = find_vacate_cell_after_sleep(
                        grid,
                        content,
                        (building_x, building_y),
                        (gx, gy),
                        &occupancy,
                        colonist_entity,
                    );
                    clear_task(world, colonist_entity);
                    if let Some(vacate) = vacate {
                        let blocked = colonist_blocked_cells(&occupancy, colonist_entity);
                        if let Some(waypoints) =
                            find_path_avoiding(grid, content, (gx, gy), vacate, &blocked)
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
            TaskKind::Idle | TaskKind::Build | TaskKind::Deconstruct => {}
        }

        clear_task(world, colonist_entity);
    }

    for (entity, gx, gy) in to_despawn {
        grid.remove_building(gx, gy);
        let _ = world.despawn(entity);
    }
}

fn apply_build_work(world: &mut World, grid: &mut WorldGrid, content: &ContentRegistry) {
    let workers: Vec<(Entity, i32, i32, i32, i32)> = {
        let mut colonists = world.query::<(Entity, &Position, &Task, &Path)>();
        colonists
            .iter(world)
            .filter_map(|(entity, pos, task, _path)| {
                if !colonist_is_building(pos, task) {
                    return None;
                }
                Some((
                    entity,
                    pos.grid_cell().0,
                    pos.grid_cell().1,
                    task.building_x,
                    task.building_y,
                ))
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
                    content,
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

fn apply_deconstruct_work(world: &mut World, grid: &mut WorldGrid, _content: &ContentRegistry) {
    let workers: Vec<(Entity, i32, i32, i32, i32)> = {
        let mut colonists = world.query::<(Entity, &Position, &Task, &Path)>();
        colonists
            .iter(world)
            .filter_map(|(entity, pos, task, _path)| {
                if !colonist_is_deconstructing(pos, task) {
                    return None;
                }
                Some((
                    entity,
                    pos.grid_cell().0,
                    pos.grid_cell().1,
                    task.building_x,
                    task.building_y,
                ))
            })
            .collect()
    };

    for (colonist_entity, _gx, _gy, building_x, building_y) in workers {
        let site_entity = deconstruction_site_at(world, building_x, building_y);
        let Some(site_entity) = site_entity else {
            release_labor_reservation(world, colonist_entity);
            clear_task(world, colonist_entity);
            continue;
        };

        let should_complete = {
            let mut site = match world.get_mut::<DeconstructionSite>(site_entity) {
                Some(s) => s,
                None => continue,
            };
            site.work_remaining -= BUILD_WORK_PER_TICK;
            site.work_remaining <= 0.0
        };

        if should_complete {
            if let Some(site) = world.get::<DeconstructionSite>(site_entity).copied() {
                complete_deconstruction(world, grid, site_entity, &site, building_x, building_y);
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

fn release_deconstruction_reservation(world: &mut World, colonist: Entity) {
    let mut sites = world.query::<&mut DeconstructionSite>();
    for mut site in sites.iter_mut(world) {
        if site.reserved_by == Some(colonist) {
            site.reserved_by = None;
        }
    }
}

fn release_deconstruction_site_reservations(world: &mut World, site_entity: Entity) {
    if let Some(mut site) = world.get_mut::<DeconstructionSite>(site_entity) {
        if let Some(colonist) = site.reserved_by.take() {
            clear_task(world, colonist);
        }
    }
}

fn release_labor_reservation(world: &mut World, colonist: Entity) {
    release_construction_reservation(world, colonist);
    release_deconstruction_reservation(world, colonist);
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
            TaskKind::Deconstruct => release_deconstruction_reservation(world, colonist),
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
    let mut q = world.query::<(Entity, &Position, &BuildingKind)>();
    q.iter(world)
        .find(|(_, pos, _)| pos.grid_cell() == (x, y))
        .map(|(entity, _, _)| entity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::Building;
    use crate::content::base_content;
    use crate::pathfinding::best_adjacent_stand;
    use crate::world::{WorldGrid, VACATE_SEARCH_RADIUS, WORLD_SIZE};

    fn test_content() -> ContentRegistry {
        base_content()
    }

    fn grass_grid(content: &ContentRegistry) -> WorldGrid {
        let len = (WORLD_SIZE * WORLD_SIZE) as usize;
        WorldGrid {
            terrain: vec![content.grass_terrain; len],
            buildings: vec![None; len],
            seed: 0,
        }
    }

    fn run_auto_assign(world: &mut World, grid: &WorldGrid, content: &ContentRegistry) {
        sync_statuses(world, content);
        auto_assign_tasks(world, grid, content);
    }

    fn run_sleep_and_vacate_at(
        world: &mut World,
        grid: &mut WorldGrid,
        content: &ContentRegistry,
        colonist: Entity,
        bed: (i32, i32),
    ) {
        let sleep_sec = content
            .sleep_interaction(content.bed_building)
            .unwrap()
            .duration_sec;
        world
            .entity_mut(colonist)
            .insert(SleepingOnBed { remaining: 0.0 });
        task_execution(world, grid, content, sleep_sec);
        for _ in 0..64 {
            if world.get::<Position>(colonist).unwrap().grid_cell() == bed {
                colonist_movement(world, grid, content, 0.05);
            } else {
                break;
            }
        }
    }

    #[test]
    fn hungry_buff_applied_when_food_below_threshold() {
        let content = test_content();
        let mut world = World::new();
        let colonist = world
            .spawn((
                Colonist,
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    100.0,
                ),
            ))
            .id();

        sync_statuses(&mut world, &content);

        assert!(world
            .get::<ActiveStatuses>(colonist)
            .unwrap()
            .0
            .contains(&content.hungry_status));
        assert!(!world
            .get::<ActiveStatuses>(colonist)
            .map(|s| s.0.contains(&content.wants_sleep_status))
            .unwrap_or(false));
    }

    #[test]
    fn eat_task_targets_adjacent_stand_not_bush_tile() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    100.0,
                ),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

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
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    100.0,
                ),
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

        task_execution(&mut world, &mut grid, &content, 0.05);

        let needs = world.get::<Needs>(colonist).unwrap();
        assert!(
            needs.get(content.food_need) < content.need_def(content.food_need).critical_threshold,
            "standing on bush must not eat"
        );
    }

    #[test]
    fn only_one_colonist_reserves_a_bed() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        let sleepy = |world: &mut World, x: f32, y: f32| {
            world
                .spawn((
                    Colonist,
                    Position { x, y },
                    Needs::with_values(
                        &content,
                        100.0,
                        content.need_def(content.sleep_need).critical_threshold - 1.0,
                    ),
                    ActiveStatuses::default(),
                    Task::default(),
                    Path::default(),
                ))
                .id()
        };

        let c1 = sleepy(&mut world, 5.0, 12.0);
        let c2 = sleepy(&mut world, 6.0, 12.0);

        run_auto_assign(&mut world, &grid, &content);

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
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 12.0 },
                Needs::with_values(&content, 100.0, 100.0),
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

        run_auto_assign(&mut world, &grid, &content);

        let occ = world.get::<BedOccupancy>(bed).unwrap();
        assert!(occ.reserved_by.is_none());
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
    }

    fn spawn_colonist_at(world: &mut World, content: &ContentRegistry, x: i32, y: i32) -> Entity {
        world
            .spawn((
                Colonist,
                Position {
                    x: x as f32,
                    y: y as f32,
                },
                Needs::new_full(content),
                ActiveStatuses::default(),
                Task::default(),
                Path::default(),
            ))
            .id()
    }

    fn spawn_sleeping_on_bed(
        world: &mut World,
        content: &ContentRegistry,
        bed: Entity,
        bed_x: i32,
        bed_y: i32,
    ) -> Entity {
        let colonist = world
            .spawn((
                Colonist,
                Position {
                    x: bed_x as f32,
                    y: bed_y as f32,
                },
                Needs::with_values(
                    content,
                    100.0,
                    content.need_def(content.sleep_need).critical_threshold - 1.0,
                ),
                ActiveStatuses::default(),
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
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        let colonist = spawn_sleeping_on_bed(&mut world, &content, bed, 12, 12);

        task_execution(&mut world, &mut grid, &content, 0.05);

        assert!(world.get::<SleepingOnBed>(colonist).is_some());
        assert_eq!(
            world.get::<Position>(colonist).unwrap().grid_cell(),
            (12, 12)
        );
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Sleep);
        assert_eq!(
            world.get::<BedOccupancy>(bed).unwrap().reserved_by,
            Some(colonist)
        );
    }

    #[test]
    fn sleep_vacates_to_adjacent_cell_after_completion() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        let colonist = spawn_sleeping_on_bed(&mut world, &content, bed, 12, 12);
        run_sleep_and_vacate_at(&mut world, &mut grid, &content, colonist, (12, 12));

        let pos = world.get::<Position>(colonist).unwrap();
        let (gx, gy) = pos.grid_cell();
        assert_ne!((gx, gy), (12, 12), "colonist must leave the bed tile");
        assert_eq!(
            (gx - 12).abs() + (gy - 12).abs(),
            1,
            "vacate should prefer ring 1"
        );
        assert_eq!(
            world
                .get::<Needs>(colonist)
                .unwrap()
                .get(content.sleep_need),
            content.need_def(content.sleep_need).max
        );
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
        assert!(world
            .get::<BedOccupancy>(bed)
            .unwrap()
            .reserved_by
            .is_none());
    }

    #[test]
    fn sleep_vacates_to_free_adjacent_when_one_ring_one_cell_open() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        for (x, y) in [(12, 11), (12, 13), (11, 12)] {
            spawn_colonist_at(&mut world, &content, x, y);
        }

        let colonist = spawn_sleeping_on_bed(&mut world, &content, bed, 12, 12);
        run_sleep_and_vacate_at(&mut world, &mut grid, &content, colonist, (12, 12));

        let pos = world.get::<Position>(colonist).unwrap();
        assert_eq!(pos.grid_cell(), (13, 12));
        assert_eq!(
            world
                .get::<Needs>(colonist)
                .unwrap()
                .get(content.sleep_need),
            content.need_def(content.sleep_need).max
        );
        assert!(world
            .get::<BedOccupancy>(bed)
            .unwrap()
            .reserved_by
            .is_none());
    }

    #[test]
    fn sleep_stays_on_bed_when_ring_one_fully_blocked() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        for (x, y) in [(12, 11), (12, 13), (11, 12), (13, 12)] {
            spawn_colonist_at(&mut world, &content, x, y);
        }

        let colonist = spawn_sleeping_on_bed(&mut world, &content, bed, 12, 12);
        world
            .entity_mut(colonist)
            .insert(SleepingOnBed { remaining: 0.0 });
        task_execution(
            &mut world,
            &mut grid,
            &content,
            content
                .sleep_interaction(content.bed_building)
                .unwrap()
                .duration_sec,
        );
        for _ in 0..64 {
            colonist_movement(&mut world, &grid, &content, 0.05);
        }

        assert_eq!(
            world.get::<Position>(colonist).unwrap().grid_cell(),
            (12, 12)
        );
        assert_eq!(
            world
                .get::<Needs>(colonist)
                .unwrap()
                .get(content.sleep_need),
            content.need_def(content.sleep_need).max
        );
        assert!(world
            .get::<BedOccupancy>(bed)
            .unwrap()
            .reserved_by
            .is_none());
    }

    #[test]
    fn sleep_stays_on_bed_when_no_vacate_cell() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
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
                spawn_colonist_at(&mut world, &content, cell.0, cell.1);
            }
        }

        let colonist = spawn_sleeping_on_bed(&mut world, &content, bed, 12, 12);
        world
            .entity_mut(colonist)
            .insert(SleepingOnBed { remaining: 0.0 });
        task_execution(
            &mut world,
            &mut grid,
            &content,
            content
                .sleep_interaction(content.bed_building)
                .unwrap()
                .duration_sec,
        );

        let pos = world.get::<Position>(colonist).unwrap();
        assert_eq!(pos.grid_cell(), bed_cell);
        assert_eq!(
            world
                .get::<Needs>(colonist)
                .unwrap()
                .get(content.sleep_need),
            content.need_def(content.sleep_need).max
        );
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
        assert!(world
            .get::<BedOccupancy>(bed)
            .unwrap()
            .reserved_by
            .is_none());
    }

    #[test]
    fn second_colonist_can_sleep_after_first_vacates() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        let first = spawn_sleeping_on_bed(&mut world, &content, bed, 12, 12);
        run_sleep_and_vacate_at(&mut world, &mut grid, &content, first, (12, 12));
        assert_ne!(world.get::<Position>(first).unwrap().grid_cell(), (12, 12));

        let second = spawn_sleeping_on_bed(&mut world, &content, bed, 12, 12);
        world
            .entity_mut(second)
            .insert(SleepingOnBed { remaining: 0.0 });
        task_execution(
            &mut world,
            &mut grid,
            &content,
            content
                .sleep_interaction(content.bed_building)
                .unwrap()
                .duration_sec,
        );

        assert_eq!(
            world.get::<Needs>(second).unwrap().get(content.sleep_need),
            content.need_def(content.sleep_need).max
        );
        assert!(world
            .get::<BedOccupancy>(bed)
            .unwrap()
            .reserved_by
            .is_none());
    }

    #[test]
    fn build_task_targets_adjacent_stand_not_site_tile() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();
        world.spawn((
            ConstructionSite {
                building_id: content.wall_building,
                work_remaining: 30.0,
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::new_full(&content),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

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
        let content = test_content();
        let mut grid = grass_grid(&content);
        grid.terrain[WorldGrid::index(5, 4).unwrap()] = content.water_terrain;
        grid.terrain[WorldGrid::index(5, 6).unwrap()] = content.water_terrain;
        grid.terrain[WorldGrid::index(4, 5).unwrap()] = content.water_terrain;

        let stand = best_adjacent_stand(&grid, &content, (5, 5), (0, 5)).unwrap();
        assert_eq!(stand, (6, 5));
    }

    #[test]
    fn second_colonist_waits_when_snap_target_occupied() {
        let content = test_content();
        let grid = grass_grid(&content);
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

        colonist_movement(&mut world, &grid, &content, 0.25);

        let blocker_cell = world.get::<Position>(blocker).unwrap().grid_cell();
        let waiter_cell = world.get::<Position>(waiter).unwrap().grid_cell();
        assert_eq!(blocker_cell, (10, 10));
        assert_ne!(waiter_cell, (10, 10));
        assert_eq!(world.get::<Path>(waiter).unwrap().index, 0);
    }

    #[test]
    fn colonists_partial_step_without_snap_blocking() {
        let content = test_content();
        let grid = grass_grid(&content);
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

        colonist_movement(&mut world, &grid, &content, 0.01);

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
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));

        world.spawn((
            Colonist,
            Position { x: 9.0, y: 10.0 },
            Needs::new_full(&content),
            Task::default(),
            Path::default(),
        ));

        let hungry = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    100.0,
                ),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(hungry).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
        assert_ne!((task.target_x, task.target_y), (9, 10));
    }

    #[test]
    fn only_one_colonist_assigned_single_bush_stand() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        grid.terrain[WorldGrid::index(10, 9).unwrap()] = content.water_terrain;
        grid.terrain[WorldGrid::index(10, 11).unwrap()] = content.water_terrain;
        grid.terrain[WorldGrid::index(9, 10).unwrap()] = content.water_terrain;
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));

        let hungry = |world: &mut World, x: f32, y: f32| {
            world
                .spawn((
                    Colonist,
                    Position { x, y },
                    Needs::with_values(
                        &content,
                        content.need_def(content.food_need).critical_threshold - 1.0,
                        100.0,
                    ),
                    Task::default(),
                    Path::default(),
                ))
                .id()
        };

        let c1 = hungry(&mut world, 5.0, 10.0);
        let c2 = hungry(&mut world, 6.0, 10.0);

        run_auto_assign(&mut world, &grid, &content);

        let assigned: Vec<_> = [c1, c2]
            .iter()
            .filter(|&&e| world.get::<Task>(e).unwrap().kind == TaskKind::Eat)
            .filter(|&&e| {
                (
                    world.get::<Task>(e).unwrap().target_x,
                    world.get::<Task>(e).unwrap().target_y,
                ) == (11, 10)
            })
            .copied()
            .collect();
        assert_eq!(assigned.len(), 1);
    }

    #[test]
    fn idle_mode_assigns_wander_when_no_need_buffs() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs::new_full(&content),
                Task::default(),
                Path::default(),
            ))
            .id();

        sync_statuses(&mut world, &content);
        assert!(!world
            .get::<ActiveStatuses>(colonist)
            .map(|s| s.0.contains(&content.hungry_status))
            .unwrap_or(false));
        assert!(!world
            .get::<ActiveStatuses>(colonist)
            .map(|s| s.0.contains(&content.wants_sleep_status))
            .unwrap_or(false));

        auto_assign_tasks(&mut world, &grid, &content);

        let path = world.get::<Path>(colonist).unwrap();
        assert!(
            !path.waypoints.is_empty(),
            "idle mode should assign wander when no need buffs"
        );
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
    }

    #[test]
    fn idle_colonist_gets_wander_path_when_no_needs_or_build() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs::new_full(&content),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let path = world.get::<Path>(colonist).unwrap();
        assert!(
            !path.waypoints.is_empty(),
            "idle colonist should receive a wander path"
        );
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
    }

    #[test]
    fn wander_path_replaced_when_critical_need_triggers_eat() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::new_full(&content),
                Task::default(),
                Path {
                    waypoints: vec![(6, 10), (7, 10)],
                    index: 0,
                },
            ))
            .id();

        if let Some(mut needs) = world.get_mut::<Needs>(colonist) {
            needs.set(
                content.food_need,
                content.need_def(content.food_need).critical_threshold - 1.0,
            );
        }

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
        let path = world.get::<Path>(colonist).unwrap();
        assert_ne!(path.waypoints, vec![(6, 10), (7, 10)]);
        assert!(!path.waypoints.is_empty());
    }

    #[test]
    fn fallback_to_sleep_when_food_path_blocked_by_colonist() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        grid.terrain[WorldGrid::index(10, 9).unwrap()] = content.water_terrain;
        grid.terrain[WorldGrid::index(10, 11).unwrap()] = content.water_terrain;
        grid.terrain[WorldGrid::index(11, 10).unwrap()] = content.water_terrain;
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));
        world.spawn((
            Building,
            Position { x: 12.0, y: 12.0 },
            BuildingKind(content.bed_building),
            BedOccupancy::default(),
        ));

        // Block the only stand tile (9, 10) next to the bush.
        world.spawn((
            Colonist,
            Position { x: 9.0, y: 10.0 },
            Needs::new_full(&content),
            Task::default(),
            Path::default(),
        ));

        let hungry = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    content.need_def(content.sleep_need).critical_threshold - 1.0,
                ),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(hungry).unwrap();
        assert_eq!(
            task.kind,
            TaskKind::Sleep,
            "blocked food path should fall back to sleep"
        );
    }

    #[test]
    fn fallback_to_sleep_when_food_unavailable() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 12.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    content.need_def(content.sleep_need).critical_threshold - 1.0,
                ),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Sleep);
        assert_eq!((task.building_x, task.building_y), (12, 12));
        let occ = world.get::<BedOccupancy>(bed).unwrap();
        assert_eq!(occ.reserved_by, Some(colonist));
    }

    #[test]
    fn fallback_to_eat_when_sleep_unavailable() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    content.need_def(content.sleep_need).critical_threshold - 1.0,
                ),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
        assert_eq!((task.building_x, task.building_y), (10, 10));
    }

    #[test]
    fn food_priority_when_both_needs_critical() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));
        world.spawn((
            Building,
            Position { x: 12.0, y: 12.0 },
            BuildingKind(content.bed_building),
            BedOccupancy::default(),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    content.need_def(content.sleep_need).critical_threshold - 1.0,
                ),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
    }

    #[test]
    fn stays_idle_when_both_needs_critical_but_nothing_satisfiable() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    content.need_def(content.sleep_need).critical_threshold - 1.0,
                ),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

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
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs::with_values(&content, 0.0, content.need_def(content.sleep_need).max),
                Task::default(),
                Path::default(),
            ))
            .id();

        sync_statuses(&mut world, &content);
        assert!(world
            .get::<ActiveStatuses>(colonist)
            .unwrap()
            .0
            .contains(&content.hungry_status));

        auto_assign_tasks(&mut world, &grid, &content);

        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
        assert!(
            !world.get::<Path>(colonist).unwrap().waypoints.is_empty(),
            "hungry colonist with no food source should wander"
        );
    }

    #[test]
    fn eat_task_released_when_bush_depleted_then_fallback_to_sleep() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        let bush = world
            .spawn((
                Building,
                Position { x: 10.0, y: 10.0 },
                BuildingKind(content.berry_bush_building),
                BerrySupply::new(0),
            ))
            .id();
        let bed = world
            .spawn((
                Building,
                Position { x: 12.0, y: 12.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    content.need_def(content.sleep_need).critical_threshold - 1.0,
                ),
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
        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Sleep);
        let occ = world.get::<BedOccupancy>(bed).unwrap();
        assert_eq!(occ.reserved_by, Some(colonist));
    }

    #[test]
    fn wander_path_replaced_when_fallback_sleep_assignment_succeeds() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 12.0, y: 12.0 },
            BuildingKind(content.bed_building),
            BedOccupancy::default(),
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 12.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    content.need_def(content.sleep_need).critical_threshold - 1.0,
                ),
                Task::default(),
                Path {
                    waypoints: vec![(6, 12), (7, 12)],
                    index: 0,
                },
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Sleep);
        let path = world.get::<Path>(colonist).unwrap();
        assert_ne!(path.waypoints, vec![(6, 12), (7, 12)]);
        assert!(!path.waypoints.is_empty());
    }

    #[test]
    fn wander_target_excludes_current_and_occupied_cells() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();

        let occupant = world
            .spawn((Colonist, Position { x: 11.0, y: 10.0 }, Path::default()))
            .id();

        let wanderer = world
            .spawn((Colonist, Position { x: 10.0, y: 10.0 }, Path::default()))
            .id();

        let occupancy = colonist_occupancy_map(&mut world);
        let from = (10, 10);

        for _ in 0..20 {
            let Some(waypoints) = pick_wander_target(&grid, &content, from, &occupancy, wanderer)
            else {
                continue;
            };
            let target = waypoints.last().copied().unwrap();
            let dist = (target.0 - from.0).abs() + (target.1 - from.1).abs();
            assert!(
                dist >= WANDER_MIN_RADIUS,
                "wander must be at least {WANDER_MIN_RADIUS} cells away"
            );
            assert_ne!(target, (11, 10), "wander must not target occupied cell");
            assert_ne!(
                occupancy.get(&target),
                Some(&occupant),
                "wander must not target another colonist's cell"
            );
        }
    }

    #[test]
    fn wander_keeps_task_kind_idle() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs::new_full(&content),
                Task::default(),
                Path::default(),
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(colonist).unwrap();
        let path = world.get::<Path>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Idle);
        assert!(!path.waypoints.is_empty());
    }

    #[test]
    fn wander_path_avoids_occupied_cells_on_first_step() {
        let content = test_content();
        use crate::world::generate_world;

        for seed in 0..200u32 {
            let grid = generate_world(seed, &content);
            let mut world = World::new();
            spawn_colonists(&mut world, &grid, &content);

            sync_statuses(&mut world, &content);
            auto_assign_tasks(&mut world, &grid, &content);
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
        let content = test_content();
        use crate::world::generate_world;

        let grid = generate_world(42, &content);
        let mut world = World::new();
        spawn_colonists(&mut world, &grid, &content);

        let mut start_positions: HashMap<Entity, (i32, i32)> = HashMap::new();
        {
            let mut q = world.query::<(Entity, &Position, &Colonist)>();
            for (entity, pos, _) in q.iter(&world) {
                start_positions.insert(entity, pos.grid_cell());
            }
        }
        assert_eq!(start_positions.len(), 3, "expected 3 spawned colonists");

        for _ in 0..200 {
            sync_statuses(&mut world, &content);
            auto_assign_tasks(&mut world, &grid, &content);
            colonist_movement(&mut world, &grid, &content, 0.05);
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
                    let blocked_first = occupancy.get(&wp).filter(|&&e| e != entity).copied();
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

    #[test]
    fn eat_repaths_when_intermediate_waypoint_blocked() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));

        world.spawn((
            Colonist,
            Position { x: 7.0, y: 10.0 },
            Needs::new_full(&content),
            Task::default(),
            Path::default(),
        ));

        let eater = world
            .spawn((
                Colonist,
                Position { x: 6.0, y: 10.0 },
                Needs::new_full(&content),
                Task {
                    kind: TaskKind::Eat,
                    building_x: 10,
                    building_y: 10,
                    target_x: 9,
                    target_y: 10,
                },
                Path {
                    waypoints: vec![(7, 10), (8, 10), (9, 10)],
                    index: 0,
                },
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let path = world.get::<Path>(eater).unwrap();
        assert_eq!(world.get::<Task>(eater).unwrap().kind, TaskKind::Eat);
        assert_ne!(path.waypoints, vec![(7, 10), (8, 10), (9, 10)]);
        assert!(!path.waypoints.is_empty());
    }

    #[test]
    fn build_repaths_when_intermediate_waypoint_blocked() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();
        world.spawn((
            ConstructionSite {
                building_id: content.wall_building,
                work_remaining: 30.0,
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        world.spawn((
            Colonist,
            Position { x: 8.0, y: 10.0 },
            Needs::new_full(&content),
            Task::default(),
            Path::default(),
        ));

        let builder = world
            .spawn((
                Colonist,
                Position { x: 7.0, y: 10.0 },
                Needs::new_full(&content),
                Task {
                    kind: TaskKind::Build,
                    building_x: 10,
                    building_y: 10,
                    target_x: 9,
                    target_y: 10,
                },
                Path {
                    waypoints: vec![(8, 10), (9, 10)],
                    index: 0,
                },
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let path = world.get::<Path>(builder).unwrap();
        assert_eq!(world.get::<Task>(builder).unwrap().kind, TaskKind::Build);
        assert_ne!(path.waypoints, vec![(8, 10), (9, 10)]);
        assert!(!path.waypoints.is_empty());
    }

    #[test]
    fn sleep_repaths_when_intermediate_waypoint_blocked() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 12, 12, content.bed_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 12.0, y: 12.0 },
            BuildingKind(content.bed_building),
            BedOccupancy::default(),
        ));

        world.spawn((
            Colonist,
            Position { x: 11.0, y: 12.0 },
            Needs::new_full(&content),
            Task::default(),
            Path::default(),
        ));

        let sleeper = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 12.0 },
                Needs::new_full(&content),
                Task {
                    kind: TaskKind::Sleep,
                    building_x: 12,
                    building_y: 12,
                    target_x: 12,
                    target_y: 12,
                },
                Path {
                    waypoints: vec![(11, 12), (12, 12)],
                    index: 0,
                },
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let path = world.get::<Path>(sleeper).unwrap();
        assert_eq!(world.get::<Task>(sleeper).unwrap().kind, TaskKind::Sleep);
        assert_ne!(path.waypoints, vec![(11, 12), (12, 12)]);
        assert!(!path.waypoints.is_empty());
    }

    #[test]
    fn eat_task_cleared_when_repath_unreachable() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        for y in 0..WORLD_SIZE {
            grid.terrain[WorldGrid::index(8, y).unwrap()] = content.water_terrain;
        }
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        world.spawn((
            Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.berry_bush_building),
            BerrySupply::new(3),
        ));

        world.spawn((
            Colonist,
            Position { x: 6.0, y: 10.0 },
            Needs::new_full(&content),
            Task::default(),
            Path::default(),
        ));

        let eater = world
            .spawn((
                Colonist,
                Position { x: 5.0, y: 10.0 },
                Needs::new_full(&content),
                Task {
                    kind: TaskKind::Eat,
                    building_x: 10,
                    building_y: 10,
                    target_x: 9,
                    target_y: 10,
                },
                Path {
                    waypoints: vec![(6, 10)],
                    index: 0,
                },
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        assert_eq!(world.get::<Task>(eater).unwrap().kind, TaskKind::Idle);
    }

    #[test]
    fn eat_reassigns_when_goal_stand_occupied() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));
        assert!(grid.place_building(&content, 20, 10, content.berry_bush_building));

        let mut world = World::new();
        for &(bx, by) in &[(10, 10), (20, 10)] {
            world.spawn((
                Building,
                Position {
                    x: bx as f32,
                    y: by as f32,
                },
                BuildingKind(content.berry_bush_building),
                BerrySupply::new(3),
            ));
        }

        world.spawn((
            Colonist,
            Position { x: 9.0, y: 10.0 },
            Needs::new_full(&content),
            Task::default(),
            Path::default(),
        ));

        let eater = world
            .spawn((
                Colonist,
                Position { x: 8.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    100.0,
                ),
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

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(eater).unwrap();
        assert_eq!(task.kind, TaskKind::Eat);
        assert_ne!((task.target_x, task.target_y), (9, 10));
    }

    #[test]
    fn build_waits_when_goal_stand_occupied() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();
        world.spawn((
            ConstructionSite {
                building_id: content.wall_building,
                work_remaining: 30.0,
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        world.spawn((
            Colonist,
            Position { x: 9.0, y: 10.0 },
            Needs::new_full(&content),
            Task::default(),
            Path::default(),
        ));

        let builder = world
            .spawn((
                Colonist,
                Position { x: 8.0, y: 10.0 },
                Needs::new_full(&content),
                Task {
                    kind: TaskKind::Build,
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

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(builder).unwrap();
        assert_eq!(task.kind, TaskKind::Build);
        assert_eq!((task.target_x, task.target_y), (9, 10));
    }

    #[test]
    fn build_colonist_holds_position_at_stand() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        let mut world = World::new();
        world.spawn((
            ConstructionSite {
                building_id: content.wall_building,
                work_remaining: 500.0,
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        let builder = world
            .spawn((
                Colonist,
                Position { x: 9.0, y: 10.0 },
                Needs::new_full(&content),
                Task {
                    kind: TaskKind::Build,
                    building_x: 10,
                    building_y: 10,
                    target_x: 9,
                    target_y: 10,
                },
                Path::default(),
            ))
            .id();

        for _ in 0..30 {
            run_auto_assign(&mut world, &grid, &content);
            colonist_movement(&mut world, &grid, &content, 0.05);
            task_execution(&mut world, &mut grid, &content, 0.05);
        }

        let pos = world.get::<Position>(builder).unwrap();
        assert_eq!(pos.grid_cell(), (9, 10));
        assert_eq!(world.get::<Task>(builder).unwrap().kind, TaskKind::Build);
    }

    #[test]
    fn build_colonist_stable_at_high_sim_dt() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        let mut world = World::new();
        world.spawn((
            ConstructionSite {
                building_id: content.wall_building,
                work_remaining: 500.0,
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        let builder = world
            .spawn((
                Colonist,
                Position { x: 7.0, y: 10.0 },
                Needs::new_full(&content),
                Task {
                    kind: TaskKind::Build,
                    building_x: 10,
                    building_y: 10,
                    target_x: 9,
                    target_y: 10,
                },
                Path {
                    waypoints: vec![(8, 10), (9, 10)],
                    index: 0,
                },
            ))
            .id();

        // dt = 0.5 matches worker BASE_DT * 10x speed.
        for _ in 0..20 {
            run_auto_assign(&mut world, &grid, &content);
            colonist_movement(&mut world, &grid, &content, 0.5);
            task_execution(&mut world, &mut grid, &content, 0.5);
        }
        let pos = world.get::<Position>(builder).unwrap();
        assert_eq!(
            pos.grid_cell(),
            (9, 10),
            "builder should reach and hold stand at 10x dt"
        );
        assert_eq!(world.get::<Task>(builder).unwrap().kind, TaskKind::Build);
    }

    #[test]
    fn build_colonist_stays_on_arrival_cell_without_forced_repath() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        let mut world = World::new();
        world.spawn((
            ConstructionSite {
                building_id: content.wall_building,
                work_remaining: 500.0,
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        let builder = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 9.0 },
                Needs::new_full(&content),
                Task {
                    kind: TaskKind::Build,
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

        for _ in 0..30 {
            run_auto_assign(&mut world, &grid, &content);
            colonist_movement(&mut world, &grid, &content, 0.5);
            task_execution(&mut world, &mut grid, &content, 0.5);
        }

        let pos = world.get::<Position>(builder).unwrap();
        assert_eq!(
            pos.grid_cell(),
            (10, 9),
            "builder should stay on arrival cell, not walk to assigned stand"
        );
        assert_eq!(world.get::<Task>(builder).unwrap().kind, TaskKind::Build);
        let site = world
            .query::<&ConstructionSite>()
            .iter(&world)
            .next()
            .unwrap();
        assert!(
            site.work_remaining < 500.0,
            "builder should apply work while standing still"
        );
    }

    #[test]
    fn sleep_reassigns_when_goal_bed_occupied() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.bed_building));
        assert!(grid.place_building(&content, 20, 10, content.bed_building));

        let mut world = World::new();
        for &(bx, by) in &[(10, 10), (20, 10)] {
            world.spawn((
                Building,
                Position {
                    x: bx as f32,
                    y: by as f32,
                },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ));
        }

        world.spawn((
            Colonist,
            Position { x: 10.0, y: 10.0 },
            Needs::new_full(&content),
            Task::default(),
            Path::default(),
        ));

        let sleeper = world
            .spawn((
                Colonist,
                Position { x: 9.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    100.0,
                    content.need_def(content.sleep_need).critical_threshold - 1.0,
                ),
                Task {
                    kind: TaskKind::Sleep,
                    building_x: 10,
                    building_y: 10,
                    target_x: 10,
                    target_y: 10,
                },
                Path {
                    waypoints: vec![(10, 10)],
                    index: 0,
                },
            ))
            .id();

        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(sleeper).unwrap();
        assert_eq!(task.kind, TaskKind::Sleep);
        assert_eq!((task.target_x, task.target_y), (20, 10));
    }

    #[test]
    fn idle_wander_reassigned_when_first_step_blocked() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();

        world.spawn((Colonist, Position { x: 9.0, y: 10.0 }, Path::default()));

        let blocked = world
            .spawn((
                Colonist,
                Position { x: 10.0, y: 10.0 },
                Needs::new_full(&content),
                Task::default(),
                Path {
                    waypoints: vec![(9, 10)],
                    index: 0,
                },
            ))
            .id();

        for _ in 0..30 {
            run_auto_assign(&mut world, &grid, &content);
            colonist_movement(&mut world, &grid, &content, 0.05);
        }

        let path = world.get::<Path>(blocked).unwrap();
        let still_blocked =
            path.index < path.waypoints.len() && path.waypoints[path.index] == (9, 10);
        assert!(
            !still_blocked,
            "blocked idle wander should be cleared or retargeted, path: {:?}",
            path.waypoints
        );
    }

    #[test]
    fn colonist_path_through_bush_does_not_settle_on_bush() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        let colonist = world
            .spawn((
                Colonist,
                Position { x: 9.0, y: 10.0 },
                Path {
                    waypoints: vec![(10, 10), (11, 10)],
                    index: 0,
                },
            ))
            .id();

        for _ in 0..50 {
            colonist_movement(&mut world, &grid, &content, 0.1);
            let cell = world.get::<Position>(colonist).unwrap().grid_cell();
            assert_ne!(cell, (10, 10), "colonist must not settle on bush");
        }

        let final_cell = world.get::<Position>(colonist).unwrap().grid_cell();
        assert!(!is_blocks_settle_cell(&grid, &content, final_cell));
    }

    #[test]
    fn colonist_ejected_when_standing_on_bush() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.berry_bush_building));

        let mut world = World::new();
        let colonist = world
            .spawn((Colonist, Position { x: 10.0, y: 10.0 }, Path::default()))
            .id();

        colonist_movement(&mut world, &grid, &content, 0.05);

        assert_ne!(
            world.get::<Position>(colonist).unwrap().grid_cell(),
            (10, 10)
        );
    }

    #[test]
    fn wander_target_never_on_bush_cell() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        for &(x, y) in &[(15, 10), (15, 11), (15, 9), (14, 10), (16, 10)] {
            assert!(grid.place_building(&content, x, y, content.berry_bush_building));
        }

        let mut world = World::new();
        let colonist = world
            .spawn((Colonist, Position { x: 10.0, y: 10.0 }, Path::default()))
            .id();

        let occupancy = colonist_occupancy_map(&mut world);
        for _ in 0..32 {
            if let Some(waypoints) =
                pick_wander_target(&grid, &content, (10, 10), &occupancy, colonist)
            {
                let dest = waypoints.last().copied().unwrap();
                assert!(
                    !is_blocks_settle_cell(&grid, &content, dest),
                    "wander destination must not be a bush cell"
                );
            }
        }
    }

    #[test]
    fn overlapping_colonists_separated_after_movement() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();

        let a = world
            .spawn((Colonist, Position { x: 10.0, y: 10.0 }, Path::default()))
            .id();
        let b = world
            .spawn((Colonist, Position { x: 10.4, y: 10.4 }, Path::default()))
            .id();

        colonist_movement(&mut world, &grid, &content, 0.01);

        let cell_a = world.get::<Position>(a).unwrap().grid_cell();
        let cell_b = world.get::<Position>(b).unwrap().grid_cell();
        assert_ne!(cell_a, cell_b, "colonists must not share a grid cell");
    }

    #[test]
    fn deconstruct_instant_cancel_removes_zero_progress_site() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();
        world.spawn((
            ConstructionSite {
                building_id: content.wall_building,
                work_remaining: content.work_required(content.wall_building),
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        assert!(construction_site_at(&mut world, 10, 10).is_some());
        let site_entity = construction_site_at(&mut world, 10, 10).unwrap();
        let _ = world.despawn(site_entity);

        assert!(construction_site_at(&mut world, 10, 10).is_none());
        assert!(deconstruction_site_at(&mut world, 10, 10).is_none());
        assert!(grid.building_at(10, 10).is_none());
    }

    #[test]
    fn deconstruct_labor_removes_finished_wall() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.wall_building));

        let mut world = World::new();
        world.spawn((
            crate::components::Building,
            Position { x: 10.0, y: 10.0 },
            BuildingKind(content.wall_building),
        ));
        world.spawn((
            DeconstructionSite {
                building_id: content.wall_building,
                work_remaining: content.work_to_deconstruct(content.wall_building),
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 9.0, y: 10.0 },
                Needs::new_full(&content),
                Task {
                    kind: TaskKind::Deconstruct,
                    building_x: 10,
                    building_y: 10,
                    target_x: 9,
                    target_y: 10,
                },
                Path::default(),
            ))
            .id();

        let site_entity = deconstruction_site_at(&mut world, 10, 10).unwrap();
        if let Some(mut site) = world.get_mut::<DeconstructionSite>(site_entity) {
            site.reserved_by = Some(colonist);
        }

        for _ in 0..20 {
            task_execution(&mut world, &mut grid, &content, 1.0);
        }

        assert!(grid.building_at(10, 10).is_none());
        assert!(deconstruction_site_at(&mut world, 10, 10).is_none());
        assert_eq!(world.get::<Task>(colonist).unwrap().kind, TaskKind::Idle);
    }

    #[test]
    fn deconstruct_in_progress_site_replaces_construction_and_releases_builder() {
        let content = test_content();
        let _grid = grass_grid(&content);
        let mut world = World::new();
        let site_entity = world
            .spawn((
                ConstructionSite {
                    building_id: content.wall_building,
                    work_remaining: 20.0,
                    reserved_by: None,
                },
                Position { x: 10.0, y: 10.0 },
            ))
            .id();

        let builder = world
            .spawn((
                Colonist,
                Position { x: 9.0, y: 10.0 },
                Task {
                    kind: TaskKind::Build,
                    building_x: 10,
                    building_y: 10,
                    target_x: 9,
                    target_y: 10,
                },
                Path::default(),
            ))
            .id();

        if let Some(mut site) = world.get_mut::<ConstructionSite>(site_entity) {
            site.reserved_by = Some(builder);
        }

        let _ = world.despawn(site_entity);
        if let Some(mut task) = world.get_mut::<Task>(builder) {
            task.kind = TaskKind::Idle;
        }
        world.spawn((
            DeconstructionSite {
                building_id: content.wall_building,
                work_remaining: content.work_to_deconstruct(content.wall_building),
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        assert!(construction_site_at(&mut world, 10, 10).is_none());
        assert!(deconstruction_site_at(&mut world, 10, 10).is_some());
        assert_eq!(world.get::<Task>(builder).unwrap().kind, TaskKind::Idle);
    }

    #[test]
    fn deconstruct_bed_not_assigned_while_occupied() {
        let content = test_content();
        let mut grid = grass_grid(&content);
        assert!(grid.place_building(&content, 10, 10, content.bed_building));

        let mut world = World::new();
        let bed = world
            .spawn((
                Building,
                Position { x: 10.0, y: 10.0 },
                BuildingKind(content.bed_building),
                BedOccupancy::default(),
            ))
            .id();

        world.spawn((
            DeconstructionSite {
                building_id: content.bed_building,
                work_remaining: content.work_to_deconstruct(content.bed_building),
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        let sleeper = spawn_sleeping_on_bed(&mut world, &content, bed, 10, 10);
        let _ = sleeper;

        let idle = spawn_colonist_at(&mut world, &content, 5, 10);
        run_auto_assign(&mut world, &grid, &content);

        assert_eq!(world.get::<Task>(idle).unwrap().kind, TaskKind::Idle);

        if let Some(mut occ) = world.get_mut::<BedOccupancy>(bed) {
            occ.reserved_by = None;
        }
        if let Some(mut task) = world.get_mut::<Task>(sleeper) {
            task.kind = TaskKind::Idle;
        }
        if let Some(mut pos) = world.get_mut::<Position>(sleeper) {
            pos.x = 11.0;
            pos.y = 10.0;
        }

        run_auto_assign(&mut world, &grid, &content);
        assert_eq!(world.get::<Task>(idle).unwrap().kind, TaskKind::Deconstruct);
    }

    #[test]
    fn nearest_job_picks_closer_deconstruct_over_farther_build() {
        let content = test_content();
        let grid = grass_grid(&content);
        let mut world = World::new();

        world.spawn((
            ConstructionSite {
                building_id: content.wall_building,
                work_remaining: 30.0,
                reserved_by: None,
            },
            Position { x: 20.0, y: 10.0 },
        ));
        world.spawn((
            DeconstructionSite {
                building_id: content.wall_building,
                work_remaining: 15.0,
                reserved_by: None,
            },
            Position { x: 11.0, y: 10.0 },
        ));

        let colonist = spawn_colonist_at(&mut world, &content, 10, 10);
        run_auto_assign(&mut world, &grid, &content);

        let task = world.get::<Task>(colonist).unwrap();
        assert_eq!(task.kind, TaskKind::Deconstruct);
        assert_eq!((task.building_x, task.building_y), (11, 10));
    }

    #[test]
    fn hungry_colonist_preempts_deconstruct_task() {
        let content = test_content();
        let _grid = grass_grid(&content);
        let mut world = World::new();

        world.spawn((
            DeconstructionSite {
                building_id: content.wall_building,
                work_remaining: 15.0,
                reserved_by: None,
            },
            Position { x: 10.0, y: 10.0 },
        ));

        let colonist = world
            .spawn((
                Colonist,
                Position { x: 9.0, y: 10.0 },
                Needs::with_values(
                    &content,
                    content.need_def(content.food_need).critical_threshold - 1.0,
                    100.0,
                ),
                ActiveStatuses::default(),
                Task {
                    kind: TaskKind::Deconstruct,
                    building_x: 10,
                    building_y: 10,
                    target_x: 9,
                    target_y: 10,
                },
                Path::default(),
            ))
            .id();

        let site_entity = deconstruction_site_at(&mut world, 10, 10).unwrap();
        if let Some(mut site) = world.get_mut::<DeconstructionSite>(site_entity) {
            site.reserved_by = Some(colonist);
        }

        sync_statuses(&mut world, &content);
        preempt_labor_for_critical_needs(&mut world, &content);

        assert_ne!(
            world.get::<Task>(colonist).unwrap().kind,
            TaskKind::Deconstruct
        );
        if let Some(site) = world.get::<DeconstructionSite>(site_entity) {
            assert!(site.reserved_by.is_none());
        }
    }
}
