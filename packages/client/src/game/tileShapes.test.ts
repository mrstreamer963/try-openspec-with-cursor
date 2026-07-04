import { describe, expect, it } from 'vitest';
import { horizontalVerticalLineTiles, rectTiles } from './tileShapes';

describe('horizontalVerticalLineTiles', () => {
  it('returns horizontal line inclusive', () => {
    expect(horizontalVerticalLineTiles({ x: 2, y: 5 }, { x: 7, y: 5 })).toEqual([
      { x: 2, y: 5 },
      { x: 3, y: 5 },
      { x: 4, y: 5 },
      { x: 5, y: 5 },
      { x: 6, y: 5 },
      { x: 7, y: 5 },
    ]);
  });

  it('returns vertical line inclusive', () => {
    expect(horizontalVerticalLineTiles({ x: 3, y: 1 }, { x: 3, y: 6 })).toEqual([
      { x: 3, y: 1 },
      { x: 3, y: 2 },
      { x: 3, y: 3 },
      { x: 3, y: 4 },
      { x: 3, y: 5 },
      { x: 3, y: 6 },
    ]);
  });

  it('snaps horizontal when dx equals dy', () => {
    expect(horizontalVerticalLineTiles({ x: 0, y: 0 }, { x: 3, y: 3 }).every((t) => t.y === 0)).toBe(true);
  });

  it('handles reversed drag direction', () => {
    expect(horizontalVerticalLineTiles({ x: 5, y: 2 }, { x: 2, y: 2 })).toEqual([
      { x: 2, y: 2 },
      { x: 3, y: 2 },
      { x: 4, y: 2 },
      { x: 5, y: 2 },
    ]);
  });
});

describe('rectTiles', () => {
  it('returns inclusive rectangle', () => {
    expect(rectTiles({ x: 1, y: 2 }, { x: 4, y: 5 })).toEqual([
      { x: 1, y: 2 },
      { x: 2, y: 2 },
      { x: 3, y: 2 },
      { x: 4, y: 2 },
      { x: 1, y: 3 },
      { x: 2, y: 3 },
      { x: 3, y: 3 },
      { x: 4, y: 3 },
      { x: 1, y: 4 },
      { x: 2, y: 4 },
      { x: 3, y: 4 },
      { x: 4, y: 4 },
      { x: 1, y: 5 },
      { x: 2, y: 5 },
      { x: 3, y: 5 },
      { x: 4, y: 5 },
    ]);
  });

  it('returns single tile for same start and end', () => {
    expect(rectTiles({ x: 2, y: 2 }, { x: 2, y: 2 })).toEqual([{ x: 2, y: 2 }]);
  });
});
