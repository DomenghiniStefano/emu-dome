import type { InjectionKey, Ref } from 'vue'
import type { TabsSize } from './variants'

/** The size Tabs gives its triggers. */
export const TabsSizeKey: InjectionKey<Ref<TabsSize>> = Symbol('TabsSize')
