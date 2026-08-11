import type { RenderOptions } from '@testing-library/react'
import { render } from '@testing-library/react'
import { ThemeProvider } from 'next-themes'
import { NuqsTestingAdapter } from 'nuqs/adapters/testing'
import type { ReactElement } from 'react'
import { I18nextProvider } from 'react-i18next'
import testI18n from '@/test/i18n-test'

interface AppRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  searchParams?: Record<string, string>
}

export function renderApp(ui: ReactElement, options: AppRenderOptions = {}) {
  const { searchParams = {}, ...renderOptions } = options
  const languageFromUrl = Object.hasOwn(searchParams, 'lang') ? searchParams.lang : undefined
  const languageFromStorage = window.localStorage.getItem('i18nextLng')
  const language = languageFromUrl ?? languageFromStorage ?? 'en'
  void testI18n.changeLanguage(language)

  return render(ui, {
    wrapper: ({ children }) => (
      <I18nextProvider i18n={testI18n}>
        <NuqsTestingAdapter searchParams={searchParams}>
          <ThemeProvider attribute="class" defaultTheme="light" enableColorScheme>
            {children}
          </ThemeProvider>
        </NuqsTestingAdapter>
      </I18nextProvider>
    ),
    ...renderOptions,
  })
}
