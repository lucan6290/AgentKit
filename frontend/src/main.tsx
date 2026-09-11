import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import './styles/index.css'
import './i18n'
import App from './App.tsx'
import { ErrorBoundary } from './app/ErrorBoundary'
import { installGlobalErrorLogging } from './lib/logger'

installGlobalErrorLogging()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </StrictMode>,
)
