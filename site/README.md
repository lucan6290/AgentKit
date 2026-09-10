# site/ — 产品宣传页（GitHub Pages 站点）

> 本目录是项目的**对外产品宣传着陆页**，通过 GitHub Actions 自动部署到 `https://<owner>.github.io/<repo>/`。
> 同时它也是 README 截图的生成源——使用 Chrome headless 将每个区块渲染为 PNG 输出到 `docs/picture/`。

---

## 文件清单

```
site/
├── index.html        ← 宣传页主文件（单文件，内联 CSS/SVG/JS，零外部依赖）
├── screenshot.cjs    ← Chrome headless 自动截图脚本（输出到 ../docs/picture/）
├── .nojekyll         ← 告诉 GitHub Pages 不要用 Jekyll 处理（避免 _ 开头文件被忽略）
└── README.md         ← 你正在读
```

---

## 定制清单（复制模板后必改）

1. `index.html` 中所有 `<PROJECT_NAME>` 占位符、标题、标语、功能文案、下载链接（GitHub 仓库/Releases 地址）
2. `screenshot.cjs` 中的 Chrome 路径与截图清单
3. `pages.yml` 中的分支名（默认 `main`）

---

## 本地预览

```powershell
# 在 site/ 目录启动静态服务器
cd site
python -m http.server 7891

# 浏览器打开
# http://localhost:7891/
```

---

## 重新生成截图

```powershell
# 1. 确保静态服务器正在运行（端口 7891，见上）
# 2. 在项目根目录运行截图脚本
node site/screenshot.cjs
```

截图输出到 `docs/picture/`，README.md 直接引用这些图片。

---

## 线上部署

部署由 `.github/workflows/pages.yml` 自动完成：

- 触发时机：推送到 `main` 分支且 `site/**` 有变更，或手动触发（workflow_dispatch）
- 部署目标：GitHub Pages（Project Site）
- 访问地址：**https://<owner>.github.io/<repo>/**
- 与个人博客 `https://<owner>.github.io/`（User Site）互不影响

**首次启用步骤**（只需在仓库 Settings 操作一次）：

1. 进入仓库 `Settings` → `Pages`
2. `Source` 选择 **GitHub Actions**（不是 Deploy from a branch）
3. 推送代码到 main，workflow 会自动构建并部署

---

## 设计风格

ccswitch.io 温暖奶油色营销风（可自行替换为主题色）：

- 背景：奶油色 `#f3ece1` + 网格纹理
- 主色：陶土橙 `#c9623a`
- 产品 UI mockup：深色 `#1c1c1e`，与暖色背景形成对比
- 字体：系统字体栈（苹方/微软雅黑/Segoe UI）
- 所有图标使用内联 SVG（lucide 风格），零 CDN 依赖
