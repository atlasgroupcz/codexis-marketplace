import type { DetailResponse, OverviewResponse } from '@/lib/schemas'

export const overviewFixture: OverviewResponse = {
  mode: 'overview',
  generated_at: '2026-02-25T09:00:00Z',
  tracked_documents: [
    {
      uuid: '2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d',
      codexisId: 'cdx://doc/A10001',
      name: 'Zakonik prace',
      added_on: '2026-02-18T09:15:00Z',
      tracking_type: 'all',
      unconfirmed_changes: 1,
      total_changes: 1,
    },
    {
      uuid: '6a6de088-1909-4d80-a764-d2fcb8ec4eb2',
      codexisId: 'cdx://doc/B10002',
      name: 'Obcansky zakonik',
      added_on: '2026-02-22T13:20:00Z',
      tracking_type: 'document_changes',
      unconfirmed_changes: 0,
      total_changes: 1,
    },
    {
      uuid: 'e748e106-0fd0-4662-9576-6dd5a1949570',
      codexisId: 'cdx://doc/C10003',
      name: 'Spravni rad',
      added_on: '2026-02-25T07:40:00Z',
      tracking_type: 'related_documents_changes',
      unconfirmed_changes: 1,
      total_changes: 1,
    },
  ],
}

export const detailFixture: DetailResponse = {
  mode: 'detail',
  generated_at: '2026-02-25T09:00:00Z',
  document: {
    uuid: '2f4b1f72-3fa2-4b65-9e7a-4bb4ddda1f1d',
    codexisId: 'cdx://doc/A10001',
    name: 'Zakonik prace',
    added_on: '2026-02-18T09:15:00Z',
    tracking_type: 'all',
    parts: [],
    changes: [
      {
        source_documents: [
          { codexisId: 'cdx://doc/A10001', name: 'Zakonik prace' },
        ],
        detected_on: '2026-02-19T08:30:00Z',
        effective_on: '2026-03-01',
        change_type: 'document_change',
        description_md: 'Text zakona byl novelizovan v casti pracovni doby.',
        confirmed_on: null,
      },
    ],
    total_changes: 1,
    unconfirmed_changes: 1,
  },
}

export const detailWithPartsFixture: DetailResponse = {
  mode: 'detail',
  generated_at: '2026-02-25T09:00:00Z',
  document: {
    uuid: '6a6de088-1909-4d80-a764-d2fcb8ec4eb2',
    codexisId: 'cdx://doc/B10002',
    name: 'Obcansky zakonik',
    added_on: '2026-02-22T13:20:00Z',
    tracking_type: 'document_changes',
    parts: [
      { partId: 'paragraf123', label: '\u00a7 123' },
      { partId: 'paragraf2991', label: '\u00a7 2991' },
    ],
    changes: [
      {
        source_documents: [
          { codexisId: 'cdx://doc/B10002', name: 'Obcansky zakonik' },
        ],
        detected_on: '2026-02-23T11:00:00Z',
        effective_on: '2026-04-01',
        change_type: 'document_change',
        description_md: 'Uprava ustanoveni \u00a7 2991.',
        confirmed_on: '2026-02-24T07:00:00Z',
      },
    ],
    total_changes: 1,
    unconfirmed_changes: 0,
  },
}
