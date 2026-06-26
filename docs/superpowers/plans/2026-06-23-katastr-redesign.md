# Katastr UI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Redesign the katastr frontend into the CODEXIS family look (shell + green accent), apply progressive disclosure to the list, and add a horizontal "Průběh řízení" timeline as the signature element in the detail.

**Architecture:** Frontend-only change to `frontend/apps/katastr:katastr`. Backend, API shape, and `schemas.ts` are untouched. The timeline is derived from existing data (`stav` + `provedene_operace`) by a pure function `deriveTimeline`. The list and detail screens are re-skinned to match `sledovana-judikatura`; navigation (list ↔ detail via nuqs URL state) is unchanged.

**Tech Stack:** React 19, Vite 7, TypeScript, Tailwind 4 + `@workspace/ui`, lucide-react, react-i18next, nuqs, zod, vitest.

## Global Constraints

- Work on branch `lm-121-katastr-redesign`. Do NOT touch `main`.
- App directory contains a colon — always quote it: `'frontend/apps/katastr:katastr'`.
- Do NOT change `src/lib/schemas.ts`, the API layer (`src/lib/api.ts`), or any backend.
- Primary accent = CODEXIS green `#5ea500` (already in `src/styles.css`). Plomba accent = `#c2410c`.
- Preserve all existing `data-testid` attributes and the behaviors they cover (add, check-all, edit label, remove, confirm change).
- All user-facing strings go through i18n (`react-i18next`); add keys to all three locales `cs`, `en`, `sk`. Technical identifiers stay untranslated.
- **Lint gate (important):** the app carries pre-existing whole-app lint debt that lives only in files Tasks 4 & 5 fully rewrite. Per-task gate is **per-file**: each task's created/modified files must be lint-clean via `bunx eslint --max-warnings=0 <files>` from the app dir. Whole-app `bun run lint` (which has `--max-warnings=0`) only needs to be clean at **Task 6**, after Tasks 4 & 5 clear their files' warnings.
- **ESLint rules all new code must respect** (else per-file lint fails): `@typescript-eslint/array-type` is `array-simple` — use `Array<T>` for non-primitive element types (`Array<Operace>`, `Array<TimelineNode>`, not `Operace[]`; primitives like `string[]` are fine); `import/order` — `import type` from `@/…` ordered before value imports, relative imports grouped correctly; `react-hooks/static-components` — NEVER define a component inside another component's render body; define it at module scope and pass props; `@typescript-eslint/no-unnecessary-condition` — don't add `??` / optional chains on values the types already say are non-nullish.
- Tests run from the app dir: `bun run test` (vitest, node env by default).

---

### Task 1: Plomba tokens + unified badge colors

**Files:**
- Modify: `frontend/apps/katastr:katastr/src/styles.css` (add plomba tokens to `@theme` and `:root`)
- Modify: `frontend/apps/katastr:katastr/src/components/stav-badge.tsx`
- Modify: `frontend/apps/katastr:katastr/src/components/uhrada-badge.tsx`

**Interfaces:**
- Consumes: nothing (first task).
- Produces: CSS classes `bg-brand-50 text-brand-700`, `bg-plomba-soft text-plomba` usable by later tasks; `StavBadge` and `UhradaBadge` keep the same props (`stav: string`; `stavUhrady`, `label`).

This task is visual only — no unit test. The gate is typecheck + lint + a manual class audit. Fold the verification into the final steps.

- [ ] **Step 1: Add plomba tokens to `styles.css`**

In the `@theme` block, after the brand ramp, add:

```css
  /* plomba (lien marker) accent */
  --color-plomba: #c2410c;
  --color-plomba-soft: #fdf0e9;
```

In the `:root` block, after `--ring: #5ea500;`, add (so non-`@theme` consumers can read them too):

```css
  --plomba: #c2410c;
  --plomba-soft: #fdf0e9;
```

- [ ] **Step 2: Unify `stav-badge.tsx` to brand/plomba tokens**

Replace the whole file with a single color map driven by keyword matching against brand/plomba/amber/muted (drop the hardcoded `green-100`/`yellow-100`/`orange-100` Tailwind classes):

```tsx
import { Badge } from '@workspace/ui/components/badge'

interface StavBadgeProps {
  stav: string
}

// Keyword → tailwind classes. Order matters: most specific first.
function getStavColor(stav: string): string | null {
  const s = stav.toLowerCase()
  if (s.includes('ukončeno')) return 'bg-muted text-muted-foreground'
  if (s.includes('vyrozumění') || s.includes('provedení vkladu'))
    return 'bg-brand-50 text-brand-700'
  if (s.includes('plomb')) return 'bg-plomba-soft text-plomba'
  if (s.includes('povolení vkladu')) return 'bg-brand-50 text-brand-700'
  if (s.includes('probíhá') || s.includes('podáno') || s.includes('přijet'))
    return 'bg-amber-100 text-amber-800'
  return null
}

export function StavBadge({ stav }: StavBadgeProps) {
  const colorClass = getStavColor(stav)
  if (colorClass) {
    return (
      <span
        className={`inline-flex items-center rounded-md px-2 py-1 text-xs font-medium ${colorClass}`}
      >
        {stav}
      </span>
    )
  }
  return (
    <Badge variant="outline" className="text-xs font-medium">
      {stav}
    </Badge>
  )
}
```

- [ ] **Step 3: Unify `uhrada-badge.tsx` to brand/plomba tokens**

Replace the color map:

```tsx
interface UhradaBadgeProps {
  stavUhrady: string | null | undefined
  label: string | null | undefined
}

export function UhradaBadge({ stavUhrady, label }: UhradaBadgeProps) {
  if (!stavUhrady || !label) {
    return <span className="text-muted-foreground text-sm">---</span>
  }
  const colorMap: Record<string, string> = {
    U: 'bg-brand-50 text-brand-700',
    N: 'bg-plomba-soft text-plomba',
    O: 'bg-amber-100 text-amber-800',
  }
  const color = colorMap[stavUhrady] ?? 'bg-muted text-muted-foreground'
  return (
    <span className={`inline-flex items-center rounded-md px-2 py-1 text-xs font-medium ${color}`}>
      {label}
    </span>
  )
}
```

- [ ] **Step 4: Typecheck + lint**

Run: `cd 'frontend/apps/katastr:katastr' && bun run typecheck && bun run lint`
Expected: both pass, no errors.

