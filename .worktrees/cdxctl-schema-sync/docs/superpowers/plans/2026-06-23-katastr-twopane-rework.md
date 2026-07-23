# Katastr Two-Pane Rework Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Rework the katastr UI into a two-pane master–detail (mail-client) layout that fills the daemon content pane, declutter the list and change history, keep the timeline, and make the timeline safe for rejected/halted/unknown proceedings.

**Architecture:** Frontend-only, branch `lm-121-katastr-redesign`. Builds on the already-merged redesign. `app.tsx` becomes the shell (owns overview data + selection + header); `proceedings-list.tsx` becomes the left pane (always-visible add field + slim mail rows); `proceeding-detail.tsx` becomes the right pane (no full-page wrapper). `deriveTimeline` gains terminal/unknown handling. Backend, API, `schemas.ts` untouched.

**Tech Stack:** React 19, Vite 7, TS, Tailwind 4 + `@workspace/ui`, lucide-react, react-i18next, nuqs, zod, vitest.

**Spec:** `docs/superpowers/specs/2026-06-23-katastr-twopane-rework-design.md`

## Global Constraints

- Branch `lm-121-katastr-redesign`; do NOT touch `main`. Always quote the app dir: `'frontend/apps/katastr:katastr'`.
- Do NOT change `schemas.ts`, the API layer, or backend.
- CODEXIS green `#5ea500`; plomba `#c2410c`. Use existing tokens/classes (`bg-brand-50`, `text-brand-700`, `bg-plomba-soft`, `text-plomba`, `bg-secondary`, `font-display`).
- **Per-file lint gate:** each task's changed/created files must pass `bunx eslint --max-warnings=0 <files>` from the app dir. eslint is strict: `array-type` is `generic` (`Array<T>`, even `Array<string>`); `import/order` (type imports from `@/…` first, correct grouping); `react-hooks/static-components` (components at module scope, never inside render); `react-hooks/exhaustive-deps`; `@typescript-eslint/no-unnecessary-condition`. Whole-app `bun run lint` must be clean by the final build task.
- Preserve `data-testid`s and behaviors: `app-root`, `add-proceeding-input`, `add-proceeding-button`, `check-all-button`, `open-settings`, `proceeding-row-<cislo>`, `proceedings-table`, `empty-state`, `back-to-list`, `remove-proceeding`, `label-input`, `label-edit-trigger`.
- Tests from app dir: `bun --bun vitest run <file>`; full suite `bun run test`.

---

### Task 1: Timeline terminal/unknown safety + `isTerminal`

**Files:**
- Modify: `frontend/apps/katastr:katastr/src/lib/timeline.ts`
- Modify: `frontend/apps/katastr:katastr/src/lib/timeline.test.ts`
- Modify: `frontend/apps/katastr:katastr/src/components/detail/timeline.tsx`
- Modify: `frontend/apps/katastr:katastr/src/components/detail/timeline.test.tsx`

**Interfaces:**
- Produces: `TimelineNode` gains `isTerminal: boolean`. `deriveTimeline` returns a terminal node (no `future` nodes) when a vklad is negatively terminal or unrecognized-with-no-matching-ops.

- [ ] **Step 1: Add failing tests to `timeline.test.ts`**

Append these cases:

