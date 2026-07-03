## 1. Game core — construction site model

- [x] 1.1 Add `ConstructionSite` component with `building_type`, `work_remaining`, `reserved_by`, and work constants per building type
- [x] 1.2 Add `TaskKind::Build` to `TaskKind` enum and expose in `ColonistSnapshot`
- [x] 1.3 Add helper to check for existing construction site at grid cell

## 2. Game core — build command and completion

- [x] 2.1 Change `IncomingEvent::Build` handler to spawn `ConstructionSite` instead of instant building
- [x] 2.2 Reject build on water, occupied tiles, or duplicate construction orders
- [x] 2.3 Add `complete_construction` helper: place building on grid, spawn finished entity (with `BerrySupply` for bushes), despawn site

## 3. Game core — build task systems

- [x] 3.1 Update `auto_assign_tasks`: after need tasks, assign Build to nearest unassigned construction site
- [x] 3.2 Preempt Build tasks when critical needs arise; release site reservation
- [x] 3.3 Update `task_execution`: apply work per tick when colonist is on site; complete when `work_remaining` hits 0
- [x] 3.4 Handle path failure and task cancellation with reservation release

## 4. Game core — snapshot

- [x] 4.1 Add `ConstructionSiteSnapshot` with x, y, building, progress (0.0–1.0)
- [x] 4.2 Include `construction_sites` array in `StateSnapshot` builder

## 5. Client — types and rendering

- [x] 5.1 Add `ConstructionSiteSnapshot` to TypeScript types and parse from worker snapshot
- [x] 5.2 Render construction site ghost sprites on buildings layer with progress-based opacity
- [x] 5.3 Remove ghost when site completes and building appears in snapshot