- [ ] **Step 5: Commit**

```bash
git add "frontend/apps/katastr:katastr/src/styles.css" "frontend/apps/katastr:katastr/src/components/stav-badge.tsx" "frontend/apps/katastr:katastr/src/components/uhrada-badge.tsx"
git commit -m "style(katastr): add plomba tokens, unify badges on brand palette"
```

---

### Task 2: `deriveTimeline` pure function + tests

**Files:**
- Create: `frontend/apps/katastr:katastr/src/lib/timeline.ts`
- Test: `frontend/apps/katastr:katastr/src/lib/timeline.test.ts`

**Interfaces:**
- Consumes: `ProceedingSummary` / `ProceedingDetail` type from `@/lib/schemas` (read-only).
- Produces:
  - `type TimelineState = 'done' | 'current' | 'future'`
  - `interface TimelineNode { key: string; nameKey: string | null; rawName: string | null; date: string | null; state: TimelineState; isPlomba: boolean }`
  - `function deriveTimeline(proc: { cislo_rizeni: string; typ_rizeni: string; stav: string; datum_prijeti: string; provedene_operace: { nazev: string; datumProvedeni: string }[] }): TimelineNode[]`
  - `function latestMilestone(proc: { stav: string; last_op_date?: string | null; provedene_operace: { nazev: string; datumProvedeni: string }[] }): { name: string | null; nameKey: string | null; date: string | null }`
  - Exported constant `VKLAD_PHASE_KEYS = ['prijeti','plomba','povoleniVkladu','provedeniVkladu','vyrozumeni']` (used by i18n + components).

- [ ] **Step 1: Write the failing test**

Create `src/lib/timeline.test.ts`:

```ts
import { describe, it, expect } from 'vitest'
import { deriveTimeline, latestMilestone } from './timeline'

const op = (nazev: string, datumProvedeni: string) => ({ nazev, datumProvedeni })

describe('deriveTimeline — vklad (canonical)', () => {
  const base = {
    cislo_rizeni: 'V-1234/2026-701',
    typ_rizeni: 'Vklad',
    datum_prijeti: '2026-06-01',
  }

  it('produces 5 canonical phases with plomba flagged', () => {
    const nodes = deriveTimeline({
      ...base,
      stav: 'Povolení vkladu',
      provedene_operace: [
        op('Přijetí návrhu', '2026-06-01'),
        op('Vyznačení plomby', '2026-06-01'),
        op('Povolení vkladu', '2026-06-12'),
      ],
    })
    expect(nodes.map((n) => n.key)).toEqual([
      'prijeti',
      'plomba',
      'povoleniVkladu',
      'provedeniVkladu',
      'vyrozumeni',
    ])
    expect(nodes[1].isPlomba).toBe(true)
  })

  it('marks reached phases done, the highest reached current, the rest future', () => {
    const nodes = deriveTimeline({
      ...base,
      stav: 'Povolení vkladu',
      provedene_operace: [
        op('Přijetí návrhu', '2026-06-01'),
        op('Vyznačení plomby', '2026-06-01'),
        op('Povolení vkladu', '2026-06-12'),
      ],
    })
    expect(nodes.map((n) => n.state)).toEqual([
      'done',
      'done',
      'current',
      'future',
      'future',
    ])
    expect(nodes[2].date).toBe('2026-06-12')
    expect(nodes[3].date).toBeNull()
  })

  it('uses stav to place current when no operation matches yet, dating prijeti from datum_prijeti', () => {
    const nodes = deriveTimeline({
      ...base,
      stav: 'Plomba vyznačena',
      provedene_operace: [],
    })
    expect(nodes[0].date).toBe('2026-06-01')
    expect(nodes[1].state).toBe('current')
    expect(nodes[2].state).toBe('future')
  })
})

describe('deriveTimeline — fallback (non-vklad)', () => {
  it('builds one node per operation, sorted by date, last is current', () => {
    const nodes = deriveTimeline({
      cislo_rizeni: 'Z-77/2026-701',
      typ_rizeni: 'Záznam',
      stav: 'Provedení záznamu',
      datum_prijeti: '2026-06-01',
      provedene_operace: [
        op('Přijetí návrhu', '2026-06-01'),
        op('Provedení záznamu', '2026-06-05'),
      ],
    })
    expect(nodes).toHaveLength(2)
    expect(nodes[0].state).toBe('done')
    expect(nodes[1].state).toBe('current')
    expect(nodes[1].rawName).toBe('Provedení záznamu')
    expect(nodes[1].nameKey).toBeNull()
  })

  it('flags plomba in fallback mode', () => {
    const nodes = deriveTimeline({
      cislo_rizeni: 'Z-9/2026-701',
      typ_rizeni: 'Záznam',
      stav: 'Plomba vyznačena',
      datum_prijeti: '2026-06-01',
      provedene_operace: [op('Vyznačení plomby', '2026-06-02')],
    })
    expect(nodes[0].isPlomba).toBe(true)
  })

  it('renders a single current node from stav when there are no operations', () => {
    const nodes = deriveTimeline({
      cislo_rizeni: 'Z-1/2026-701',
      typ_rizeni: 'Záznam',
      stav: 'Podáno',
      datum_prijeti: '2026-06-01',
      provedene_operace: [],
    })
    expect(nodes).toHaveLength(1)
    expect(nodes[0].state).toBe('current')
    expect(nodes[0].rawName).toBe('Podáno')
  })
})

describe('latestMilestone', () => {
  it('returns the most recent operation', () => {
    const m = latestMilestone({
      stav: 'Povolení vkladu',
      provedene_operace: [
        op('Přijetí návrhu', '2026-06-01'),
        op('Povolení vkladu', '2026-06-12'),
      ],
    })
    expect(m.name).toBe('Povolení vkladu')
    expect(m.date).toBe('2026-06-12')
  })

  it('falls back to stav when there are no operations', () => {
    const m = latestMilestone({ stav: 'Podáno', provedene_operace: [] })
    expect(m.name).toBe('Podáno')
    expect(m.date).toBeNull()
  })
})
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/lib/timeline.test.ts`
Expected: FAIL — cannot resolve `./timeline` / `deriveTimeline is not a function`.

- [ ] **Step 3: Write the implementation**

Create `src/lib/timeline.ts`:

