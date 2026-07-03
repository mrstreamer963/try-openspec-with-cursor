<script setup lang="ts">
import { onMounted, onUnmounted, ref, shallowRef, useTemplateRef } from 'vue';
import LoadingScreen from './components/LoadingScreen.vue';
import MainMenu from './components/MainMenu.vue';
import ModPicker from './components/ModPicker.vue';
import LoadGameScreen from './components/LoadGameScreen.vue';
import GameSession from './components/GameSession.vue';
import { discoverModCatalog, type ModCatalogEntry } from './content/modCatalog';
import { clearContentCache, loadContent } from './content/loadContent';
import { contentPackToJson } from './content/loadBaseContent';
import { loadAtlases } from './game/loadAtlases';
import { SpriteResolver } from './game/spriteResolver';
import { validateSaveFile, type ValidatedSave } from './game/saveFile';
import { resolveModMismatch } from './game/loadFlow';
import { setPendingSessionState } from './game/pendingSessionState';
import type { StateSnapshot } from './game/types';
import { getResources, readSave, type SaveId } from './resources';
import { getUi } from './ui';
import { loadSettings, saveSettings } from './settings/settings';
import { DEFAULT_SETTINGS, type AppSettings } from './settings/types';
import type { ContentPack } from './content/types';

type AppScreen = 'menu' | 'mods' | 'load-game' | 'loading' | 'playing';

const screen = ref<AppScreen>('menu');
const settings = ref<AppSettings>({ ...DEFAULT_SETTINGS });
const hasAutosave = ref(false);
const loadError = ref<string | null>(null);
const catalog = ref<ModCatalogEntry[]>([]);
const contentPack = shallowRef<ContentPack | null>(null);
const spriteResolver = shallowRef<SpriteResolver | null>(null);
const contentJson = ref('');
const sessionModIds = ref<string[]>(['base']);
const initialState = ref<StateSnapshot | null>(null);
const sessionKey = ref(0);
const dirty = ref(false);
const statusMessage = ref<string | null>(null);
const gameSessionRef = useTemplateRef<InstanceType<typeof GameSession>>('gameSession');

let statusTimeout: ReturnType<typeof setTimeout> | null = null;
let autosaveTimer: ReturnType<typeof setInterval> | null = null;

function showStatus(message: string, isError = false): void {
  statusMessage.value = isError ? `Error: ${message}` : message;
  if (statusTimeout) clearTimeout(statusTimeout);
  statusTimeout = setTimeout(() => {
    statusMessage.value = null;
  }, isError ? 6000 : 3000);
}

async function refreshAutosaveFlag(): Promise<void> {
  const save = await readSave(getResources(), 'autosave');
  hasAutosave.value = save !== null;
}

async function refreshCatalog(): Promise<void> {
  catalog.value = await discoverModCatalog(getResources());
}

onMounted(async () => {
  settings.value = await loadSettings();
  await refreshCatalog();
  await refreshAutosaveFlag();
});

onUnmounted(() => {
  if (statusTimeout) clearTimeout(statusTimeout);
  stopAutosaveTimer();
});

function stopAutosaveTimer(): void {
  if (autosaveTimer) {
    clearInterval(autosaveTimer);
    autosaveTimer = null;
  }
}

function startAutosaveTimer(): void {
  stopAutosaveTimer();
  if (!settings.value.autosave.enabled) return;
  const ms = settings.value.autosave.interval_minutes * 60 * 1000;
  autosaveTimer = setInterval(() => {
    if (screen.value === 'playing' && dirty.value) {
      void gameSessionRef.value?.saveAutosave();
    }
  }, ms);
}

async function beginSession(modIds: string[], state?: StateSnapshot | null): Promise<void> {
  screen.value = 'loading';
  loadError.value = null;
  clearContentCache();
  sessionModIds.value = modIds;
  const sessionState = state ?? null;
  initialState.value = sessionState;
  setPendingSessionState(sessionState);

  try {
    const loaded = await loadContent({
      enabledModIds: modIds,
    });
    const atlasManager = await loadAtlases(getResources());
    spriteResolver.value = new SpriteResolver(loaded.pack, atlasManager);
    contentPack.value = loaded.pack;
    sessionModIds.value = loaded.modIds;
    contentJson.value = contentPackToJson(loaded.pack);
    sessionKey.value += 1;
    screen.value = 'playing';
    dirty.value = false;
    startAutosaveTimer();
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    loadError.value = message;
    setPendingSessionState(null);
    screen.value = 'loading';
  }
}

function startNewGame(): void {
  void beginSession(settings.value.enabled_mods);
}

async function continueFromAutosave(): Promise<void> {
  const save = await readSave(getResources(), 'autosave');
  if (!save) {
    showStatus('No autosave found', true);
    return;
  }
  const result = validateSaveFile(save);
  if (typeof result === 'string') {
    showStatus(result, true);
    return;
  }
  await applySave(result, settings.value.enabled_mods);
}

