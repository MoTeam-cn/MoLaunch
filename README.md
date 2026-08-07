# MoLaunch

现代化、跨平台的 Minecraft Java 版启动器。

[![License](https://img.shields.io/badge/License-MoLaunch%20Limited%20Distribution%20License-red.svg)](./LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/MoTeam-cn/MoLaunch)
[![Tauri](https://img.shields.io/badge/Tauri-2-orange.svg)](https://v2.tauri.app/)

> MoLaunch 是独立的第三方 Minecraft 启动器项目，不是 Mojang 或 Microsoft 的官方产品，也未获其批准或与其建立关联。

## 项目简介

MoLaunch 使用 Tauri 2、Vue 3、TypeScript 和 Rust 构建，面向希望在一个桌面应用中管理 Minecraft Java 版实例、Java 环境、模组和联机功能的用户。

项目重点关注：

- 跨平台桌面体验
- 原生 Rust 后端与异步任务处理
- 可维护的版本、下载和启动流程
- 清晰的第三方依赖版权与许可证说明

## 主要功能

- **账户与认证**：离线账户、Microsoft 设备码登录、令牌存储与刷新
- **版本管理**：原版、Forge、Fabric、NeoForge、OptiFine 等版本和加载器
- **实例启动**：Java 检测、启动参数、内存配置、进程监控与崩溃分析
- **资源管理**：Mod、整合包、社区资源搜索与安装
- **下载管理**：并发下载、分片下载、断点续传、暂停、取消与校验
- **Java 管理**：本机 Java 扫描、版本匹配和运行时下载
- **皮肤管理**：皮肤、披风及 3D 预览
- **联机功能**：房间、信令、虚拟网络和 FRP 隧道管理
- **开发者能力**：日志查看、实验性 AI 辅助和 SDK 扩展能力

## 技术栈

### 前端

- Vue 3
- TypeScript
- Vite
- Pinia
- Vue Router
- Tailwind CSS
- Tauri JavaScript API

### 后端

- Rust 2021
- Tauri 2
- Tokio
- Reqwest
- Serde / serde_json
- SQLite / rusqlite
- ZIP、NBT、哈希和文件系统工具链

完整依赖、版本、来源及许可证信息请查看：

- [前端运行时依赖](./src-tauri/resources/about/frontend-deps.txt)
- [前端开发依赖](./src-tauri/resources/about/frontend-dev-deps.txt)
- [后端依赖](./src-tauri/resources/about/backend-deps.txt)
- [第三方版权与许可证清单](./src-tauri/resources/about/licenses.txt)

项目引用、安装或随附的第三方项目库、字体、图标、运行时和其他资源，其版权声明、许可证类型、来源链接及许可证链接均已记录在 `src-tauri/resources/about/licenses.txt`。该文件也会随应用资源分发，并在应用的“设置 → 更多 → 鸣谢 → 许可与版权声明”中展示。

## 获取项目

```bash
git clone https://github.com/MoTeam-cn/MoLaunch.git
cd MoLaunch
```

## 环境要求

- Node.js 18 或更高版本
- Rust stable，支持 Rust 2021 edition
- Tauri 2 所需的系统依赖

详细系统依赖请参考 [Tauri 官方文档](https://v2.tauri.app/start/prerequisites/)。

## 安装依赖

安装前端依赖：

```bash
npm ci
```

安装后端依赖并检查项目：

```bash
cd src-tauri
cargo check
cd ..
```

## 开发

启动 Tauri 开发环境：

```bash
npm run tauri dev
```

仅启动前端开发服务器：

```bash
npm run dev
```

## 构建

构建前端：

```bash
npm run build
```

构建桌面应用：

```bash
npm run tauri build
```

## 质量检查与测试

```bash
npm run lint
npm run typecheck
npm run test
```

```bash
cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## 项目结构

```text
MoLaunch/
├── src/                         # Vue、TypeScript 前端
├── src-tauri/src/               # Rust 后端与 Tauri 命令
├── src-tauri/resources/         # 内嵌资源和第三方许可清单
├── src-tauri/LICENSE            # 后端 crate 使用的许可证副本
├── src-tauri/updater/           # 独立更新器 crate
├── src-tauri/updater/LICENSE    # 更新器 crate 使用的许可证副本
├── LICENSE                      # 项目根许可证
├── package.json                 # 前端脚本与依赖
└── src-tauri/Cargo.toml         # Rust crate 配置
```

## 许可证与使用限制

MoLaunch 自有代码和原创资源遵循 [MoLaunch 分发有限许可证](./LICENSE)。该许可证的核心要求包括：

- 禁止将 MoLaunch 或其二次开发版本作为商业产品使用或收费主体
- 二次开发必须公开完整源代码
- 二次开发版本必须明确声明其为第三方版本，与 MoTeam 官方版本无关
- 二次开发版本不得使用容易使公众误认为官方发布的名称
- 不得移除版权、许可证、商标和免责声明

第三方依赖、内嵌资源和引用项目不适用 MoLaunch 自有许可证，必须遵守其各自的原始许可证。具体信息请以 [`src-tauri/resources/about/licenses.txt`](./src-tauri/resources/about/licenses.txt) 及对应上游项目的许可证文件为准。

如需商业授权、品牌授权或其他例外许可，请联系 MoTeam 并取得书面授权。

## 商标与免责声明

MoLaunch、MoTeam 及相关标识属于其权利人。Minecraft 是 Mojang Synergies AB 的商标。MoLaunch 不隶属于 Mojang、Microsoft 或其他相关权利人。

本项目按“现状”提供。在适用法律允许的最大范围内，MoTeam 不对软件的适用性、连续可用性、无错误性、数据安全、第三方服务或因使用软件造成的损失承担责任。

## 贡献与反馈

欢迎通过 GitHub 提交 Issue、讨论和改进建议。提交代码或资源前，请确认其来源、版权和许可证允许将其纳入本项目，并确保不会违反本项目许可证或第三方许可证。

- 仓库：https://github.com/MoTeam-cn/MoLaunch
- 问题反馈：https://github.com/MoTeam-cn/MoLaunch/issues
- 许可证：[LICENSE](./LICENSE)
- 第三方版权清单：[licenses.txt](./src-tauri/resources/about/licenses.txt)
