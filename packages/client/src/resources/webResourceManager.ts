import { isMissingBundledAsset } from './bundledAsset';
import type { ResourceLocation, ResourceManager } from './types';

const LEGACY_SETTINGS_KEY = 'idle-colony-sim-settings';
const LEGACY_SAVE_PREFIX = 'idle-colony-sim-save-';
const DATA_KEY_PREFIX = 'idle-colony-sim-data-';

function normalizePath(path: string): string {
  return path.startsWith('/') ? path.slice(1) : path;
}

function dataStorageKey(path: string): string {
  const normalized = normalizePath(path);
  if (normalized === 'settings.json') return LEGACY_SETTINGS_KEY;
  const saveMatch = normalized.match(/^saves\/(.+)\.json$/);
  if (saveMatch) return `${LEGACY_SAVE_PREFIX}${saveMatch[1]}`;
  return `${DATA_KEY_PREFIX}${normalized}`;
}

function bundledUrl(path: string, baseUrl: string): string {
  const normalized = normalizePath(path);
  const root = baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`;
  return `${root}${normalized}`;
}

export function createWebResourceManager(baseUrl = import.meta.env.BASE_URL): ResourceManager {
  return {
    async readText(location: ResourceLocation, path: string): Promise<string> {
      if (location === 'bundled') {
        const url = bundledUrl(path, baseUrl);
        let response: Response;
        try {
          response = await fetch(url);
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err);
          throw new Error(`Failed to fetch (${url}): ${message}`);
        }
        if (!response.ok) {
          throw new Error(`Failed to fetch (${url}): HTTP ${response.status} ${response.statusText}`);
        }
        return response.text();
      }

      const key = dataStorageKey(path);
      const raw = localStorage.getItem(key);
      if (raw === null) {
        throw new Error(`Missing data resource: ${path}`);
      }
      return raw;
    },

    async exists(location: ResourceLocation, path: string): Promise<boolean> {
      if (location === 'bundled') {
        try {
          const response = await fetch(bundledUrl(path, baseUrl), { method: 'HEAD' });
          return response.ok && !isMissingBundledAsset(response);
        } catch {
          return false;
        }
      }
      return localStorage.getItem(dataStorageKey(path)) !== null;
    },

    async listDir(location: ResourceLocation, _path: string): Promise<string[]> {
      if (location === 'bundled') return [];
      return [];
    },

    async writeText(location: ResourceLocation, path: string, content: string): Promise<void> {
      if (location !== 'data') {
        throw new Error(`Cannot write bundled resource: ${path}`);
      }
      localStorage.setItem(dataStorageKey(path), content);
    },

    async rename(location: ResourceLocation, fromPath: string, toPath: string): Promise<void> {
      if (location !== 'data') {
        throw new Error(`Cannot rename bundled resource: ${fromPath}`);
      }
      const fromKey = dataStorageKey(fromPath);
      const raw = localStorage.getItem(fromKey);
      if (raw === null) {
        throw new Error(`Missing data resource: ${fromPath}`);
      }
      localStorage.setItem(dataStorageKey(toPath), raw);
      localStorage.removeItem(fromKey);
    },

    async delete(location: ResourceLocation, path: string): Promise<void> {
      if (location !== 'data') {
        throw new Error(`Cannot delete bundled resource: ${path}`);
      }
      localStorage.removeItem(dataStorageKey(path));
    },

    async initialize(): Promise<void> {},

    async revealInFileManager(): Promise<void> {},
  };
}
