/**
 * Whether a credential is stored.
 *
 * `unknown` is its own state, not a synonym for `disconnected`: until the first
 * fetch answers, the app has checked nothing, and saying "not connected" in red
 * is an assertion. Collapsing the two with `?? false` is why several panels used
 * to open red and correct themselves a frame later.
 */
export type ConnState = 'unknown' | 'connected' | 'disconnected'

/** `unknown` while `configured` is still null/undefined. */
export function connStateOf(configured: boolean | null | undefined): ConnState {
  if (configured === null || configured === undefined) return 'unknown'
  return configured ? 'connected' : 'disconnected'
}

export interface ConnectionStatusProps {
  state: ConnState
  labels: { connected: string; notConnected: string }
  /**
   * Colours for the connected state. Defaults to the theme's primary, which is
   * the brand colour in most apps - pass `success` where primary is something
   * else and green is meant (datify's primary is Datify blue).
   */
  tone?: 'primary' | 'success'
}

/**
 * The connected / not-connected badge that sits on a section heading.
 *
 * While the state is unknown it renders nothing but keeps the row's height, so
 * the heading does not jump when the answer lands.
 */
export function ConnectionStatus({ state, labels, tone = 'primary' }: ConnectionStatusProps) {
  if (state === 'unknown') return <span className="h-4" aria-hidden="true" />

  const connected = state === 'connected'
  const text = connected
    ? tone === 'success'
      ? 'text-[var(--success)]'
      : 'text-primary'
    : 'text-destructive'
  const dot = connected
    ? tone === 'success'
      ? 'bg-[var(--success)]'
      : 'bg-primary'
    : 'bg-destructive'

  return (
    <span
      className={`inline-flex shrink-0 items-center gap-1.5 text-[11px] font-semibold tracking-normal normal-case ${text}`}
    >
      <span aria-hidden="true" className={`size-2 rounded-full ${dot}`} />
      {connected ? labels.connected : labels.notConnected}
    </span>
  )
}
