## 1. Content — YAML and registry

- [x] 1.1 Add `work_to_deconstruct` to wall (15), bed (25), and berry_bush (20) in `content/base/buildings.yaml`
- [x] 1.2 Parse `work_to_deconstruct` in content loader and add `ContentRegistry::work_to_deconstruct(id) -> f32`
- [x] 1.3 Expose `work_to_deconstruct` in client `BuildingDef` type from loaded content pack

## 2. Game core — deconstruction site model

- [x] 2.1 Add `DeconstructionSite` component with `building_id`, `work_remaining`, `reserved_by`
- [x] 2.2 Add `TaskKind::Deconstruct` to enum and expose in `ColonistSnapshot`
- [x] 2.3 Add `IncomingEvent::Deconstruct { x, y }` and `DeconstructionSiteSnapshot` types
- [x] 2.4 Add `deconstruction_sites` array to `StateSnapshot` builder with progress formula `1.0 - work_remaining / work_to_deconstruct`

## 3. Game core — deconstruct command handling

- [x] 3.1 Implement `IncomingEvent::Deconstruct` handler in `game.rs`: reject empty/duplicate, instant cancel at 0% progress, spawn site for in-progress sites and finished buildings
- [x] 3.2 Release builder reservation on instant cancel and in-progress site replacement
- [x] 3.3 Implement `complete_deconstruction`: remove building from grid, despawn building entity and site, clear colonist task

## 4. Game core — assignment and availability

- [x] 4.1 Implement availability gate: wall always free; bed checks occupancy/reservation; bush checks Eat task; construction site checks Build reservation
- [x] 4.2 Update `auto_assign_tasks`: merge unreserved construction sites and assignable deconstruction sites into nearest-job pool
- [x] 4.3 Preempt Deconstruct tasks when critical needs arise; release site reservation
- [x] 4.4 Update idle wander checks to account for unassigned deconstruction orders

## 5. Game core — deconstruct execution

- [x] 5.1 Add deconstruct work application in task execution (mirror Build: adjacent stand, lock in place, `BUILD_WORK_PER_TICK`)
- [x] 5.2 Cancel deconstruct task when site/target vanishes or path fails; release reservation
- [x] 5.3 Add Deconstruct cases to blocked-waypoint repath logic (repath, wait at stand, clear on unreachable)

## 6. Game core — save/load

- [x] 6.1 Serialize and restore `DeconstructionSite` entities in load/snapshot round-trip
- [x] 6.2 Include `deconstruction_sites` in save validation (default empty array for backward compat)

## 7. Game core — tests

- [x] 7.1 Test instant cancel: 0% construction site removed, no `DeconstructionSite` spawned
- [x] 7.2 Test labor path: finished wall gets site, colonist completes removal
- [x] 7.3 Test in-progress site: `ConstructionSite` replaced by `DeconstructionSite`, builder released
- [x] 7.4 Test availability gate: bed deconstruct not assigned while occupied; assigned after free
- [x] 7.5 Test nearest job: colonist picks closer deconstruct over farther build (and vice versa)
- [x] 7.6 Test preemption: hungry colonist abandons deconstruct task
- [x] 7.7 Test snapshot round-trip: deconstruction sites survive save/load

## 8. Client — types and protocol

- [x] 8.1 Add `DeconstructionSiteSnapshot`, `deconstruction_sites` to TypeScript types and protocol parser
- [x] 8.2 Add `{ type: 'deconstruct'; x: number; y: number }` to client event types and worker forwarding
- [x] 8.3 Update save file validation to require/accept `deconstruction_sites` array

## 9. Client — toolbar and click handling

- [x] 9.1 Add Deconstruct button to `Toolbar.vue`; extend `ToolMode` with `'deconstruct'` (mutually exclusive with select and build modes)
- [x] 9.2 Route tile clicks in `App.vue` to send deconstruct events when deconstruct mode is active

## 10. Client — rendering

- [x] 10.1 Render red semi-transparent overlay on deconstruction sites with alpha scaling by progress
- [x] 10.2 Draw red progress bar above deconstruction site tiles
- [x] 10.3 Remove overlay when site completes and building disappears from snapshot
