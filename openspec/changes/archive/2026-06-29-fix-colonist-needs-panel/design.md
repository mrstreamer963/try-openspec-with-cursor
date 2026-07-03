## Context

The colonist info panel (`ColonistInfo.vue`) renders Food and Sleep as progress bars with rounded numeric values. The simulation already tracks critical needs via `Hungry` and `WantsSleep` ECS components (synced from `Needs` values below `NEED_THRESHOLD = 30.0` in `update_need_buffs`), but the snapshot builder only serializes raw `food` and `sleep` floats. The client has no way to show "Hungry" or "Wants sleep" without duplicating threshold logic.

## Goals / Non-Goals

**Goals:**
- Expose critical need status in `ColonistSnapshot` as booleans aligned with simulation buffs
- Display status labels on the info panel when a need is critical
- Visually distinguish critical need rows from satisfied ones

**Non-Goals:**
- New need types beyond Food and Sleep
- Canvas sprite indicators for hunger/sleep (panel-only change)
- Exposing `NEED_THRESHOLD` as a separate snapshot constant
- Player-facing localization of status strings

## Decisions

### Snapshot fields from ECS buffs

Add `hungry: bool` and `wants_sleep: bool` to `ColonistSnapshot`. Populate them in `Game::build_snapshot` by checking for `Hungry` / `WantsSleep` components on each colonist entity (same source of truth as task assignment).

**Alternatives considered:**
- *Derive on client from `food < 30`* — duplicates threshold constant; client and sim could drift if threshold changes
- *Single `critical_needs: NeedKind[]` array* — more flexible but heavier protocol change for two fixed needs

### Info panel layout

Keep existing Food/Sleep rows with bar + number. Append a status badge below each bar when the corresponding snapshot flag is true:
- Food critical → "Hungry" (red/warning tone)
- Sleep critical → "Wants sleep" (purple/warning tone)

When satisfied, no badge is shown (clean default state).

**Alternatives considered:**
- *Replace numeric value with status text only* — loses useful granularity (e.g. Food = 5 vs 25)
- *Separate "Needs" section listing only critical items* — hides satisfied needs; user expects both bars always visible

### TypeScript mirror

Add `hungry: boolean` and `wants_sleep: boolean` to client `ColonistSnapshot` interface. No runtime defaulting needed — WASM always sends both fields after rebuild.

## Risks / Trade-offs

- **[Snapshot size]** → Two booleans per colonist at 3 colonists / 20 Hz is negligible
- **[Buff/snapshot ordering]** → `update_need_buffs` runs before snapshot in tick; flags always current within the same tick
- **[Label staleness on panel open]** → Panel reads latest snapshot on each render cycle via Vue reactivity; no extra work needed

## Migration Plan

Additive protocol change. Rebuild WASM and client together. Old clients would ignore new fields; new client requires rebuilt WASM for flags (defaults to false if missing during dev — not a production concern).

## Open Questions

_(none)_
