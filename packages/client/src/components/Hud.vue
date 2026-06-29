<script setup lang="ts">
defineProps<{
  paused: boolean;
  speed: number;
  speedPresets: readonly number[];
}>();

const emit = defineEmits<{
  togglePause: [];
  setSpeed: [speed: number];
  save: [];
  load: [];
}>();
</script>

<template>
  <div class="hud">
    <button class="btn" @click="emit('togglePause')">
      {{ paused ? '▶ Resume' : '⏸ Pause' }}
    </button>
    <div class="speed-group">
      <span class="label">Speed</span>
      <button
        v-for="s in speedPresets"
        :key="s"
        class="btn"
        :class="{ active: speed === s }"
        @click="emit('setSpeed', s)"
      >
        {{ s }}×
      </button>
    </div>
    <div class="file-group">
      <button class="btn" @click="emit('save')">Save</button>
      <button class="btn" @click="emit('load')">Load</button>
    </div>
  </div>
</template>

<style scoped>
.hud {
  position: fixed;
  top: 12px;
  left: 12px;
  display: flex;
  gap: 12px;
  align-items: center;
  z-index: 10;
  font-family: system-ui, sans-serif;
}
.speed-group {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(26, 32, 44, 0.85);
  padding: 6px 10px;
  border-radius: 8px;
}
.label {
  color: #a0aec0;
  font-size: 13px;
  margin-right: 4px;
}
.btn {
  background: #2d3748;
  color: #e2e8f0;
  border: 1px solid #4a5568;
  border-radius: 6px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 13px;
}
.btn:hover { background: #4a5568; }
.btn.active {
  background: #3182ce;
  border-color: #4299e1;
}
.file-group {
  display: flex;
  gap: 6px;
}
</style>
