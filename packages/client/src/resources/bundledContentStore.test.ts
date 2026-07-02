import { describe, expect, it } from 'vitest';
import { getBundledContent, hasBundledContent } from './bundledContentStore';

describe('bundledContentStore', () => {
  it('includes base needs.yaml', () => {
    expect(hasBundledContent('base/needs.yaml')).toBe(true);
    expect(getBundledContent('base/needs.yaml')).toContain('id: food');
  });

  it('includes mods manifest', () => {
    expect(hasBundledContent('mods.yaml')).toBe(true);
  });

  it('includes bundled mod files', () => {
    expect(hasBundledContent('mods/hardmode/needs.yaml')).toBe(true);
  });
});
