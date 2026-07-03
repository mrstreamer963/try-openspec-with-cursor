## Context

The `IncomingEvent::Build` handler in `game.rs` immediately calls `WorldGrid::place_building` and spawns a finished `Building` ECS entity. The client sends the same event on toolbar click. There is no intermediate state — colonists never participate in construction.

## Goals / Non-Goals

**Goals:**
- Player click creates a construction order (blueprint) at a valid tile
- Colonists path to the site and apply work over time until the building is complete
- Critical need tasks (Eat/Sleep) take priority over Build
- Completed buildings behave exactly as today (Bed sleep, BerryBush food, Wall blocking)
- Snapshots expose pending sites with progress for client rendering

**Non-Goals:**
- Resource/material costs or hauling
- Construction skill, quality, or multiple workers per site
- Cancelling or reordering construction queues from UI
- Construction during pause (simulation paused = no progress)
- Diagonal stand tiles for building (colonist paths to the site tile itself)

## Decisions

### 1. ConstructionSite as ECS entity (not grid slot)

Spawn a `ConstructionSite` entity on valid build orders with fields:
```rust
#[derive(Component)]
pub struct ConstructionSite {
    pub building_type: BuildingType,
    pub work_remaining: f32,  // ticks of work left
}
```

Position component holds the tile coords. Site does **not** occupy `WorldGrid.buildings` until complete.

**Rationale:** Keeps walkability unchanged during construction; colonists can stand on the tile. Matches ECS patterns used for buildings/colonists.

**Alternatives considered:**
- *Grid flag `buildings[i] = Some(UnderConstruction)* — blocks movement for walls prematurely; harder to track progress
- *Separate `construction: Vec<Option<...>>` on WorldGrid* — duplicates entity model

### 2. Fixed work amount per building type

| Building   | Work (simulation ticks at 1×) |
|------------|-------------------------------|
| Wall       | 30                            |
| Bed        | 50                            |
| BerryBush  | 40                            |

Each tick while a colonist is on-site with an active Build task, subtract `BUILD_WORK_PER_TICK` (e.g. 1.0) from `work_remaining`. At 0, finalize.

**Rationale:** Simple, predictable; scales with game speed multiplier automatically via tick count.

### 3. TaskKind::Build added to colonist tasks

Extend `TaskKind` with `Build`. `Task` stores `building_x/y` and `target_x/y` both as the construction tile (same coords).

Assignment in `auto_assign_tasks`:
1. Satisfy critical Eat/Sleep first (existing logic)
2. For idle colonists, assign nearest unassigned `ConstructionSite` (no colonist currently building it)

Track assignment with `ConstructionSite.reserved_by: Option<Entity>` (mirrors bed occupancy pattern).

### 4. Build event creates site, not building

`IncomingEvent::Build` handler:
- Validate: walkable, no existing building, no existing construction site at tile
- Spawn `ConstructionSite` entity; do **not** call `place_building`

On completion:
- `grid.place_building(x, y, building_type)`
- Spawn finished `Building` entity (with `BerrySupply` for bushes)
- Despawn `ConstructionSite`

### 5. Snapshot: separate `construction_sites` array

```rust
pub struct ConstructionSiteSnapshot {
    pub x: i32,
    pub y: i32,
    pub building: BuildingType,
    pub progress: f32,  // 0.0..1.0 for UI
}
```

Add to `StateSnapshot`. Client renders semi-transparent ghost on buildings layer.

### 6. Client: ghost rendering only

Toolbar and click flow unchanged — still sends `build` event. PixiJS draws construction sites with reduced alpha and optional progress tint. No new UI panels.

## Risks / Trade-offs

- **[One colonist per site]** → Large build queues wait; acceptable for v1 with 3 colonists
- **[Site reserved while colonist walks]** → Other colonists skip; player sees idle colonists near pending sites — intended
- **[Build vs needs priority]** → Hungry colonists abandon build mid-progress if needs become critical; work pauses until reassigned
- **[Wall site walkable]** → Colonist stands on tile during build; wall blocks only after completion

## Migration Plan

No persistence. Replace instant-build path in `dispatch`. Existing `Build` event shape unchanged — behavior change only.

## Open Questions

_(none — stand-on-tile construction confirmed as simplest v1 approach)_
