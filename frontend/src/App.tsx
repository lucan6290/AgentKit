import { lazy, Suspense, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { RefreshCw, Tag, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Toaster } from 'sonner'
import { getCurrent, onOpenUrl } from '@tauri-apps/plugin-deep-link'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Header, LoadingOverlay } from '@/components/layout'
import {
  FilterBar,
  SkillsList,
  useSkills,
  useSkillFilter,
  useScopeState,
  useScopeManager,
  useAddSkill,
  useTagActions,
  useSkillActions,
} from '@/features/skills'
import { useTheme } from '@/features/settings'
import { useImportFlow } from '@/features/import-flow'
import { skillService, promptService } from '@/services'
import { AppStateProvider, useAppState } from '@/context/AppStateContext'
import { ModalProvider, useModal } from '@/context/ModalContext'
import type { ManagedSkill } from '@/features/skills'
import { checkUpdate, getAutoCheckUpdate, getLogLevel, type CheckUpdateResult } from '@/lib/api'
import { isLogLevel, logger } from '@/lib/logger'
import UpdateDialog from '@/features/settings/components/UpdateDialog'

// ─── Lazy-loaded views ──────────────────────────────
const SkillDetailView = lazy(() => import('@/features/skills/components/SkillDetailView'))
const TagsPage = lazy(() => import('@/features/tags/components/TagsPage'))
const SettingsPage = lazy(() => import('@/features/settings/components/SettingsPage'))
const ToolsPage = lazy(() => import('@/features/tools/components/ToolsPage'))
const PromptsPage = lazy(() => import('@/features/prompts/components/PromptsPage'))

// ─── Lazy-loaded modals ─────────────────────────────
const SkillInfoModal = lazy(() => import('@/features/skills/modals/SkillInfoModal'))
const AddSkillModal = lazy(() => import('@/features/skills/modals/AddSkillModal'))
const EditSkillTagsModal = lazy(() => import('@/features/skills/modals/EditSkillTagsModal'))
const ImportModal = lazy(() => import('@/features/import-flow/components/ImportModal'))
const LocalPickModal = lazy(() => import('@/features/import-flow/components/LocalPickModal'))
const SharedDirModal = lazy(() => import('@/features/skills/modals/SharedDirModal'))
const SuiteSyncModal = lazy(() => import('@/features/skills/modals/SuiteSyncModal'))
const ScopeSyncModal = lazy(() => import('@/features/skills/modals/ScopeSyncModal'))
const NewToolsModal = lazy(() => import('@/features/tools/modals/NewToolsModal'))
const DeleteModal = lazy(() => import('@/features/skills/modals/DeleteModal'))

function App() {
  return (
    <AppStateProvider>
      <ModalProvider>
        <AppContent />
      </ModalProvider>
    </AppStateProvider>
  )
}

