import { describe, expect, it } from 'vitest';
import { contentPackToJson } from './loadBaseContent';
import type { ContentPack } from './types';

const pack: ContentPack = {
  needs: [{ id: 'food', label: 'Food', max: 100, decay_per_sec: 2, critical_threshold: 30 }],
  statuses: [],
  terrain: [
    { id: 'grass', walkable: true, color: 0x38a169, sprite: { atlas: 'kenney-roguelike', frame: 855 } },
  ],
  buildings: [
    {
      id: 'wall',
      label: 'Wall',
      work_required: 30,
      blocks_movement: true,
      blocks_settle: false,
      color: 0x718096,
      sprite: { atlas: 'kenney-roguelike', frame: 746 },
      on_complete: [],
      interactions: [],
    },
  ],
  entities: [{ id: 'colonist', color: 0xf6e05e, sprite: { atlas: 'ever-rogue', frame: 49 } }],
};

describe('contentPackToJson', () => {
  it('omits sprite fields from terrain and buildings', () => {
    const json = JSON.parse(contentPackToJson(pack)) as Record<string, unknown>;
    expect(json.terrain).toEqual([{ id: 'grass', walkable: true, color: 0x38a169 }]);
    expect(json.buildings).toEqual([
      {
        id: 'wall',
        label: 'Wall',
        work_required: 30,
        work_to_deconstruct: undefined,
        blocks_movement: true,
        blocks_settle: false,
        buildable: undefined,
        color: 0x718096,
        on_complete: [],
        interactions: [],
      },
    ]);
  });

  it('omits entities category entirely', () => {
    const json = JSON.parse(contentPackToJson(pack)) as Record<string, unknown>;
    expect(json.entities).toBeUndefined();
  });

  it('preserves simulation fields', () => {
    const json = JSON.parse(contentPackToJson(pack)) as ContentPack;
    expect(json.needs).toHaveLength(1);
    expect(json.needs[0]?.decay_per_sec).toBe(2);
  });
});
