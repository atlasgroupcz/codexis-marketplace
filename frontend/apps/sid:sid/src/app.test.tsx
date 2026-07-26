// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'

import { App } from './app'
import { Providers } from '@/components/providers'
import i18n from '@/lib/i18n'

// next-themes reads the OS colour-scheme preference; jsdom has no
// matchMedia, so ThemeProvider throws on render without this.
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
  fetchStatus: vi.fn(),
  saveApiKey: vi.fn(),
  deleteApiKey: vi.fn(),
  saveNotifySettings: vi.fn(),
}))

vi.mock('@/lib/api', () => mocks)

const NOT_CONFIGURED = {
  ok: true as const,
  configured: false,
  baseUrl: 'https://sid.test',
  userEmail: 'ja@firma.cz',
  notify: { email: false, inApp: true, recipients: [] },
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
  mocks.fetchStatus.mockResolvedValue(NOT_CONFIGURED)
  mocks.saveApiKey.mockResolvedValue({ ok: true, verified: true, start: 'sid_abc' })
  mocks.saveNotifySettings.mockResolvedValue({ ok: true, notify: NOT_CONFIGURED.notify })
})

describe('App', () => {
  it('shows the state the CGI reported', async () => {
    mocks.fetchStatus.mockResolvedValue({
      ...NOT_CONFIGURED,
      configured: true,
      start: 'sid_abc',
      notify: { email: true, inApp: true, recipients: ['sef@firma.cz'] },
    })
    renderApp()
    await waitFor(() => expect(mocks.fetchStatus).toHaveBeenCalled())

    // The masked prefix, never the whole key.
    expect(await screen.findByText(/sid_abc…/)).toBeTruthy()
    expect(screen.getByText('sef@firma.cz')).toBeTruthy()
  })

  it('keeps unsaved notification edits when the API key is saved', async () => {
    // Saving the key refetches the status. Before the dirty guard, that
    // refetch overwrote whatever the user had just toggled, silently.
    const user = userEvent.setup()
    renderApp()
    await waitFor(() => expect(mocks.fetchStatus).toHaveBeenCalled())

    const emailToggle = await screen.findByRole('switch', { name: i18n.t('notify.emailLabel') })
    await user.click(emailToggle)

    await user.type(screen.getByPlaceholderText(i18n.t('settings.placeholder')), 'sid_novy_klic')
    await user.click(screen.getByRole('button', { name: i18n.t('connection.save') }))
    await waitFor(() => expect(mocks.saveApiKey).toHaveBeenCalled())

    await user.click(screen.getByRole('button', { name: i18n.t('notify.save') }))
    await waitFor(() => expect(mocks.saveNotifySettings).toHaveBeenCalled())

    expect(mocks.saveNotifySettings).toHaveBeenCalledWith(
      expect.objectContaining({ email: true }),
    )
  })

  it('reports a failed status load instead of rendering an empty panel', async () => {
    mocks.fetchStatus.mockRejectedValue(new Error('down'))
    renderApp()
    expect(await screen.findByText(i18n.t('app.loadFailed'))).toBeTruthy()
  })
})
