import { memo, useEffect, useId, useMemo, useRef, useState, type ReactNode } from 'react'
import { ArrowUpDown, Check, CheckSquare, ChevronDown, Globe, LayoutGrid, List, Monitor, Plus, RefreshCw, Search, Tags, X } from '@/components/icons'
import type { TFunction } from 'i18next'
import type { TagWithCountDto, ToolOption } from '@/features/skills/types'

type FilterBarProps = {
  sourceTabs: ReactNode
  sortBy: 'manual' | 'updated' | 'name'
  searchQuery: string
  scopeFilter: 'all' | 'global' | 'project'
  toolFilter: string
  installedTools: ToolOption[]
  tags: TagWithCountDto[]
  selectedTagIds: number[]
  includeUntagged: boolean
  untaggedCount: number
  totalCount: number
  refreshing: boolean
  loading: boolean
  onSortChange: (value: 'manual' | 'updated' | 'name') => void
  onSearchChange: (value: string) => void
  onScopeFilterChange: (value: 'all' | 'global' | 'project') => void
  onToolFilterChange: (value: string) => void
  onRefresh: () => void
  onOpenAdd: () => void
  onToggleTag: (tagId: number) => void
  onToggleUntagged: () => void
  onClearTags: () => void
  onManageTags: () => void
  bulkMode: boolean
  bulkSelectedCount: number
  viewMode: 'list' | 'cards'
  onToggleBulkMode: () => void
  onViewModeChange: (value: 'list' | 'cards') => void
  t: TFunction
}

