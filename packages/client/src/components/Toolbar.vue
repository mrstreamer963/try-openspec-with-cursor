<script setup lang="ts">
import type { BuildMode, BuildingType } from '../game/types';

defineProps<{
  buildMode: BuildMode;
}>();

const emit = defineEmits<{
  selectMode: [mode: BuildingType | null];
}>();

const tools: { type: BuildingType; label: string }[] = [
  { type: 'Wall', label: '🧱 Wall' },
  { type: 'Bed', label: '🛏 Bed' },
  { type: 'BerryBush', label: '🫐 Berry Bush' },
];
</script>

<template>
  <div class="toolbar">
    <button class="btn" :class="{ active: buildMode === null }" @click="emit('selectMode', null)">
      Select
    </button>
    <button
      v-for="tool in tools"
      :key="tool.type"
      class="btn"
      :class="{ active: buildMode === tool.type }"
      @click="emit('selectMode', tool.type)"
    >
      {{ tool.label }}
    </button>
  </div>
</template>

<style scoped>
.toolbar {
  position: fixed;
  bottom: 12px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 8px;
  background: rgba(26, 32, 44, 0.9);
  padding: 8px 12px;
  border-radius: 10px;
  z-index: 10;
  font-family: system-ui, sans-serif;
}
.btn {
  background: #2d3748;
  color: #e2e8f0;
  border: 1px solid #4a5568;
  border-radius: 6px;
  padding: 8px 14px;
  cursor: pointer;
  font-size: 13px;
}
.btn:hover { background: #4a5568; }
.btn.active {
  background: #38a169;
  border-color: #48bb78;
}
</style>