```ts
export type TimelineState = 'done' | 'current' | 'future'

export interface TimelineNode {
  key: string
  nameKey: string | null // i18n key suffix for canonical phases, null in fallback
  rawName: string | null // raw operation/stav text in fallback, null for canonical
  date: string | null // ISO date or null (future / unknown)
  state: TimelineState
  isPlomba: boolean
}

interface Operace {
  nazev: string
  datumProvedeni: string
}

interface TimelineInput {
  cislo_rizeni: string
  typ_rizeni: string
  stav: string
  datum_prijeti: string
  provedene_operace: Operace[]
}

export const VKLAD_PHASE_KEYS = [
  'prijeti',
  'plomba',
  'povoleniVkladu',
  'provedeniVkladu',
  'vyrozumeni',
] as const

// Distinct stems so "povolení vkladu" and "provedení vkladu" never collide.
const VKLAD_PHASES: { key: string; stems: string[]; isPlomba: boolean }[] = [
  { key: 'prijeti', stems: ['přijet', 'podáno'], isPlomba: false },
  { key: 'plomba', stems: ['plomb'], isPlomba: true },
  { key: 'povoleniVkladu', stems: ['povolen'], isPlomba: false },
  { key: 'provedeniVkladu', stems: ['proveden'], isPlomba: false },
  { key: 'vyrozumeni', stems: ['vyrozum'], isPlomba: false },
]

function isVklad(proc: TimelineInput): boolean {
  return (
    proc.cislo_rizeni.toUpperCase().startsWith('V-') ||
    proc.typ_rizeni.toLowerCase().includes('vklad')
  )
}

function matchPhaseIndex(text: string): number {
  const t = text.toLowerCase()
  for (let i = 0; i < VKLAD_PHASES.length; i++) {
    if (VKLAD_PHASES[i].stems.some((stem) => t.includes(stem))) return i
  }
  return -1
}

function canonicalTimeline(proc: TimelineInput): TimelineNode[] {
  // First operation date that maps to each phase.
  const phaseDate: (string | null)[] = VKLAD_PHASES.map(() => null)
  for (const op of proc.provedene_operace) {
    const idx = matchPhaseIndex(op.nazev)
    if (idx >= 0 && phaseDate[idx] === null) phaseDate[idx] = op.datumProvedeni
  }

  // Highest reached phase: from matched operations OR from current stav.
  let reached = phaseDate.reduce((max, d, i) => (d !== null ? i : max), -1)
  const stavIdx = matchPhaseIndex(proc.stav)
  if (stavIdx > reached) reached = stavIdx
  if (reached < 0) reached = 0 // a tracked proceeding has at least been accepted

  return VKLAD_PHASES.map((phase, i) => {
    const state: TimelineState =
      i < reached ? 'done' : i === reached ? 'current' : 'future'
    let date = phaseDate[i]
    if (date === null && i === 0 && state !== 'future') date = proc.datum_prijeti
    if (state === 'future') date = null
    return {
      key: phase.key,
      nameKey: phase.key,
      rawName: null,
      date,
      state,
      isPlomba: phase.isPlomba,
    }
  })
}

function fallbackTimeline(proc: TimelineInput): TimelineNode[] {
  const ops = [...proc.provedene_operace].sort((a, b) =>
    a.datumProvedeni < b.datumProvedeni ? -1 : a.datumProvedeni > b.datumProvedeni ? 1 : 0,
  )
  if (ops.length === 0) {
    return [
      {
        key: 'stav',
        nameKey: null,
        rawName: proc.stav,
        date: null,
        state: 'current',
        isPlomba: proc.stav.toLowerCase().includes('plomb'),
      },
    ]
  }
  return ops.map((op, i) => ({
    key: `op:${i}`,
    nameKey: null,
    rawName: op.nazev,
    date: op.datumProvedeni,
    state: i === ops.length - 1 ? 'current' : 'done',
    isPlomba: op.nazev.toLowerCase().includes('plomb'),
  }))
}

export function deriveTimeline(proc: TimelineInput): TimelineNode[] {
  return isVklad(proc) ? canonicalTimeline(proc) : fallbackTimeline(proc)
}

export function latestMilestone(proc: {
  stav: string
  last_op_date?: string | null
  provedene_operace: Operace[]
}): { name: string | null; nameKey: string | null; date: string | null } {
  const ops = proc.provedene_operace
  if (ops.length > 0) {
    const last = ops.reduce((a, b) => (a.datumProvedeni >= b.datumProvedeni ? a : b))
    return { name: last.nazev, nameKey: null, date: last.datumProvedeni }
  }
  return { name: proc.stav, nameKey: null, date: proc.last_op_date ?? null }
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/lib/timeline.test.ts`
Expected: PASS — all cases green.

- [ ] **Step 5: Typecheck + lint + commit**

```bash
cd 'frontend/apps/katastr:katastr' && bun run typecheck && bunx eslint --max-warnings=0 src/lib/timeline.ts src/lib/timeline.test.ts && cd -
git add "frontend/apps/katastr:katastr/src/lib/timeline.ts" "frontend/apps/katastr:katastr/src/lib/timeline.test.ts"
git commit -m "feat(katastr): derive proceeding timeline from stav + operations"
```

Note: the plan code below uses `T[]` in a few type positions; convert any non-primitive element type to `Array<T>` (e.g. `Array<Operace>`, `Array<TimelineNode>`, `Array<string | null>`) to satisfy `array-type: array-simple`.

---

### Task 3: i18n keys (cs / en / sk)

**Files:**
- Modify: `frontend/apps/katastr:katastr/public/locales/cs/translation.json`
- Modify: `frontend/apps/katastr:katastr/public/locales/en/translation.json`
- Modify: `frontend/apps/katastr:katastr/public/locales/sk/translation.json`

**Interfaces:**
- Consumes: `VKLAD_PHASE_KEYS` naming from Task 2 (the timeline phase keys must match).
- Produces: i18n keys used by Tasks 4 & 5:
  - `proceedings.subtitle`
  - `proceedings.timeline.title`, `.prijeti`, `.plomba`, `.povoleniVkladu`, `.provedeniVkladu`, `.vyrozumeni`, `.now`, `.expected`
  - `proceedings.allOperations`
  - `proceedings.lastMilestone`

No unit test (data file). Gate: valid JSON + build picks them up.

- [ ] **Step 1: Add keys to `cs/translation.json`**

Inside the `proceedings` object, add:

