import { loadContent } from './loadContent';
import type { ContentPack } from './types';

export { loadContent, clearContentCache } from './loadContent';
export type { LoadedContent } from './loadContent';

/** Load and merge content mods into a single pack (base-only shortcut). */
export async function loadBaseContent(): Promise<ContentPack> {
  return (await loadContent()).pack;
}

/** Serialize content pack for WASM init. */
export function contentPackToJson(pack: ContentPack): string {
  return JSON.stringify(pack);
}

export function terrainColorMap(pack: ContentPack): Record<string, number> {
  const map: Record<string, number> = {};
  for (const t of pack.terrain) {
    map[t.id] = t.color;
  }
  return map;
}

export function buildingColorMap(pack: ContentPack): Record<string, number> {
  const map: Record<string, number> = {};
  for (const b of pack.buildings) {
    map[b.id] = b.color;
  }
  return map;
}

export function buildableBuildings(pack: ContentPack) {
  return pack.buildings.filter((b) => b.buildable !== false);
}

export function needLabel(pack: ContentPack, needId: string): string {
  return pack.needs.find((n) => n.id === needId)?.label ?? needId;
}

export function statusLabel(pack: ContentPack, statusId: string): string {
  return pack.statuses.find((s) => s.id === statusId)?.label ?? statusId;
}
