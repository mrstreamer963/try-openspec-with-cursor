import { load as parseYaml } from 'js-yaml';
import type { ResourceManager } from '../resources/types';
import type { AtlasDef } from '../content/types';
import { AtlasManager } from './atlasManager';

interface AtlasesDoc {
  atlases: AtlasDef[];
}

export async function loadAtlases(
  resources: ResourceManager,
  baseUrl = import.meta.env.BASE_URL,
): Promise<AtlasManager> {
  const raw = await resources.readText('bundled', 'base/atlases.yaml');
  let doc: AtlasesDoc;
  try {
    doc = parseYaml(raw) as AtlasesDoc;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to parse atlases.yaml: ${message}`);
  }
  if (!Array.isArray(doc.atlases)) {
    throw new Error('atlases.yaml is missing an atlases array');
  }

  const manager = new AtlasManager();
  await manager.loadAll(doc.atlases, baseUrl);
  return manager;
}
