import { memo, type PointerEvent } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  ChevronLeft,
  FileText,
  Languages,
  Layers,
  Minus,
  Monitor,
  Settings,
  Square,
  Tag,
  X,
} from 'lucide-react'
import type { TFunction } from 'i18next'
import logoLight from '@/assets/logo.svg'
import logoDark from '@/assets/logo-dark.svg'

type HeaderProps = {
  language: string
  activeView: 'myskills' | 'detail' | 'settings' | 'tags' | 'tools' | 'prompts'
  skillCount: number
  tagCount: number
  toolCount: number
  collapsed: boolean
  onToggleCollapsed: () => void
  onSidebarHoverEnter?: () => void
  onSidebarHoverLeave?: () => void
  onToggleLanguage: () => void
  onOpenSettings: () => void
  onViewChange: (view: 'myskills' | 'tags' | 'tools' | 'prompts') => void
  t: TFunction
}

const appWindow = getCurrentWindow()

const startWindowDrag = (event: PointerEvent<HTMLElement>) => {
  // Only start dragging on primary button (left click), and skip if the
  // event target is (or is inside) an interactive element such as a button.
  if (event.button !== 0) return
  const target = event.target as HTMLElement
  if (target.closest('button, input, select, textarea, a, [role="button"]')) return
  void appWindow.startDragging().catch(() => undefined)
}

const Header = ({
  language,
  activeView,
  skillCount,
  tagCount,
  toolCount,
  collapsed,
  onToggleCollapsed,
  onSidebarHoverEnter,
  onSidebarHoverLeave,
  onToggleLanguage,
  onOpenSettings,
  onViewChange,
  t,
}: HeaderProps) => (
  <>
    {/* Left-edge hover hitbox (only present when collapsed) */}
    {collapsed && (
      <div
        className="sidebar-hover-hitzone"
        onMouseEnter={onSidebarHoverEnter}
        onMouseLeave={onSidebarHoverLeave}
        aria-hidden
      />
    )}
    <div
      className="window-titlebar"
      onPointerDown={startWindowDrag}
    >
      <strong className="titlebar-title">
        {t('appName')}
      </strong>
      <div className="titlebar-right">
        <span className="titlebar-version">
          v{__APP_VERSION__}
        </span>
        <div className="titlebar-window-controls">
          <button
            type="button"
            onClick={(e) => { e.stopPropagation(); void appWindow.minimize() }}
            aria-label={t('window.minimize')}
          >
            <Minus size={16} />
          </button>
          <button
            type="button"
            onClick={(e) => { e.stopPropagation(); void appWindow.toggleMaximize() }}
            aria-label={t('window.maximize')}
          >
            <Square size={13} />
          </button>
          <button
            type="button"
            className="close"
            onClick={(e) => { e.stopPropagation(); void appWindow.close() }}
            aria-label={t('window.close')}
          >
            <X size={16} />
          </button>
        </div>
      </div>
    </div>
    <aside
      className={`skills-sidebar${collapsed ? ' collapsed' : ''}`}
      onMouseEnter={collapsed ? onSidebarHoverEnter : undefined}
      onMouseLeave={collapsed ? onSidebarHoverLeave : undefined}
    >
      <div className="sidebar-brand">
        <div className="sidebar-logo">
          <img
            className="brand-logo brand-logo-light"
            src={logoLight}
            alt="Skills Hub"
            width={32}
            height={32}
          />
          <img
            className="brand-logo brand-logo-dark"
            src={logoDark}
            alt="Skills Hub"
            width={32}
            height={32}
          />
        </div>
        <div className="sidebar-brand-copy">
          <strong>{t('appName')}</strong>
          <span>{t('subtitle')}</span>
        </div>
        <button
          className="sidebar-collapse"
          type="button"
          onClick={onToggleCollapsed}
          aria-label={collapsed ? t('sidebar.expand') : t('sidebar.collapse')}
          title={collapsed ? t('sidebar.expand') : t('sidebar.collapse')}
        >
          <ChevronLeft size={16} />
        </button>
      </div>

      <div className="sidebar-section-label">{t('sidebar.workspace')}</div>
      <nav className="sidebar-nav" aria-label={t('sidebar.workspace')}>
        <button
          className={activeView === 'myskills' || activeView === 'detail' ? 'active' : ''}
          type="button"
          onClick={() => onViewChange('myskills')}
          title={collapsed ? t('navMySkills') : undefined}
        >
          <Layers size={18} />
          <span>{t('navMySkills')}</span>
          <em>{skillCount}</em>
        </button>
        <button
          className={activeView === 'prompts' ? 'active' : ''}
          type="button"
          onClick={() => onViewChange('prompts')}
          title={collapsed ? t('navPrompts') : undefined}
        >
          <FileText size={18} />
          <span>{t('navPrompts')}</span>
        </button>
      </nav>

      <div className="sidebar-section-label">{t('sidebar.manage')}</div>
      <nav className="sidebar-nav" aria-label={t('sidebar.manage')}>
        <button
          className={activeView === 'tags' ? 'active' : ''}
          type="button"
          onClick={() => onViewChange('tags')}
          title={collapsed ? t('navTags') : undefined}
        >
          <Tag size={18} />
          <span>{t('navTags')}</span>
          <em>{tagCount}</em>
        </button>
        <button
          className={activeView === 'tools' ? 'active' : ''}
          type="button"
          onClick={() => onViewChange('tools')}
          title={collapsed ? t('navTools') : undefined}
        >
          <Monitor size={18} />
          <span>{t('navTools')}</span>
          <em>{toolCount}</em>
        </button>
      </nav>

      <div className="sidebar-spacer" />
      <div className="sidebar-footer">
        <button type="button" onClick={onToggleLanguage} title={collapsed ? t('settings.language') : undefined}>
          <Languages size={18} />
          <span>{language === 'en' ? t('languageShort.en') : t('languageShort.zh')}</span>
        </button>
        <button
          className={activeView === 'settings' ? 'active' : ''}
          type="button"
          onClick={onOpenSettings}
          title={collapsed ? t('settings.title') : undefined}
        >
          <Settings size={18} />
          <span>{t('settings.title')}</span>
        </button>
      </div>
    </aside>
  </>
)

export default memo(Header)