async function applySave(result: ValidatedSave, currentMods: string[]): Promise<void> {
  const choice = await resolveModMismatch(result.content_mods, currentMods);
  if (choice === 'cancel') return;

  if (choice === 'switchMods') {
    const mods = result.content_mods ?? ['base'];
    settings.value = { ...settings.value, enabled_mods: mods };
    await saveSettings(settings.value);
    await restartGame(mods, result.state);
    return;
  }

  await beginSession(currentMods, result.state);
}

async function loadFromSlot(id: SaveId): Promise<void> {
  const save = await readSave(getResources(), id);
  if (!save) {
    showStatus('Save slot is empty', true);
    return;
  }
  const result = validateSaveFile(save);
  if (typeof result === 'string') {
    showStatus(result, true);
    return;
  }
  await applySave(result, settings.value.enabled_mods);
}

async function importSaveFile(): Promise<void> {
  const raw = await getUi().pickOpenFile();
  if (!raw) return;
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    showStatus('Invalid JSON in save file', true);
    return;
  }
  const result = validateSaveFile(parsed);
  if (typeof result === 'string') {
    showStatus(result, true);
    return;
  }
  await applySave(result, settings.value.enabled_mods);
}

async function restartGame(modIds: string[], state: StateSnapshot): Promise<void> {
  settings.value = { ...settings.value, enabled_mods: modIds };
  await saveSettings(settings.value);
  await beginSession(modIds, state);
}

async function quitToMenu(): Promise<void> {
  if (screen.value === 'playing') {
    await gameSessionRef.value?.saveAutosave();
  }
  stopAutosaveTimer();
  setPendingSessionState(null);
  contentPack.value = null;
  spriteResolver.value = null;
  contentJson.value = '';
  initialState.value = null;
  dirty.value = false;
  screen.value = 'menu';
  await refreshAutosaveFlag();
}

async function applyModSettings(enabledMods: string[]): Promise<void> {
  settings.value = { ...settings.value, enabled_mods: enabledMods };
  await saveSettings(settings.value);
  await refreshCatalog();
  screen.value = 'menu';
  showStatus('Mod settings saved');
}

async function requestClose(): Promise<boolean> {
  if (screen.value !== 'playing' || !dirty.value) {
    return true;
  }

  const choice = await getUi().quitGuard();
  if (choice === 'cancel') return false;
  if (choice === 'save') {
    await gameSessionRef.value?.saveAutosave();
  }
  return true;
}

function handleMenuAction(action: string): void {
  if (screen.value !== 'playing') {
    if (action === 'mods-manage') screen.value = 'mods';
    if (action === 'mods-open-folder') void getResources().revealInFileManager('data', 'mods');
    return;
  }

  switch (action) {
    case 'save':
      void gameSessionRef.value?.saveQuick();
      break;
    case 'load':
      gameSessionRef.value?.openLoadPicker();
      break;
    case 'export':
      void gameSessionRef.value?.exportSave();
      break;
    case 'mods-manage':
      void quitToMenu().then(() => { screen.value = 'mods'; });
      break;
    case 'mods-open-folder':
      void getResources().revealInFileManager('data', 'mods');
      break;
  }
}

defineExpose({
  requestClose,
  handleMenuAction,
});

function onDirtyChange(value: boolean): void {
  dirty.value = value;
}

function onSessionStatus(message: string, isError = false): void {
  showStatus(message, isError);
  if (!isError) void refreshAutosaveFlag();
}

function backFromLoading(): void {
  loadError.value = null;
  screen.value = 'menu';
}
</script>

<template>
  <MainMenu
    v-if="screen === 'menu'"
    :can-continue="hasAutosave"
    @continue="continueFromAutosave"
    @new-game="startNewGame"
    @load-game="screen = 'load-game'"
    @mods="screen = 'mods'"
  />

  <ModPicker
    v-if="screen === 'mods'"
    :catalog="catalog"
    :enabled-mods="settings.enabled_mods"
    @apply="applyModSettings"
    @back="screen = 'menu'"
  />

  <LoadGameScreen
    v-if="screen === 'load-game'"
    @load-slot="loadFromSlot"
    @import-file="importSaveFile"
    @back="screen = 'menu'"
  />

  <LoadingScreen
    v-if="screen === 'loading'"
    :visible="true"
    :error="loadError"
    @back="backFromLoading"
  />

  <GameSession
    v-if="screen === 'playing' && contentPack && spriteResolver"
    ref="gameSession"
    :key="sessionKey"
    :content-pack="contentPack"
    :sprite-resolver="spriteResolver"
    :content-json="contentJson"
    :mod-ids="sessionModIds"
    :settings="settings"
    :initial-state="initialState"
    @dirty-change="onDirtyChange"
    @status="onSessionStatus"
    @switch-mods-load="(mods, state) => restartGame(mods, state)"
    @quit-to-menu="quitToMenu"
  />

  <div v-if="statusMessage" class="status-toast">{{ statusMessage }}</div>
</template>

<style scoped>
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
  z-index: 100;
  max-width: min(90vw, 480px);
  text-align: center;
}
</style>
