import { describe, expect, it } from 'vitest';
import { mergeById, mergeContentPacks } from './mergeContent';
import type { ContentPack } from './types';

const basePack: ContentPack = {
  needs: [
    { id: 'food', label: 'Food', max: 100, decay_per_sec: 2, critical_threshold: 30 },
    { id: 'sleep', label: 'Sleep', max: 100, decay_per_sec: 1.5, critical_threshold: 30 },
  ],
  statuses: [{ id: 'hungry', label: 'Hungry', apply_when: { need: 'food', condition: 'below_threshold' }, effects: [] }],
  buildings: [
    {
      id: 'wall',
      label: 'Wall',
      work_required: 30,
      blocks_movement: true,
      blocks_settle: false,
      color: 1,
      on_complete: [],
      interactions: [],
    },
  ],
  terrain: [
    { id: 'grass', walkable: true, color: 1 },
    { id: 'water', walkable: false, color: 2 },
  ],
};

describe('mergeById', () => {
  it('replaces existing entry with later overlay', () => {
    const result = mergeById(
      basePack.needs,
      [{ id: 'food', label: 'Food', max: 100, decay_per_sec: 4, critical_threshold: 30 }],
      'needs',
    );
    expect(result.find((n) => n.id === 'food')?.decay_per_sec).toBe(4);
    expect(result.find((n) => n.id === 'sleep')?.decay_per_sec).toBe(1.5);
  });

  it('appends new ids after base order', () => {
    const result = mergeById(
      basePack.buildings,
      [
        {
          id: 'well',
          label: 'Well',
          work_required: 40,
          blocks_movement: false,
          blocks_settle: false,
          color: 3,
          on_complete: [],
          interactions: [],
        },
      ],
      'buildings',
    );
    expect(result.map((b) => b.id)).toEqual(['wall', 'well']);
  });

  it('throws on duplicate ids in overlay', () => {
    expect(() =>
      mergeById(
        basePack.terrain,
        [
          { id: 'sand', walkable: true, color: 1 },
          { id: 'sand', walkable: false, color: 2 },
        ],
        'terrain',
      ),
    ).toThrow(/duplicate id "sand"/);
  });
});

describe('mergeContentPacks', () => {
  it('merges hardmode-style food override', () => {
    const merged = mergeContentPacks(basePack, {
      needs: [{ id: 'food', label: 'Food', max: 100, decay_per_sec: 4, critical_threshold: 30 }],
    });
    expect(merged.needs.find((n) => n.id === 'food')?.decay_per_sec).toBe(4);
    expect(merged.statuses).toHaveLength(1);
    expect(merged.buildings).toHaveLength(1);
  });
});
