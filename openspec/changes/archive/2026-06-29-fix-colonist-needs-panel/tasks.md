## 1. Game core — snapshot need status

- [x] 1.1 Add `hungry: bool` and `wants_sleep: bool` to `ColonistSnapshot` in `events.rs`
- [x] 1.2 Populate flags in `Game::build_snapshot` from `Hungry` / `WantsSleep` ECS components
- [x] 1.3 Add unit test asserting snapshot flags match buff presence when needs are below threshold

## 2. Client — types and info panel

- [x] 2.1 Add `hungry` and `wants_sleep` to TypeScript `ColonistSnapshot` interface
- [x] 2.2 Show "Hungry" status badge on Food row when `colonist.hungry` is true
- [x] 2.3 Show "Wants sleep" status badge on Sleep row when `colonist.wants_sleep` is true
- [x] 2.4 Style critical need rows with warning emphasis (distinct from satisfied state)

## 3. Verification

- [x] 3.1 Rebuild WASM (`npm run build:wasm` or project script) and confirm dev server shows status labels when Food/Sleep drop below threshold
