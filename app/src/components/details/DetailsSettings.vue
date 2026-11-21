<script setup lang="ts">
// Game details › Impostazioni del gioco (L06, board «Dettagli» desktop and «Tel-Dettagli»): only
// the settings this game has a value of its own for (`settings_game_overrides`), each an
// OverrideRow; above them «Apri tutte le impostazioni di questo gioco», the Settings section at
// «Solo questo gioco».
//
// Controller / keyboard: the shell's nav scope.
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  emu,
  type GameOverride,
  type LibraryGame,
  type SettingKey,
  type SettingsSchema,
} from '@/emu'
import { useSettingsChanged } from '@/composables/useSettingsChanged'
import { gameScopeOf } from '@/lib/gameDetails'
import { useErrorSink } from '@/composables/useToasts'
import { loadSchema } from '@/lib/settingsModel'
import { Button, ButtonFade, ButtonSize, ButtonVariant } from '@/components/ui/button'
import DetailsCard from './DetailsCard.vue'
import { overrideRows } from './gameOverrides'
import OverrideRow from './OverrideRow.vue'

const props = defineProps<{ game: LibraryGame; phone: boolean }>()
const emit = defineEmits<{
  /** «Apri tutte le impostazioni di questo gioco». */
  open: []
}>()

const { t, te } = useI18n()
const failWith = useErrorSink()

const scope = computed(() => gameScopeOf(props.game))
const consoleName = computed(() => t(`console.${props.game.console}`))
const schema = ref<SettingsSchema | null>(null)
const overrides = ref<GameOverride[]>([])
const loaded = ref(false)
const rows = computed(() => overrideRows(overrides.value, schema.value, { t, te }))

async function load() {
  const s = scope.value
  if (!s) {
    overrides.value = []
    loaded.value = true
    return
  }
  try {
    schema.value ??= await loadSchema()
    const found = await emu.settings.gameOverrides(s)
    // Rule 7: the details show another game now; its own load shows.
    if (scope.value.game !== s.game) return
    overrides.value = found
  } catch (e) {
    if (scope.value.game !== s.game) return
    failWith(e)
  }
  loaded.value = true
}
watch(scope, () => void load())

async function reset(key: SettingKey) {
  const s = scope.value
  if (!s) return
  try {
    await emu.settings.reset(key, s)
  } catch (e) {
    failWith(e)
  }
  await load()
}

onMounted(() => void load())
useSettingsChanged(
  c => c.level === 'game',
  () => void load(),
)
</script>

<template>
  <div class="flex flex-col gap-12">
    <div
      class="flex items-center justify-between gap-12 touch:flex-col touch:items-start touch:justify-start"
    >
      <p class="m-0 text-ink-muted">
        {{
          scope
            ? t('details.settings.lead', { console: consoleName })
            : t('details.settings.noIdentity')
        }}
      </p>
      <Button
        :variant="ButtonVariant.Link"
        :size="ButtonSize.Bare"
        :fade="ButtonFade.None"
        class="flex-none"
        :disabled="!scope"
        @click="emit('open')"
      >
        {{ t('details.settings.openAll') }}
      </Button>
    </div>
    <DetailsCard v-if="rows.length" class="divide-y divide-line overflow-hidden p-0 touch:p-0">
      <OverrideRow
        v-for="r in rows"
        :key="r.key"
        :row="r"
        :console-name="consoleName"
        :phone="phone"
        @reset="reset(r.key)"
      />
    </DetailsCard>
    <DetailsCard v-else-if="loaded && scope" as="p" class="m-0 text-ink-muted">
      {{ t('details.settings.none', { console: consoleName }) }}
    </DetailsCard>
  </div>
</template>
