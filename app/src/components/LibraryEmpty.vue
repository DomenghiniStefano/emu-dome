<script setup lang="ts">
// Library with no games (design L02, Stati board): choose a folder (the backend adds and scans
// it), drag games or folders onto the window (App.vue), or open a single file the Phase 0 way.
import { useI18n } from 'vue-i18n'
import { Library } from '@lucide/vue'

defineProps<{ error: string | null }>()
const emit = defineEmits<{ chooseFolder: []; openFile: [] }>()

const { t } = useI18n()
</script>

<template>
  <div class="wrap">
    <section class="empty">
      <Library class="icon" :size="44" :stroke-width="1.5" aria-hidden="true" />
      <h1 class="title">{{ t('library.emptyTitle') }}</h1>
      <p class="lead">{{ t('library.emptyLead') }}</p>
      <div class="actions">
        <button class="btn btn-primary" data-nav-initial @click="emit('chooseFolder')">
          {{ t('library.chooseFolder') }}
        </button>
        <button class="btn" @click="emit('openFile')">
          {{ t('library.openFile') }}
        </button>
      </div>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
    </section>
  </div>
</template>

<style scoped>
.wrap {
  flex: 1;
  display: grid;
  place-items: center;
}
.empty {
  max-width: 480px;
  padding: 48px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 14px;
  text-align: center;
  border: 1px dashed var(--line-strong);
  border-radius: var(--radius-16);
}
.icon {
  color: var(--accent-text);
}
.title {
  margin: 0;
  font-size: 20px;
  line-height: 26px;
  font-weight: 600;
}
.lead,
.error {
  margin: 0;
}
.lead {
  color: var(--ink-muted);
}
.actions {
  display: flex;
  gap: 8px;
}
.actions .btn {
  height: 36px;
  white-space: nowrap;
}
.actions .btn-primary {
  padding: 0 16px;
  font-size: 13px;
}
.error {
  color: var(--danger);
}
</style>