```ts
describe('deriveTimeline — terminal / unknown vklad', () => {
  const op = (nazev: string, datumProvedeni: string) => ({ nazev, datumProvedeni })

  it('rejected vklad: no future nodes, last node is terminal = stav', () => {
    const nodes = deriveTimeline({
      cislo_rizeni: 'V-9/2026-701',
      typ_rizeni: 'Vklad',
      stav: 'Zamítnuto',
      datum_prijeti: '2026-06-01',
      provedene_operace: [
        op('Přijetí návrhu', '2026-06-01'),
        op('Vyznačení plomby', '2026-06-02'),
      ],
    })
    expect(nodes.some((n) => n.state === 'future')).toBe(false)
    const last = nodes[nodes.length - 1]
    expect(last.state).toBe('current')
    expect(last.isTerminal).toBe(true)
    expect(last.rawName).toBe('Zamítnuto')
    // the real operations are kept as done nodes
    expect(nodes.filter((n) => n.state === 'done')).toHaveLength(2)
  })

  it('halted vklad with no ops: single terminal node from stav', () => {
    const nodes = deriveTimeline({
      cislo_rizeni: 'V-10/2026-701',
      typ_rizeni: 'Vklad',
      stav: 'Řízení zastaveno',
      datum_prijeti: '2026-06-01',
      provedene_operace: [],
    })
    expect(nodes).toHaveLength(1)
    expect(nodes[0].isTerminal).toBe(true)
    expect(nodes[0].state).toBe('current')
  })

  it('unknown stav with no matching ops: falls back, no fabricated future', () => {
    const nodes = deriveTimeline({
      cislo_rizeni: 'V-11/2026-701',
      typ_rizeni: 'Vklad',
      stav: 'Něco úplně jiného',
      datum_prijeti: '2026-06-01',
      provedene_operace: [],
    })
    expect(nodes.some((n) => n.state === 'future')).toBe(false)
    expect(nodes[nodes.length - 1].state).toBe('current')
  })

  it('normal in-progress vklad still gets the 5-phase axis with future steps', () => {
    const nodes = deriveTimeline({
      cislo_rizeni: 'V-12/2026-701',
      typ_rizeni: 'Vklad',
      stav: 'Povolení vkladu',
      datum_prijeti: '2026-06-01',
      provedene_operace: [op('Přijetí návrhu', '2026-06-01')],
    })
    expect(nodes).toHaveLength(5)
    expect(nodes.some((n) => n.state === 'future')).toBe(true)
    expect(nodes.every((n) => !n.isTerminal)).toBe(true)
  })
})
```

Also update the existing `TimelineNode` assertions if any check the object shape — add `isTerminal: false` to canonical/fallback expectations only where the test builds an exact object (most tests check fields individually, so no change needed there).

- [ ] **Step 2: Run tests, confirm new ones FAIL**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/lib/timeline.test.ts`
Expected: the 4 new cases fail (`isTerminal` undefined / future nodes present); existing 8 still pass.

- [ ] **Step 3: Implement in `timeline.ts`**

Add `isTerminal` to the interface and a negative-terminal detector; branch `deriveTimeline` to a terminal path.

```ts
export interface TimelineNode {
  key: string
  nameKey: string | null
  rawName: string | null
  date: string | null
  state: TimelineState
  isPlomba: boolean
  isTerminal: boolean
}
```

Add near the phase constants:

```ts
// Negative/closing outcomes — a vklad that ended without an entry.
const NEGATIVE_TERMINAL_STEMS = ['zamít', 'zastav', 'zpětvz', 'zpetvz', 'odmít', 'vzat zpět']

function isNegativeTerminal(stav: string): boolean {
  const s = stav.toLowerCase()
  return NEGATIVE_TERMINAL_STEMS.some((stem) => s.includes(stem))
}
```

In `deriveTimeline`, before returning the canonical axis, decide whether the canonical path is valid:

```ts
export function deriveTimeline(proc: TimelineInput): Array<TimelineNode> {
  if (!isVklad(proc)) return fallbackTimeline(proc, false)

  const negativeTerminal = isNegativeTerminal(proc.stav)
  const anyOpMatched = proc.provedene_operace.some((op) => matchPhaseIndex(op.nazev) >= 0)
  const stavRecognized = matchPhaseIndex(proc.stav) >= 0
  // Don't draw a speculative future axis when the proceeding ended negatively,
  // or when we can't place it on the canonical track at all.
  if (negativeTerminal || (!stavRecognized && !anyOpMatched)) {
    return fallbackTimeline(proc, true)
  }
  return canonicalTimeline(proc)
}
```

Update `canonicalTimeline` to set `isTerminal: false` on every node it returns.

Rework `fallbackTimeline` to accept a `terminal` flag and append a terminal current node from `stav` when terminal; otherwise behave as before:

```ts
function fallbackTimeline(proc: TimelineInput, terminal: boolean): Array<TimelineNode> {
  const ops = [...proc.provedene_operace].sort((a, b) =>
    a.datumProvedeni < b.datumProvedeni ? -1 : a.datumProvedeni > b.datumProvedeni ? 1 : 0,
  )

  if (terminal) {
    // All real operations are done; the current state is a distinct terminal node.
    const doneNodes: Array<TimelineNode> = ops.map((op, i) => ({
      key: `op:${i}`,
      nameKey: null,
      rawName: op.nazev,
      date: op.datumProvedeni,
      state: 'done',
      isPlomba: op.nazev.toLowerCase().includes('plomb'),
      isTerminal: false,
    }))
    doneNodes.push({
      key: 'terminal',
      nameKey: null,
      rawName: proc.stav,
      date: null,
      state: 'current',
      isPlomba: false,
      isTerminal: true,
    })
    return doneNodes
  }

  if (ops.length === 0) {
    return [
      {
        key: 'stav',
        nameKey: null,
        rawName: proc.stav,
        date: null,
        state: 'current',
        isPlomba: proc.stav.toLowerCase().includes('plomb'),
        isTerminal: false,
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
    isTerminal: false,
  }))
}
```

(Existing non-vklad callers now go through `fallbackTimeline(proc, false)` — same behavior as before, just with `isTerminal: false` added.) Keep all element types as `Array<...>` (generic) for lint.

- [ ] **Step 4: Run tests, confirm all PASS**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/lib/timeline.test.ts`
Expected: all (8 existing + 4 new) pass.

