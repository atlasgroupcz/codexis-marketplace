import { describe, expect, it } from 'vitest'
import type { Rizeni } from '@/lib/schemas'
import { eisirProceedingUrl } from '@/lib/isir'
import { rizeniSchema } from '@/lib/schemas'

const FALLBACK = 'https://eisir.justice.cz/insolvencni-rejstrik/dle-ico/07862784'

function rizeni(urlDetail: string): Rizeni {
  return rizeniSchema.parse({ spisova_znacka: 'MSPH 94 INS 4498/2026', url_detail: urlDetail })
}

describe('eisirProceedingUrl', () => {
  it('rewrites a legacy url_detail into the eISIR record deep-link', () => {
    expect(
      eisirProceedingUrl(
        rizeni('https://isir.justice.cz/isir/ueu/evidence_upadcu_detail.do?id=10009596'),
        FALLBACK,
      ),
    ).toBe('https://eisir.justice.cz/insolvencni-rejstrik/r/10009596')
  })

  it('falls back when url_detail is empty', () => {
    expect(eisirProceedingUrl(rizeni(''), FALLBACK)).toBe(FALLBACK)
  })

  it('falls back when the id is missing from the query', () => {
    expect(
      eisirProceedingUrl(
        rizeni('https://isir.justice.cz/isir/ueu/evidence_upadcu_detail.do'),
        FALLBACK,
      ),
    ).toBe(FALLBACK)
  })

  it('falls back when the id is non-numeric', () => {
    expect(
      eisirProceedingUrl(
        rizeni('https://isir.justice.cz/isir/ueu/evidence_upadcu_detail.do?id=ABC'),
        FALLBACK,
      ),
    ).toBe(FALLBACK)
  })

  it('falls back when url_detail is not a valid URL', () => {
    expect(eisirProceedingUrl(rizeni('not a url'), FALLBACK)).toBe(FALLBACK)
  })
})
