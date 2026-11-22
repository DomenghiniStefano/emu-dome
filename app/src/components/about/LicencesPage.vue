<script setup lang="ts">
// `about.lic` (Trello #44), a functional placeholder until the owner designs it: the Settings
// page's place shows EmuDome, the cores this build ships and the other components, each with its
// licence and version; choosing one shows its licence files' text. Built from existing primitives
// only (the page's heading, a list card of LinkRow, a card of text).
// Controller: the shell's nav scope; B goes back from a text to the list, then to «Informazioni»
// (this part's own `back`, nav.ts `useNavSection`).
import { nextTick, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { ChevronLeft } from '@lucide/vue'
import type { Licence } from '@/emu'
import { Button, ButtonSize, ButtonVariant } from '@/components/ui/button'
import { Card, CardVariant } from '@/components/ui/card'
import { Heading, HeadingVariant } from '@/components/ui/heading'
import { LinkRow } from '@/components/ui/link-row'
import { useSay } from '@/i18n/message'
import { focusInitial, useNavSection } from '@/nav'
import { licenceLine, licenceName, paragraphs } from './licences'
import { useLicences } from './useLicences'

const emit = defineEmits<{ close: [] }>()
const { t } = useI18n()
const say = useSay()
const { rows, opened, texts, open, close } = useLicences()

const name = (l: Licence) => {
  const n = licenceName(l)
  return typeof n === 'string' ? n : say(n)
}

const root = ref<HTMLElement | null>(null)
function back() {
  if (opened.value) {
    close()
    void nextTick(() => root.value && focusInitial(root.value))
  } else emit('close')
}
useNavSection(root, {
  back: () => {
    back()
    return true
  },
})
async function show(l: Licence) {
  await open(l)
  await nextTick()
  if (root.value) focusInitial(root.value)
}
onMounted(() => root.value && focusInitial(root.value))
</script>

<template>
  <div ref="root" class="flex flex-col gap-12">
    <Button
      :variant="ButtonVariant.Text"
      :size="ButtonSize.Bare"
      class="self-start"
      :data-nav-initial="opened ? '' : undefined"
      @click="back"
    >
      <ChevronLeft :size="16" :stroke-width="1.75" aria-hidden="true" />
      {{ opened ? t('about.licences.title') : t('settings.pages.about.name') }}
    </Button>

    <template v-if="!opened">
      <div class="flex flex-col gap-4">
        <Heading>{{ t('about.licences.title') }}</Heading>
        <div class="text-ink-muted">{{ t('about.licences.intro') }}</div>
      </div>
      <Card :variant="CardVariant.List">
        <LinkRow
          v-for="(l, i) in rows"
          :key="l.id"
          :data-nav-initial="i === 0 ? '' : undefined"
          :name="name(l)"
          :sub="say(licenceLine(l))"
          @click="show(l)"
        />
      </Card>
    </template>

    <template v-else>
      <div class="flex flex-col gap-4">
        <Heading>{{ name(opened) }}</Heading>
        <div class="text-ink-muted">{{ say(licenceLine(opened)) }}</div>
        <div class="text-caption break-all text-ink-muted">
          {{ t('about.licences.source', { url: opened.source }) }}
        </div>
      </div>
      <Card v-for="x in texts" :key="x.file" class="gap-8">
        <Heading :variant="HeadingVariant.Overline" as="h3">{{ x.file }}</Heading>
        <!-- One focusable block per paragraph: the d-pad steps through a long text, which
             scrolls with the focus. -->
        <p
          v-for="(para, i) in paragraphs(x.text)"
          :key="i"
          tabindex="0"
          class="m-0 font-mono text-caption break-words whitespace-pre-wrap"
        >
          {{ para }}
        </p>
      </Card>
    </template>
  </div>
</template>
