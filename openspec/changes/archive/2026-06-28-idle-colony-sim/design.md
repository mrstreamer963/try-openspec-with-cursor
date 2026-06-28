## Context

This is a greenfield browser-based idle colony simulation inspired by RimWorld. The architecture splits concerns across three modules: a Rust ECS game core running in a WebWorker, a Vue/PixiJS view layer on the main thread, and a message bridge connecting them. The v1 scope covers a 50×50 world, 3 colonists with Food/Sleep needs, basic buildings, and A* pathfinding — all without resource costs or combat.

## Goals / Non-Goals

**Goals:**
- Playable v1 colony sim running entirely in the browser
- Clean separation: simulation logic in Rust/WASM, rendering in TypeScript/PixiJS
- Non-blocking UI: game loop in WebWorker, render on main thread via rAF
- Typed event protocol for all host ↔ core communication
- Manual tick control with pause and speed multiplier support

**Non-Goals:**
- Resource gathering, crafting, or economy systems
- Combat, raids, or health/damage
- Save/load or persistence
- Multiplayer or networking
- Procedural world beyond simple terrain noise
- Mobile/touch-first UX (desktop mouse/keyboard only for v1)
- Audio

## Decisions

### 1. Rust + bevy_ecs for game core (not full Bevy engine)

**Choice:** Use `bevy_ecs` as a standalone ECS library, not the full Bevy game engine.

**Rationale:** Full Bevy targets native platforms and has no WASM-first rendering pipeline. We only need the ECS scheduler, component system, and query API. This keeps the WASM binary small and avoids pulling in wgpu/winit.

**Alternatives considered:**
- *Pure Rust structs + manual update loops* — simpler but doesn't scale as systems grow
- *Full Bevy with bevy_web* — experimental, heavy binary, unnecessary rendering stack

### 2. Manual tick instead of Bevy's built-in schedule runner

**Choice:** Expose `tick(dt)` and run systems manually each call rather than using Bevy's `Schedule::run()`.

**Rationale:** The host (WebWorker) controls timing via setInterval. A manual tick gives precise control over dt, pause, and speed multiplier without fighting Bevy's time resources.

### 3. Snapshot-based state sync (not per-entity diffs)

**Choice:** After each tick, serialize the full world state into an `OutgoingEvent::StateSnapshot` and send it to the main thread.

**Rationale:** With 50×50 tiles + 3 colonists + a handful of buildings, the snapshot is small (<50 KB). Full snapshots are simpler to implement and debug than delta/diff protocols. Optimize later if needed.

**Alternatives considered:**
- *Event-sourced diffs* — lower bandwidth but complex serialization and ordering
- *SharedArrayBuffer* — zero-copy but requires cross-origin isolation headers

### 4. PixiJS 8 for rendering (not Canvas 2D or Three.js)

**Choice:** PixiJS 8 with three Container layers (terrain, buildings, entities).

**Rationale:** PixiJS is optimized for 2D sprite/tile rendering with good WebGL performance. Layer containers map naturally to the three visual tiers. Vue 3 integrates cleanly as an overlay for HUD/toolbar DOM elements.

**Alternatives considered:**
- *Raw Canvas 2D* — sufficient for 2500 tiles but lacks sprite/layer management
- *Three.js* — overkill for top-down 2D tile game

### 5. Monorepo layout with two packages

**Choice:**
```
/
├── packages/
│   ├── game-core/     # Rust crate → WASM
│   └── client/        # Vite + Vue 3 + PixiJS + worker
├── Cargo.toml         # workspace root
└── package.json       # npm workspace root
```

**Rationale:** Keeps Rust and TypeScript tooling separate while sharing a single repo. Vite builds the client; `wasm-pack` or `cargo build --target wasm32` produces the WASM artifact consumed by the client.

### 6. Event protocol via serde + wasm-bindgen

**Choice:** Define `IncomingEvent` and `OutgoingEvent` as Rust enums with serde derives. Expose via wasm-bindgen as JS objects or JSON strings.

**Rationale:** Serde gives type-safe serialization on both sides. wasm-bindgen handles the FFI boundary. JSON strings via postMessage are simple and debuggable; optimize to structured clone / Transferable later if profiling shows bottlenecks.

### 7. A* pathfinding in Rust (not in JS)

**Choice:** Implement A* in the game-core crate using a grid-based navmesh derived from tile walkability.

**Rationale:** Pathfinding is simulation logic — it belongs in the ECS core. Keeps the view layer purely presentational. A 50×50 grid is trivial for A* performance.

## Risks / Trade-offs

- **[Large WASM binary]** bevy_ecs + serde adds weight → Mitigation: use `wasm-opt`, strip debug symbols, profile size early
- **[Full snapshot bandwidth]** Sending entire state each tick may lag on slow devices → Mitigation: v1 scope is tiny; add diff protocol in v2 if profiling shows issues
- **[No save/load]** Players lose state on refresh → Mitigation: explicitly out of scope for v1; add localStorage persistence in v2
- **[setInterval drift]** 50 ms interval isn't frame-perfect → Mitigation: accumulate elapsed time and tick multiple frames if behind; acceptable for idle sim
- **[WASM startup latency]** Initial module load may cause visible delay → Mitigation: show loading screen in Vue while worker initializes

## Migration Plan

Greenfield — no migration needed. Deployment steps:
1. `cargo build --target wasm32-unknown-unknown --release` in `packages/game-core`
2. `wasm-bindgen` or `wasm-pack` to generate JS glue
3. `vite build` in `packages/client` bundles WASM + worker + Vue app
4. Serve static output from any static host (GitHub Pages, Netlify, etc.)

## Open Questions

- **World generation algorithm:** Simple Perlin noise for terrain, or hand-crafted layout for v1 demo?
- **Colonist sprites:** Placeholder colored circles vs. simple pixel-art sprites?
- **Build mode UX:** Click-to-place single tile, or drag-to-place multiple?
