import { describe, expect, it, vi } from 'vitest'
import { fireEvent, render, screen } from '@testing-library/react'
import { ConnectionSettings } from './connection-settings'

const labels = {
  title: 'Připojení',
  currentKey: 'Nastavený klíč',
  deleteKey: 'Odstranit',
  save: 'Uložit',
  cancel: 'Zavřít',
  deleting: 'Odstraňuji…',
}
const getKey = { href: 'https://example.test', label: 'Získat klíč' }

describe('ConnectionSettings', () => {
  it('renders input fields when not configured and saves values', () => {
    const onSave = vi.fn()
    render(
      <ConnectionSettings
        open
        onOpenChange={() => {}}
        configured={false}
        maskedKey=""
        fields={[{ name: 'apiKey', label: 'API klíč', placeholder: 'klíč', mono: true }]}
        getKey={getKey}
        labels={labels}
        onSave={onSave}
        onDelete={() => {}}
      />,
    )
    fireEvent.change(screen.getByLabelText('API klíč'), { target: { value: 'ABC123' } })
    fireEvent.click(screen.getByRole('button', { name: 'Uložit' }))
    expect(onSave).toHaveBeenCalledWith({ apiKey: 'ABC123' })
  })

  it('renders masked key + delete when configured', () => {
    const onDelete = vi.fn()
    render(
      <ConnectionSettings
        open
        onOpenChange={() => {}}
        configured
        maskedKey="ABCD****WXYZ"
        fields={[{ name: 'apiKey', label: 'API klíč' }]}
        getKey={getKey}
        labels={labels}
        onSave={() => {}}
        onDelete={onDelete}
      />,
    )
    expect(screen.getByText('ABCD****WXYZ')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Odstranit' }))
    expect(onDelete).toHaveBeenCalledOnce()
  })
})
