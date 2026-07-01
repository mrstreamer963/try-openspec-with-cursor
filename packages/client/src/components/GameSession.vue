<script setup lang="ts">
import { onMounted, onUnmounted, provide, ref, shallowRef } from 'vue';
import Hud from './Hud.vue';
import Toolbar from './Toolbar.vue';
import ColonistInfo from './ColonistInfo.vue';
import SaveSlotPicker from './SaveSlotPicker.vue';
import { GameManager } from '../game/GameManager';
import { PixiRenderer } from '../game/PixiRenderer';
import { buildSaveFile, validateSaveFile } from '../game/saveFile';
import { resolveModMismatch } from '../game/loadFlow';
import type { ColonistSnapshot, StateSnapshot, ToolMode } from '../game/types';
import { SPEED_PRESETS } from '../speedPresets';
import { contentPackKey } from '../content/injection';
import type { ContentPack } from '../content/types';
import { getResources, readSave, writeSave, exportSave as exportSaveToFile } from '../resources';
import { getUi } from '../ui';
import type { SaveId } from '../resources';
import type { AppSettings } from '../settings/types';

const props = defineProps<{
  contentPack: ContentPack;
  contentJson: string;
  modIds: string[];
  settings: AppSettings;
  initialState?: StateSnapshot | null;
}>();

const emit = defineEmits<{
  dirtyChange: [dirty: boolean];
  status: [message: string, isError?: boolean];
  switchModsLoad: [modIds: string[], state: StateSnapshot];
  quitToMenu: [];
}>();

const canvasMount = ref<HTMLElement | null>(null);
const contentPackRef = shallowRef(props.contentPack);
provide(contentPackKey, contentPackRef);
const contentReady = ref(true);
const paused = ref(false);
const speed = ref(1);
const toolMode = ref<ToolMode>(null);
const selectedColonist = ref<ColonistSnapshot | null>(null);
const showLoadPicker = ref(false);

let gameManager: GameManager | null = null;
let renderer: PixiRenderer | null = null;
let dirty = false;

function setDirty(value: boolean): void {
  if (dirty === value) return;
  dirty = value;
  emit('dirtyChange', dirty);
}

function markDirty(): void {
  setDirty(true);
}

function onKeyDown(event: KeyboardEvent): void {
  if (event.code === 'Space') {
    event.preventDefault();
    togglePause();
    return;
  }
  const speedByKey: Record<string, number> = {
    '1': SPEED_PRESETS[0],
    '2': SPEED_PRESETS[1],
    '3': SPEED_PRESETS[2],
  };
  const preset = speedByKey[event.key];
  if (preset !== undefined) setSpeed(preset);
}

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown);
  if (!canvasMount.value) return;

  try {
    gameManager = new GameManager();
    gameManager.onReady(() => {
      if (props.initialState) {
        gameManager?.sendEvent({ type: 'load_state', state: props.initialState });
        setDirty(false);
      }
    });
    gameManager.onError((msg) => {
      emit('status', msg, true);
    });
    gameManager.onSnapshot((snapshot) => {
      paused.value = snapshot.paused;
      speed.value = snapshot.speed;
      renderer?.updateSnapshot(snapshot);
      if (selectedColonist.value) {
        const updated = snapshot.colonists.find((c) => c.id === selectedColonist.value!.id);
        selectedColonist.value = updated ?? null;
      }
    });

    renderer = new PixiRenderer(canvasMount.value, props.contentPack);
    await renderer.init();
    renderer.setOnSceneClick((click) => {
      if (click.kind === 'colonist') {
        selectedColonist.value = click.colonist;
        return;
      }
      selectedColonist.value = null;
      if (toolMode.value === 'deconstruct') {
        gameManager?.sendEvent({ type: 'deconstruct', x: click.x, y: click.y });
        markDirty();
      } else if (toolMode.value) {
        gameManager?.sendEvent({ type: 'build', building: toolMode.value, x: click.x, y: click.y });
        markDirty();
      }
    });
    renderer.startRenderLoop(() => renderer?.renderFrame());
    gameManager.start(props.contentJson);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    emit('status', `Renderer failed: ${message}`, true);
  }
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
  gameManager?.destroy();
  renderer?.destroy();
});

function togglePause(): void {
  gameManager?.sendEvent({ type: 'set_paused', paused: !paused.value });
  markDirty();
}

function setSpeed(s: number): void {
  gameManager?.sendEvent({ type: 'set_speed', multiplier: s });
  markDirty();
}

async function saveToSlot(slot: SaveId): Promise<void> {
  const snapshot = gameManager?.snapshot;
  if (!snapshot) {
    emit('status', 'No game state available to save', true);
    return;
  }
  await writeSave(getResources(), slot, buildSaveFile(snapshot, props.modIds));
  setDirty(false);
  emit('status', `Saved to ${slot}`);
}

async function exportSave(): Promise<void> {
  const snapshot = gameManager?.snapshot;
  if (!snapshot) {
    emit('status', 'No game state available to export', true);
    return;
  }
  const date = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  await exportSaveToFile(getUi(), buildSaveFile(snapshot, props.modIds), `colony-save-${date}.json`);
  emit('status', 'Save exported');
}

async function loadFromSlot(slot: SaveId): Promise<void> {
  showLoadPicker.value = false;
  const save = await readSave(getResources(), slot);
  if (!save) {
    emit('status', 'Save slot is empty', true);
    return;
  }
  const result = validateSaveFile(save);
  if (typeof result === 'string') {
    emit('status', result, true);
    return;
  }

  const choice = await resolveModMismatch(result.content_mods, props.modIds);
  if (choice === 'cancel') return;
  if (choice === 'switchMods') {
    emit('switchModsLoad', result.content_mods ?? ['base'], result.state);
    return;
  }

  gameManager?.sendEvent({ type: 'load_state', state: result.state });
  setDirty(false);
  emit('status', 'Game loaded');
}

defineExpose({
  saveToSlot,
  saveQuick: () => saveToSlot(props.settings.last_slot),
  saveAutosave: () => saveToSlot('autosave'),
  exportSave,
  loadFromSlot,
  openLoadPicker: () => { showLoadPicker.value = true; },
  getSnapshot: () => gameManager?.snapshot ?? null,
  getDirty: () => dirty,
});
</script>

<template>
  <div ref="canvasMount" class="canvas-host" />
  <Hud
    :paused="paused"
    :speed="speed"
    :speed-presets="SPEED_PRESETS"
    @toggle-pause="togglePause"
    @set-speed="setSpeed"
    @save="saveToSlot(settings.last_slot)"
    @load="showLoadPicker = true"
    @quit-to-menu="emit('quitToMenu')"
  />
  <SaveSlotPicker
    v-if="showLoadPicker"
    mode="load"
    :last-slot="settings.last_slot"
    @select="loadFromSlot"
    @cancel="showLoadPicker = false"
  />
  <Toolbar v-if="contentReady" :tool-mode="toolMode" @select-mode="(m) => (toolMode = m)" />
  <ColonistInfo v-if="contentReady" :colonist="selectedColonist" />
</template>

<style scoped>
.canvas-host {
  position: fixed;
  inset: 0;
  z-index: 0;
  width: 100%;
  height: 100%;
}
</style>
