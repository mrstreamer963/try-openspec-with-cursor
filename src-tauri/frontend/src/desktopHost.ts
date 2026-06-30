import { listen } from '@tauri-apps/api/event';
import { exit } from '@tauri-apps/plugin-process';
import type { DesktopAppBridge, DesktopHost } from '@idle-colony/client/desktop/types';

export function createDesktopHost(): DesktopHost {
  return {
    async setup(bridge: DesktopAppBridge): Promise<() => void> {
      const unlistenMenu = await listen<string>('menu-action', (event) => {
        bridge.onMenuAction(event.payload);
      });
      const unlistenClose = await listen('close-requested', () => {
        void bridge.onCloseRequested();
      });
      return () => {
        unlistenMenu();
        unlistenClose();
      };
    },

    async quit(): Promise<void> {
      await exit(0);
    },
  };
}
