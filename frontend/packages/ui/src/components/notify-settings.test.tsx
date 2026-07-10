import { describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { NotifySettings } from './notify-settings'
import type { NotifyLabels } from './notify-settings'

const labels: NotifyLabels = {
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
  cancel: 'Zavřít',
}
const me = 'jan.novak@firma.cz'

function setup(
  values: { email: boolean; inApp: boolean; recipients: Array<string> } = {
    email: true,
    inApp: true,
    recipients: [],
  },
  onSave = vi.fn(),
) {
  const utils = render(
    <NotifySettings
      open
      onOpenChange={() => {}}
      values={values}
      userEmail={me}
      labels={labels}
      onSave={onSave}
    />,
  )
  return { onSave, ...utils }
}

// The "Kopie na" field is a token input: click "+ příjemce" to reveal it, type, Enter commits.
function addRecipient(email: string) {
  fireEvent.click(screen.getByRole('button', { name: /příjemce/ }))
  const input = screen.getByPlaceholderText('jmeno@firma.cz')
  fireEvent.change(input, { target: { value: email } })
  fireEvent.keyDown(input, { key: 'Enter' })
}

describe('NotifySettings (variant 2)', () => {
  it('shows the signed-in address as a pill when email is on', () => {
    setup()
    expect(screen.getByText(me)).toBeInTheDocument()
    expect(screen.getByTitle('Vaše adresa')).toBeInTheDocument()
  })

  it('collapses (not removes) the recipients section when email is off', () => {
    const { baseElement } = setup({ email: false, inApp: true, recipients: [] })
    const region = baseElement.querySelector('.grid-rows-\\[0fr\\]')
    expect(region).not.toBeNull()
    expect(region?.getAttribute('aria-hidden')).toBe('true')
  })

  it('adds a valid recipient to "Kopie na"', () => {
    setup()
    addRecipient('kolega@firma.cz')
    expect(screen.getByText('kolega@firma.cz')).toBeInTheDocument()
  })

  it('rejects an invalid address', () => {
    setup()
    fireEvent.click(screen.getByRole('button', { name: /příjemce/ }))
    const input = screen.getByPlaceholderText('jmeno@firma.cz')
    fireEvent.change(input, { target: { value: 'nope' } })
    fireEvent.keyDown(input, { key: 'Enter' })
    expect(screen.getByText('Neplatná e-mailová adresa')).toBeInTheDocument()
  })

  it("skips the user's own address (redundant, case-insensitive)", () => {
    setup()
    addRecipient(me.toUpperCase())
    // me shown once (the "Vaše adresa" row), never added as a chip
    expect(screen.getAllByText(me).length).toBe(1)
  })

  it('removes a recipient when its remove button is clicked', () => {
    setup()
    addRecipient('kolega@firma.cz')
    expect(screen.getByText('kolega@firma.cz')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Odebrat: kolega@firma.cz' }))
    expect(screen.queryByText('kolega@firma.cz')).toBeNull()
  })

  it('dedupes recipients added case-insensitively', () => {
    setup()
    addRecipient('kolega@firma.cz')
    addRecipient('KOLEGA@firma.cz')
    expect(screen.getAllByText(/kolega@firma.cz/i).length).toBe(1)
  })

  it('persists a new recipient instantly (no footer Save button)', () => {
    const { onSave } = setup({ email: true, inApp: true, recipients: [] })
    addRecipient('kolega@firma.cz')
    expect(onSave).toHaveBeenLastCalledWith({
      email: true,
      inApp: true,
      recipients: ['kolega@firma.cz'],
    })
  })

  it('persists a toggle instantly', () => {
    const { onSave } = setup({ email: true, inApp: true, recipients: [] })
    fireEvent.click(screen.getByLabelText('Zvoneček v aplikaci'))
    expect(onSave).toHaveBeenLastCalledWith({ email: true, inApp: false, recipients: [] })
  })
})
