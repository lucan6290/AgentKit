import { memo, useCallback, useEffect, useRef, useState } from 'react'
import { ExternalLink, Loader2 } from 'lucide-react'
import { listen } from '@tauri-apps/api/event'
import type { TFunction } from 'i18next'
import { toast } from 'sonner'
import { performUpdate, type CheckUpdateResult } from '@/lib/api'

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
        toast.error(res.message)
        setState('idle')
      }
    } catch (e) {
      toast.error(`${t('update.updateFailed')}: ${e instanceof Error ? e.message : String(e)}`)
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

  const downloadUrl = result.release_url

  return (
    <div className="modal-backdrop" onClick={isDownloading ? undefined : onClose}>
      <div className="update-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="update-dialog-header">
          <div className="update-dialog-title">
            {t('update.versionReady', { version: result.latest_version })}
          </div>
          <a
            href={downloadUrl}
            target="_blank"
            rel="noopener noreferrer"
            className="update-dialog-download-btn"
          >
            <ExternalLink size={14} />
            {t('update.goDownload')}
          </a>
        </div>

        <div className="update-dialog-body">
          <div className="update-dialog-version">{result.latest_version}</div>
          {result.release_notes ? (
            <div className="update-dialog-notes">{result.release_notes}</div>
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
              {t('update.cancel')}
            </button>
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
                t('update.update')
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

export default memo(UpdateDialog)
