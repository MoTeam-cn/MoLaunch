# AI Agent 开发规范

> **适用对象**: AI Agent (Cursor/Copilot/OpenCode 等)
> **版本**: v1.0.0
> **更新日期**: 2026-06-26

---

## 核心原则

**每次修改代码，必须同步更新相关文档和日志。**

---

## 一、修改代码前的检查清单

在进行任何代码修改之前，必须确认：

```
□ 当前版本号是多少？
□ 这次修改属于什么类型？(feat/fix/refactor/docs/ci/chore)
□ 这次修改涉及哪些模块？
□ 是否需要更新 CHANGELOG.md？
□ 是否需要更新版本号？
```

---

## 二、CHANGELOG 更新规则

### 何时必须更新 CHANGELOG

| 修改类型 | 是否更新 | 示例 |
|---------|---------|------|
| 新增功能 | ✅ 必须 | 添加新的页面组件 |
| 修复 Bug | ✅ 必须 | 修复登录失败问题 |
| 重构代码 | ⚠️ 视情况 | 重构状态管理 |
| 性能优化 | ⚠️ 视情况 | 优化渲染性能 |
| 文档更新 | ❌ 不需要 | 修改注释 |
| CI/CD 配置 | ❌ 不需要 | 修改 workflow |
| 依赖更新 | ❌ 不需要 | 更新 package.json |

### CHANGELOG 格式规范

```markdown
## [版本号] - YYYY-MM-DD

### 新增
- 功能描述 (模块: 文件)

### 修复
- 问题描述 (模块: 文件)

### 变更
- 变更描述 (模块: 文件)

### 移除
- 移除描述
```

### 示例

```markdown
## [0.2.0] - 2026-06-26

### 新增
- 添加版本管理页面 (frontend: src/views/VersionManager.vue)
- 实现 Mod 搜索功能 (frontend: src/components/ModSearch.vue)
- 添加 Tauri 命令: 获取版本列表 (backend: src-tauri/src/commands/version.rs)

### 修复
- 修复登录状态丢失问题 (frontend: src/stores/auth.ts)
- 修复下载进度显示错误 (frontend: src/components/DownloadProgress.vue)

### 变更
- 重构认证状态管理 (frontend: src/stores/auth.ts)
- 优化侧边栏导航样式 (frontend: src/components/Sidebar.vue)
```

---

## 三、版本号管理规则

### 版本号格式

遵循 Semantic Versioning: `MAJOR.MINOR.PATCH`

| 变更类型 | 版本变化 | 示例 |
|---------|---------|------|
| 不兼容的 API 变更 | MAJOR +1 | 0.1.0 → 1.0.0 |
| 新增功能 | MINOR +1 | 0.1.0 → 0.2.0 |
| Bug 修复 | PATCH +1 | 0.1.0 → 0.1.1 |

### 何时更新版本号

```
□ 新增了页面/组件 → MINOR
□ 新增了 Tauri 命令 → MINOR
□ 新增了功能模块 → MINOR
□ 修复了严重 Bug → PATCH
□ 重构了内部实现 → PATCH
□ 不兼容的 API 变更 → MAJOR
```

### 版本号同步位置

更新版本号时，必须同步以下位置：

```
□ package.json → "version": "X.Y.Z"
□ src-tauri/Cargo.toml → version = "X.Y.Z"
□ src-tauri/tauri.conf.json → "version": "X.Y.Z"
□ CHANGELOG.md → ## [X.Y.Z] - YYYY-MM-DD
```

---

## 四、Git 提交规范

### 提交信息格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Type 类型

