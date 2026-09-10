# 主题与样式规范

> 本文件描述前端 CSS 变量系统、主题切换机制、模块化样式架构和浅色/暗色模式开发策略。修改样式后同步更新。

## 1. 核心原则

**浅色工作台为主要设计方向，使用蓝色强调和清晰的操作层级，同时兼顾深色主题。**

## 2. 开发优先级

- ✅ **优先开发浅色模式**：所有新功能、新组件首先在浅色模式下完成开发和测试
- ✅ **浅色模式精细化优化**：确保浅色模式的视觉效果、交互体验完美
- ⚠️ **暗色模式基础兼容**：仅保证暗色模式下基础功能可用，不做精细化优化

## 3. 样式文件架构

### 3.1 两个 CSS 入口

样式系统由两个独立入口组成，均在 `main.tsx` 中导入：

```typescript
// main.tsx
import './index.css'         // ① CSS 变量 + Tailwind + 全局 reset
import './styles/index.css'  // ② 模块化组件样式（@import 聚合）
```

### 3.2 入口 ①：`src/index.css`

**职责：** CSS 变量定义 + Tailwind 导入 + 全局 reset。不包含组件样式。

| 内容 | 说明 |
|------|------|
| 字体导入 | Google Fonts: Inter + IBM Plex Mono |
| `@import "tailwindcss"` | Tailwind CSS 4 |
| `:root` | 浅色模式 CSS 变量（默认） |
| `:root[data-theme='dark']` | 暗色模式 CSS 变量覆盖 |
| `*` / `body` / `#root` / `a` | 全局 reset + 工作台背景 |

### 3.3 入口 ②：`src/styles/index.css`

**职责：** 纯聚合入口，通过 `@import` 导入所有模块化 CSS 文件：

```css
/* Styles entry point — imports all modular CSS files */
@import './app.css';
@import './buttons.css';
@import './skill-card.css';
@import './modal.css';
@import './modals.css';
@import './skill-detail-modal.css';
@import './markdown.css';
@import './database.css';
@import './explore.css';
@import './skill-detail-view.css';
@import './tags-page.css';
@import './tools-page.css';
@import './settings.css';
```

### 3.4 模块化样式文件

| 文件 | 职责 |
|------|------|
| `styles/app.css` | App 外壳 + Header + 导航 + 筛选栏 |
| `styles/buttons.css` | 按钮系统（btn-primary/secondary/danger/icon/card-btn） |
| `styles/skill-card.css` | 技能卡片 + 拖拽 + 工具矩阵 + scope badge |
| `styles/modal.css` | Modal 基础设施 + 通用弹窗样式 |
| `styles/modals.css` | 专用弹窗（删除/scope/加载/发现/导入/工具配置） |
| `styles/skill-detail-modal.css` | 技能详情/信息弹窗 |
| `styles/explore.css` | Explore 全页（hero/搜索/卡片网格） |
| `styles/skill-detail-view.css` | Skill 详情视图（文件树/Markdown/元数据） |
| `styles/tags-page.css` | Tags 管理页（hero/表格/token） |
| `styles/tools-page.css` | Tools 管理页（工具卡片/配置弹窗） |
| `styles/settings.css` | 设置页 v2（CC Switch 风格 Tab/segmented） |
| `styles/database.css` | 数据库面板（概览/表格/维护/详情弹窗） |
| `styles/markdown.css` | Markdown 渲染样式 |
| `styles/prompts-page.css` | 提示词列表与编辑面板 |

### 3.5 样式规则

- **按功能模块拆分**：每个功能模块一个 CSS 文件，由 `styles/index.css` 统一 `@import` 聚合
- **禁止**在组件目录下创建独立 CSS 文件
- **禁止**创建 `src/App.css` 等集中式样式文件
- **CSS 类名使用 kebab-case**，前缀与模块名保持一致（如 `.skill-card` 对应 `skill-card.css`，`.settings-v2-*` 对应 `settings.css`）
- **同一选择器只定义一次**：禁止后写覆盖模式
- **新增组件样式**：添加到对应的功能模块 CSS 文件中
- **公共/复用样式**：放入 `buttons.css` 等共享模块

## 4. CSS 变量系统