```json
    "subtitle": "Stav vašich katastrálních řízení",
    "lastMilestone": "Poslední milník",
    "allOperations": "Všechny provedené operace",
    "timeline": {
      "title": "Průběh řízení",
      "prijeti": "Přijetí návrhu",
      "plomba": "Plomba vyznačena",
      "povoleniVkladu": "Povolení vkladu",
      "provedeniVkladu": "Provedení vkladu",
      "vyrozumeni": "Vyrozumění o vkladu",
      "now": "teď",
      "expected": "—"
    },
```

- [ ] **Step 2: Add keys to `en/translation.json`**

```json
    "subtitle": "Status of your cadastral proceedings",
    "lastMilestone": "Latest milestone",
    "allOperations": "All recorded operations",
    "timeline": {
      "title": "Proceeding progress",
      "prijeti": "Petition received",
      "plomba": "Lien marked",
      "povoleniVkladu": "Entry authorised",
      "provedeniVkladu": "Entry executed",
      "vyrozumeni": "Notice of entry",
      "now": "now",
      "expected": "—"
    },
```

- [ ] **Step 3: Add keys to `sk/translation.json`**

```json
    "subtitle": "Stav vašich katastrálnych konaní",
    "lastMilestone": "Posledný míľnik",
    "allOperations": "Všetky vykonané operácie",
    "timeline": {
      "title": "Priebeh konania",
      "prijeti": "Prijatie návrhu",
      "plomba": "Plomba vyznačená",
      "povoleniVkladu": "Povolenie vkladu",
      "provedeniVkladu": "Vykonanie vkladu",
      "vyrozumeni": "Oznámenie o vklade",
      "now": "teraz",
      "expected": "—"
    },
```

- [ ] **Step 4: Validate JSON**

Run: `cd 'frontend/apps/katastr:katastr' && for l in cs en sk; do bunx --bun node -e "JSON.parse(require('fs').readFileSync('public/locales/$l/translation.json','utf8')); console.log('$l ok')"; done`
Expected: `cs ok` / `en ok` / `sk ok` (no parse errors).

- [ ] **Step 5: Commit**

```bash
git add "frontend/apps/katastr:katastr/public/locales"
git commit -m "i18n(katastr): add timeline and list subtitle keys (cs/en/sk)"
```

---

### Task 4: List shell + header + redesigned row (progressive disclosure)

**Files:**
- Modify: `frontend/apps/katastr:katastr/src/components/proceedings-list.tsx`
- Modify: `frontend/apps/katastr:katastr/src/components/proceeding-row.tsx`
- Modify: `frontend/apps/katastr:katastr/src/components/add-proceeding-form.tsx`
- Test: `frontend/apps/katastr:katastr/src/components/proceeding-row.test.tsx`

**Interfaces:**
- Consumes: `latestMilestone` from Task 2; i18n keys from Task 3; `StavBadge`/`UhradaBadge` from Task 1.
- Produces: a list shell whose row is a clickable element with `data-testid="proceeding-row-<cislo_rizeni>"` (unchanged testid) and **no longer renders the full operations list** in the row.

The signature change: the row shows current stav + latest milestone only. The render-smoke test guards exactly this — that the row shows the milestone string and does NOT dump every operation.

- [ ] **Step 1: Write the failing render test**

Create `src/components/proceeding-row.test.tsx`:

```tsx
// @vitest-environment jsdom
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ProceedingRow } from './proceeding-row'
import type { ProceedingSummary } from '@/lib/schemas'

// react-i18next returns the key by default in tests (no provider) — assert on keys/values we control.
const proc: ProceedingSummary = {
  uuid: 'u1',
  cislo_rizeni: 'V-1234/2026-701',
  typ_rizeni: 'Vklad',
  label: 'Byt Brno-střed',
  stav: 'Povolení vkladu',
  stav_uhrady: 'U',
  stav_uhrady_label: 'Uhrazeno',
  datum_prijeti: '2026-06-01',
  added_on: '2026-06-01',
  last_check_at: '2026-06-12',
  last_op_date: '2026-06-12',
  operace_count: 3,
  changes_count: 1,
  unconfirmed_count: 2,
  provedene_operace: [
    { nazev: 'Přijetí návrhu', datumProvedeni: '2026-06-01' },
    { nazev: 'Vyznačení plomby', datumProvedeni: '2026-06-01' },
    { nazev: 'Povolení vkladu', datumProvedeni: '2026-06-12' },
  ],
}

describe('ProceedingRow', () => {
  it('shows číslo, label, stav and only the latest milestone — not every operation', () => {
    render(<ProceedingRow proceeding={proc} onClick={() => {}} />)
    expect(screen.getByText('V-1234/2026-701')).toBeTruthy()
    expect(screen.getByText(/Byt Brno-střed/)).toBeTruthy()
    // latest milestone present
    expect(screen.getAllByText(/Povolení vkladu/).length).toBeGreaterThan(0)
    // progressive disclosure: the earlier operation must NOT appear in the row
    expect(screen.queryByText('Vyznačení plomby')).toBeNull()
    expect(screen.queryByText('Přijetí návrhu')).toBeNull()
  })
})
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/components/proceeding-row.test.tsx`
Expected: FAIL — the current row renders every operation (so `Vyznačení plomby` is found), or `<tr>` rendered outside a table warns/throws.

- [ ] **Step 3: Rewrite `proceeding-row.tsx` as a non-table row (var. A)**

