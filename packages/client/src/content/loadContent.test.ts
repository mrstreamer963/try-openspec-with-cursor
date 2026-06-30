import { describe, expect, it } from 'vitest';
import { isMissingYamlAsset } from './loadContent';
import { discoverModCatalog, resolveModSource } from './modCatalog';
import type { ResourceManager } from '../resources/types';
import type { ModCatalogEntry } from './modCatalog';

function mockResponse(status: number, contentType: string | null): Response {
  return { status, headers: { get: (name: string) => (name === 'content-type' ? contentType : null) } } as Response;
}

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

describe('isMissingYamlAsset', () => {
  it('detects 404', () => {
    expect(isMissingYamlAsset(mockResponse(404, null), '')).toBe(true);
  });

  it('detects HTML content-type', () => {
    expect(isMissingYamlAsset(mockResponse(200, 'text/html'), 'needs:\n  - id: food')).toBe(true);
  });

  it('detects HTML body from SPA fallback', () => {
    const html = '<!DOCTYPE html><html><head><title>Idle Colony Sim</title></head></html>';
    expect(isMissingYamlAsset(mockResponse(200, 'text/html; charset=utf-8'), html)).toBe(true);
  });

  it('accepts real YAML', () => {
    expect(isMissingYamlAsset(mockResponse(200, 'text/yaml'), 'needs:\n  - id: food')).toBe(false);
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
