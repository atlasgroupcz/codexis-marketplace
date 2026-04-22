import { describe, expect, it } from 'vitest'
import {
  changeTypeSchema,
  detailResponseSchema,
  overviewResponseSchema,
  trackingTypeSchema,
} from '@/lib/schemas'
import { detailFixture, detailWithPartsFixture, overviewFixture } from '@/test/fixtures'

describe('overviewResponseSchema', () => {
  it('parses valid overview response', () => {
    const result = overviewResponseSchema.parse(overviewFixture)
    expect(result.mode).toBe('overview')
    expect(result.tracked_documents).toHaveLength(3)
    expect(result.tracked_documents[0].name).toBe('Zákoník práce (262/2006 Sb.)')
  })

  it('rejects response with wrong mode', () => {
    expect(() =>
      overviewResponseSchema.parse({ ...overviewFixture, mode: 'detail' }),
    ).toThrow()
  })

  it('rejects document with invalid tracking_type', () => {
    const invalid = {
      ...overviewFixture,
      tracked_documents: [
        { ...overviewFixture.tracked_documents[0], tracking_type: 'invalid' },
      ],
    }
    expect(() => overviewResponseSchema.parse(invalid)).toThrow()
  })

  it('rejects document missing required field', () => {
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    const { name: _name, ...missingName } = overviewFixture.tracked_documents[0]
    const invalid = {
      ...overviewFixture,
      tracked_documents: [missingName],
    }
    expect(() => overviewResponseSchema.parse(invalid)).toThrow()
  })
})

describe('detailResponseSchema', () => {
  it('parses valid detail response without parts', () => {
    const result = detailResponseSchema.parse(detailFixture)
    expect(result.mode).toBe('detail')
    expect(result.document.name).toBe('Zákoník práce (262/2006 Sb.)')
    expect(result.document.changes).toHaveLength(1)
    expect(result.document.parts).toHaveLength(0)
  })

  it('parses valid detail response with parts', () => {
    const result = detailResponseSchema.parse(detailWithPartsFixture)
    expect(result.document.parts).toHaveLength(2)
    expect(result.document.parts[0].label).toBe('\u00a7 89')
  })

  it('parses nullable confirmed_on', () => {
    const result = detailResponseSchema.parse(detailFixture)
    expect(result.document.changes[0].confirmed_on).toBeNull()
  })

  it('parses non-null confirmed_on', () => {
    const result = detailResponseSchema.parse(detailWithPartsFixture)
    expect(result.document.changes[0].confirmed_on).toBe('2026-02-24T07:00:00Z')
  })
})

describe('enum schemas', () => {
  it('trackingTypeSchema accepts valid values', () => {
    expect(trackingTypeSchema.parse('all')).toBe('all')
    expect(trackingTypeSchema.parse('document_changes')).toBe('document_changes')
    expect(trackingTypeSchema.parse('related_documents_changes')).toBe('related_documents_changes')
  })

  it('changeTypeSchema accepts valid values', () => {
    expect(changeTypeSchema.parse('document_change')).toBe('document_change')
    expect(changeTypeSchema.parse('related_change')).toBe('related_change')
  })

  it('rejects invalid enum values', () => {
    expect(() => trackingTypeSchema.parse('invalid')).toThrow()
    expect(() => changeTypeSchema.parse('invalid')).toThrow()
  })
})
