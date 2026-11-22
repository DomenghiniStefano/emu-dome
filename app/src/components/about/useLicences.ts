// The licences page's state (`about.lic`): the rows from Rust, and the texts of the one opened.
// A failure reaches the user (useErrorSink); the texts of a row closed or changed meanwhile are
// dropped (rule 7).
import { onMounted, ref } from 'vue'
import { emu, type Licence } from '@/emu'
import { useErrorSink } from '@/composables/useToasts'
import { latest } from '@/lib/latest'

/** One licence file's text. */
export interface LicenceText {
  file: string
  text: string
}

export function useLicences() {
  const failWith = useErrorSink()
  const rows = ref<Licence[]>([])
  const opened = ref<Licence | null>(null)
  const texts = ref<LicenceText[]>([])
  const opens = latest()

  onMounted(async () => {
    try {
      rows.value = await emu.about.licences()
    } catch (e) {
      failWith(e)
    }
  })

  /** Shows `row`'s licence files, in order. */
  async function open(row: Licence) {
    const fresh = opens.next()
    opened.value = row
    texts.value = []
    try {
      const read = await Promise.all(
        row.files.map(async file => ({ file, text: await emu.about.licenceText(row.id, file) })),
      )
      if (fresh()) texts.value = read
    } catch (e) {
      if (fresh()) failWith(e)
    }
  }

  function close() {
    opens.next()
    opened.value = null
    texts.value = []
  }

  return { rows, opened, texts, open, close }
}
