import { describe, expect, it } from 'vitest'
import { byNewest, slNumber } from './listiny'

describe('slNumber', () => {
  it('extracts the SL sequence number, tolerating leading zeros', () => {
    expect(slNumber('B 8573/SL95/MSPH')).toBe(95)
    expect(slNumber('B 8573/SL1/MSPH')).toBe(1)
    expect(slNumber('B 8573/SL007/MSPH')).toBe(7)
  })
  it('is 0 when there is no SL segment', () => {
    expect(slNumber('C 12345/MSPH')).toBe(0)
  })
})

describe('byNewest', () => {
  const l = (cislo: string) => ({ cislo, typ: 'listina' })

  it('orders by SL number descending — SL10 before SL2, not lexicographically', () => {
    const out = byNewest([l('B/SL2/M'), l('B/SL10/M'), l('B/SL1/M')])
    expect(out.map((x) => x.cislo)).toEqual(['B/SL10/M', 'B/SL2/M', 'B/SL1/M'])
  })

  it('does not mutate the input array', () => {
    const input = [l('B/SL1/M'), l('B/SL2/M')]
    byNewest(input)
    expect(input.map((x) => x.cislo)).toEqual(['B/SL1/M', 'B/SL2/M'])
  })
})
