import { describe, expect, it } from 'vitest'
import { DAEMON_FILE_URL, REGISTER_SUBJECT_URL } from './constants'

describe('REGISTER_SUBJECT_URL', () => {
  it('builds the or.justice.cz subject page from the IČO', () => {
    expect(REGISTER_SUBJECT_URL('27082440')).toBe(
      'https://or.justice.cz/ias/ui/rejstrik-$firma?ico=27082440',
    )
  })
})

describe('DAEMON_FILE_URL', () => {
  it('opens the document on the daemon home page with the file previewed and attached', () => {
    const path = '/home/u/.cdx/apps/sbirka-listin/dokumenty/27082440/B-8573-SL95-MSPH.pdf'
    const url = DAEMON_FILE_URL(path)
    expect(url.startsWith('/?explorer=')).toBe(true)
    const state = JSON.parse(decodeURIComponent(url.slice('/?explorer='.length)))
    expect(state.previewPath).toBe(path)
    expect(state.path).toBe('/home/u/.cdx/apps/sbirka-listin/dokumenty/27082440')
  })
})