| Type | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat(auth): 添加微软 OAuth 登录` |
| `fix` | 修复 Bug | `fix(download): 修复断点续传失败` |
| `refactor` | 重构 | `refactor(state): 重构状态管理` |
| `perf` | 性能优化 | `perf(render): 优化列表渲染` |
| `docs` | 文档 | `docs: 更新 CHANGELOG` |
| `test` | 测试 | `test(auth): 添加认证测试` |
| `ci` | CI/CD | `ci: 添加 GitHub Actions` |
| `chore` | 构建/工具 | `chore: 更新依赖版本` |
| `style` | 样式 | `style: 优化按钮样式` |

### Scope 范围

```
frontend, backend, auth, version, mod, skin, java, settings, ui, state, commands
```

### 提交前检查

```
□ npm run lint (前端代码检查)
□ npm run typecheck (TypeScript 类型检查)
□ cargo clippy (Rust 代码检查)
□ cargo fmt (Rust 代码格式化)
□ 更新 CHANGELOG.md (如果需要)
□ 提交信息格式正确
```

---

## 五、日志记录规范

### 何时添加日志

| 场景 | 日志级别 | 示例 |
|------|---------|------|
| 初始化/启动 | INFO | `log::info!("Initializing MoLaunch v{}", version)` |
| 重要操作完成 | INFO | `log::info!("Version downloaded: {}", version_id)` |
| 警告情况 | WARN | `log::warn!("File already exists, skipping: {}", path)` |
| 可恢复错误 | ERROR | `log::error!("Failed to download: {}", e)` |
| 调试信息 | DEBUG | `log::debug!("Parsed version: {}", version)` |
| 详细跟踪 | TRACE | `log::trace!("Processing file: {}", path)` |

### 日志格式规范

```rust
// ✅ 正确: 包含上下文信息
log::info!("Download completed: {} ({} bytes)", url, size);
log::error!("Failed to parse version manifest: {}", e);

