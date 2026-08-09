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
4. 提交信息遵循规范并带 `!c` 标记（详见下方速查）

## Git 提交规范（速查）

格式：`<type>(<scope>): <subject> !c`

| type | 用途 | 示例 |
|------|------|------|
| feat | 新功能 | `feat(auth): 实现登录流程 !c` |
| fix | 修复 Bug | `fix(download): 修复断点续传异常 !c` |
| refactor | 重构 | `refactor(state): 提取状态访问 helper !c` |
| docs | 文档 | `docs: 更新开发规范 !c` |
| ci | CI 配置 | `ci: 更新构建流程 !c` |
| chore | 构建 / 工具 / 依赖 | `chore: 更新依赖版本 !c` |

要点：

- **末尾必须带 `!c`**：CI 检测到该标记会跳过推送触发的构建，避免重复占用资源
- **泛化描述**：禁止出现敏感引用或第三方项目代号（如 PCL2、pcl 等），使用通用技术术语
- `note:` 前缀的提交表示「作者的话」，会展示在更新日志弹窗顶部（详见开发规范 2.5 节）

## 环境要求

- 前端：Node.js + npm
- 后端：Rust 工具链（stable），Tauri 2 系统依赖见 [DEVELOPMENT_GUIDELINES.md](./DEVELOPMENT_GUIDELINES.md#六开发工具配置)

完整规范请阅读 [DEVELOPMENT_GUIDELINES.md](./DEVELOPMENT_GUIDELINES.md)。