```tsx
import { useTranslation } from 'react-i18next'
import { Badge } from '@workspace/ui/components/badge'
import { StavBadge } from './stav-badge'
import { UhradaBadge } from './uhrada-badge'
import { latestMilestone } from '@/lib/timeline'
import { formatDate } from '@/lib/format'
import type { ProceedingSummary } from '@/lib/schemas'

interface ProceedingRowProps {
  proceeding: ProceedingSummary
  onClick: (uuid: string) => void
}

export function ProceedingRow({ proceeding, onClick }: ProceedingRowProps) {
  const { t } = useTranslation()
  const milestone = latestMilestone(proceeding)
  const milestoneDate = milestone.date ? formatDate(milestone.date) : null

  return (
    <button
      type="button"
      onClick={() => onClick(proceeding.uuid)}
      data-testid={`proceeding-row-${proceeding.cislo_rizeni}`}
      className="hover:bg-secondary/50 flex w-full flex-col gap-1 border-b px-4 py-3 text-left transition-colors last:border-b-0"
    >
      <div className="flex items-center gap-2">
        <span className="font-mono text-sm font-medium">{proceeding.cislo_rizeni}</span>
        {proceeding.label && (
          <span className="text-muted-foreground truncate text-sm">· {proceeding.label}</span>
        )}
        <span className="flex-1" />
        <StavBadge stav={proceeding.stav} />
      </div>
      <div className="text-muted-foreground flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs">
        <UhradaBadge
          stavUhrady={proceeding.stav_uhrady}
          label={proceeding.stav_uhrady_label}
        />
        {milestone.name && (
          <>
            <span className="text-border">·</span>
            <span>
              {t('proceedings.lastMilestone')}: {milestone.name}
              {milestoneDate ? ` ${milestoneDate}` : ''}
            </span>
          </>
        )}
        {proceeding.unconfirmed_count > 0 && (
          <>
            <span className="text-border">·</span>
            <Badge variant="destructive" className="text-[10px]">
              {t('proceedings.unconfirmedChangesBadge', {
                count: proceeding.unconfirmed_count,
              })}
            </Badge>
          </>
        )}
      </div>
    </button>
  )
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/components/proceeding-row.test.tsx`
Expected: PASS.

- [ ] **Step 5: Rewrite `proceedings-list.tsx` shell + header**

Replace the rendered tree so it mirrors `sledovana-judikatura`'s shell (full-height column, `h-14` header with icon-in-rounded-square, serif title + muted subtitle, actions on the right), and replace the `<table>` with a bordered list of `ProceedingRow`. Keep all states (pre-config, loading, error, empty) and the settings dialog. Use `Landmark` from lucide-react for the header icon.

**Important:** define the header as a **module-scope** component `ListHeader` (NOT inside `ProceedingsList` — an inline component trips `react-hooks/static-components`). It takes props and calls `useTranslation` itself:

```tsx
import { useState } from 'react'
import { Landmark, Plus, RefreshCw, Settings as SettingsIcon, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@workspace/ui/components/button'
import { toast } from '@workspace/ui/components/sonner'
import { ListSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { ProceedingRow } from './proceeding-row'
import { AddProceedingForm } from './add-proceeding-form'
import { SettingsDialog } from './settings-dialog'
import { ApiKeyBanner } from './api-key-banner'
import { useOverview } from '@/hooks/use-overview'

interface ProceedingsListProps {
  onSelect: (uuid: string) => void
}

interface ListHeaderProps {
  showActions: boolean
  addOpen: boolean
  checking: boolean
  onToggleAdd: () => void
  onCheck: () => void
  onOpenSettings: () => void
}

function ListHeader({
  showActions,
  addOpen,
  checking,
  onToggleAdd,
  onCheck,
  onOpenSettings,
}: ListHeaderProps) {
  const { t } = useTranslation()
  return (
    <header className="flex h-14 shrink-0 items-center justify-between gap-3 border-b px-4">
      <div className="flex min-w-0 items-center gap-2.5">
        <span className="bg-secondary text-primary ring-brand-100 flex size-9 items-center justify-center rounded-xl ring-1">
          <Landmark className="size-5" strokeWidth={1.75} />
        </span>
        <div className="min-w-0">
          <h1 className="font-display text-foreground truncate text-[15px] leading-tight">
            {t('proceedings.title')}
          </h1>
          <p className="text-muted-foreground hidden truncate text-xs sm:block">
            {t('proceedings.subtitle')}
          </p>
        </div>
      </div>
      <div className="flex items-center gap-2">
        {showActions && (
          <>
            <Button size="sm" onClick={onToggleAdd} data-testid="toggle-add">
              {addOpen ? <X className="size-4" /> : <Plus className="size-4" />}
              <span className="hidden sm:inline">{t('proceedings.add')}</span>
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={onCheck}
              disabled={checking}
              data-testid="check-all-button"
            >
              <RefreshCw className={`size-4 ${checking ? 'animate-spin' : ''}`} />
              <span className="hidden sm:inline">{t('proceedings.checkAll')}</span>
            </Button>
          </>
        )}
        <Button
          variant="ghost"
          size="sm"
          onClick={onOpenSettings}
          title={t('settings.title')}
          data-testid="open-settings"
        >
          <SettingsIcon className="size-4" />
        </Button>
      </div>
    </header>
  )
}

export function ProceedingsList({ onSelect }: ProceedingsListProps) {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [addOpen, setAddOpen] = useState(false)
  const [checking, setChecking] = useState(false)

  const apiKeyConfigured = data?.api_key_configured ?? false

  const handleChecked = (newChanges: number, errors: number) => {
    if (errors > 0) toast.error(t('proceedings.checkResultErrors', { newChanges, errors }))
    else if (newChanges > 0) toast.success(t('proceedings.checkResultChanges', { count: newChanges }))
    else toast.success(t('proceedings.checkResultNone'))
    refetch()
  }

  const header = (showActions: boolean) => (
    <ListHeader
      showActions={showActions}
      addOpen={addOpen}
      checking={checking}
      onToggleAdd={() => setAddOpen((v) => !v)}
      onCheck={() => setChecking(true)}
      onOpenSettings={() => setSettingsOpen(true)}
    />
  )

  const settings = (
    <SettingsDialog
      open={settingsOpen}
      onOpenChange={setSettingsOpen}
      configured={data?.api_key_configured ?? false}
      maskedKey={data?.api_key_masked ?? ''}
      onSaved={refetch}
    />
  )

  if (loading && !data) {
    return (
      <div className="flex h-screen flex-col" data-testid="app-root">
        {header(false)}
        <div className="p-4 sm:p-6">
          <ListSkeleton />
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="flex h-screen flex-col" data-testid="app-root">
        {header(false)}
        <div className="p-6">
          <ErrorMessage message={error.message} onRetry={refetch} />
        </div>
        {settings}
      </div>
    )
  }

  if (!data) return null

  return (
    <div className="flex h-screen flex-col" data-testid="app-root">
      {header(apiKeyConfigured)}
      <main className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto max-w-3xl p-4 sm:p-6">
          {!apiKeyConfigured ? (
            <ApiKeyBanner onOpenSettings={() => setSettingsOpen(true)} />
          ) : (
            <>
              {addOpen && (
                <div className="mb-4">
                  <AddProceedingForm
                    onAdded={() => {
                      setAddOpen(false)
                      refetch()
                    }}
                    onChecked={handleChecked}
                    checking={checking}
                    onCheckingChange={setChecking}
                  />
                </div>
              )}
              {data.proceedings.length === 0 ? (
                <p className="text-muted-foreground text-sm" data-testid="empty-state">
                  {t('proceedings.empty')}
                </p>
              ) : (
                <div className="overflow-hidden rounded-lg border" data-testid="proceedings-table">
                  {data.proceedings.map((proc) => (
                    <ProceedingRow key={proc.uuid} proceeding={proc} onClick={onSelect} />
                  ))}
                </div>
              )}
            </>
          )}
        </div>
      </main>
      {settings}
    </div>
  )
}
```

