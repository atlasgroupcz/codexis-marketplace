import { describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { SettingsDialog } from './settings-dialog'
import type { NotifyLabels } from './notify-settings'
import type { ConnectionSectionLabels } from './connection-section'

const notifyLabels: NotifyLabels = {
  title: 'Oznámení',
  description: 'Jak vás upozorníme',
  emailLabel: 'E-mail při změně',
  emailDescription: 'Souhrn změn',
  yourAddressLabel: 'Vaše adresa',
  copyToLabel: 'Kopie na',
  addRecipient: 'příjemce',
  removeRecipient: 'Odebrat',
  addPlaceholder: 'jmeno@firma.cz',
  invalidEmail: 'Neplatná e-mailová adresa',
  ownAddress: 'Tato adresa je vaše',
  duplicate: 'Tato adresa už je v seznamu',
  inAppLabel: 'Zvoneček v aplikaci',
  inAppDescription: 'Centrum oznámení',
  save: 'Uložit',
  cancel: 'Zrušit',
}
const connectionLabels: ConnectionSectionLabels = {
  connected: 'Připojeno',
  notConnected: 'Nepřipojeno',
  remove: 'Odebrat',
  confirmRemove: 'Opravdu odebrat?',
  save: 'Uložit',
  saving: 'Ověřuji…',
}
const getKey = { href: 'https://example.test', label: 'Získat klíč' }

describe('SettingsDialog', () => {
  it('renders only the notify section when no connection prop is given', () => {
    render(
      <SettingsDialog
        open
        onOpenChange={() => {}}
        notify={{
          values: { email: true, inApp: true, recipients: [] },
          userEmail: 'jan.novak@firma.cz',
          labels: notifyLabels,
          onSave: vi.fn(),
        }}
      />,
    )
    expect(screen.getByText('Oznámení')).toBeInTheDocument()
    expect(screen.queryByText('Nepřipojeno')).toBeNull()
  })

  it('renders both sections and saves notify values via the footer Save', () => {
    const onSaveNotify = vi.fn()
    const onSaveConnection = vi.fn()
    render(
      <SettingsDialog
        open
        onOpenChange={() => {}}
        notify={{
          values: { email: true, inApp: true, recipients: [] },
          userEmail: 'jan.novak@firma.cz',
          labels: notifyLabels,
          onSave: onSaveNotify,
        }}
        connection={{
          configured: false,
          maskedKey: '',
          fields: [{ name: 'apiKey', label: 'API klíč' }],
          getKey,
          labels: connectionLabels,
          sectionLabel: 'Připojení',
          onSave: onSaveConnection,
          onDelete: vi.fn(),
        }}
      />,
    )
    // both sections present in one dialog
    expect(screen.getByText('Oznámení')).toBeInTheDocument()
    expect(screen.getByText('Nepřipojeno')).toBeInTheDocument()

    // connection self-manages via its own inline control, independent of notify
    fireEvent.change(screen.getByRole('textbox'), { target: { value: 'XYZ' } })
    fireEvent.click(screen.getByRole('button', { name: 'Uložit' }))
    expect(onSaveConnection).toHaveBeenCalledWith({ apiKey: 'XYZ' })
    expect(onSaveNotify).not.toHaveBeenCalled()

    // notify persists instantly (no footer) — toggling the bell saves immediately
    fireEvent.click(screen.getByLabelText('Zvoneček v aplikaci'))
    expect(onSaveNotify).toHaveBeenLastCalledWith({ email: true, inApp: false, recipients: [] })
  })

  it('a missing/unconfigured connection never hides the notify section', () => {
    render(
      <SettingsDialog
        open
        onOpenChange={() => {}}
        notify={{
          values: { email: false, inApp: false, recipients: [] },
          userEmail: 'jan.novak@firma.cz',
          labels: notifyLabels,
          onSave: vi.fn(),
        }}
        connection={{
          configured: false,
          maskedKey: '',
          fields: [{ name: 'apiKey', label: 'API klíč' }],
          getKey,
          labels: connectionLabels,
          sectionLabel: 'Připojení',
          onSave: vi.fn(),
          onDelete: vi.fn(),
        }}
      />,
    )
    expect(screen.getByText('Zvoneček v aplikaci')).toBeInTheDocument()
    expect(screen.getByText('Nepřipojeno')).toBeInTheDocument()
  })

  it('renders optional section labels when provided', () => {
    render(
      <SettingsDialog
        open
        onOpenChange={() => {}}
        notify={{
          values: { email: true, inApp: true, recipients: [] },
          userEmail: 'jan.novak@firma.cz',
          labels: notifyLabels,
          sectionLabel: 'Sekce oznámení',
          onSave: vi.fn(),
        }}
        connection={{
          configured: false,
          maskedKey: '',
          fields: [{ name: 'apiKey', label: 'API klíč' }],
          getKey,
          labels: connectionLabels,
          sectionLabel: 'Sekce připojení',
          onSave: vi.fn(),
          onDelete: vi.fn(),
        }}
      />,
    )
    expect(screen.getByText('Sekce oznámení')).toBeInTheDocument()
    expect(screen.getByText('Sekce připojení')).toBeInTheDocument()
  })
})
