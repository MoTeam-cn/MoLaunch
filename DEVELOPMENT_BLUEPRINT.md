# MoLaunch 开发蓝图 (Development Blueprint)

> **版本**: v2.0.0
> **更新日期**: 2026-07-20
> **维护者**: MoLaunch Team

---

## 目录

1. [项目架构总览](#1-项目架构总览)
2. [模块依赖关系](#2-模块依赖关系)
3. [核心数据流](#3-核心数据流)
4. [UI 设计规范](#4-ui-设计规范)
5. [状态管理策略](#5-状态管理策略)
6. [安全规范](#6-安全规范)
7. [开发工具配置](#7-开发工具配置)

---

## 1. 项目架构总览

MoLaunch 是一款基于 Tauri 2 的现代化 Minecraft 启动器。后端核心业务逻辑完全使用 Rust 实现（`minecraft/` 模块），通过 Tauri 命令层向前端暴露 IPC 接口；前端使用 Vue 3 + TypeScript 构建自定义组件库，整体 UI 风格参考 Arco Design。

### 1.1 后端目录结构 (src-tauri/src/)

```
├── commands/          # Tauri 命令层（IPC 入口）
│   ├── auth/          # 认证命令（account, microsoft, offline）
│   ├── community/     # 社区资源命令
│   │   ├── install/   # 安装（concurrent, curseforge, modrinth, modpack_stages, helpers, types）
│   │   ├── community_config.rs
│   │   ├── detail.rs
│   │   ├── search.rs
│   │   └── secure_config.rs
│   ├── system/        # 系统命令
│   │   ├── apply_config/  # 配置应用（apply, secure, validate, types）
│   │   ├── config.rs, developer.rs, download.rs
│   │   ├── game.rs, game_dir.rs, proxy.rs
│   ├── version/       # 版本命令
│   │   ├── install/   # 安装（cleanup, fabric_api, loader_helpers, post_install, setup_persist, stages, version_naming）
│   │   ├── mods/      # Mod 管理（metadata/sources, helpers, types, watcher）
│   │   ├── download.rs, folder.rs, launch.rs, list.rs, loaders.rs
│   │   ├── manage.rs, personalization.rs, preload.rs, progress.rs
│   │   ├── script_export.rs, types.rs
│   ├── image_cache.rs
│   ├── java.rs
│   ├── sdk.rs
│   └── skin.rs
├── minecraft/         # 核心业务模块（纯 Rust 实现）
│   ├── auth/          # 认证
│   │   ├── microsoft/ # 微软登录（config, exchange, oauth, types）
│   │   └── storage/   # 凭证存储（operations, registry, types）
│   ├── community/     # 社区资源
│   │   ├── curseforge/  # CurseForge（convert, http, types）
│   │   ├── modrinth/    # Modrinth（convert, http, types）
│   │   ├── preload/     # 预加载（cache, hash, jar_metadata, online_query, types）
│   │   ├── cache.rs, common.rs, mcmod.rs, searcher.rs
│   │   ├── secure_storage.rs, tags.rs, version_extract.rs, types.rs
│   ├── download/      # 下载
│   │   ├── chunk/      # 分块下载（download, merge, probe, util）
│   │   ├── assets.rs, downloader.rs, full_download.rs, manager.rs
│   │   ├── rate_limiter.rs, stages.rs, types.rs, util.rs
│   │   ├── version_list.rs, fix.rs
│   ├── java/          # Java 管理（detect, search, select, download/...）
│   ├── java_selector/ # Java 选择器（compat, installer, rules, select, weight, tests）
│   ├── launch/        # 启动
│   │   ├── pipeline/  # 启动流水线（execute, java_check, natives, pre_launch, process_spawn, validate, types）
│   │   ├── watcher/   # 进程监控（analyzer/collect/crit1/crit3/stack/util, log_parser, types, window_title）
│   │   ├── arguments.rs, classpath.rs, embedded.rs, game_args.rs, jvm_args.rs
│   ├── loaders/       # 加载器（fabric, fabric_api, forge, forge_html, forge_installer, neoforge, optifine, liteloader, shared, utils）
│   ├── system/        # 系统调用（shell）- 所有系统级 shell 命令必须走此模块
│   ├── utils/         # 工具（file_checker, maven）
│   ├── version/       # 版本（setup/helpers/load/save/types/update/tests, json_merge, libraries, scan, state）
│   ├── fools.rs, image_cache.rs, isolation.rs, language.rs
│   ├── launcher_profiles.rs, skin.rs, sources.rs
├── sdk/               # SDK 兼容层（ffi_types, instance, types）
├── state/             # 应用状态（app, auth, config, download, launch）
├── storage/           # 存储（cache, ini, registry）
├── config.rs, error_util.rs, http.rs, lib.rs, logger.rs, main.rs, resources.rs
```

### 1.2 前端目录结构 (src/)

```
├── assets/            # 静态资源
│   ├── styles/main.css
│   ├── Skins/         # 默认皮肤
│   ├── Mods/          # Mod 默认图标
│   └── blocks/        # 方块图标
├── components/
│   ├── common/        # 通用组件（Button, Input, Select, Modal, Toast, Tooltip,
│   │                  #   CrashDialog, DeviceCodeModal, DownloadPanel, LoaderCard,
│   │                  #   SegmentedButtons, SkinManager, SkinAvatar, SkinModel3D,
│   │                  #   skin-manager/*, Alert, BackToTop, HorizontalFilter, MultiSelectBar）
│   ├── community/     # 社区资源（SearchBar, ResourceCard, ResourceDetail, Pagination, resource-detail/*）
│   ├── downloads/     # 下载管理（TaskGroupCard）
│   ├── home/          # 首页（LaunchPanel, VersionSelector, LaunchLog, AccountSelector, account-selector/*）
│   ├── install/       # 安装（FabricApiInfoCard）
│   ├── layout/        # 布局（TopNavLayout）
│   ├── settings/      # 设置（LogViewer, DevModeToggle, ToggleRow）
│   ├── version/       # 版本（InstalledList, VersionSection）
│   └── version-settings/  # 版本设置（AdvanceFieldsPanel）
├── composables/       # 组合式函数（约 24 个，见 5.2 节）
├── router/            # 路由配置
├── stores/            # Pinia 状态（auth, java, sdk, settings, version）
├── types/             # TypeScript 类型定义（auth, community, download, java, settings, version）
├── utils/
│   ├── api/           # API 封装（auth, community, config, developer, image-cache,
│   │                  #   java, launch, loader, personalization, sdk, skin, system, version）
│   └── *.ts           # 工具函数（async, format, modal, toast, tauri, crashDialog,
│                      #   cape-icon, default-skin, image-crop, log-display, mod-display, system-display, version）
├── views/             # 页面视图
│   ├── Home.vue, Login.vue, Settings.vue, Community.vue
│   ├── Downloads.vue, Versions.vue, VersionSelect.vue, VersionSettings.vue
│   ├── LoaderSelect.vue
│   ├── downloads/      # DownloadSidebar, DownloadStatsPanel
│   ├── settings/       # SettingsAdvanced/Developer/Download/Launch/Other/Personal, settings-launch/*
│   ├── version-select/ # FolderSidebar
│   └── version-settings/  # mod-tab/*, setup-tab/*, JavaDownloadBar, MemorySection, ModTab, OverviewTab, SetupTab
├── App.vue
└── main.ts
```

### 1.3 技术栈

**前端技术栈**：

| 技术 | 版本 | 用途 |
|------|------|------|
| Vue 3 | ^3.4 | 前端框架（Composition API） |
| TypeScript | ^5.3 | 类型系统 |
| Vite | ^5.0 | 构建工具 |
| Pinia | ^2.1 | 状态管理 |
| Vue Router | ^4.2 | 路由管理 |
| Tailwind CSS | ^3.4 | 原子化样式框架 |
| @heroicons/vue | ^2.2 | 图标库 |
| skinview3d | ^3.4 | 3D 皮肤渲染 |
| vue-virtual-scroller | ^3.0 | 虚拟滚动列表 |
| @tauri-apps/api | ^2.11 | Tauri 前端 API |

**后端技术栈**：

| 技术 | 版本 | 用途 |
|------|------|------|
| Tauri | 2 | 桌面应用框架（含 shell / dialog / fs / notification 插件） |
| Rust | edition 2021 | 后端语言 |
| tokio | ^1.35 | 异步运行时（sync / rt / macros / process） |
| reqwest | ^0.11 | HTTP 客户端（json / stream / multipart / gzip） |
| serde / serde_json | ^1.0 | 序列化 |
| zip | ^2 | 压缩包处理 |
| sha1 / sha2 / md5 | 0.10 / 0.10 / 0.7 | 哈希校验 |
| sysinfo | ^0.29 | 系统信息（内存检测等） |
| libloading | ^0.8 | 动态库加载（SDK lite 兼容层） |
| notify | ^8 | 文件系统监听（mods 目录自动刷新） |
| regex / anyhow / chrono | 1 / 1 / 0.4 | 正则 / 错误处理 / 时间 |

平台特定依赖：Windows 使用 `winreg`（注册表访问）与 `windows`（Win32 API，用于游戏窗口标题修改）。

### 1.4 模块职责划分

| 模块 | 职责 | 实现语言 |
|------|------|----------|
| `commands/` | Tauri 命令层，IPC 入口，参数校验与结果序列化 | Rust |
| `minecraft/` | 核心业务逻辑（纯 Rust，不依赖 Tauri 运行时） | Rust |
| `sdk/` | SDK lite 兼容层（动态库加载、FFI 类型转换） | Rust |
| `state/` | 应用全局状态（AppState 聚合各 Arc 句柄） | Rust |
| `storage/` | 持久化存储（ini 配置、registry 注册表、cache 缓存） | Rust |
| `components/` | UI 组件库 | Vue 3 + TypeScript |
| `views/` | 页面视图 | Vue 3 + TypeScript |
| `stores/` | 前端状态管理 | Pinia |
| `composables/` | 可复用组合式逻辑 | Vue 3 Composition API |
| `utils/api/` | 后端 IPC 调用封装 | TypeScript |

---

## 2. 模块依赖关系

本节以文字描述各模块之间的依赖方向，箭头表示「依赖于」。整体遵循分层架构：前端调用命令层，命令层调用业务层，业务层调用底层工具与状态。

### 2.1 后端分层依赖

- **commands/ 依赖 minecraft/ 和 state/**：所有 Tauri 命令在接收前端 IPC 调用后，通过 `state::AppState` 获取共享状态（配置、下载状态、启动历史、SDK 句柄等），再委托给 `minecraft/` 下对应业务模块执行。命令层自身不实现业务逻辑，只做参数组装、状态句柄获取与结果转换。
- **minecraft/auth 依赖 minecraft/auth 的 storage 和 microsoft 子模块**：`microsoft/`（config / exchange / oauth / types）负责 OAuth 流程与 token 交换；`storage/`（operations / registry / types）负责凭证持久化与多账号管理；`commands/auth` 通过 `AppState.auth_storage`（即 `AuthStorage`）调用两者。
- **minecraft/download 依赖 minecraft/version 和 chunk 子模块**：`download/manager.rs` 与 `download/full_download.rs` 编排下载流程，从 `minecraft/version` 获取下载信息（libraries、assets、client jar 清单），具体文件下载委托给 `chunk/`（download / merge / probe / util）实现分块并发下载与断点续传。
- **minecraft/launch 依赖 minecraft/version、minecraft/java_selector、minecraft/system/shell**：`launch/pipeline/` 负责启动流水线编排，通过 `minecraft/version` 读取版本 JSON 与 libraries 计算 classpath，通过 `minecraft/java_selector` 选择兼容的 Java 运行时，进程管理（kill_process_tree）与文件权限（restrict_file_permissions）统一走 `minecraft/system/shell`。
- **minecraft/loaders 被 commands/version/install 调用**：加载器安装逻辑（fabric / fabric_api / forge / neoforge / optifine / liteloader）由 `commands/version/install/` 下的 stages 与 loader_helpers 编排，实际下载与注入由 `minecraft/loaders/` 各模块实现。
- **minecraft/community 被 commands/community 调用**：`commands/community/search` 与 `commands/community/detail` 委托给 `minecraft/community/searcher` 及 `curseforge/`、`modrinth/` 子模块；`commands/community/install` 调用 `minecraft/community/preload` 与各平台 http 模块。
- **state/ 被 commands/ 共享**：`AppState`（state/app.rs）聚合了 SDK 句柄、AppConfig、AuthState、AuthStorage、DownloadState、LaunchHistory、当前 PID、LaunchPipeline 句柄、下载取消/暂停信号等，全部以 `Arc<TokioMutex<T>>` 或 `Arc<Mutex<T>>` 形式共享给命令层。
- **storage/ 被 state/ 和 minecraft/auth/storage 使用**：`storage/` 提供 ini 读写、registry、cache 能力；`state/config` 通过 `config.rs` 加载配置文件，`minecraft/auth/storage` 的注册表通过 `storage/registry` 持久化账号信息。

### 2.2 前端依赖方向

- **views/ 依赖 components/ 和 composables/**：页面视图组合通用组件与业务组件，并通过 composables 复用跨页面逻辑（如 `useLaunchState`、`useDownloadPolling`、`useVersionSettings`）。
- **components/ 依赖 stores/ 和 utils/api/**：组件通过 Pinia store 读写全局状态，通过 `utils/api/` 封装的函数调用后端 IPC。
- **stores/ 依赖 utils/api/ 和 composables/**：store 负责状态持有与业务编排，具体 IPC 调用委托给 `utils/api/`；部分复杂逻辑（如启动状态机）由 store 委托给 composable（`useVersionStore` 委托 `useLaunchState`）。
- **utils/api/ 依赖 @tauri-apps/api**：所有后端调用最终通过 `@tauri-apps/api/core` 的 `invoke` 发起，`utils/tauri.ts` 为统一封装层。

### 2.3 前后端边界

前端与后端唯一的交互通道是 Tauri IPC：前端 `invoke('command_name', args)` 调用后端 `#[tauri::command]` 函数，参数与返回值均通过 serde 序列化。除 IPC 外，后端还通过 `app.emit(event, payload)` 主动向前端推送事件（如 `ms-login-progress`、`ms-auth-code`），前端通过 `@tauri-apps/api/event` 的 `listen` 订阅。命令注册集中在 `src-tauri/src/lib.rs` 的 `tauri::generate_handler!` 宏中。

---

## 3. 核心数据流

### 3.1 IPC 调用流程

典型的一次 IPC 调用遵循以下链路：

前端组件或 store 调用 `utils/api/` 中的封装函数（例如 `version.ts` 的 `listVersions`），该函数内部调用 `utils/tauri.ts` 的统一 `invoke` 封装，向 Tauri 运行时发起 IPC 调用。Tauri 将请求路由到 `lib.rs` 中注册的对应 `#[tauri::command]` 函数（如 `commands::version::list::list_versions`）。命令函数从注入的 `State<'_, AppState>` 获取共享状态句柄，调用 `minecraft/` 下业务模块执行实际逻辑。业务模块完成后返回 Rust 结构体，Tauri 通过 serde 将其序列化为 JSON 返回前端，前端在 `utils/api` 层反序列化为 TypeScript 类型，更新 Pinia store 或直接驱动组件渲染。

对于长耗时操作（下载、启动、安装），命令函数通常立即返回，后续进度通过两种方式同步：一是前端轮询（见 3.2），二是后端主动 emit 事件（见 3.3 与微软登录流程）。

### 3.2 下载状态同步

下载流程采用「后端持有状态 + 前端轮询」的模式，避免高频 emit 造成的开销：

后端 `minecraft/download/manager.rs` 在执行下载时，将各阶段进度写入 `state::DownloadState`（`state/download.rs`），包括每个 stage 的字节数、文件数、状态、权重，以及全局速度与字节数。`commands::version::progress::get_download_progress` 命令读取该状态并返回前端。

前端 `composables/useDownloadPolling.ts` 监听 `versionStore.downloading` 标志，当其变为 true 时启动 `setInterval`（300ms 间隔）轮询 `getDownloadProgress`。每次轮询将原始 stage 数据映射为前端 `DownloadStage` 类型，按权重计算加权百分比，检测进度回退（downloaded/total 突然变小）并记录警告日志，检测暂停状态（任意 stage 携带 `is_paused`），最后调用 `versionStore.updateProgress` 更新 store。当检测到 `is_complete` 为 true 或 `error_code` 非零时，停止轮询并调用 `versionStore.finishDownload`。

下载控制命令（暂停 / 恢复 / 取消）通过 `commands::version::progress` 下的 `pause_download`、`resume_download`、`cancel_download` 实现，它们设置 `AppState` 中的 `download_pause_flag` 或 `download_cancel_flag`（均为 `Arc<AtomicBool>`），下载管理器在每次文件循环前检查这些信号。

### 3.3 启动流程

启动流程是 MoLaunch 最复杂的数据流，涉及流水线编排、进程监控与崩溃分析：

**前端发起**：`Home.vue` 的 `LaunchPanel` 触发 `versionStore.launchGame()`，该方法委托给 `composables/useLaunchState.ts`，后者调用 `utils/api/launch.ts` 的 `launch_game`，发起 IPC 调用 `commands::version::launch::launch_game`。

**后端流水线**：`launch_game` 命令构造 `LaunchConfig` 后创建 `minecraft::launch::LaunchPipeline` 实例并存入 `AppState.launch_pipeline`，随后调用 `LaunchPipeline::execute()`（`pipeline/execute.rs`）。流水线按顺序执行以下阶段，每个阶段通过 `update_progress` 写入 `Arc<RwLock<LaunchProgress>>`：GetJava（检测 Java，委托 `java_check.rs` 与 `minecraft/java_selector`）、ValidateFiles（文件校验补全，委托 `validate.rs` 与 `minecraft/version`、`minecraft/download/fix`）、BuildArgs（构建 JVM 与游戏参数，委托 `arguments.rs`、`jvm_args.rs`、`game_args.rs`、`classpath.rs`）、PreLaunch（执行启动前命令，委托 `pre_launch.rs`）、ExtractNatives（解压原生库，委托 `natives.rs`）、LaunchProcess（启动进程，委托 `process_spawn.rs`）、WaitWindow（等待游戏窗口）。

**进程监控**：进程启动后，`pipeline` 创建 `GameWatcher`（`launch/watcher/`）包装子进程。Watcher 通过 `log_parser.rs` 解析游戏输出日志，通过 `window_title.rs` 监控窗口标题（Windows 使用 Win32 API），通过 `analyzer/` 子模块（collect / crit1 / crit3 / stack / util）进行崩溃原因分析。Watcher 持有 `GameState`、`LoadProgress`、最近日志缓冲区与退出通知 channel。

**前端同步**：前端通过两个途径获取启动状态。一是轮询 `get_launch_progress` 命令读取 `LaunchProgress`，更新 `launchProgress`、`launchStageName` 与 `javaDownloadProgress`；二是 `useLaunchState` 在启动后注册游戏退出监听，当 watcher 检测到进程退出时，若判定为崩溃则触发崩溃分析结果，前端弹出 `CrashDialog` 显示分析结论。

**停止与取消**：`stop_game` 命令调用 `LaunchPipeline::stop_game`，先通过 `mark_manual_stop` 标记 watcher 跳过崩溃分析，再通过 watcher 停止子进程（内部调用 `system/shell::kill_process_tree`）。`cancel_launch` 命令设置 `cancel_flag`，流水线在下一个阶段检查点中止。

---

## 4. UI 设计规范

### 4.1 设计风格

UI 整体参考 Arco Design Vue 的视觉语言，但在 `src/assets/styles/main.css` 中以自定义组件类实现，而非直接引入 Arco 组件库。核心设计原则：

- **紧凑现代**：默认控件高度 32px，圆角偏小（2px / 4px），符合桌面应用而非移动端的视觉密度。
- **状态明确**：每个交互控件均有 default / hover / active / disabled 四态配色，颜色变化通过 `transition: all 0.1s cubic-bezier(0, 0, 1, 1)` 平滑过渡。
- **自定义优先**：组件均在 `components/common/` 内自行实现，不依赖第三方 UI 库，保证样式可控与体积精简。

### 4.2 自定义组件清单

通用组件（`components/common/`）：

- **Button**：通过 `.btn` / `.btn-primary` / `.btn-secondary` / `.btn-outline` / `.btn-ghost` / `.btn-text` 类实现，支持 primary / secondary / outline / ghost / text 五种样式。
- **Input**：输入框，支持焦点态、错误态，焦点时 `z-index: 1` 以覆盖相邻元素边框。
- **Select**：下拉选择器。
- **Modal**：模态弹窗，z-index 10000（见 4.4）。
- **Toast**：全局提示，z-index 10001（见 4.4）。
- **Tooltip**：文字提示。
- **CrashDialog**：游戏崩溃分析对话框，z-index 9999。
- **DeviceCodeModal**：微软设备码登录模态框。
- **DownloadPanel**：下载悬浮面板（右下角 FAB）。
- **LoaderCard**：加载器卡片，使用 Tailwind safelist 动态颜色。
- **SegmentedButtons**：分段按钮组。
- **SkinManager / SkinAvatar / SkinModel3D**：皮肤管理组件，基于 skinview3d 渲染 3D 模型。
- **Alert / BackToTop / HorizontalFilter / MultiSelectBar**：辅助交互组件。

业务组件分布在 `components/community/`、`components/downloads/`、`components/home/`、`components/install/`、`components/settings/`、`components/version/`、`components/version-settings/` 下，按业务域组织。

### 4.3 配色系统

主色调（定义于 `main.css` 与 `tailwind.config.js`）：

- 主色 primary：`#165dff`（default）/ `#4080ff`（hover）/ `#0e42d2`（active）/ `#94bfff`（disabled）
- 成功 success：`#10b981`（对应 `--color-success: 16 185 129`）
- 警告 warning：`#f59e0b`（对应 `--color-warning: 245 158 11`）
- 错误 error：`#ef4444`（对应 `--color-error: 239 68 68`）

品牌色 brand（定义于 `tailwind.config.js`，用于标题与 Highlight 按钮）：

- brand-1 `#343d4a`：深灰蓝，正文 / 默认文字 / 阴影
- brand-2 `#0b5bcb`：主蓝，标题 / Highlight 按钮
- brand-3 `#1370f3`：亮蓝，悬停态边框
- brand-4 `#4890f5` / brand-5 `#96c0f9` / brand-6 `#d5e6fd` / brand-7 `#e0eafd` / brand-8 `#eaf2fe`：渐进浅色，用于背景与悬停态

弹窗配色：

- `dialog.bg #FBFBFB`：弹窗背景
- `dialog.caption #5C5C5C`：弹窗正文文字（写死，不随主题变）

页面背景 `page: #f0f5ff`，body 背景色 `#e0ecff`。

### 4.4 z-index 层级

为确保各类浮层与弹窗的叠加顺序正确，全局约定以下 z-index 层级（已在实际组件中落地）：

- **业务弹窗（z-index 9999）**：`CrashDialog`、`ResourceDetail`（社区资源详情全屏覆盖）、`SkinManager`（皮肤管理全屏覆盖）。这些是业务场景内的全屏覆盖层，使用 Tailwind `z-[9999]` 类。
- **通用 Modal（z-index 10000）**：`components/common/Modal.vue`，作为通用模态弹窗，层级高于业务弹窗，确保在业务弹窗之上仍可弹出确认框。
- **Toast 全局提示（z-index 10001）**：`components/common/Toast.vue`，层级最高，确保任何场景下的提示都能被用户看到。
- **辅助浮层（z-index 40 / 50）**：`DownloadPanel` 悬浮按钮（z-50）、`DeviceCodeModal`（z-50）、`MultiSelectBar`（z-40）、`BackToTop`（z-index 50）等局部浮层，层级低于弹窗体系。
- **局部层叠（z-index 1 / 10）**：`Input` 焦点态（z-index 1）、`HorizontalFilter` 滚动遮罩按钮（z-10）、`BackToTop` 图标（z-10）等组件内部层叠。

### 4.5 布局规范

- 默认布局为顶部导航布局（`components/layout/TopNavLayout.vue`），支持 sidebar / topnav 两种模式（由 `settingsStore.layoutMode` 控制）。
- 按钮统一高度 32px、内边距 `0 15px`、字号 14px、圆角 2px。
- 输入框与按钮遵循相同的 32px 高度基线，保证表单控件对齐。
- 动画使用 `fade-in` / `slide-in` / `slide-up` 三种关键帧（定义于 `tailwind.config.js`），时长 0.3s ease-out。

---

## 5. 状态管理策略

### 5.1 Pinia Stores

前端状态管理使用 Pinia，全部采用 Composition API（setup store）风格。共有 5 个 store，位于 `src/stores/`：

- **auth.ts**：认证状态。持有 `currentUser`、`loginStatus`、`msLoginStatus`、`msFlow`、`deviceCodeInfo`、`msAccounts`、`offlineAccounts`、`msLoginStep`、`isRestoring` 等。支持离线登录、微软登录（Web Auth Code Flow 与 Device Code Flow 两种）、token 刷新、多账号切换、会话恢复。微软登录进度通过监听 `ms-login-progress` 事件更新步骤标签，Web Flow 通过监听 `ms-auth-code` 事件接收授权码。`restoreSession` 使用 Promise 缓存防重入，避免并发触发 silent refresh 冲击 Mojang API 触发 429 风控。
- **version.ts**：版本与下载状态。持有版本列表、最新 release / snapshot、下载状态（downloading、downloadingVersion、downloadProgress）、启动状态（委托 `useLaunchState`）、selectedVersion（持久化到 config.ini）、加载器版本缓存。通过 `watch(selectedVersion)` 自动持久化选中版本。
- **settings.ts**：UI 设置（layoutMode、theme、language），持久化到 localStorage。注意后端业务配置（游戏目录、内存、下载线程等）不在此 store，而是通过 `utils/api/config.ts` 直接读写后端 config.ini。
- **java.ts**：Java 检测与选择状态。
- **sdk.ts**：SDK lite 状态（平台信息、版本、初始化状态、device_id）。

### 5.2 Composables

`src/composables/` 下约 24 个组合式函数，按职责分类：

**下载相关**：
- `useDownloadPolling`：下载进度轮询引擎（见 3.2）。
- `useDownloadTaskGroups`：下载任务分组。
- `useCommunityDownload`：社区资源下载流程。
- `useSearchProgress`：搜索进度。

**版本与安装相关**：
- `useVersionMeta`：版本元数据。
- `useVersionGroups`：版本分组。
- `useVersionInstallActions`：版本安装动作。
- `useVersionOverviewActions`：版本概览动作。
- `useVersionSettings`：版本设置。
- `useLoaderData`：加载器数据。
- `useFabricApi`：Fabric API 处理。
- `useModsPreload`：Mod 详情预加载。
- `useModDetailQuery`：Mod 详情查询。
- `useModOperations`：Mod 操作。

**启动相关**：
- `useLaunchState`：启动状态机（launchGame / stopGame / cancelLaunch / 进度轮询 / Java 下载进度 / 游戏退出监听），被 `versionStore` 委托调用。

**通用工具**：
- `usePolling`：通用轮询基础。
- `useTauriEvent`：Tauri 事件订阅封装。
- `useImageCache`：图片缓存。
- `useSkinOperations`：皮肤操作。
- `useMultiSelect`：多选。
- `useMemoryVisualizer`：内存可视化。
- `useConfigPage`：配置页面。
- `useDebouncedSave`：防抖保存。
- `useSwipeNavigation`：滑动导航。

Composables 与 stores 的协作模式：store 负责状态持有与跨页面共享，composable 负责封装具体的异步流程与副作用。复杂逻辑（如启动）由 store 委托 composable，避免 store 膨胀；纯局部逻辑（如内存可视化）则由 composable 独立持有状态，不进入全局 store。

---

## 6. 安全规范

### 6.1 Token 脱敏与 IPC 不传明文 Token

后端认证模块（`minecraft/auth/`）在设计上保证 access_token 与 refresh_token 不通过 IPC 明文传输：

- `commands/auth/account.rs` 中的 `get_ms_accounts` 与 `get_offline_accounts` 命令返回的账号信息结构体（`MsAccountInfo`、`OfflineAccountInfo`）只包含 `username`、`uuid`、`expires_at`、`is_expired`、`skin` 等非敏感字段，不包含 access_token 或 refresh_token。
- 前端 `authStore` 持有的 `currentUser`（`AuthResult` 类型）虽然包含登录态信息，但 token 相关字段不作为前端持久化对象。当前登录态通过 `get_login_status` 命令从后端恢复，后端从 `AuthStorage` 读取后返回，token 始终留在后端内存与加密存储中。
- Token 的实际使用（如启动游戏时注入 `--accessToken` 参数）完全在后端 `minecraft/launch/` 内完成，前端不接触 token 字符串。

### 6.2 凭证加密存储

`minecraft/auth/storage/` 负责凭证持久化。`AuthStorage` 通过 `state/auth_storage`（`Arc<AuthStorage>`）共享给命令层。微软账号的 access_token 与 refresh_token 在写入注册表前经过加密（DES，由 SDK lite 提供），读取时懒加载解密，避免启动时触发杀软告警（见 `lib.rs` 中 `secure_storage::init_enabled` 与 `set_sdk` 的注释）。CurseForge API key 同样通过 `minecraft/community/secure_storage` 加密存储。

### 6.3 日志自动打码

`src-tauri/src/logger.rs` 实现了统一的日志脱敏机制 `sanitize_sensitive_info`，所有写入日志文件与控制台的日志内容都会先经过该函数处理。脱敏规则使用 `regex` 与 `OnceLock` 缓存编译后的正则，识别并替换以下模式：

1. JWT 格式 token：匹配 `eyJ` 开头、三段点分隔、每段至少 10 字符的字符串，替换为 `***`。
2. JSON 中的 token 字段：匹配 `"access_token"` / `"accessToken"` / `"refresh_token"` / `"client_token"` / `"session"` / `"token"` 等键名对应的值（长度 >= 8），替换为 `***`，保留键名。
3. 超长 token 字符串：匹配长度 >= 40 的连续 base64/hex 字符串，替换为 `***`。

此外，`read_log_file` 命令在返回日志内容给前端日志查看器前，会再次调用 `sanitize_sensitive_info` 进行脱敏，双重保障前端不展示敏感信息。`read_log_file` 还做了路径遍历防护：拒绝包含 `/`、`\`、`..` 或非 `.log` 后缀的文件名。

日志系统使用自定义 `Logger` 实现（不使用 `env_logger`），避免第三方库日志绕过脱敏过滤。日志级别支持热重载（`set_level`），日志文件按日期命名（`molaunch_YYYY-MM-DD.log`）存放于 `storage/logs` 目录。

### 6.4 system/shell 模块统一调用

所有系统级 shell 命令必须通过 `minecraft/system/shell.rs` 调用，禁止业务代码直接使用 `std::process::Command`。该模块提供以下能力并统一处理跨平台差异与安全校验：

- **`open_path`**：用系统文件管理器打开文件夹。Windows 使用 `cmd /c start`（因 `explorer` 对带引号裸路径解析失败会回退到文档库），macOS 使用 `open`，Linux 使用 `xdg-open`。Windows 下通过 `creation_flags(CREATE_NO_WINDOW)` 隐藏控制台窗口。
- **`reveal_in_file_manager`**：在文件管理器中打开父目录并选中文件。Windows 使用 Win32 API `ShellExecuteW` 直接调用 explorer.exe（绕过 Rust Command 的引号转义问题），macOS 使用 `open -R`，Linux 回退到打开父目录。
- **`kill_process_tree`**：杀掉进程树（含子进程）。Windows 使用 `taskkill /PID <pid> /T /F`，Unix 使用 `kill -9 <pid>`。供 `LaunchPipeline::stop_game` 调用。
- **`restrict_file_permissions`**：限制文件权限为当前用户。Windows 使用 `icacls /inheritance:r /grant:r "<user>:F"`，Unix 使用 `chmod 600`。供启动脚本导出等场景调用，防止敏感信息被其他用户读取。

所有函数均执行 `validate_path` 安全校验：拒绝路径遍历（包含 `..`）与 UNC 路径（以 `\\` 或 `//` 开头，防止 SMB 认证泄露），并校验路径存在。调用前后统一记录 `[Shell]` 前缀日志，错误统一格式化为字符串以便 Tauri 命令直接返回。

---

## 7. 开发工具配置

### 7.1 前端工具链

**ESLint**（`.eslintrc.cjs`）：

- 基于 `@vue/eslint-config-typescript` 与 `@vue/eslint-config-prettier` 扩展。
- 检查范围：`.vue`、`.js`、`.jsx`、`.cjs`、`.mjs`、`.ts`、`.tsx`、`.cts`、`.mts` 文件。
- 忽略路径遵循 `.gitignore`。
- 通过 `npm run lint` 执行自动修复（`--fix`）。
- 通过 `npm run typecheck` 执行 `vue-tsc --noEmit` 类型检查。

**Prettier**（`^3.2.4`）：

- 与 ESLint 通过 `@vue/eslint-config-prettier` 集成，避免格式规则冲突。
- 负责代码格式化（缩进、引号、尾逗号等）。

**Vite**（`vite.config.ts`）：

- 构建工具，开发服务器通过 `npm run dev` 启动。
- 生产构建通过 `npm run build`。

**Vitest**（`^1.2.2`）：

- 前端测试框架，配合 `@vue/test-utils` 进行组件测试。
- 通过 `npm run test` 执行。

### 7.2 后端工具链

**rustfmt**：

- Rust 官方格式化工具，遵循 `rustfmt.toml`（若存在）或默认规则。
- 提交前应执行 `cargo fmt` 统一代码风格。

**Clippy**：

- Rust 官方 lint 工具，通过 `cargo clippy` 执行。
- 项目中部分模块使用 `#[allow(dead_code)]` 等属性抑制特定告警（如 `LaunchPipeline::current_stage`），应有明确注释说明原因。

**Cargo**（`Cargo.toml`）：

- 发布配置（`[profile.release]`）启用 `panic = "abort"`、`codegen-units = 1`、`lto = true`、`opt-level = "s"`、`strip = true`，优化体积与性能。
- Windows 平台在 `main.rs` 顶部通过 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` 隐藏 release 模式下的控制台窗口。

### 7.3 提交前检查清单

- 前端：`npm run lint` 通过、`npm run typecheck` 通过、`npm run test` 通过。
- 后端：`cargo fmt --check` 通过、`cargo clippy` 无警告、`cargo test` 通过。
- 涉及认证或 token 的改动：确认无明文 token 写入日志（依赖 `sanitize_sensitive_info` 自动脱敏）、无明文 token 通过 IPC 传输。
- 涉及系统调用的改动：确认走 `minecraft/system/shell` 模块，未在业务代码中直接使用 `std::process::Command`。

---

## 附录: 关键文件索引

| 关注点 | 文件路径 |
|--------|----------|
| Tauri 命令注册 | `src-tauri/src/lib.rs` |
| 应用全局状态 | `src-tauri/src/state/app.rs` |
| 启动流水线 | `src-tauri/src/minecraft/launch/pipeline/execute.rs` |
| 下载进度轮询 | `src/composables/useDownloadPolling.ts` |
| 日志脱敏 | `src-tauri/src/logger.rs`（`sanitize_sensitive_info`） |
| 系统调用统一入口 | `src-tauri/src/minecraft/system/shell.rs` |
| 认证状态管理 | `src/stores/auth.ts` |
| 前端 IPC 封装 | `src/utils/tauri.ts`、`src/utils/api/` |
| UI 样式基线 | `src/assets/styles/main.css` |
| Tailwind 配色 | `tailwind.config.js` |

---

*本文档最后更新于 2026-07-20*
