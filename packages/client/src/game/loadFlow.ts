import { getUi } from '../ui';
import type { ModMismatchChoice } from '../ui/types';
import { formatModMismatchMessage, modListsEqual } from '../game/saveFile';

export async function resolveModMismatch(
  saveMods: string[] | undefined,
  currentMods: string[],
): Promise<ModMismatchChoice> {
  if (modListsEqual(saveMods, currentMods)) return 'loadAnyway';
  return getUi().modMismatchDialog(
    formatModMismatchMessage(saveMods, currentMods),
    saveMods ?? ['base'],
    currentMods,
  );
}
