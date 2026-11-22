<script setup lang="ts">
// One issue under a section of the Kit page: the problem in words, what it looks like now and the
// proposed solution side by side (both rendered with the real components: the proposal composes
// the primitives with other props or classes, or lives in kit/proposals/ until the owner decides),
// then the solution in words. The two sides wrap under each other when they do not fit (the
// theme resets Tailwind's breakpoints, so no `md:`). Written in the section's own file, in the #review slot of
// KitSection; a decided issue is deleted once the change is in the component.
import { Badge, BadgeVariant } from '@/components/ui/badge'
import { IssueStatus, ISSUE_STATUS_LABELS, useKitIssue } from '@/kit/review'

const props = withDefaults(defineProps<{ title: string; status?: IssueStatus }>(), {
  status: IssueStatus.Open,
})
useKitIssue(() => props.status)

const STATUS_BADGE: Record<IssueStatus, BadgeVariant> = {
  [IssueStatus.Open]: BadgeVariant.Warning,
  [IssueStatus.Proposed]: BadgeVariant.Neutral,
  [IssueStatus.Decided]: BadgeVariant.Success,
}
</script>

<template>
  <article class="flex flex-col gap-8 rounded-card border border-line bg-surface p-card-pad">
    <header class="flex flex-wrap items-center gap-8">
      <Badge :variant="STATUS_BADGE[status]">{{ ISSUE_STATUS_LABELS[status] }}</Badge>
      <h3 class="m-0 text-body font-semibold text-ink">{{ title }}</h3>
    </header>
    <div class="text-body text-ink-muted"><slot /></div>
    <div v-if="$slots.current || $slots.proposed" class="flex flex-wrap gap-12">
      <figure v-if="$slots.current" class="m-0 flex min-w-0 flex-1 basis-field flex-col gap-4">
        <figcaption class="text-caption text-ink-faint">Ora</figcaption>
        <div class="flex flex-wrap items-start gap-8"><slot name="current" /></div>
      </figure>
      <figure v-if="$slots.proposed" class="m-0 flex min-w-0 flex-1 basis-field flex-col gap-4">
        <figcaption class="text-caption text-ink-faint">Proposta</figcaption>
        <div class="flex flex-wrap items-start gap-8"><slot name="proposed" /></div>
      </figure>
    </div>
    <div v-if="$slots.solution" class="text-body text-ink">
      <span class="font-semibold">Soluzione: </span><slot name="solution" />
    </div>
  </article>
</template>
