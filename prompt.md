# Idle Colony Sim (RimWorld-like)

## Архитектура (3 модуля)

### 1. Game Core (Rust + bevy_ecs → WASM)
ECS-движок внутри WebWorker. Мир 50×50 (вода, песок, трава), колонисты с потребностями (Еда/Сон), постройки (кровать, куст с ягодами - еда), A* pathfinding. Ручной `tick(dt)`, события через `IncomingEvent`/`OutgoingEvent`. Данные через `wasm-bindgen` + `serde`.

### 2. View Layer (Vite + Vue 3 + TypeScript + PixiJS 8)
Рендер тайлов, синепринтов, сущностей (3 слоя). Камера (pan/zoom). HUD с кнопками паузы и скорости. Панель инструментов (Wall/Bed/BerryBush). При клике на колониста открывается информационная панель (нужды, задача, позиция).

### 3. Worker Bridge
Связь через `postMessage`. Game loop: `setInterval(50ms)` в воркере, рендер по `requestAnimationFrame`. Vite плагины: `vite-plugin-wasm`.

## Геймплей v1
3 колониста, автоматические задачи при низких потребностях, строительство без ресурсов, реалистичное время с паузой.