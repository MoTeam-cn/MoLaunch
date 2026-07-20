# AI Agent 协作规范

> **适用对象**：AI Agent（Cursor / Copilot / Trae 等）
> **最后更新**：2026-07-20

---

## 一、核心原则

【必须】每次修改代码，必须同步更新 `CHANGELOG.md`。这是 AI Agent 协作的第一约束，违反即视为任务未完成。

完整的开发规范（代码风格、UI 组件、分支管理等）见 `DEVELOPMENT_GUIDELINES.md`，本文件仅聚焦 AI 协作过程中必须遵守的行为约束。

---

## 二、CHANGELOG 更新规则

### 2.1 何时必须更新

| 修改类型 | 是否更新 CHANGELOG |
|---------|-------------------|
| 新增功能 | 必须 |
| 修复 Bug | 必须 |
| 重构代码 | 视情况（影响外部行为则更新） |
| 性能优化 | 视情况 |
| 文档 / 注释 | 不需要 |
| CI / CD 配置 | 不需要 |
| 依赖更新 | 不需要 |

### 2.2 格式规范

遵循项目现有 `CHANGELOG.md` 风格（基于 Keep a Changelog）：

```markdown
## [版本号或 未发布]

### 新增

#### 功能标题
- `文件路径`：变更说明
- `文件路径`：变更说明

### 修复

#### 修复标题
- `文件路径`：变更说明

### 重构
### 变更
### 移除
```

要点：
- 分类标题统一使用：新增 / 修复 / 重构 / 变更 / 移除
- 每个功能点用 `####` 四级标题概括
- 每条变更必须标注涉及的文件路径（用反引号包裹）
- 新增的未发布内容写入 `## [未发布]` 段下

---

## 三、Git 提交规范

### 3.1 提交信息格式

```
<type>(<scope>): <subject> !c
```

### 3.2 常见 type

| type | 说明 |
|------|------|
| feat | 新功能 |
| fix | 修复 Bug |
| refactor | 重构（不改变外部行为） |
| perf | 性能优化 |
| docs | 文档更新 |
| test | 测试相关 |
| ci | CI / CD 配置 |
| chore | 构建 / 工具 / 依赖 |
| style | 代码格式 |

scope 与完整示例见 `DEVELOPMENT_GUIDELINES.md` 第二节。

### 3.3 【必须】提交信息末尾携带 `!c`

提交信息末尾必须包含 `!c` 标记。项目的 CI 流水线会检查 commit message 是否包含 `!c` 子串，包含则跳过本次推送触发的 CI，避免重复占用构建资源。

示例：

```
feat(version): 实现版本下载功能 !c
```

### 3.4 【必须】提交信息使用常规泛化描述

提交信息必须使用常规、泛化的技术描述，严禁出现敏感引用或第三方项目代号（如 PCL2、pcl 等）。描述应遮掩实际意图，使用通用术语。

- 允许：`refactor(state): 提取状态访问 helper 消除重复套件`
- 允许：`feat(auth): 实现登录流程`
- 禁止：包含第三方项目名、内部代号、特定实现来源引用
- 禁止：`refactor(state): 参考 PCL2 实现 FileSystemWatcher`

---

## 四、修改前检查清单

开始任何代码修改前，必须确认：

- 当前版本号是多少？（`package.json` / `Cargo.toml` / `tauri.conf.json` 三处应一致）
- 本次修改属于什么类型？（feat / fix / refactor / perf / docs / chore ...）
- 本次修改涉及哪些模块？（frontend / backend / auth / version / mod / skin / java / settings / ui / state / commands ...）
- 是否需要更新 `CHANGELOG.md`？
- 是否需要更新版本号？（新增功能 MINOR，Bug 修复 PATCH，不兼容变更 MAJOR）

---

## 五、修改后检查清单

修改完成后，【必须】按以下顺序执行，全部通过才能提交：

1. `cargo fmt`（Rust 格式化，在 `src-tauri` 目录执行）
2. `cargo clippy -- -D warnings`（Rust 检查，必须 0 warnings）
3. `npm run lint`（前端检查，必须 0 errors）
4. `npm run typecheck`（TypeScript 类型检查，必须通过）
5. 更新 `CHANGELOG.md`（如本次修改属于"必须更新"类型）
6. 提交（commit message 遵循第三节规范，末尾带 `!c`）

命令速查：

```bash
cd src-tauri && cargo fmt
cd src-tauri && cargo clippy -- -D warnings
npm run lint
npm run typecheck
```

---

## 六、禁止事项

【禁止】在 Rust 生产代码中使用 `unwrap()` / `expect()` / `panic!()`
【禁止】在 TypeScript 中使用 `any` 类型（如确需，使用 `unknown` 配合类型守卫）
【禁止】提交包含 `TODO` 的代码（除非明确标注为"未来功能"并说明计划）
【禁止】提交不完整的功能（如 Tauri 命令直接返回 `Ok(())` 占位、空实现）
【禁止】修改代码后不更新 `CHANGELOG.md`
【禁止】跳过 lint / clippy / typecheck 检查直接提交
【禁止】在 commit message 中出现敏感引用或第三方项目代号
【禁止】遗漏错误处理（所有可能失败的操作必须返回 `Result`）

---

## 七、必须事项

【必须】所有 Tauri 命令使用 `Result<T, String>` 返回，错误转换为字符串
【必须】所有 Tauri 命令通过 `State<'_, AppState>` 注入状态，`lock()` 后及时 `drop()` 释放锁
【必须】重要操作记录日志（使用项目自定义宏 `log_info!` / `log_warn!` / `log_error!`，非 `log::` crate）
【必须】前端组件使用 `<script setup lang="ts">` + Composition API
【必须】UI 交互组件使用项目自定义组件（Button / Input / Select / Modal / Toast 等），不引入第三方 UI 库
【必须】修改后更新 `CHANGELOG.md`
【必须】前端代码为所有变量、参数、返回值标注 TypeScript 类型

---

*本文档最后更新于 2026-07-20*
