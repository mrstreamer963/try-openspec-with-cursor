## Context

`colonist_movement` moves each colonist independently toward path waypoints and snaps to integer cells when within step distance. There is no check for other colonists on the target cell. Pathfinding (`find_path`) only considers terrain and buildings — not other colonists. This is intentional for pass-through behavior during movement.

`building-interaction-rules` (when applied) paths Eat tasks to adjacent stand tiles and Sleep tasks to bed tiles with `BedOccupancy` reservation. Cell occupancy is a separate, global layer: one colonist per settled cell, including eat stand tiles and idle colonists after eating.

## Goals / Non-Goals

**Goals:**
- Enforce at most one colonist per grid cell when a movement step completes (waypoint snap)
- Allow visual/interpolated pass-through during movement; do not block A* with colonist positions
- Wait (do not snap, do not advance path index) when target cell is occupied by another colonist
- Prefer unoccupied stand tiles when assigning Eat tasks; skip stands occupied by settled colonists or already chosen in the same assign batch

**Non-Goals:**
- Dynamic path replanning when blocked mid-route
- Treating colonists as pathfinding obstacles
- Stand-tile reservation component (cell occupancy + assign-time filtering is sufficient)
- Post-eat "step aside" behavior for idle colonists blocking a stand
- Client or snapshot changes

## Decisions

### 1. Occupancy derived from settled positions (no new component)

Build a per-tick occupancy map from all colonists' `Position::grid_cell()`. A colonist is considered settled on its current grid cell. No persistent `CellOccupancy` ECS component.

**Rationale:** Occupancy is always derivable from positions; avoids sync bugs between component and float coords.

**Alternatives considered:**
- *WorldGrid colonist layer* — duplicates ECS; harder to keep in sync with smooth movement
- *Component on each cell* — overkill for 3 colonists

### 2. Two-phase movement per tick

Refactor `colonist_movement` into:

1. **Intent pass** — for each moving colonist, compute whether this tick would complete a waypoint snap
2. **Apply pass** — process snap intents in stable entity order; allow snap only if target cell is free (or already held by self); otherwise skip snap and continue interpolating next tick

Non-snap interpolation (partial step toward waypoint) proceeds without occupancy checks — pass-through.

**Rationale:** Resolves same-tick races deterministically; first claimant wins, second waits.

**Alternatives considered:**
- *Per-colonist check inside single loop* — order-dependent without two-phase commit
- *Slide to adjacent free cell* — breaks task destination accuracy

### 3. Wait, don't replan

When snap is blocked, colonist stays at current float position (may be visually adjacent to occupied cell) and does not increment `path.index`. Task execution still requires arrival at exact `target_x/y`.

**Rationale:** Minimal change; natural queuing at chokepoints and single stand tiles.

### 4. Eat assignment filters occupied stands

Extend stand selection (in `nearest_eat_assignment` / `best_adjacent_stand`):

- Exclude cells where another colonist's `grid_cell()` equals the stand
- Exclude stands already assigned to another colonist in the current `auto_assign_tasks` batch (same pattern as `reserved_beds`)

If no free stand exists for a bush, try the next bush. No Eat task if no bush has a free reachable stand.

**Rationale:** Reduces wasted pathing; aligns with global no-sharing rule without a separate reservation system.

### 5. Pathfinding unchanged

`find_path` does not receive colonist occupancy. Colonists may route through cells currently occupied by idle colonists (they pass through during movement).

## Risks / Trade-offs

- **[Idle colonist blocks eat stand after eating]** → Acceptable for v1; colonist leaves when assigned a new task. Mitigate later with wander/step-aside if needed
- **[Swap deadlock A↔B]** → Rare on 50×50 with 3 colonists; stable ordering + wait resolves in 1–2 ticks
- **[Visual overlap during lerp]** → Expected pass-through behavior; disappears on snap
- **[Assign batch vs movement race]** → Batch reserved stands reduce but don't eliminate; movement wait handles remainder

## Migration Plan

No data migration. Game-core only. Depends on `building-interaction-rules` eat stand coordinates being in place (or compatible `Task.target_x/y` as path destination).

## Open Questions

_(none)_
