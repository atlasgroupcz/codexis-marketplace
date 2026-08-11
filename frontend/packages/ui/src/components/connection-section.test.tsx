import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { ConnectionSectionLabels } from './connection-section'
import { ConnectionSection } from './connection-section'

const labels: ConnectionSectionLabels = {
  connected: 'Připojeno',
  notConnected: 'Nepřipojeno',
  remove: 'Odebrat',
  confirmRemove: 'Opravdu odebrat?',
  save: 'Uložit',
  saving: 'Ověřuji…',
}
const getKey = { href: 'https://example.test', label: 'Získat klíč' }
const fields = [{ name: 'apiKey', label: 'API klíč', placeholder: 'Vložte klíč', mono: true }]

function setup(props: Partial<React.ComponentProps<typeof ConnectionSection>> = {}) {
  return render(
    <ConnectionSection
      configured={false}
      maskedKey=""
      fields={fields}
      getKey={getKey}
      labels={labels}
      onSave={() => {}}
      onDelete={() => {}}
      {...props}
    />,
  )
}

describe('ConnectionSection', () => {
  it('not configured: Uložit is disabled until a value is typed, then saves it', async () => {
    const onSave = vi.fn()
    setup({ onSave })
    expect(screen.getByRole('button', { name: 'Uložit' })).toBeDisabled()
    fireEvent.change(screen.getByRole('textbox'), { target: { value: 'ABC123' } })
    const save = screen.getByRole('button', { name: 'Uložit' })
    expect(save).toBeEnabled()
    // Saving ends in `.finally(() => setSaving(false))`, which lands after the
    // click returns. Waiting on the label rather than on a number of promise
    // ticks: waitFor retries until the condition holds, so it still works if
    // the save path ever grows a step that is not a microtask.
    fireEvent.click(save)
    await waitFor(() => expect(save).toHaveTextContent(labels.save))
    expect(onSave).toHaveBeenCalledWith({ apiKey: 'ABC123' })
  })

  it('configured: shows the masked key read-only (no input) with a 2-step Odebrat', () => {
    const onDelete = vi.fn()
    setup({ configured: true, maskedKey: 'ABCD****WXYZ', onDelete })
    expect(screen.getByText('ABCD****WXYZ')).toBeInTheDocument()
    expect(screen.queryByRole('textbox')).toBeNull()
    // first click arms the confirm, does not delete
    fireEvent.click(screen.getByRole('button', { name: 'Odebrat' }))
    expect(onDelete).not.toHaveBeenCalled()
    // second click (now labelled confirm) deletes
    fireEvent.click(screen.getByRole('button', { name: 'Opravdu odebrat?' }))
    expect(onDelete).toHaveBeenCalledOnce()
  })

  it('shows the validation error inline when onSave rejects (key not saved)', async () => {
    const onSave = vi.fn().mockRejectedValue(new Error('API klíč je neplatný'))
    setup({ onSave })
    fireEvent.change(screen.getByRole('textbox'), { target: { value: 'BAD' } })
    fireEvent.click(screen.getByRole('button', { name: 'Uložit' }))
    expect(await screen.findByText('API klíč je neplatný')).toBeInTheDocument()
  })
})

describe('loading', () => {
  // Rendering the real section first shows an empty field, then swaps it for the
  // stored key a frame later - and a placeholder measured by hand in a consumer
  // drifts the moment this component's padding changes.
  it('shows neither the field nor the actions until the answer arrives', () => {
    setup({ loading: true })
    expect(screen.queryByPlaceholderText('Vložte klíč')).toBeNull()
    expect(screen.queryByText('Uložit')).toBeNull()
    expect(screen.queryByText('Získat klíč')).toBeNull()
  })

  it('marks the placeholder as busy for assistive tech', () => {
    const { container } = setup({ loading: true })
    expect(container.querySelector('[aria-busy="true"]')).not.toBeNull()
  })

  it('renders normally once loading is over', () => {
    setup({ loading: false })
    expect(screen.getByPlaceholderText('Vložte klíč')).toBeTruthy()
  })
})
