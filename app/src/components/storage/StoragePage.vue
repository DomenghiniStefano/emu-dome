<script setup lang="ts">
// «Gestisci lo spazio» (Trello #29, S1–S3, the owner's answers A A A): one screen, in the place
// of Settings › Salvataggi's page (from its `saves.space` row, and on the phone from E13's
// «Gestisci lo spazio»). On top the space card; then «Giochi, dal più pesante» (StorageGameRow:
// «Togli dal telefono» only on a game EmuDome downloaded that Drive has too, «Apri la cartella»
// on a user's folder); at the bottom «Altri dati»: the quick saves («Scegli»: the chooser), the
// screenshots («Apri»: the gallery), the temporary files («Svuota»). Every deletion asks first
// (StorageConfirm); a game's saves are never deleted here.
// Controller: the shell's nav scope; B goes back from the chooser or the gallery to the page, then
// to Settings › Salvataggi (this part's own `back`, nav.ts `useNavSection`).
import { computed, nextTick, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft, Eraser, Images, ListChecks, Trash2 } from '@lucide/vue'
import type { StorageGame } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/design-system/components/button'
import { Card, CardVariant } from '@/design-system/components/card'
import { Heading, HeadingVariant } from '@/design-system/components/heading'
import { useDeviceLayout } from '@/composables/useDeviceLayout'
import { sizeLabel } from '@/i18n/labels'
import { focusInitial, useNavSection } from '@/nav'
import QuickSavesChooser from './QuickSavesChooser.vue'
import ScreenshotsGallery from './ScreenshotsGallery.vue'
import SpaceCard from './SpaceCard.vue'
import StorageConfirm from './StorageConfirm.vue'
import StorageGameRow from './StorageGameRow.vue'
import { useStorage } from './useStorage'

const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()
const store = useStorage()
// The owner's words are the phone's («Togli dal telefono»); a computer says «dispositivo».
const { device } = useDeviceLayout()
const onPhone = computed(() => device.value === 'phone')
const removeLabel = computed(() =>
  onPhone.value ? t('storage.page.remove') : t('storage.page.removeHere'),
)
const removeTitle = (title: string) =>
  onPhone.value
    ? t('storage.page.askRemoveTitle', { title })
    : t('storage.page.askRemoveTitleHere', { title })

/** The page, or one of its two lists. */
type View = 'page' | 'quick' | 'shots'
const view = ref<View>('page')
/** The deletion waiting for its answer. */
const asking = ref<{ kind: 'game'; game: StorageGame } | { kind: 'temp' } | null>(null)

const root = ref<HTMLElement | null>(null)
async function show(v: View) {
  view.value = v
  await nextTick()
  if (root.value) focusInitial(root.value)
}
function back() {
  if (view.value !== 'page') void show('page')
  else emit('close')
}
useNavSection(root, {
  back: () => {
    back()
    return true
  },
})
onMounted(() => root.value && focusInitial(root.value))

async function confirm() {
  const a = asking.value
  asking.value = null
  if (a?.kind === 'game') await store.removeGame(a.game)
  else if (a?.kind === 'temp') await store.clearTemp()
}
</script>

