# 支持指南

> 中文 | [English](docs/SUPPORT.en.md)

本指南用于 Skills Hub 的安装、使用与故障排查。贡献代码请阅读 [贡献指南](CONTRIBUTING.md)；漏洞和敏感信息请使用 [私密安全报告渠道](SECURITY.md)。

## 到哪里提问

| 情况 | 入口 |
|---|---|
| 安装、使用或同步问题 | 先查 [README](README.md) 与 [已有 Issues](https://github.com/lucan6290/skills-hub/issues)；仍无法解决时提交使用问题 |
| 能稳定复现的 Bug | 使用 [Bug 报告表单](https://github.com/lucan6290/skills-hub/issues/new?template=bug_report.yml) |
| 功能建议 | 使用 [功能请求表单](https://github.com/lucan6290/skills-hub/issues/new?template=feature_request.yml)，说明使用场景和替代方案 |
| 安全漏洞、凭据泄露 | 阅读 [SECURITY.md](SECURITY.md)，不要公开报告 |
| 骚扰或社区行为问题 | 按 [行为准则](CODE_OF_CONDUCT.md) 中的渠道私密联系 |

普通问题请优先在公开 Issue 中沟通，便于其他人复用结论。项目由社区维护，不承诺固定响应时间、商业支持或每个功能请求都被实现。中文和英文均可。

## 提问前先检查

1. 记录应用版本或构建提交、操作系统与架构，并核对 [发布页面](https://github.com/lucan6290/skills-hub/releases) 和 [变更日志](CHANGELOG.md)
2. 搜索是否已有相同问题；已有时补充环境和最小复现，不要重复开 Issue
3. 使用临时目录与不含个人信息的示例 Skill 重现，确认是安装、导入、同步、编辑还是某个工具读取失败
4. 同步问题记录目标 AI 工具、global / project 范围、路径类型，以及已知的 symlink / junction / copy 状态；不知道的项目可以注明未知
5. 构建失败时记录 Node.js / npm / Rust 版本和失败命令，先按 [贡献指南](CONTRIBUTING.md) 检查依赖环境

不要把删除整个数据目录、清空数据库、关闭权限校验、禁用更新签名验证或以管理员身份运行所有操作当作通用修复步骤。进行恢复、导入备份、卸载或覆盖文件前，先确认影响范围并保留备份。

## 报告中请提供

- 一句话总结问题，以及可重复的最少操作步骤
- 预期行为和实际行为
- 应用版本、系统与架构；开发版还需提交号及构建工具版本
- 同步相关问题的目标工具、范围和路径结构（用示例路径替代用户名、私有项目名）
- 相关错误文本、必要的日志片段或截图；不要直接上传完整数据库或整套工作目录
- 已尝试的办法，以及结果是否偶发、能否在临时目录复现

## 隐私与日志

提交前手动检查日志和截图，移除 Token、API Key、密码、私钥、邮箱、个人路径、私有仓库地址、提示词及 Skill 中的机密内容。不要上传 `.env` 或签名文件。

如果问题需要敏感材料，请先私密联系维护者确认最小必要信息与传递方式；如果已经泄露凭据，停止传播并撤销或轮换相关凭据，仅删除公开内容并不足够。

## 下载和自动更新

- 优先使用本仓库 [Releases](https://github.com/lucan6290/skills-hub/releases) 中维护者正式发布的附件，不使用来源不明的重新打包文件
- 当前 Release 工作流构建 Windows NSIS 和 MSI 安装包；macOS / Linux 开发需按 Tauri 前置条件自行构建，不代表已提供对应的预编译发行包
- 自动更新失败时，保留错误信息并按上述格式报告；可在确认版本和来源后手动安装正式发布的安装包，不要绕过签名校验
- 发布页缺少文件或仍为 Draft 时，不代表发布已完成；维护者应按 [发布工作流](docs/release-workflow.md) 核验产物
