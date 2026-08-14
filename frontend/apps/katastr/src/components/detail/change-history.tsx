import { CheckCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { formatDate, formatTime } from '@/lib/format'
import type { ProceedingChange } from '@/lib/schemas'

interface ChangeHistoryProps {
  changes: Array<ProceedingChange>
  /** Marks the change at `index` read, or every unread one when index is omitted. */
  onConfirm: (index?: number) => void
  /** True while any confirm is in flight, so both controls go dead together. */
  confirming: boolean
}

/**
 * The state a proceeding was in after this check, in ČÚZK's own words.
 *
 * The previous state is never printed: changes are consecutive snapshots, so a
 * row's `old_stav` is always the `new_stav` of the row below it.
 *
 * Neither are the operation names. They are the very rows the Průběh tab draws,
 * there with ČÚZK's own timestamps, and ČÚZK names one event twice in two
 * vocabularies - "Řízení bylo ukončeno" as a state, "Ukončení řízení" as an
 * operation - so half the rows said it twice. They stay in the change record, so
 * `describe_change` still lists them in the CLI output and the e-mail.
 */
function changeLine(change: ProceedingChange, uhrada: (code?: string | null) => string): string {
  if (change.old_stav !== change.new_stav) return change.new_stav
  if (change.new_operations.length > 0)
    return change.new_operations.map((op) => op.nazev).join(', ')
  return uhrada(change.new_stav_uhrady)
}

export function ChangeHistory({ changes, onConfirm, confirming }: ChangeHistoryProps) {
  const { t } = useTranslation()

  if (changes.length === 0) {
    return (
      <div className="flex flex-col items-center px-6 py-10 text-center" data-testid="no-changes">
        <p className="text-foreground text-sm font-medium">{t('proceedings.noChanges')}</p>
        <p className="text-muted-foreground mt-1 max-w-xs text-xs text-pretty">
          {t('proceedings.noChangesHint')}
        </p>
      </div>
    )
  }

  // ČÚZK's own code is the fallback, so an unknown one shows rather than blanks.
  const uhrada = (code?: string | null) =>
    code ? t(`proceedings.uhrada.${code}`, { defaultValue: code }) : '---'

  // Newest first. The index is the one `confirmChange` takes, so it has to
  // survive the sort.
  const ordered = changes
    .map((change, index) => ({ change, index }))
    .sort((a, b) => (a.change.detected_on < b.change.detected_on ? 1 : -1))

  const unconfirmedCount = changes.filter((c) => !c.confirmed_on).length

  return (
    // Neither the columns nor the rows may be sized by what a row happens to
    // contain, because confirming a change removes things: the row's button, and
    // on the last one the header's button too. With the browser's defaults each
    // of those re-measured the table and moved everything under it - the header
    // one moved the whole list, since it sits above it.
    //
    // So the columns are declared and both row heights are pinned. `h-` on a
    // table row is a minimum, so a state name long enough to wrap still fits.
    <table className="w-full table-fixed border-collapse">
      <colgroup>
        {/* date and time, tabular figures, so it never needs more */}
        <col className="w-[9.5rem]" />
        <col />
        {/* the widest thing that can sit here is the header's own action */}
        <col className="w-[12rem]" />
      </colgroup>
      <thead>
        <tr className="text-muted-foreground text-[10.5px] font-bold tracking-wider uppercase">
          <th className="border-b pr-3 pb-2 text-left font-bold">{t('proceedings.colDetected')}</th>
          <th className="border-b pr-3 pb-2 text-left font-bold">{t('proceedings.colStav')}</th>
          {/* The bulk action heads the column of per-row ones, so it needs no
              row of its own - the row that used to hold it dropped the content
              of this tab below where the Průběh tab starts. */}
          <th className="border-b pb-2 text-right">
            {/* Kept in the tree once everything is read, only hidden. It is the
                tallest thing in this row, so removing it shortened the header -
                and the header sits above the list, so the whole list moved. */}
            <button
              type="button"
              onClick={() => onConfirm()}
              disabled={confirming || unconfirmedCount === 0}
              aria-hidden={unconfirmedCount === 0}
              tabIndex={unconfirmedCount === 0 ? -1 : undefined}
              data-testid={unconfirmedCount > 0 ? 'mark-all-read' : undefined}
              className={`text-muted-foreground hover:text-brand-700 hover:border-primary inline-flex items-center gap-1.5 rounded-md border px-2 py-1 text-[11px] font-semibold whitespace-nowrap transition-colors ${
                unconfirmedCount === 0 ? 'invisible' : ''
              }`}
            >
              <CheckCheck className="size-3.5" />
              {t('proceedings.markAllRead')}
            </button>
          </th>
        </tr>
      </thead>
      <tbody>
        {ordered.map(({ change, index }) => {
          const isUnread = !change.confirmed_on
          return (
            <tr key={index} className="border-secondary border-b last:border-b-0">
              <td className="text-muted-foreground py-2.5 pr-3 align-baseline text-xs whitespace-nowrap tabular-nums">
                <span
                  aria-hidden
                  className={`mr-2 inline-block size-[6px] rounded-full align-[1px] ${isUnread ? 'bg-primary' : ''}`}
                />
                {formatDate(change.detected_on)} {formatTime(change.detected_on)}
              </td>
              <td
                // Read and unread differ in colour, not in weight. Bold text is
                // wider, so confirming a change could re-wrap a long state name
                // and change the row's height under the reader's cursor.
                className={`py-2.5 pr-3 align-baseline text-[13.5px] leading-snug font-medium ${
                  isUnread ? 'text-foreground' : 'text-muted-foreground'
                }`}
              >
                {changeLine(change, uhrada)}
                {change.stav_uhrady_changed && change.old_stav !== change.new_stav && (
                  <span className="text-muted-foreground font-normal">
                    {' '}
                    &middot; {uhrada(change.new_stav_uhrady)}
                  </span>
                )}
              </td>
              <td className="py-2.5 text-right align-baseline">
                {/* Hidden rather than removed once the change is read: the button
                    is taller than the text beside it, so taking it out of the
                    tree shrank the row under the reader's cursor. Reserving the
                    space with the button itself beats guessing a row height. */}
                <button
                  type="button"
                  onClick={() => onConfirm(index)}
                  disabled={confirming || !isUnread}
                  aria-hidden={!isUnread}
                  tabIndex={isUnread ? undefined : -1}
                  data-testid={isUnread ? `mark-read-${index}` : undefined}
                  className={`text-muted-foreground hover:text-brand-700 hover:border-primary rounded-md border px-1.5 py-0.5 text-[11px] whitespace-nowrap transition-colors ${
                    isUnread ? '' : 'invisible'
                  }`}
                >
                  {t('proceedings.markRead')}
                </button>
              </td>
            </tr>
          )
        })}
      </tbody>
    </table>
  )
}
