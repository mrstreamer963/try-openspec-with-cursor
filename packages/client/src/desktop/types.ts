export interface DesktopAppBridge {
  onMenuAction: (action: string) => void;
  onCloseRequested: () => void | Promise<void>;
}

export interface DesktopHost {
  setup(bridge: DesktopAppBridge): Promise<() => void>;
  quit(): Promise<void>;
}
