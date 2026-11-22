<script setup lang="ts">
import { ref } from 'vue'
import { Camera, ChevronRight, Rows2, VolumeX } from '@lucide/vue'
import { Card, CardDescription, CardPad, CardTitle, CardVariant } from '@/components/ui/card'
import { StatTile } from '@/components/ui/stat-tile'
import { LinkRow } from '@/components/ui/link-row'
import KitIssue from '@/kit/KitIssue.vue'
import KitSection from '@/kit/KitSection.vue'
import KitState from '@/kit/KitState.vue'
import { IssueStatus } from '@/kit/review'

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
      <Card class="w-field gap-6">
        <CardTitle>Files</CardTitle>
        <CardDescription>The game file and where its saves are.</CardDescription>
      </Card>
    </KitState>
    <KitState label="Surface · pad Snug (8), Tight (6)">
      <div class="flex flex-wrap gap-6">
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
      <div class="grid w-dialog max-w-full grid-cols-4 gap-6">
        <StatTile :value="12" label="Game Boy Advance" />
        <StatTile :value="3" label="Nintendo DS" />
        <StatTile :value="0" label="Nintendo 3DS" />
        <StatTile :value="7" label="Game Boy and Color" />
      </div>
    </KitState>
    <KitState label="Detail: desktop, and on the phone (panel corners, the phone room)">
      <div class="flex flex-wrap gap-6">
        <Card :variant="CardVariant.Detail" class="w-field gap-6">
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
      <div class="flex flex-wrap gap-6">
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
      <div class="flex flex-wrap gap-6">
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
      <div class="flex flex-wrap gap-6">
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
      <div class="flex flex-wrap gap-6">
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
      <div class="flex flex-wrap gap-6">
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
      <Card :variant="CardVariant.List" class="w-dialog max-w-full">
        <LinkRow name="Remap buttons" sub="Change what each button does" />
        <LinkRow name="Screen effect" sub="Video" value="CRT" />
        <LinkRow name="Sticks and triggers" sub="Dead zone 15%" disabled />
        <LinkRow as="div" name="Touch controls" sub="Customised on the phone" />
      </Card>
    </KitState>
    <KitState label="Tile · a slot to choose (as button), chosen and not; chosen on the phone">
      <div class="flex flex-wrap gap-8">
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
      <div class="flex flex-wrap gap-8">
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
        <div data-density="touch" class="flex flex-wrap gap-8">
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
        <Card as="button" :variant="CardVariant.NavRow" class="gap-8">
          <Rows2 :size="18" :stroke-width="1.75" aria-hidden="true" />
          <span class="flex-1">Screens and picture</span>
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Card>
        <Card as="button" :variant="CardVariant.NavRow" disabled class="gap-8">
          <span class="flex-1">Not here</span>
        </Card>
        <Card as="button" :variant="CardVariant.ListRow" class="gap-8">
          <span class="flex-1">Controller shortcuts</span>
          <ChevronRight :size="16" :stroke-width="1.75" aria-hidden="true" />
        </Card>
      </div>
    </KitState>
    <template #review>
      <KitIssue
        title="Raised e RaisedAccent differiscono per una classe"
        :status="IssueStatus.Proposed"
      >
        Le due varianti hanno le stesse classi tranne una: la card scelta di RaisedAccent ha anche
        il testo in accento. Ognuna ha un solo uso (Raised: le card di scelta di un'impostazione,
        SettingCards; RaisedAccent: la barra dei livelli, SettingsScopeBar), due schermate della
        stessa pagina Impostazioni che così mostrano la scelta in due modi.
        <template #current>
          <Card :variant="CardVariant.Raised" selected class="w-field gap-2">
            <CardTitle>Raised</CardTitle>
            <CardDescription>Scelta, testo in inchiostro</CardDescription>
          </Card>
          <Card :variant="CardVariant.RaisedAccent" selected class="w-field gap-2">
            <span class="font-semibold">RaisedAccent</span>
            <CardDescription>Scelta, testo in accento</CardDescription>
          </Card>
        </template>
        <template #proposed>
          <Card :variant="CardVariant.RaisedAccent" selected class="w-field gap-2">
            <span class="font-semibold">Raised</span>
            <CardDescription>Scelta, testo in accento</CardDescription>
          </Card>
          <Card :variant="CardVariant.RaisedAccent" selected class="w-field gap-2">
            <span class="font-semibold">RaisedAccent</span>
            <CardDescription>Scelta, testo in accento</CardDescription>
          </Card>
        </template>
        <template #solution>
          Una sola variante Raised in card/variants.ts, con il testo in accento quando è scelta (o
          sempre in inchiostro: lo decide il proprietario); RaisedAccent sparisce.
        </template>
      </KitIssue>
      <KitIssue title="Hover solo su metà delle card cliccabili">
        Choice, ListItem, Tool, NavRow e ListRow si schiariscono sotto il mouse; Raised,
        RaisedAccent, PictureChoice e Tile no, anche quando sono pulsanti (as="button"). Passa il
        mouse sugli stati qui sopra: «normal / wide» e gli slot non reagiscono, «pad / keyboard» sì.
        <template #solution>
          Ogni variante che può essere un pulsante prende lo stesso hover di Choice
          (enabled:hover:not-data-selected:bg-surface-hover) in card/variants.ts.
        </template>
      </KitIssue>
    </template>
  </KitSection>
</template>
