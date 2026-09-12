import { memo, useCallback, useEffect, useRef, useState } from 'react'
import { ExternalLink, Globe, Loader2 } from '@/components/icons'
import { listen } from '@tauri-apps/api/event'
import type { TFunction } from 'i18next'
import Markdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { performUpdate, openExternalUrl, type CheckUpdateResult } from '@/lib/api'
import { showError } from '@/lib/uiFeedback'

type UpdateDialogProps = {
  open: boolean
  result: CheckUpdateResult | null
  t: TFunction
  onClose: () => void
}

type UpdateState = 'idle' | 'downloading' | 'done'

const UpdateDialog = ({ open, result, t, onClose }: UpdateDialogProps) => {
  const [state, setState] = useState<UpdateState>('idle')
  const [downloaded, setDownloaded] = useState(0)
  const [totalSize, setTotalSize] = useState<number | null>(null)
  const unlistenRef = useRef<(() => void) | null>(null)

  // Reset state when dialog opens/closes
  useEffect(() => {
    if (!open) {
      setState('idle')
      setDownloaded(0)
      setTotalSize(null)
      if (unlistenRef.current) {
        unlistenRef.current()
        unlistenRef.current = null
      }
    }
  }, [open])

  const handleGoWebsite = useCallback(() => {
    if (!result?.release_url) return
    openExternalUrl(result.release_url).catch(() => {
      // Fallback: open in new tab (browser dev mode)
      window.open(result.release_url, '_blank', 'noopener,noreferrer')
    })
  }, [result])

  const handleUpdate = useCallback(async () => {
    if (!result?.update_available) return
    setState('downloading')
    setDownloaded(0)
    setTotalSize(null)

    // Listen for progress events from Rust backend
    try {
      unlistenRef.current = await listen<{
        chunk_length: number
        content_length: number | null
      }>('update://progress', (event) => {
        const { chunk_length, content_length } = event.payload
        setDownloaded((prev) => prev + chunk_length)
        if (content_length && content_length > 0) {
          setTotalSize(content_length)
        }
      })
    } catch {
      // listener setup failure is non-fatal
    }

    try {
      const res = await performUpdate()
      if (res.ok) {
        setState('done')
      } else {
        showError(res.message, res)
        setState('idle')
      }
    } catch (e) {
      showError(`${t('update.updateFailed')}: ${e instanceof Error ? e.message : String(e)}`, e)
      setState('idle')
    } finally {
      if (unlistenRef.current) {
        unlistenRef.current()
        unlistenRef.current = null
      }
    }
  }, [result, t])

  // Cleanup listener on unmount
  useEffect(() => {
    return () => {
      if (unlistenRef.current) {
        unlistenRef.current()
      }
    }
  }, [])

  if (!open || !result) return null

  const percent = totalSize && totalSize > 0
    ? Math.min(100, Math.round((downloaded / totalSize) * 100))
    : null
  const isDownloading = state === 'downloading'
  const isDone = state === 'done'

  return (
    <div className="modal-backdrop" onClick={isDownloading ? undefined : onClose}>
      <div className="update-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="update-dialog-header">
          <div className="update-dialog-title">
            {t('update.versionReady', { version: result.latest_version })}
          </div>
        </div>

        <div className="update-dialog-body">
          <div className="update-dialog-version-row">
            <span className="update-dialog-version-label">{t('update.currentVersion')}</span>
            <span className="update-dialog-version-value">v{result.current_version}</span>
            <span className="update-dialog-version-arrow">→</span>
            <span className="update-dialog-version-value update-dialog-version-new">v{result.latest_version}</span>
          </div>
          {result.release_notes ? (
            <div className="update-dialog-notes-section">
              <div className="update-dialog-notes-title">{t('update.releaseNotes')}</div>
              <div className="update-dialog-notes markdown-body">
                <Markdown remarkPlugins={[remarkGfm]}>
                  {result.release_notes}
                </Markdown>
              </div>
            </div>
          ) : null}
        </div>

        <div className="update-dialog-footer">
          {isDownloading && (
            <>
              <div className="update-dialog-progress">
                <div
                  className="update-dialog-progress-bar"
                  style={{ width: percent != null ? `${percent}%` : '40%' }}
                />
              </div>
              <div className="update-dialog-progress-text">
                {percent != null ? t('update.downloadingPercent', { percent }) : t('update.downloading')}
              </div>
            </>
          )}

          <div className="update-dialog-actions">
            <button
              type="button"
              className="update-dialog-cancel"
              onClick={onClose}
              disabled={isDownloading}
            >
              {isDone ? t('close') : t('update.cancel')}
            </button>
            {!isDone && !isDownloading && (
              <button
                type="button"
                className="update-dialog-website"
                onClick={handleGoWebsite}
              >
                <Globe size={14} />
                {t('update.goWebsite')}
              </button>
            )}
            <button
              type="button"
              className="update-dialog-update"
              onClick={handleUpdate}
              disabled={isDownloading || isDone}
            >
              {isDownloading ? (
                <>
                  <Loader2 size={16} className="spin" />
                  {percent != null ? `${percent}%` : t('update.downloading')}
                </>
              ) : isDone ? (
                t('update.restarting')
              ) : (
                <>
                  <ExternalLink size={14} />
                  {t('update.update')}
                </>
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

export default memo(UpdateDialog)
