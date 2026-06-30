export interface NeedDef {
  id: string;
  label: string;
  max: number;
  decay_per_sec: number;
  critical_threshold: number;
}

export interface StatusEffect {
  type: 'task_priority';
  task: string;
  priority: number;
}

export interface StatusDef {
  id: string;
  label: string;
  apply_when: {
    need: string;
    condition: string;
  };
  effects: StatusEffect[];
}

export interface TerrainDef {
  id: string;
  walkable: boolean;
  color: number;
}

export interface SpawnPrimitive {
  type: 'supply' | 'reservation';
  resource?: string;
  amount?: number;
  kind?: string;
}

export interface InteractionEffect {
  type: 'restore_need' | 'consume_supply';
  need?: string;
  amount?: number;
  resource?: string;
}

export interface InteractionDef {
  mode: string;
  duration_sec: number;
  effects: InteractionEffect[];
}

export interface BuildingDef {
  id: string;
  label: string;
  work_required: number;
  work_to_deconstruct?: number;
  blocks_movement: boolean;
  blocks_settle: boolean;
  buildable?: boolean;
  color: number;
  on_complete: SpawnPrimitive[];
  interactions: InteractionDef[];
}

export interface ContentPack {
  needs: NeedDef[];
  statuses: StatusDef[];
  terrain: TerrainDef[];
  buildings: BuildingDef[];
}

export type TerrainId = string;
export type BuildingId = string;
export type TaskKind = 'Idle' | 'Eat' | 'Sleep' | 'Build' | 'Deconstruct';

export interface TileSnapshot {
  x: number;
  y: number;
  terrain: TerrainId;
}

export interface BuildingSnapshot {
  x: number;
  y: number;
  building: BuildingId;
  berries?: number;
}

export interface ConstructionSiteSnapshot {
  x: number;
  y: number;
  building: BuildingId;
  progress: number;
}

export interface DeconstructionSiteSnapshot {
  x: number;
  y: number;
  building: BuildingId;
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
  building_x?: number;
  building_y?: number;
  target_x?: number;
  target_y?: number;
  at_task_stand?: boolean;
}

export interface StateSnapshot {
  tiles: TileSnapshot[];
  buildings: BuildingSnapshot[];
  construction_sites: ConstructionSiteSnapshot[];
  deconstruction_sites?: DeconstructionSiteSnapshot[];
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
      deconstruction_sites?: DeconstructionSiteSnapshot[];
      colonists: ColonistSnapshot[];
      paused: boolean;
      speed: number;
    }
  | { type: 'error'; message: string };

export type IncomingEvent =
  | { type: 'set_paused'; paused: boolean }
  | { type: 'set_speed'; multiplier: number }
  | { type: 'build'; building: BuildingId; x: number; y: number }
  | { type: 'deconstruct'; x: number; y: number }
  | { type: 'load_state'; state: StateSnapshot };

export type ToolMode = BuildingId | 'deconstruct' | null;

export type BuildMode = BuildingId | null;

export const TILE_SIZE = 16;
export const WORLD_SIZE = 50;
/** Wall-clock interval between simulation snapshots from the worker (ms). */
export const SIM_TICK_MS = 50;

export const COLONIST_COLOR = 0xf6e05e;
