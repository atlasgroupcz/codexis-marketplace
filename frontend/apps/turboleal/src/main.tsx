import { ErrorBoundary } from '@workspace/ui/components/error-boundary'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { App } from '@/app'
import { Providers } from '@/components/providers'
import '@/lib/i18n'
import '@/styles.css'

const root = document.getElementById('app')
if (!root) {
  throw new Error('Root element #app not found')
}

createRoot(root).render(
  <StrictMode>
    <ErrorBoundary>
      <Providers>
        <App />
      </Providers>
    </ErrorBoundary>
  </StrictMode>,
)