- [ ] **Step 5: Add terminal visual state to `timeline.tsx` + a test**

In `timeline.tsx`, give the terminal node a distinct, ring-less, muted/plomba marker. Update `dotClass`:

```tsx
function dotClass(node: TimelineNode): string {
  if (node.isTerminal) return 'bg-plomba border-plomba opacity-80'
  if (node.isPlomba) return 'bg-plomba border-plomba'
  if (node.state === 'future') return 'bg-background border-border border-dashed'
  return 'bg-primary border-primary'
}
```

And suppress the green "current" ring + the `now` chip when the node is terminal (a terminal node is the end, not "now"):

```tsx
            <span
              className={`relative z-10 mt-1 size-3.5 shrink-0 rounded-full border-2 sm:mt-0 ${dotClass(
                n,
              )} ${n.state === 'current' && !n.isTerminal ? 'ring-brand-50 ring-4' : ''}`}
            />
```

```tsx
                {n.state === 'current' && !n.isTerminal && (
                  <span className="bg-brand-50 text-brand-700 ml-1 rounded px-1 text-[10px] font-bold">
                    {t('proceedings.timeline.now')}
                  </span>
                )}
```

In `timeline.test.tsx`, add a case rendering a rejected vklad and asserting the terminal node exists and does NOT carry the `now` chip / ring (assert via `data-state` + absence of the now text):

```tsx
  it('rejected vklad renders a terminal node without the "now" chip', () => {
    const rejected = {
      ...proc,
      stav: 'Zamítnuto',
      provedene_operace: [{ nazev: 'Přijetí návrhu', datumProvedeni: '2026-06-01' }],
    } as ProceedingDetail
    render(<ProceedingTimeline proc={rejected} />)
    expect(screen.getByTestId('timeline-node-terminal')).toBeTruthy()
    expect(screen.queryByText('proceedings.timeline.now')).toBeNull()
  })
```

(The node key for the terminal node is `terminal`, so its testid is `timeline-node-terminal`.)

- [ ] **Step 6: Run, typecheck, per-file lint, commit**

Run: `cd 'frontend/apps/katastr:katastr' && bun run typecheck && bunx eslint --max-warnings=0 src/lib/timeline.ts src/lib/timeline.test.ts src/components/detail/timeline.tsx src/components/detail/timeline.test.tsx && bun --bun vitest run src/lib/timeline.test.ts src/components/detail/timeline.test.tsx`
Expected: typecheck clean, lint 0, all pass.

```bash
git add "frontend/apps/katastr:katastr/src/lib/timeline.ts" "frontend/apps/katastr:katastr/src/lib/timeline.test.ts" "frontend/apps/katastr:katastr/src/components/detail/timeline.tsx" "frontend/apps/katastr:katastr/src/components/detail/timeline.test.tsx"
git commit -m "feat(katastr): timeline terminal-state handling for rejected/halted proceedings"
```

---

### Task 2: Two-pane shell + slim left list + mail rows

