## Context

Construction is unit-based: the player designates a tile, a `ConstructionSite` is spawned, and colonists with `TaskKind::Build` work until the building is complete. There is no way to remove player-placed structures except berry bush auto-removal when depleted.

This change adds **deconstruct** — the inverse flow — with a hybrid instant/labor model and a dedicated toolbar tool. It mirrors the existing `ConstructionSite` / `TaskKind::Build` pattern for consistency with reservation, pathfinding, preemption, and snapshot infrastructure.

## Goals / Non-Goals

**Goals:**

- Player can remove construction sites and finished buildings via a **Deconstruct** toolbar button
- Construction sites at **0% progress** are cancelled instantly (no colonist labor)
- All other targets require colonist labor using per-building `work_to_deconstruct` from YAML
- Deconstruct orders can be placed while a target is in use; assignment waits until the target is free
- Idle colonists pick the **nearest** available Build or Deconstruct job (no fixed priority between them)
- Snapshots expose pending deconstruction sites for client rendering
- Save/load preserves deconstruction state

**Non-Goals:**

- Resource or material refunds on deconstruct (v1 remains resource-free)
- Cancelling a pending deconstruct order from UI
- Partial refund proportional to construction progress
- Deconstruct during simulation pause (paused = no progress, same as build)
- Diagonal stand tiles (colonist uses orthogonal adjacent cells, same as build)

## Decisions

### Mirror ConstructionSite with DeconstructionSite

| Build | Deconstruct |
|---|---|
| `ConstructionSite` | `DeconstructionSite` |
| `TaskKind::Build` | `TaskKind::Deconstruct` |
| `IncomingEvent::Build` | `IncomingEvent::Deconstruct` |
| `ConstructionSiteSnapshot` | `DeconstructionSiteSnapshot` |
| `work_required` (YAML) | `work_to_deconstruct` (YAML) |

**Rationale:** Reuses existing ECS patterns, reservation, pathfinding, preemption, and snapshot infrastructure. Minimal new concepts.

**Alternative considered:** Single unified "WorkSite" component with a mode enum. Rejected — two distinct components keep build and deconstruct logic separate and match current codebase style.

### Hybrid instant/labor model

0% construction sites cancel instantly; everything else requires colonist labor.

**Rationale:** Undoing a just-placed order should feel immediate; removing partial or finished structures should cost time proportional to building type.

**Alternative considered:** All deconstruct requires labor. Rejected — poor UX for accidental placement.

### Allow order while target in use, gate assignment

Player can place a deconstruct order on an occupied bed or bush; colonists skip it until the target is free.

**Rationale:** Player intent is recorded immediately; building remains functional until work begins.

**Alternative considered:** Reject deconstruct when target in use. Rejected — forces player to wait and retry manually.

### Nearest job selection (Build + Deconstruct pool)

No fixed priority between Build and Deconstruct — idle colonists pick the nearest reachable candidate from the merged pool.

**Rationale:** Simple, predictable, consistent with existing nearest-build assignment.

**Alternative considered:** Deconstruct always lower priority than Build. Rejected — arbitrary and harder to reason about.

### work_to_deconstruct in YAML

Per-building values in `content/base/buildings.yaml`, parsed by `ContentRegistry`:

| Building | `work_required` | `work_to_deconstruct` |
|---|---|---|
| wall | 30 | 15 |
| bed | 50 | 25 |
| berry_bush | 40 | 20 |

**Rationale:** Content-driven, moddable, consistent with existing `work_required` pattern.

## Risks / Trade-offs

- **[Target vanishes mid-deconstruct]** Berry bush depleted by eating removes building → cancel deconstruct task and release reservation. Mitigation: check target existence each tick in execution system.
- **[Bed deconstruct while colonist sleeping]** Order created but unassignable until bed free → player may not understand delay. Mitigation: red overlay shows pending order; building stays functional.
- **[Save format change]** Adding `deconstruction_sites` to snapshot. Mitigation: field defaults to empty array on load of older saves if needed; version stays at 1 with optional field.
- **[Build vs Deconstruct contention]** Nearest-job may starve one type in edge layouts. Mitigation: acceptable for v1; distance-based fairness is intuitive.

## Migration Plan

1. Add YAML field and registry accessor (backward-compatible — new field with defaults)
2. Add Rust types, systems, and snapshot fields
3. Update client types, toolbar, rendering
4. Deploy — no data migration needed for existing saves (new array field optional)

Rollback: revert commit; saves with `deconstruction_sites` ignored by older clients.

## Open Questions

_(none — design doc decisions are finalized)_
