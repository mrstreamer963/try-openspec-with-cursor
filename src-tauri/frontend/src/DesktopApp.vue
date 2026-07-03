<script setup lang="ts">
import { onMounted, onUnmounted, useTemplateRef } from 'vue';
import App from '@idle-colony/client/App.vue';
import { listen } from '@tauri-apps/api/event';
import { exit } from '@tauri-apps/plugin-process';

interface AppHandle {
  requestClose(): Promise<boolean>;
  handleMenuAction(action: string): void;
}

const appRef = useTemplateRef<AppHandle>('app');
const unlisteners: Array<() => void> = [];

onMounted(async () => {
  const unlistenMenu = await listen<string>('menu-action', (event) => {
    void onMenuAction(event.payload);
  });
  const unlistenClose = await listen('close-requested', () => {
    void tryQuit();
  });
  unlisteners.push(unlistenMenu, unlistenClose);
});

onUnmounted(() => {
  unlisteners.forEach((fn) => fn());
});

async function tryQuit(): Promise<void> {
  const app = appRef.value;
  if (!app) return;
  if (await app.requestClose()) {
    await exit(0);
  }
}

async function onMenuAction(action: string): Promise<void> {
  if (action === 'quit') {
    await tryQuit();
    return;
  }
  appRef.value?.handleMenuAction(action);
}
</script>

<template>
  <App ref="app" />
</template>