**Files:**
- Modify: `frontend/apps/katastr:katastr/src/app.tsx`
- Modify: `frontend/apps/katastr:katastr/src/components/proceedings-list.tsx`
- Modify: `frontend/apps/katastr:katastr/src/components/proceeding-row.tsx`
- Modify: `frontend/apps/katastr:katastr/src/components/proceeding-detail.tsx` (remove its full-page wrapper only)
- Modify: `frontend/apps/katastr:katastr/src/components/detail/detail-header.tsx` (back button mobile-only)
- Modify: `frontend/apps/katastr:katastr/src/components/proceeding-row.test.tsx`
- Add i18n key: `proceedings.selectHint` in cs/en/sk (`public/locales/*/translation.json`)

**Interfaces:**
- `App` owns `useOverview` and renders the shell: header + two-pane. It passes overview data into `ProceedingsList` and renders `ProceedingDetail` in the right pane.
- New `ProceedingsList` props: `{ data: OverviewResponse; loading: boolean; error: Error | null; refetch: () => void; selectedUuid: string | null; onSelect: (uuid: string) => void; onOpenSettings: () => void }` — it renders the add field + the list rows + empty/loading/error/pre-config states. It NO LONGER calls `useOverview` itself or renders the header/shell.
- `ProceedingRow` props unchanged (`proceeding`, `onClick`) plus `selected: boolean`.

**Decision (hook ownership):** `App` is the single owner of `useOverview`. `ProceedingsList` receives data via props (no second fetch). `ProceedingDetail` keeps its own `useDetail(uuid)`, and gets an `onChanged` callback that calls the overview `refetch` (so confirming/removing updates the left-pane dots).

- [ ] **Step 1: Update the row render test (TDD for the slim row)**

In `proceeding-row.test.tsx`, the row no longer shows úhrada / milestone / dates — only číslo, label, stav. Replace the assertions:

```tsx
  it('shows číslo, label, stav badge; no úhrada/milestone/date meta', () => {
    render(<ProceedingRow proceeding={proc} onClick={() => {}} selected={false} />)
    expect(screen.getByText('V-1234/2026-701')).toBeTruthy()
    expect(screen.getByText(/Byt Brno-střed/)).toBeTruthy()
    // stav appears
    expect(screen.getAllByText(/Povolení vkladu/).length).toBeGreaterThan(0)
    // decluttered: no "Uhrazeno", no "Poslední milník", no operation dump
    expect(screen.queryByText(/Uhrazeno/)).toBeNull()
    expect(screen.queryByText(/Vyznačení plomby/)).toBeNull()
    expect(screen.queryByText('Přijetí návrhu')).toBeNull()
  })

  it('shows the unread dot when there are unconfirmed changes', () => {
    render(<ProceedingRow proceeding={proc} onClick={() => {}} selected={false} />)
    expect(screen.getByTestId('unread-dot')).toBeTruthy()
  })
```

(`proc` has `unconfirmed_count: 2`, so the dot shows. Keep the existing fixture.)

- [ ] **Step 2: Run, confirm FAIL**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/components/proceeding-row.test.tsx`
Expected: fails — current row renders "Uhrazeno"/milestone and has no `selected` prop / `unread-dot`.

- [ ] **Step 3: Rewrite `proceeding-row.tsx` as a mail row**

```tsx
import { useTranslation } from 'react-i18next'
import { StavBadge } from './stav-badge'
import type { ProceedingSummary } from '@/lib/schemas'

interface ProceedingRowProps {
  proceeding: ProceedingSummary
  selected: boolean
  onClick: (uuid: string) => void
}

