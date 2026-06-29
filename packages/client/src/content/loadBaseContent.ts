import { load as parseYaml } from 'js-yaml';
import modYaml from '@content/base/mod.yaml?raw';
import needsYaml from '@content/base/needs.yaml?raw';
import statusesYaml from '@content/base/statuses.yaml?raw';
import buildingsYaml from '@content/base/buildings.yaml?raw';
import terrainYaml from '@content/base/terrain.yaml?raw';
import type { ContentPack } from './types';

function parseFile<T>(raw: string): T {
  return parseYaml(raw) as T;
}

let cached: ContentPack | null = null;

/** Load and merge the base content pack YAML files into a single object. */
export function loadBaseContent(): ContentPack {
  if (cached) return cached;

  const needsDoc = parseFile<{ needs: ContentPack['needs'] }>(needsYaml);
  const statusesDoc = parseFile<{ statuses: ContentPack['statuses'] }>(statusesYaml);
  const buildingsDoc = parseFile<{ buildings: ContentPack['buildings'] }>(buildingsYaml);
  const terrainDoc = parseFile<{ terrain: ContentPack['terrain'] }>(terrainYaml);

  // mod.yaml reserved for future mod metadata (id, version)
  void parseFile<{ id: string; version: number }>(modYaml);

  cached = {
    needs: needsDoc.needs,
    statuses: statusesDoc.statuses,
    buildings: buildingsDoc.buildings,
    terrain: terrainDoc.terrain,
  };

  return cached;
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
