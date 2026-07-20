# MoLaunch - 现代化 Minecraft 启动器

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/MoTeam-cn/MoLaunch)
[![Tauri](https://img.shields.io/badge/Tauri-2-orange.svg)](https://tauri.app/)

一个使用 **Tauri 2 + Vue 3 + TypeScript + Rust** 构建的现代化 Minecraft 启动器。后端采用纯 Rust 实现的 `minecraft` 模块，覆盖认证、下载、启动、版本管理、社区资源等完整能力，无任何 C FFI 依赖。

- 仓库地址：<https://github.com/MoTeam-cn/MoLaunch>
- 当前版本：0.1.0
- License：MIT

## 特性

- **跨平台**：基于 Tauri 2，支持 Windows、macOS、Linux
- **现代化 UI**：Vue 3 + Tailwind CSS + @heroicons/vue，自研组件体系，支持主题定制
- **高性能**：纯 Rust 后端 + tokio 异步架构，启动快速、内存占用低
- **3D 皮肤预览**：集成 skinview3d，支持皮肤/披风实时预览
- **纯 Rust 实现**：所有 Minecraft 业务逻辑（认证、下载、启动、加载器等）均由 Rust 原生编写
- **虚拟滚动**：使用 vue-virtual-scroller 处理大型列表（版本、资源）的性能

## 技术栈

### 前端

- **框架**：Vue 3 + TypeScript
- **构建工具**：Vite
- **状态管理**：Pinia
- **路由**：Vue Router
- **UI 组件**：Tailwind CSS + @heroicons/vue（自研组件库）
- **3D 皮肤预览**：skinview3d
- **虚拟滚动**：vue-virtual-scroller

### 后端

- **框架**：Tauri 2
- **语言**：Rust（edition 2021）
- **异步运行时**：tokio
- **HTTP 客户端**：reqwest
- **序列化**：serde / serde_json
- **压缩/解压**：zip
- **哈希校验**：sha1 / sha2 / md5
- **系统信息**：sysinfo
- **动态库加载**：libloading
- **其他**：notify（文件监听）、windows / winreg（Windows 平台能力）

## 功能模块

| 模块 | 功能 |
|------|------|
| **认证系统** | 离线登录、微软 OAuth 设备码登录、Token 加密存储 |
| **版本管理** | 原版 / Forge / Fabric / NeoForge / OptiFine 下载安装、多实例、版本独立设置 |
| **社区资源** | CurseForge / Modrinth 搜索下载、整合包安装、MC 百科直链 |
| **Java 管理** | 全磁盘扫描、自动检测、手动选择、自动下载 |
| **皮肤管理** | 上传 / 预览 / 3D 模型（skinview3d）、披风设置 |
| **下载管理** | 分片下载、断点续传、限速、进度跟踪 |
| **启动管理** | 参数构建、进程监控、窗口标题改写、崩溃分析 |
| **系统设置** | 启动参数、内存分配、下载配置、开发者工具（日志查看） |

### 认证系统

- 离线模式登录
- 微软 OAuth 2.0 设备码登录
- Token 加密存储与自动刷新

### 版本管理

- 原版 / Forge / Fabric / NeoForge / OptiFine / LiteLoader 加载器支持
- 一键下载安装
- 多版本切换与独立目录管理
- 版本独立设置（Java、内存、启动参数、Mod 管理）

### 社区资源

- CurseForge / Modrinth 双源搜索
- Mod 与整合包安装
- 依赖解析与并发下载
- MC 百科直链获取

### Java 管理

- 全磁盘自动扫描
- 多版本自动检测与匹配
- 手动选择与自定义路径
- 自动下载安装

### 皮肤管理

- 皮肤上传与本地选择
- 3D 模型实时预览（skinview3d）
- 披风管理

### 下载管理

- 分片并行下载
- 断点续传
- 下载限速与速率控制
- 任务分组与进度统计

### 启动管理

- JVM / 游戏参数自动构建
- 进程监控与日志解析
- 游戏窗口标题改写（Windows）
- 崩溃日志分析与提示

## 快速开始

### 环境要求

- **Node.js**：>= 18
- **Rust**：>= 1.75（edition 2021）
- **系统依赖**：[Tauri 2 系统依赖](https://tauri.app/start/prerequisites/)

### 安装

```bash
# 克隆项目
git clone https://github.com/MoTeam-cn/MoLaunch.git
cd MoLaunch

# 安装前端依赖
npm install

# 安装后端依赖（首次构建会自动拉取 crate）
cd src-tauri
cargo build
cd ..
```

### 开发

```bash
# 启动开发服务器（同时启动前端与 Tauri 后端）
npm run tauri dev
```

### 构建

```bash
# 构建发布版本
npm run tauri build
```

## 项目结构

### 前端 (`src/`)

```
src/
├── assets/          # 静态资源（styles/main.css, Skins/, Mods/, blocks/, logo.svg）
├── components/
│   ├── common/      # 通用组件（Button, Input, Select, Modal, Toast, Tooltip, CrashDialog,
│   │                # DeviceCodeModal, DownloadPanel, LoaderCard, SegmentedButtons, Alert,
│   │                # BackToTop, MultiSelectBar, SkinManager, SkinAvatar, SkinModel3D）
│   ├── community/   # 社区资源（SearchBar, ResourceCard, ResourceDetail, Pagination,
│   │                # CommunityConfigCard, resource-detail/）
│   ├── downloads/   # 下载管理（TaskGroupCard）
│   ├── home/        # 首页（LaunchPanel, VersionSelector, LaunchLog, AccountSelector）
│   ├── install/     # 安装（FabricApiInfoCard）
│   ├── layout/      # 布局（TopNavLayout）
│   ├── settings/    # 设置（DevModeToggle, LogViewer, ToggleRow）
│   ├── version/     # 版本（InstalledList, VersionSection）
│   └── version-settings/  # 版本设置（AdvanceFieldsPanel）
├── composables/     # 组合式函数（约 20 个，useAuth, useDownloadPolling,
│                    # useLaunchState, useVersionSettings 等）
├── router/          # 路由
├── stores/          # Pinia 状态（auth, java, sdk, settings, version）
├── types/           # TypeScript 类型定义
├── utils/
│   ├── api/         # API 封装（auth, community, config, developer, image-cache,
│   │                # java, launch, loader, personalization, sdk, skin, system, version）
│   └── *.ts         # 工具函数（async, cape-icon, crashDialog, default-skin, format,
│                    # image-crop, log-display, mod-display, modal, system-display,
│                    # tauri, toast, version）
├── views/
│   ├── Community.vue, Downloads.vue, Home.vue, Login.vue, Settings.vue,
│   ├── VersionSelect.vue, VersionSettings.vue, Versions.vue, LoaderSelect.vue
│   ├── downloads/        # DownloadSidebar, DownloadStatsPanel
│   ├── settings/         # SettingsLaunch, SettingsDownload, SettingsDeveloper,
│   │                     # SettingsAdvanced, SettingsOther, SettingsPersonal
│   ├── version-select/   # FolderSidebar
│   └── version-settings/ # ModTab, OverviewTab, SetupTab, JavaDownloadBar, MemorySection
├── App.vue
└── main.ts
```

### 后端 (`src-tauri/src/`)

```
src-tauri/src/
├── commands/          # Tauri 命令层（前端调用入口）
│   ├── auth/          # 认证（account, microsoft, offline）
│   ├── community/     # 社区资源（search, detail, install/, community_config, secure_config）
│   ├── system/        # 系统（apply_config/, config, developer, download, game, game_dir, proxy）
│   ├── version/       # 版本（install/, mods/, download, folder, launch, list, loaders,
│   │                  # manage, personalization, preload, progress, script_export）
│   ├── image_cache.rs, java.rs, sdk.rs, skin.rs
├── minecraft/         # 核心业务模块（纯 Rust 实现，非 FFI）
│   ├── auth/          # 认证（microsoft/, storage/）
│   ├── community/     # 社区（curseforge/, modrinth/, preload/, cache, common, mcmod,
│   │                  # searcher, secure_storage, tags, version_extract）
│   ├── download/      # 下载（chunk/, assets, downloader, full_download, manager,
│   │                  # rate_limiter, stages）
│   ├── java/          # Java（detect, search, select, download/）
│   ├── java_selector/ # Java 选择器（compat, installer, rules, select, weight）
│   ├── launch/        # 启动（pipeline/, watcher/, arguments, classpath, embedded,
│   │                  # game_args, jvm_args）
│   ├── loaders/       # 加载器（fabric, forge, neoforge, optifine, liteloader,
│   │                  # forge_installer, fabric_api, forge_html, shared, utils）
│   ├── system/        # 系统调用（shell）
│   ├── utils/         # 工具（file_checker, maven）
│   ├── version/       # 版本管理（setup/, json_merge, libraries, scan, state）
│   ├── fools.rs, image_cache.rs, isolation.rs, language.rs,
│   └── launcher_profiles.rs, skin.rs, sources.rs
├── sdk/               # SDK 兼容层（ffi_types, instance, types）
├── state/             # 应用状态（app, auth, config, download, launch）
├── storage/           # 存储（cache, ini, registry）
├── config.rs, error_util.rs, http.rs, lib.rs, logger.rs, main.rs, resources.rs
```

## 开发指南

### 代码规范

- 前端：ESLint + Prettier
- 后端：Clippy + rustfmt

```bash
# 前端代码检查（自动修复）
npm run lint

# 前端类型检查
npm run typecheck

# 后端代码检查
cd src-tauri && cargo clippy -- -D warnings

# 后端代码格式化
cd src-tauri && cargo fmt
```

### 测试

```bash
# 前端测试
npm run test

# 后端测试
cd src-tauri && cargo test
```

### 提交规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/)：

```
<type>(<scope>): <subject>

feat(auth): 添加微软 OAuth 登录
fix(download): 修复断点续传失败
docs: 更新 README
```

## 文档

- [开发蓝图](DEVELOPMENT_BLUEPRINT.md) - 详细的技术设计
- [开发规范](DEVELOPMENT_GUIDELINES.md) - 代码规范和流程
- [AI Agent 规范](AI_AGENT_GUIDELINES.md) - AI Agent 开发约束
- [更新日志](CHANGELOG.md) - 版本变更记录

## 许可证

MIT License

## 致谢

- [Tauri](https://tauri.app/) - 桌面应用框架
- [Vue.js](https://vuejs.org/) - 前端框架
- [Tailwind CSS](https://tailwindcss.com/) - CSS 框架
- [@heroicons/vue](https://github.com/tailwindlabs/heroicons) - 图标组件库
- [skinview3d](https://github.com/bs-community/skinview3d) - Minecraft 3D 皮肤预览

---

*本文档最后更新于 2026-07-20*
