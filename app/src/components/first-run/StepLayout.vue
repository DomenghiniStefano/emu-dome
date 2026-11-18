<script setup lang="ts">
// One step of the first run: a column, centred (S01, S06) or left-aligned, at most 560 wide
// (S02, S04, S06) and the full width on the phone.
defineProps<{ centred?: boolean; wide?: boolean; phone: boolean }>()
</script>

<template>
  <section
    class="my-16 flex flex-col gap-16"
    :class="[
      centred && 'items-center text-center',
      wide && (phone ? 'w-full' : 'w-[min(560px,100%)]'),
    ]"
  >
    <slot />
  </section>
</template>