function AppContent() {
  const { t } = useTranslation()
  const appState = useAppState()
  const modal = useModal()

  // ─── Layer 1：基础数据 ─────────────────────────
  const scopeState = useScopeState()
  const skills = useSkills(t, appState.setError, appState.setSuccessToastMessage)
  const theme = useTheme(t, skills.loadManagedSkills, appState.setError)

  // ─── 共享 loading 状态（供 useAddSkill 等使用）─
  const [loading, setLoading] = useState(false)
  const [loadingStartAt, setLoadingStartAt] = useState<number | null>(null)
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false)
  const [promptCount, setPromptCount] = useState(0)
  // When collapsed, track whether the user is hovering over the left edge
  // hot-zone or the sidebar itself (floating overlay) so we can reveal it.
  const [sidebarHovered, setSidebarHovered] = useState(false)
  const sidebarHoverTimer = useRef<number | null>(null)

  const handleToggleSidebar = useCallback(() => {
    setSidebarCollapsed((value) => !value)
    setSidebarHovered(false)
  }, [])

  const handleSidebarEnter = useCallback(() => {
    if (sidebarHoverTimer.current !== null) {
      window.clearTimeout(sidebarHoverTimer.current)
      sidebarHoverTimer.current = null
    }
    setSidebarHovered(true)
  }, [])

  const handleSidebarLeave = useCallback(() => {
    if (sidebarHoverTimer.current !== null) {
      window.clearTimeout(sidebarHoverTimer.current)
    }
    // Small delay so that clicking the collapse button inside the sidebar
    // can complete before the overlay hides itself.
    sidebarHoverTimer.current = window.setTimeout(() => {
      setSidebarHovered(false)
      sidebarHoverTimer.current = null
    }, 120)
  }, [])

  // Clean up the pending timer on unmount
  useEffect(() => {
    return () => {
      if (sidebarHoverTimer.current !== null) {
        window.clearTimeout(sidebarHoverTimer.current)
      }
    }
  }, [])

  // ─── Prompt count badge ───────────────────────
  const refreshPromptCount = useCallback(() => {
    promptService.listPrompts()
      .then((items) => setPromptCount(items.length))
      .catch(() => { /* non-critical */ })
  }, [])

  // Load on mount, and refresh when leaving the prompts view (count may have changed)
  useEffect(() => {
    if (modal.activeView !== 'prompts') refreshPromptCount()
  }, [modal.activeView, refreshPromptCount])

  // ─── Window event logging ──────────────────────
  useEffect(() => {
    const appWindow = getCurrentWindow()
    const unlisteners: Promise<() => void>[] = []

    getLogLevel()
      .then((level) => {
        if (isLogLevel(level)) logger.setLevel(level)
      })
      .catch((err) => {
        logger.warn({
          event: 'settings.log_level.load.failed',
          area: 'settings',
          outcome: 'failed',
          command: 'get_log_level',
          message: 'Failed to load frontend log level',
        }, err)
      })

    const onResized = appWindow.onResized(({ payload }) => {
      logger.debug({
        event: 'window.resize.changed',
        area: 'window',
        message: 'Window size changed',
        meta: { width: payload.width, height: payload.height },
      })
    })
    const onMoved = appWindow.onMoved(({ payload }) => {
      logger.debug({
        event: 'window.move.changed',
        area: 'window',
        message: 'Window position changed',
        meta: { x: payload.x, y: payload.y },
      })
    })
    const onFocusChanged = appWindow.onFocusChanged(({ payload }) => {
      logger.debug({
        event: 'window.focus.changed',
        area: 'window',
        message: 'Window focus changed',
        meta: { focused: payload },
      })
    })

    unlisteners.push(onResized, onMoved, onFocusChanged)

    // Log initial window state
    appWindow.isMaximized().then((maximized) => {
      logger.info({
        event: 'window.initialized',
        area: 'window',
        outcome: 'success',
        message: 'Window initialized',
        meta: { maximized },
      })
    }).catch((err) => {
      logger.warn({
        event: 'window.initial_state.load.failed',
        area: 'window',
        outcome: 'failed',
        message: 'Failed to load initial window state',
      }, err)
    })

    return () => {
      unlisteners.forEach((p) => {
        p.then((unlisten) => unlisten()).catch(() => {})
      })
    }
  }, [])

  // ─── Update check ────────────────────────────────
  const [updateResult, setUpdateResult] = useState<CheckUpdateResult | null>(null)
  const [updateDialogOpen, setUpdateDialogOpen] = useState(false)
  const hasAutoChecked = useRef(false)

  useEffect(() => {
    if (hasAutoChecked.current) return
    hasAutoChecked.current = true
    // Check auto-check setting, then check for update if enabled
    getAutoCheckUpdate()
      .then((enabled) => {
        if (enabled) {
          return checkUpdate().then((res) => {
            if (res.update_available && !res.error) {
              setUpdateResult(res)
            }
          })
        }
      })
      .catch(() => {
        // Silently ignore update check failures (network etc.)
      })
  }, [])

  const handleOpenUpdateDialog = useCallback(() => {
    setUpdateDialogOpen(true)
  }, [])

  const handleCloseUpdateDialog = useCallback(() => {
    setUpdateDialogOpen(false)
  }, [])

  const [viewMode, setViewMode] = useState<'list' | 'cards'>(() => {
    if (typeof window === 'undefined') return 'cards'
    return window.localStorage.getItem('skills-view-mode') === 'list' ? 'list' : 'cards'
  })
  const [bulkMode, setBulkMode] = useState(false)
  const [selectedSkillIds, setSelectedSkillIds] = useState<string[]>([])
  const [showBulkTagsModal, setShowBulkTagsModal] = useState(false)
  const [bulkTagIds, setBulkTagIds] = useState<number[]>([])

  const handleViewModeChange = useCallback((value: 'list' | 'cards') => {
    setViewMode(value)
    if (typeof window !== 'undefined') {
      window.localStorage.setItem('skills-view-mode', value)
    }
  }, [])

  const handleToggleBulkMode = useCallback(() => {
    setBulkMode((value) => !value)
    setSelectedSkillIds([])
  }, [])

  const handleToggleBulkSelection = useCallback((skillId: string) => {
    setSelectedSkillIds((prev) =>
      prev.includes(skillId) ? prev.filter((id) => id !== skillId) : [...prev, skillId],
    )
  }, [])

  const handleToggleBulkTag = useCallback((tagId: number) => {
    setBulkTagIds((prev) =>
      prev.includes(tagId) ? prev.filter((id) => id !== tagId) : [...prev, tagId],
    )
  }, [])

  // ─── Layer 1.5：派生 helper ──────────────────
  const getSkillScope = useCallback(
    (skill: ManagedSkill): 'global' | 'project' => {
      const hasGlobalTarget = skill.targets.some((t) => (t.scope ?? 'global') === 'global')
      const hasProjectTarget = skill.targets.some((t) => (t.scope ?? 'global') === 'project')
      if (hasGlobalTarget && !hasProjectTarget) return 'global'
      if (hasProjectTarget && !hasGlobalTarget) return 'project'
      const stored = scopeState.skillScopeState[skill.id]?.scope
      if (stored === 'global' || stored === 'project') return stored
      return hasProjectTarget ? 'project' : 'global'
    },
    [scopeState.skillScopeState],
  )

  // ─── Layer 2：功能 hooks ──────────────────────
  const sourceSkills = useMemo(
    () => skills.managedSkills.filter((skill) => {
      const normalizedSource = skill.source_type === 'custom' ? 'custom' : 'community'
      return normalizedSource === modal.activeSkillSource
    }),
    [modal.activeSkillSource, skills.managedSkills],
  )

  const customSkillCount = useMemo(
    () => skills.managedSkills.filter((skill) => skill.source_type === 'custom').length,
    [skills.managedSkills],
  )

  const communitySkillCount = skills.managedSkills.length - customSkillCount

  const filter = useSkillFilter(
    sourceSkills,
    getSkillScope,
    skills.toolSkillNamesByTool,
  )

  const setError = appState.setError
  const setSuccessToastMessage = appState.setSuccessToastMessage

  const loadTags = skills.loadTags
  const loadManagedSkills = skills.loadManagedSkills

  const handleBulkDelete = useCallback(async () => {
    if (selectedSkillIds.length === 0) return
    setLoading(true)
    setLoadingStartAt(Date.now())
    try {
      await skillService.deleteManagedSkills(selectedSkillIds)
      setSuccessToastMessage(t('bulk.deleted', { count: selectedSkillIds.length }))
      setSelectedSkillIds([])
      setBulkMode(false)
      await loadManagedSkills()
      await loadTags(modal.activeSkillSource)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err), err)
    } finally {
      setLoading(false)
      setLoadingStartAt(null)
    }
  }, [selectedSkillIds, setLoading, setLoadingStartAt, setSuccessToastMessage, t, loadManagedSkills, loadTags, modal.activeSkillSource, setError])

  const handleBulkSync = useCallback(async () => {
    if (selectedSkillIds.length === 0) return
    setLoading(true)
    setLoadingStartAt(Date.now())
    try {
      const result = await skillService.bulkSyncSkills(selectedSkillIds)
      setSuccessToastMessage(t('bulk.synced', { count: result.synced }))
      setSelectedSkillIds([])
      setBulkMode(false)
      await loadManagedSkills()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err), err)
    } finally {
      setLoading(false)
      setLoadingStartAt(null)
    }
  }, [selectedSkillIds, setLoading, setLoadingStartAt, setSuccessToastMessage, t, loadManagedSkills, setError])

  const handleBulkSetTags = useCallback(async (tagIds: number[]) => {
    if (selectedSkillIds.length === 0) return
    setLoading(true)
    setLoadingStartAt(Date.now())
    try {
      await skillService.bulkSetSkillTags(selectedSkillIds, tagIds)
      setSuccessToastMessage(t('tagsUpdated'))
      setShowBulkTagsModal(false)
      setBulkTagIds([])
      setSelectedSkillIds([])
      setBulkMode(false)
      await loadManagedSkills()
      await loadTags(modal.activeSkillSource)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err), err)
    } finally {
      setLoading(false)
      setLoadingStartAt(null)
    }
  }, [selectedSkillIds, setLoading, setLoadingStartAt, setSuccessToastMessage, t, loadManagedSkills, loadTags, modal.activeSkillSource, setError])

  useEffect(() => {
    void loadTags(modal.activeSkillSource, filter.sortBy)
  }, [modal.activeSkillSource, loadTags, filter.sortBy])

  useEffect(() => {
    void loadManagedSkills(false, undefined, filter.sortBy)
  }, [filter.sortBy, loadManagedSkills])

  const importFlow = useImportFlow({
    t,
    tools: skills.tools,
    installedToolIds: skills.installedToolIds,
    isInstalled: skills.isInstalled,
    uniqueToolIdsBySkillsDir: skills.uniqueToolIdsBySkillsDir,
    sharedToolIdsByToolId: skills.sharedToolIdsByToolId,
    toolLabelById: skills.toolLabelById,
    loadManagedSkills: skills.loadManagedSkills,
    isSkillNameTaken: skills.isSkillNameTaken,
    showActionErrors: appState.showActionErrors,
    setError: appState.setError,
    setActionMessage: appState.setActionMessage,
    setSuccessToastMessage: appState.setSuccessToastMessage,
  })

  const scopeManager = useScopeManager({
    t,
    tools: skills.tools,
    installedToolIds: skills.installedToolIds,
    installedProjectToolIds: skills.installedProjectToolIds,
    toolSupportsProjectScope: skills.toolSupportsProjectScope,
    sharedToolIdsByToolId: skills.sharedToolIdsByToolId,
    toolLabelById: skills.toolLabelById,
    skillScopeState: scopeState.skillScopeState,
    setSkillScopeState: scopeState.setSkillScopeState,
    managedSkills: skills.managedSkills,
    loadManagedSkills: skills.loadManagedSkills,
    setError: appState.setError,
    setActionMessage: appState.setActionMessage,
    setSuccessToastMessage: appState.setSuccessToastMessage,
  })

  const addSkill = useAddSkill({
    t,
    tools: skills.tools,
    isInstalled: skills.isInstalled,
    uniqueToolIdsBySkillsDir: skills.uniqueToolIdsBySkillsDir,
    syncTargets: importFlow.syncTargets,
    setSyncTargets: importFlow.setSyncTargets,
    loadManagedSkills: skills.loadManagedSkills,
    loadTags: skills.loadTags,
    isSkillNameTaken: skills.isSkillNameTaken,
    showActionErrors: appState.showActionErrors,
    setError: appState.setError,
    loading,
    setLoading,
    setLoadingStartAt,
    setActionMessage: appState.setActionMessage,
    setSuccessToastMessage: appState.setSuccessToastMessage,
  })

  // ─── 合并 loading 状态 ──────────────────────────
  const globalLoading = loading || importFlow.loading || scopeManager.loading
  const globalLoadingStartAt =
    loadingStartAt || importFlow.loadingStartAt || scopeManager.loadingStartAt

  // ─── toolFilter 有效性 ──────────────────────────
  const { toolFilter, handleToolFilterChange } = filter

  useEffect(() => {
    if (toolFilter === 'all') return
    if (!skills.installedToolIds.includes(toolFilter)) {
      handleToolFilterChange('all')
    }
  }, [skills.installedToolIds, toolFilter, handleToolFilterChange])

  // ─── showNewToolsModal ──────────────────────────
  useEffect(() => {
    const loadStatus = async () => {
      const status = await skills.loadToolStatus()
      if (status && status.newly_installed.length > 0) {
        modal.setShowNewToolsModal(true)
      }
    }
    void loadStatus()
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  // ─── Deep Link handler (agentkit://) ──────────
  useEffect(() => {
    const handleDeepLink = (url: string) => {
      // agentkit://skill/{id} → 打开 skill 详情
      // agentkit://import → 打开导入界面
      const match = url.match(/^agentkit:\/\/([^/]+)(?:\/(.+))?/)
      if (!match) return
      const [, action, param] = match
      if (action === 'skill' && param) {
        const skill = skills.managedSkills.find((s) => s.id === param)
        if (skill) modal.openInfoModal(skill)
      } else if (action === 'import') {
        modal.handleViewChange('myskills')
      }
    }

    // 处理启动时传入的 deep link
    getCurrent()
      .then((urls) => {
        if (urls) {
          for (const url of urls) handleDeepLink(url)
        }
      })
      .catch(() => {})

    // 监听后续 deep link 事件
    const unlisten = onOpenUrl((urls) => {
      for (const url of urls) handleDeepLink(url)
    })

    return () => {
      void unlisten.then((fn) => fn())
    }
  }, [skills.managedSkills, modal])

  // ─── 计算衍生值 ────────────────────────────────
  const currentInfoModalSkill = useMemo(() => {
    if (!modal.infoModalSkill) return null
    return (
      skills.managedSkills.find((s) => s.id === modal.infoModalSkill!.id) ??
      modal.infoModalSkill
    )
  }, [modal.infoModalSkill, skills.managedSkills])

  const pendingDeleteSkill = useMemo(
    () =>
      modal.pendingDeleteId
        ? skills.managedSkills.find((s) => s.id === modal.pendingDeleteId) ?? null
        : null,
    [skills.managedSkills, modal.pendingDeleteId],
  )

  // ─── Tag & Skill actions (extracted hooks) ───────
  const {
    handleCreateTag,
    handleRenameTag,
    handleDeleteTag,
    handleCloseDeleteTag,
    handleConfirmDeleteTag,
  } = useTagActions({
    t,
    loadManagedSkills: skills.loadManagedSkills,
    loadTags: skills.loadTags,
    activeSkillSource: modal.activeSkillSource,
    setError: appState.setError,
    setSuccessToastMessage: appState.setSuccessToastMessage,
    setActionMessage: appState.setActionMessage,
    selectedTagIds: filter.selectedTagIds,
    setSelectedTagIds: filter.setSelectedTagIds,
    pendingDeleteTag: modal.pendingDeleteTag,
    setPendingDeleteTag: modal.setPendingDeleteTag,
    globalLoading,
    setLoading,
    setLoadingStartAt,
  })

  const {
    handleDeleteManaged,
    handleSaveSkillTags,
    handleCloseDelete,
  } = useSkillActions({
    t,
    loadManagedSkills: skills.loadManagedSkills,
    loadTags: skills.loadTags,
    activeSkillSource: modal.activeSkillSource,
    setError: appState.setError,
    setSuccessToastMessage: appState.setSuccessToastMessage,
    setActionMessage: appState.setActionMessage,
    setSkillScopeState: scopeState.setSkillScopeState,
    pendingDeleteId: modal.pendingDeleteId,
    setPendingDeleteId: modal.setPendingDeleteId,
    closeEditTags: modal.closeEditTags,
    globalLoading,
    setLoading,
    setLoadingStartAt,
  })

  // ─── Review import 触发 showImportModal ────────
  const handleReviewImport = useCallback(async () => {
    if (importFlow.plan) {
      modal.setShowImportModal(true)
      return
    }
    const result = await importFlow.loadPlan(true)
    if (result) {
      modal.setShowImportModal(true)
    }
  }, [importFlow, modal])

  // ─── Tags page nav ───────────────────────────────
  const handleOpenTagsPage = useCallback(() => {
    modal.handleViewChange('tags')
  }, [modal])

  const handleReviewUntagged = useCallback(() => {
    filter.setSelectedTagIds([])
    filter.setIncludeUntagged(true)
    modal.handleViewChange('myskills')
  }, [filter, modal])

  const handleViewTag = useCallback(
    (tagId: number) => {
      filter.setSelectedTagIds([tagId])
      filter.setIncludeUntagged(false)
      modal.handleViewChange('myskills')
    },
    [filter, modal],
  )

  const handleSyncAllNewTools = useCallback(() => {
    modal.setShowNewToolsModal(false)
  }, [modal])

  // ─── Render ──────────────────────────────────────
  return (
    <div className={`skills-app${sidebarCollapsed ? ' sidebar-collapsed' : ''}${sidebarCollapsed && sidebarHovered ? ' sidebar-hovered' : ''}`}>
      <Toaster position="top-right" richColors offset={48} toastOptions={{ duration: 1800 }} />
      <LoadingOverlay
        loading={globalLoading}
        actionMessage={appState.actionMessage}
        loadingStartAt={globalLoadingStartAt}
        onCancel={importFlow.handleCancelLoading}
        t={t}
      />

      <Header
        language={appState.language}
        activeView={modal.activeView}
        skillCount={skills.managedSkills.length}
        tagCount={skills.tags.length}
        toolCount={skills.installedTools.length}
        promptCount={promptCount}
        collapsed={sidebarCollapsed}
        onToggleCollapsed={handleToggleSidebar}
        onSidebarHoverEnter={handleSidebarEnter}
        onSidebarHoverLeave={handleSidebarLeave}
        onToggleLanguage={appState.toggleLanguage}
        onOpenSettings={modal.openSettings}
        onViewChange={modal.handleViewChange}
        updateVersion={updateResult?.update_available ? updateResult.latest_version : null}
        onOpenUpdateDialog={handleOpenUpdateDialog}
        t={t}
      />

      <main className="skills-main">
        <Suspense fallback={<div className="view-loading" />}>
        {modal.activeView === 'detail' && modal.detailSkill ? (
          <SkillDetailView
            skill={modal.detailSkill}
            onBack={modal.backToList}
            formatRelative={skills.formatRelative}
            t={t}
          />
        ) : modal.activeView === 'myskills' ? (
          <div className="dashboard-stack">
            <div className="dashboard-toolbar">
              <FilterBar
                sourceTabs={
                  <div className="source-tabs" role="tablist" aria-label={t('sourceTabs.label')}>
                    <button
                      className={`source-tab${modal.activeSkillSource === 'custom' ? ' active' : ''}`}
                      type="button"
                      role="tab"
                      aria-selected={modal.activeSkillSource === 'custom'}
                      onClick={() => {
                        modal.setActiveSkillSource('custom')
                        modal.backToList()
                      }}
                    >
                      {t('sourceTabs.custom')}
                      <span>{customSkillCount}</span>
                    </button>
                    <button
                      className={`source-tab${modal.activeSkillSource === 'community' ? ' active' : ''}`}
                      type="button"
                      role="tab"
                      aria-selected={modal.activeSkillSource === 'community'}
                      onClick={() => {
                        modal.setActiveSkillSource('community')
                        modal.backToList()
                      }}
                    >
                      {t('sourceTabs.community')}
                      <span>{communitySkillCount}</span>
                    </button>
                  </div>
                }
                sortBy={filter.sortBy}
                searchQuery={filter.searchQuery}
                scopeFilter={filter.scopeFilter}
                toolFilter={filter.toolFilter}
                installedTools={skills.installedTools}
                tags={skills.tags}
                selectedTagIds={filter.selectedTagIds}
                includeUntagged={filter.includeUntagged}
                untaggedCount={filter.untaggedCount}
                totalCount={filter.visibleSkills.length}
                refreshing={skills.refreshingSkills}
                loading={globalLoading}
                onSortChange={filter.handleSortChange}
                onSearchChange={filter.handleSearchChange}
                onScopeFilterChange={filter.handleScopeFilterChange}
                onToolFilterChange={filter.handleToolFilterChange}
                onRefresh={() => skills.handleRefreshSkills(modal.activeSkillSource)}
                onOpenAdd={() => addSkill.handleOpenAdd(modal.activeSkillSource)}
                onToggleTag={filter.handleToggleTagFilter}
                onToggleUntagged={filter.handleToggleUntaggedFilter}
                onClearTags={filter.handleClearTagFilters}
                onManageTags={handleOpenTagsPage}
                bulkMode={bulkMode}
                bulkSelectedCount={selectedSkillIds.length}
                viewMode={viewMode}
                onToggleBulkMode={handleToggleBulkMode}
                onViewModeChange={handleViewModeChange}
                t={t}
              />
            </div>
            <SkillsList
              plan={modal.activeSkillSource === 'community' ? importFlow.plan : null}
              visibleSkills={filter.visibleSkills}
              installedTools={skills.installedTools}
              loading={globalLoading}
              getSkillSourceLabel={skills.getSkillSourceLabel}
              formatRelative={skills.formatRelative}
              onReviewImport={handleReviewImport}
              onDeleteSkill={modal.setPendingDeleteId}
              onToggleTool={scopeManager.handleToggleToolForSkill}
              onOpenScope={scopeManager.handleOpenScope}
              onOpenDetail={modal.openInfoModal}
              onEditTags={modal.openEditTags}
              getSkillScope={getSkillScope}
              getSkillProjects={skills.getSkillProjects}
              draggable={filter.sortBy === 'manual'}
              onReorder={skills.reorderSkills}
              viewMode={viewMode}
              bulkMode={bulkMode}
              selectedSkillIds={selectedSkillIds}
              onToggleBulkSelection={handleToggleBulkSelection}
              onToggleEnabled={skills.handleToggleEnabled}
              t={t}
            />
            {bulkMode && selectedSkillIds.length > 0 ? (
              <div className="bulk-action-bar">
                <div className="bulk-action-copy">
                  <span>{t('bulk.selectedShort', { count: selectedSkillIds.length })}</span>
                </div>
                <div className="bulk-action-buttons">
                  <button
                    className="btn btn-secondary"
                    type="button"
                    onClick={() => void handleBulkSync()}
                    disabled={globalLoading}
                  >
                    <RefreshCw size={14} />
                    {t('bulk.sync')}
                  </button>
                  <button
                    className="btn btn-secondary"
                    type="button"
                    onClick={() => setShowBulkTagsModal(true)}
                    disabled={globalLoading}
                  >
                    <Tag size={14} />
                    {t('bulk.tags')}
                  </button>
                  <button
                    className="btn btn-danger"
                    type="button"
                    onClick={() => void handleBulkDelete()}
                    disabled={globalLoading}
                  >
                    <Trash2 size={14} />
                    {t('bulk.delete')}
                  </button>
                </div>
              </div>
            ) : null}
          </div>
        ) : modal.activeView === 'tags' ? (
          <TagsPage
            tags={skills.tags}
            untaggedCount={filter.untaggedCount}
            loading={globalLoading}
            formatRelative={skills.formatRelative}
            onReviewUntagged={handleReviewUntagged}
            onViewTag={handleViewTag}
            onCreateTag={handleCreateTag}
            onRenameTag={handleRenameTag}
            onDeleteTag={handleDeleteTag}
            onReorder={skills.reorderTags}
            t={t}
          />
        ) : modal.activeView === 'settings' ? (
          <SettingsPage
            language={appState.language}
            storagePath={theme.storagePath}
            customRepoPath={theme.customRepoPath}
            themePreference={theme.themePreference}
            onBack={() => modal.handleViewChange('myskills')}
            onPickStoragePath={theme.handlePickStoragePath}
            onPickCustomRepoPath={theme.handlePickCustomRepoPath}
            onOpenFolder={theme.handleOpenFolder}
            onResetDefaults={theme.handleResetDefaults}
            onSetLanguage={appState.setLanguage}
            onThemeChange={theme.handleThemeChange}
            t={t}
          />
        ) : modal.activeView === 'tools' ? (
          <ToolsPage t={t} />
        ) : modal.activeView === 'prompts' ? (
          <PromptsPage t={t} />
        ) : null}
        </Suspense>
      </main>

      {/* Modals – lazy-loaded & rendered only when needed */}
      <Suspense fallback={null}>
        {currentInfoModalSkill ? (
          <SkillInfoModal
            skill={currentInfoModalSkill}
            installedTools={skills.installedTools}
            loading={globalLoading}
            getSkillSourceLabel={skills.getSkillSourceLabel}
            formatRelative={skills.formatRelative}
            onRequestClose={modal.closeInfoModal}
            onViewFiles={modal.viewSkillFiles}
            onDelete={(skillId) => {
              modal.closeInfoModal()
              modal.setPendingDeleteId(skillId)
            }}
            onEditTags={(skill) => {
              modal.closeInfoModal()
              modal.openEditTags(skill)
            }}
            onOpenScope={(skill) => {
              modal.closeInfoModal()
              scopeManager.handleOpenScope(skill)
            }}
            getSkillScope={getSkillScope}
            getSkillProjects={skills.getSkillProjects}
            onUpdateSourceUrl={skills.handleUpdateSourceUrl}
            t={t}
          />
        ) : null}

        {addSkill.showAddModal ? (
          <AddSkillModal
            open={addSkill.showAddModal}
            loading={globalLoading}
            canClose={!globalLoading}
            localPath={addSkill.localPath}
            localName={addSkill.localName}
            sourceType={addSkill.addSourceType}
            tags={skills.tags}
            selectedTagIds={addSkill.addModalTagIds}
            syncTargets={importFlow.syncTargets}
            installedTools={skills.installedTools}
            toolStatus={skills.toolStatus}
            onRequestClose={addSkill.handleCloseAdd}
            onLocalPathChange={addSkill.setLocalPath}
            onPickLocalPath={addSkill.handlePickLocalPath}
            onLocalNameChange={addSkill.setLocalName}
            onSourceTypeChange={addSkill.setAddSourceType}
            onToggleTag={addSkill.handleToggleAddModalTag}
            onSyncTargetChange={importFlow.handleSyncTargetChange}
            onSubmit={addSkill.handleCreateLocal}
            t={t}
          />
        ) : null}

        {modal.tagEditorSkill ? (
          <EditSkillTagsModal
            key={`${modal.tagEditorSkill.id}-${modal.tagEditorSkill.tags.map((tag) => tag.id).join('-')}`}
            open={Boolean(modal.tagEditorSkill)}
            loading={globalLoading}
            skill={
              skills.managedSkills.find((s) => s.id === modal.tagEditorSkill!.id) ?? modal.tagEditorSkill
            }
            tags={skills.tags}
            onRequestClose={modal.closeEditTags}
            onSave={handleSaveSkillTags}
            t={t}
          />
        ) : null}

        {modal.showImportModal && importFlow.plan ? (
          <ImportModal
            open={modal.showImportModal}
            loading={globalLoading}
            plan={importFlow.plan}
            selected={importFlow.selected}
            variantChoice={importFlow.variantChoice}
            storagePath={theme.storagePath}
            onRequestClose={() => {
              if (!globalLoading) modal.setShowImportModal(false)
            }}
            onToggleGroup={importFlow.handleToggleGroup}
            onSelectVariant={importFlow.handleSelectVariant}
            onImport={async () => {
              const ok = await importFlow.handleImport()
              if (ok) modal.setShowImportModal(false)
            }}
            t={t}
          />
        ) : null}

        {scopeManager.pendingSharedToggle ? (
          <SharedDirModal
            open={Boolean(scopeManager.pendingSharedToggle)}
            loading={globalLoading}
            toolLabel={scopeManager.pendingSharedLabels?.toolLabel ?? ''}
            otherLabels={scopeManager.pendingSharedLabels?.otherLabels ?? ''}
            onRequestClose={scopeManager.handleSharedCancel}
            onConfirm={scopeManager.handleSharedConfirm}
            t={t}
          />
        ) : null}

        {scopeManager.suiteSyncState ? (
          <SuiteSyncModal
            open={Boolean(scopeManager.suiteSyncState)}
            loading={globalLoading}
            toolLabel={scopeManager.suiteSyncToolLabel}
            subSkills={scopeManager.suiteSyncState.subSkills}
            loadingSubSkills={scopeManager.suiteSyncState.loadingSubSkills}
            onRequestClose={scopeManager.handleSuiteSyncClose}
            onConfirm={scopeManager.handleSuiteSyncConfirm}
            t={t}
          />
        ) : null}

        {scopeManager.currentScopeModalSkill ? (
          <ScopeSyncModal
            key={`${scopeManager.currentScopeModalSkill.id}-${getSkillScope(scopeManager.currentScopeModalSkill)}`}
            open={Boolean(scopeManager.currentScopeModalSkill)}
            loading={globalLoading}
            skill={scopeManager.currentScopeModalSkill}
            scope={getSkillScope(scopeManager.currentScopeModalSkill)}
            projects={skills.getSkillProjects(scopeManager.currentScopeModalSkill)}
            recentProjects={scopeManager.recentProjects}
            onRequestClose={scopeManager.handleCloseScope}
            onScopeChange={scopeManager.handleScopeChange}
            onPickProject={scopeManager.handlePickProject}
            t={t}
          />
        ) : null}

        {modal.showNewToolsModal && skills.newlyInstalledToolsText ? (
          <NewToolsModal
            open={Boolean(modal.showNewToolsModal && skills.newlyInstalledToolsText)}
            loading={globalLoading}
            toolsLabelText={skills.newlyInstalledToolsText}
            onDismiss={handleSyncAllNewTools}
            t={t}
          />
        ) : null}

        {modal.pendingDeleteId ? (
          <DeleteModal
            open={Boolean(modal.pendingDeleteId)}
            loading={globalLoading}
            skillName={pendingDeleteSkill?.name ?? null}
            onRequestClose={handleCloseDelete}
            onConfirm={() => {
              if (pendingDeleteSkill) void handleDeleteManaged(pendingDeleteSkill)
            }}
            t={t}
          />
        ) : null}
      </Suspense>

      {/* Inline tag delete modal (simple, no lazy needed) */}
      {modal.pendingDeleteTag ? (
        <div
          className="modal-backdrop"
          onClick={globalLoading ? undefined : handleCloseDeleteTag}
        >
          <div
            className="modal modal-delete tag-delete-modal"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="modal-header">
              <div className="modal-title">{t('deleteTagTitle')}</div>
              <button
                className="modal-close"
                type="button"
                onClick={handleCloseDeleteTag}
                disabled={globalLoading}
              >
                {'×'}
              </button>
            </div>
            <div className="modal-body tag-delete-body">
              {t('deleteTagConfirm', {
                name: modal.pendingDeleteTag.name,
                count: modal.pendingDeleteTag.skill_count,
              })}
            </div>
            <div className="modal-footer">
              <button
                className="btn btn-secondary"
                type="button"
                onClick={handleCloseDeleteTag}
                disabled={globalLoading}
              >
                {t('cancel')}
              </button>
              <button
                className="btn btn-danger"
                type="button"
                onClick={() => void handleConfirmDeleteTag()}
                disabled={globalLoading}
              >
                {t('deleteAction')}
              </button>
            </div>
          </div>
        </div>
      ) : null}

      {/* Inline bulk tags modal */}
      {showBulkTagsModal ? (
        <div
          className="modal-backdrop"
          onClick={globalLoading ? undefined : () => setShowBulkTagsModal(false)}
        >
          <div
            className="modal bulk-tags-modal"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="modal-header">
              <div className="modal-title">{t('bulk.tags')}</div>
              <button
                className="modal-close"
                type="button"
                onClick={() => setShowBulkTagsModal(false)}
                disabled={globalLoading}
              >
                {'×'}
              </button>
            </div>
            <div className="modal-body">
              <div className="add-tags-list">
                {skills.tags.length === 0 ? (
                  <span className="table-empty">{t('noTags')}</span>
                ) : (
                  skills.tags.map((tag) => (
                    <button
                      key={tag.id}
                      type="button"
                      className={`add-tag-pill${bulkTagIds.includes(tag.id) ? ' selected' : ''}`}
                      onClick={() => handleToggleBulkTag(tag.id)}
                    >
                      {tag.name}
                    </button>
                  ))
                )}
              </div>
            </div>
            <div className="modal-footer">
              <button
                className="btn btn-secondary"
                type="button"
                onClick={() => setShowBulkTagsModal(false)}
                disabled={globalLoading}
              >
                {t('cancel')}
              </button>
              <button
                className="btn btn-primary"
                type="button"
                onClick={() => void handleBulkSetTags(bulkTagIds)}
                disabled={globalLoading}
              >
                {t('apply')}
              </button>
            </div>
          </div>
        </div>
      ) : null}

      <Suspense fallback={null}>
        {addSkill.showLocalPickModal ? (
          <LocalPickModal
            open={addSkill.showLocalPickModal}
            loading={globalLoading}
            localCandidates={addSkill.localCandidates}
            localCandidateSelected={addSkill.localCandidateSelected}
            onRequestClose={addSkill.handleCloseLocalPick}
            onCancel={addSkill.handleCancelLocalPick}
            onToggleCandidate={addSkill.handleToggleLocalCandidate}
            onInstall={addSkill.handleInstallSelectedLocalCandidates}
            t={t}
          />
        ) : null}
      </Suspense>

      {/* Update dialog (triggered by titlebar badge) */}
      <UpdateDialog
        open={updateDialogOpen}
        result={updateResult}
        t={t}
        onClose={handleCloseUpdateDialog}
      />
    </div>
  )
}

export default App
