import type { SaveFile } from '../game/saveFile';

export type ModMismatchChoice = 'loadAnyway' | 'switchMods' | 'cancel';

export type QuitGuardChoice = 'save' | 'discard' | 'cancel';

export interface NativeUi {
  confirm(message: string): Promise<boolean>;
  modMismatchDialog(
    message: string,
    savedMods: string[],
    currentMods: string[],
  ): Promise<ModMismatchChoice>;
  pickOpenFile(): Promise<string | null>;
  exportSaveFile(save: SaveFile, defaultName: string): Promise<void>;
  quitGuard(): Promise<QuitGuardChoice>;
}
