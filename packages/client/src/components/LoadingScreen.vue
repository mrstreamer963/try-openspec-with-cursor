<script setup lang="ts">
defineProps<{
  visible: boolean;
}>();
</script>

<template>
  <div v-if="visible" class="loading">
    <div class="spinner" />
    <p>Loading colony simulation…</p>
  </div>
</template>

<style scoped>
.loading {
  position: fixed;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1rem;
  background: #1a202c;
  color: #e2e8f0;
  z-index: 100;
  font-family: system-ui, sans-serif;
}
.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid #4a5568;
  border-top-color: #63b3ed;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>
