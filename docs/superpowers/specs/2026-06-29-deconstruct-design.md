# Deconstruct — Design

## Context

Construction is unit-based: the player designates a tile, a `ConstructionSite` is spawned, and colonists with `TaskKind::Build` work until the building is complete. There is no way to remove player-placed structures except berry bush auto-removal when depleted.

This change adds **deconstruct** — the inverse flow — with a hybrid instant/labor model and a dedicated toolbar tool.

## Goals

- Player can remove construction sites and finished buildings via a **Deconstruct** toolbar button
- Construction sites at **0% progress** are cancelled instantly (no colonist labor)
- All other targets require colonist labor using per-building `work_to_deconstruct` from YAML
- Deconstruct orders can be placed while a target is in use; assignment waits until the target is free
- Idle colonists pick the **nearest** available Build or Deconstruct job (no fixed priority between them)
- Snapshots expose pending deconstruction sites for client rendering
- Save/load preserves deconstruction state

## Non-Goals

- Resource or material refunds on deconstruct (v1 remains resource-free)
- Cancelling a pending deconstruct order from UI
- Partial refund proportional to construction progress
- Deconstruct during simulation pause (paused = no progress, same as build)
- Diagonal stand tiles (colonist uses orthogonal adjacent cells, same as build)

## Architecture

Mirror the existing `ConstructionSite` / `TaskKind::Build` pattern:

| Build | Deconstruct |
|---|---|
| `ConstructionSite` | `DeconstructionSite` |
| `TaskKind::Build` | `TaskKind::Deconstruct` |
| `IncomingEvent::Build` | `IncomingEvent::Deconstruct` |
| `ConstructionSiteSnapshot` | `DeconstructionSiteSnapshot` |
| `work_required` (YAML) | `work_to_deconstruct` (YAML) |

**Rationale:** Consistent with current ECS patterns; reuses reservation, pathfinding, preemption, and snapshot infrastructure.

## Player Flow

1. Player clicks **🔨 Deconstruct** in the toolbar (mutually exclusive with Select and build modes).
2. Player clicks a tile with a construction site or finished building.
3. Client sends `IncomingEvent::Deconstruct { x, y }`.
4. Simulation applies rules (see Command Handling).
5. For labor-based removal, a `DeconstructionSite` appears in snapshots with a red overlay; colonists work until the target is gone.

## YAML Content

Add `work_to_deconstruct` to each buildable building in `content/base/buildings.yaml`:

| Building | `work_required` | `work_to_deconstruct` (initial values) |
|---|---|---|
| wall | 30 | 15 |
| bed | 50 | 25 |
| berry_bush | 40 | 20 |

Parsed in `ContentRegistry` with accessor `work_to_deconstruct(id: BuildingId) -> f32`.

## Command Handling

`IncomingEvent::Deconstruct { x, y }`:

| Tile state | Result |
|---|---|
| Empty tile | Reject silently (same as invalid build) |
| `DeconstructionSite` already present | Reject (duplicate order) |
| `ConstructionSite` with `progress == 0` | **Instant:** despawn site; release builder `reserved_by` if any |
| `ConstructionSite` with `progress > 0` | Release builder reservation; despawn `ConstructionSite`; spawn `DeconstructionSite` with `work_remaining = work_to_deconstruct` |
| Finished building on grid | Spawn `DeconstructionSite` with `work_remaining = work_to_deconstruct`; building remains on grid until complete |

Progress for a construction site: `1 - work_remaining / work_required` (same formula as snapshots today). Instant cancel applies only when `progress == 0`.

## Target Availability (assignment gate)

A `DeconstructionSite` is **assignable** only when its target tile passes a free check. The order exists immediately; colonists skip unassignable sites until they become free.

| Target | Free when |
|---|---|
| Wall | Always |
| Construction site (being removed) | No colonist has `Build` reserved on or actively working at this site |
| Bed | No colonist on the bed tile; no `Sleep` reservation on this bed |
| Berry bush | No colonist with `Eat` task targeting this bush |

While waiting for availability, the building or site remains **fully functional** (eat, sleep, and build interactions continue).

Once a colonist is assigned and reaches an adjacent stand tile, deconstruct work proceeds each tick until complete.

## Task Assignment

In `auto_assign_tasks`, after critical-need handling, for each idle colonist without active need statuses:

1. Collect unreserved `ConstructionSite` entities (existing logic).
2. Collect unreserved `DeconstructionSite` entities that pass the availability gate.
3. Merge into a single pool of `(site_entity, target_x, target_y, stand_x, stand_y, task_kind)` candidates.
4. Pick the **nearest** reachable candidate (Manhattan distance to stand tile, same as build).
5. Reserve site and assign `TaskKind::Build` or `TaskKind::Deconstruct`.

