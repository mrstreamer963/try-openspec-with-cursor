import { Assets, Rectangle, Texture } from 'pixi.js';
import type { AtlasDef } from '../content/types';
import { frameToRect, maxFrameCount, resolveColumns, rowCount } from './atlasFrameMath';

interface AtlasState {
  def: AtlasDef;
  texture: Texture;
  columns: number;
  rows: number;
  frameCache: Map<number, Texture>;
}

export class AtlasManager {
  private atlases = new Map<string, AtlasState>();
  private readonly warnedAtlases = new Set<string>();

  async loadAll(defs: AtlasDef[], baseUrl = import.meta.env.BASE_URL): Promise<void> {
    await Promise.all(defs.map((def) => this.loadOne(def, baseUrl)));
  }

  private async loadOne(def: AtlasDef, baseUrl: string): Promise<void> {
    const relative = def.path.startsWith('/') ? def.path.slice(1) : def.path;
    const root = baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`;
    const url = `${root}${relative}`.replace(/\/{2,}/g, '/').replace(':/', '://');
    try {
      const texture = await Assets.load<Texture>({ src: url });
      const columns = resolveColumns(texture.width, def.tile_size, def.spacing, def.columns);
      const rows = rowCount(texture.height, def.tile_size, def.spacing);
      this.atlases.set(def.id, {
        def,
        texture,
        columns,
        rows,
        frameCache: new Map(),
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      console.warn(`[AtlasManager] Failed to load atlas "${def.id}" from ${url}: ${message}`);
    }
  }

  getFrameTexture(atlasId: string, frame: number): Texture | null {
    const state = this.atlases.get(atlasId);
    if (!state) {
      if (!this.warnedAtlases.has(atlasId)) {
        console.warn(`[AtlasManager] Atlas not loaded: "${atlasId}"`);
        this.warnedAtlases.add(atlasId);
      }
      return null;
    }

    const max = maxFrameCount(
      state.texture.width,
      state.texture.height,
      state.def.tile_size,
      state.def.spacing,
      state.columns,
    );
    if (frame < 0 || frame >= max) {
      console.warn(`[AtlasManager] Frame ${frame} out of bounds for atlas "${atlasId}" (max ${max - 1})`);
      return null;
    }

    const cached = state.frameCache.get(frame);
    if (cached) return cached;

    const rect = frameToRect(frame, state.columns, state.def.tile_size, state.def.spacing, state.rows);
    if (!rect) return null;

    const frameTexture = new Texture({
      source: state.texture.source,
      frame: new Rectangle(rect.x, rect.y, rect.width, rect.height),
    });
    state.frameCache.set(frame, frameTexture);
    return frameTexture;
  }

  hasAtlas(atlasId: string): boolean {
    return this.atlases.has(atlasId);
  }
}
