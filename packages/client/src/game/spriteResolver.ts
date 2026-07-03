import type { Texture } from 'pixi.js';
import type { ContentPack, SpriteRef } from '../content/types';
import type { AtlasManager } from './atlasManager';

export class SpriteResolver {
  private readonly warnedIds = new Set<string>();

  constructor(
    private readonly content: ContentPack,
    private readonly atlases: AtlasManager,
  ) {}

  resolveTerrain(id: string): Texture | null {
    const def = this.content.terrain.find((t) => t.id === id);
    return this.resolveSpriteRef(def?.sprite, `terrain:${id}`);
  }

  resolveBuilding(id: string): Texture | null {
    const def = this.content.buildings.find((b) => b.id === id);
    return this.resolveSpriteRef(def?.sprite, `building:${id}`);
  }

  resolveEntity(id: string): Texture | null {
    const def = this.content.entities.find((e) => e.id === id);
    return this.resolveSpriteRef(def?.sprite, `entity:${id}`);
  }

  private resolveSpriteRef(ref: SpriteRef | undefined, label: string): Texture | null {
    if (!ref) {
      this.warnOnce(label, 'no sprite field in content');
      return null;
    }
    const texture = this.atlases.getFrameTexture(ref.atlas, ref.frame);
    if (!texture) {
      this.warnOnce(label, `atlas "${ref.atlas}" frame ${ref.frame} unavailable`);
    }
    return texture;
  }

  private warnOnce(label: string, detail: string): void {
    if (this.warnedIds.has(label)) return;
    console.warn(`[SpriteResolver] ${label}: ${detail}`);
    this.warnedIds.add(label);
  }
}
