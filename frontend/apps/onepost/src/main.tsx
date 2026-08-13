import { ErrorBoundary } from '@workspace/ui/components/error-boundary'
import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { App } from '@/app'
import '@/styles.css'

const root = document.getElementById('app')
if (!root) {
  throw new Error('Root element #app not found')
}

createRoot(root).render(
  <StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </StrictMode>,
)
