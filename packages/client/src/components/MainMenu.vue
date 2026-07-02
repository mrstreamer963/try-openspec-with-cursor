<script setup lang="ts">
defineProps<{
  canContinue: boolean;
}>();

const emit = defineEmits<{
  continue: [];
  newGame: [];
  loadGame: [];
  mods: [];
}>();
</script>

<template>
  <div class="main-menu">
    <h1>Idle Colony Sim</h1>
    <div class="actions">
      <button class="btn primary" :disabled="!canContinue" @click="emit('continue')">
        Continue
      </button>
      <button class="btn" @click="emit('newGame')">New Game</button>
      <button class="btn" @click="emit('loadGame')">Load Game</button>
      <button class="btn" @click="emit('mods')">Mods</button>
    </div>
  </div>
</template>

<style scoped>
.main-menu {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2rem;
  background: #1a202c;
  color: #e2e8f0;
  font-family: system-ui, sans-serif;
  z-index: 50;
}
h1 {
  margin: 0;
  font-size: 2rem;
  font-weight: 600;
}
.actions {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  min-width: 220px;
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
.btn:hover:not(:disabled) {
  background: #4a5568;
}
.btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
.btn.primary {
  background: #3182ce;
  border-color: #4299e1;
}
.btn.primary:hover:not(:disabled) {
  background: #4299e1;
}
</style>
