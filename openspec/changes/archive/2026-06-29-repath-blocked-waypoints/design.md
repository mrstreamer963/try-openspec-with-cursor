## Context

Cell occupancy blocks waypoint snaps in `colonist_movement`. The original `colonist-cell-occupancy` change deferred mid-route replanning; idle wander later added path clearing only for `TaskKind::Idle`.

## Goals / Non-Goals

**Goals:**

- Recalculate waypoints on the assignment pass when the next waypoint is blocked, for Eat, Build, Sleep, and Idle
- Avoid infinite repath loops when the goal stand/bed is occupied (same route → wait)
- Clear tasks when no route exists to the target

**Non-Goals:**

- Making berry bushes or beds block movement (separate follow-up)
- Reassigning alternate stand tiles when the goal cell is permanently occupied — **updated:** goal-stand occupied now clears task for same-pass reassignment

## Decisions

### Repath on assignment pass, before movement

Same hook as idle wander: `auto_assign_tasks` calls `repath_blocked_colonists` after `release_stuck_tasks`. Movement runs later in the tick with updated waypoints.

### Idle vs active tasks

- **Idle**: `path.clear()` — existing wander/vacate reassignment on the same pass
- **Eat / Build / Sleep**: `find_path_for_colonist` from current cell to `(task.target_x, task.target_y)`; update waypoints only if the trimmed route differs from remaining waypoints
- **Blocked on goal waypoint**: clear task immediately; Eat adds `prefer_sleep_first`; same assignment pass re-runs `nearest_eat_assignment`, `nearest_build_assignment`, or `nearest_free_bed`

### Unreachable target

Mirror `release_unreachable_eat_tasks` / `release_stuck_tasks`: `clear_task` plus `prefer_sleep_first` for failed Eat repaths.

### Occupied stands at assignment time

`nearest_build_assignment` and `nearest_free_bed` now skip tiles occupied by other colonists (matching eat stand filtering).

## Risks / Trade-offs

- **[Goal occupied]** Repath cannot free a stand occupied by another colonist → colonist waits. Mitigation: acceptable per occupancy spec; bush/bed blocking is a separate fix.
- **[Per-tick cost]** Repath runs only for colonists blocked on the next waypoint, not every colonist every tick.