Critical needs **preempt** Deconstruct the same way they preempt Build (clear task, release site reservation).

## Task Execution

Deconstruct execution mirrors Build:

- Colonist paths to an orthogonally adjacent stand cell.
- When adjacent with no remaining path waypoints, colonist is locked in place.
- Each tick: `work_remaining -= BUILD_WORK_PER_TICK`.
- On completion: `complete_deconstruction` removes the building from `WorldGrid` (if present), despawns the building ECS entity, despawns `DeconstructionSite`, clears colonist task to Idle.

If the site or building vanishes before completion (e.g. berry bush depleted by eating), cancel the deconstruct task and release reservation.

## Data Model

### Rust (`packages/game-core`)

```rust
#[derive(Component, Clone, Copy, Debug)]
pub struct DeconstructionSite {
    pub building_id: BuildingId,
    pub work_remaining: f32,
    pub reserved_by: Option<Entity>,
}

pub enum TaskKind {
    Idle,
    Eat,
    Sleep,
    Build,
    Deconstruct,
}
```

### Events / Snapshots

```rust
IncomingEvent::Deconstruct { x: i32, y: i32 }

pub struct DeconstructionSiteSnapshot {
    pub x: i32,
    pub y: i32,
    pub building: String,
    pub progress: f32, // 0.0–1.0, increases as work is applied
}
```

`StateSnapshot` gains `deconstruction_sites: Vec<DeconstructionSiteSnapshot>`.

Progress: `1.0 - work_remaining / work_to_deconstruct(building_id)`.

### Client (`packages/client`)

- `ToolMode`: `'select' | 'deconstruct' | BuildingId` (or equivalent boolean + build mode).
- `TaskKind` includes `'Deconstruct'`.
- `IncomingEvent` includes `{ type: 'deconstruct'; x: number; y: number }`.
- `BuildingDef` includes `work_to_deconstruct: number`.

## View Layer

### Toolbar

Add **🔨 Deconstruct** button to `Toolbar.vue`. Active state is mutually exclusive with Select and build-type buttons.

### Click handling

In `App.vue`, when deconstruct mode is active, tile clicks send `deconstruct` events instead of `build`.

### Rendering (`PixiRenderer`)

For each `deconstruction_sites` entry:

- Draw a semi-transparent **red** overlay on the tile (alpha scales with progress, similar to construction ghost).
- Draw a **red** progress bar above the tile.
- Finished buildings remain visible underneath until deconstruction completes.

Construction site ghosts remain green; deconstruction overlays are red — visually distinct.

## Edge Cases

| Scenario | Behavior |
|---|---|
| Deconstruct bed while colonist sleeping | Order created; assignment waits until colonist leaves bed |
| Deconstruct bush while colonist eating | Order created; assignment waits until eat task ends |
| Deconstruct site while colonist building | Order created (after despawn of `ConstructionSite` for progress > 0); assignment waits until builder finishes or is released |
| Builder on 0% site when player deconstructs | Site removed instantly; builder's task cleared, reservation released |
| Berry bush depletes while deconstruct pending | Bush removed by eat logic; `DeconstructionSite` despawned or task cancelled |
| No path to deconstruct stand | Task cancelled, reservation released, colonist returns to idle |
| Load game with pending deconstruct | Sites restored from snapshot; assignment resumes |

## Testing

### Game-core unit tests

- Instant cancel: 0% construction site removed on deconstruct command; no `DeconstructionSite` spawned
- Labor path: finished wall gets `DeconstructionSite`; colonist completes removal
- In-progress site: `ConstructionSite` replaced by `DeconstructionSite`; builder reservation released
- Availability gate: deconstruct not assigned while bed reserved for sleep; assigned after reservation clears
- Nearest job: colonist picks closer deconstruct over farther build (and vice versa)
- Preemption: hungry colonist abandons deconstruct task
- Snapshot round-trip: deconstruction sites survive save/load

### Client (manual)

- Deconstruct toolbar toggles mode; clicks send correct event
- Red overlay and progress bar render on deconstruction sites
- Building disappears from view when snapshot no longer includes it

## Affected Specs

Delta updates needed in OpenSpec:

- `world-simulation` — deconstruct command, instant cancel, completion
- `colonist-simulation` — `TaskKind::Deconstruct`, assignment, execution, preemption
- `view-layer` — toolbar, rendering, click events
- `worker-bridge` — new event and snapshot fields
- `content-definitions` (if present) — `work_to_deconstruct` field

## Summary of Decisions

| Question | Decision |
|---|---|
| Labor model | Hybrid: 0% sites instant; rest via colonists |
| UI | Separate Deconstruct toolbar button |
| Work amount | `work_to_deconstruct` per building in YAML |
| Target in use | Allow order; assign only when free |
| Build vs Deconstruct priority | Nearest available job |
