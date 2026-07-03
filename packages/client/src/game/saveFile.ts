import { WORLD_SIZE, type StateSnapshot } from './types';

export const SAVE_VERSION = 1;

/**
 * Save files from before yaml-content-definitions used PascalCase building/terrain ids
 * (e.g. "BerryBush", "Grass"). Current saves use snake_case content ids from YAML
 * (e.g. "berry_bush", "grass"). Old saves must be migrated manually or re-exported.
 */
export const SAVE_ID_MIGRATION_NOTE =
  'Building and terrain ids in save state changed from PascalCase enum names to snake_case YAML ids.';

export interface SaveFile {
  version: number;
  saved_at: string;
  content_mods?: string[];
  state: StateSnapshot;
}

export interface ValidatedSave {
  state: StateSnapshot;
  content_mods?: string[];
}

export function buildSaveFile(snapshot: StateSnapshot, modIds?: string[]): SaveFile {
  const file: SaveFile = {
    version: SAVE_VERSION,
    saved_at: new Date().toISOString(),
    state: snapshot,
  };
  if (modIds && modIds.length > 0) {
    file.content_mods = modIds;
  }
  return file;
}

export function validateSaveFile(raw: unknown): ValidatedSave | string {
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

  if (file.content_mods !== undefined) {
    if (!Array.isArray(file.content_mods) || !file.content_mods.every((m) => typeof m === 'string')) {
      return 'Save file content_mods must be an array of strings';
    }
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
  if (snapshot.deconstruction_sites != null && !Array.isArray(snapshot.deconstruction_sites)) {
    return 'Save state deconstruction_sites must be an array';
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

  return {
    state: snapshot as unknown as StateSnapshot,
    content_mods: file.content_mods as string[] | undefined,
  };
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

export function modListsEqual(a: string[] | undefined, b: string[]): boolean {
  const left = a ?? ['base'];
  if (left.length !== b.length) return false;
  return left.every((id, i) => id === b[i]);
}

export function formatModMismatchMessage(saveMods: string[] | undefined, currentMods: string[]): string {
  const saved = (saveMods ?? ['base']).join(', ');
  const current = currentMods.join(', ');
  return `This save was created with mods: [${saved}]. Current mods: [${current}]. Load anyway?`;
}
