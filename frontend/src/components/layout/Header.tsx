import { memo } from 'react'
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
  /** New version available to show the red badge button; null/undefined when up-to-date */
  updateVersion: string | null
  onOpenUpdateDialog: () => void
  t: TFunction
}

const appWindow = getCurrentWindow()

const handleMinimize = () => {
  console.info('[Window] 用户点击最小化按钮')
  appWindow.minimize().catch((err) => {
    console.error('[Window] 最小化窗口失败:', err)
  })
}
const handleToggleMaximize = () => {
  console.info('[Window] 用户点击最大化/还原按钮')
  appWindow.toggleMaximize().catch((err) => {
    console.error('[Window] 切换最大化状态失败:', err)
  })
}
const handleClose = () => {
  console.info('[Window] 用户点击关闭按钮')
  appWindow.close().catch((err) => {
    console.error('[Window] 关闭窗口失败:', err)
  })
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
  updateVersion,
  onOpenUpdateDialog,
  t,
}: HeaderProps) => (
  <>
    <div
      className="window-titlebar"
      data-tauri-drag-region
    >
      <strong className="titlebar-title" data-tauri-drag-region>
        {t('appName')}
      </strong>
      <div className="titlebar-right titlebar-no-drag">
        {updateVersion ? (
          <button
            type="button"
            className="titlebar-update-btn"
            onClick={onOpenUpdateDialog}
            title={t('update.newVersion')}
            aria-label={t('update.newVersion')}
          >
            v{updateVersion}
          </button>
        ) : (
          <span className="titlebar-version">
            v{__APP_VERSION__}
          </span>
        )}
        <div className="titlebar-window-controls">
          <button
            type="button"
            onClick={handleMinimize}
            aria-label={t('window.minimize')}
            title={t('window.minimize')}
          >
            <Minus size={16} />
          </button>
          <button
            type="button"
            onClick={handleToggleMaximize}
            aria-label={t('window.maximize')}
            title={t('window.maximize')}
          >
            <Square size={13} />
          </button>
          <button
            type="button"
            className="close"
            onClick={handleClose}
            aria-label={t('window.close')}
            title={t('window.close')}
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
