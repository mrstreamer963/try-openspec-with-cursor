## Context

Task assignment currently sets `target_x/y` to the building tile for both Eat and Sleep. Colonists pathfind directly onto BerryBush and Bed tiles. Berry bushes and beds do not block movement (`blocks_movement` is Wall-only), so colonists visually overlap buildings. There is no bed occupancy tracking — all idle colonists with low Sleep can target the same nearest bed.

## Goals / Non-Goals

**Goals:**
- Eat interaction from an orthogonally adjacent stand tile; colonist never occupies the bush tile
- Sleep interaction on the bed tile; at most one colonist per bed at a time
- Reserve bed at task assignment to prevent races
- Reuse existing `BerrySupply` depletion and bush removal logic

**Non-Goals:**
- Reserving adjacent stand tiles for eating (multiple colonists may share a stand tile in v1)
- Multi-tick sleep animation or duration
- Snapshot field for bed occupancy (optional later)
- Diagonal adjacency for eating (orthogonal only)

## Decisions

### 1. Split Task into building + stand coordinates

Extend `Task` with `building_x`, `building_y` (interaction target) and repurpose `target_x`, `target_y` as the stand tile (path destination).

| Task kind | `building_x/y` | `target_x/y` (stand) |
|-----------|----------------|----------------------|
| Eat       | bush tile      | adjacent walkable tile |
| Sleep     | bed tile       | bed tile (same) |
| Idle      | unused (0)     | unused (0) |

**Rationale:** Single struct covers both patterns without a separate interaction-mode enum.

### 2. Adjacent stand tile selection for Eat

Helper `best_adjacent_stand(grid, building, from)` returns the orthogonally adjacent walkable cell with the shortest path from the colonist. If none exists, Eat task is not assigned (same as no path).

**Alternatives considered:**
- *Nearest by Manhattan distance only* — simpler but may pick unreachable cells; path-length tie-break is better
- *Fixed preference order (N,E,S,W)* — arbitrary and suboptimal paths

### 3. Eat execution checks adjacency, not co-location

`task_execution` for Eat: colonist at stand tile AND Manhattan distance 1 from `building_x/y` AND bush has berries. Bush lookup uses building coords, not colonist cell.

### 4. BedOccupancy component on Bed entities

```rust
#[derive(Component)]
pub struct BedOccupancy {
    pub reserved_by: Option<Entity>,
}
```

Spawned on every Bed. Set `reserved_by` when Sleep task is assigned; cleared when sleep completes or task is cancelled.

**Reservation at assign time** (not arrival): prevents two colonists pathing to the same bed.

**Alternatives considered:**
- *First-come at arrival* — race on same tick when paths complete together
- *Global occupancy map on WorldGrid* — duplicates ECS entity model

### 5. Sleep task assignment filters occupied beds

`auto_assign_tasks` excludes beds where `BedOccupancy.reserved_by.is_some()`. Second colonist picks next nearest free bed or stays idle until next tick.

### 6. Release reservation on task clear

Whenever a colonist's task returns to Idle (sleep complete, path failure, or future cancellation), release the bed if this colonist held the reservation.

## Risks / Trade-offs

- **[Stand tile shared by multiple eaters]** → Acceptable for v1; bush has 3 berries
- **[Bed reserved while colonist walks]** → Other colonists skip that bed; may walk farther — intended
- **[Bush removed while colonist paths to stand]** → Existing depleted-bush fail path applies; check building still exists at execution
- **[No adjacent stand tile]** → Bush unreachable for eating until terrain changes — edge case, no task assigned

## Migration Plan

No data migration. Existing saves are out of scope. Deploy as game-core logic change only.

## Open Questions

_(none — bed on-tile confirmed by user)_
