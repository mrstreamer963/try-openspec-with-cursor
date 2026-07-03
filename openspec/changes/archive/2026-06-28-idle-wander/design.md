## Context

Idle colonists with satisfied needs and no construction orders currently have an empty `Path` and do not move. `auto_assign_tasks` only assigns Eat, Sleep, or Build — when none apply, the colonist stands still indefinitely. `TaskKind::Idle` already exists; movement is driven entirely by the `Path` component and `colonist_movement`.

Cell occupancy is enforced at waypoint snap (one colonist per settled cell). Wander should respect occupied cells when picking targets so colonists naturally vacate tiles they no longer need.

## Goals / Non-Goals

**Goals:**
- Idle colonists walk to random nearby walkable cells when no Eat, Sleep, or Build task applies
- On arrival, assign a new wander destination on the next assignment pass
- Eat, Sleep, and Build preempt wander immediately
- Reuse existing `Path`, `find_path`, and `colonist_movement` — no new task type or snapshot fields
- Exclude occupied cells and the colonist's current cell from wander target candidates

**Non-Goals:**
- New `TaskKind::Wander` or client/snapshot changes
- Pauses or idle animations at wander destinations
- Dynamic mid-route replanning when blocked (wait at snap, same as other movement)
- Treating colonists as pathfinding obstacles

## Decisions

### 1. Idle + Path (no new task type)

Wander is expressed as `TaskKind::Idle` with a non-empty `Path`. `task_execution` already ignores Idle; no changes there.

**Rationale:** Wander is ambient behavior, not a work task. Snapshot stays `Idle`; client renders movement normally.

**Alternatives considered:**
- *TaskKind::Wander* — clearer semantics but unnecessary protocol and UI churn
- *Separate WanderTarget component* — extra state to sync with Path

### 2. Assignment in `auto_assign_tasks`

After the existing need/build branches fail for an idle colonist, if `path.index >= path.waypoints.len()` (no active path), pick a wander target and populate `Path`.

**Rationale:** Single assignment pass per tick; same place that preempts wander when needs/build match.

**Alternatives considered:**
- *Dedicated `idle_wander` system* — cleaner SRP but redundant for 3 colonists

### 3. Random nearby cell selection

Add `WANDER_RADIUS` (default 5, Manhattan distance) in `world.rs`. Collect all walkable cells within radius, excluding:
- the colonist's current grid cell
- cells occupied by another colonist's settled position (reuse occupancy helper from cell-occupancy work)

Shuffle or pick uniformly at random with `getrandom` (same pattern as name assignment). Try up to `WANDER_PICK_ATTEMPTS` (e.g. 8) candidates until `find_path` succeeds.

**Rationale:** Predictable local movement; retries handle unreachable picks without expensive full-map search.

**Alternatives considered:**
- *Random angle + distance* — can land on same cell or off-grid more often
- *Always nearest free cell* — too deterministic, looks robotic

### 4. Preemption

When assigning Eat, Sleep, or Build to an idle colonist, overwrite `Path` with the task path (existing behavior). No explicit wander cancellation needed.

When `clear_task` runs (task complete), path is cleared; next `auto_assign_tasks` tick starts wander if still idle.

### 5. Blocked arrival

If wander destination is occupied at snap time, existing movement wait logic applies — colonist does not advance `path.index`. On subsequent ticks, if path is still incomplete, do not reassign. If path completes but colonist is not on target (shouldn't happen with wait), or after prolonged block, next assignment pass when `path.index >= waypoints.len()` picks a new target.

**Rationale:** Consistent with global occupancy rules; no special wander replanning.

## Risks / Trade-offs

- **[Fidgety movement at radius edge]** → Small radius (5) and random picks keep motion local; acceptable for v1
- **[All nearby cells occupied]** → Retries may fail; colonist stands until a cell frees or radius effectively expands on retry next tick
- **[Wander vs build race]** → Build assignment runs in same pass before wander branch; build always wins
- **[RNG in WASM tests]** → Tests can seed via deterministic grid setup or mock helper returning fixed cells

## Migration Plan

No data migration. Game-core only. Independent of client rebuild beyond normal WASM refresh.

## Open Questions

_(none — defaults: radius 5, no pause at destination, immediate new target on arrival)_
