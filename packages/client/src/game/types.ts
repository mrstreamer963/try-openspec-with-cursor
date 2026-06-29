export type TerrainType = 'Water' | 'Sand' | 'Grass';
export type BuildingType = 'Wall' | 'Bed' | 'BerryBush';
export type TaskKind = 'Idle' | 'Eat' | 'Sleep' | 'Build';

export interface TileSnapshot {
  x: number;
  y: number;
  terrain: TerrainType;
}

export interface BuildingSnapshot {
  x: number;
  y: number;
  building: BuildingType;
  berries?: number;
}

export interface ConstructionSiteSnapshot {
  x: number;
  y: number;
  building: BuildingType;
  progress: number;
}

export interface ColonistSnapshot {
  id: number;
  name: string;
  x: number;
  y: number;
  food: number;
  sleep: number;
  hungry: boolean;
  wants_sleep: boolean;
  task: TaskKind;
}

export interface StateSnapshot {
  tiles: TileSnapshot[];
  buildings: BuildingSnapshot[];
  construction_sites: ConstructionSiteSnapshot[];
  colonists: ColonistSnapshot[];
  paused: boolean;
  speed: number;
}

export type OutgoingEvent =
  | {
      type: 'state_snapshot';
      tiles: TileSnapshot[];
      buildings: BuildingSnapshot[];
      construction_sites: ConstructionSiteSnapshot[];
      colonists: ColonistSnapshot[];
      paused: boolean;
      speed: number;
    }
  | { type: 'error'; message: string };

export type IncomingEvent =
  | { type: 'set_paused'; paused: boolean }
  | { type: 'set_speed'; multiplier: number }
  | { type: 'build'; building: BuildingType; x: number; y: number }
  | { type: 'load_state'; state: StateSnapshot };

export type BuildMode = BuildingType | null;

export const TILE_SIZE = 16;
export const WORLD_SIZE = 50;
/** Wall-clock interval between simulation snapshots from the worker (ms). */
export const SIM_TICK_MS = 50;

export const TERRAIN_COLORS: Record<TerrainType, number> = {
  Water: 0x2b6cb0,
  Sand: 0xd69e2e,
  Grass: 0x38a169,
};

export const BUILDING_COLORS: Record<BuildingType, number> = {
  Wall: 0x718096,
  Bed: 0x9f7aea,
  BerryBush: 0xe53e3e,
};

export const COLONIST_COLOR = 0xf6e05e;
