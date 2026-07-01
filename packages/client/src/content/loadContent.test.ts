import { beforeEach, describe, expect, it } from 'vitest';
import { clearContentCache, loadContent } from './loadContent';
import { discoverModCatalog, resolveModSource } from './modCatalog';
import type { ResourceManager } from '../resources/types';
import type { ModCatalogEntry } from './modCatalog';

function mockResourceManager(files: {
  bundled?: Record<string, string>;
  data?: Record<string, string>;
  dataDirs?: Record<string, string[]>;
}): ResourceManager {
  const bundled = files.bundled ?? {};
  const data = files.data ?? {};
  const dataDirs = files.dataDirs ?? {};

  function normalize(path: string): string {
    return path.startsWith('/') ? path.slice(1) : path;
  }

  return {
    async readText(location, path) {
      const key = normalize(path);
      const store = location === 'bundled' ? bundled : data;
      if (!(key in store)) throw new Error(`missing ${location}:${key}`);
      return store[key];
    },
    async exists(location, path) {
      const key = normalize(path);
      const store = location === 'bundled' ? bundled : data;
      return key in store;
    },
    async listDir(location, path) {
      if (location !== 'data') return [];
      return dataDirs[normalize(path)] ?? [];
    },
    async writeText() {},
    async rename() {},
    async delete() {},
    async initialize() {},
    async revealInFileManager() {},
  };
}

const basePackFiles = {
  'mods.yaml': 'mods:\n  - hardmode',
  'base/mod.yaml': 'id: base\nversion: 1',
  'base/needs.yaml': 'needs:\n  - id: food\n    max: 100',
  'base/statuses.yaml': 'statuses:\n  - id: hungry',
  'base/buildings.yaml': 'buildings:\n  - id: wall',
  'base/terrain.yaml': 'terrain:\n  - id: grass',
};

const hardmodePartialFiles = {
  'mods/hardmode/mod.yaml': 'id: hardmode\nversion: 1',
  'mods/hardmode/needs.yaml': 'needs:\n  - id: food\n    max: 100\n    decay_per_sec: 4',
};

beforeEach(() => {
  clearContentCache();
});

describe('loadContent optional categories', () => {
  const html = `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Idle Colony Sim</title>
    <style>* { margin: 0; }</style>
  </head>
</html>`;

  const baseFiles = {
    ...basePackFiles,
    ...hardmodePartialFiles,
  };

  it('skips optional categories when file is absent', async () => {
    const resources = mockResourceManager({
      bundled: { ...basePackFiles, ...hardmodePartialFiles },
    });

    const { pack } = await loadContent({ enabledModIds: ['base', 'hardmode'], resources });
    expect(pack.needs.find((n) => n.id === 'food')?.decay_per_sec).toBe(4);
    expect(pack.statuses[0]?.id).toBe('hungry');
  });

  it('skips optional categories when bundled file is SPA HTML fallback', async () => {
    const resources: ResourceManager = {
      async readText(_location, path) {
        if (path === 'mods/hardmode/statuses.yaml') return html;
        const store = baseFiles as Record<string, string>;
        if (!(path in store)) throw new Error(`missing ${path}`);
        return store[path];
      },
      async exists(_location, path) {
        return path in baseFiles || path === 'mods/hardmode/statuses.yaml';
      },
      async listDir() {
        return [];
      },
      async writeText() {},
      async rename() {},
      async delete() {},
      async initialize() {},
      async revealInFileManager() {},
    };

    const { pack, modIds } = await loadContent({ enabledModIds: ['base', 'hardmode'], resources });
    expect(modIds).toEqual(['base', 'hardmode']);
    expect(pack.needs.find((n) => n.id === 'food')?.decay_per_sec).toBe(4);
    expect(pack.statuses).toHaveLength(1);
    expect(pack.statuses[0]?.id).toBe('hungry');
  });

  it('throws when optional mod file has invalid YAML', async () => {
    const resources = mockResourceManager({
      bundled: {
        ...basePackFiles,
        ...hardmodePartialFiles,
        'mods/hardmode/statuses.yaml': 'statuses:\n  - id: [broken',
      },
    });

    await expect(
      loadContent({ enabledModIds: ['base', 'hardmode'], resources }),
    ).rejects.toThrow(/Failed to parse hardmode statuses YAML/);
  });
});

describe('loadContent errors', () => {
  it('throws when base is missing a required category file', async () => {
    const resources = mockResourceManager({
      bundled: {
        'mods.yaml': 'mods: []',
        'base/mod.yaml': 'id: base\nversion: 1',
        'base/needs.yaml': 'needs:\n  - id: food\n    max: 100',
        'base/statuses.yaml': 'statuses:\n  - id: hungry',
        'base/buildings.yaml': 'buildings:\n  - id: wall',
      },
    });

    await expect(loadContent({ enabledModIds: ['base'], resources })).rejects.toThrow(
      /Failed to fetch base terrain/,
    );
  });

  it('throws when mod.yaml has invalid YAML', async () => {
    const resources = mockResourceManager({
      bundled: {
        ...basePackFiles,
        'mods/hardmode/mod.yaml': 'id: [broken',
      },
    });

    await expect(
      loadContent({ enabledModIds: ['base', 'hardmode'], resources }),
    ).rejects.toThrow(/Failed to parse hardmode mod metadata YAML/);
  });

  it('throws when mod id in mod.yaml does not match folder name', async () => {
    const resources = mockResourceManager({
      bundled: {
        ...basePackFiles,
        'mods/hardmode/mod.yaml': 'id: wrong-id\nversion: 1',
      },
    });

    await expect(
      loadContent({ enabledModIds: ['base', 'hardmode'], resources }),
    ).rejects.toThrow(/Mod id mismatch: expected hardmode, got wrong-id/);
  });
});

describe('mod catalog merge', () => {
  it('user mod overrides bundled id in catalog', async () => {
    const resources = mockResourceManager({
      bundled: {
        'mods.yaml': 'mods:\n  - hardmode',
        'base/mod.yaml': 'id: base\nversion: 1',
        'mods/hardmode/mod.yaml': 'id: hardmode\nversion: 1',
      },
      data: {
        'mods/hardmode/mod.yaml': 'id: hardmode\nversion: 2',
      },
      dataDirs: {
        mods: ['hardmode'],
      },
    });

    const catalog = await discoverModCatalog(resources);
    const hardmode = catalog.find((m) => m.id === 'hardmode');
    expect(hardmode?.source).toBe('user');
    expect(hardmode?.version).toBe(2);
    expect(catalog.filter((m) => m.id === 'hardmode')).toHaveLength(1);
  });

  it('resolveModSource prefers catalog entry', () => {
    const catalog: ModCatalogEntry[] = [
      { id: 'base', source: 'bundled' },
      { id: 'hardmode', source: 'user' },
    ];
    expect(resolveModSource('hardmode', catalog)).toBe('user');
    expect(resolveModSource('base', catalog)).toBe('bundled');
  });
});