### 4.1 浅色模式（`:root`，默认）

| 类别 | 变量名 | 值 |
|------|--------|-----|
| **背景** | `--bg-app` | `#f5f7fb` |
| | `--bg-panel` | `#ffffff` |
| | `--bg-element` | `#edf1f7` |
| | `--bg-element-hover` | `#e4eaf3` |
| | `--bg-header` | `rgba(248, 250, 253, 0.96)` |
| | `--bg-badge` | `#edf1f7` |
| | `--bg-hover` | `rgba(37, 56, 88, 0.045)` |
| **边框** | `--border-subtle` | `#e1e7f0` |
| | `--border-strong` | `#c9d4e4` |
| | `--border-faint` | `#edf1f6` |
| | `--border-hover` | `#bfcde0` |
| **文字** | `--text-primary` | `#202d43` |
| | `--text-secondary` | `#586980` |
| | `--text-tertiary` | `#738298` |
| **强调色** | `--accent-primary` | `#3269db` |
| | `--accent-primary-hover` | `#2556bd` |
| | `--accent-primary-fg` | `#ffffff` |
| | `--accent-soft-bg` | `#eaf1ff` |
| | `--accent-soft-border` | `#d3e1fc` |
| **状态色** | `--status-success` | `#08795a` |
| | `--status-warning` | `#9a6208` |
| | `--status-error` | `#dc2626` |
| | `--status-info` | `#2563eb` |
| **软背景** | `--success-soft-bg` | `#ecfdf5` |
| | `--success-soft-border` | `#bbf7d0` |
| | `--warning-soft-bg` | `#fffbeb` |
| | `--warning-soft-border` | `#fcd34d` |
| | `--danger-soft-bg` | `#fef2f2` |
| | `--danger-soft-border` | `#fee2e2` |
| | `--danger-soft-bg-strong` | `#fff1f2` |
| **字体** | `--font-ui` | `Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", "Microsoft YaHei UI", "PingFang SC", sans-serif` |
| | `--font-mono` | `"IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace` |
| **项目作用域色** | `--accent-project` | `#2563eb` |
| | `--accent-project-hover` | `#1d4ed8` |
| | `--accent-project-soft-bg` | `rgba(37, 99, 235, 0.08)` |
| | `--accent-project-soft-border` | `rgba(37, 99, 235, 0.35)` |
| **品牌色** | `--brand-accent` | `#3269db` |
| **别名** | `--text-muted` | `var(--text-tertiary)` |
| | `--success` | `var(--status-success)` |
| | `--danger` | `var(--status-error)` |
| **强调阴影** | `--accent-primary-shadow` | `rgba(47, 111, 237, 0.2)` |
| | `--accent-primary-shadow-strong` | `rgba(47, 111, 237, 0.3)` |
| **字号层级** | `--text-2xs` ~ `--text-4xl` | `10px` ~ `24px`（10 级） |
| **行高** | `--leading-tight` / `--leading-snug` / `--leading-normal` / `--leading-relaxed` | `1.2` / `1.35` / `1.5` / `1.6` |
| **圆角** | `--radius-sm` / `--radius-md` / `--radius-lg` / `--radius-xl` | `6px` / `10px` / `14px` / `18px` |
| **阴影** | `--shadow-xs` / `--shadow-sm` / `--shadow-md` / `--shadow-lg` | 4 级递进 |
| **过渡** | `--transition-fast` / `--transition-base` / `--transition-slow` | `0.15s` / `0.2s` / `0.3s` |
| | `--ease-out` / `--ease-spring` | `cubic-bezier(0.4, 0, 0.2, 1)` / `cubic-bezier(0.34, 1.56, 0.64, 1)` |
| **遮罩** | `--overlay-bg` | `rgba(24, 38, 60, 0.3)` |
| **开关滑块** | `--control-thumb` | `#ffffff` |

### 4.2 暗色模式（`:root[data-theme='dark']`，覆盖）

暗色模式覆盖颜色和阴影变量，字号、圆角与过渡沿用默认值。关键差异：

