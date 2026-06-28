# Idle Colony Sim

Browser-based idle colony simulation with Rust ECS (WASM) + Vue/PixiJS.

## Development

```bash
# Install dependencies
npm install

# Build WASM + start dev server
npm run dev
```

## Production build

```bash
npm run build
npm run preview
```

## Architecture

- `packages/game-core` — Rust + bevy_ecs compiled to WASM
- `packages/client` — Vite + Vue 3 + PixiJS + WebWorker bridge
