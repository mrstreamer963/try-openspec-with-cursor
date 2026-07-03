import { describe, expect, it } from 'vitest';
import {
  frameToRect,
  maxFrameCount,
  resolveColumns,
} from './atlasFrameMath';

describe('resolveColumns', () => {
  it('returns explicit column count', () => {
    expect(resolveColumns(800, 16, 1, 12)).toBe(12);
  });

  it('computes auto columns from image width', () => {
    expect(resolveColumns(204, 16, 1, 'auto')).toBe(12);
  });

  it('computes auto columns with zero spacing', () => {
    expect(resolveColumns(256, 16, 0, 'auto')).toBe(16);
  });
});

describe('frameToRect', () => {
  const columns = 12;
  const tileSize = 16;
  const spacing = 1;

  it('maps frame 0 to origin', () => {
    expect(frameToRect(0, columns, tileSize, spacing)).toEqual({ x: 0, y: 0, width: 16, height: 16 });
  });

  it('maps frame 1 to second column', () => {
    expect(frameToRect(1, columns, tileSize, spacing)).toEqual({ x: 17, y: 0, width: 16, height: 16 });
  });

  it('maps frame 12 to second row', () => {
    expect(frameToRect(12, columns, tileSize, spacing)).toEqual({ x: 0, y: 17, width: 16, height: 16 });
  });

  it('returns null when frame is out of bounds', () => {
    expect(frameToRect(-1, columns, tileSize, spacing)).toBeNull();
    expect(frameToRect(9999, columns, tileSize, spacing, 11)).toBeNull();
  });
});

describe('maxFrameCount', () => {
  it('counts frames for a gridded sheet', () => {
    expect(maxFrameCount(204, 187, 16, 1, 12)).toBe(12 * 11);
  });
});
