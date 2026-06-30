import type { DesktopHost } from './types';

let host: DesktopHost | null = null;

export function getDesktopHost(): DesktopHost | null {
  return host;
}

export function setDesktopHost(desktopHost: DesktopHost): void {
  host = desktopHost;
}

export function hasDesktopHost(): boolean {
  return host !== null;
}

export function resetDesktopHostForTests(): void {
  host = null;
}
