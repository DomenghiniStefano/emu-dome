<script setup lang="ts">
import { ref } from 'vue'
import { Camera, ChevronRight, Rows2, VolumeX } from '@lucide/vue'
import { Card, CardDescription, CardPad, CardTitle, CardVariant } from '@/components/ui/card'
import { StatTile } from '@/components/ui/stat-tile'
import { LinkRow } from '@/components/ui/link-row'
import KitSection from '../KitSection.vue'
import KitState from '../KitState.vue'

const chosen = ref('normal')
const picture = ref('generated')
const row = ref('Pocket Quest')
const device = ref('pad')
const choice = ref('nintendo')
const slot = ref(1)
const muted = ref(true)
</script>

<template>
  <KitSection title="Card">
    <KitState label="Surface">
      <Card class="w-field gap-10">
        <CardTitle>Files</CardTitle>
        <CardDescription>The game file and where its saves are.</CardDescription>
      </Card>
    </KitState>
    <KitState label="Surface · pad Snug (12), Tight (10)">
      <div class="flex gap-10">
        <Card :pad="CardPad.Snug" class="w-field gap-6">
          <CardTitle>Snug</CardTitle>
          <CardDescription>A line with its action.</CardDescription>
        </Card>
        <Card :pad="CardPad.Tight" class="w-field gap-6">
          <CardTitle>Tight</CardTitle>
          <CardDescription>A slot tile, a banner.</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="StatTile · a count per console">
      <div class="grid w-dialog grid-cols-4 gap-10">
        <StatTile :value="12" label="Game Boy Advance" />
        <StatTile :value="3" label="Nintendo DS" />
        <StatTile :value="0" label="Nintendo 3DS" />
        <StatTile :value="7" label="Game Boy and Color" />
      </div>
    </KitState>
    <KitState label="Detail: desktop, and on the phone (rounder, tighter)">
      <div class="flex gap-10">
        <Card :variant="CardVariant.Detail" class="w-field gap-10">
          <CardTitle>Files</CardTitle>
          <CardDescription>A card of the game details.</CardDescription>
        </Card>
        <div data-density="touch" class="flex">
          <Card :variant="CardVariant.Detail" class="w-field gap-6">
            <CardTitle>Files</CardTitle>
            <CardDescription>A card of the game details.</CardDescription>
          </Card>
        </div>
      </div>
    </KitState>
    <KitState label="Raised · a choice (as button), selected and not">
      <div class="flex gap-10">
        <Card
          v-for="id in ['normal', 'wide']"
          :key="id"
          as="button"
          :variant="CardVariant.Raised"
          :selected="chosen === id"
          role="radio"
          :aria-checked="chosen === id"
          class="w-field cursor-pointer gap-6"
          @click="chosen = id"
        >
          <CardTitle>{{ id }}</CardTitle>
          <CardDescription>One of a choice of cards.</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="Choice · on a page (first run A/B), selected and not">
      <div class="flex gap-10">
        <Card
          v-for="id in ['nintendo', 'printed']"
          :key="id"
          as="button"
          :variant="CardVariant.Choice"
          :selected="choice === id"
          role="radio"
          :aria-checked="choice === id"
          class="w-field gap-6"
          @click="choice = id"
        >
          <CardTitle>{{ id }}</CardTitle>
          <CardDescription>One of a choice of cards.</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="RaisedAccent · the chosen one's text in accent too (scope levels)">
      <div class="flex gap-10">
        <Card
          v-for="id in ['normal', 'wide']"
          :key="id"
          as="button"
          :variant="CardVariant.RaisedAccent"
          :selected="chosen === id"
          role="radio"
          :aria-checked="chosen === id"
          class="w-field cursor-pointer gap-2"
          @click="chosen = id"
        >
          <span class="font-semibold">{{ id }}</span>
          <CardDescription>One level among a few.</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="PictureChoice · selected, not, empty (data-empty)">
      <div class="flex gap-10">
        <Card
          v-for="id in ['generated', 'icon', 'user']"
          :key="id"
          as="button"
          :variant="CardVariant.PictureChoice"
          :selected="picture === id"
          :data-empty="id === 'user' ? '' : undefined"
          role="radio"
          :aria-checked="picture === id"
          class="w-cover-hero-touch cursor-pointer gap-8"
          @click="picture = id"
        >
          <CardTitle>{{ id }}</CardTitle>
          <CardDescription>One picture of a choice.</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="ListItem · a list's rows (as button), selected and not">
      <div class="flex w-field flex-col gap-2">
        <Card
          v-for="id in ['Pocket Quest', 'Star Pilot']"
          :key="id"
          as="button"
          :variant="CardVariant.ListItem"
          :selected="row === id"
          @click="row = id"
        >
          <span class="flex min-w-0 flex-1 flex-col gap-2">
            <span class="font-medium">{{ id }}</span>
            <span class="text-caption text-ink-muted">1 quick save · 384 kB</span>
          </span>
        </Card>
      </div>
    </KitState>
    <KitState label="Choice · selected and not">
      <div class="flex gap-10">
        <Card
          v-for="id in ['pad', 'keyboard']"
          :key="id"
          as="button"
          :variant="CardVariant.Choice"
          :selected="device === id"
          role="radio"
          :aria-checked="device === id"
          class="w-field cursor-pointer gap-2"
          @click="device = id"
        >
          <CardTitle>{{ id }}</CardTitle>
          <CardDescription>One device of a few.</CardDescription>
        </Card>
      </div>
    </KitState>
    <KitState label="List · with LinkRow (button, value, disabled, div)">
      <Card :variant="CardVariant.List" class="w-dialog">
        <LinkRow name="Remap buttons" sub="Change what each button does" />
        <LinkRow name="Screen effect" sub="Video" value="CRT" />
        <LinkRow name="Sticks and triggers" sub="Dead zone 15%" disabled />
        <LinkRow as="div" name="Touch controls" sub="Customised on the phone" />
      </Card>
    </KitState>
    <KitState label="Tile · a slot to choose (as button), chosen and not; chosen on the phone">
      <div class="flex gap-8">
        <Card
          v-for="n in [1, 2]"
          :key="n"
          as="button"
          :variant="CardVariant.Tile"
          :selected="slot === n"
          class="w-cover-hero-touch gap-4"
          @click="slot = n"
        >
          <CardTitle>Slot {{ n }}</CardTitle>
          <CardDescription>18/11/26</CardDescription>
        </Card>
        <div data-density="touch" class="flex">
          <Card as="button" :variant="CardVariant.Tile" selected class="w-cover-hero-touch gap-3">
            <CardTitle>Slot 3</CardTitle>
            <CardDescription>Phone</CardDescription>
          </Card>
        </div>
      </div>
    </KitState>
    <KitState label="Tool (on, off, disabled), and on the phone (off, on)">
      <div class="flex gap-8">
        <Card
          as="button"
          :variant="CardVariant.Tool"
          :selected="muted"
          :aria-pressed="muted"
          class="w-cover-hero-touch gap-6"
          @click="muted = !muted"
        >
          <VolumeX :size="20" :stroke-width="1.75" aria-hidden="true" />
          Mute
        </Card>
        <Card as="button" :variant="CardVariant.Tool" class="w-cover-hero-touch gap-6">
          <Camera :size="20" :stroke-width="1.75" aria-hidden="true" />
          Screenshot
        </Card>
        <Card as="button" :variant="CardVariant.Tool" disabled class="w-cover-hero-touch gap-6">
          <Camera :size="20" :stroke-width="1.75" aria-hidden="true" />
          Not yet
        </Card>
        <div data-density="touch" class="flex gap-8">
          <Card as="button" :variant="CardVariant.Tool" class="w-stat-min gap-8">
            <Camera :size="18" :stroke-width="1.75" aria-hidden="true" />
            Screenshot
          </Card>
          <Card as="button" :variant="CardVariant.Tool" selected class="w-stat-min gap-8">
            <VolumeX :size="18" :stroke-width="1.75" aria-hidden="true" />
            Mute
          </Card>
        </div>
      </div>
    </KitState>
    <KitState label="NavRow (and disabled), ListRow">
      <div class="flex w-field flex-col gap-8">
        <Card as="button" :variant="CardVariant.NavRow" class="gap-12">
          <Rows2 :size="18" :stroke-width="1.75" aria-hidden="true" />
          <span class="flex-1">Screens and picture</span>
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Card>
        <Card as="button" :variant="CardVariant.NavRow" disabled class="gap-12">
          <span class="flex-1">Not here</span>
        </Card>
        <Card as="button" :variant="CardVariant.ListRow" class="gap-12">
          <span class="flex-1">Controller shortcuts</span>
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Card>
      </div>
    </KitState>
  </KitSection>
</template>
