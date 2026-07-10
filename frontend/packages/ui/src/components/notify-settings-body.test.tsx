import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { NotifySettingsBody } from './notify-settings-body'
import type { NotifyBodyLabels, NotifyValues } from './notify-settings-body'

const labels: NotifyBodyLabels = {
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
}
const me = 'jan.novak@firma.cz'

function Controlled({
  initial,
  onChange,
  userEmail = me,
}: {
  initial: NotifyValues
  onChange?: (value: NotifyValues) => void
  userEmail?: string
}) {
  const [value, setValue] = useState(initial)
  return (
    <NotifySettingsBody
      value={value}
      onChange={(v: NotifyValues) => {
        setValue(v)
        onChange?.(v)
      }}
      userEmail={userEmail}
      labels={labels}
    />
  )
}

function setup(
  initial: NotifyValues = { email: true, inApp: true, recipients: [] },
  onChange = vi.fn(),
  userEmail = me,
) {
  const utils = render(<Controlled initial={initial} onChange={onChange} userEmail={userEmail} />)
  return { onChange, ...utils }
}

describe('NotifySettingsBody (controlled)', () => {
  it('shows the signed-in address as a pill when email is on', () => {
    setup()
    // the address is a pill; its label ("Vaše adresa") is a tooltip, not visible text
    expect(screen.getByText(me)).toBeInTheDocument()
    expect(screen.getByTitle('Vaše adresa')).toBeInTheDocument()
  })

  it('hides the signed-in pill when userEmail is empty but keeps the add control', () => {
    setup({ email: true, inApp: true, recipients: [] }, vi.fn(), '')
    expect(screen.queryByTitle('Vaše adresa')).toBeNull()
    expect(screen.getByRole('button', { name: /příjemce/ })).toBeInTheDocument()
  })

  it('collapses (not removes) the recipients section when email is off', () => {
    // it stays in the DOM but collapsed (aria-hidden + grid 0fr) so it animates open
    // instead of the panel jumping when e-mail is toggled on
    const { container } = setup({ email: false, inApp: true, recipients: [] })
    const region = container.querySelector('.grid-rows-\\[0fr\\]')
    expect(region).not.toBeNull()
    expect(region?.getAttribute('aria-hidden')).toBe('true')
  })

  it('reveals the recipient input only after clicking add', () => {
    setup()
    expect(screen.queryByPlaceholderText('jmeno@firma.cz')).toBeNull()
    fireEvent.click(screen.getByRole('button', { name: /příjemce/ }))
    expect(screen.getByPlaceholderText('jmeno@firma.cz')).toBeInTheDocument()
  })

  it('adds a valid recipient via onChange', () => {
    const { onChange } = setup()
    fireEvent.click(screen.getByRole('button', { name: /příjemce/ }))
    fireEvent.change(screen.getByPlaceholderText('jmeno@firma.cz'), {
      target: { value: 'kolega@firma.cz' },
    })
    fireEvent.keyDown(screen.getByPlaceholderText('jmeno@firma.cz'), { key: 'Enter' })
    expect(screen.getByText('kolega@firma.cz')).toBeInTheDocument()
    expect(onChange).toHaveBeenLastCalledWith({
      email: true,
      inApp: true,
      recipients: ['kolega@firma.cz'],
    })
  })

  it('rejects an invalid address', () => {
    setup()
    fireEvent.click(screen.getByRole('button', { name: /příjemce/ }))
    fireEvent.change(screen.getByPlaceholderText('jmeno@firma.cz'), { target: { value: 'nope' } })
    fireEvent.keyDown(screen.getByPlaceholderText('jmeno@firma.cz'), { key: 'Enter' })
    expect(screen.getByText('Neplatná e-mailová adresa')).toBeInTheDocument()
  })

  it("warns and skips the user's own address (case-insensitive)", () => {
    const { onChange } = setup()
    fireEvent.click(screen.getByRole('button', { name: /příjemce/ }))
    fireEvent.change(screen.getByPlaceholderText('jmeno@firma.cz'), {
      target: { value: me.toUpperCase() },
    })
    fireEvent.keyDown(screen.getByPlaceholderText('jmeno@firma.cz'), { key: 'Enter' })
    expect(onChange).not.toHaveBeenCalled()
    expect(screen.getByText('Tato adresa je vaše')).toBeInTheDocument()
  })

  it('warns and skips a duplicate recipient (case-insensitive)', () => {
    const { onChange } = setup({ email: true, inApp: true, recipients: ['kolega@firma.cz'] })
    fireEvent.click(screen.getByRole('button', { name: /příjemce/ }))
    fireEvent.change(screen.getByPlaceholderText('jmeno@firma.cz'), {
      target: { value: 'KOLEGA@firma.cz' },
    })
    fireEvent.keyDown(screen.getByPlaceholderText('jmeno@firma.cz'), { key: 'Enter' })
    expect(onChange).not.toHaveBeenCalled()
    expect(screen.getByText('Tato adresa už je v seznamu')).toBeInTheDocument()
  })

  it('removes a recipient when its remove button is clicked', () => {
    setup({ email: true, inApp: true, recipients: ['kolega@firma.cz'] })
    expect(screen.getByText('kolega@firma.cz')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Odebrat: kolega@firma.cz' }))
    expect(screen.queryByText('kolega@firma.cz')).toBeNull()
  })

  it('toggles the in-app bell via onChange', () => {
    const { onChange } = setup()
    fireEvent.click(screen.getByLabelText('Zvoneček v aplikaci'))
    expect(onChange).toHaveBeenLastCalledWith({ email: true, inApp: false, recipients: [] })
  })
})
