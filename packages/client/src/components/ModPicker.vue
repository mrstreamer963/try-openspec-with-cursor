<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { ModCatalogEntry } from '../content/modCatalog';

const props = defineProps<{
  catalog: ModCatalogEntry[];
  enabledMods: string[];
}>();

const emit = defineEmits<{
  apply: [enabledMods: string[]];
  back: [];
}>();

const toggled = ref<string[]>([...props.enabledMods]);

watch(
  () => props.enabledMods,
  (mods) => {
    toggled.value = [...mods];
  },
);

const sortedCatalog = computed(() =>
  [...props.catalog].sort((a, b) => {
    if (a.id === 'base') return -1;
    if (b.id === 'base') return 1;
    return a.id.localeCompare(b.id);
  }),
);

function isEnabled(id: string): boolean {
  return toggled.value.includes(id);
}

function toggleMod(id: string): void {
  if (id === 'base') return;
  if (isEnabled(id)) {
    toggled.value = toggled.value.filter((m) => m !== id);
  } else {
    toggled.value = [...toggled.value, id];
  }
}

function apply(): void {
  const mods = ['base', ...toggled.value.filter((m) => m !== 'base')];
  emit('apply', mods);
}
</script>

<template>
  <div class="mod-picker">
    <div class="panel">
      <h2>Mods</h2>
      <ul class="mod-list">
        <li v-for="mod in sortedCatalog" :key="mod.id" class="mod-row">
          <label class="mod-label">
            <input
              type="checkbox"
              :checked="isEnabled(mod.id)"
              :disabled="mod.id === 'base'"
              @change="toggleMod(mod.id)"
            />
            <span class="mod-id">{{ mod.id }}</span>
            <span class="mod-source">{{ mod.source }}</span>
          </label>
        </li>
      </ul>
      <p class="hint">Mod changes apply to the next game session.</p>
      <div class="actions">
        <button class="btn primary" @click="apply">Apply</button>
        <button class="btn" @click="emit('back')">Back</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.mod-picker {
  position: fixed;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  z-index: 60;
  font-family: system-ui, sans-serif;
}
.panel {
  background: #1a202c;
  color: #e2e8f0;
  border-radius: 12px;
  padding: 24px;
  min-width: min(90vw, 420px);
  max-height: 80vh;
  overflow-y: auto;
}
h2 {
  margin: 0 0 16px;
}
.mod-list {
  list-style: none;
  margin: 0 0 12px;
  padding: 0;
}
.mod-row {
  padding: 8px 0;
  border-bottom: 1px solid #2d3748;
}
.mod-label {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
}
.mod-id {
  font-weight: 500;
}
.mod-source {
  margin-left: auto;
  font-size: 12px;
  color: #a0aec0;
  text-transform: capitalize;
}
.hint {
  font-size: 13px;
  color: #a0aec0;
  margin: 0 0 16px;
}
.actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.btn {
  background: #2d3748;
  color: #e2e8f0;
  border: 1px solid #4a5568;
  border-radius: 6px;
  padding: 8px 14px;
  cursor: pointer;
  font-size: 14px;
}
.btn:hover {
  background: #4a5568;
}
.btn.primary {
  background: #3182ce;
  border-color: #4299e1;
}
</style>
