import { WORLD_SIZE, type StateSnapshot } from './types';

export const SAVE_VERSION = 1;

export interface SaveFile {
  version: number;
  saved_at: string;
  state: StateSnapshot;
}

export function buildSaveFile(snapshot: StateSnapshot): SaveFile {
  return {
    version: SAVE_VERSION,
    saved_at: new Date().toISOString(),
    state: snapshot,
  };
}

export function validateSaveFile(raw: unknown): StateSnapshot | string {
  if (typeof raw !== 'object' || raw === null) {
    return 'Save file must be a JSON object';
  }

  const file = raw as Record<string, unknown>;

  if (file.version !== SAVE_VERSION) {
    return `Unsupported save version: expected ${SAVE_VERSION}, got ${String(file.version)}`;
  }

  if (typeof file.saved_at !== 'string') {
    return 'Save file is missing saved_at timestamp';
  }

  const state = file.state;
  if (typeof state !== 'object' || state === null) {
    return 'Save file is missing state';
  }

  const snapshot = state as Record<string, unknown>;
  const expectedTiles = WORLD_SIZE * WORLD_SIZE;

  if (!Array.isArray(snapshot.tiles)) {
    return 'Save state is missing tiles array';
  }
  if (snapshot.tiles.length !== expectedTiles) {
    return `Invalid tile count: expected ${expectedTiles}, got ${snapshot.tiles.length}`;
  }

  if (!Array.isArray(snapshot.buildings)) {
    return 'Save state is missing buildings array';
  }
  if (!Array.isArray(snapshot.construction_sites)) {
    return 'Save state is missing construction_sites array';
  }
  if (!Array.isArray(snapshot.colonists)) {
    return 'Save state is missing colonists array';
  }
  if (typeof snapshot.paused !== 'boolean') {
    return 'Save state is missing paused flag';
  }
  if (typeof snapshot.speed !== 'number' || !Number.isFinite(snapshot.speed)) {
    return 'Save state is missing valid speed';
  }

  return snapshot as unknown as StateSnapshot;
}

export function downloadSaveFile(saveFile: SaveFile): void {
  const json = JSON.stringify(saveFile, null, 2);
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement('a');
  const date = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  anchor.href = url;
  anchor.download = `colony-save-${date}.json`;
  anchor.click();
  URL.revokeObjectURL(url);
}
