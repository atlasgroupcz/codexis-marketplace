import { describe, expect, it } from 'vitest'
import { isirProceedingUrl } from '@/lib/isir'
import type { Rizeni } from '@/lib/schemas'
import { rizeniSchema } from '@/lib/schemas'

const FALLBACK = 'https://eisir.justice.cz/insolvencni-rejstrik/dle-ico/07862784'

function rizeni(urlDetail: string): Rizeni {
  return rizeniSchema.parse({ spisova_znacka: 'MSPH 94 INS 4498/2026', url_detail: urlDetail })
}

describe('isirProceedingUrl', () => {
  it('uses the ISIR-provided detail url verbatim - hex-GUID ids (the real ones)', () => {
    const url =
      'https://isir.justice.cz/isir/ueu/evidence_upadcu_detail.do?id=D18B3F0893D33690E05333F21FACE915'
    expect(isirProceedingUrl(rizeni(url), FALLBACK)).toBe(url)
  })

  it('uses the ISIR-provided detail url verbatim - numeric ids too', () => {
    const url = 'https://isir.justice.cz/isir/ueu/evidence_upadcu_detail.do?id=10009596'
    expect(isirProceedingUrl(rizeni(url), FALLBACK)).toBe(url)
  })

  it('falls back when url_detail is empty', () => {
    expect(isirProceedingUrl(rizeni(''), FALLBACK)).toBe(FALLBACK)
  })
})
