import type { InjectionKey, Ref } from 'vue'
import type { SegmentedVariant } from './variants'

/** The variant Segmented gives its items. */
export const SegmentedVariantKey: InjectionKey<Ref<SegmentedVariant>> = Symbol('SegmentedVariant')
