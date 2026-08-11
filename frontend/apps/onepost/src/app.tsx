import {
  AlertCircle,
  Check,
  CheckCircle2,
  ChevronRight,
  ExternalLink,
  Eye,
  Loader2,
  RefreshCw,
  SquarePen,
  Trash2,
} from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'

interface SettingsPayload {
  configured: boolean
  maskedKey: string
  keySource: string
  apiUrl: string
  settingsFile: string
  generateUrl: string
}

interface ActionPayload {
  ok: boolean
  error?: string
  mailboxCount?: number | null
}

const emptySettings: SettingsPayload = {
  configured: false,
  maskedKey: '',
  keySource: '',
  apiUrl: '',
  settingsFile: '',
  generateUrl: '',
}

async function parseJson<T>(response: Response): Promise<T> {
  const text = await response.text()
  if (!text.trim()) {
    throw new Error(
      `Server nevrátil žádná data (HTTP ${response.status}). Zkontrolujte, že je plugin propojený s ONEPOSTem.`,
    )
  }
  try {
    return JSON.parse(text) as T
  } catch {
    throw new Error(`Neočekávaná odpověď serveru (HTTP ${response.status}).`)
  }
}

async function loadSettings(): Promise<SettingsPayload> {
  const response = await fetch('./api?settings=1', {
    headers: { Accept: 'application/json' },
    cache: 'no-store',
  })
  const payload = await parseJson<SettingsPayload | { error?: string }>(response)
  if (!response.ok) {
    throw new Error('error' in payload && payload.error ? payload.error : `HTTP ${response.status}`)
  }
  return payload as SettingsPayload
}

async function runAction(action: string, apiKey?: string): Promise<ActionPayload> {
  const response = await fetch('./api', {
    method: 'POST',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ action, apiKey }),
  })
  const payload = await parseJson<ActionPayload>(response)
  if (!response.ok || !payload.ok) {
    throw new Error(payload.error || `HTTP ${response.status}`)
  }
  return payload
}

function OnepostSymbol({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 169 220" className={className} aria-hidden="true">
      <path
        d="M52.6862 220V84.2901L0 91.6913V48.4034L112.49 34.3784V220.016H52.6862V220Z"
        fill="#0C4A6E"
      />
      <path d="M112.49 34.3784V6.81038L166.763 0V27.568L112.49 34.3629V34.3784Z" fill="#34D399" />
    </svg>
  )
}

const PRIMARY = '#7CB305'
const PRIMARY_HOVER = '#6EA004'

const primaryButton =
  'inline-flex items-center gap-2 px-4 py-2.5 rounded-xl text-white text-[13px] font-semibold transition-all shadow-sm shrink-0 disabled:opacity-40 disabled:cursor-default'

function primaryHandlers(enabled: boolean) {
  return {
    onMouseEnter: (e: React.MouseEvent<HTMLElement>) => {
      if (enabled) e.currentTarget.style.backgroundColor = PRIMARY_HOVER
    },
    onMouseLeave: (e: React.MouseEvent<HTMLElement>) => {
      e.currentTarget.style.backgroundColor = PRIMARY
    },
  }
}

