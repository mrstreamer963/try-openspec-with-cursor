<script setup lang="ts">
import type { ColonistSnapshot } from '../game/types';

defineProps<{
  colonist: ColonistSnapshot | null;
}>();
</script>

<template>
  <div v-if="colonist" class="panel">
    <h3>{{ colonist.name }} (#{{ colonist.id }})</h3>
    <dl>
      <dt>Position</dt>
      <dd>{{ Math.floor(colonist.x) }}, {{ Math.floor(colonist.y) }}</dd>
      <dt :class="{ critical: colonist.hungry }">Food</dt>
      <dd :class="{ critical: colonist.hungry }">
        <div class="bar"><div class="fill food" :style="{ width: colonist.food + '%' }" /></div>
        {{ Math.round(colonist.food) }}
        <span v-if="colonist.hungry" class="status hungry">Hungry</span>
      </dd>
      <dt :class="{ critical: colonist.wants_sleep }">Sleep</dt>
      <dd :class="{ critical: colonist.wants_sleep }">
        <div class="bar"><div class="fill sleep" :style="{ width: colonist.sleep + '%' }" /></div>
        {{ Math.round(colonist.sleep) }}
        <span v-if="colonist.wants_sleep" class="status sleep">Wants sleep</span>
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
dt.critical { color: #fc8181; font-weight: 600; }
dd { margin: 0; }
dd.critical { color: #fed7d7; }
.status {
  display: inline-block;
  margin-top: 4px;
  padding: 1px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
}
.status.hungry {
  color: #fed7d7;
  background: rgba(229, 62, 62, 0.25);
}
.status.sleep {
  color: #e9d8fd;
  background: rgba(159, 122, 234, 0.25);
}
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
