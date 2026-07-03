import type { ContentPack } from './types';

type IdItem = { id: string };

/** Merge overlay entries into base by id; later entries replace; new ids append. */
export function mergeById<T extends IdItem>(
  base: T[],
  overlay: T[],
  context: string,
): T[] {
  const seenInOverlay = new Set<string>();
  for (const item of overlay) {
    if (seenInOverlay.has(item.id)) {
      throw new Error(`${context}: duplicate id "${item.id}"`);
    }
    seenInOverlay.add(item.id);
  }

  const map = new Map(base.map((item) => [item.id, item]));
  const baseOrder = base.map((item) => item.id);
  const newIds: string[] = [];

  for (const item of overlay) {
    if (!map.has(item.id)) {
      newIds.push(item.id);
    }
    map.set(item.id, item);
  }

  const result: T[] = [];
  for (const id of baseOrder) {
    const item = map.get(id);
    if (item) result.push(item);
  }
  for (const id of newIds) {
    result.push(map.get(id)!);
  }
  return result;
}

export function mergeContentPacks(base: ContentPack, overlay: Partial<ContentPack>): ContentPack {
  return {
    needs: overlay.needs ? mergeById(base.needs, overlay.needs, 'needs') : base.needs,
    statuses: overlay.statuses
      ? mergeById(base.statuses, overlay.statuses, 'statuses')
      : base.statuses,
    buildings: overlay.buildings
      ? mergeById(base.buildings, overlay.buildings, 'buildings')
      : base.buildings,
    terrain: overlay.terrain ? mergeById(base.terrain, overlay.terrain, 'terrain') : base.terrain,
    entities: overlay.entities
      ? mergeById(base.entities, overlay.entities, 'entities')
      : base.entities,
  };
}

export function emptyContentPack(): ContentPack {
  return { needs: [], statuses: [], buildings: [], terrain: [], entities: [] };
}
