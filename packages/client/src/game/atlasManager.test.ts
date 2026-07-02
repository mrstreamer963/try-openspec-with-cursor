import { beforeEach, describe, expect, it, vi } from 'vitest';
import { Rectangle, Texture } from 'pixi.js';
import type { AtlasDef } from '../content/types';

const mockLoad = vi.fn();

vi.mock('pixi.js', async () => {
  const actual = await vi.importActual<typeof import('pixi.js')>('pixi.js');
  return {
    ...actual,
    Assets: { load: (...args: unknown[]) => mockLoad(...args) },
  };
});

import { AtlasManager } from './atlasManager';

const testDef: AtlasDef = {
  id: 'test-atlas',
  path: 'assets/test.png',
  tile_size: 16,
  spacing: 1,
  columns: 12,
};

function fakeTexture(width: number, height: number): Texture {
  return { width, height, source: { uid: 1 } } as unknown as Texture;
}

beforeEach(() => {
  mockLoad.mockReset();
  vi.spyOn(console, 'warn').mockImplementation(() => {});
});

describe('AtlasManager', () => {
  it('loads atlas and resolves frame texture', async () => {
    mockLoad.mockResolvedValue(fakeTexture(204, 187));
    const manager = new AtlasManager();
    await manager.loadAll([testDef], '/');

    expect(manager.hasAtlas('test-atlas')).toBe(true);
    const frame0 = manager.getFrameTexture('test-atlas', 0);
    const frame1 = manager.getFrameTexture('test-atlas', 1);
    expect(frame0).toBeInstanceOf(Texture);
    expect(frame1).toBeInstanceOf(Texture);
    expect(frame0).not.toBe(frame1);
    expect(frame0?.frame).toEqual(new Rectangle(0, 0, 16, 16));
    expect(frame1?.frame).toEqual(new Rectangle(17, 0, 16, 16));
  });

  it('caches frame textures', async () => {
    mockLoad.mockResolvedValue(fakeTexture(204, 187));
    const manager = new AtlasManager();
    await manager.loadAll([testDef], '/');
    const first = manager.getFrameTexture('test-atlas', 0);
    const second = manager.getFrameTexture('test-atlas', 0);
    expect(first).toBe(second);
  });

  it('returns null for missing atlas and warns once', async () => {
    const manager = new AtlasManager();
    expect(manager.getFrameTexture('missing', 0)).toBeNull();
    expect(manager.getFrameTexture('missing', 0)).toBeNull();
    expect(console.warn).toHaveBeenCalledTimes(1);
  });

  it('returns null for out-of-bounds frame', async () => {
    mockLoad.mockResolvedValue(fakeTexture(204, 187));
    const manager = new AtlasManager();
    await manager.loadAll([testDef], '/');
    expect(manager.getFrameTexture('test-atlas', -1)).toBeNull();
    expect(manager.getFrameTexture('test-atlas', 9999)).toBeNull();
  });

  it('continues when one atlas fails to load', async () => {
    mockLoad
      .mockRejectedValueOnce(new Error('network'))
      .mockResolvedValueOnce(fakeTexture(204, 187));
    const manager = new AtlasManager();
    await manager.loadAll(
      [
        { ...testDef, id: 'bad' },
        testDef,
      ],
      '/',
    );
    expect(manager.hasAtlas('bad')).toBe(false);
    expect(manager.hasAtlas('test-atlas')).toBe(true);
    expect(console.warn).toHaveBeenCalled();
  });
});
