import { Component, type ErrorInfo, type ReactNode } from 'react'
import i18n from '@/i18n'
import { logger } from '@/lib/logger'
import { copyDiagnosticText } from '@/lib/uiFeedback'

interface ErrorBoundaryProps {
  children: ReactNode
  fallback?: ReactNode
}

interface ErrorBoundaryState {
  hasError: boolean
  error: Error | null
  componentStack: string
}

export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { hasError: false, error: null, componentStack: '' }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error, componentStack: '' }
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    this.setState({ componentStack: info.componentStack ?? '' })

    logger.error({
      event: 'frontend.react.error',
      area: 'error-boundary',
      outcome: 'failed',
      message: 'React error boundary caught an error',
      meta: { component_stack: info.componentStack },
    }, error)
  }

  handleReset = () => {
    this.setState({ hasError: false, error: null, componentStack: '' })
  }

  buildDiagnosticText() {
    const { error, componentStack } = this.state
    const lines = [
      `${i18n.t('feedback.errorBoundaryTitle')}`,
      '',
      error ? `${error.name}: ${error.message}` : i18n.t('feedback.errorBoundaryFallback'),
    ]

    if (error?.stack) {
      lines.push('', 'Stack:', error.stack)
    }

    if (componentStack) {
      lines.push('', 'Component stack:', componentStack)
    }

    return lines.join('\n')
  }

  handleCopyDetails = () => {
    void copyDiagnosticText(this.buildDiagnosticText())
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) return this.props.fallback

      return (
        <div className="error-boundary">
          <div className="error-boundary-content">
            <h2>{i18n.t('feedback.errorBoundaryTitle')}</h2>
            <p>{i18n.t('feedback.errorBoundaryDescription')}</p>
            <pre className="error-boundary-detail">
              {this.state.error?.message ?? i18n.t('feedback.errorBoundaryFallback')}
            </pre>
            <div className="error-boundary-actions">
              <button className="btn btn-primary" type="button" onClick={this.handleReset}>
                {i18n.t('feedback.tryAgain')}
              </button>
              <button className="btn btn-secondary" type="button" onClick={this.handleCopyDetails}>
                {i18n.t('feedback.copyDetails')}
              </button>
            </div>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}
