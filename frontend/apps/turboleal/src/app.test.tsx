// @vitest-environment jsdom

import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { Providers } from '@/components/providers'
import i18n from '@/lib/i18n'
import { App } from './app'

vi.stubGlobal(
  'matchMedia',
  vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
)

const mocks = vi.hoisted(() => ({
  fetchSettings: vi.fn(),
  saveApiKey: vi.fn(),
  deleteApiKey: vi.fn(),
}))

vi.mock('@/lib/api', () => mocks)

const NOT_CONFIGURED = {
  configured: false,
  maskedKey: '',
  keySource: '',
  settingsFile: '/home/codexis/.cdx/apps/turboleal/.env',
  apiUrl: 'https://turboleal.test',
  generateUrl: 'https://turboleal.test/api-keys',
}

const CONFIGURED = {
  ...NOT_CONFIGURED,
  configured: true,
  maskedKey: 'api-key-AbCd…',
  keySource: NOT_CONFIGURED.settingsFile,
}

function renderApp() {
  return render(
    <Providers>
      <App />
    </Providers>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  mocks.fetchSettings.mockResolvedValue(NOT_CONFIGURED)
  mocks.saveApiKey.mockResolvedValue({ ok: true })
  mocks.deleteApiKey.mockResolvedValue({ ok: true })
})

describe('App', () => {
  it('links to the API keys page of Turboleal', async () => {
    renderApp()

    const link = await screen.findByRole('link', { name: new RegExp(i18n.t('settings.getKey')) })
    expect(link.getAttribute('href')).toBe('https://turboleal.test/api-keys')
  })

  it('shows only the masked key once one is stored', async () => {
    mocks.fetchSettings.mockResolvedValue(CONFIGURED)
    renderApp()

    expect(await screen.findByText(/api-key-AbCd…/)).toBeTruthy()
    expect(screen.getByText(i18n.t('app.leadConnected'))).toBeTruthy()
    expect(screen.queryByText(i18n.t('connection.connected'))).toBeNull()
  })

  it('removes the stored key from its row after a confirming second click', async () => {
    const user = userEvent.setup()
    mocks.fetchSettings.mockResolvedValue(CONFIGURED)
    renderApp()
    const remove = await screen.findByRole('button', { name: i18n.t('connection.removeKey') })
    mocks.fetchSettings.mockResolvedValue(NOT_CONFIGURED)

    await user.click(remove)
    expect(mocks.deleteApiKey).not.toHaveBeenCalled()
    await user.click(screen.getByRole('button', { name: i18n.t('connection.confirmRemove') }))

    await waitFor(() => expect(mocks.deleteApiKey).toHaveBeenCalledOnce())
    expect(await screen.findByPlaceholderText(i18n.t('settings.placeholder'))).toBeTruthy()
  })

  it('offers no separate remove button below a stored key', async () => {
    mocks.fetchSettings.mockResolvedValue(CONFIGURED)
    renderApp()

    await screen.findByText(/api-key-AbCd…/)
    expect(screen.queryByRole('button', { name: i18n.t('connection.remove') })).toBeNull()
    expect(screen.queryByRole('button', { name: i18n.t('connection.save') })).toBeNull()
  })

  it('shows no connection badge without a stored key', async () => {
    renderApp()

    expect(await screen.findByText(i18n.t('app.leadIdle'))).toBeTruthy()
    expect(screen.queryByText(i18n.t('connection.notConnected'))).toBeNull()
  })

  it('saves the pasted key and reloads the state', async () => {
    const user = userEvent.setup()
    renderApp()
    const input = await screen.findByPlaceholderText(i18n.t('settings.placeholder'))
    mocks.fetchSettings.mockResolvedValue(CONFIGURED)

    await user.type(input, 'api-key-AbCdEfGhIjKlMnOp')
    await user.click(screen.getByRole('button', { name: i18n.t('connection.save') }))

    await waitFor(() => expect(mocks.saveApiKey).toHaveBeenCalledWith('api-key-AbCdEfGhIjKlMnOp'))
    expect(await screen.findByText(/api-key-AbCd…/)).toBeTruthy()
  })

  it('reports a key Turboleal rejects next to the field', async () => {
    const user = userEvent.setup()
    mocks.saveApiKey.mockRejectedValue(new Error('Turboleal rejected the API key (HTTP 401)'))
    renderApp()
    const input = await screen.findByPlaceholderText(i18n.t('settings.placeholder'))

    await user.type(input, 'api-key-wrong')
    await user.click(screen.getByRole('button', { name: i18n.t('connection.save') }))

    expect(await screen.findByText(/rejected the API key/)).toBeTruthy()
  })
})
