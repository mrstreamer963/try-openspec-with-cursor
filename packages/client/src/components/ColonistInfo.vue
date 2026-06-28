<script setup lang="ts">
import type { ColonistSnapshot } from '../game/types';

defineProps<{
  colonist: ColonistSnapshot | null;
}>();
</script>

<template>
  <div v-if="colonist" class="panel">
    <h3>Colonist #{{ colonist.id }}</h3>
    <dl>
      <dt>Position</dt>
      <dd>{{ colonist.x }}, {{ colonist.y }}</dd>
      <dt>Food</dt>
      <dd>
        <div class="bar"><div class="fill food" :style="{ width: colonist.food + '%' }" /></div>
        {{ Math.round(colonist.food) }}
      </dd>
      <dt>Sleep</dt>
      <dd>
        <div class="bar"><div class="fill sleep" :style="{ width: colonist.sleep + '%' }" /></div>
        {{ Math.round(colonist.sleep) }}
      </dd>
      <dt>Task</dt>
      <dd>{{ colonist.task }}</dd>
    </dl>
  </div>
</template>

<style scoped>
.panel {
  position: fixed;
  top: 12px;
  right: 12px;
  width: 220px;
  background: rgba(26, 32, 44, 0.92);
  color: #e2e8f0;
  border: 1px solid #4a5568;
  border-radius: 10px;
  padding: 14px;
  z-index: 10;
  font-family: system-ui, sans-serif;
  font-size: 13px;
}
h3 {
  margin: 0 0 10px;
  font-size: 15px;
}
dl {
  margin: 0;
  display: grid;
  grid-template-columns: auto 1fr;
  gap: 6px 12px;
}
dt { color: #a0aec0; }
dd { margin: 0; }
.bar {
  height: 6px;
  background: #2d3748;
  border-radius: 3px;
  margin-bottom: 2px;
  overflow: hidden;
}
.fill { height: 100%; border-radius: 3px; }
.fill.food { background: #e53e3e; }
.fill.sleep { background: #9f7aea; }
</style>
