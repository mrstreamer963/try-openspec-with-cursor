import { getResources } from '../resources';
import { DEFAULT_SETTINGS, normalizeSettings, type AppSettings } from './types';

const SETTINGS_PATH = 'settings.json';

export async function loadSettings(): Promise<AppSettings> {
  const resources = getResources();
  await resources.initialize();

  if (await resources.exists('data', SETTINGS_PATH)) {
    try {
      const raw = JSON.parse(await resources.readText('data', SETTINGS_PATH));
      return normalizeSettings(raw);
    } catch {
      return { ...DEFAULT_SETTINGS };
    }
  }

  const settings = { ...DEFAULT_SETTINGS };
  await saveSettings(settings);
  return settings;
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  const normalized = normalizeSettings(settings);
  await getResources().writeText('data', SETTINGS_PATH, JSON.stringify(normalized, null, 2));
}
