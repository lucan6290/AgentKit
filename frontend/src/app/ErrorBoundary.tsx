import { Component, type ErrorInfo, type ReactNode } from 'react'
import { logger } from '@/lib/logger'

interface ErrorBoundaryProps {
  children: ReactNode
  fallback?: ReactNode
}

interface ErrorBoundaryState {
  hasError: boolean
  error: Error | null
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false, error: null }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    logger.error({
      event: 'frontend.react.error',
      area: 'error-boundary',
      outcome: 'failed',
      message: 'React error boundary caught an error',
      meta: { component_stack: info.componentStack },
    }, error)
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null })
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback

      return (
        <div className="error-boundary">
          <div className="error-boundary-content">
            <h2>Something went wrong</h2>
            <p>{this.state.error?.message ?? 'An unexpected error occurred'}</p>
            <button className="btn btn-primary" type="button" onClick={this.handleReset}>
              Try Again
            </button>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}
