import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { Providers } from '@/components/providers'
import { App } from '@/app'
import '@/lib/i18n'
import '@/styles.css'

const root = document.getElementById('app')
if (!root) {
  throw new Error('Root element #app not found')
}

createRoot(root).render(
  <StrictMode>
    <Providers>
      <App />
    </Providers>
  </StrictMode>,
)