- [ ] **Step 6: Adapt `add-proceeding-form.tsx` to the lifted check state**

The check-all action now lives in the header. Lift `checking` into props so the header spinner and the form share it; keep the add input + button. Update the props interface and the check handler:

```tsx
interface AddProceedingFormProps {
  onAdded: () => void
  onChecked: (newChanges: number, errors: number) => void
  checking: boolean
  onCheckingChange: (v: boolean) => void
}
```

In the component body, drive the check-all effect from the lifted flag and drop the in-form check button (the header owns it):

```tsx
  // run check-all whenever the header sets checking=true
  useEffect(() => {
    if (!checking) return
    checkAll()
      .then((res) => onChecked(res.new_changes, res.errors.length))
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => onCheckingChange(false))
  }, [checking])
```

Add `import { useEffect } from 'react'`, drop the local `checking` state and the `RefreshCw`/check button JSX, and keep only the add input + `Plus` button. The input keeps `data-testid="add-proceeding-input"` and the button keeps `data-testid="add-proceeding-button"`. The check-all `data-testid="check-all-button"` now lives on the header's "Aktualizovat" button (already present in the `ListHeader` code above).

- [ ] **Step 7: Verify the check-all testid is on the header button**

Confirm `data-testid="check-all-button"` is present on the "Aktualizovat" `Button` inside `ListHeader` (it is, in the Step 5 code). No edit needed unless missing.

- [ ] **Step 8: Typecheck, lint, run tests**

Run: `cd 'frontend/apps/katastr:katastr' && bun run typecheck && bunx eslint --max-warnings=0 src/components/proceedings-list.tsx src/components/proceeding-row.tsx src/components/proceeding-row.test.tsx src/components/add-proceeding-form.tsx && bun run test`
Expected: typecheck clean; per-file lint clean (0 warnings — these files previously had warnings that your rewrite removes); `timeline.test.ts` and `proceeding-row.test.tsx` PASS.

- [ ] **Step 9: Commit**

```bash
git add "frontend/apps/katastr:katastr/src/components/proceedings-list.tsx" "frontend/apps/katastr:katastr/src/components/proceeding-row.tsx" "frontend/apps/katastr:katastr/src/components/proceeding-row.test.tsx" "frontend/apps/katastr:katastr/src/components/add-proceeding-form.tsx"
git commit -m "feat(katastr): CODEXIS shell + progressive-disclosure list rows"
```

---

### Task 5: Detail — compact status line + horizontal timeline + operations disclosure

**Files:**
- Create: `frontend/apps/katastr:katastr/src/components/detail/timeline.tsx`
- Modify: `frontend/apps/katastr:katastr/src/components/detail/status-grid.tsx` (becomes a compact status line)
- Modify: `frontend/apps/katastr:katastr/src/components/detail/operations-table.tsx` (becomes a collapsed disclosure, canonical mode only)
- Modify: `frontend/apps/katastr:katastr/src/components/proceeding-detail.tsx` (wire new order, wrap in centered column)
- Test: `frontend/apps/katastr:katastr/src/components/detail/timeline.test.tsx`

**Interfaces:**
- Consumes: `deriveTimeline`, `TimelineNode`, `VKLAD_PHASE_KEYS` from Task 2; i18n keys from Task 3.
- Produces: `<ProceedingTimeline proc={...} />` rendering the horizontal osa (vertical on mobile).

- [ ] **Step 1: Write the failing timeline render test**

Create `src/components/detail/timeline.test.tsx`:

```tsx
// @vitest-environment jsdom
import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import { ProceedingTimeline } from './timeline'
import type { ProceedingDetail } from '@/lib/schemas'

const proc = {
  uuid: 'u1',
  cislo_rizeni: 'V-1234/2026-701',
  typ_rizeni: 'Vklad',
  label: '',
  stav: 'Povolení vkladu',
  stav_uhrady: 'U',
  stav_uhrady_label: 'Uhrazeno',
  datum_prijeti: '2026-06-01',
  added_on: '2026-06-01',
  last_check_at: '2026-06-12',
  last_op_date: '2026-06-12',
  operace_count: 3,
  changes_count: 0,
  unconfirmed_count: 0,
  provedene_operace: [
    { nazev: 'Přijetí návrhu', datumProvedeni: '2026-06-01' },
    { nazev: 'Vyznačení plomby', datumProvedeni: '2026-06-01' },
    { nazev: 'Povolení vkladu', datumProvedeni: '2026-06-12' },
  ],
  changes: [],
} as ProceedingDetail

describe('ProceedingTimeline', () => {
  it('renders a node per canonical phase with testids', () => {
    render(<ProceedingTimeline proc={proc} />)
    // keys come back from i18n as raw keys in test (no provider)
    expect(screen.getByTestId('timeline-node-prijeti')).toBeTruthy()
    expect(screen.getByTestId('timeline-node-plomba')).toBeTruthy()
    expect(screen.getByTestId('timeline-node-povoleniVkladu')).toBeTruthy()
    expect(screen.getByTestId('timeline-node-vyrozumeni')).toBeTruthy()
  })

  it('marks the current node', () => {
    render(<ProceedingTimeline proc={proc} />)
    expect(screen.getByTestId('timeline-node-povoleniVkladu').getAttribute('data-state')).toBe(
      'current',
    )
  })
})
```

