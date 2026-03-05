/**
 * Utility for building backend URLs.
 *
 * Web app: no base URL set, uses relative paths (same-origin via proxy/deploy).
 * Desktop app: calls setBackendBaseUrl() once at startup, all paths become absolute.
 */

let _backendBaseUrl: string | undefined

export const setBackendBaseUrl = (url: string | undefined): void => {
  _backendBaseUrl = url?.replace(/\/+$/, '')
}

export const getBackendBaseUrl = (): string | undefined => _backendBaseUrl

export const backendUrl = (path: string): string => {
  const normalizedPath = path.startsWith('/') ? path : `/${path}`
  if (_backendBaseUrl) {
    return `${_backendBaseUrl}${normalizedPath}`
  }
  return normalizedPath
}

/**
 * Convert HTTP path to WebSocket URL.
 * Uses configured backend base URL if set, otherwise derives from window.location.
 */
export const toWsUrl = (path: string): string => {
  if (_backendBaseUrl) {
    const url = new URL(_backendBaseUrl)
    const protocol = url.protocol === 'https:' ? 'wss:' : 'ws:'
    return `${protocol}//${url.host}${path}`
  }
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${protocol}//${window.location.host}${path}`
}
