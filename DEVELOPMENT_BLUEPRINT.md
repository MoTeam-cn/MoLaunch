# MoLaunch 开发蓝图

> **适用范围**：所有参与 MoLaunch 开发的人员
> **最后更新**：2026-08-08

---

## 目录

- 第一章 [项目概况](#一项目概况)
- 第二章 [仓库结构](#二仓库结构)
- 第三章 [前端架构](#三前端架构)
- 第四章 [后端架构](#四后端架构)
- 第五章 [UI 规范](#五ui-规范)
- 第六章 [状态管理](#六状态管理)
- 第七章 [数据流与通信](#七数据流与通信)
- 第八章 [MoLaunch 云端（api-server）](#八molaunch-云端api-server)
- 附件 Module 索引

---

## 一、项目概况

MoLaunch 是一款现代化、跨平台的 Minecraft Java 版启动器，提供下载、安装、启动、联机等完整游戏管理能力，并内置一整套实用的游戏工具。

### 1.1 技术栈

| 层 | 技术 |
|----|------|
| 前端 | Vue 3 + TypeScript + Vite + Pinia + Vue Router + Tailwind CSS + `@heroicons/vue` |
| 后端 | Tauri 2 + Rust 2021 + tokio + reqwest + serde + sha1/sha2/md5 + sysinfo |
| 辅助 | cubiomes（WASM，种子地图）、skinview3d（皮肤预览）、openlayers（结构地图）、vue-virtual-scroller |
| 云端 | 独立 axum 服务 `api-server`（见第八章），登录 / 联机 / FRP 调度 / 更新分发 |

### 1.2 版本管理

版本号在 `package.json` / `src-tauri/Cargo.toml` / `src-tauri/tauri.conf.json` 三处保持一致。发布走 tag：启动器 `v0.3.x-rcN`，云端 `v0.1.x-rcN`。更新日志弹窗的「作者的话」通过 `note:` 前缀 commit 下发（见 `DEVELOPMENT_GUIDELINES.md` 2.5）。

---

## 二、仓库结构

```text
MoLaunch/                          # 启动器主仓库
├── src/                           # 前端（Vue 3 + TS）
├── src-tauri/                     # 后端（Rust + Tauri 2）
├── docs/                          # 设计与审计文档
│   ├── fix-bug/                   #   违规排查规范（fix-message.md）等
│   ├── *.md                       #   各功能设计与审计（frp、ai、pow 等）
├── api-server/                    # MoLaunch 云端（独立 git 仓库）
├── AI_AGENT_GUIDELINES.md         # AI 协作行为约束
├── DEVELOPMENT_GUIDELINES.md      # 开发规范
├── DEVELOPMENT_BLUEPRINT.md       # 本文件：架构蓝图
├── CHANGELOG.md
└── LICENSE
```

前端与后端细节见第三、四章。`docs/` 目录已添加 git 排除规则，仅本地使用。

---

## 三、前端架构

### 3.1 分层

```text
views/（路由页面）
  └─ components/（业务组件 + common 通用组件）
      └─ composables/（组合式逻辑，useXxx）
          └─ stores/（Pinia，跨页面共享）
              └─ utils/（与 IPC / 纯函数打交道，含 utils/api 三层封装）
```

单向依赖：轻量 function 在最底层。业务组件不允许直接依赖 IPC；IPC 全部经 `utils/api/` 统一封装并由类型声明约束。

### 3.2 目录总览

```text
src/
├── components/
│   ├── common/          # 通用组件库：Button / Input / Select / Drawer / Modal / Toast / Tooltip / Alert / Slider / BackToTop ...
│   ├── about/           # 关于页相关：MoLaunchIntro / ReleaseTimeline / UpdateDialog / UpdateLogDialog / DisclaimerDialog ...
│   ├── settings/        # 设置页细分 Section
│   ├── download/        # 下载列表、进度条、侧栏
│   ├── skin/            # 皮肤 / 披风管理
│   ├── frp/             # FRP 隧道相关 Table / StatCard / StatusChip
│   ├── online/          # 联机房卡、邀请码、WebRTC
│   ├── experimental/    # AI Chat 对话框
│   └── community/       # 社区整合包
├── composables/         # useXxx：起点经 useTauriEvent 监听，逻辑可复用（70+ 个）
├── stores/              # Pinia：settings / auth / version / java / online / frp / sdk / plugins ...
├── plugins/             # 插件 SDK 沙箱：sandbox / custom-layout / launch-history / system-monitor / quick-stats / version-stats
├── utils/
│   ├── api/             # IPC 三层封装：XxxManager.ts 资源管理 + 单函数 + 事件监听（frp-manager / online-manager / personalization / tools ...）
│   ├── online/          # 联机协议：crypto / protocol / detect / nat-type / device-id / webrtc-helpers
│   ├── seedmap/         # 种子地图：wasm-bindings / structure-search / tile-render / workerPool / chunk-finder
│   ├── markdown.ts      # renderMarkdown（marked）+ DOMPurify；handleMarkdownLinkClick
│   ├── modal.ts / toast.ts   # 全局弹窗/提示（showXxx / toastXxx）
│   ├── updateLog.ts     # 更新日志虚拟模块读取（getChangelogContent / getChangelogNotes / getChangelogVersion）
│   ├── version.ts       # 版本比较 compareVersion / getVersionInfo
│   ├── format.ts / color.ts / clipboard.ts / fileDialog.ts ...
│   ├── tokens.ts        # token 脱敏
│   └── updater/         # 自动更新：check / install / state
├── views/               # Home / Versions / VersionSelect / VersionSettings / Community / Online / Tools / Settings /
│                        # Experimental / ExternalDownload / QuickTools / downloads
├── types/               # 前后端共享类型定义
└── main.ts / router.ts  # 入口与路由
```

### 3.3 数据与功能三大域

前端业务以三大「管理器」域为主，各域有后续写 Manager 封装：

1. **版本与启动域**（`version-*`、`java`、`loader` 相关 Manager）
2. **资源域**（`community` Modrinth/CurseForge/MCMOD、`skin`、`personalization`、`image-cache`）
3. **联机域**（`frp-manager`、`online-manager`、`tools` 等独立管理）

### 3.4 前 / 后端协议

前端 `utils/api` 封装所有 Tauri 命令，返回 `Promise<T>`；Tauri 命令失败统一返回 `Result` 与错误字符串，页面用 `toast` + `modal` 呈现错误。

---

## 四、后端架构

Rust 侧按域拆分模块，命令入口层在 `commands/`，核心业务在 `minecraft/`、`online/` 与各个 support 模块。

### 4.1 模块总览

| 模块 | 职责 | 关键内容 |
|------|------|----------|
| `commands/` | 所有 Tauri IPC 命令入口 | auth / account → microsoft/offline/authlib；java；version（install/launch/export/mods/progress/settings）；mod（loaders...）；skin；community；frp；online；experimental（agent 等）；ai；plugins（plugins SDK）；sdk；tools；system（config/apply_config/setting） |
| `minecraft/` | 游戏核心 | auth（微软 OAuth、离线、authlib）、community（curseforge/mcmod/modrinth/preload/searcher）、download（chunk 分片 + downloader）、online（signaling / pow / ecies / crypto / tun / protocol）、launch（watcher / analyzer）、version（scan / libraries / setup）、java_selector、system/shell、sources、utils |
| `online/` | 联机信号与 P2P | 房间信令（websocket）、WebRTC Mesh、虚拟 TUN、FRP 隧道管理、NAT 检测 |
| `ai_core/` | AI 对话引擎 | OpenAI 兼容客户端、SSE 流式、多轮 tool calling、token 估算、上下文压缩、会话存储 |
| `storage/` | 跨平台存储 | 双后端（Windows 注册表 + 跨平台文件）、SQLite、缓存 |
| `state/` | 应用状态 | AppState（config / download / launch 等，`Arc<Mutex>`）+ 状态 helper |
| `config/` | 配置管理 | 基于 `apply_config` / `get_config` 读写（禁止 `set_*` / `get_*` 单字段命令）；支持热加载 |
| `logger/` | 日志 | 自定义宏（`log_info!` 等）、脱敏 sanitize、日志查看器、配色 |
| `certs/` | 证书 | 管理（生成 / 导入 / 信任）、PEM 存储、Windows 证书安装 |
| `http/` | HTTP 基础设施 | client（reqwest 自定义）、TLS 配置 |
| `deeplink/` | 深链接 | 协议路由与安全校验（`(tauri)://` / 自定义 scheme） |
| `sdk/` | 插件 SDK | 沙箱执行、宿主应用生命周期、权限判定 |
| `migrations/` | 数据迁移 | 版本间数据结构变更 |
| `resources/` | 内嵌资源 | 图片、licenses.txt、read_resource() 访问 |

### 4.2 命令执行链路

```
前端 invoke → commands/xxx.rs #[tauri::command] → 业务模块（minecraft/online/
ai_core/state...）→ repositories/storage/http → 结果返回
```

所有命令：`Result<T, String>`、`State<'_, AppState>` 注入、`lock()` 后及时 `drop()`、`log_info!` 等记录日志。模块内部不直接暴露 IPC；需要暴露的通过 `commands` 层统一注册（见 `DEVELOPMENT_GUIDELINES.md`）。

---

## 五、UI 规范

- 单列布局（参考 PCL2）；图标用 `@heroicons/vue`，不使用 Emoji
- 主色 `brand-2 #0b5bcb`，悬停 `brand-3 #1370f3`，页面背景 `page #f0f5ff`，圆角 2px/4px
- 空状态：icon + text 垂直水平居中
- 图标通用名：`@heroicons/vue`；品牌图标走 `element-icons.ts` / `md-icons.ts` 等
- 日志颜色：ERROR=red-400、WARN=yellow-400、INFO=green-400、DEBUG=cyan-400、TRACE=slate-500

---

## 六、状态管理

### 6.1 Rust 侧（AppState）

```rust
pub struct AppState {
  pub config: Arc<Mutex<AppConfig>>,
  pub download: Arc<Mutex<DownloadState>>,
  // ... 其他运行时状态
}
```

- 锁内操作尽量短，clone 后立即 drop
- 状态 helper 集中在 `state/`，避免重复 lock/clone/drop 套件

### 6.2 Vue 侧（Pinia）

- `settings`、`auth`、`version`、`online`、`frp`、`download` 等 store 按需拆分，组合式使用
- 复杂业务逻辑放 composables，store 负责跨页状态；UI 状态放组件本地

---

## 七、数据流与通信

### 7.1 IPC 双向

```mermaid
flowchart LR
    UI[Vue 组件] -->|invoke| CMD[Tauri Commands]
    CMD -->|返回 Promise| UI
    Rust <--|事件 emit| UI
```

前端统一 `utils/api` 封装；后端事件用 `app.emit`，前端公式 composable `useTauriEvent`。

### 7.2 与服务端（api-server）通信

- 注册 / 登录 / 刷新走 MoSign 协议 + 加密封装（登录体在客户端处理，签名定义见 `api-server/docs/pow.md`）
- 业务响应走 ECIES 信封：客户端持私钥解密；`tokens.ts` 统一 token 管理
- PoW challenge：`PowChallengeResponse` DTO（challenge_id/salt/difficulty/ttl/path/header_name）；客户端不硬编码请求头名，用 `header_name`

### 7.3 安全策略

| 点 | 方式 |
|----|------|
| 凭据存储 | 本地加密存储（clave，token 不落明文） |
| 传输 | TLS（默认）+ HTTPS + ECIES 信封 |
| 防滥用 | 云端 PoW（难度分级 注册>登录>刷新；自动清理、响应 `x-molaunch-pow` 头） |
| Web 资源 | CSP 全局策略、日志脱敏（token 替换） |
| 更新 | 分通道（stable/beta/alpha）+ 签名校验（自有 updater） |

---

## 八、MoLaunch 云端（api-server）

独立仓库，remote 指向 `Molaunch-ApiServer`。axum 服务，目录结构：

| 路径 | 职责 |
|--------|----------|
| `controllers/v1` / `controllers/v3` | 路由与处理器（auth / csrf / frp_server / signaling / updates / oauth / jwks / health ...） |
| `middlewares/` | JWT / CSRF / rate_limit / pow（PoW challenge）/ mosign_v2_guard / admin_guard / request_logger / frp_hmac ... 洋葱模型 |
| `services/` | 业务层（auth / envelope 加密信封 / oauth / signaling / frp_server / updates） |
| `repositories/` | 数据访问（postgres / sqlite 双引擎，migrations 目录存放模式变更） |
| `models/` | 请求/响应 DTO（如 `PowChallengeResponse`、`CsrfTokenResponse`） |
| `errors/` | 统一错误码（`code: 1007` = PoW 未通过等） |

**PoW 机制（核心）**：纯内存不落库，challenge 由 `DashMap` 保存；`salt` 每次 CSPRNG 随机、`difficulty` 按接口分级（注册 20 最高 / 登录 16 / 刷新 14），`ttl` 过期、一次性；生成/校验均不占用数据库。响应体通过 `PowerChallengeResponse` DTO 下发，路径强绑定防跨接口复用。兼容旧客户端（无 `header_name` 时回退默认头名）。设计方案见 `api-server/docs/pow.md`。

**发布流程**：修改代码必须同时更新 `api-server/CHANGELOG.md`；版本 tag 独立（如 `v0.1.x-rc`），提交信息带 `!c`；云服务动作（构建）触发时通过 CI 自动打包部署。客户端升级时与启动器相互配合。

---

## 附录：AI 协作速查

- 动手前阅读 `DEVELOPMENT_BLUEPRINT.md` 建立整体认知，`DEVELOPMENT_GUIDELINES.md` 提供风格与过关要求，`AI_AGENT_GUIDELINES.md` 提供行为约束
- 所有修改必须同步 `CHANGELOG.md`；提交带 `!c`；每完成一批同性质修改拆一个 commit
- 复用既有组件、函数、Hook、IPC 命令；不重复造轮子
- 最小验证：每步修改跑对应 typecheck / lint / clippy / test

*本文档最后更新于 2026-08-08*