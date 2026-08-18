import type { ProceedingDetail, ProceedingSummary } from '@/lib/schemas'

// `ProceedingSummary` has thirteen required fields, so a test that built one
// inline had to restate all of them to vary one. Four files did, and the copies
// drifted - one of them cast through `unknown` and so had stopped being checked
// against the schema at all.

/** A row as the overview list carries it. Override only what the test is about. */
export function proceedingFixture(overrides: Partial<ProceedingSummary> = {}): ProceedingSummary {
  return {
    cislo_rizeni: 'V-1/2026-701',
    typ_rizeni: 'Vklad',
    label: '',
    stav: 'Zapsáno',
    stav_uhrady: null,
    datum_prijeti: '2026-04-01',
    added_on: '2026-04-01',
    last_check_at: '2026-04-01',
    last_op_date: null,
    operace_count: 0,
    changes_count: 0,
    unconfirmed_count: 0,
    ...overrides,
  }
}

/** The same row plus the two arrays only the detail request carries. */
export function detailFixture(overrides: Partial<ProceedingDetail> = {}): ProceedingDetail {
  return {
    ...proceedingFixture(overrides),
    provedene_operace: [],
    changes: [],
    ...overrides,
  }
}
