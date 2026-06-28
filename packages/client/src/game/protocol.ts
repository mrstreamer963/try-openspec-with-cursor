import type { IncomingEvent, OutgoingEvent, StateSnapshot } from './types';

export type WorkerMessage =
  | { kind: 'ready' }
  | { kind: 'snapshot'; data: StateSnapshot }
  | { kind: 'error'; message: string };

export type MainToWorkerMessage =
  | { kind: 'event'; event: IncomingEvent }
  | { kind: 'start' };

export function parseOutgoingEvent(json: string): StateSnapshot | null {
  try {
    const parsed = JSON.parse(json) as OutgoingEvent & { type?: string };
    if (parsed.type === 'state_snapshot') {
      return {
        tiles: parsed.tiles,
        buildings: parsed.buildings,
        colonists: parsed.colonists,
        paused: parsed.paused,
        speed: parsed.speed,
      };
    }
    return null;
  } catch {
    return null;
  }
}

export function serializeIncomingEvent(event: IncomingEvent): string {
  return JSON.stringify(event);
}
