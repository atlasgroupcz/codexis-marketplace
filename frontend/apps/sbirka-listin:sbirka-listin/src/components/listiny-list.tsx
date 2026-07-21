import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Check, Download, ExternalLink, FileText } from 'lucide-react'
import { toast } from '@workspace/ui/components/sonner'
import type { Listina } from '@/lib/schemas'
import { downloadDokument } from '@/lib/api'
import { DAEMON_FILE_URL } from '@/lib/constants'
import { errorMessage } from '@/lib/format'
import { byNewest } from '@/lib/listiny'

interface ListinyListProps {
  ico: string
  listiny: Array<Listina>
  /** Refetch the detail after a download, so the button flips to "Otevřít". */
  onChanged: () => void
}

export function ListinyList({ ico, listiny, onChanged }: ListinyListProps) {
  const { t } = useTranslation()

  if (listiny.length === 0) {
    return (
      <div className="flex flex-col items-center px-6 py-10 text-center" data-testid="no-listiny">
        <p className="text-foreground text-sm font-medium">{t('subjekty.noListiny')}</p>
        <p className="text-muted-foreground mt-1 max-w-xs text-xs text-pretty">
          {t('subjekty.noListinyHint')}
        </p>
      </div>
    )
  }

  const ordered = byNewest(listiny)

  return (
    <div data-testid="listiny-list">
      {ordered.map((listina) => (
        <ListinaRow key={listina.cislo} ico={ico} listina={listina} onDownloaded={onChanged} />
      ))}
    </div>
  )
}

function ListinaRow({
  ico,
  listina,
  onDownloaded,
}: {
  ico: string
  listina: Listina
  onDownloaded: () => void
}) {
  const { t } = useTranslation()
  const [downloading, setDownloading] = useState(false)

  const handleDownload = () => {
    // Saves the document server-side (in the daemon's app dir); on success the
    // detail refetches so this row flips to "Otevřít".
    setDownloading(true)
    downloadDokument(ico, listina.cislo)
      .then(() => {
        toast.success(t('subjekty.downloadDone'))
        onDownloaded()
      })
      .catch((e: unknown) => toast.error(errorMessage(e)))
      .finally(() => setDownloading(false))
  }

  const saved = Boolean(listina.path)
  // Ghost buttons — the downloaded state is carried by the green icon + tag, so
  // the actions stay quiet (opening isn't loud, downloading isn't pushed).
  const actBase =
    'inline-flex h-[30px] shrink-0 items-center gap-1.5 rounded-lg px-2.5 text-[12.5px] font-medium transition-colors'

  return (
    <div className="border-border/60 grid grid-cols-[32px_1fr_auto] items-start gap-3.5 border-b py-4 pr-1 pl-1 last:border-b-0">
      <span
        className={`mt-0.5 grid size-8 place-items-center rounded-lg border ${
          saved ? 'border-brand-100 bg-secondary text-brand-700' : 'border-border text-muted-foreground/70'
        }`}
      >
        <FileText className="size-4" strokeWidth={1.6} />
      </span>

      <div className="min-w-0">
        <p className="text-[13.5px] font-medium leading-snug">{listina.typ}</p>
        <p className="text-muted-foreground mt-2 flex flex-wrap items-center gap-x-1.5 gap-y-0.5 text-xs">
          <span className="text-muted-foreground/70 font-mono">{listina.cislo}</span>
          {listina.zalozeno && (
            <>
              <span className="text-border">·</span>
              <span>
                {t('subjekty.zalozeno')} {listina.zalozeno}
              </span>
            </>
          )}
          {saved && (
            <>
              <span className="text-border">·</span>
              <span className="text-brand-700 inline-flex items-center gap-1 font-medium">
                <Check className="size-3" strokeWidth={2.6} />
                {t('subjekty.saved')}
              </span>
            </>
          )}
        </p>
      </div>

      {listina.path ? (
        // Downloaded → open the file inline from the daemon (just the file).
        <a
          href={DAEMON_FILE_URL(listina.path)}
          target="_top"
          className={`${actBase} bg-secondary text-brand-700 hover:bg-brand-100`}
          data-testid={`open-${listina.cislo}`}
        >
          <ExternalLink className="size-3.5" />
          <span className="hidden sm:inline">{t('subjekty.open')}</span>
        </a>
      ) : (
        <button
          type="button"
          onClick={handleDownload}
          disabled={downloading}
          className={`${actBase} text-muted-foreground hover:bg-muted hover:text-foreground disabled:opacity-50`}
          data-testid={`download-${listina.cislo}`}
        >
          <Download className={`size-3.5 ${downloading ? 'animate-pulse' : ''}`} />
          <span className="hidden sm:inline">
            {downloading ? t('subjekty.downloading') : t('subjekty.download')}
          </span>
        </button>
      )}
    </div>
  )
}
