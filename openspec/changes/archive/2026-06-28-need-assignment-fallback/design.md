## Context

`auto_assign_tasks` picks at most one critical need per colonist using a rigid `if food else if sleep` chain. When Food is critical, it only attempts Eat; if no BerryBush is satisfiable (no bushes, depleted, no stand tile, or no path), the colonist gets no task — Sleep is never considered even when also critical and a free Bed exists.

This surfaced in play: colonists with food=0 and sleep=0 ignore an empty bed and stay idle or keep wandering. `idle-wander` added ambient movement for satisfied colonists but did not address unsatisfiable primary needs.

## Goals / Non-Goals

**Goals:**
- Try critical needs in priority order (Food, then Sleep); assign the first need that has a satisfiable target and valid path
- Preserve Food-over-Sleep priority when both are satisfiable
- Replace wander path when any need assignment succeeds (Eat or Sleep)
- Cover fallback with unit tests

**Non-Goals:**
- New task types, components, or snapshot fields
- Sleep-over-Food priority when both are satisfiable
- Multi-need queuing (assign Eat now, Sleep after Eat completes — existing single-task model stays)
- Fallback from Build to needs (already handled by `preempt_build_for_critical_needs`)
- Client changes

## Decisions

### 1. Priority-ordered try loop

Replace the single `need_kind` branch with an ordered list of critical needs: `[Food, Sleep]`. For each need below threshold, attempt assignment (target lookup + pathfinding). Use the first success; stop.

**Rationale:** Minimal change to existing helpers (`nearest_eat_assignment`, `nearest_free_bed`). Matches RimWorld-style hunger priority while fixing the deadlock when food is unavailable.

**Alternatives considered:**
- *Separate fallback only Food→Sleep* — simpler but misses symmetric Sleep→Food when no bed exists
- *Score by need severity (lowest value wins)* — different semantics; hunger no longer strictly first when both satisfiable

### 2. "Satisfiable" means target + path

A need is assignable only when:
- Eat: `nearest_eat_assignment` returns a bush + stand tile **and** `find_path` succeeds
- Sleep: `nearest_free_bed` returns a bed **and** `find_path` succeeds

Same bar as today for a successful assignment; the change is trying the next need when the first fails any step.

**Rationale:** Consistent with existing no-path cancellation behavior; avoids assigning unreachable tasks.

### 3. Wander preemption via path overwrite

When a need assignment succeeds, populate `Path` with task waypoints (existing behavior). No separate wander-clear step.

When all critical needs fail assignment, `continue` without touching path — colonist may keep wandering. Acceptable for v1; only the success path must replace wander.

**Rationale:** Eat/Sleep assignment already overwrites `Path`; explicit clear is redundant when assignment succeeds.

### 4. Extract `try_need_assignment` helper (optional refactor)

Small helper in `systems.rs`:

```
try_assign_need(world, grid, entity, from, need, &context) -> Option<PendingAssignment>
```

Keeps the colonist loop readable and mirrors test setup.

**Alternatives considered:**
- *Inline loop only* — fine for ~15 lines; helper preferred for tests and readability

## Risks / Trade-offs

- **[Sleep while starving]** → Colonist may sleep when food is unavailable; intentional fallback, not a bug
- **[Both needs unsatisfiable]** → Colonist stays idle/wandering with both at zero; no change from current worst case
- **[Per-tick retry cost]** → Two pathfinding calls worst case per idle colonist per tick; negligible at 3 colonists
- **[idle-wander interaction]** → Wander preempt scenario in idle-wander assumed need always assignable; this change makes that true for Sleep fallback

## Migration Plan

Game-core only. Rebuild WASM. No save migration or client changes.

## Open Questions

_(none)_