export function ProceedingRow({ proceeding, selected, onClick }: ProceedingRowProps) {
  const { t } = useTranslation()
  const hasUnread = proceeding.unconfirmed_count > 0
  return (
    <button
      type="button"
      onClick={() => onClick(proceeding.uuid)}
      data-testid={`proceeding-row-${proceeding.cislo_rizeni}`}
      aria-current={selected}
      className={`flex w-full items-start gap-2.5 border-b px-3 py-2.5 text-left transition-colors last:border-b-0 ${
        selected ? 'bg-secondary shadow-[inset_3px_0_0_var(--primary)]' : 'hover:bg-secondary/50'
      }`}
    >
      <span
        data-testid={hasUnread ? 'unread-dot' : undefined}
        aria-hidden
        className={`mt-1.5 size-2 shrink-0 rounded-full ${hasUnread ? 'bg-primary' : 'bg-transparent'}`}
      />
      <span className="min-w-0 flex-1">
        <span className="block truncate font-mono text-[13px] font-semibold">
          {proceeding.cislo_rizeni}
        </span>
        {proceeding.label && (
          <span className="text-muted-foreground block truncate text-xs">{proceeding.label}</span>
        )}
        <span className="mt-1.5 block">
          <StavBadge stav={proceeding.stav} />
        </span>
      </span>
      <span className="sr-only">{t('proceedings.colStav')}</span>
    </button>
  )
}
```

(`latestMilestone`/`formatDate` imports are dropped from this file. The `sr-only` line keeps an accessible label; drop it if it trips no-unnecessary lint — it's optional.)

- [ ] **Step 4: Run row test, confirm PASS**

Run: `cd 'frontend/apps/katastr:katastr' && bun --bun vitest run src/components/proceeding-row.test.tsx`
Expected: pass.

- [ ] **Step 5: Add `proceedings.selectHint` to the three locales**

In each `public/locales/<l>/translation.json`, inside `proceedings`, add:
- cs: `"selectHint": "Vyberte řízení ze seznamu vlevo."`
- en: `"selectHint": "Select a proceeding from the list on the left."`
- sk: `"selectHint": "Vyberte konanie zo zoznamu vľavo."`

- [ ] **Step 6: Rewrite `app.tsx` as the shell (header + two-pane + responsive)**

```tsx
import { useState } from 'react'
import { useQueryState } from 'nuqs'
import { Landmark, RefreshCw, Settings as SettingsIcon } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@workspace/ui/components/button'
import { Toaster, toast } from '@workspace/ui/components/sonner'
import { uuidParser, viewParser } from '@/lib/url-state'
import { ProceedingsList } from '@/components/proceedings-list'
import { ProceedingDetail } from '@/components/proceeding-detail'
import { SettingsDialog } from '@/components/settings-dialog'
import { useOverview } from '@/hooks/use-overview'
import { checkAll } from '@/lib/api'

export function App() {
  const { t } = useTranslation()
  const { data, loading, error, refetch } = useOverview()
  const [view, setView] = useQueryState('view', viewParser)
  const [uuid, setUuid] = useQueryState('uuid', uuidParser)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [checking, setChecking] = useState(false)

  const apiKeyConfigured = data?.api_key_configured ?? false
  const detailOpen = view === 'detail' && uuid !== null

  const handleSelect = (next: string) => {
    void setView('detail')
    void setUuid(next)
  }
  const handleBack = () => {
    void setView('list')
    void setUuid(null)
  }
  const handleCheck = () => {
    setChecking(true)
    checkAll()
      .then((res) => {
        if (res.errors.length > 0) {
          toast.error(t('proceedings.checkResultErrors', { newChanges: res.new_changes, errors: res.errors.length }))
        } else if (res.new_changes > 0) {
          toast.success(t('proceedings.checkResultChanges', { count: res.new_changes }))
        } else {
          toast.success(t('proceedings.checkResultNone'))
        }
        refetch()
      })
      .catch((e: unknown) => toast.error(e instanceof Error ? e.message : String(e)))
      .finally(() => setChecking(false))
  }

  return (
    <div className="bg-background text-foreground flex h-screen flex-col" data-testid="app-root">
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
          {apiKeyConfigured && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleCheck}
              disabled={checking}
              data-testid="check-all-button"
            >
              <RefreshCw className={`size-4 ${checking ? 'animate-spin' : ''}`} />
              <span className="hidden sm:inline">{t('proceedings.checkAll')}</span>
            </Button>
          )}
          <Button
            variant="ghost"
            size="sm"
            onClick={() => setSettingsOpen(true)}
            title={t('settings.title')}
            data-testid="open-settings"
          >
            <SettingsIcon className="size-4" />
          </Button>
        </div>
      </header>

      <main className="grid min-h-0 flex-1 md:grid-cols-[300px_1fr]">
        <section
          className={`min-h-0 overflow-y-auto border-r ${detailOpen ? 'hidden md:block' : 'block'}`}
        >
          <ProceedingsList
            data={data}
            loading={loading}
            error={error}
            refetch={refetch}
            selectedUuid={uuid}
            onSelect={handleSelect}
            onOpenSettings={() => setSettingsOpen(true)}
          />
        </section>
        <section className={`min-h-0 overflow-y-auto ${detailOpen ? 'block' : 'hidden md:block'}`}>
          {detailOpen && uuid ? (
            <ProceedingDetail uuid={uuid} onBack={handleBack} onChanged={refetch} />
          ) : (
            <div className="text-muted-foreground flex h-full items-center justify-center p-8 text-sm">
              {t('proceedings.selectHint')}
            </div>
          )}
        </section>
      </main>

      <SettingsDialog
        open={settingsOpen}
        onOpenChange={setSettingsOpen}
        configured={data?.api_key_configured ?? false}
        maskedKey={data?.api_key_masked ?? ''}
        onSaved={refetch}
      />
      <Toaster position="bottom-right" />
    </div>
  )
}
```

(If `toast` is not exported from `@workspace/ui/components/sonner`, import it from wherever the other components import it — check `add-proceeding-form.tsx`'s import.)

- [ ] **Step 7: Rewrite `proceedings-list.tsx` as the left pane (props-driven, add field on top)**

It renders: the always-visible `AddProceedingForm` (top), then the list / states. It takes the new props (Interfaces above), no `useOverview`, no header/shell. Pre-config (no API key) → show `ApiKeyBanner`. Loading → `ListSkeleton`. Error → `ErrorMessage`. Empty → `empty-state`. Else → add form + rows.

```tsx
import { useTranslation } from 'react-i18next'
import { toast } from '@workspace/ui/components/sonner'
import { ListSkeleton } from './loading-skeleton'
import { ErrorMessage } from './error-message'
import { ProceedingRow } from './proceeding-row'
import { AddProceedingForm } from './add-proceeding-form'
import { ApiKeyBanner } from './api-key-banner'
import type { OverviewResponse } from '@/lib/schemas'

