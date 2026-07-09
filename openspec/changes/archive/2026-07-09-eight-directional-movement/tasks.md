## 1. Pathfinding core

- [x] 1.1 Add 8-direction neighbor list with orthogonal cost `1.0` and diagonal cost `SQRT_2` in `pathfinding.rs`
- [x] 1.2 Implement corner-cutting check: diagonal step allowed only when both sharing-edge orthogonal cells are walkable
- [x] 1.3 Replace Manhattan heuristic with octile distance on `Node`

## 2. Tests

- [x] 2.1 Add test: open-area path prefers diagonal shortcut over L-shaped orthogonal route
- [x] 2.2 Add test: diagonal step blocked when corner is bounded by two impassable orthogonal cells
- [x] 2.3 Run `cargo test -p game-core` and confirm all existing pathfinding/movement tests still pass

## 3. Verification

- [x] 3.1 Manually verify in-game: colonists take diagonal routes across open ground; walls still block corner-cutting
