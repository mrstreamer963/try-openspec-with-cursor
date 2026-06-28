<script setup lang="ts">
import { onMounted, onUnmounted, ref, shallowRef } from 'vue';
import LoadingScreen from './components/LoadingScreen.vue';
import Hud from './components/Hud.vue';
import Toolbar from './components/Toolbar.vue';
import ColonistInfo from './components/ColonistInfo.vue';
import { GameManager } from './game/GameManager';
import { PixiRenderer } from './game/PixiRenderer';
import type { BuildMode, ColonistSnapshot, StateSnapshot } from './game/types';
import { SPEED_PRESETS } from './speedPresets';

const canvasMount = ref<HTMLElement | null>(null);
const loading = ref(true);
const paused = ref(false);
const speed = ref(1);
const buildMode = ref<BuildMode>(null);
const selectedColonist = ref<ColonistSnapshot | null>(null);
const latestSnapshot = shallowRef<StateSnapshot | null>(null);

let gameManager: GameManager | null = null;
let renderer: PixiRenderer | null = null;

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
    if (buildMode.value) {
      gameManager?.sendEvent({
        type: 'build',
        building: buildMode.value,
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
  />
  <Toolbar :build-mode="buildMode" @select-mode="(m) => (buildMode = m)" />
  <ColonistInfo :colonist="selectedColonist" />
</template>

<style scoped>
.canvas-host {
  width: 100%;
  height: 100%;
}
</style>