export function App() {
  const [settings, setSettings] = useState<SettingsPayload>(emptySettings)
  const [apiKey, setApiKey] = useState('')
  const [busyAction, setBusyAction] = useState<'save' | 'test' | 'delete' | null>(null)
  const [status, setStatus] = useState<{ kind: 'ok' | 'error'; message: string } | null>(null)
  const onepostUrl = settings.generateUrl || ''
  const apiKeyValue = apiKey.trim()
  const isBusy = busyAction !== null
  const configured = settings.configured

  const refreshSettings = useCallback(async () => {
    setSettings(await loadSettings())
  }, [])

  useEffect(() => {
    refreshSettings().catch(() => {
      // Backend pluginu nemusí být v lokálním náhledu dostupný - ponecháme stav z initial settings.
    })
  }, [refreshSettings])

  async function withBusy(action: 'save' | 'test' | 'delete', fn: () => Promise<void>) {
    setBusyAction(action)
    setStatus(null)
    try {
      await fn()
    } catch (error: unknown) {
      setStatus({ kind: 'error', message: error instanceof Error ? error.message : String(error) })
    } finally {
      setBusyAction(null)
    }
  }

  const saveKey = () =>
    withBusy('save', async () => {
      await runAction('save_api_key', apiKeyValue)
      setApiKey('')
      await refreshSettings()
      setStatus({ kind: 'ok', message: 'API klíč byl uložen a ověřen.' })
    })

  const testKey = () =>
    withBusy('test', async () => {
      const result = await runAction('test_api_key')
      setStatus({
        kind: 'ok',
        message: `API klíč funguje. Dostupných schránek: ${result.mailboxCount ?? '?'}.`,
      })
    })

  const deleteKey = () =>
    withBusy('delete', async () => {
      await runAction('delete_api_key')
      await refreshSettings()
      setStatus({ kind: 'ok', message: 'Uložený API klíč byl smazán.' })
    })

  const statusBanner = status ? (
    <div
      className={
        status.kind === 'ok'
          ? 'mt-4 flex gap-2 rounded-lg border border-emerald-200 bg-emerald-50 px-3 py-2.5 text-[12.5px] text-emerald-800'
          : 'mt-4 flex gap-2 rounded-lg border border-red-200 bg-red-50 px-3 py-2.5 text-[12.5px] text-red-700'
      }
      role="status"
      aria-live="polite"
    >
      {status.kind === 'ok' ? (
        <CheckCircle2 className="mt-0.5 size-4 shrink-0" />
      ) : (
        <AlertCircle className="mt-0.5 size-4 shrink-0" />
      )}
      <span>{status.message}</span>
    </div>
  ) : null

  return (
    <div className="flex-1 overflow-y-auto bg-white text-gray-800 min-h-screen">
      {/* Breadcrumb */}
      <div className="px-5 sm:px-6 py-3 border-b border-gray-100 flex items-center gap-1.5 text-[13px] flex-wrap shrink-0">
        <button type="button" className="text-gray-500 hover:text-gray-800 transition-colors">
          Aplikace
        </button>
        <ChevronRight className="size-3.5 text-gray-300" />
        <span className="text-gray-500">ONEPOST - datové schránky</span>
        <ChevronRight className="size-3.5 text-gray-300" />
        <span className="text-gray-800 font-medium">ONEPOST API klíč</span>
      </div>

      {/* Header */}
      <div className="relative px-5 sm:px-6 py-5 border-b border-gray-100 overflow-hidden shrink-0">
        <div
          className="absolute inset-0 pointer-events-none"
          style={{
            background:
              'radial-gradient(620px 200px at left top, rgba(52, 211, 153, 0.06), transparent 72%)',
          }}
        />
        <div className="relative flex items-center justify-between gap-4">
          <div className="flex items-center gap-3 flex-1 min-w-0">
            <div
              className="w-11 h-11 rounded-xl flex items-center justify-center shrink-0"
              style={{ backgroundColor: 'rgba(52, 211, 153, 0.15)' }}
            >
              <OnepostSymbol className="h-6 w-auto" />
            </div>
            <div className="min-w-0">
              <h1 className="text-gray-800 text-[18px] sm:text-[20px] font-semibold leading-tight truncate">
                ONEPOST API klíč
              </h1>
              <p className="text-gray-500 text-[13.5px] mt-0.5 leading-relaxed truncate">
                Připojte AI agenta k ONEPOSTu pomocí API klíče z uživatelského profilu.
              </p>
            </div>
          </div>
          <div className="shrink-0">
            {configured ? (
              <span
                className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[12.5px] font-semibold"
                style={{ backgroundColor: 'rgba(124, 179, 5, 0.1)', color: '#5C8A02' }}
              >
                <CheckCircle2 className="size-3.5" />
                Nastaveno
              </span>
            ) : (
              <span className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-full text-[12.5px] font-semibold bg-amber-50 text-amber-600">
                <AlertCircle className="size-3.5" />
                Nenastaveno
              </span>
            )}
          </div>
        </div>
      </div>

      {/* Body */}
      <div className="p-5 sm:p-6">
        {configured ? (
          /* ---- Nastaveno: přehled + akce + aktualizace ---- */
          <>
            <div className="rounded-xl border border-gray-200 divide-y divide-gray-100">
              <div className="p-5 flex items-start justify-between gap-4">
                <dl className="grid grid-cols-1 sm:grid-cols-[160px_1fr] gap-x-4 gap-y-2.5 min-w-0">
                  <dt className="text-gray-500 text-[13px] font-medium">API URL</dt>
                  <dd className="text-gray-800 text-[12.5px] font-mono break-all">
                    {settings.apiUrl || '-'}
                  </dd>
                  <dt className="text-gray-500 text-[13px] font-medium">Uložený klíč</dt>
                  <dd className="text-gray-800 text-[12.5px] font-mono break-all">
                    {settings.maskedKey || '-'}
                  </dd>
                  <dt className="text-gray-500 text-[13px] font-medium">Zdroj</dt>
                  <dd className="text-gray-800 text-[12.5px] font-mono break-all">
                    {settings.settingsFile || '-'}
                  </dd>
                </dl>
                <div className="flex items-center gap-1.5 shrink-0">
                  <button
                    type="button"
                    disabled={isBusy}
                    onClick={testKey}
                    className="inline-flex items-center gap-2 px-3.5 py-2 rounded-xl border border-gray-200 text-gray-700 text-[13px] font-medium hover:bg-gray-50 hover:border-gray-300 transition-all disabled:opacity-40"
                  >
                    {busyAction === 'test' ? (
                      <Loader2 className="size-4 animate-spin" />
                    ) : (
                      <RefreshCw className="size-4" />
                    )}
                    Otestovat
                  </button>
                  <button
                    type="button"
                    disabled={isBusy}
                    onClick={deleteKey}
                    aria-label="Smazat klíč"
                    className="w-9 h-9 rounded-xl text-red-500 hover:bg-red-50 flex items-center justify-center transition-all shrink-0 disabled:opacity-40"
                  >
                    {busyAction === 'delete' ? (
                      <Loader2 className="size-4 animate-spin" />
                    ) : (
                      <Trash2 className="size-4" />
                    )}
                  </button>
                </div>
              </div>

              <div className="p-5">
                <label
                  htmlFor="onepost-api-key"
                  className="text-gray-700 text-[12.5px] font-semibold mb-1.5 block"
                >
                  Aktualizovat klíč
                </label>
                <div className="flex items-center gap-2 flex-wrap">
                  <input
                    id="onepost-api-key"
                    type="password"
                    autoComplete="off"
                    spellCheck={false}
                    className="w-full rounded-xl border border-gray-200 bg-white px-3.5 py-2.5 text-[13px] text-gray-800 placeholder-gray-400 outline-none focus:border-gray-300 focus:ring-2 focus:ring-gray-200 transition-all flex-1 min-w-[220px] font-mono"
                    placeholder="Vložte nový klíč z ONEPOSTu"
                    value={apiKey}
                    onChange={(event) => setApiKey(event.target.value)}
                    disabled={isBusy}
                  />
                  <button
                    type="button"
                    disabled={isBusy || apiKeyValue.length === 0}
                    onClick={saveKey}
                    className={primaryButton}
                    style={{ backgroundColor: PRIMARY }}
                    {...primaryHandlers(!(isBusy || apiKeyValue.length === 0))}
                  >
                    {busyAction === 'save' ? (
                      <Loader2 className="size-4 animate-spin" />
                    ) : (
                      <Check className="size-4" />
                    )}
                    Uložit
                  </button>
                </div>
              </div>
            </div>
            {statusBanner}
          </>
        ) : (
          /* ---- Nenastaveno: postup ve dvou krocích ---- */
          <div className="max-w-[680px]">
            {/* Krok 1 */}
            <div className="flex gap-4">
              <div className="flex flex-col items-center">
                <div
                  className="w-7 h-7 rounded-full flex items-center justify-center text-[12px] font-bold shrink-0"
                  style={{ backgroundColor: 'rgba(124, 179, 5, 0.1)', color: '#5C8A02' }}
                >
                  1
                </div>
                <div className="w-px flex-1 my-1.5 bg-gray-200" />
              </div>
              <div className="flex-1 min-w-0 pb-7">
                <h3 className="text-gray-900 text-[14px] font-semibold leading-tight">
                  Vygenerujte klíč v ONEPOSTu
                </h3>
                <p className="text-gray-500 text-[12.5px] mt-1 leading-relaxed">
                  Otevřete profil v ONEPOSTu, vyberte typ přístupu a vygenerujte klíč.
                </p>
                <div className="mt-3.5">
                  <a
                    href={onepostUrl || undefined}
                    target="_blank"
                    rel="noreferrer"
                    className={primaryButton}
                    style={{ backgroundColor: PRIMARY }}
                    {...primaryHandlers(true)}
                  >
                    Otevřít ONEPOST
                    <ExternalLink className="size-4" />
                  </a>
                  <div className="mt-4">
                    <div className="text-gray-400 text-[11.5px] font-medium mb-2">
                      Typy přístupu, které v ONEPOSTu nabízí:
                    </div>
                    <div className="grid grid-cols-1 gap-2.5">
                      <div className="flex items-start gap-2.5 p-3 rounded-lg border border-gray-200 bg-white">
                        <div className="w-7 h-7 rounded-md bg-gray-100 flex items-center justify-center shrink-0">
                          <Eye className="size-4 text-gray-500" />
                        </div>
                        <div className="min-w-0">
                          <div className="text-gray-800 text-[12.5px] font-semibold leading-tight">
                            Pouze čtení
                          </div>
                          <p className="text-gray-500 text-[11.5px] leading-snug mt-0.5">
                            Načítání schránek, zpráv, příloh a vyhledávání.
                          </p>
                        </div>
                      </div>
                      <div className="flex items-start gap-2.5 p-3 rounded-lg border border-gray-200 bg-white">
                        <div className="w-7 h-7 rounded-md bg-gray-100 flex items-center justify-center shrink-0">
                          <SquarePen className="size-4 text-gray-500" />
                        </div>
                        <div className="min-w-0">
                          <div className="text-gray-800 text-[12.5px] font-semibold leading-tight">
                            Čtení a akce
                          </div>
                          <p className="text-gray-500 text-[11.5px] leading-snug mt-0.5">
                            Čtení i akce, které agent provádí vaším jménem.
                          </p>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Krok 2 */}
            <div className="flex gap-4">
              <div className="flex flex-col items-center">
                <div
                  className="w-7 h-7 rounded-full flex items-center justify-center text-[12px] font-bold shrink-0"
                  style={{ backgroundColor: 'rgba(124, 179, 5, 0.1)', color: '#5C8A02' }}
                >
                  2
                </div>
              </div>
              <div className="flex-1 min-w-0">
                <h3 className="text-gray-900 text-[14px] font-semibold leading-tight">
                  Vložte a uložte klíč
                </h3>
                <p className="text-gray-500 text-[12.5px] mt-1 leading-relaxed">
                  Vložte zkopírovaný klíč - agent ho ihned ověří.
                </p>
                <div className="mt-3.5">
                  <div className="flex items-center gap-2 flex-wrap">
                    <input
                      type="password"
                      autoComplete="off"
                      spellCheck={false}
                      className="w-full rounded-xl border border-gray-200 bg-white px-3.5 py-2.5 text-[13px] text-gray-800 placeholder-gray-400 outline-none focus:border-gray-300 focus:ring-2 focus:ring-gray-200 transition-all flex-1 min-w-[220px] font-mono"
                      placeholder="Vložte klíč z ONEPOSTu"
                      value={apiKey}
                      onChange={(event) => setApiKey(event.target.value)}
                      disabled={isBusy}
                    />
                    <button
                      type="button"
                      disabled={isBusy || apiKeyValue.length === 0}
                      onClick={saveKey}
                      className={primaryButton}
                      style={{ backgroundColor: PRIMARY }}
                      {...primaryHandlers(!(isBusy || apiKeyValue.length === 0))}
                    >
                      {busyAction === 'save' ? (
                        <Loader2 className="size-4 animate-spin" />
                      ) : (
                        <CheckCircle2 className="size-4" />
                      )}
                      Uložit a ověřit
                    </button>
                  </div>
                </div>
              </div>
            </div>

            {statusBanner}
          </div>
        )}
      </div>
    </div>
  )
}
