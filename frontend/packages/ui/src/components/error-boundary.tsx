import { Component, type ErrorInfo, type ReactNode } from 'react'
import { ErrorMessage } from './error-message'

interface ErrorBoundaryProps {
  children: ReactNode
  /** Shown above the error text. Defaults to the message alone. */
  title?: string
  retryLabel?: string
}

interface ErrorBoundaryState {
  error: Error | null
}

/**
 * Catches a render-time throw and shows it instead of losing the page.
 *
 * These apps run inside an iframe on the plugin page, so an uncaught render
 * error leaves the reader looking at a blank white panel with nothing to click -
 * no message, no way back, and no hint that a reload would help. React only
 * offers this as a class component; there is no hook equivalent.
 *
 * Recovery is a reload rather than a re-render: whatever produced the bad state
 * is still in memory, so re-rendering the same tree usually throws again.
 *
 * It does not catch errors thrown outside rendering - event handlers, promise
 * rejections - which already surface as toasts through `errorMessage`.
 */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    // The component stack is the only place that says *where* it broke, and it
    // is gone once the fallback renders.
    console.error('Unhandled render error', error, info.componentStack)
  }

  render() {
    const { error } = this.state
    if (!error) return this.props.children
    return (
      <div className="flex h-full items-center justify-center p-6">
        <ErrorMessage
          message={this.props.title ? `${this.props.title}: ${error.message}` : error.message}
          onRetry={() => window.location.reload()}
          retryLabel={this.props.retryLabel}
        />
      </div>
    )
  }
}
