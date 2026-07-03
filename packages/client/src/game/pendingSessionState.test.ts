import { describe, expect, it } from 'vitest';
import { consumePendingSessionState, setPendingSessionState } from './pendingSessionState';
import type { StateSnapshot } from './types';

function sampleState(paused: boolean): StateSnapshot {
  return {
    tiles: [],
    buildings: [],
    construction_sites: [],
    colonists: [],
    paused,
    speed: 1,
  };
}

describe('pendingSessionState', () => {
  it('passes state from beginSession to GameSession mount once', () => {
    setPendingSessionState(null);
    const state = sampleState(true);
    setPendingSessionState(state);
    expect(consumePendingSessionState()).toBe(state);
    expect(consumePendingSessionState()).toBeNull();
  });
});