- [ ] **Step 2: Run the test to verify it fails**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/components/detail/timeline.test.tsx`
Expected: FAIL — cannot resolve `./timeline`.

- [ ] **Step 3: Implement `detail/timeline.tsx`**

Horizontal on `sm+` (flex row, connecting segments), vertical on mobile (flex col with a left rail). Each node carries `data-testid="timeline-node-<key>"` and `data-state`.

```tsx
import { useTranslation } from 'react-i18next'
import { deriveTimeline, type TimelineNode } from '@/lib/timeline'
import { formatDate } from '@/lib/format'
import type { ProceedingDetail } from '@/lib/schemas'

interface ProceedingTimelineProps {
  proc: ProceedingDetail
}

function dotClass(node: TimelineNode): string {
  if (node.isPlomba) return 'bg-plomba border-plomba'
  if (node.state === 'future') return 'bg-background border-border border-dashed'
  return 'bg-primary border-primary'
}

export function ProceedingTimeline({ proc }: ProceedingTimelineProps) {
  const { t } = useTranslation()
  const nodes = deriveTimeline(proc)

  const nodeName = (n: TimelineNode) => (n.nameKey ? t(`proceedings.timeline.${n.nameKey}`) : n.rawName)

  return (
    <section>
      <h2 className="font-display mb-5 text-[15px]">{t('proceedings.timeline.title')}</h2>
      {/* horizontal on sm+, vertical on mobile */}
      <ol className="flex flex-col gap-0 sm:flex-row sm:items-start sm:gap-0">
        {nodes.map((n, i) => (
          <li
            key={n.key}
            data-testid={`timeline-node-${n.key}`}
            data-state={n.state}
            className="relative flex flex-1 gap-3 pb-5 sm:flex-col sm:items-center sm:gap-2 sm:pb-0 sm:text-center"
          >
            {/* connector segment to the previous node (hidden on first) */}
            {i > 0 && (
              <span
                className={`absolute hidden h-0.5 sm:block ${
                  n.state === 'future' ? 'bg-border' : 'bg-primary'
                }`}
                style={{ top: '6px', right: '50%', width: '100%' }}
                aria-hidden
              />
            )}
            <span
              className={`relative z-10 mt-1 size-3.5 shrink-0 rounded-full border-2 sm:mt-0 ${dotClass(
                n,
              )} ${n.state === 'current' ? 'ring-brand-50 ring-4' : ''}`}
            />
            <div className="min-w-0 sm:px-1">
              <div
                className={`text-xs font-semibold leading-tight ${
                  n.state === 'future' ? 'text-muted-foreground font-medium' : ''
                }`}
              >
                {nodeName(n)}
                {n.isPlomba && <span className="text-plomba ml-1">⬡</span>}
                {n.state === 'current' && (
                  <span className="bg-brand-50 text-brand-700 ml-1 rounded px-1 text-[10px] font-bold">
                    {t('proceedings.timeline.now')}
                  </span>
                )}
              </div>
              <div className="text-muted-foreground mt-0.5 text-[11px] tabular-nums">
                {n.date ? formatDate(n.date) : t('proceedings.timeline.expected')}
              </div>
            </div>
          </li>
        ))}
      </ol>
    </section>
  )
}
```

- [ ] **Step 4: Run the test to verify it passes**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/components/detail/timeline.test.tsx`
Expected: PASS.

- [ ] **Step 5: Convert `status-grid.tsx` into a compact status line**

Replace the 4-box grid with a single wrapping line (`stav · úhrada · datum přijetí · poslední kontrola`):

```tsx
import { useTranslation } from 'react-i18next'
import { StavBadge } from '../stav-badge'
import { UhradaBadge } from '../uhrada-badge'
import { formatDateTime } from '@/lib/format'
import type { ProceedingDetail } from '@/lib/schemas'

interface StatusGridProps {
  proc: ProceedingDetail
}

export function StatusGrid({ proc }: StatusGridProps) {
  const { t } = useTranslation()
  const sep = <span className="text-border">·</span>
  return (
    <div className="text-muted-foreground flex flex-wrap items-center gap-x-2 gap-y-1 text-sm">
      <StavBadge stav={proc.stav} />
      {sep}
      <span>{t('proceedings.colUhrada')}</span>
      <UhradaBadge stavUhrady={proc.stav_uhrady} label={proc.stav_uhrady_label} />
      {sep}
      <span>{t('proceedings.datumPrijeti')}</span>
      <span className="text-foreground">
        {proc.datum_prijeti ? formatDateTime(proc.datum_prijeti) : '---'}
      </span>
      {sep}
      <span>{t('proceedings.lastCheck')}</span>
      <span className="text-foreground">
        {proc.last_check_at ? formatDateTime(proc.last_check_at) : '---'}
      </span>
    </div>
  )
}
```

- [ ] **Step 6: Convert `operations-table.tsx` into a collapsed disclosure**

Use a native `<details>` so all operations stay reachable without dominating the detail:

```tsx
import { useTranslation } from 'react-i18next'
import { formatDateTime } from '@/lib/format'
import type { Operace } from '@/lib/schemas'

interface OperationsTableProps {
  operations: Operace[]
}

export function OperationsTable({ operations }: OperationsTableProps) {
  const { t } = useTranslation()
  if (operations.length === 0) return null
  return (
    <details className="rounded-lg border">
      <summary className="text-muted-foreground cursor-pointer px-4 py-2 text-sm">
        {t('proceedings.allOperations')} ({operations.length})
      </summary>
      <ul className="border-t">
        {operations.map((op, i) => (
          <li key={i} className="flex justify-between border-b px-4 py-2 text-sm last:border-b-0">
            <span>{op.nazev}</span>
            <span className="text-muted-foreground tabular-nums">
              {formatDateTime(op.datumProvedeni)}
            </span>
          </li>
        ))}
      </ul>
    </details>
  )
}
```

- [ ] **Step 7: Rewire `proceeding-detail.tsx` (centered column + new order)**

Wrap content in a centered max-width column and order it status line → timeline → operations disclosure → change history. Render the operations disclosure only for vklad (`V-`) proceedings, since in fallback mode the timeline already lists every operation:

