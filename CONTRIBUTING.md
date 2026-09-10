# 贡献指南

> 中文 | [English](docs/CONTRIBUTING.en.md)

感谢你参与 Skills Hub！欢迎修复问题、补充测试、改进文档和翻译，也欢迎提出有明确使用场景的新功能。参与前请阅读 [行为准则](CODE_OF_CONDUCT.md)。

## 先选择合适的渠道

- 使用问题与故障排查：先看 [支持指南](SUPPORT.md) 和 [README](README.md)
- 可复现的 Bug：搜索 [已有 Issues](https://github.com/lucan6290/skills-hub/issues)，没有重复项再提交 Bug 表单
- 新功能或较大的设计变更：先通过功能请求说明问题、使用场景和替代方案，达成共识后再实现
- 安全漏洞：按照 [安全策略](SECURITY.md) 私密报告，不要创建公开 Issue 或公开 PoC
- 小型文档修正可直接提交 PR；Issue 和讨论均可使用中文或英文

## 本地开发

### 环境要求

- Git、Node.js 和 npm；建议使用 Node.js 22.12+（22 系列，与 Release 工作流对齐）。Vite 7 的 Node.js 要求见 [官方指南](https://vite.dev/guide/)
- Rust stable；`Cargo.toml` 声明的最低版本为 1.77.2，但锁定的依赖可能要求更高版本
- 对应操作系统的 [Tauri 2 系统依赖](https://v2.tauri.app/start/prerequisites/)，Windows 需要相应的 C++ 构建工具与 WebView2 环境

先 fork 仓库并克隆自己的 fork，然后在项目根目录执行：

```bash
git switch -c codex/describe-your-change
cd frontend
npm ci
npm run tauri dev
```

上面的 `codex/describe-your-change` 是分支名示例。`npm ci` 使用现有 `package-lock.json`；不要为解决安装失败直接删除锁文件或批量升级依赖。

`npm run tauri dev` 启动 Vite 和桌面窗口；`npm run dev` 仅启动前端，浏览器环境不能代替 Tauri 原生功能验证。日常开发不需要发布签名私钥，不要索取或替换上游密钥。

> **数据安全**：同步、导入、卸载、文件编辑和数据库维护可能影响本地文件。测试时使用临时 Skill、独立测试目录及备份，不要直接对日常使用的工具目录或唯一副本做破坏性测试。

## 工程约定

修改前阅读根目录 [AGENTS.md](AGENTS.md)，按改动范围继续阅读 [前端规范](frontend/AGENTS.md) 或 [Rust 规范](frontend/src-tauri/AGENTS.md) 及其子目录规则。以下是贡献者需要特别注意的约定：

- 只解决当前问题，不附带全仓格式化、无关重构或依赖升级
- 跨端 DTO、JSON 和 Tauri command 参数统一使用 `snake_case`；前端内部状态和函数使用 `camelCase`，详见 [命名规范](docs/naming-conventions.md)
- 沿用前端 feature 结构、路径别名和 Rust commands / services / repositories 分层
- 用户可见文案同步维护中英文；公共文档同时维护对应的 `docs/*.en.md`，历史 CHANGELOG 保持中文
- 使用 AI 辅助时，提交者仍需理解、审查并验证代码，不能提交未经核验的实现或测试结论
- 保留已有版权与署名，确认引入代码、图片、字体及其他资源的来源和许可；项目沿用 [MIT License](LICENSE)，第三方内容不因被导入而自动改用 MIT

## 验证要求

命令均从项目根目录开始：

| 改动范围 | 验证命令 / 检查 |
|---|---|
| 前端逻辑、组件、样式、类型 | `npm --prefix frontend run check`（ESLint + TypeScript + Vite 构建） |
| Rust 或前后端契约 | `cargo test --manifest-path frontend/src-tauri/Cargo.toml`；跨端改动还需前端检查 |
| 版本和发布文档 | `node scripts/version.mjs check`；用真实版本执行 `node scripts/extract-changelog.mjs v0.2.3`（示例） |
| 仅文档或模板 | 检查相对链接、语言入口、命令、表单字段及 `git diff --check` |
| UI 或文件同步行为 | 补充截图 / 录屏或手动复现结果，注明操作系统、工具版本及 global / project 范围 |

修复 Bug 时先提供复现步骤，并尽可能添加回归测试。涉及 symlink / junction / copy、中文路径或 Windows 行为时，注明实际覆盖的平台与场景。未运行的检查必须说明原因，不得写成通过。

当前 [CI](.github/workflows/ci.yml) 在 PR 和推送到 `main` 时执行前端 lint 与 build，**没有 Rust 测试任务**；CI 通过不替代 Rust 和桌面功能验证。

## 提交信息与变更日志

采用 [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) 的类型前缀，项目约定使用中文简述：

`类型: 简要描述`，例如 `docs: 补充贡献指南`、`fix: 修复中文路径下的技能导入`。

| 类型 | 用途 |
|---|---|
| `feat:` | 新功能 |
| `fix:` | Bug 修复 |
| `docs:` | 文档或翻译 |
| `refactor:` / `perf:` / `style:` | 重构、性能、样式或格式 |
| `test:` | 测试 |
| `build:` / `ci:` | 构建与 CI |
| `chore:` | 依赖或维护工作 |

每次提交只包含一个相关工作单元。破坏性变更在正文使用 `BREAKING CHANGE:` 说明影响和迁移方式。维护者的版本提交使用 `release: vX.Y.Z`，不作为 CHANGELOG 条目。

值得对用户说明的变更写入 [CHANGELOG.md](CHANGELOG.md) 的 `Unreleased`，沿用 Added / Changed / Deprecated / Removed / Fixed / Security 分类；构建、文档等使用本项目扩展的 Technical 分类。不要改写历史版本条目，不要在普通 PR 中擅自升版本或打 tag。

## 提交 Pull Request

1. 在功能分支完成最小改动，执行相关验证，检查 `git diff` 与 `git diff --check`
2. 只暂存本次贡献涉及的文件，检查 `git diff --cached`，避免带入他人的改动、构建产物、个人数据或密钥
3. 将分支推送到自己的 fork，并向本仓库 `main` 提交 PR；使用 Agent 操作时，推送须另行获得仓库所有者明确授权
4. 填写 [PR 模板](.github/pull_request_template.md)：目的、关联 Issue、改动范围、验证命令与结果、截图，以及兼容性或数据迁移风险
5. 等待维护者评审并响应反馈；较大的实现可先开 Draft PR。合并和发版由维护者确认

`.env`、Token、私钥、签名密码、数据库和未经脱敏的日志不得提交。发现误泄露时先按 [安全策略](SECURITY.md) 联系维护者，不要认为删除当前文件即可撤销凭据。

## 维护者资料

- [发布工作流](docs/release-workflow.md)：版本、质量门禁、签名、Draft Release 和失败处理
- [安全策略](SECURITY.md)：支持范围、私密报告与协调披露
- [支持指南](SUPPORT.md)：问题分类、诊断信息与数据保护