interface ProceedingsListProps {
  data: OverviewResponse | null
  loading: boolean
  error: Error | null
  refetch: () => void
  selectedUuid: string | null
  onSelect: (uuid: string) => void
  onOpenSettings: () => void
}

export function ProceedingsList({
  data,
  loading,
  error,
  refetch,
  selectedUuid,
  onSelect,
  onOpenSettings,
}: ProceedingsListProps) {
  const { t } = useTranslation()

  if (loading && !data) return <div className="p-3"><ListSkeleton /></div>
  if (error) return <div className="p-3"><ErrorMessage message={error.message} onRetry={refetch} /></div>
  if (!data) return null

  if (!data.api_key_configured) {
    return (
      <div className="p-3">
        <ApiKeyBanner onOpenSettings={onOpenSettings} />
      </div>
    )
  }

  return (
    <div>
      <div className="border-b p-3">
        <AddProceedingForm onAdded={refetch} />
      </div>
      {data.proceedings.length === 0 ? (
        <p className="text-muted-foreground p-4 text-sm" data-testid="empty-state">
          {t('proceedings.empty')}
        </p>
      ) : (
        <div data-testid="proceedings-table">
          {data.proceedings.map((proc) => (
            <ProceedingRow
              key={proc.uuid}
              proceeding={proc}
              selected={proc.uuid === selectedUuid}
              onClick={onSelect}
            />
          ))}
        </div>
      )}
    </div>
  )
}
```

(`AddProceedingForm` is the add-only version from the prior task; it takes `onAdded`. Keep the `toast` import only if used; remove if not — lint will tell you.)

- [ ] **Step 8: Trim `AddProceedingForm` layout for the narrow left pane**

The form sits in a ~300px column. Make the input + button stack vertically (input full width, button under it) instead of side-by-side, so it fits. Keep testids and behavior. (Change only the wrapper classes — e.g. the input row to `flex flex-col gap-2`.)

- [ ] **Step 9: Remove the full-page wrapper from `proceeding-detail.tsx` and add `onChanged`**

The shell now owns `app-root` and the scroll container, so the detail must be a plain content block (no `flex h-screen`, no `app-root`). Add an `onChanged?: () => void` prop and call it after confirm/remove so the overview refreshes.

```tsx
interface ProceedingDetailProps {
  uuid: string
  onBack: () => void
  onChanged?: () => void
}
```

Replace the outer wrapper:

```tsx
  return (
    <div className="mx-auto max-w-2xl space-y-6 p-4 sm:p-6">
      <DetailHeader
        uuid={uuid}
        cisloRizeni={proc.cislo_rizeni}
        label={currentLabel}
        onBack={onBack}
        onLabelChanged={(next) => {
          setLabelOverride(next)
          refetch()
          onChanged?.()
        }}
        onDeleted={() => {
          onChanged?.()
          onBack()
        }}
      />
      <StatusGrid proc={proc} />
      <ProceedingTimeline proc={proc} />
      {isVklad && <OperationsTable operations={proc.provedene_operace} />}
      <ChangeHistory
        uuid={uuid}
        changes={proc.changes}
        onConfirmed={() => {
          refetch()
          onChanged?.()
        }}
      />
    </div>
  )
