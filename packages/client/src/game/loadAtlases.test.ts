import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { ResourceManager } from '../resources/types';

const loadAll = vi.fn().mockResolvedValue(undefined);

vi.mock('./atlasManager', () => ({
  AtlasManager: class MockAtlasManager {
    loadAll = loadAll;
  },
}));

import { loadAtlases } from './loadAtlases';

function mockResources(files: Record<string, string>): ResourceManager {
  return {
    async readText(_location, path) {
      if (!(path in files)) throw new Error(`missing ${path}`);
      return files[path];
    },
    async exists(_location, path) {
      return path in files;
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
}

const validAtlasesYaml = `atlases:
  - id: kenney-roguelike
    path: assets/kenney-roguelike/spritesheet.png
    tile_size: 16
    spacing: 1
    columns: 57
`;

beforeEach(() => {
  loadAll.mockClear();
});

describe('loadAtlases', () => {
  it('parses atlases.yaml and loads all atlases', async () => {
    const resources = mockResources({ 'base/atlases.yaml': validAtlasesYaml });
    const manager = await loadAtlases(resources, '/content/');
    expect(manager).toBeDefined();
    expect(loadAll).toHaveBeenCalledWith(
      [
        expect.objectContaining({
          id: 'kenney-roguelike',
          path: 'assets/kenney-roguelike/spritesheet.png',
          columns: 57,
        }),
      ],
      '/content/',
    );
  });

  it('throws when atlases.yaml has invalid YAML', async () => {
    const resources = mockResources({ 'base/atlases.yaml': 'atlases:\n  - id: [broken' });
    await expect(loadAtlases(resources)).rejects.toThrow(/Failed to parse atlases.yaml/);
  });

  it('throws when atlases array is missing', async () => {
    const resources = mockResources({ 'base/atlases.yaml': 'not_atlases: []' });
    await expect(loadAtlases(resources)).rejects.toThrow(/missing an atlases array/);
  });
});