<template>
  <div ref="root" class="flex flex-col gap-8">
    <div class="flex">
      <Button
        :variant="ButtonVariant.Ghost"
        :size="ButtonSize.TextLine"
        class="-ml-6"
        :icon="ChevronLeft"
        :data-nav-initial="view === 'page' ? '' : undefined"
        @click="back"
      >
        {{ view === 'page' ? t('settings.pages.saves.name') : t('storage.page.title') }}
      </Button>
    </div>

    <template v-if="view === 'page'">
      <Heading>{{ t('storage.page.title') }}</Heading>
      <template v-if="store.breakdown.value">
        <SpaceCard :breakdown="store.breakdown.value" />

        <Heading as="h2" :variant="HeadingVariant.Overline" class="mt-4">
          {{ t('storage.page.games') }}
        </Heading>
        <Card v-if="store.games.value.length > 0" :variant="CardVariant.List" as="ul">
          <StorageGameRow
            v-for="g in store.games.value"
            :key="g.id"
            :game="g"
            :action="store.actionOf(g)"
            :busy="store.busy.value"
            :remove-label="removeLabel"
            @remove="asking = { kind: 'game', game: g }"
            @open-folder="store.openFolder(g)"
          />
        </Card>
        <p v-else class="m-0 text-ink-muted">{{ t('storage.page.noGames') }}</p>
        <p class="m-0 text-caption text-ink-muted">{{ t('storage.page.yourFiles') }}</p>

        <Heading as="h2" :variant="HeadingVariant.Overline" class="mt-4">
          {{ t('storage.page.otherData') }}
        </Heading>
        <Card :variant="CardVariant.List" as="ul">
          <li class="flex items-center gap-8 px-10 py-6">
            <div class="flex min-w-0 flex-1 flex-col">
              <span class="font-medium">{{ t('storage.page.quick') }}</span>
              <span class="text-caption text-ink-muted tabular-nums">
                {{
                  t(
                    'storage.page.quickLine',
                    {
                      size: sizeLabel(store.breakdown.value.quickBytes),
                      slots: store.breakdown.value.quickSlots,
                      games: store.breakdown.value.quickGames,
                    },
                    store.breakdown.value.quickGames,
                  )
                }}
              </span>
            </div>
            <Button
              :size="ButtonSize.Small"
              :icon="ListChecks"
              :disabled="store.breakdown.value.quickSlots === 0"
              @click="show('quick')"
            >
              {{ t('storage.page.choose') }}
            </Button>
          </li>
          <li class="flex items-center gap-8 border-t border-line px-10 py-6">
            <div class="flex min-w-0 flex-1 flex-col">
              <span class="font-medium">{{ t('storage.page.shots') }}</span>
              <span class="text-caption text-ink-muted tabular-nums">
                {{
                  t(
                    'storage.page.shotsLine',
                    {
                      size: sizeLabel(store.breakdown.value.screenshotBytes),
                      n: store.breakdown.value.screenshotCount,
                    },
                    store.breakdown.value.screenshotCount,
                  )
                }}
              </span>
            </div>
            <Button
              :size="ButtonSize.Small"
              :icon="Images"
              :disabled="store.breakdown.value.screenshotCount === 0"
              @click="show('shots')"
            >
              {{ t('storage.page.open') }}
            </Button>
          </li>
          <li class="flex items-center gap-8 border-t border-line px-10 py-6">
            <div class="flex min-w-0 flex-1 flex-col">
              <span class="font-medium">{{ t('storage.page.temp') }}</span>
              <span class="text-caption text-ink-muted tabular-nums">
                {{
                  t('storage.page.tempLine', { size: sizeLabel(store.breakdown.value.tempBytes) })
                }}
              </span>
            </div>
            <Button
              :size="ButtonSize.Small"
              :icon="Eraser"
              :disabled="store.breakdown.value.tempBytes === 0 || store.busy.value"
              @click="asking = { kind: 'temp' }"
            >
              {{ t('storage.page.empty') }}
            </Button>
          </li>
        </Card>
      </template>
      <p v-else class="m-0 text-ink-muted">{{ t('storage.page.measuring') }}</p>
    </template>

    <template v-else-if="view === 'quick'">
      <Heading>{{ t('storage.page.quick') }}</Heading>
      <QuickSavesChooser @changed="store.load" />
    </template>

    <template v-else>
      <Heading>{{ t('storage.page.shots') }}</Heading>
      <ScreenshotsGallery
        :games="store.breakdown.value?.screenshotGames ?? []"
        @changed="store.load"
      />
    </template>

    <StorageConfirm
      v-if="asking?.kind === 'game'"
      :title="removeTitle(asking.game.title)"
      :text="t('storage.page.askRemoveText')"
      :confirm="removeLabel"
      :icon="Trash2"
      @confirm="confirm"
      @cancel="asking = null"
    />
    <StorageConfirm
      v-else-if="asking?.kind === 'temp'"
      :title="t('storage.page.askTempTitle')"
      :text="t('storage.page.askTempText')"
      :confirm="t('storage.page.empty')"
      :icon="Eraser"
      @confirm="confirm"
      @cancel="asking = null"
    />
  </div>
</template>
