import type { SaveId } from '../resources';

export interface AppSettings {
  enabled_mods: string[];
  autosave: {
    enabled: boolean;
    interval_minutes: number;
  };
  window?: {
    width: number;
    height: number;
    x?: number;
    y?: number;
  };
  last_slot: SaveId;
}

export const DEFAULT_SETTINGS: AppSettings = {
  enabled_mods: ['base'],
  autosave: {
    enabled: true,
    interval_minutes: 5,
  },
  last_slot: 'slot-1',
};

export function normalizeSettings(raw: Partial<AppSettings> | null | undefined): AppSettings {
  const enabled = raw?.enabled_mods?.length ? [...raw.enabled_mods] : ['base'];
  if (!enabled.includes('base')) enabled.unshift('base');
  if (enabled[0] !== 'base') {
    const baseIdx = enabled.indexOf('base');
    if (baseIdx > 0) {
      enabled.splice(baseIdx, 1);
      enabled.unshift('base');
    }
  }
  return {
    enabled_mods: enabled,
    autosave: {
      enabled: raw?.autosave?.enabled ?? DEFAULT_SETTINGS.autosave.enabled,
      interval_minutes: raw?.autosave?.interval_minutes ?? DEFAULT_SETTINGS.autosave.interval_minutes,
    },
    window: raw?.window,
    last_slot: raw?.last_slot ?? DEFAULT_SETTINGS.last_slot,
  };
}
