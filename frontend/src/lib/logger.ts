import { invoke } from '@tauri-apps/api/core'

export type LogLevel = 'debug' | 'info' | 'warn' | 'error' | 'silent'
export type LogArea =
  | 'app'
  | 'api'
  | 'error-boundary'
  | 'prompts'
  | 'settings'
  | 'skills'
  | 'window'

export type LogOutcome = 'started' | 'success' | 'failed' | 'canceled' | 'unknown'
export type LogMeta = Record<string, unknown>

type BackendLogPayload = {
  level: Exclude<LogLevel, 'silent'>
  event: string
  message: string
  area?: LogArea
  outcome?: LogOutcome
  command?: string
  task_id?: string
  error_code?: string
  duration_ms?: number
  app_version?: string
  meta?: LogMeta
}

type LogOptions = {
  event: string
  message: string
  area?: LogArea
  outcome?: LogOutcome
  command?: string
  taskId?: string
  errorCode?: string
  durationMs?: number
  meta?: LogMeta
}

const LEVEL_ORDER: Record<LogLevel, number> = {
  debug: 10,
  info: 20,
  warn: 30,
  error: 40,
  silent: 50,
}

const SENSITIVE_KEYS = new Set([
  'token',
  'secret',
  'password',
  'authorization',
  'cookie',
  'api_key',
  'access_key',
  'refresh_token',
  'private_key',
])

const MAX_STRING_LENGTH = 4096
const MAX_DEPTH = 8

let currentLevel: LogLevel = import.meta.env.DEV ? 'debug' : 'info'
let globalErrorLoggingInstalled = false

export function isLogLevel(value: string): value is Exclude<LogLevel, 'silent'> {
  return value === 'debug' || value === 'info' || value === 'warn' || value === 'error'
}

export function setLoggerLevel(level: LogLevel) {
  currentLevel = level
}

export function getLoggerLevel(): LogLevel {
  return currentLevel
}

export function installGlobalErrorLogging() {
  if (globalErrorLoggingInstalled || typeof window === 'undefined') return
  globalErrorLoggingInstalled = true

  window.addEventListener('error', (event) => {
    logger.error(
      {
        event: 'frontend.window.error',
        area: 'window',
        outcome: 'failed',
        message: event.message || 'Unhandled window error',
        meta: {
          filename: event.filename,
          line: event.lineno,
          column: event.colno,
        },
      },
      event.error,
    )
  })

  window.addEventListener('unhandledrejection', (event) => {
    logger.error(
      {
        event: 'frontend.promise.unhandled_rejection',
        area: 'window',
        outcome: 'failed',
        message: 'Unhandled promise rejection',
      },
      event.reason,
    )
  })
}

export const logger = {
  getLevel: getLoggerLevel,
  setLevel: setLoggerLevel,
  debug: (options: LogOptions) => writeLog('debug', options),
  info: (options: LogOptions) => writeLog('info', options),
  warn: (options: LogOptions, error?: unknown) => writeLog('warn', withError(options, error)),
  error: (options: LogOptions, error?: unknown) => writeLog('error', withError(options, error)),
  child: (area: LogArea) => ({
    debug: (options: Omit<LogOptions, 'area'>) => writeLog('debug', { ...options, area }),
    info: (options: Omit<LogOptions, 'area'>) => writeLog('info', { ...options, area }),
    warn: (options: Omit<LogOptions, 'area'>, error?: unknown) =>
      writeLog('warn', withError({ ...options, area }, error)),
    error: (options: Omit<LogOptions, 'area'>, error?: unknown) =>
      writeLog('error', withError({ ...options, area }, error)),
  }),
}

function writeLog(level: Exclude<LogLevel, 'silent'>, options: LogOptions) {
  if (LEVEL_ORDER[level] < LEVEL_ORDER[currentLevel]) return

  const payload: BackendLogPayload = {
    level,
    event: options.event,
    message: options.message,
    area: options.area,
    outcome: options.outcome,
    command: options.command,
    task_id: options.taskId,
    error_code: options.errorCode,
    duration_ms: options.durationMs,
    app_version: __APP_VERSION__,
    meta: sanitizeMeta(options.meta),
  }

  writeConsole(payload)
  void invoke('write_frontend_log', { payload }).catch((err) => {
    console.warn('[logger] failed to write backend log', err)
  })
}

function writeConsole(payload: BackendLogPayload) {
  const args: unknown[] = [`[${payload.area ?? 'app'}] ${payload.event}: ${payload.message}`]
  if (payload.meta && Object.keys(payload.meta).length > 0) {
    args.push(payload.meta)
  }

  switch (payload.level) {
    case 'debug':
      console.debug(...args)
      break
    case 'warn':
      console.warn(...args)
      break
    case 'error':
      console.error(...args)
      break
    default:
      console.info(...args)
      break
  }
}

function withError(options: LogOptions, error: unknown): LogOptions {
  if (error === undefined) return options
  return {
    ...options,
    meta: {
      ...options.meta,
      error: serializeError(error),
    },
  }
}

function sanitizeMeta(meta: LogMeta | undefined): LogMeta | undefined {
  if (!meta) return undefined
  const sanitized = sanitizeValue(meta, 0)
  return isPlainRecord(sanitized) ? sanitized : undefined
}

function sanitizeValue(value: unknown, depth: number): unknown {
  if (depth >= MAX_DEPTH) return '[TRUNCATED]'
  if (typeof value === 'string') return truncateString(value)
  if (value === null || typeof value === 'number' || typeof value === 'boolean') return value
  if (Array.isArray(value)) return value.map((item) => sanitizeValue(item, depth + 1))
  if (value instanceof Error) return serializeError(value)
  if (isPlainRecord(value)) {
    const next: LogMeta = {}
    for (const [key, item] of Object.entries(value)) {
      next[key] = SENSITIVE_KEYS.has(key.toLowerCase()) ? '[REDACTED]' : sanitizeValue(item, depth + 1)
    }
    return next
  }
  return String(value)
}

function serializeError(error: unknown): LogMeta {
  if (error instanceof Error) {
    return {
      name: error.name,
      message: truncateString(error.message),
      stack: error.stack ? truncateString(error.stack) : undefined,
    }
  }

  if (isPlainRecord(error)) {
    return sanitizeValue(error, 0) as LogMeta
  }

  return { message: truncateString(String(error)) }
}

function isPlainRecord(value: unknown): value is LogMeta {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function truncateString(value: string): string {
  if (value.length <= MAX_STRING_LENGTH) return value
  return `${value.slice(0, MAX_STRING_LENGTH)}...[TRUNCATED]`
}
