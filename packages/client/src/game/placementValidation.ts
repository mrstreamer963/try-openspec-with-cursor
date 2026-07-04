import type { ContentPack, StateSnapshot } from '../content/types';

export function isBuildableTile(
  snapshot: StateSnapshot,
  content: ContentPack,
  x: number,
  y: number,
): boolean {
  const tile = snapshot.tiles.find((t) => t.x === x && t.y === y);
  if (!tile) return false;

  const terrainDef = content.terrain.find((t) => t.id === tile.terrain);
  if (!terrainDef?.walkable) return false;
  if (snapshot.buildings.some((b) => b.x === x && b.y === y)) return false;
  if (snapshot.construction_sites.some((s) => s.x === x && s.y === y)) return false;
  return true;
}

export function isDeconstructibleTile(snapshot: StateSnapshot, x: number, y: number): boolean {
  if (snapshot.buildings.some((b) => b.x === x && b.y === y)) return true;
  if (snapshot.construction_sites.some((s) => s.x === x && s.y === y)) return true;
  if (snapshot.deconstruction_sites?.some((s) => s.x === x && s.y === y)) return true;
  return false;
}
