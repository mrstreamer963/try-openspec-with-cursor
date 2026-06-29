<script setup lang="ts">
import { onMounted, onUnmounted, ref, shallowRef, useTemplateRef } from 'vue';
import LoadingScreen from './components/LoadingScreen.vue';
import Hud from './components/Hud.vue';
import Toolbar from './components/Toolbar.vue';
import ColonistInfo from './components/ColonistInfo.vue';
import { GameManager } from './game/GameManager';
import { PixiRenderer } from './game/PixiRenderer';
import { buildSaveFile, downloadSaveFile, validateSaveFile } from './game/saveFile';
import type { ColonistSnapshot, StateSnapshot, ToolMode } from './game/types';
import { SPEED_PRESETS } from './speedPresets';

const canvasMount = ref<HTMLElement | null>(null);
const loadInput = useTemplateRef<HTMLInputElement>('loadInput');
const loading = ref(true);
const paused = ref(false);
const speed = ref(1);
const toolMode = ref<ToolMode>(null);
const selectedColonist = ref<ColonistSnapshot | null>(null);
const latestSnapshot = shallowRef<StateSnapshot | null>(null);
const statusMessage = ref<string | null>(null);

let gameManager: GameManager | null = null;
let renderer: PixiRenderer | null = null;
let statusTimeout: ReturnType<typeof setTimeout> | null = null;

function showStatus(message: string, isError = false): void {
  statusMessage.value = isError ? `Error: ${message}` : message;
  if (statusTimeout) clearTimeout(statusTimeout);
  statusTimeout = setTimeout(() => {
    statusMessage.value = null;
  }, isError ? 6000 : 3000);
}

function onKeyDown(event: KeyboardEvent): void {
  if (loading.value) return;

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
  if (preset !== undefined) {
    setSpeed(preset);
  }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeyDown);
  if (!canvasMount.value) return;

  gameManager = new GameManager();

  gameManager.onReady(() => {
    loading.value = false;
  });

  gameManager.onError((msg) => {
    console.error('Worker error:', msg);
    showStatus(msg, true);
    loading.value = false;
  });

  gameManager.onSnapshot((snapshot) => {
    latestSnapshot.value = snapshot;
    paused.value = snapshot.paused;
    speed.value = snapshot.speed;
    renderer?.updateSnapshot(snapshot);
    if (selectedColonist.value) {
      const updated = snapshot.colonists.find((c) => c.id === selectedColonist.value!.id);
      selectedColonist.value = updated ?? null;
    }
  });

  gameManager.start();

  renderer = new PixiRenderer(canvasMount.value);
  await renderer.init();

  renderer.setOnSceneClick((click) => {
    if (click.kind === 'colonist') {
      selectedColonist.value = click.colonist;
      return;
    }
    selectedColonist.value = null;
    if (toolMode.value === 'deconstruct') {
      gameManager?.sendEvent({
        type: 'deconstruct',
        x: click.x,
        y: click.y,
      });
    } else if (toolMode.value) {
      gameManager?.sendEvent({
        type: 'build',
        building: toolMode.value,
        x: click.x,
        y: click.y,
      });
    }
  });

  renderer.startRenderLoop(() => {
    renderer?.renderFrame();
  });
});

onUnmounted(() => {
  window.removeEventListener('keydown', onKeyDown);
  if (statusTimeout) clearTimeout(statusTimeout);
  gameManager?.destroy();
  renderer?.destroy();
});

function togglePause(): void {
  const next = !paused.value;
  gameManager?.sendEvent({ type: 'set_paused', paused: next });
}

function setSpeed(s: number): void {
  gameManager?.sendEvent({ type: 'set_speed', multiplier: s });
}

function saveGame(): void {
  const snapshot = gameManager?.snapshot;
  if (!snapshot) {
    showStatus('No game state available to save', true);
    return;
  }
  downloadSaveFile(buildSaveFile(snapshot));
  showStatus('Game saved');
}

function triggerLoad(): void {
  loadInput.value?.click();
}

async function onLoadFileSelected(event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  input.value = '';
  if (!file) return;

  let parsed: unknown;
  try {
    parsed = JSON.parse(await file.text());
  } catch {
    showStatus('Invalid JSON in save file', true);
    return;
  }

  const result = validateSaveFile(parsed);
  if (typeof result === 'string') {
    showStatus(result, true);
    return;
  }

  gameManager?.sendEvent({ type: 'load_state', state: result });
  showStatus('Game loaded');
}
</script>

<template>
  <LoadingScreen :visible="loading" />
  <div ref="canvasMount" class="canvas-host" />
  <Hud
    :paused="paused"
    :speed="speed"
    :speed-presets="SPEED_PRESETS"
    @toggle-pause="togglePause"
    @set-speed="setSpeed"
    @save="saveGame"
    @load="triggerLoad"
  />
  <input
    ref="loadInput"
    type="file"
    accept=".json,application/json"
    class="hidden-input"
    @change="onLoadFileSelected"
  />
  <div v-if="statusMessage" class="status-toast">{{ statusMessage }}</div>
  <Toolbar :tool-mode="toolMode" @select-mode="(m) => (toolMode = m)" />
  <ColonistInfo :colonist="selectedColonist" />
</template>

<style scoped>
.canvas-host {
  width: 100%;
  height: 100%;
}
.hidden-input {
  display: none;
}
.status-toast {
  position: fixed;
  bottom: 16px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(26, 32, 44, 0.92);
  color: #e2e8f0;
  padding: 10px 16px;
  border-radius: 8px;
  font-family: system-ui, sans-serif;
  font-size: 14px;
  z-index: 20;
  max-width: min(90vw, 480px);
  text-align: center;
}
</style>
