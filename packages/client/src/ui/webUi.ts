import { downloadSaveFile } from '../game/saveFile';
import type { ModMismatchChoice, NativeUi, QuitGuardChoice } from './types';

export function createWebUi(): NativeUi {
  return {
    async confirm(message: string): Promise<boolean> {
      return window.confirm(message);
    },

    async modMismatchDialog(
      message: string,
      savedMods: string[],
      currentMods: string[],
    ): Promise<ModMismatchChoice> {
      const loadAnyway = window.confirm(
        `${message}\n\nSaved mods: [${savedMods.join(', ')}]\nCurrent mods: [${currentMods.join(', ')}]\n\nOK = Load anyway\nCancel = see other options`,
      );
      if (loadAnyway) return 'loadAnyway';

      const switchMods = window.confirm(
        'Switch to the save\'s mod list and reload? (Cancel to abort load)',
      );
      if (switchMods) return 'switchMods';
      return 'cancel';
    },

    async pickOpenFile(): Promise<string | null> {
      return new Promise((resolve) => {
        const input = document.createElement('input');
        input.type = 'file';
        input.accept = '.json,application/json';
        input.onchange = async () => {
          const file = input.files?.[0];
          if (!file) {
            resolve(null);
            return;
          }
          resolve(await file.text());
        };
        input.click();
      });
    },

    async exportSaveFile(save, _defaultName): Promise<void> {
      downloadSaveFile(save);
    },

    async quitGuard(): Promise<QuitGuardChoice> {
      const save = window.confirm('Save game before quitting?\n\nOK = Save and quit\nCancel = see other options');
      if (save) return 'save';
      const discard = window.confirm('Quit without saving?');
      if (discard) return 'discard';
      return 'cancel';
    },
  };
}
