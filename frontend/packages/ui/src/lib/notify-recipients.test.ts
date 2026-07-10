import { describe, expect, it } from 'vitest'
import { isValidEmail } from './notify-recipients'

describe('isValidEmail', () => {
  it('accepts a valid address (trimmed)', () => {
    expect(isValidEmail(' a.b@c.cz ')).toBe(true)
  })
  it('rejects an invalid address', () => {
    expect(isValidEmail('nope')).toBe(false)
  })
})