```

(The timeline gets the full content width up to `max-w-2xl`; that's the "readable measure" from the spec. Keep `isVklad` as-is.)

- [ ] **Step 10: Make the detail-header back button mobile-only**

In `detail-header.tsx`, add `className="md:hidden"` (or wrap) on the back `Button` so it only shows on mobile (on desktop both panes are visible, so "back" is meaningless). Keep `data-testid="back-to-list"`.

- [ ] **Step 11: Typecheck, per-file lint, full test suite**

Run: `cd 'frontend/apps/katastr:katastr' && bun run typecheck && bunx eslint --max-warnings=0 src/app.tsx src/components/proceedings-list.tsx src/components/proceeding-row.tsx src/components/proceeding-row.test.tsx src/components/proceeding-detail.tsx src/components/detail/detail-header.tsx src/components/add-proceeding-form.tsx && bun run test`
Expected: typecheck clean, lint 0, all suites pass.

- [ ] **Step 12: Commit**

```bash
git add "frontend/apps/katastr:katastr/src/app.tsx" "frontend/apps/katastr:katastr/src/components/proceedings-list.tsx" "frontend/apps/katastr:katastr/src/components/proceeding-row.tsx" "frontend/apps/katastr:katastr/src/components/proceeding-row.test.tsx" "frontend/apps/katastr:katastr/src/components/proceeding-detail.tsx" "frontend/apps/katastr:katastr/src/components/detail/detail-header.tsx" "frontend/apps/katastr:katastr/src/components/add-proceeding-form.tsx" "frontend/apps/katastr:katastr/public/locales"
git commit -m "feat(katastr): two-pane master-detail shell with always-on add field"
```

---

### Task 3: Declutter the change history

**Files:**
- Modify: `frontend/apps/katastr:katastr/src/components/detail/change-history.tsx`

**Interfaces:** Consumes existing props (`uuid`, `changes`, `onConfirmed`). No signature change.

- [ ] **Step 1: Restyle the change list to calm rows**

Replace the big bordered/blue-tinted cards with quiet rows: header line "Historie změn · N nepotvrzené" + "Označit vše přečteno" on the right; each change is a row with a left brand bar when unread, a muted date, a compact `starý → nový` body, and a "✓ Potvrdit" affordance on the right for unread. Added operations / úhrada changes render as small sub-lines, not blocks. Keep all logic (confirm one, confirm all, `confirming` state) and `array-type: generic` (`Array<...>`). Example structure:

```tsx
  return (
    <section>
      <div className="mb-3 flex items-center justify-between border-t pt-4">
        <h2 className="text-[13px] font-semibold">
          {t('proceedings.changeHistory')}
          {unconfirmedCount > 0 && (
            <span className="text-brand-700"> · {t('proceedings.unconfirmedChangesBadge', { count: unconfirmedCount })}</span>
          )}
        </h2>
        {unconfirmedCount > 0 && (
          <button
            type="button"
            onClick={handleConfirmAll}
            disabled={confirming}
            className="text-muted-foreground hover:text-foreground rounded-md border px-2.5 py-1 text-xs"
          >
            {t('proceedings.markAllRead', { count: unconfirmedCount })}
          </button>
        )}
      </div>
      <div>
        {changes.map((change, i) => {
          const isUnread = !change.confirmed_on
          return (
            <div
              key={i}
              className={`relative border-b py-2.5 pl-3 last:border-b-0 ${
                isUnread ? 'shadow-[inset_3px_0_0_var(--primary)]' : ''
              }`}
            >
              {isUnread && (
                <button
                  type="button"
                  onClick={() => handleConfirmOne(i)}
                  disabled={confirming}
                  className="text-brand-700 absolute right-0 top-2.5 text-[11px]"
                >
                  {t('proceedings.markRead')}
                </button>
              )}
              <div className="text-muted-foreground text-[11px]">
                {formatDateTime(change.detected_on)}
              </div>
              {change.old_stav !== change.new_stav && (
                <div className="mt-0.5 text-[13px]">
                  <span className="text-muted-foreground">{change.old_stav}</span>
                  <span className="mx-1.5 text-[#bdbdb8]">&rarr;</span>
                  <span>{change.new_stav}</span>
                </div>
              )}
              {change.new_operations.map((op, j) => (
                <div key={j} className="text-muted-foreground mt-0.5 text-[11px]">
                  + {op.nazev} ({formatDateTime(op.datumProvedeni)})
                </div>
              ))}
              {change.stav_uhrady_changed && (
                <div className="text-muted-foreground mt-0.5 text-[11px]">
                  {t('proceedings.uhradaChanged')}: {change.old_stav_uhrady ?? '---'} &rarr; {change.new_stav_uhrady ?? '---'}
                </div>
              )}
            </div>
          )
        })}
      </div>
    </section>
  )
