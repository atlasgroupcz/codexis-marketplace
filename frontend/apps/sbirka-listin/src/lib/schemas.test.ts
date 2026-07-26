import { describe, expect, it } from 'vitest'
import {
  changeSchema,
  detailResponseSchema,
  notifyResponseSchema,
  overviewResponseSchema,
} from './schemas'

describe('overviewResponseSchema', () => {
  it('parses the overview shape and defaults nepotvrzenych_zmen to 0', () => {
    const parsed = overviewResponseSchema.parse({
      mode: 'overview',
      generated_at: '2026-07-21T10:00:00Z',
      subjekty: [{ ico: '27082440', nazev: 'Alza.cz a.s.', listin: 93 }],
    })
    expect(parsed.subjekty[0]?.ico).toBe('27082440')
    expect(parsed.subjekty[0]?.nepotvrzenych_zmen).toBe(0)
  })
})

describe('detailResponseSchema', () => {
  it('carries an optional download path per listina', () => {
    const parsed = detailResponseSchema.parse({
      mode: 'detail',
      generated_at: 'x',
      subjekt: {
        ico: '27082440',
        nazev: 'Alza',
        added_on: 'x',
        listiny: [
          { cislo: 'B 8573/SL95/MSPH', typ: 'stanovy', path: '/p/x.pdf' },
          { cislo: 'B 8573/SL94/MSPH', typ: 'ostatní' },
        ],
        changes: [],
      },
    })
    expect(parsed.subjekt.listiny[0]?.path).toBe('/p/x.pdf')
    expect(parsed.subjekt.listiny[1]?.path).toBeUndefined()
  })
})

describe('changeSchema', () => {
  it('reuses the listina shape for nove_listiny and defaults the arrays', () => {
    const parsed = changeSchema.parse({
      detected_on: 'x',
      nove_listiny: [{ cislo: 'B/SL1/M', typ: 'stanovy' }],
    })
    expect(parsed.nove_listiny[0]?.cislo).toBe('B/SL1/M')
    expect(parsed.zmizele_listiny).toEqual([])
  })
})

describe('notifyResponseSchema', () => {
  it('parses the notify settings block', () => {
    const parsed = notifyResponseSchema.parse({
      mode: 'notify',
      settings: { email: false, inApp: true, recipients: ['a@b.cz'] },
      userEmail: 'me@x.cz',
    })
    expect(parsed.settings.inApp).toBe(true)
    expect(parsed.settings.recipients).toEqual(['a@b.cz'])
  })
})
