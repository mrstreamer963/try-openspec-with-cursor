import type { StateSnapshot } from './types';

let pending: StateSnapshot | null = null;

export function setPendingSessionState(state: StateSnapshot | null): void {
  pending = state;
}

export function consumePendingSessionState(): StateSnapshot | null {
  const state = pending;
  pending = null;
  return state;
}