```

Keep the early `if (changes.length === 0) return null`. Preserve the confirm handlers exactly.

- [ ] **Step 2: Typecheck, per-file lint, tests**

Run: `cd 'frontend/apps/katastr:katastr' && bun run typecheck && bunx eslint --max-warnings=0 src/components/detail/change-history.tsx && bun run test`
Expected: clean + all pass.

- [ ] **Step 3: Commit**

```bash
git add "frontend/apps/katastr:katastr/src/components/detail/change-history.tsx"
git commit -m "feat(katastr): calmer change-history rows with brand unread accent"
```

---

### Task 4: Build, sync, verify

**Files:** No source edits. Produces synced bundle in `plugins/katastr/components/katastr/`.

- [ ] **Step 1: Whole-app verification**

Run: `cd 'frontend/apps/katastr:katastr' && bun run typecheck && bun run lint && bun run test`
Expected: typecheck clean, whole-app `bun run lint` 0 warnings, all vitest suites pass. If lint reports warnings, list them with file:line and STOP (DONE_WITH_CONCERNS) — do not fix unrelated files.

- [ ] **Step 2: Production build**

Run: `cd 'frontend/apps/katastr:katastr' && bun run build`
Expected: `tsc` + `vite build` write `dist/` with no errors.

- [ ] **Step 3: Build + sync the plugin component**

Run: `bash plugins/katastr/components/build.sh`
Expected: Docker build succeeds; assets sync into `plugins/katastr/components/katastr/`; `icon.png`/`icon.svg` preserved. If Docker is unavailable, note it and rely on Step 2 (DONE_WITH_CONCERNS).

- [ ] **Step 4: Inspect synced diff**

Run: `git status --short plugins/katastr/components/katastr`
Expected: only built assets + locales changed; icons preserved.

- [ ] **Step 5: Commit the bundle**

```bash
git add plugins/katastr/components/katastr && git commit -m "build(katastr): sync two-pane redesign bundle"
```

---

## Self-Review

**Spec coverage:** two-pane shell (Task 2), fill-width + readable detail measure (Task 2 `max-w-2xl`), slim mail rows + unread dot (Task 2), always-visible add field above list (Task 2 steps 7–8), empty-selection hint (Task 2 step 5/6), declutter change history (Task 3), timeline terminal safety + `isTerminal` visual (Task 1), build/sync (Task 4). Mobile single-pane via `detailOpen` + `hidden md:block` (Task 2 step 6). Back button mobile-only (Task 2 step 10).

**Placeholder scan:** all steps carry concrete code or exact class/prop changes.

**Type consistency:** `ProceedingsList` new prop set defined in Task 2 interfaces and used identically in `app.tsx`. `ProceedingRow` gains `selected: boolean` (Task 2 row code + test + list usage all pass it). `ProceedingDetail` gains `onChanged?` (Task 2 app usage + detail signature). `TimelineNode.isTerminal` added in Task 1 and consumed in `timeline.tsx` Task 1. `deriveTimeline` returns `Array<TimelineNode>` (generic) throughout.

**Risk to watch during review:** hook ownership move (overview now in `App`, passed to list) — verify no component still calls `useOverview` twice, and that confirm/remove in the detail refresh the left-pane unread dots via `onChanged`.
