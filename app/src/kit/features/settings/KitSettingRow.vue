<script setup lang="ts">
// One real SettingRow on the Kit page with its own little store: a change sets the value at the
// row's level, «Ripristina» / «Usa il valore di …» puts it back where it came from. The page's
// props (schema, level names) are the mock's.
import { ref } from 'vue'
import type { ConsoleId, SettingEntry, SettingLevel, SettingValue } from '@/emu'
import SettingRowView from '@/components/settings/SettingRow.vue'
import { levelName, row, schema } from './mock'

const props = withDefaults(
  defineProps<{
    entry: SettingEntry
    value?: SettingValue | null
    /** Where the value comes from (Default unless told). */
    source?: SettingLevel
    level?: SettingLevel
    scoped?: boolean
    consoleId?: ConsoleId | null
    compact?: boolean
    lockNote?: boolean
  }>(),
  {
    value: undefined,
    source: 'default',
    level: 'global',
    scoped: false,
    consoleId: null,
    compact: false,
    lockNote: false,
  },
)

const initial = () =>
  row(props.entry, props.value === undefined ? props.entry.default : props.value, props.source)
const current = ref(initial())

function set(v: SettingValue) {
  current.value = { ...current.value, value: v, source: props.level, changed: true }
}
function reset() {
  current.value = row(props.entry)
}
</script>

<template>
  <SettingRowView
    :entry="entry"
    :row="current"
    :schema="schema"
    :level="level"
    :level-name="levelName"
    :scoped="scoped"
    :console="consoleId"
    :compact="compact"
    :lock-note="lockNote"
    @set="set"
    @reset="reset"
  />
</template>
