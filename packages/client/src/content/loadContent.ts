import { load as parseYaml } from 'js-yaml';
import { emptyContentPack, mergeContentPacks } from './mergeContent';
import type { ContentPack } from './types';

const CATEGORY_FILES = ['needs', 'statuses', 'buildings', 'terrain'] as const;

export interface LoadedContent {
  pack: ContentPack;
  modIds: string[];
}

async function fetchText(url: string): Promise<string> {
  let response: Response;
  try {
    response = await fetch(url);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to fetch (${url}): ${message}`);
  }
  if (!response.ok) {
    throw new Error(`Failed to fetch (${url}): HTTP ${response.status} ${response.statusText}`);
  }
  return response.text();
}

async function fetchYaml<T>(url: string, label: string): Promise<T> {
  const raw = await fetchText(url);
  try {
    return parseYaml(raw) as T;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to parse ${label} YAML: ${message}`);
  }
}

function modBasePath(modId: string): string {
  return modId === 'base' ? '/base' : `/mods/${modId}`;
}

/** True when the server returned SPA HTML instead of a YAML file (common Vite dev fallback). */
export function isMissingYamlAsset(response: Response, raw: string): boolean {
  if (response.status === 404) return true;
  const contentType = response.headers.get('content-type');
  if (contentType?.toLowerCase().includes('text/html')) return true;
  const trimmed = raw.trimStart().toLowerCase();
  return trimmed.startsWith('<!doctype') || trimmed.startsWith('<html');
}

async function loadModManifest(): Promise<string[]> {
  try {
    const doc = await fetchYaml<{ mods?: string[] }>('/mods.yaml', 'mods manifest');
    if (!Array.isArray(doc.mods) || doc.mods.length === 0) {
      throw new Error('mods.yaml must contain a non-empty mods array');
    }
    return doc.mods;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    if (message.includes('HTTP 404')) {
      return ['base'];
    }
    throw err;
  }
}

async function loadModPartial(
  modId: string,
  requireAllCategories: boolean,
): Promise<Partial<ContentPack>> {
  const basePath = modBasePath(modId);
  const modMeta = await fetchYaml<{ id: string; version: number }>(
    `${basePath}/mod.yaml`,
    `${modId} mod metadata`,
  );
  if (modMeta.id !== modId) {
    throw new Error(`Mod id mismatch: expected ${modId}, got ${modMeta.id}`);
  }

  const partial: Partial<ContentPack> = {};
  for (const category of CATEGORY_FILES) {
    const url = `${basePath}/${category}.yaml`;
    let response: Response;
    try {
      response = await fetch(url);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      throw new Error(`Failed to fetch ${modId} ${category} (${url}): ${message}`);
    }
    if (!response.ok) {
      if (response.status === 404 && !requireAllCategories) {
        continue;
      }
      throw new Error(
        `Failed to fetch ${modId} ${category} (${url}): HTTP ${response.status} ${response.statusText}`,
      );
    }
    const raw = await response.text();
    if (isMissingYamlAsset(response, raw)) {
      if (!requireAllCategories) {
        continue;
      }
      throw new Error(`Failed to fetch ${modId} ${category} (${url}): file not found`);
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
      throw new Error(`${modId}/${category}.yaml is missing a ${category} array`);
    }
    if (category === 'needs') partial.needs = items as ContentPack['needs'];
    else if (category === 'statuses') partial.statuses = items as ContentPack['statuses'];
    else if (category === 'buildings') partial.buildings = items as ContentPack['buildings'];
    else partial.terrain = items as ContentPack['terrain'];
  }
  return partial;
}

let cached: LoadedContent | null = null;

/** Load mod manifest, fetch each mod, merge into one ContentPack. */
export async function loadContent(): Promise<LoadedContent> {
  if (cached) return cached;

  const modIds = await loadModManifest();
  let pack = emptyContentPack();

  for (const modId of modIds) {
    const partial = await loadModPartial(modId, modId === 'base');
    pack = mergeContentPacks(pack, partial);
  }

  cached = { pack, modIds };
  return cached;
}

/** Clear cached content (for tests). */
export function clearContentCache(): void {
  cached = null;
}
