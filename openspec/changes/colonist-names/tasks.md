## 1. Game core — colonist names

- [ ] 1.1 Add `ColonistName` component and static name pool constant
- [ ] 1.2 Assign unique names in `spawn_colonists` (shuffle pool, no duplicates)
- [ ] 1.3 Add `name` field to `ColonistSnapshot` and include in snapshot builder

## 2. Client — types and UI

- [ ] 2.1 Add `name` to TypeScript `ColonistSnapshot` interface
- [ ] 2.2 Render name labels above colonist sprites in `PixiRenderer`
- [ ] 2.3 Update `ColonistInfo.vue` header to `{{ name }} (#{{ id }})`
