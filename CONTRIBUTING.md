# 参与贡献

感谢你考虑为 **MoLaunch** 贡献代码。动手前请先阅读仓库内的两份核心文档：

- [DEVELOPMENT_GUIDELINES.md](./DEVELOPMENT_GUIDELINES.md) — 开发规范（代码风格、Git 提交规范、测试规范、常用命令）
- [DEVELOPMENT_BLUEPRINT.md](./DEVELOPMENT_BLUEPRINT.md) — 架构蓝图（前后端架构、数据流、模块总览）

## 提交 Issue

- Bug 报告 / 功能建议 / 使用提问，请分别使用 `.github/ISSUE_TEMPLATE/` 下的对应模板
- 提交前先在 [Issues](https://github.com/MoTeam-cn/MoLaunch/issues) 中搜索，避免重复反馈
- Bug 报告请附上环境信息（操作系统 / 启动器版本 / 游戏版本 / Java 版本）

## 提交 PR

1. 从 `main` 创建功能或修复分支：`feat/<name>`、`fix/<name>`
2. 本地验证必须通过：

   ```bash
   # 前端
   npm run typecheck
   npm run lint

   # 后端（在 src-tauri 目录）
   cargo check
   cargo clippy -- -D warnings
   ```

3. 同步更新 `CHANGELOG.md`（追加到 `[Unreleased]` 区）
4. 提交信息遵循规范（默认不带 `!c`，详见下方速查）

## Git 提交规范（速查）

格式：`<type>(<scope>): <subject>`

| type | 用途 | 示例 |
|------|------|------|
| feat | 新功能 | `feat(auth): 实现登录流程` |
| fix | 修复 Bug | `fix(download): 修复断点续传异常` |
| refactor | 重构 | `refactor(state): 提取状态访问 helper` |
| docs | 文档 | `docs: 更新开发规范` |
| ci | CI 配置 | `ci: 更新构建流程` |
| chore | 构建 / 工具 / 依赖 | `chore: 更新依赖版本` |

要点：

- **默认不带 `!c`**：`!c` 仅在需要跳过本次推送触发的 CI 构建时附加
- `note:` 前缀的提交表示「作者的话」，会展示在更新日志弹窗顶部（详见开发规范 2.5 节）

## 发布与通道 tag

发布由 tag 驱动：推送 `v*` 标签后，`version-sync.yml` 先把仓库内版本号文件同步到标签版本并提交，
再以 `workflow_call` 激活 `release.yml`；**不要**手工改完版本号再打 tag 之外的额外操作。

一次发版产出两条分发链路：

- **云端通道**：安装包推 S3 并在 api-server 注册版本，启动器默认按此检查更新（需设备凭证、可按设备灰度）。
- **GitHub 通道**：产物附加到 GitHub Release 供手动下载，同时由 `release.yml` 的 `Generate update manifest` 步骤
  生成 `latest.json`（`scripts/generate-update-manifest.cjs`），供启动器「检查更新源 = GitHub」时读取。

`latest.json` 要点：

- 结构为 `channels.<通道>.{version, notes, force_update, platforms.<平台>.{url, signature}}`，通道取 `stable` / `beta` / `alpha`；
- Windows 平台指向**便携版**（`MoLaunch_<ver>_x64.exe`），不是 NSIS 安装包——自更新要替换主程序本体；
- `signature` 取自随包的 `.sig`，缺失时该字段省略（不会阻断发版，但该平台无法校验更新包）；
- `force_update` 默认 `false`，需要强制更新时在发版任务里置 `FORCE_UPDATE=true`。

**滚动通道 tag**（`channel-beta` / `channel-alpha`）：

- 预发布发版时，清单会被额外推到 `channel-<通道>` 的专用 Release，因为 GitHub 没有「最新预发布」的固定路径；
- 该 Release 标记 `prerelease` 且 `make_latest: false`，永远不会被 `/releases/latest` 命中，stable 通道仍由正式 Release 提供；
- **不要删除这两个 tag / Release**，否则对应通道的 GitHub 更新源会 404；
- 它们不是版本节点：`git describe` 计算上一个 tag 时已用 `--exclude='channel-*'` 排除，手动查历史时也请忽略。

## 环境要求

- 前端：Node.js + npm
- 后端：Rust 工具链（stable），Tauri 2 系统依赖见 [DEVELOPMENT_GUIDELINES.md](./DEVELOPMENT_GUIDELINES.md#六开发工具配置)

完整规范请阅读 [DEVELOPMENT_GUIDELINES.md](./DEVELOPMENT_GUIDELINES.md)。