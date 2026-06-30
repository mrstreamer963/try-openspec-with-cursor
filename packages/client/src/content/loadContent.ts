import { load as parseYaml } from 'js-yaml';
import { getResources } from '../resources';
import type { ResourceManager } from '../resources/types';
import { discoverModCatalog, resolveModSource, type ModCatalogEntry } from './modCatalog';
import { emptyContentPack, mergeContentPacks } from './mergeContent';
import type { ContentPack } from './types';
import type { ContentSource } from './ContentSource';

const CATEGORY_FILES = ['needs', 'statuses', 'buildings', 'terrain'] as const;

export interface LoadedContent {
  pack: ContentPack;
  modIds: string[];
}

export interface ContentLoadOptions {
  enabledModIds: string[];
  resources?: ResourceManager;
  catalog?: ModCatalogEntry[];
}

function modPath(modId: string, filename: string): string {
  const base = modId === 'base' ? 'base' : `mods/${modId}`;
  return `${base}/${filename}`;
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

/** True when the server returned SPA HTML instead of a YAML file (common Vite dev fallback). */
export function isMissingYamlAsset(response: Response, raw: string): boolean {
  if (response.status === 404) return true;
  const contentType = response.headers.get('content-type');
  if (contentType?.toLowerCase().includes('text/html')) return true;
  const trimmed = raw.trimStart().toLowerCase();
  return trimmed.startsWith('<!doctype') || trimmed.startsWith('<html');
}

async function readYamlFromSource<T>(
  source: ContentSource,
  path: string,
  label: string,
): Promise<T> {
  const raw = await source.readText(path);
  try {
    return parseYaml(raw) as T;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to parse ${label} YAML: ${message}`);
  }
}

async function categoryExists(source: ContentSource, path: string): Promise<boolean> {
  if (source.exists) return source.exists(path);
  try {
    await source.readText(path);
    return true;
  } catch {
    return false;
  }
}

async function loadModPartial(
  modId: string,
  source: ContentSource,
  requireAllCategories: boolean,
): Promise<Partial<ContentPack>> {
  const modMeta = await readYamlFromSource<{ id: string; version: number }>(
    source,
    modPath(modId, 'mod.yaml'),
    `${modId} mod metadata`,
  );
  if (modMeta.id !== modId) {
    throw new Error(`Mod id mismatch: expected ${modId}, got ${modMeta.id}`);
  }

  const partial: Partial<ContentPack> = {};
  for (const category of CATEGORY_FILES) {
    const path = modPath(modId, `${category}.yaml`);
    if (!(await categoryExists(source, path))) {
      if (!requireAllCategories) continue;
      throw new Error(`Failed to fetch ${modId} ${category} (${path}): file not found`);
    }

    let raw: string;
    try {
      raw = await source.readText(path);
    } catch (err) {
      if (!requireAllCategories) continue;
      const message = err instanceof Error ? err.message : String(err);
      throw new Error(`Failed to fetch ${modId} ${category} (${path}): ${message}`);
    }

    let doc: Record<string, unknown>;
    try {
      doc = parseYaml(raw) as Record<string, unknown>;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      throw new Error(`Failed to parse ${modId} ${category} YAML: ${message}`);
    }
    const items = doc[category];
    if (!Array.isArray(items)) {
      if (!requireAllCategories) continue;
      throw new Error(`${modId}/${category}.yaml is missing a ${category} array`);
    }
    if (category === 'needs') partial.needs = items as ContentPack['needs'];
    else if (category === 'statuses') partial.statuses = items as ContentPack['statuses'];
    else if (category === 'buildings') partial.buildings = items as ContentPack['buildings'];
    else partial.terrain = items as ContentPack['terrain'];
  }
  return partial;
}

function normalizeEnabledMods(enabledModIds: string[]): string[] {
  const ids = enabledModIds.length ? [...enabledModIds] : ['base'];
  if (!ids.includes('base')) ids.unshift('base');
  if (ids[0] !== 'base') {
    const idx = ids.indexOf('base');
    if (idx > 0) {
      ids.splice(idx, 1);
      ids.unshift('base');
    }
  }
  return ids;
}

let cached: LoadedContent | null = null;
let cacheKey: string | null = null;

/** Load enabled mods from sources, merge into one ContentPack. */
export async function loadContent(options?: ContentLoadOptions): Promise<LoadedContent> {
  const enabledModIds = normalizeEnabledMods(options?.enabledModIds ?? ['base']);
  const key = enabledModIds.join(',');
  if (cached && cacheKey === key) return cached;

  const resources = options?.resources ?? getResources();
  const catalog = options?.catalog ?? (await discoverModCatalog(resources));
  const bundled = bundledSource(resources);
  const user = dataSource(resources);

  let pack = emptyContentPack();
  for (const modId of enabledModIds) {
    const sourceKind = resolveModSource(modId, catalog);
    const source = sourceKind === 'user' ? user : bundled;
    const partial = await loadModPartial(modId, source, modId === 'base');
    pack = mergeContentPacks(pack, partial);
  }

  cached = { pack, modIds: enabledModIds };
  cacheKey = key;
  return cached;
}

/** Clear cached content (for tests). */
export function clearContentCache(): void {
  cached = null;
  cacheKey = null;
}