```tsx
import { useEffect, useState } from 'react'
import { ListSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { DetailHeader } from './detail/detail-header'
import { StatusGrid } from './detail/status-grid'
import { ProceedingTimeline } from './detail/timeline'
import { OperationsTable } from './detail/operations-table'
import { ChangeHistory } from './detail/change-history'
import { useDetail } from '@/hooks/use-detail'

interface ProceedingDetailProps {
  uuid: string
  onBack: () => void
}

export function ProceedingDetail({ uuid, onBack }: ProceedingDetailProps) {
  const { data, loading, error, refetch } = useDetail(uuid)
  const [labelOverride, setLabelOverride] = useState<string | null>(null)

  useEffect(() => {
    if (labelOverride !== null && data?.proceeding.label === labelOverride) {
      setLabelOverride(null)
    }
  }, [data?.proceeding.label, labelOverride])

  if (loading && !data) return <ListSkeleton />
  if (error) return <ErrorMessage message={error.message} onRetry={refetch} />
  if (!data) return null

  const proc = data.proceeding
  const currentLabel = labelOverride !== null ? labelOverride : proc.label
  const isVklad =
    proc.cislo_rizeni.toUpperCase().startsWith('V-') ||
    proc.typ_rizeni.toLowerCase().includes('vklad')

  return (
    <div className="flex h-screen flex-col" data-testid="app-root">
      <div className="min-h-0 flex-1 overflow-y-auto">
        <div className="mx-auto max-w-3xl space-y-6 p-4 sm:p-6">
          <DetailHeader
            uuid={uuid}
            cisloRizeni={proc.cislo_rizeni}
            label={currentLabel}
            onBack={onBack}
            onLabelChanged={(next) => {
              setLabelOverride(next)
              refetch()
            }}
            onDeleted={onBack}
          />
          <StatusGrid proc={proc} />
          <ProceedingTimeline proc={proc} />
          {isVklad && <OperationsTable operations={proc.provedene_operace} />}
          <ChangeHistory uuid={uuid} changes={proc.changes} onConfirmed={refetch} />
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 8: Sync change-history unread accent to brand**

In `detail/change-history.tsx`, swap the blue unread accent for the brand tint. Replace the unread `className` branch:

```tsx
              className={`rounded-lg border p-3 ${
                isUnread ? 'border-brand-200 bg-brand-50/60' : ''
              }`}
```

- [ ] **Step 9: Typecheck, lint, run all tests**

Run: `cd 'frontend/apps/katastr:katastr' && bun run typecheck && bunx eslint --max-warnings=0 src/components/detail/timeline.tsx src/components/detail/timeline.test.tsx src/components/detail/status-grid.tsx src/components/detail/operations-table.tsx src/components/detail/change-history.tsx src/components/proceeding-detail.tsx && bun run test`
Expected: typecheck clean; per-file lint clean (your rewrites must also clear the pre-existing `array-type`/`import/order` warnings in `change-history.tsx`, `operations-table.tsx`, `status-grid.tsx` — use `Array<T>` and correct import order); all vitest suites PASS.

- [ ] **Step 10: Commit**

```bash
git add "frontend/apps/katastr:katastr/src/components/detail" "frontend/apps/katastr:katastr/src/components/proceeding-detail.tsx"
git commit -m "feat(katastr): detail with compact status, horizontal timeline, operations disclosure"
```

---

### Task 6: Build, sync component, verify

**Files:**
- No source edits. Produces built assets in `plugins/katastr/components/katastr/` via the existing pipeline.

**Interfaces:**
- Consumes: all prior tasks.
- Produces: a synced component bundle; this is the integration gate.

- [ ] **Step 1: Full app verification**

Run: `cd 'frontend/apps/katastr:katastr' && bun run typecheck && bun run lint && bun run test`
Expected: typecheck/lint clean; all vitest suites PASS.

- [ ] **Step 2: Production build**

Run: `cd 'frontend/apps/katastr:katastr' && bun run build`
Expected: `tsc` passes and `vite build` writes `dist/` with no errors.

- [ ] **Step 3: Build + sync the plugin component (Docker pipeline)**

Run: `bash plugins/katastr/components/build.sh`
Expected: Docker build succeeds; assets sync into `plugins/katastr/components/katastr/`; `icon.png`/`icon.svg` preserved. If Docker is unavailable in this environment, note it and rely on Step 2's `dist/` as the build gate.

- [ ] **Step 4: Inspect the diff for the synced bundle**

Run: `git status --short plugins/katastr/components/katastr`
Expected: only built assets changed (hashed JS/CSS, `index.html`); no stray source or icon changes.

- [ ] **Step 5: Commit the built component**

```bash
git add plugins/katastr/components/katastr
git commit -m "build(katastr): sync redesigned component bundle"
```

---

## Self-Review

**1. Spec coverage**

| Spec section | Task |
|---|---|
| §3.1 styles.css plomba tokens | Task 1 |
| §3.2 unify badges | Task 1 |
| §4 deriveTimeline (modes A/B, milestone) | Task 2 |
| §4.3 latest milestone for list | Task 2 + Task 4 |
| §5.1 shell + header + row (var A, no operation dump) | Task 4 |
| §5.1 Add/Refresh in header, inline add input | Task 4 |
| §5.2 compact status line | Task 5 |
| §5.2 horizontal timeline (mobile vertical) | Task 5 |
| §5.2 operations disclosure (canonical mode) | Task 5 |
| §5.2 change-history color sync | Task 5 |
| §6 states (pre-config/loading/error/empty) | Task 4 |
| §7 i18n keys cs/en/sk | Task 3 |
| §8 build + sync | Task 6 |
| §9 testing (deriveTimeline unit, no-regression testids) | Tasks 2,4,5 |

No uncovered spec requirement.

**2. Placeholder scan:** No TBD/TODO; every code step shows full code; test steps show full assertions.

**3. Type consistency:** `deriveTimeline`/`latestMilestone`/`TimelineNode`/`VKLAD_PHASE_KEYS` defined in Task 2 are consumed with identical names/signatures in Tasks 4–5. `ProceedingTimeline` prop `proc: ProceedingDetail` matches its usage in `proceeding-detail.tsx`. i18n key paths (`proceedings.timeline.<phaseKey>`) match `VKLAD_PHASE_KEYS` and Task 3's JSON. `data-testid` names preserved: `proceeding-row-*`, `open-settings`, `add-proceeding-input`, `add-proceeding-button`, `check-all-button` (moved to header), `proceedings-table`, `empty-state`, `back-to-list`, `remove-proceeding`, `label-input`, `label-edit-trigger`, `app-root`.

**Known follow-up (not blocking):** the render-smoke tests assume react-i18next returns the raw key without a provider (vitest has no i18n init); the assertions target values/keys we control, so they hold regardless. If a global test i18n init is later added, the milestone assertions still pass.
