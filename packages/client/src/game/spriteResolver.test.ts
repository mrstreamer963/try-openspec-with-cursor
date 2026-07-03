import { describe, expect, it, vi } from 'vitest';
import { SpriteResolver } from './spriteResolver';
import type { ContentPack } from '../content/types';
import type { AtlasManager } from './atlasManager';
import type { Texture } from 'pixi.js';

const fakeTexture = { label: 'frame' } as unknown as Texture;

function mockAtlasManager(frames: Record<string, Texture | null>): AtlasManager {
  return {
    getFrameTexture: vi.fn((atlas: string, frame: number) => frames[`${atlas}:${frame}`] ?? null),
    hasAtlas: vi.fn((atlas: string) => atlas in { 'kenney-roguelike': true }),
  } as unknown as AtlasManager;
}

const pack: ContentPack = {
  needs: [],
  statuses: [],
  entities: [{ id: 'colonist', color: 0xf6e05e, sprite: { atlas: 'kenney-roguelike', frame: 1 } }],
  terrain: [{ id: 'grass', walkable: true, color: 1, sprite: { atlas: 'kenney-roguelike', frame: 2 } }],
  buildings: [
    {
      id: 'wall',
      label: 'w',
      work_required: 1,
      blocks_movement: true,
      blocks_settle: false,
      color: 2,
      sprite: { atlas: 'kenney-roguelike', frame: 3 },
      on_complete: [],
      interactions: [],
    },
  ],
};

describe('SpriteResolver', () => {
  it('resolves terrain texture', () => {
    const atlas = mockAtlasManager({ 'kenney-roguelike:2': fakeTexture });
    const resolver = new SpriteResolver(pack, atlas);
    expect(resolver.resolveTerrain('grass')).toBe(fakeTexture);
  });

  it('returns null when sprite field is missing', () => {
    const noSprite: ContentPack = { ...pack, terrain: [{ id: 'grass', walkable: true, color: 1 }] };
    const atlas = mockAtlasManager({});
    const resolver = new SpriteResolver(noSprite, atlas);
    expect(resolver.resolveTerrain('grass')).toBeNull();
  });

  it('returns null when atlas frame is missing', () => {
    const atlas = mockAtlasManager({});
    const resolver = new SpriteResolver(pack, atlas);
    expect(resolver.resolveBuilding('wall')).toBeNull();
  });

  it('resolves entity texture', () => {
    const atlas = mockAtlasManager({ 'kenney-roguelike:1': fakeTexture });
    const resolver = new SpriteResolver(pack, atlas);
    expect(resolver.resolveEntity('colonist')).toBe(fakeTexture);
  });

  it('warns once per missing sprite id', () => {
    vi.spyOn(console, 'warn').mockImplementation(() => {});
    const noSprite: ContentPack = { ...pack, terrain: [{ id: 'grass', walkable: true, color: 1 }] };
    const atlas = mockAtlasManager({});
    const resolver = new SpriteResolver(noSprite, atlas);
    expect(resolver.resolveTerrain('grass')).toBeNull();
    expect(resolver.resolveTerrain('grass')).toBeNull();
    expect(console.warn).toHaveBeenCalledTimes(1);
    expect(console.warn).toHaveBeenCalledWith(
      expect.stringContaining('terrain:grass'),
    );
  });
});
