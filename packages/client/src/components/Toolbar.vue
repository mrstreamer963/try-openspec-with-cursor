<script setup lang="ts">
import { computed, inject } from 'vue';
import { buildableBuildings } from '../content/loadBaseContent';
import { contentPackKey } from '../content/injection';
import type { BuildingId, ToolMode } from '../game/types';

defineProps<{
  toolMode: ToolMode;
}>();

const emit = defineEmits<{
  selectMode: [mode: ToolMode];
}>();

const contentRef = inject(contentPackKey);
const tools = computed(() => {
  const content = contentRef?.value;
  if (!content) return [];
  return buildableBuildings(content).map((b) => ({
    type: b.id,
    label: b.label,
  }));
});
</script>

<template>
  <div class="toolbar">
    <button
      class="btn"
      :class="{ active: toolMode === null }"
      @click="emit('selectMode', null)"
    >
      Select
    </button>
    <button
      class="btn deconstruct"
      :class="{ active: toolMode === 'deconstruct' }"
      @click="emit('selectMode', 'deconstruct')"
    >
      🔨 Deconstruct
    </button>
    <button
      v-for="tool in tools"
      :key="tool.type"
      class="btn"
      :class="{ active: toolMode === tool.type }"
      @click="emit('selectMode', tool.type as BuildingId)"
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
.btn.deconstruct.active {
  background: #c53030;
  border-color: #fc8181;
}
</style>
