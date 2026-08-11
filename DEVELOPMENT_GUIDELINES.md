# MoLaunch 开发规范

> **适用范围**：所有参与 MoLaunch 开发的人员
> **最后更新**：2026-08-08

---

## 目录

1. [代码风格规范](#一代码风格规范)
2. [Git 提交规范](#二git-提交规范)
3. [分支管理策略](#三分支管理策略)
4. [文档编写规范](#四文档编写规范)
5. [测试编写规范](#五测试编写规范)
6. [开发工具配置](#六开发工具配置)
7. [常用命令速查](#七常用命令速查)

---

## 一、代码风格规范

### 1.1 前端（Vue 3 + TypeScript）

#### 技术栈

Vue 3 + TypeScript + Vite + Pinia + Vue Router + Tailwind CSS + @heroicons/vue + skinview3d + vue-virtual-scroller + openlayers（种子地图，WASM 由 cubiomes 编译）

#### 命名规范

| 对象 | 规范 | 示例 |
|------|------|------|
| 组件文件名 | PascalCase.vue | `VersionList.vue` |
| 组件名 | PascalCase | `VersionList` |
| 组合式函数 | useXxx | `useAuth` |
| 变量 / 函数 | camelCase | `getUserProfile` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_RETRY_COUNT` |
| 类型 / 接口 | PascalCase | `UserProfile` |

#### TypeScript 类型规范

【必须】为所有变量、函数参数、返回值显式标注类型。
【禁止】使用 `any` 类型；如确实无法确定类型，使用 `unknown` 并配合类型守卫。

```typescript
// 正确
interface UserProfile {
  id: string
  username: string
  avatar?: string
}

const getUserProfile = async (id: string): Promise<UserProfile> => {
  // ...
}

// 错误：缺少类型标注
const getUserProfile = async (id) => {
  // ...
}

// 错误：使用 any
const data: any = await fetch(...)
```

#### Vue 组件规范

- 所有组件必须使用 `<script setup lang="ts">` + Composition API
- **组件文件不超过 300 行**；超长时应拆分（提取子组件 / composable / 工具函数）
- 可复用逻辑必须提取到 `src/composables/` 或 `src/utils/`，不得在 Vue 文件中重复代码
- 组件文档注释：文件头部包含功能描述、特性列表、用法示例

```vue
<script setup lang="ts">
/**
 * 版本卡片组件
 *
 * 显示单个版本的信息，支持下载、删除操作。
 *
 * 用法：
 * <VersionCard :version="version" @download="handleDownload" />
 */
import { computed, ref } from 'vue'
import Button from '@/components/common/Button.vue'

interface Props {
  /** 版本信息对象 */
  title: string
  count?: number
}

const props = withDefaults(defineProps<Props>(), {
  count: 0,
})

const emit = defineEmits<{
  select: [id: string]
  delete: [id: string]
}>()

const loading = ref(false)
const displayTitle = computed(() => props.title.toUpperCase())

const handleSelect = (id: string) => {
  emit('select', id)
}
</script>
```

#### 组合式函数（Composable）规范

- 文件位置：`src/composables/`
- 命名：`useXxx.ts`
- 必须返回响应式状态（`ref` / `computed`）和方法
- 涉及 Tauri 事件监听时使用 `useTauriEvent` 封装，自动管理 unlisten 与 onUnmounted 清理

```typescript
// src/composables/useExample.ts
import { ref, onMounted } from 'vue'
import { useTauriEvent } from '@/composables/useTauriEvent'

export function useExample() {
  const data = ref<ExampleData | null>(null)
  const loading = ref(false)

  const { start } = useTauriEvent<ExamplePayload>('example-event', (payload) => {
    data.value = payload
  })

  onMounted(() => start())

  return { data, loading }
}
```

#### 工具函数（utils）规范

- 通用能力放在 `src/utils/`：IPC 封装（`utils/api/`）、Markdown 渲染（`utils/markdown.ts`，基于 marked + DOMPurify，用 `renderMarkdown` / `handleMarkdownLinkClick`）、Toast（`utils/toast.ts`）、Modal（`utils/modal.ts`）、版本比较（`utils/version.ts`）、更新日志（`utils/updateLog.ts`）等
- **禁止在组件内重复实现已有工具函数**，一律从 utils 引入
- `v-html` 渲染用户/构建内容时，必须经过 `renderMarkdown`（DOMPurify 消毒），点击事件复用 `handleMarkdownLinkClick`

#### 更新日志「作者的话」（多 note）约定

- 更新弹窗 `UpdateLogDialog` 顶部可展示多条「作者的话」
- **commit message 以 `note:` 开头**即为作者寄语（如 `note: 感谢大家的支持`），同一版本区间内全部 `note:` 提交按顺序展示
- 数据来源：vite 构建时 `updateLogPlugin`（vite.config.ts）从 git 提交提取，虚拟模块 `virtual:update-log` 导出 `notes: string[]`；前端经 `getChangelogNotes()` 读取
- `note:` 提交默认同样不带 `!c`，且不会出现在 commit 列表中（`ReleaseTimeline` 不渲染）
- 无 note 时弹窗不渲染该区块，向后兼容

### 1.2 后端（Rust + Tauri 2）

#### 技术栈

Tauri 2 + Rust + tokio + reqwest + serde + zip + sha1 / sha2 / md5 + sysinfo；联机模块含 WebRTC 信令、虚拟 TUN（wintun/tun）、FRP 隧道；AI 模块为 OpenAI 兼容 SSE 客户端；种子地图通过 cubiomes WASM 调用。后端 `minecraft` 模块为纯 Rust 实现，不依赖外部 C FFI（cubiomes 以 WASM 形式内嵌）。

#### 命名规范

| 对象 | 规范 | 示例 |
|------|------|------|
| 模块名 | snake_case | `version_list` |
| 函数名 | snake_case | `parse_timestamp` |
| 类型 / 结构体 / 枚举 | PascalCase | `AppState` |
| 常量 | SCREAMING_SNAKE_CASE | `MAX_THREADS` |
| 生命周期 | 'lowercase | `'a` |

#### 错误处理规范

【必须】所有可能失败的操作使用 `Result` 返回。
【必须】Tauri 命令统一返回 `Result<T, String>`，将错误转换为字符串。
【禁止】在生产代码中使用 `unwrap()` / `expect()` / `panic!()`。

```rust
// 正确：内部函数使用具体错误类型
pub fn parse_version(json: &str) -> Result<VersionInfo, serde_json::Error> {
    serde_json::from_str(json)
}

// 正确：Tauri 命令转换为字符串
#[tauri::command]
pub async fn get_version(id: String) -> Result<VersionInfo, String> {
    fetch_raw(&id)
        .await
        .map_err(|e| e.to_string())
        .and_then(|raw| parse_version(&raw).map_err(|e| e.to_string()))
}
```

#### 状态管理规范

应用全局状态通过 `AppState` 结构体管理，字段使用 `Arc<tokio::sync::Mutex<T>>` 共享。Tauri 命令通过 `State<'_, AppState>` 注入，`lock()` 后及时 `drop()` 释放锁，避免长时间持有阻塞其他命令。

```rust
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn example_command(state: State<'_, AppState>) -> Result<ExampleData, String> {
    let config = state.config.lock().await;
    let game_dir = resolve_game_dir(&config.game_dir);
    drop(config); // 释放锁后再执行后续耗时操作
    // ...
}
```

对于频繁出现的 lock / clone / drop 套件，优先复用 `src-tauri/src/state/` 提供的 helper（如 `resolve_game_dir_from_state`、`resolve_mirror_and_source`）。

#### 架构硬约束

| 约束 | 说明 |
|------|------|
| 配置读写 | 统一走 `apply_config` / `get_config`，禁止新增 `set_*` / `get_*` 单字段命令 |
| shell 调用 | 必须走 `crate::minecraft::system::shell` 模块 |
| 资源访问 | 必须用 `crate::resources::read_resource()` |
| 下载源配置 | 必须拆分为 `download_source` 与 `meta_source` 两个独立字段 |
| 下载进度 | 必须增量更新 `progress.downloaded_bytes`，失败时回滚 |
| CurseForge | secure_storage 必须懒加载 |

#### 日志规范

项目使用自定义日志宏（`src-tauri/src/logger/`），非 `log` crate：

| 宏 | 用途 | 示例 |
|----|------|------|
| `log_info!` | 重要操作 / 初始化 | `log_info!("Fetching version list")` |
| `log_warn!` | 警告情况 | `log_warn!("Config not found, using defaults")` |
| `log_error!` | 错误（可恢复） | `log_error!("Failed to download: {}", e)` |
| `log_debug!` | 调试信息 | `log_debug!("Parsed {} entries", entries.len())` |
| `log_trace!` | 详细跟踪 | `log_trace!("Processing file: {}", path)` |

日志系统会自动对 token 等敏感信息脱敏（logger 内置 sanitize），但仍应避免在日志中直接打印敏感数据。前端日志查看器按级别配色：ERROR=red-400、WARN=yellow-400、INFO=green-400、DEBUG=cyan-400、TRACE=slate-500。

#### Tauri 命令模板

```rust
use crate::state::AppState;
use crate::{log_error, log_info, log_warn};
use tauri::State;

/// 命令功能描述
///
/// # Arguments
/// * `state` - 应用状态
/// * `param` - 参数说明
///
/// # Returns
/// 成功返回数据，失败返回错误信息字符串
#[tauri::command]
pub async fn example_command(
    state: State<'_, AppState>,
    param: String,
) -> Result<ExampleData, String> {
    if param.is_empty() {
        log_warn!("Empty parameter received");
        return Err("参数不能为空".to_string());
    }

    log_info!("Executing example command: {}", param);

    let config = state.config.lock().await;
    let game_dir = resolve_game_dir(&config.game_dir);
    drop(config);

    let result = do_something(&game_dir, &param)
        .await
        .map_err(|e| {
            log_error!("Command failed: {}", e);
            e.to_string()
        })?;

    log_info!("Command executed successfully");
    Ok(result)
}
```

新增 Tauri 命令的完整流程：

1. 在 `src-tauri/src/commands/` 对应模块中添加函数
2. 添加 `#[tauri::command]` 属性
3. 使用 `State<'_, AppState>` 注入状态
4. 返回 `Result<T, String>`
5. 添加文档注释（`///`）
6. 在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册命令
7. 在前端 `src/types/` 创建对应的 TypeScript 类型
8. 前端经 `src/utils/api/` 统一封装调用

#### 与 api-server（MoLaunch 云端）联动的约定

- api-server 是独立仓库（`api-server/`，remote 指向 Molaunch-ApiServer），改动需两侧对齐
- 启动器与云端通信协议：MoSign / MoSign-v2 签名 + ECIES 信封加密，注册 / 登录 / 刷新等接口前置 PoW 工作量证明
- PoW 响应为结构化 DTO（`PowChallengeResponse`，含 `header_name`），客户端**不得硬编码请求头字段名**，必须从服务端下发字段动态读取
- 服务端下载源相关配置必须拆分为 `download_source` 与 `meta_source` 两个独立字段

### 1.3 UI 组件规范

#### 必须使用项目自定义组件

【必须】UI 交互组件必须使用项目自定义组件，禁止引入第三方 UI 库（如 Headless UI、Element Plus 等）。

项目自定义组件位于 `src/components/common/`：

| 组件 | 用途 |
|------|------|
| Button | 按钮（primary / secondary / outline / ghost / text） |
| Input | 输入框（支持 prefix / suffix / clearable / textarea） |
| Select | 下拉选择（禁止使用原生 `<select>`） |
| Drawer | 抽屉（从右侧滑出的面板） |
| Modal | 弹窗（通过 `utils/modal.ts` 全局调用） |
| Toast | 全局提示（通过 `utils/toast.ts` 全局调用） |
| Tooltip | 文字提示（禁止使用 `title` 属性） |
| Alert | 警告提示 |
| BackToTop | 回到顶部 |
| Slider | 滑条 |

#### 风格参考：Arco Design

组件视觉风格参考 Arco Design Vue（https://arco.design/），关键规范：

- 默认高度：32px（mini 24px / small 28px / large 36px）
- 圆角：2px（小元素）/ 4px（卡片、容器）
- 主色：`brand-2 #0b5bcb`（主蓝）
- 悬停态：`brand-3 #1370f3`
- 页面背景：`page #f0f5ff`
- 输入框默认灰底 `#f2f3f5`，focus 时白底蓝边框
- 图标统一使用 `@heroicons/vue`，不使用 Emoji
- UI 采用单列布局（参考 PCL2）
- 空状态提示必须 icon + text 垂直水平居中

#### 全局组件调用方式

Modal 和 Toast 通过全局工具函数调用，无需在模板中手动管理实例：

```typescript
import { showError, showConfirm, showPrompt } from '@/utils/modal'
import { toastSuccess, toastError, toastWarning, toastInfo } from '@/utils/toast'

// Modal 弹窗
showError('错误', '下载失败，请重试')
showConfirm('确认', '是否删除该版本？', () => {
  // 确认回调
})
showPrompt('输入名称', '请输入版本名称', (value) => {
  // 确认回调，接收输入值
})

// Toast 全局提示
toastSuccess('保存成功')
toastError('操作失败')
toastWarning('内存不足')
toastInfo('提示信息')
```

注意：当文件同时引入 `modal` 和 `toast` 时，`toast.ts` 的 `showSuccess` / `showError` 等别名与 `modal.ts` 同名易混淆，必须使用 `toastXxx` 前缀函数。

---

## 二、Git 提交规范

### 2.1 提交信息格式

```
<type>(<scope>): <subject>

<body>

<footer>
```

### 2.2 type 类型

| type | 说明 | 示例 |
|------|------|------|
| feat | 新功能 | `feat(auth): 实现登录流程` |
| fix | 修复 Bug | `fix(download): 修复断点续传异常` |
| refactor | 重构（不改外部行为） | `refactor(state): 提取状态访问 helper` |
| perf | 性能优化 | `perf(render): 优化列表渲染` |
| docs | 文档更新 | `docs: 更新开发规范` |
| test | 测试相关 | `test(auth): 添加认证测试` |
| ci | CI / CD 配置 | `ci: 更新构建流程` |
| chore | 构建 / 工具 / 依赖 | `chore: 更新依赖版本` |
| style | 代码格式 | `style: 格式化代码` |

### 2.3 scope 范围

| scope | 说明 |
|-------|------|
| frontend | 前端相关 |
| backend | 后端相关 |
| auth | 认证模块 |
| version | 版本管理 |
| mod | Mod 管理 |
| skin | 皮肤管理 |
| java | Java 管理 |
| settings | 设置 |
| ui | UI 组件 |
| state | 状态管理 |
| commands | Tauri 命令 |
| community | 社区 / 资源 |
| download | 下载管理 |
| launch | 启动流程 |
| online | 联机（信令 / P2P / FRP） |
| frp | FRP 隧道 |
| ai | AI 对话模块 |
| plugins | 插件 SDK |
| tools | 工具模块 |

### 2.4 `!c` 后缀（可选，跳过 CI）

提交信息默认不带 `!c` 标记。CI 流水线检测 commit message 是否包含 `!c` 子串，包含则跳过本次推送触发的 CI，避免重复占用构建资源；仅在需要跳过本次推送触发的构建时附加。

### 2.5 `note:` 前缀（作者的话）

- commit message 以 `note:` 开头表示「作者的话」，会被 `updateLogPlugin` 提取进更新日志弹窗顶部（支持多条）
- `note:` 提交默认同样不带 `!c`（如 `note: 感谢大家的支持`）
- `note:` 提交不需要任何文件变动，使用空提交创建：`git commit --allow-empty -m "note: 感谢大家的支持"`
- 【发布顺序】`note:` 提交必须在**打版本 tag 之前**执行——插件按「上一 tag → 最新 tag」区间提取，tag 之后再提交的 note 会落到下一个版本区间

```bash
git commit --allow-empty -m "note: 感谢大家的支持"
# ... 其他发版提交完成后 ...
git tag v0.3.5-rc1
git push origin main --tags
```

### 2.6 常规泛化描述约束

【必须】提交信息使用常规、泛化的技术描述，严禁出现敏感引用或第三方项目代号（如 PCL2、pcl 等）。描述应遮掩实际意图，使用通用术语。

- 允许：`refactor(state): 提取状态访问 helper 消除重复套件`
- 禁止：`refactor(state): 参考 PCL2 实现 FileSystemWatcher`

### 2.7 完整示例

```
feat(version): 实现版本下载功能

- 支持版本列表获取
- 支持版本下载与进度显示
- 支持断点续传
```

```
fix(auth): 修复登录超时问题

- 增加轮询超时时间
- 优化错误提示信息
```

---

## 三、分支管理策略

### 3.1 分支类型

| 分支 | 命名 | 用途 | 生命周期 |
|------|------|------|----------|
| 主分支 | `main` | 稳定发布版本 | 永久 |
| 功能分支 | `feat/<name>` | 新功能开发 | 临时 |
| 修复分支 | `fix/<name>` | Bug 修复 | 临时 |

### 3.2 工作流程

```
main ─────────────────────────────────────────────►
  │                                  ▲
  │  ┌─ feat/version-download ───────┤
  └──┤                                │
     └─ fix/auth-timeout ─────────────┘
```

1. 从 `main` 创建分支：`git checkout -b feat/version-download`
2. 开发完成后确保本地检查全部通过（cargo fmt / clippy / lint / typecheck）
3. 创建 PR 合并到 `main`
4. 合并后删除功能分支

### 3.3 合并规则

- `feat/*` 和 `fix/*` 通过 PR 合并到 `main`
- PR 需通过 CI 检查（lint / typecheck / clippy / build）
- 紧急修复可直接合并到 `main`，但必须确保本地检查通过

---

## 四、文档编写规范

### 4.1 代码注释

#### 前端

```typescript
/**
 * 获取用户资料
 *
 * @param id - 用户 ID
 * @returns 用户资料对象
 * @throws 当用户不存在时抛出错误
 *
 * @example
 * const user = await getUserProfile('123')
 * console.log(user.username)
 */
async function getUserProfile(id: string): Promise<UserProfile> {
  // ...
}
```

#### 后端

```rust
/// 获取版本列表
///
/// # Arguments
/// * `state` - 应用状态
///
/// # Returns
/// 成功返回版本列表，失败返回错误信息字符串
///
/// # Examples
/// ```ignore
/// let result = list_versions(state).await?;
/// println!("Found {} versions", result.versions.len());
/// ```
#[tauri::command]
pub async fn list_versions(state: State<'_, AppState>) -> Result<VersionListResult, String> {
    // ...
}
```

### 4.2 文档文件

- 根目录三份核心文档分工：`AI_AGENT_GUIDELINES.md`（AI 协作行为约束）、`DEVELOPMENT_GUIDELINES.md`（开发规范）、`DEVELOPMENT_BLUEPRINT.md`（架构蓝图）
- 新增设计 / 方案请写入 `docs/` 下，重大功能在 `CHANGELOG.md` 关联说明
- `docs/fix-bug/fix-message.md` 为通用违规排查规范，涉及文件拆分、行数、注释、测试位置等约束

---

## 五、测试编写规范

### 5.1 前端测试（vitest）

测试文件放置于对应目录的 `__tests__/` 子目录中。

```typescript
// src/composables/__tests__/useAuth.test.ts
import { describe, it, expect } from 'vitest'
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

测试命名格式：`should <行为> when <条件>` 或 `should <行为> with <输入>`。

### 5.2 后端测试（cargo test）

【必须】测试代码不内联在源文件中，放同目录 `xxx_test.rs`，源文件以 `#[cfg(test)] mod xxx_test;` 声明引用（遵循 `docs/fix-bug/fix-message.md` 规范）。

```rust
// src-tauri/src/utils/example_test.rs
use super::*;

#[test]
fn test_parse_timestamp_iso8601() {
    let ts = parse_timestamp("2026-01-15T10:30:00+00:00");
    assert!(ts > 0);
}

#[test]
fn test_parse_timestamp_invalid_returns_zero() {
    let ts = parse_timestamp("invalid");
    assert_eq!(ts, 0);
}
```

测试命名格式：`test_<被测功能>_<场景>_<预期结果>`。

### 5.3 测试覆盖率目标

| 模块 | 覆盖率目标 |
|-----------|-----------|
| 前端组合式函数（composables） | 80%+ |
| 前端组件（components） | 70%+ |
| Rust 命令层（commands） | 90%+ |
| Rust 核心模块（minecraft） | 95%+ |

---

## 六、开发工具配置

### 6.1 ESLint（`.eslintrc.cjs`）

```javascript
/* eslint-env node */
require('@rushstack/eslint-patch/modern-module-resolution')

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

### 6.2 Prettier（`.prettierrc`）

```json
{
  "semi": false,
  "singleQuote": true,
  "tabWidth": 2,
  "trailingComma": "es5",
  "printWidth": 100
}
```

### 6.3 rustfmt（`rustfmt.toml`）

```toml
edition = "2021"
max_width = 100
tab_spaces = 4
use_field_init_shorthand = true
use_try_shorthand = true
```

### 6.4 clippy

使用 `cargo clippy -- -D warnings` 将所有警告视为错误。CI 中同样以此标准检查：

```bash
cargo clippy --all-features -- -D warnings
```

---

## 七、常用命令速查

```bash
# ---------- 前端 ----------
npm run dev                  # 启动开发服务器
npm run build                # 构建前端
npm run lint                 # ESLint 检查并自动修复
npm run typecheck            # TypeScript 类型检查
npm run test                 # vitest 测试

# ---------- 后端（在 src-tauri 目录执行）----------
cargo fmt                    # 格式化
cargo clippy -- -D warnings  # clippy 检查（0 warnings）
cargo test                   # 运行测试

# ---------- MoLaunch 云端（在 api-server 目录执行）----------
cargo clippy --all-targets   # clippy 检查（0 warnings）
cargo test --lib             # 运行单元测试

# ---------- Tauri ----------
npm run tauri dev            # 开发模式运行
npm run tauri build          # 构建发布版本

# ---------- Git 提交 ----------
git add -A
git commit -m "feat(version): 实现版本下载功能"
git push origin main
```

---

*本文档最后更新于 2026-08-11*
