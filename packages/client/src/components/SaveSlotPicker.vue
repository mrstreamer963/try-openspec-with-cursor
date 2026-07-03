<script setup lang="ts">
import type { SaveId } from '../resources';
import { MANUAL_SLOTS } from '../resources';

defineProps<{
  mode: 'load' | 'save';
  lastSlot: SaveId;
}>();

const emit = defineEmits<{
  select: [id: SaveId];
  cancel: [];
}>();

const slots: { id: SaveId; label: string }[] = [
  { id: 'autosave', label: 'Autosave' },
  ...MANUAL_SLOTS.map((id, i) => ({ id, label: `Slot ${i + 1}` })),
];

function labelFor(id: SaveId, lastSlot: SaveId): string {
  const slot = slots.find((s) => s.id === id);
  if (!slot) return id;
  return id === lastSlot ? `${slot.label} (quick save)` : slot.label;
}
</script>

<template>
  <div class="overlay" @click.self="emit('cancel')">
    <div class="panel">
      <h3>{{ mode === 'load' ? 'Load from slot' : 'Save to slot' }}</h3>
      <button
        v-for="slot in slots"
        :key="slot.id"
        class="slot-btn"
        @click="emit('select', slot.id)"
      >
        {{ labelFor(slot.id, lastSlot) }}
      </button>
      <button class="cancel-btn" @click="emit('cancel')">Cancel</button>
    </div>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 30;
  font-family: system-ui, sans-serif;
}
.panel {
  background: #1a202c;
  color: #e2e8f0;
  border-radius: 10px;
  padding: 20px;
  min-width: 240px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
h3 {
  margin: 0 0 8px;
  font-size: 16px;
}
.slot-btn,
.cancel-btn {
  background: #2d3748;
  color: #e2e8f0;
  border: 1px solid #4a5568;
  border-radius: 6px;
  padding: 10px;
  cursor: pointer;
  text-align: left;
}
.slot-btn:hover,
.cancel-btn:hover {
  background: #4a5568;
}
</style>
