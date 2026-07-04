export interface TileCoord {
  x: number;
  y: number;
}

export function clampTile(x: number, y: number, worldSize: number): TileCoord {
  return {
    x: Math.min(worldSize - 1, Math.max(0, x)),
    y: Math.min(worldSize - 1, Math.max(0, y)),
  };
}

/** Horizontal or vertical line inclusive on the dominant axis (ties go horizontal). */
export function horizontalVerticalLineTiles(start: TileCoord, end: TileCoord): TileCoord[] {
  const dx = Math.abs(end.x - start.x);
  const dy = Math.abs(end.y - start.y);
  const tiles: TileCoord[] = [];

  if (dx >= dy) {
    const y = start.y;
    const x0 = Math.min(start.x, end.x);
    const x1 = Math.max(start.x, end.x);
    for (let x = x0; x <= x1; x++) {
      tiles.push({ x, y });
    }
  } else {
    const x = start.x;
    const y0 = Math.min(start.y, end.y);
    const y1 = Math.max(start.y, end.y);
    for (let y = y0; y <= y1; y++) {
      tiles.push({ x, y });
    }
  }

  return tiles;
}

/** Axis-aligned rectangle inclusive on both corners. */
export function rectTiles(start: TileCoord, end: TileCoord): TileCoord[] {
  const x0 = Math.min(start.x, end.x);
  const x1 = Math.max(start.x, end.x);
  const y0 = Math.min(start.y, end.y);
  const y1 = Math.max(start.y, end.y);
  const tiles: TileCoord[] = [];

  for (let y = y0; y <= y1; y++) {
    for (let x = x0; x <= x1; x++) {
      tiles.push({ x, y });
    }
  }

  return tiles;
}