- 背景使用蓝灰色而非纯黑（`#10151e` / `#18212e`）
- 主文字使用冷白色（`#edf2fa`）
- 强调色偏亮（`#6f9cff`），前景色为深色（`#111214`）
- 遮罩使用 `rgba(4, 9, 16, 0.6)`，开关滑块为 `#ffffff`
- 阴影采用低亮度黑色，额外设置 `color-scheme: dark`

## 5. 主题切换机制

### 5.1 useTheme Hook（`src/features/settings/hooks/useTheme.ts`）

三层实现：

1. **用户偏好**（`themePreference`）：`'system'` | `'light'` | `'dark'`，持久化到 `localStorage`（key: `skills-theme`）
2. **系统主题检测**：监听 `window.matchMedia('(prefers-color-scheme: dark)')`
3. **应用主题**：计算 `resolvedTheme`，设置 `document.documentElement.dataset.theme = resolvedTheme`

### 5.2 CSS 生效机制

- `:root`（浅色）是默认主题，始终生效
- `:root[data-theme='dark']` 覆盖浅色变量，仅当 `document.documentElement.dataset.theme === 'dark'` 时生效
- 用户在 `SettingsPage` 选择主题偏好 → `handleThemeChange(preference)` → 更新状态 + localStorage → 触发 DOM 更新 → CSS 变量切换

### 5.3 深色主题自动适配

通过 CSS 变量系统，暗色模式自动适配：组件样式引用 `var(--xxx)`，切换 `data-theme` 时变量值自动替换。**无需**在模块 CSS 中额外写 `[data-theme='dark']` 覆盖（除非确有必要）。

## 6. 代码规范

### 6.1 硬性规则

- **禁止硬编码颜色值**：所有颜色必须通过 `var(--xxx)` 引用 CSS 变量
- **禁止内联样式颜色**：`style={{ color: '#xxx' }}` 不允许，应使用 CSS 类 + 变量
- **新增 CSS 变量必须同时定义浅色和暗色值**：在 `:root` 中定义默认值，在 `:root[data-theme='dark']` 中添加覆盖值
- **CSS 类名使用 kebab-case**：`.skill-card`、`.modal-backdrop`、`.filter-bar`
- **新增样式写入对应模块文件**：不在组件目录下创建 CSS，不新建集中式样式文件

### 6.2 响应式断点

| 断点 | 适用场景 |
|------|---------|
| `@media (max-width: 920px)` | 平板/窄屏 |
| `@media (max-width: 720px)` | 手机横屏 |
| `@media (max-width: 520px)` | 手机竖屏 |

### 6.3 全局背景

`body` 使用纯色 `var(--bg-app)`，不叠加网格；主工作区仅在右上角使用轻微强调色渐变。基础字号为 14px，行高为 1.5，UI 字体采用 Inter 和系统中英文字体回退。

### 6.4 布局与交互尺寸

- 工作台、工具、提示词、标签页面采用 28px / 32px 顶部与水平留白；920px、720px 窗口逐级缩小。
- 常规主操作按钮至少 40px；筛选控件为 36px；紧凑按钮与卡片操作为 32px。
- 卡片圆角使用 `--radius-lg`（14px），通用弹窗使用 `--radius-xl`（18px）。
- 弹窗高度限制在标题栏下方遮罩的可用区域内；正文滚动，页头与操作区不收缩。
- hover、active 不应用于禁用按钮；键盘导航保留清晰的 `:focus-visible` 轮廓。
- 系统启用 `prefers-reduced-motion: reduce` 时缩短动画与过渡并关闭平滑滚动。

## 7. 暗色模式兼容要求

- 保证文字可读（对比度达标）
- 背景不刺眼（使用深灰色而非纯黑）
- 基础交互状态可见（hover、active、focus）
- 无需追求与浅色模式完全一致的视觉效果

## 8. 检查清单

开发完成后确认：

- [ ] 浅色模式下所有状态正常显示
- [ ] 浅色模式下交互反馈清晰可见
- [ ] 所有颜色使用 CSS 变量，无硬编码颜色值
- [ ] 新增 CSS 变量在浅色和暗色中都有定义
- [ ] 新增样式写入对应模块 CSS 文件（非新建独立文件）
- [ ] 暗色模式下文字可读、功能可用
- [ ] 新增样式未破坏现有主题机制
