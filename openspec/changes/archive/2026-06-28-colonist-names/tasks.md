## 1. Game core — colonist names

- [x] 1.1 Add `ColonistName` component and static name pool constant
- [x] 1.2 Assign unique names in `spawn_colonists` (shuffle pool, no duplicates)
- [x] 1.3 Add `name` field to `ColonistSnapshot` and include in snapshot builder

## 2. Client — types and UI

- [x] 2.1 Add `name` to TypeScript `ColonistSnapshot` interface
- [x] 2.2 Render name labels above colonist sprites in `PixiRenderer`
- [x] 2.3 Update `ColonistInfo.vue` header to `{{ name }} (#{{ id }})`