const FilterBar = ({
  sourceTabs,
  sortBy,
  searchQuery,
  scopeFilter,
  toolFilter,
  installedTools,
  tags,
  selectedTagIds,
  includeUntagged,
  untaggedCount,
  totalCount,
  refreshing,
  loading,
  onSortChange,
  onSearchChange,
  onScopeFilterChange,
  onToolFilterChange,
  onRefresh,
  onOpenAdd,
  onToggleTag,
  onToggleUntagged,
  onClearTags,
  onManageTags,
  bulkMode,
  bulkSelectedCount,
  viewMode,
  onToggleBulkMode,
  onViewModeChange,
  t,
}: FilterBarProps) => {
  const [tagMenuOpen, setTagMenuOpen] = useState(false)
  const [tagQuery, setTagQuery] = useState('')
  const tagMenuRef = useRef<HTMLDivElement | null>(null)
  const tagMenuId = useId()
  const scopeOptions: { value: 'all' | 'global' | 'project'; label: string }[] = [
    { value: 'all', label: t('scope.allLabel') },
    { value: 'global', label: t('scope.globalLabel') },
    { value: 'project', label: t('scope.projectLabel') },
  ]
  const selectedTagSet = useMemo(() => new Set(selectedTagIds), [selectedTagIds])
  const selectedCount = selectedTagIds.length + (includeUntagged ? 1 : 0)
  const filteredTags = useMemo(() => {
    const query = tagQuery.trim().toLowerCase()
    if (!query) return tags
    return tags.filter((tag) => tag.name.toLowerCase().includes(query))
  }, [tagQuery, tags])
  const selectedTool = installedTools.find((tool) => tool.id === toolFilter)
  const title = selectedTool
    ? t('toolFilter.skillsTitle', { tool: selectedTool.label })
    : t('allSkills')

  useEffect(() => {
    if (!tagMenuOpen) return
    const handlePointerDown = (event: Event) => {
      if (!tagMenuRef.current?.contains(event.target as Node)) {
        setTagMenuOpen(false)
      }
    }
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setTagMenuOpen(false)
        tagMenuRef.current?.querySelector('button')?.focus()
      }
    }
    tagMenuRef.current?.querySelector('input')?.focus()
    document.addEventListener('mousedown', handlePointerDown)
    document.addEventListener('focusin', handlePointerDown)
    document.addEventListener('keydown', handleKeyDown)
    return () => {
      document.removeEventListener('mousedown', handlePointerDown)
      document.removeEventListener('focusin', handlePointerDown)
      document.removeEventListener('keydown', handleKeyDown)
    }
  }, [tagMenuOpen])

  return (
    <div className="filter-bar">
      {/* 标题与主操作 */}
      <div className="filter-row filter-row-top">
        <div className="filter-heading">
          <h1 className="filter-title">
            {title}<span className="filter-count">{totalCount}</span>
          </h1>
          <p className="filter-description">{t('workspace.description')}</p>
        </div>
        <div className="filter-primary-actions">
          <button
            className="btn btn-secondary refresh-btn"
            type="button"
            onClick={onRefresh}
            disabled={refreshing || loading}
            title={t('refreshSkills')}
            aria-label={t('refreshSkills')}
          >
            <RefreshCw size={14} className={refreshing ? 'spin' : undefined} />
            {refreshing ? t('refreshing') : t('refresh')}
          </button>
          <button className="btn btn-primary" type="button" onClick={onOpenAdd} disabled={loading}>
            <Plus size={14} />
            {t('newSkill')}
          </button>
        </div>
      </div>
      <div className="filter-row filter-row-search">
        {sourceTabs}
        <div className="filter-search-wrap">
          <div className="search-container">
            <Search size={16} className="search-icon-abs" />
            <input
              className="search-input"
              value={searchQuery}
              onChange={(event) => onSearchChange(event.target.value)}
              placeholder={t('searchPlaceholder')}
              aria-label={t('searchPlaceholder')}
            />
            {searchQuery ? (
              <button
                className="search-clear"
                type="button"
                onClick={() => onSearchChange('')}
                aria-label={t('workspace.clearSearch')}
                title={t('workspace.clearSearch')}
              >
                <X size={14} />
              </button>
            ) : null}
          </div>
        </div>
      </div>
      {/* 第二行：筛选器 + 排序 + 视图切换 */}
      <div className="filter-row filter-row-bottom">
        <div className="filter-secondary-actions">
          <label className="btn btn-secondary sort-btn tool-filter-btn">
            <Monitor size={14} />
            {selectedTool?.label ?? t('toolFilter.all')}
            <ChevronDown size={12} />
            <select
              aria-label={t('toolFilter.label')}
              value={toolFilter}
              onChange={(event) => onToolFilterChange(event.target.value)}
            >
              <option value="all">{t('toolFilter.all')}</option>
              {installedTools.map((tool) => (
                <option key={tool.id} value={tool.id}>
                  {tool.label}
                </option>
              ))}
            </select>
          </label>
          <label className="btn btn-secondary sort-btn">
            <Globe size={14} />
            {scopeOptions.find((option) => option.value === scopeFilter)?.label ?? t('scope.allLabel')}
            <ChevronDown size={12} />
            <select
              aria-label={t('scope.filterLabel')}
              value={scopeFilter}
              onChange={(event) =>
                onScopeFilterChange(event.target.value as 'all' | 'global' | 'project')
              }
            >
              {scopeOptions.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </label>
          <label className="btn btn-secondary sort-btn">
            {sortBy === 'manual' ? t('sortManual') : sortBy === 'updated' ? t('sortUpdated') : t('sortName')}
            <ArrowUpDown size={12} />
            <select
              aria-label={t('filterSort')}
              value={sortBy}
              onChange={(event) => onSortChange(event.target.value as 'manual' | 'updated' | 'name')}
            >
              <option value="manual">{t('sortManual')}</option>
              <option value="updated">{t('sortUpdated')}</option>
              <option value="name">{t('sortName')}</option>
            </select>
          </label>
          <div className="tag-filter-wrap" ref={tagMenuRef}>
            <button
              className={`btn btn-secondary tag-filter-btn${selectedCount > 0 ? ' active' : ''}`}
              type="button"
              onClick={() => setTagMenuOpen((open) => !open)}
              aria-expanded={tagMenuOpen}
              aria-controls={tagMenuOpen ? tagMenuId : undefined}
            >
              <Tags size={14} />
              {selectedCount > 0
                ? t('tagsSelected', { count: selectedCount })
                : t('tags')}
              <ChevronDown size={12} />
            </button>
            {tagMenuOpen ? (
              <div className="tag-filter-menu" id={tagMenuId} role="group" aria-label={t('tags')}>
                <div className="tag-filter-head">
                  <span>{t('tags')}</span>
                  <span>{t('matchAny')}</span>
                </div>
                <div className="tag-filter-search">
                  <Search size={15} />
                  <input
                    value={tagQuery}
                    onChange={(event) => setTagQuery(event.target.value)}
                    placeholder={t('searchTags')}
                    aria-label={t('searchTags')}
                  />
                </div>
                <div className="tag-filter-options">
                  <button
                    className={`tag-filter-option${includeUntagged ? ' selected' : ''}`}
                    type="button"
                    onClick={onToggleUntagged}
                    aria-pressed={includeUntagged}
                  >
                    <span className="tag-check">{includeUntagged ? <Check size={14} /> : null}</span>
                    <span>{t('untagged')}</span>
                    <span className="tag-count">{untaggedCount}</span>
                  </button>
                  {filteredTags.map((tag) => {
                    const selected = selectedTagSet.has(tag.id)
                    return (
                      <button
                        key={tag.id}
                        className={`tag-filter-option${selected ? ' selected' : ''}`}
                        type="button"
                        onClick={() => onToggleTag(tag.id)}
                        aria-pressed={selected}
                      >
                        <span className="tag-check">{selected ? <Check size={14} /> : null}</span>
                        <span>{tag.name}</span>
                        <span className="tag-count">{tag.skill_count}</span>
                      </button>
                    )
                  })}
                </div>
                <div className="tag-filter-footer">
                  <button type="button" onClick={onClearTags} disabled={selectedCount === 0}>
                    {t('clearAll')}
                  </button>
                  <button type="button" onClick={onManageTags}>
                    {t('manageTags')}
                  </button>
                </div>
              </div>
            ) : null}
          </div>
          <button
            className={`btn btn-secondary bulk-mode-btn${bulkMode ? ' active' : ''}`}
            type="button"
            onClick={onToggleBulkMode}
            aria-pressed={bulkMode}
          >
            <CheckSquare size={14} />
            {bulkMode ? t('bulk.selectedShort', { count: bulkSelectedCount }) : t('bulk.manage')}
          </button>
        </div>
        <div className="filter-right-actions">
          <div className="view-mode-toggle" role="group" aria-label={t('viewMode.label')}>
            <button
              className={viewMode === 'list' ? 'active' : ''}
              type="button"
              onClick={() => onViewModeChange('list')}
              aria-label={t('viewMode.list')}
              title={t('viewMode.list')}
              aria-pressed={viewMode === 'list'}
            >
              <List size={15} />
            </button>
            <button
              className={viewMode === 'cards' ? 'active' : ''}
              type="button"
              onClick={() => onViewModeChange('cards')}
              aria-label={t('viewMode.cards')}
              title={t('viewMode.cards')}
              aria-pressed={viewMode === 'cards'}
            >
              <LayoutGrid size={15} />
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

export default memo(FilterBar)
