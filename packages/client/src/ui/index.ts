import { createWebUi } from './webUi';
import type { NativeUi } from './types';

let cached: NativeUi = createWebUi();

export function getUi(): NativeUi {
  return cached;
}

export function setUi(ui: NativeUi): void {
  cached = ui;
}

export function resetUiForTests(): void {
  cached = createWebUi();
}
