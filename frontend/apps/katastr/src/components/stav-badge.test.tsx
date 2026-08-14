// @vitest-environment jsdom

import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { StavBadge } from './stav-badge'
import { CLOSED_CLASSES, NEGATIVE_CLASSES } from './stav-colors'

// The full ČÚZK sentence is the tooltip, so it is how a rendered badge is found
// no matter what short name it ended up wearing.
const badgeFor = (stav: string) => {
  const { unmount } = render(<StavBadge stav={stav} />)
  const el = screen.getByTitle(stav)
  const found = { className: el.className, text: el.textContent }
  unmount()
  return found
}

describe('StavBadge - colour', () => {
  // The timeline chip reads these same stavs as endings through `isTerminal`.
  // Before they shared the test, "Řízení bylo uzavřeno jako mylné" was closed in
  // the timeline and an unrecognised state in the badge.
  it.each(['Ukončeno', 'Ukončení řízení', 'Řízení bylo uzavřeno jako mylné'])(
    'colours %s as a closed proceeding',
    (stav) => {
      expect(badgeFor(stav).className).toContain(CLOSED_CLASSES)
    },
  )

  it('keeps red for an outcome that went badly', () => {
    expect(badgeFor('Zamítnuto').className).toContain(NEGATIVE_CLASSES)
  })

  it('leaves a running proceeding out of both', () => {
    const { className } = badgeFor('Probíhá zpracování')
    expect(className).not.toContain(CLOSED_CLASSES)
    expect(className).not.toContain(NEGATIVE_CLASSES)
  })
})

describe('StavBadge - label', () => {
  // Only a few characters fit, so a recognised state wears our short name for
  // it. Test locale is English; these were hardcoded Czech before, which an
  // English reader got beside otherwise translated text.
  it.each([
    ['Vyrozumění o provedení vkladu', 'Notified'],
    ['Provedení vkladu', 'Entry completed'],
    ['Povolení vkladu', 'Entry approved'],
    ['Probíhá zpracování', 'In progress'],
    ['Ukončení řízení', 'Closed'],
  ])('shortens %s to %s in the reader language', (stav, expected) => {
    expect(badgeFor(stav).text).toBe(expected)
  })

  // Colour said "this ended" while the label fell through to the raw sentence,
  // so this state alone wore a 31-character pill.
  it('gives the mylné ending its own name rather than the whole sentence', () => {
    expect(badgeFor('Řízení bylo uzavřeno jako mylné').text).toBe('Filed in error')
  })

  // "Zamítnuto", "Zastaveno" and "Zpětvzetí" are three different outcomes and
  // each is short already - one shared word would say less than ČÚZK did.
  it.each(['Zamítnuto', 'Zastaveno'])('keeps ČÚZK’s own word for %s', (stav) => {
    expect(badgeFor(stav).text).toBe(stav)
  })

  it('shows an unrecognised state in full rather than hiding it', () => {
    const stav = 'Něco, co ČÚZK vymyslel dnes ráno'
    expect(badgeFor(stav).text).toBe(stav)
  })
})
