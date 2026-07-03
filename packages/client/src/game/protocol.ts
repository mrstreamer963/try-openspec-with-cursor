import type { IncomingEvent, OutgoingEvent, StateSnapshot } from './types';

export type WorkerMessage =
  | { kind: 'ready' }
  | { kind: 'snapshot'; data: StateSnapshot }
  | { kind: 'error'; message: string };

export type MainToWorkerMessage =
  | { kind: 'event'; event: IncomingEvent }
  | { kind: 'start'; contentJson: string; initialState?: StateSnapshot };

export type ParsedOutgoingEvent =
  | { kind: 'snapshot'; data: StateSnapshot }
  | { kind: 'error'; message: string };

export function parseOutgoingEvent(json: string): ParsedOutgoingEvent | null {
  if (!json) return null;

  try {
    const parsed = JSON.parse(json) as OutgoingEvent;
    if (parsed.type === 'state_snapshot') {
      return {
        kind: 'snapshot',
        data: {
          tiles: parsed.tiles,
          buildings: parsed.buildings,
          construction_sites: parsed.construction_sites ?? [],
          deconstruction_sites: parsed.deconstruction_sites ?? [],
          colonists: parsed.colonists,
          paused: parsed.paused,
          speed: parsed.speed,
        },
      };
    }
    if (parsed.type === 'error') {
      return { kind: 'error', message: parsed.message };
    }
    return null;
  } catch {
    return null;
  }
}

export function serializeIncomingEvent(event: IncomingEvent): string {
  return JSON.stringify(event);
}
