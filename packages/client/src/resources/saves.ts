import { validateSaveFile, type SaveFile } from '../game/saveFile';
import type { NativeUi } from '../ui/types';
import type { ResourceManager } from './types';

export type SaveId = 'autosave' | 'slot-1' | 'slot-2' | 'slot-3';

export const MANUAL_SLOTS: SaveId[] = ['slot-1', 'slot-2', 'slot-3'];

export interface SaveMetadata {
  id: SaveId;
  saved_at: string;
  content_mods?: string[];
}

function savePath(id: SaveId): string {
  return `saves/${id}.json`;
}

function tempSavePath(id: SaveId): string {
  return `saves/${id}.json.tmp`;
}

function parseSaveMetadata(id: SaveId, raw: string): SaveMetadata | null {
  try {
    const parsed = JSON.parse(raw) as { saved_at?: string; content_mods?: string[] };
    if (typeof parsed.saved_at !== 'string') return null;
    return { id, saved_at: parsed.saved_at, content_mods: parsed.content_mods };
  } catch {
    return null;
  }
}

export async function readSave(resources: ResourceManager, id: SaveId): Promise<SaveFile | null> {
  const path = savePath(id);
  if (!(await resources.exists('data', path))) return null;
  const raw = await resources.readText('data', path);
  const parsed = JSON.parse(raw);
  const result = validateSaveFile(parsed);
  if (typeof result === 'string') return null;
  return parsed as SaveFile;
}

export async function writeSave(resources: ResourceManager, id: SaveId, save: SaveFile): Promise<void> {
  const json = JSON.stringify(save, null, 2);
  const path = savePath(id);
  const tmpPath = tempSavePath(id);
  await resources.writeText('data', tmpPath, json);
  await resources.rename('data', tmpPath, path);
}

export async function deleteSave(resources: ResourceManager, id: SaveId): Promise<void> {
  const path = savePath(id);
  if (await resources.exists('data', path)) {
    await resources.delete('data', path);
  }
}

export async function listSaves(resources: ResourceManager): Promise<SaveMetadata[]> {
  const ids: SaveId[] = ['autosave', ...MANUAL_SLOTS];
  const results: SaveMetadata[] = [];
  for (const id of ids) {
    const path = savePath(id);
    if (!(await resources.exists('data', path))) continue;
    const raw = await resources.readText('data', path);
    const meta = parseSaveMetadata(id, raw);
    if (meta) results.push(meta);
  }
  return results;
}

export async function exportSave(
  ui: NativeUi,
  save: SaveFile,
  defaultName: string,
): Promise<void> {
  await ui.exportSaveFile(save, defaultName);
}
