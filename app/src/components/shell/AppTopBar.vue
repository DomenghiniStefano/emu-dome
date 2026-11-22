<script setup lang="ts">
// The shell's top bar (design L01): thin glass with the small mark and the wordmark (#101).
import markSmall from '@/assets/brand/emudome-mark-small.svg'
</script>

<template>
  <header
    class="flex shrink-0 basis-titlebar items-center gap-8 border-b border-line bg-glass-thin px-16 backdrop-glass"
  >
    <!-- Decoration: the wordmark beside it says the name. -->
    <img :src="markSmall" alt="" aria-hidden="true" class="block h-mark-small w-auto" />
    <!-- The wordmark's 14px and -0.025em are off the type scale (Cowork, spec Decision 5). -->
    <div class="text-wordmark">EmuDome</div>
  </header>
</template>
