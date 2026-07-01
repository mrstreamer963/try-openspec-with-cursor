import { BaseDirectory, exists, readTextFile, rename, writeTextFile } from '@tauri-apps/plugin-fs';
import { confirm, open, save as saveDialog } from '@tauri-apps/plugin-dialog';
import { open as openShell } from '@tauri-apps/plugin-shell';
import { appDataDir, join } from '@tauri-apps/api/path';
import type { SaveFile } from '@idle-colony/client/game/saveFile';
import type { ResourceLocation, ResourceManager } from '@idle-colony/client/resources/types';
import type { ModMismatchChoice, NativeUi, QuitGuardChoice } from '@idle-colony/client/ui/types';

const APP_SUBDIR = 'idle-colony-sim';

const MODS_README = `Idle Colony Sim — User Mods

Place each mod in its own folder under this directory:
  mods/<mod-id>/mod.yaml
  mods/<mod-id>/needs.yaml   (optional)
  mods/<mod-id>/buildings.yaml (optional)
  mods/<mod-id>/statuses.yaml (optional)
  mods/<mod-id>/terrain.yaml (optional)

The mod id in mod.yaml must match the folder name.
User mods with the same id as a bundled mod override the bundled version.
`;

function dataPath(relativePath: string): string {
  return relativePath ? `${APP_SUBDIR}/${relativePath}` : APP_SUBDIR;
}

async function appRoot(): Promise<string> {
  const base = await appDataDir();
  return join(base, APP_SUBDIR);
}

function bundledUrl(path: string, baseUrl: string): string {
  const normalized = path.startsWith('/') ? path.slice(1) : path;
  const root = baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`;
  return `${root}${normalized}`;
}

export function createTauriResourceManager(baseUrl = import.meta.env.BASE_URL): ResourceManager {
  let rootPath: string | null = null;

  async function resolveRoot(): Promise<string> {
    if (!rootPath) rootPath = await appRoot();
    return rootPath;
  }

  async function toAbsolute(relativePath: string): Promise<string> {
    const root = await resolveRoot();
    return join(root, relativePath);
  }

  return {
    async readText(location: ResourceLocation, path: string): Promise<string> {
      if (location === 'bundled') {
        const url = bundledUrl(path, baseUrl);
        const response = await fetch(url);
        if (!response.ok) {
          throw new Error(`Failed to fetch bundled resource (${url}): HTTP ${response.status}`);
        }
        return response.text();
      }
      return readTextFile(dataPath(path), { baseDir: BaseDirectory.AppData });
    },

    async exists(location: ResourceLocation, path: string): Promise<boolean> {
      if (location === 'bundled') {
        try {
          const url = bundledUrl(path, baseUrl);
          const head = await fetch(url, { method: 'HEAD' });
          if (head.ok) return true;
          if (head.status !== 405 && head.status !== 501) return false;
          const get = await fetch(url, { method: 'GET', headers: { Range: 'bytes=0-0' } });
          return get.ok;
        } catch {
          return false;
        }
      }
      return exists(dataPath(path), { baseDir: BaseDirectory.AppData });
    },

    async listDir(location: ResourceLocation, path: string): Promise<string[]> {
      if (location === 'bundled') return [];
      const { readDir } = await import('@tauri-apps/plugin-fs');
      const entries = await readDir(dataPath(path), { baseDir: BaseDirectory.AppData });
      return entries.filter((e) => e.isDirectory).map((e) => e.name);
    },

    async writeText(location: ResourceLocation, path: string, content: string): Promise<void> {
      if (location === 'bundled') {
        throw new Error(`Cannot write bundled resource: ${path}`);
      }
      await writeTextFile(dataPath(path), content, { baseDir: BaseDirectory.AppData });
    },

    async rename(location: ResourceLocation, fromPath: string, toPath: string): Promise<void> {
      if (location === 'bundled') {
        throw new Error(`Cannot rename bundled resource: ${fromPath}`);
      }
      await rename(dataPath(fromPath), dataPath(toPath), {
        oldPathBaseDir: BaseDirectory.AppData,
        newPathBaseDir: BaseDirectory.AppData,
      });
    },

    async delete(location: ResourceLocation, path: string): Promise<void> {
      if (location === 'bundled') {
        throw new Error(`Cannot delete bundled resource: ${path}`);
      }
      if (await exists(dataPath(path), { baseDir: BaseDirectory.AppData })) {
        const { remove } = await import('@tauri-apps/plugin-fs');
        await remove(dataPath(path), { baseDir: BaseDirectory.AppData });
      }
    },

    async initialize(): Promise<void> {
      const { mkdir } = await import('@tauri-apps/plugin-fs');
      const root = await resolveRoot();
      await mkdir(root, { recursive: true });
      await mkdir(await join(root, 'saves'), { recursive: true });
      await mkdir(await join(root, 'mods'), { recursive: true });
      const readmePath = 'mods/README.txt';
      if (!(await this.exists('data', readmePath))) {
        await this.writeText('data', readmePath, MODS_README);
      }
    },

    async revealInFileManager(location: ResourceLocation, path: string): Promise<void> {
      if (location === 'bundled') return;
      const abs = await toAbsolute(path);
      await openShell(abs);
    },
  };
}

export function createTauriUi(): NativeUi {
  return {
    async confirm(messageText: string): Promise<boolean> {
      return confirm(messageText, { title: 'Idle Colony Sim' });
    },

    async modMismatchDialog(
      messageText: string,
      savedMods: string[],
      currentMods: string[],
    ): Promise<ModMismatchChoice> {
      const detail = `${messageText}\n\nSaved mods: [${savedMods.join(', ')}]\nCurrent mods: [${currentMods.join(', ')}]`;
      const loadAnyway = await confirm(`${detail}\n\nLoad anyway with current mods?`, {
        title: 'Mod mismatch',
        okLabel: 'Load anyway',
        cancelLabel: 'More options',
      });
      if (loadAnyway) return 'loadAnyway';

      const switchMods = await confirm('Switch to the save\'s mod list and reload?', {
        title: 'Mod mismatch',
        okLabel: 'Switch mods & load',
        cancelLabel: 'Cancel',
      });
      if (switchMods) return 'switchMods';
      return 'cancel';
    },

    async pickOpenFile(): Promise<string | null> {
      const path = await open({
        multiple: false,
        filters: [{ name: 'JSON Save', extensions: ['json'] }],
      });
      if (!path || Array.isArray(path)) return null;
      return readTextFile(path);
    },

    async exportSaveFile(saveFile: SaveFile, defaultName: string): Promise<void> {
      const path = await saveDialog({
        defaultPath: defaultName,
        filters: [{ name: 'JSON Save', extensions: ['json'] }],
      });
      if (!path) return;
      await writeTextFile(path, JSON.stringify(saveFile, null, 2));
    },

    async quitGuard(): Promise<QuitGuardChoice> {
      const saveFirst = await confirm('Save game before quitting?', {
        title: 'Unsaved changes',
        okLabel: 'Save & quit',
        cancelLabel: 'More options',
      });
      if (saveFirst) return 'save';

      const discard = await confirm('Quit without saving?', {
        title: 'Unsaved changes',
        okLabel: 'Quit without saving',
        cancelLabel: 'Cancel',
      });
      if (discard) return 'discard';
      return 'cancel';
    },
  };
}
