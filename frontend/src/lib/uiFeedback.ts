import { toast } from 'sonner'
import i18n from '@/i18n'

type FeedbackOptions = {
  duration?: number
  details?: unknown
  copyText?: string
}

const SUCCESS_DURATION = 1800
const INFO_DURATION = 2200
const ERROR_DURATION = 4200

function t(key: string) {
  return i18n.t(key)
}

function stringifyDetail(value: unknown): string {
  if (value === null || value === undefined) return ''
  if (value instanceof Error) {
    return [
      `${value.name}: ${value.message}`,
      value.stack ? `\n${value.stack}` : '',
    ].join('')
  }
  if (typeof value === 'string') return value
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return String(value)
  }
}

function buildCopyText(message: string, detail?: unknown, copyText?: string) {
  const explicit = copyText?.trim()
  if (explicit) return explicit

  const detailText = stringifyDetail(detail).trim()
  if (!detailText || detailText === message.trim()) return message
  return `${message}\n\n${t('feedback.errorDetails')}:\n${detailText}`
}

export async function copyDiagnosticText(text: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast.success(t('feedback.detailsCopied'), { duration: SUCCESS_DURATION })
    return true
  } catch {
    toast.error(t('feedback.copyDetailsFailed'), { duration: ERROR_DURATION })
    return false
  }
}

export function showSuccess(message: string, options: FeedbackOptions = {}) {
  toast.success(message, { duration: options.duration ?? SUCCESS_DURATION })
}

export function showInfo(message: string, options: FeedbackOptions = {}) {
  toast.info(message, { duration: options.duration ?? INFO_DURATION })
}

export function showError(
  message: string,
  errorOrDetails?: unknown,
  options: FeedbackOptions = {},
) {
  const copyText = buildCopyText(
    message,
    options.details ?? errorOrDetails,
    options.copyText,
  )

  toast.error(message, {
    duration: options.duration ?? ERROR_DURATION,
    action: {
      label: t('feedback.copyDetails'),
      onClick: () => {
        void copyDiagnosticText(copyText)
      },
    },
  })
}
