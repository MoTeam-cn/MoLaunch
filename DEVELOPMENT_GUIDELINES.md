# MoLaunch 开发规范 (Development Guidelines)

> **版本**: v1.0.0
> **更新日期**: 2026-06-26
> **适用范围**: 所有参与 MoLaunch 开发的人员和 AI Agent

---

## 目录

1. [代码风格规范](#1-代码风格规范)
2. [Git 提交规范](#2-git-提交规范)
3. [分支管理策略](#3-分支管理策略)
4. [文档编写规范](#4-文档编写规范)
5. [测试编写规范](#5-测试编写规范)
6. [代码审查流程](#6-代码审查流程)
7. [发布流程](#7-发布流程)
8. [AI Agent 开发约束](#8-ai-agent-开发约束)

---

## 1. 代码风格规范

### 1.1 前端规范 (Vue/TypeScript)

#### 命名规范

- **文件名**: `PascalCase.vue` (组件), `camelCase.ts` (工具/Store)
- **组件名**: `PascalCase` (如 `VersionList`)
- **组合式函数**: `useXxx` (如 `useAuth`)
- **常量**: `SCREAMING_SNAKE_CASE`
- **变量/函数**: `camelCase`
- **类型/接口**: `PascalCase`

#### 代码格式

```typescript
// ✅ 正确
interface UserProfile {
  id: string
  username: string
  avatar?: string
}

const getUserProfile = async (id: string): Promise<UserProfile> => {
  const response = await fetch(`/api/users/${id}`)
  return response.json()
}

// ❌ 错误
interface user_profile {
  ID: string
  User_Name: string
}

const get_user_profile = async (id) => {
  // ...
}
```

#### Vue 组件规范

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

### 1.2 后端规范 (Rust)

#### 命名规范

- **模块名**: `snake_case`
- **类型名**: `PascalCase`
- **函数名**: `snake_case`
- **常量**: `SCREAMING_SNAKE_CASE`
- **生命周期**: `'lowercase`

#### 代码格式

```rust
// ✅ 正确
pub struct AppState {
    pub sdk: Arc<Mutex<McSdk>>,
    pub config: Arc<Mutex<AppConfig>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sdk: Arc::new(Mutex::new(McSdk::default())),
            config: Arc::new(Mutex::new(AppConfig::default())),
        }
    }
}

// ❌ 错误
pub struct app_state {
    pub SDK: Arc<Mutex<McSdk>>,
    pub Config: Arc<Mutex<AppConfig>>,
}
```

#### Tauri 命令规范

```rust
// ✅ 正确: 完整的错误处理和日志
#[tauri::command]
pub async fn list_versions(
    state: State<'_, AppState>,
) -> Result<VersionList, String> {
    log::info!("Fetching version list");
    
    let sdk = state.sdk.lock().await;
    let versions = sdk.list_versions()
        .map_err(|e| {
            log::error!("Failed to list versions: {}", e);
            e.to_string()
        })?;
    
    log::info!("Found {} versions", versions.versions.len());
    Ok(versions)
}

// ❌ 错误: 缺少错误处理和日志
#[tauri::command]
pub async fn list_versions(
    state: State<'_, AppState>,
) -> Result<VersionList, String> {
    let sdk = state.sdk.lock().await;
    sdk.list_versions().map_err(|e| e.to_string())
}
```

#### 错误处理规范

```rust
// ✅ 正确: 使用 Result 和错误码
pub fn some_function() -> Result<Data, McSdkError> {
    let value = do_something().map_err(|e| McSdkError::IoError(e))?;
    Ok(value)
}

// ✅ 正确: Tauri 命令转换为字符串
#[tauri::command]
pub async fn some_command() -> Result<Data, String> {
    some_function().map_err(|e| e.to_string())
}

// ❌ 错误: 使用 unwrap 或 expect
#[tauri::command]
pub async fn some_command() -> Result<Data, String> {
    let value = some_function().unwrap(); // 可能 panic!
    Ok(value)
}
```

#### 内存管理规范

```rust
// ✅ 正确: 使用 Arc<Mutex<T>> 共享状态
pub struct AppState {
    pub sdk: Arc<Mutex<McSdk>>,
}

// ✅ 正确: 使用 Box 管理堆内存
let data = Box::new(large_data);
process_data(&data);

// ❌ 错误: 使用全局可变状态
static mut GLOBAL_SDK: Option<McSdk> = None;
```

---

## 2. Git 提交规范

### 2.1 提交信息格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 2.2 Type 类型

| Type | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat(auth): 添加微软 OAuth 登录` |
| `fix` | 修复 bug | `fix(download): 修复断点续传失败` |
| `docs` | 文档更新 | `docs: 更新 API 文档` |
| `style` | 代码格式 | `style: 格式化代码` |
| `refactor` | 重构 | `refactor(launch): 重构启动流程` |
| `perf` | 性能优化 | `perf(network): 优化下载速度` |
| `test` | 测试相关 | `test(auth): 添加认证测试` |
| `chore` | 构建/工具 | `chore: 更新依赖版本` |
| `ci` | CI/CD | `ci: 添加 GitHub Actions` |

### 2.3 Scope 范围

- `frontend` - 前端相关
- `backend` - 后端相关
- `auth` - 认证模块
- `version` - 版本管理
- `mod` - Mod 管理
- `skin` - 皮肤管理
- `java` - Java 管理
- `settings` - 设置
- `ui` - UI 组件
- `state` - 状态管理
- `commands` - Tauri 命令
- `docs` - 文档

### 2.4 示例

```
feat(version): 实现版本下载功能

- 支持版本列表获取
- 支持版本下载
- 支持下载进度显示

Closes #123
```

```
fix(auth): 修复微软登录超时问题

- 增加轮询超时时间
- 优化错误提示信息

Fixes #456
```

---

## 3. 分支管理策略

### 3.1 分支类型

| 分支 | 命名 | 用途 | 生命周期 |
|------|------|------|----------|
| 主分支 | `main` | 稳定版本 | 永久 |
| 开发分支 | `develop` | 日常开发 | 永久 |
| 功能分支 | `feat/<name>` | 新功能开发 | 临时 |
| 修复分支 | `fix/<name>` | Bug 修复 | 临时 |
| 发布分支 | `release/<version>` | 版本发布 | 临时 |
| 热修复 | `hotfix/<name>` | 紧急修复 | 临时 |

### 3.2 工作流程

```
main ──────────────────────────────────────────────►
  │                                    ▲
  │    ┌─────────────────────────────┐ │
  └───►│ develop                     │─┘
       │  ├─ feat/auth ─────────────►│
       │  ├─ feat/version ──────────►│
       │  └─ fix/download ──────────►│
       └─────────────────────────────┘
```

### 3.3 合并规则

- `feat/*` → `develop`: 通过 PR，需要代码审查
- `fix/*` → `develop`: 通过 PR，需要代码审查
- `develop` → `main`: 通过 PR，需要测试通过
- `hotfix/*` → `main` + `develop`: 直接合并

---

## 4. 文档编写规范

### 4.1 代码文档

#### 前端文档

```typescript
/**
 * 获取用户资料
 *
 * @param id - 用户 ID
 * @returns 用户资料对象
 * @throws {Error} 当用户不存在时抛出错误
 *
 * @example
 * ```ts
 * const user = await getUserProfile('123')
 * console.log(user.username)
 * ```
 */
async function getUserProfile(id: string): Promise<UserProfile> {
  // ...
}
```

#### 后端文档

```rust
/// 获取 Minecraft 版本列表
///
/// # Arguments
/// * `state` - 应用状态
///
/// # Returns
/// 成功返回版本列表，失败返回错误信息
///
/// # Examples
/// ```rust
/// let versions = list_versions(state).await?;
/// println!("Found {} versions", versions.versions.len());
/// ```
#[tauri::command]
pub async fn list_versions(
    state: State<'_, AppState>,
) -> Result<VersionList, String> {
    // ...
}
```

### 4.2 组件文档

```vue
<script setup lang="ts">
/**
 * 版本卡片组件
 *
 * 用于显示单个版本的信息，包括版本号、类型、发布时间等。
 * 支持下载、删除等操作。
 *
 * @example
 * ```vue
 * <VersionCard
 *   :version="version"
 *   @download="handleDownload"
 *   @delete="handleDelete"
 * />
 * ```
 */

interface Props {
  /** 版本信息对象 */
  version: VersionInfo
  /** 是否显示操作按钮 */
  showActions?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  showActions: true,
})
</script>
```

### 4.3 README 文档

- 项目根目录必须有 `README.md`
- 包含：项目介绍、快速开始、功能特性、技术栈、开发指南

---

## 5. 测试编写规范

### 5.1 前端测试

```typescript
// src/composables/__tests__/useAuth.test.ts

import { describe, it, expect, vi } from 'vitest'
import { useAuth } from '../useAuth'

describe('useAuth', () => {
  it('should login offline with valid username', async () => {
    // Arrange
    const { loginOffline, isLoggedIn, username } = useAuth()
    
    // Act
    await loginOffline('TestPlayer')
    
    // Assert
    expect(isLoggedIn.value).toBe(true)
    expect(username.value).toBe('TestPlayer')
  })
  
  it('should return error with empty username', async () => {
    // Arrange
    const { loginOffline, error } = useAuth()
    
    // Act
    await loginOffline('')
    
    // Assert
    expect(error.value).toBeTruthy()
  })
})
```

### 5.2 后端测试

```rust
// src-tauri/src/commands/version.rs

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_list_versions_returns_versions() {
        // Arrange
        let state = create_test_state();
        
        // Act
        let result = list_versions(state).await;
        
        // Assert
        assert!(result.is_ok());
        assert!(!result.unwrap().versions.is_empty());
    }
    
    #[tokio::test]
    async fn test_list_versions_with_invalid_state() {
        // Arrange
        let state = create_invalid_state();
        
        // Act
        let result = list_versions(state).await;
        
        // Assert
        assert!(result.is_err());
    }
}
```

### 5.3 测试命名规范

- 格式: `test_<被测功能>_<场景>_<预期结果>`
- 示例:
  - `test_login_offline_valid_username_returns_success`
  - `test_download_invalid_url_returns_error`
  - `test_version_parse_missing_field_returns_error`

### 5.4 测试覆盖率目标

- 前端组合式函数: 80%+
- 前端组件: 70%+
- Rust 命令层: 90%+
- SDK 封装层: 95%+

---

## 6. 代码审查流程

### 6.1 审查清单

#### 前端审查

- [ ] 代码风格符合规范
- [ ] TypeScript 类型定义完整
- [ ] 组件 Props/Emits 定义清晰
- [ ] 无 `any` 类型使用
- [ ] 有必要的注释和文档
- [ ] 有对应的测试用例
- [ ] 性能无明显问题

#### 后端审查

- [ ] 代码风格符合规范
- [ ] 无 `unwrap()` 或 `expect()` 在生产代码中
- [ ] 错误处理完整
- [ ] 内存管理正确
- [ ] 有必要的日志记录
- [ ] 有对应的测试用例
- [ ] 性能无明显问题

### 6.2 审查流程

1. 创建 PR，填写变更说明
2. 自动触发 CI 检查 (lint, typecheck, clippy)
3. 至少一位维护者审查
4. 审查通过后合并

---

## 7. 发布流程

### 7.1 版本号规范

遵循 [Semantic Versioning](https://semver.org/):

```
MAJOR.MINOR.PATCH

MAJOR: 不兼容的 API 变更
MINOR: 向后兼容的功能添加
PATCH: 向后兼容的 Bug 修复
```

### 7.2 发布步骤

1. 从 `develop` 创建 `release/<version>` 分支
2. 更新版本号:
   - `package.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/tauri.conf.json`
3. 更新 `CHANGELOG.md`
4. 创建 PR 到 `main`
5. 合并后打 Tag: `git tag v<version>`
6. 自动触发 CI 构建发布

### 7.3 Changelog 格式

```markdown
## [1.0.0] - 2026-06-26

### 新增
- 微软 OAuth 2.0 登录支持
- 版本管理功能
- Mod 管理功能

### 变更
- 重构认证状态管理

### 修复
- 修复下载进度显示错误

### 移除
- 移除过时的 API
```

---

## 8. AI Agent 开发约束

### 8.1 必须遵守

1. **类型安全**: 前端必须使用 TypeScript，避免 `any` 类型
2. **错误处理**: 所有 Tauri 命令必须使用 `Result` 返回
3. **日志记录**: 重要操作必须记录日志
4. **文档**: 所有公开 API 必须有文档注释
5. **测试**: 新功能必须有对应测试

### 8.2 禁止行为

1. ❌ 在 Rust 代码中使用 `unwrap()`, `expect()`, `panic!()`
2. ❌ 在 TypeScript 中使用 `any` 类型
3. ❌ 提交未通过 lint 检查的代码
4. ❌ 修改代码后不更新 CHANGELOG
5. ❌ 提交不完整的功能

### 8.3 代码模板

#### Tauri 命令模板

```rust
/// 命令功能描述
///
/// # Arguments
/// * `state` - 应用状态
/// * `param` - 参数说明
///
/// # Returns
/// 成功返回数据，失败返回错误信息
#[tauri::command]
pub async fn example_command(
    state: State<'_, AppState>,
    param: String,
) -> Result<ExampleData, String> {
    log::info!("Executing example command with param: {}", param);
    
    // 1. 参数验证
    if param.is_empty() {
        log::warn!("Empty parameter received");
        return Err("Parameter cannot be empty".to_string());
    }
    
    // 2. 业务逻辑
    let result = state.sdk.lock().await.some_function(&param)
        .map_err(|e| {
            log::error!("Command failed: {}", e);
            e.to_string()
        })?;
    
    // 3. 返回结果
    log::info!("Command executed successfully");
    Ok(result)
}
```

#### Vue 组件模板

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

### 8.4 AI Agent 开发检查清单

在提交代码前，AI Agent 必须确认：

- [ ] 所有 Tauri 命令使用 `Result` 返回
- [ ] 所有错误转换为字符串返回
- [ ] 所有重要操作记录日志
- [ ] 所有公开 API 有文档注释
- [ ] 新功能有测试覆盖
- [ ] 前端代码通过 lint 检查
- [ ] 后端代码通过 clippy 检查
- [ ] 更新 CHANGELOG.md

---

## 附录 A: 工具配置

### ESLint 配置

```javascript
// .eslintrc.cjs
module.exports = {
  root: true,
  extends: [
    'plugin:vue/vue3-recommended',
    'eslint:recommended',
    '@vue/eslint-config-typescript',
    '@vue/eslint-config-prettier/skip-formatting',
  ],
  parserOptions: {
    ecmaVersion: 'latest',
  },
  rules: {
    'vue/multi-word-component-names': 'off',
  },
}
```

### Prettier 配置

```json
// .prettierrc
{
  "semi": false,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5",
  "printWidth": 100
}
```

### rustfmt 配置

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
```

### clippy 配置

```toml
# clippy.toml
too-many-arguments-threshold = 8
type-complexity-threshold = 300
```

---

## 附录 B: 常用命令

```bash
# 前端代码检查
npm run lint

# 前端类型检查
npm run typecheck

# 前端测试
npm run test

# 后端代码检查
cd src-tauri && cargo clippy -- -D warnings

# 后端代码格式化
cd src-tauri && cargo fmt

# 后端测试
cd src-tauri && cargo test

# 开发模式运行
npm run tauri dev

# 构建发布版本
npm run tauri build
```

---

*本文档最后更新于 2026-06-26*
