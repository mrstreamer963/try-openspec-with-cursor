<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { getResources, listSaves, type SaveMetadata } from '../resources';
import { MANUAL_SLOTS } from '../resources';

const emit = defineEmits<{
  loadSlot: [id: SaveMetadata['id']];
  importFile: [];
  back: [];
}>();

const saves = ref<SaveMetadata[]>([]);
const loading = ref(true);

onMounted(async () => {
  saves.value = await listSaves(getResources());
  loading.value = false;
});

function labelFor(id: SaveMetadata['id']): string {
  if (id === 'autosave') return 'Autosave';
  const slotNum = MANUAL_SLOTS.indexOf(id) + 1;
  return `Slot ${slotNum}`;
}

function formatDate(iso: string): string {
  return new Date(iso).toLocaleString();
}
</script>

<template>
  <div class="load-game">
    <div class="panel">
      <h2>Load Game</h2>
      <p v-if="loading" class="hint">Loading saves…</p>
      <p v-else-if="saves.length === 0" class="hint">No saves found.</p>
      <div v-else class="save-list">
        <button
          v-for="save in saves"
          :key="save.id"
          class="save-btn"
          @click="emit('loadSlot', save.id)"
        >
          <span class="save-label">{{ labelFor(save.id) }}</span>
          <span class="save-date">{{ formatDate(save.saved_at) }}</span>
        </button>
      </div>
      <div class="actions">
        <button class="btn" @click="emit('importFile')">Import from file…</button>
        <button class="btn" @click="emit('back')">Back</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.load-game {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #1a202c;
  color: #e2e8f0;
  font-family: system-ui, sans-serif;
  z-index: 50;
}
.panel {
  width: min(90vw, 420px);
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
h2 {
  margin: 0;
  font-size: 1.5rem;
  font-weight: 600;
}
.hint {
  margin: 0;
  color: #a0aec0;
}
.save-list {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
}
.save-btn {
  display: flex;
  justify-content: space-between;
  gap: 1rem;
  background: #2d3748;
  color: #e2e8f0;
  border: 1px solid #4a5568;
  border-radius: 8px;
  padding: 12px 16px;
  cursor: pointer;
  text-align: left;
}
.save-btn:hover {
  background: #4a5568;
}
.save-label {
  font-weight: 500;
}
.save-date {
  color: #a0aec0;
  font-size: 0.9rem;
}
.actions {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
}
.btn {
  background: #2d3748;
  color: #e2e8f0;
  border: 1px solid #4a5568;
  border-radius: 8px;
  padding: 12px 20px;
  cursor: pointer;
  font-size: 15px;
}
.btn:hover {
  background: #4a5568;
}
</style>
