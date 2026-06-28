import init, { Game } from '../wasm/game_core.js';
import wasmUrl from '../wasm/game_core_bg.wasm?url';
import {
  parseOutgoingEvent,
  serializeIncomingEvent,
  type MainToWorkerMessage,
  type WorkerMessage,
} from '../game/protocol';
import type { IncomingEvent } from '../game/types';

const TICK_MS = 50;
const BASE_DT = 0.05;

let game: Game | null = null;
let intervalId: ReturnType<typeof setInterval> | null = null;
let paused = false;
let speed = 1;

async function initGame(): Promise<void> {
  await init(wasmUrl);
  game = new Game();
}

function startLoop(): void {
  if (intervalId !== null) return;

  intervalId = setInterval(() => {
    if (!game || paused) return;
    const dt = BASE_DT * speed;
    const json = game.tick(dt);
    const snapshot = parseOutgoingEvent(json);
    if (snapshot) {
      postMessage({ kind: 'snapshot', data: snapshot } satisfies WorkerMessage);
    }
  }, TICK_MS);
}

function handleEvent(event: IncomingEvent): void {
  if (!game) return;

  const json = serializeIncomingEvent(event);
  game.handle_event(json);

  if (event.type === 'set_paused') {
    paused = event.paused;
  } else if (event.type === 'set_speed') {
    speed = event.multiplier;
  }

  const snapshotJson = game.get_snapshot();
  const snapshot = parseOutgoingEvent(snapshotJson);
  if (snapshot) {
    postMessage({ kind: 'snapshot', data: snapshot } satisfies WorkerMessage);
  }
}

self.onmessage = async (e: MessageEvent<MainToWorkerMessage>) => {
  const msg = e.data;
  try {
    if (msg.kind === 'start') {
      await initGame();
      const snapshot = parseOutgoingEvent(game!.get_snapshot());
      if (snapshot) {
        postMessage({ kind: 'snapshot', data: snapshot } satisfies WorkerMessage);
      }
      postMessage({ kind: 'ready' } satisfies WorkerMessage);
      startLoop();
    } else if (msg.kind === 'event') {
      handleEvent(msg.event);
    }
  } catch (err) {
    postMessage({
      kind: 'error',
      message: err instanceof Error ? err.message : String(err),
    } satisfies WorkerMessage);
  }
};