// ❌ 错误: 过于简单
log::info!("Done");
log::error!("Error");
```

### Tauri 命令日志

```rust
#[tauri::command]
pub async fn get_versions(handle: State<'_, AppState>) -> Result<Vec<VersionInfo>, String> {
    log::info!("Getting version list");
    
    let versions = handle.sdk.list_versions()
        .map_err(|e| {
            log::error!("Failed to get versions: {}", e);
            e.to_string()
        })?;
    
    log::info!("Found {} versions", versions.len());
    Ok(versions)
}
```

---

## 六、Tauri 命令开发规范

### 新增 Tauri 命令的完整流程

```
1. 在 src-tauri/src/commands/ 中添加函数
2. 添加 #[tauri::command] 属性
3. 实现完整的错误处理
4. 添加文档注释 (///)
5. 在 main.rs 中注册命令
6. 在前端创建对应的 TypeScript 类型
7. 更新 CHANGELOG.md
8. 提交并推送
```

### Tauri 命令模板

```rust
/// 函数功能描述
///
/// # Arguments
/// * `handle` - 应用状态
/// * `param` - 参数说明
///
/// # Returns
/// 成功返回数据，失败返回错误信息
#[tauri::command]
pub async fn example_command(
    handle: State<'_, AppState>,
    param: String,
) -> Result<ExampleData, String> {
    log::info!("Executing example command with param: {}", param);
    
    // 1. 参数验证
    if param.is_empty() {
        log::warn!("Empty parameter received");
        return Err("Parameter cannot be empty".to_string());
    }
    
    // 2. 业务逻辑
    let result = handle.sdk.some_function(&param)
        .map_err(|e| {
            log::error!("Command failed: {}", e);
            e.to_string()
        })?;
    
    // 3. 返回结果
    log::info!("Command executed successfully");
    Ok(result)
}
```

---

## 七、Vue 组件开发规范

### 组件命名规范

- 文件名: `PascalCase.vue` (如 `VersionList.vue`)
- 组件名: `PascalCase` (如 `VersionList`)
- 组合式函数: `useXxx` (如 `useAuth`)

### 组件模板

```vue
<script setup lang="ts">
/**
 * 组件功能描述
 */

// Props 定义
interface Props {
  title: string
  count?: number
}

const props = withDefaults(defineProps<Props>(), {
  count: 0,
})

// Emits 定义
const emit = defineEmits<{
  select: [id: string]
  delete: [id: string]
}>()

// 组合式函数
const { data, loading, error } = useExample()

// 方法
const handleSelect = (id: string) => {
  emit('select', id)
}
</script>

<template>
  <div class="example-component">
    <h2>{{ title }}</h2>
    <p>Count: {{ count }}</p>
    <button @click="handleSelect('test')">Select</button>
  </div>
</template>

<style scoped>
.example-component {
  /* 样式 */
}
</style>
```

---

## 八、修改完成后的检查清单

每次修改代码后，**必须按顺序执行以下命令**，全部通过后才能提交：

```bash
# 1. Rust 代码格式化 (必须第一个执行)
cd src-tauri && cargo fmt

# 2. Rust 代码检查 (必须通过，0 warnings)
cd src-tauri && cargo clippy -- -D warnings

# 3. 前端代码检查 (必须通过，0 errors)
npm run lint

# 4. TypeScript 类型检查
npm run typecheck

# 5. 提交
git add -A
git commit -m "type(scope): description"
git push origin main
```

**重要提醒：**
- **绝对不要跳过本地检查直接提交！** CI 会检查代码，本地不过 CI 也不会过。
- `npm run lint` 必须 0 errors 才能提交（warnings 可以接受）
- `cargo clippy` 必须 0 warnings 才能提交（使用 `-D warnings` 将警告视为错误）
- `cargo fmt` 必须在 `cargo clippy` 之前执行

---

## 九、禁止事项

### 绝对不要

```
❌ 提交包含 TODO 的代码 (除非明确标注为"未来功能")
❌ 提交不完整的 Tauri 命令 (直接返回 Ok)
❌ 修改代码后不更新 CHANGELOG
❌ 使用 unwrap() 或 expect() 在 Rust 生产代码中
❌ 在 Tauri 命令中 panic
❌ 遗漏错误处理
❌ 提交未通过 lint 检查的代码
```

### 必须要做

```
✅ 所有 Tauri 命令使用 Result 返回
✅ 所有错误转换为字符串返回
✅ 重要操作记录日志
✅ 修改后更新 CHANGELOG
✅ 新功能更新版本号
✅ 运行测试确认通过
✅ 前端组件添加 TypeScript 类型
```

---

## 十、快速参考

### 添加新功能的完整流程

```bash
# 1. 编写代码
# 2. 前端检查
npm run lint && npm run typecheck

# 3. 后端检查
cd src-tauri && cargo fmt && cargo clippy -- -D warnings

# 4. 更新 CHANGELOG.md
# 5. 更新版本号 (如果需要)
# 6. 提交
git add -A
git commit -m "feat(module): 描述"
git push origin main
```

### 修复 Bug 的完整流程

```bash
# 1. 修复代码
# 2. 前端检查
npm run lint && npm run typecheck

# 3. 后端检查
cd src-tauri && cargo fmt && cargo clippy -- -D warnings

# 4. 更新 CHANGELOG.md
# 5. 提交
git add -A
git commit -m "fix(module): 描述"
git push origin main
```

---

## 十一、技术栈参考

### 前端
- **框架**: Vue 3 + TypeScript
- **构建工具**: Vite
- **状态管理**: Pinia
- **路由**: Vue Router
- **UI 组件**: Headless UI + Tailwind CSS
- **动画**: Framer Motion

### 后端
- **框架**: Tauri
- **语言**: Rust
- **SDK**: McSDK (C FFI)
- **异步**: Tokio

### 开发工具
- **包管理**: npm
- **代码检查**: ESLint (前端) + Clippy (后端)
- **格式化**: Prettier (前端) + rustfmt (后端)
- **类型检查**: TypeScript

---

*本文档最后更新于 2026-06-26*
