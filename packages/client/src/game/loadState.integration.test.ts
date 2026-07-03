import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';
import { describe, expect, it } from 'vitest';
import init, { Game } from '../wasm/game_core.js';
import { loadContent, contentPackToJson } from '../content/loadBaseContent';
import { parseOutgoingEvent } from './protocol';

const wasmDir = join(dirname(fileURLToPath(import.meta.url)), '../wasm');

describe('wasm load_state', () => {
  it('restores paused flag from snapshot', async () => {
    await init(readFileSync(join(wasmDir, 'game_core_bg.wasm')));
    const { pack } = await loadContent({ enabledModIds: ['base', 'hardmode'] });
    const contentJson = contentPackToJson(pack);

    const game = new Game(contentJson);
    game.handle_event(JSON.stringify({ type: 'set_paused', paused: true }));

    const before = parseOutgoingEvent(game.get_snapshot());
    expect(before?.kind).toBe('snapshot');
    if (before?.kind !== 'snapshot') return;

    const loadJson = JSON.stringify({ type: 'load_state', state: before.data });
    const err = game.handle_event(loadJson);
    expect(err).toBe('');

    const after = parseOutgoingEvent(game.get_snapshot());
    expect(after?.kind).toBe('snapshot');
    if (after?.kind !== 'snapshot') return;

    expect(after.data.paused).toBe(true);
    expect(after.data.colonists).toHaveLength(before.data.colonists.length);
    expect(after.data.colonists[0]?.x).toBeCloseTo(before.data.colonists[0]!.x, 3);
  });
});
