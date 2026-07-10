import { describe, expect, it } from 'vitest'
import { render, screen } from '@testing-library/react'
import { SettingsDialogShell } from './settings-dialog-shell'

describe('SettingsDialogShell', () => {
  it('renders title, description and children', () => {
    render(
      <SettingsDialogShell open onOpenChange={() => {}} title="Nastavení" description="Popis">
        <div>obsah</div>
      </SettingsDialogShell>,
    )
    expect(screen.getByText('Nastavení')).toBeInTheDocument()
    expect(screen.getByText('Popis')).toBeInTheDocument()
    expect(screen.getByText('obsah')).toBeInTheDocument()
  })

  it('renders no Save/Cancel footer by default (instant-save dialogs have none)', () => {
    render(
      <SettingsDialogShell open onOpenChange={() => {}} title="X">
        <div />
      </SettingsDialogShell>,
    )
    expect(screen.queryByRole('button', { name: 'Uložit' })).toBeNull()
    expect(screen.queryByRole('button', { name: 'Zrušit' })).toBeNull()
  })

  it('renders a footer only when one is provided', () => {
    render(
      <SettingsDialogShell open onOpenChange={() => {}} title="X" footer={<button>Smazat</button>}>
        <div />
      </SettingsDialogShell>,
    )
    expect(screen.getByRole('button', { name: 'Smazat' })).toBeInTheDocument()
  })
})
