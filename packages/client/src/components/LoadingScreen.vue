<script setup lang="ts">
defineProps<{
  visible: boolean;
  error?: string | null;
}>();

const emit = defineEmits<{
  back: [];
}>();
</script>

<template>
  <div v-if="visible" class="loading">
    <div v-if="!error" class="spinner" />
    <p v-if="error" class="error">{{ error }}</p>
    <p v-else>Loading colony simulation…</p>
    <button v-if="error" class="back-btn" @click="emit('back')">Back to menu</button>
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
.error {
  color: #fc8181;
  max-width: min(90vw, 480px);
  text-align: center;
  line-height: 1.4;
}
.back-btn {
  margin-top: 8px;
  background: #2d3748;
  color: #e2e8f0;
  border: 1px solid #4a5568;
  border-radius: 6px;
  padding: 8px 16px;
  cursor: pointer;
  font-size: 14px;
}
</style>
