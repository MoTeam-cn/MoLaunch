# 更新日志

本项目的所有重要更改都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增

#### 微软登录
- 微软 OAuth 2.0 Web 授权码登录流程（Authorization Code Flow，使用 login.live.com 旧版端点 + 公共 Client ID `00000000402b5328`，与 PCL2/HMCL 一致）
- 6 步 Token 交换链：授权码 → OAuth Token → XBL Token → XSTS Token → MC Token → 玩家档案
- Token 持久化存储（DES 加密，支持多账号管理）
- 会话恢复（应用重启后自动恢复登录状态）
- 静默刷新（Token 过期时使用 Refresh Token 自动刷新）
- 已存储微软账号列表展示与快速切换
- Webview 登录窗口组件（内嵌浏览器 + `on_navigation` 拦截回调 + 自动交换 Token）
- Xbox 错误码处理（封禁/未注册/地区限制/年龄不足）
- 游戏所有权验证（entitlements 检查）
- `launch_game` 命令支持动态 `login_type` 参数（Legacy/Microsoft）
- 事件驱动登录状态通信（`ms-login-success` / `ms-login-error` / `ms-login-cancelled` / `ms-login-progress`）

### 变更
- 微软登录采用 Device Code Flow（设备码流程，与 PCL2 一致），使用 v2.0 consumers 端点 + MoLaunch 独立 Azure 应用 Client ID
- 认证存储从单一 `auth.json` 文件改为 Windows 注册表分字段存储（参考 PCL2，路径 `HKCU\Software\MoLaunch`）
- 敏感字段（Token、用户名、UUID 等）单独 SDK DES 加密，非敏感字段（登录类型）明文存储
- Token 刷新使用 login.live.com 旧版端点（与 PCL2 一致）

### 修复
- 修复微软登录申请设备码时返回 `AADSTS700016` 错误（旧版 Minecraft 启动器 Client ID 与 `login.microsoftonline.com` v2.0 端点不兼容，改用 `login.live.com` 旧版端点）
- 修复 `DeviceCodeModal.vue` 中 `openUrl` 导入错误（Tauri 2 shell 插件 API 变更为 `open`）

### 待实现
- Mod 管理功能
- 皮肤管理功能
- 服务器列表功能
- 整合包支持
- 多实例管理

## [0.2.0] - 2026-06-26

### 新增

#### 版本管理
- 版本下载功能 (`mc_download_version`)
- 已安装版本列表 (`mc_list_installed_versions`)
- 版本管理页面标签页 (可用版本/已安装)
- 下载进度显示

#### Java 管理
- Java 运行时检测 (`mc_detect_java`)
- Java 列表获取 (`mc_list_java`)
- Java 管理命令

#### UI 更新
- 版本管理页面重新设计
- 添加已安装版本标签页
- 下载按钮状态显示

## [0.1.0] - 2026-06-26

### 新增

#### 项目结构
- Tauri + Vue 3 + TypeScript 项目初始化
- Rust 后端框架搭建
- Vue 前端框架搭建
- Headless UI + Tailwind CSS 集成
- 完整的项目结构和配置文件

#### SDK 集成
- 多平台 SDK 自动选择机制 (Windows/macOS/Linux)
- Rust FFI 绑定层，支持动态加载 SDK
- Tauri IPC 命令封装
- 设备 ID 获取功能

#### UI 框架
- 侧边栏布局组件
- 顶部导航栏布局组件
- 首页、登录页、版本管理页、设置页
- 布局切换功能 (侧边栏/顶部栏)
- 主题切换功能 (浅色/深色/跟随系统)

#### 状态管理
- 认证状态管理 (Pinia)
- 版本状态管理 (Pinia)
- SDK 状态管理 (Pinia)
- 设置状态管理 (Pinia)

#### 功能
- 离线模式登录
- 版本列表获取
- 设备 ID 显示

#### CI/CD
- GitHub Actions CI workflow
- GitHub Actions Release workflow

#### 核心模块
- McSDK FFI 绑定层 (`src-tauri/src/sdk/`)
- Tauri 命令层 (`src-tauri/src/commands/`)
- 应用状态管理 (`src-tauri/src/state/`)

#### 前端模块
- Vue Router 路由配置
- Pinia 状态管理
- 基础布局组件
- 主题系统框架

#### 功能模块
- SDK 初始化与生命周期管理
- 离线模式登录
- 微软 OAuth 2.0 设备码登录
- 版本列表获取
- Java 运行时检测

#### UI 组件
- 侧边栏导航
- 顶部状态栏
- 版本选择器
- 登录界面

### 变更
- 无

### 修复
- 无

### 移除
- 无

---

## 发布说明

创建新版本的步骤：

1. 更新 `package.json` 和 `src-tauri/Cargo.toml` 中的版本号
2. 更新此 CHANGELOG.md
3. 提交更改：`git commit -m "chore: 准备发布 vX.Y.Z"`
4. 创建标签：`git tag vX.Y.Z`
5. 推送标签：`git push origin vX.Y.Z`
6. GitHub Actions 将自动构建并创建发布

---

*本文档最后更新于 2026-06-26*
