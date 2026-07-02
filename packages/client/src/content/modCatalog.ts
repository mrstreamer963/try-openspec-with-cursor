import { load as parseYaml } from 'js-yaml';
import type { ResourceManager } from '../resources/types';
import type { ContentSource } from './ContentSource';

export interface ModCatalogEntry {
  id: string;
  source: 'bundled' | 'user';
  version?: number;
}

function bundledSource(resources: ResourceManager): ContentSource {
  return {
    readText: (path) => resources.readText('bundled', path),
    exists: (path) => resources.exists('bundled', path),
  };
}

function dataSource(resources: ResourceManager): ContentSource {
  return {
    readText: (path) => resources.readText('data', path),
    exists: (path) => resources.exists('data', path),
  };
}

async function readModMeta(
  source: ContentSource,
  modId: string,
  isBase: boolean,
): Promise<{ id: string; version?: number } | null> {
  const path = isBase ? 'base/mod.yaml' : `mods/${modId}/mod.yaml`;
  try {
    const raw = await source.readText(path);
    const doc = parseYaml(raw) as { id?: string; version?: number };
    if (typeof doc.id !== 'string') return null;
    if (doc.id !== modId) {
      console.warn(`Mod id mismatch in ${path}: expected ${modId}, got ${doc.id}`);
      return null;
    }
    return { id: doc.id, version: doc.version };
  } catch (err) {
    console.warn(`Failed to read mod metadata for ${modId}:`, err);
    return null;
  }
}

export async function discoverBundledMods(resources: ResourceManager): Promise<string[]> {
  const bundled = bundledSource(resources);
  try {
    const raw = await bundled.readText('mods.yaml');
    const doc = parseYaml(raw) as { mods?: string[] };
    if (!Array.isArray(doc.mods)) return ['base'];
    return ['base', ...doc.mods.filter((id) => id !== 'base')];
  } catch {
    return ['base'];
  }
}

export async function discoverUserMods(resources: ResourceManager): Promise<string[]> {
  try {
    const dirs = await resources.listDir('data', 'mods');
    const user = dataSource(resources);
    const ids: string[] = [];
    for (const dir of dirs) {
      if (dir === 'README.txt') continue;
      const meta = await readModMeta(user, dir, false);
      if (meta) ids.push(meta.id);
    }
    return ids;
  } catch {
    return [];
  }
}

/** Merge bundled catalog with user mods; user entries override bundled ids. */
export async function discoverModCatalog(resources: ResourceManager): Promise<ModCatalogEntry[]> {
  const bundled = bundledSource(resources);
  const user = dataSource(resources);
  const bundledIds = await discoverBundledMods(resources);
  const userIds = await discoverUserMods(resources);
  const userSet = new Set(userIds);
  const catalog: ModCatalogEntry[] = [];

  for (const id of bundledIds) {
    if (userSet.has(id)) continue;
    const meta = await readModMeta(bundled, id, id === 'base');
    catalog.push({ id, source: 'bundled', version: meta?.version });
  }

  for (const id of userIds) {
    const meta = await readModMeta(user, id, false);
    if (meta) {
      catalog.push({ id, source: 'user', version: meta.version });
    }
  }

  return catalog;
}

export function resolveModSource(
  modId: string,
  catalog: ModCatalogEntry[],
): 'bundled' | 'user' {
  const entry = catalog.find((m) => m.id === modId);
  return entry?.source ?? 'bundled';
}
