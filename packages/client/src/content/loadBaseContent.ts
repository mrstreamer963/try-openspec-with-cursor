import { load as parseYaml } from 'js-yaml';
import type { ContentPack } from './types';

const BASE_URL = '/base';

async function fetchYaml<T>(filename: string, label: string): Promise<T> {
  const url = `${BASE_URL}/${filename}`;
  let response: Response;
  try {
    response = await fetch(url);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to fetch ${label} (${url}): ${message}`);
  }
  if (!response.ok) {
    throw new Error(
      `Failed to fetch ${label} (${url}): HTTP ${response.status} ${response.statusText}`,
    );
  }
  const raw = await response.text();
  try {
    return parseYaml(raw) as T;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to parse ${label} YAML: ${message}`);
  }
}

let cached: ContentPack | null = null;

/** Load and merge the base content pack YAML files into a single object. */
export async function loadBaseContent(): Promise<ContentPack> {
  if (cached) return cached;

  const [needsDoc, statusesDoc, buildingsDoc, terrainDoc, modDoc] = await Promise.all([
    fetchYaml<{ needs: ContentPack['needs'] }>('needs.yaml', 'needs'),
    fetchYaml<{ statuses: ContentPack['statuses'] }>('statuses.yaml', 'statuses'),
    fetchYaml<{ buildings: ContentPack['buildings'] }>('buildings.yaml', 'buildings'),
    fetchYaml<{ terrain: ContentPack['terrain'] }>('terrain.yaml', 'terrain'),
    fetchYaml<{ id: string; version: number }>('mod.yaml', 'mod metadata'),
  ]);

  // mod.yaml reserved for future mod metadata (id, version)
  void modDoc;

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
