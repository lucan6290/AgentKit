import { memo, useCallback, useEffect, useMemo, useState } from 'react'
import type { TFunction } from 'i18next'
import {
  Copy,
  FilePlus2,
  FileText,
  FolderOpen,
  Link2,
  Plus,
  RefreshCw,
  Save,
  Search,
  Trash2,
  Unlink,
  Upload,
  X,
} from 'lucide-react'
import { promptService } from '@/services/promptService'
import { formatDisplayPath } from '@/lib/utils'
import { showError, showSuccess } from '@/lib/uiFeedback'
import { pickFile } from '@/lib/pickFolder'
import type { Prompt, PromptFileLink } from '@/features/prompts/types'

type PromptsPageProps = {
  t: TFunction
}

type ConfirmAction =
  | { type: 'delete'; prompt: Prompt }
  | { type: 'force-write'; link: PromptFileLink }

const PromptsPage = ({ t }: PromptsPageProps) => {
  const [prompts, setPrompts] = useState<Prompt[]>([])
  const [selectedPromptId, setSelectedPromptId] = useState<string | null>(null)
  const [searchQuery, setSearchQuery] = useState('')
  const [name, setName] = useState('')
  const [content, setContent] = useState('')
  const [savedName, setSavedName] = useState('')
  const [savedContent, setSavedContent] = useState('')
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [busyLinkId, setBusyLinkId] = useState<string | null>(null)
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null)
  const [showLinkDialog, setShowLinkDialog] = useState(false)
  const [linkPath, setLinkPath] = useState('')
  const [writeBackEnabled, setWriteBackEnabled] = useState(false)
  const [editingLinkId, setEditingLinkId] = useState<string | null>(null)
  const [scanning, setScanning] = useState(false)
  const [showCreateDialog, setShowCreateDialog] = useState(false)
  const [createName, setCreateName] = useState('')
  const [createFilePath, setCreateFilePath] = useState('')

  const loadPrompts = useCallback(async () => {
    setLoading(true)
    try {
      const items = await promptService.listPrompts()
      setPrompts(items)
      setSelectedPromptId((current) => current && items.some((prompt) => prompt.id === current) ? current : null)
    } catch (error) {
      showError(t('prompts.loadError'), error)
    } finally {
      setLoading(false)
    }
  }, [t])

  useEffect(() => {
    void loadPrompts()
  }, [loadPrompts])

  const selectedPrompt = useMemo(
    () => prompts.find((prompt) => prompt.id === selectedPromptId) ?? null,
    [prompts, selectedPromptId],
  )

  const filteredPrompts = useMemo(() => {
    const query = searchQuery.trim().toLocaleLowerCase()
    if (!query) return prompts
    return prompts.filter((prompt) => (
      prompt.name.toLocaleLowerCase().includes(query) || prompt.content.toLocaleLowerCase().includes(query)
    ))
  }, [prompts, searchQuery])

  const hasChanges = name !== savedName || content !== savedContent

  const selectPrompt = useCallback((prompt: Prompt) => {
    setSelectedPromptId(prompt.id)
    setName(prompt.name)
    setContent(prompt.content)
    setSavedName(prompt.name)
    setSavedContent(prompt.content)
  }, [])

  const handleSave = useCallback(async () => {
    if (!selectedPrompt) return
    setSaving(true)
    try {
      const updated = await promptService.updatePrompt(selectedPrompt.id, name.trim() || t('prompts.untitled'), content)
      setPrompts((current) => current.map((prompt) => prompt.id === updated.id ? updated : prompt))
      selectPrompt(updated)
      showSuccess(t('prompts.saved'))
    } catch (error) {
      showError(t('prompts.saveError'), error)
    } finally {
      setSaving(false)
    }
  }, [content, name, selectPrompt, selectedPrompt, t])

  const handleCopyContent = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(content)
      showSuccess(t('prompts.contentCopied'))
    } catch (error) {
      showError(t('copyFailed'), error)
    }
  }, [content, t])

  const handleDuplicate = useCallback(async () => {
    if (!selectedPrompt) return
    try {
      const duplicate = await promptService.duplicatePrompt(selectedPrompt.id)
      setPrompts((current) => [duplicate, ...current])
      selectPrompt(duplicate)
      showSuccess(t('prompts.duplicated'))
    } catch (error) {
      showError(t('prompts.duplicateError'), error)
    }
  }, [selectPrompt, selectedPrompt, t])

  const handleConfirm = useCallback(async () => {
    if (!confirmAction) return
    try {
      if (confirmAction.type === 'delete') {
        await promptService.deletePrompt(confirmAction.prompt.id)
        setPrompts((current) => current.filter((prompt) => prompt.id !== confirmAction.prompt.id))
        setSelectedPromptId(null)
        setName('')
        setContent('')
        setSavedName('')
        setSavedContent('')
        showSuccess(t('prompts.deleted'))
      } else {
        setBusyLinkId(confirmAction.link.id)
        await promptService.writePromptToFile(confirmAction.link.id, true)
        await loadPrompts()
        showSuccess(t('prompts.written'))
      }
      setConfirmAction(null)
    } catch (error) {
      showError(confirmAction.type === 'delete' ? t('prompts.deleteError') : t('prompts.writeError'), error)
    } finally {
      setBusyLinkId(null)
    }
  }, [confirmAction, loadPrompts, t])

  const handleImportFile = useCallback(async () => {
    const filePath = await pickFile(t('prompts.importFile'))
    if (!filePath) return
    try {
      const prompt = await promptService.importPromptFile(filePath)
      setPrompts((current) => [prompt, ...current])
      selectPrompt(prompt)
      showSuccess(t('prompts.imported'))
    } catch (error) {
      showError(t('prompts.importError'), error)
    }
  }, [selectPrompt, t])

  const handleScan = useCallback(async () => {
    setScanning(true)
    try {
      const result = await promptService.scanToolPromptFiles()
      showSuccess(t('prompts.scanResult', { scanned: result.scanned, created: result.created, updated: result.updated }))
      await loadPrompts()
    } catch (error) {
      showError(t('prompts.scanError'), error)
    } finally {
      setScanning(false)
    }
  }, [loadPrompts, t])

  const handleCreateFilePick = useCallback(async () => {
    const filePath = await pickFile(t('prompts.filePath'))
    if (!filePath) return
    setCreateFilePath(filePath)
  }, [t])

  const handleCreateWithFile = useCallback(async () => {
    setSaving(true)
    try {
      const trimmedName = createName.trim() || t('prompts.untitled')
      let prompt: Prompt
      if (createFilePath.trim()) {
        prompt = await promptService.importPromptFile(createFilePath.trim(), trimmedName)
      } else {
        prompt = await promptService.createPrompt(trimmedName, '')
      }
      setPrompts((current) => [prompt, ...current])
      selectPrompt(prompt)
      setShowCreateDialog(false)
      setCreateName('')
      setCreateFilePath('')
      showSuccess(t('prompts.saved'))
    } catch (error) {
      showError(t('prompts.createError'), error)
    } finally {
      setSaving(false)
    }
  }, [createFilePath, createName, selectPrompt, t])

  const handleLinkFilePick = useCallback(async () => {
    const filePath = await pickFile(t('prompts.addLink'))
    if (!filePath) return
    setLinkPath(filePath)
  }, [t])

  const handleCreateLink = useCallback(async () => {
    if (!selectedPrompt || !linkPath.trim()) return
    try {
      if (editingLinkId) {
        await promptService.updatePromptFileLink(editingLinkId, linkPath.trim(), writeBackEnabled)
      } else {
        await promptService.createPromptFileLink(selectedPrompt.id, linkPath.trim(), writeBackEnabled)
      }
      await loadPrompts()
      setShowLinkDialog(false)
      setLinkPath('')
      setWriteBackEnabled(false)
      setEditingLinkId(null)
      showSuccess(t('prompts.linked'))
    } catch (error) {
      showError(t('prompts.linkError'), error)
    }
  }, [editingLinkId, linkPath, loadPrompts, selectedPrompt, t, writeBackEnabled])

  const handleRefreshLink = useCallback(async (link: PromptFileLink) => {
    setBusyLinkId(link.id)
    try {
      const updated = await promptService.refreshPromptFileLink(link.id)
      setPrompts((current) => current.map((prompt) => prompt.id === updated.id ? updated : prompt))
      selectPrompt(updated)
      showSuccess(t('prompts.refreshed'))
    } catch (error) {
      showError(t('prompts.refreshError'), error)
    } finally {
      setBusyLinkId(null)
    }
  }, [selectPrompt, t])

  const handleWriteLink = useCallback(async (link: PromptFileLink) => {
    setBusyLinkId(link.id)
    try {
      await promptService.writePromptToFile(link.id)
      await loadPrompts()
      showSuccess(t('prompts.written'))
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      if (message.includes('conflict')) {
        setConfirmAction({ type: 'force-write', link })
      } else {
        showError(t('prompts.writeError'), error)
      }
    } finally {
      setBusyLinkId(null)
    }
  }, [loadPrompts, t])

  const handleUnlink = useCallback(async (link: PromptFileLink) => {
    setBusyLinkId(link.id)
    try {
      await promptService.unlinkPromptFile(link.id)
      await loadPrompts()
      showSuccess(t('prompts.unlinked'))
    } catch (error) {
      showError(t('prompts.unlinkError'), error)
    } finally {
      setBusyLinkId(null)
    }
  }, [loadPrompts, t])

  if (loading) {
    return <div className="prompts-page"><div className="prompts-loading">{t('prompts.loading')}</div></div>
  }

  return (
    <div className="prompts-page">
      <div className="prompts-header">
        <div>
          <h2>{t('prompts.title')}</h2>
          <div className="prompts-subtitle">{t('prompts.subtitle', { total: prompts.length })}</div>
        </div>
        <div className="prompts-header-actions">
          <button className="btn btn-secondary" type="button" disabled={scanning} onClick={() => void handleScan()}>
            <RefreshCw size={15} />{scanning ? t('prompts.scanning') : t('prompts.scanTools')}
          </button>
          <button className="btn btn-secondary" type="button" onClick={() => void handleImportFile()}>
            <Upload size={15} />{t('prompts.importFile')}
          </button>
          <button className="btn btn-primary" type="button" onClick={() => setShowCreateDialog(true)}>
            <Plus size={15} />{t('prompts.new')}
          </button>
        </div>
      </div>

      <div className="prompts-workspace">
        <aside className="prompts-list-panel">
          <div className="prompts-search">
            <Search size={15} />
            <input value={searchQuery} onChange={(event) => setSearchQuery(event.target.value)} placeholder={t('prompts.searchPlaceholder')} />
          </div>
          <div className="prompts-list">
            {filteredPrompts.map((prompt) => (
              <button key={prompt.id} className={`prompts-list-item${prompt.id === selectedPromptId ? ' active' : ''}`} type="button" onClick={() => selectPrompt(prompt)}>
                <FileText size={16} />
                <span><strong>{prompt.name}</strong><small>{prompt.content || t('prompts.emptyContent')}</small></span>
                {prompt.file_links.length > 0 ? <Link2 size={13} className="prompts-list-linked" /> : null}
              </button>
            ))}
            {filteredPrompts.length === 0 ? <div className="prompts-list-empty">{t('prompts.noResults')}</div> : null}
          </div>
        </aside>

        {selectedPrompt ? (
          <main className="prompts-editor-panel">
            <div className="prompts-editor-header">
              <input className="prompts-name-input" value={name} onChange={(event) => setName(event.target.value)} placeholder={t('prompts.namePlaceholder')} />
              <div className="prompts-editor-actions">
                <button className="btn btn-primary" type="button" disabled={saving || !hasChanges} onClick={() => void handleSave()}><Save size={14} />{saving ? t('prompts.saving') : t('prompts.save')}</button>
                <button className="btn btn-secondary" type="button" onClick={() => void handleCopyContent()}><Copy size={14} />{t('prompts.copyContent')}</button>
                <button className="btn btn-secondary" type="button" onClick={() => void handleDuplicate()}><FilePlus2 size={14} />{t('prompts.duplicate')}</button>
                <button className="btn btn-secondary prompts-delete-btn" type="button" onClick={() => setConfirmAction({ type: 'delete', prompt: selectedPrompt })}><Trash2 size={14} />{t('prompts.delete')}</button>
              </div>
            </div>
            <textarea className="prompts-editor-textarea" value={content} onChange={(event) => setContent(event.target.value)} spellCheck={false} placeholder={t('prompts.editorPlaceholder')} />
            <section className="prompts-links-section">
              <div className="prompts-links-header"><div><h3>{t('prompts.fileLinks')}</h3><p>{t('prompts.fileLinksHint')}</p></div><button className="btn btn-secondary" type="button" onClick={() => { const firstLink = selectedPrompt.file_links[0]; setLinkPath(firstLink?.file_path ?? ''); setWriteBackEnabled(firstLink?.write_back_enabled ?? false); setEditingLinkId(firstLink?.id ?? null); setShowLinkDialog(true) }}><Link2 size={14} />{t('prompts.addLink')}</button></div>
              {selectedPrompt.file_links.length === 0 ? <div className="prompts-links-empty">{t('prompts.noLinks')}</div> : selectedPrompt.file_links.map((link) => (
                <div key={link.id} className="prompts-link-item">
                  <FolderOpen size={16} /><div className="prompts-link-path"><span>{formatDisplayPath(link.file_path)}</span><small>{link.write_back_enabled ? t('prompts.writeBackEnabled') : t('prompts.writeBackDisabled')} · {link.exists_on_disk ? t('prompts.exists') : t('prompts.missing')}</small></div>
                  <div className="prompts-link-actions">
                    <button className="btn-icon" type="button" disabled={busyLinkId === link.id} title={t('prompts.refreshLink')} onClick={() => void handleRefreshLink(link)}><RefreshCw size={14} /></button>
                    <button className="btn-icon" type="button" disabled={busyLinkId === link.id || !link.write_back_enabled} title={t('prompts.writeToFile')} onClick={() => void handleWriteLink(link)}><Save size={14} /></button>
                    <button className="btn-icon prompts-unlink-btn" type="button" disabled={busyLinkId === link.id} title={t('prompts.unlink')} onClick={() => void handleUnlink(link)}><Unlink size={14} /></button>
                  </div>
                </div>
              ))}
            </section>
          </main>
        ) : (
          <main className="prompts-empty"><div className="prompts-empty-icon"><FileText size={40} /></div><p className="prompts-empty-title">{t('prompts.selectTitle')}</p><p className="prompts-empty-desc">{t('prompts.selectDescription')}</p></main>
        )}
      </div>

      {showLinkDialog ? <div className="modal-backdrop" onClick={() => setShowLinkDialog(false)}><div className="modal prompts-link-modal" role="dialog" aria-modal="true" onClick={(event) => event.stopPropagation()}><div className="modal-header"><div className="modal-title">{t('prompts.addLink')}</div><button className="icon-btn" type="button" onClick={() => setShowLinkDialog(false)} aria-label={t('close')}><X size={18} /></button></div><div className="prompts-link-form"><label>{t('prompts.filePath')}<div className="settings-input-row"><input className="settings-input" value={linkPath} onChange={(event) => setLinkPath(event.target.value)} placeholder={t('prompts.filePathPlaceholder')} /><button className="btn btn-secondary" type="button" onClick={() => void handleLinkFilePick()}>{t('browse')}</button></div></label><label className="prompts-write-back-toggle"><input type="checkbox" checked={writeBackEnabled} onChange={(event) => setWriteBackEnabled(event.target.checked)} />{t('prompts.enableWriteBack')}</label></div><div className="modal-actions"><button className="btn btn-secondary" type="button" onClick={() => setShowLinkDialog(false)}>{t('cancel')}</button><button className="btn btn-primary" type="button" disabled={!linkPath.trim()} onClick={() => void handleCreateLink()}>{t('prompts.addLink')}</button></div></div></div> : null}
      {showCreateDialog ? <div className="modal-backdrop" onClick={() => setShowCreateDialog(false)}><div className="modal prompts-link-modal" role="dialog" aria-modal="true" onClick={(event) => event.stopPropagation()}><div className="modal-header"><div className="modal-title">{t('prompts.createTitle')}</div><button className="icon-btn" type="button" onClick={() => setShowCreateDialog(false)} aria-label={t('close')}><X size={18} /></button></div><div className="prompts-link-form"><label>{t('prompts.namePlaceholder')}<input className="settings-input" value={createName} onChange={(event) => setCreateName(event.target.value)} placeholder={t('prompts.namePlaceholder')} /></label><label>{t('prompts.filePath')}<small className="prompts-file-path-hint">{t('prompts.filePathOptional')}</small><div className="settings-input-row"><input className="settings-input" value={createFilePath} onChange={(event) => setCreateFilePath(event.target.value)} placeholder={t('prompts.filePathPlaceholder')} /><button className="btn btn-secondary" type="button" onClick={() => void handleCreateFilePick()}>{t('browse')}</button></div></label></div><div className="modal-actions"><button className="btn btn-secondary" type="button" onClick={() => setShowCreateDialog(false)}>{t('cancel')}</button><button className="btn btn-primary" type="button" disabled={saving} onClick={() => void handleCreateWithFile()}><Plus size={14} />{t('prompts.create')}</button></div></div></div> : null}
      {confirmAction ? <div className="modal-backdrop" onClick={() => setConfirmAction(null)}><div className="modal prompts-confirm-modal" role="dialog" aria-modal="true" onClick={(event) => event.stopPropagation()}><div className="modal-body"><h3>{confirmAction.type === 'delete' ? t('prompts.deleteTitle') : t('prompts.conflictTitle')}</h3><p>{confirmAction.type === 'delete' ? t('prompts.deleteConfirm', { name: confirmAction.prompt.name }) : t('prompts.conflictDescription')}</p></div><div className="modal-actions"><button className="btn btn-secondary" type="button" onClick={() => setConfirmAction(null)}>{t('cancel')}</button><button className="btn btn-danger-solid" type="button" onClick={() => void handleConfirm()}>{confirmAction.type === 'delete' ? t('prompts.delete') : t('prompts.forceWrite')}</button></div></div></div> : null}
    </div>
  )
}

export default memo(PromptsPage)
