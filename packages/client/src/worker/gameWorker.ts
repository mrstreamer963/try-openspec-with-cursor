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

async function initGame(contentJson: string): Promise<void> {
  await init(wasmUrl);

  try {
    game = new Game(contentJson);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    postMessage({ kind: 'error', message: `Game init failed: ${message}` } satisfies WorkerMessage);
    throw err;
  }
}

function startLoop(): void {
  if (intervalId !== null) return;

  intervalId = setInterval(() => {
    if (!game || paused) return;
    const dt = BASE_DT * speed;
    const json = game.tick(dt);
    const parsed = parseOutgoingEvent(json);
    if (parsed?.kind === 'snapshot') {
      postMessage({ kind: 'snapshot', data: parsed.data } satisfies WorkerMessage);
    } else if (parsed?.kind === 'error') {
      postMessage({ kind: 'error', message: parsed.message } satisfies WorkerMessage);
    }
  }, TICK_MS);
}

function handleEvent(event: IncomingEvent): boolean {
  if (!game) return false;

  const json = serializeIncomingEvent(event);
  const responseJson = game.handle_event(json);
  const response = parseOutgoingEvent(responseJson);
  if (response?.kind === 'error') {
    postMessage({ kind: 'error', message: response.message } satisfies WorkerMessage);
    return false;
  }

  if (event.type === 'set_paused') {
    paused = event.paused;
  } else if (event.type === 'set_speed') {
    speed = event.multiplier;
  } else if (event.type === 'load_state') {
    paused = event.state.paused;
    speed = event.state.speed;
  }

  const snapshotJson = game.get_snapshot();
  const snapshot = parseOutgoingEvent(snapshotJson);
  if (snapshot?.kind === 'snapshot') {
    postMessage({ kind: 'snapshot', data: snapshot.data } satisfies WorkerMessage);
  }
  return true;
}

function postCurrentSnapshot(): void {
  const snapshot = parseOutgoingEvent(game!.get_snapshot());
  if (snapshot?.kind === 'snapshot') {
    postMessage({ kind: 'snapshot', data: snapshot.data } satisfies WorkerMessage);
  }
}

async function handleStart(msg: Extract<MainToWorkerMessage, { kind: 'start' }>): Promise<void> {
  await initGame(msg.contentJson);
  if (msg.initialState) {
    const loaded = handleEvent({ type: 'load_state', state: msg.initialState });
    if (!loaded) return;
  } else {
    postCurrentSnapshot();
  }
  postMessage({ kind: 'ready' } satisfies WorkerMessage);
  startLoop();
}

self.onmessage = (e: MessageEvent<MainToWorkerMessage>) => {
  const msg = e.data;
  try {
    if (msg.kind === 'start') {
      void handleStart(msg).catch((err) => {
        postMessage({
          kind: 'error',
          message: err instanceof Error ? err.message : String(err),
        } satisfies WorkerMessage);
      });
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
