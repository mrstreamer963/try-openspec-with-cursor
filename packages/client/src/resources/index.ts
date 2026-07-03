import { createWebResourceManager } from './webResourceManager';
import type { ResourceManager } from './types';

let cached: ResourceManager = createWebResourceManager();

export function getResources(): ResourceManager {
  return cached;
}

export function setResources(resources: ResourceManager): void {
  cached = resources;
}

export function resetResourcesForTests(): void {
  cached = createWebResourceManager();
}

export type { ResourceLocation, ResourceManager } from './types';
export {
  MANUAL_SLOTS,
  deleteSave,
  exportSave,
  listSaves,
  readSave,
  writeSave,
  type SaveId,
  type SaveMetadata,
} from './saves';
