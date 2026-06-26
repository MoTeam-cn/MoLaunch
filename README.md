# MoLaunch - 现代化 Minecraft 启动器

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个使用 **Tauri + Vue 3 + TypeScript** 构建的现代化 Minecraft 启动器，基于 McSDK 提供完整的游戏管理能力。

## 特性

- **跨平台**: 支持 Windows、macOS、Linux
- **现代化 UI**: 基于 Headless UI + Tailwind CSS，支持主题定制
- **高性能**: Rust 后端 + 异步架构，启动快速、内存占用低
- **功能丰富**: 版本管理、Mod 管理、皮肤管理、服务器管理等

## 功能模块

| 模块 | 功能 |
|------|------|
| **认证系统** | 离线/微软/Mojang/外置登录，Token 加密存储 |
| **版本管理** | 版本列表/下载/安装/切换，多实例支持 |
| **Mod 管理** | CurseForge/Modrinth 搜索，依赖解析，启用/禁用 |
| **皮肤系统** | 上传/预览/管理，披风设置 |
| **Java 管理** | 自动检测，手动选择，版本匹配 |
| **服务器管理** | 服务器列表，快速连接，延迟检测 |
| **整合包** | CurseForge/Modrinth 整合包安装 |
| **设置中心** | 主题/语言/镜像源/内存分配 |

## 技术栈

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

## 快速开始

### 环境要求

- **Node.js**: >= 18
- **Rust**: >= 1.75
- **系统依赖**: [Tauri 系统依赖](https://tauri.app/v1/guides/getting-started/prerequisites)

### 安装

```bash
# 克隆项目
git clone https://github.com/MoTeam-cn/MoLaunch.git
cd MoLaunch

# 安装前端依赖
npm install

# 安装后端依赖
cd src-tauri
cargo build
cd ..
```

### 开发

```bash
# 启动开发服务器
npm run tauri dev
```

### 构建

```bash
# 构建发布版本
npm run tauri build
```

## 项目结构

```
MoLaunch/
├── src-tauri/          # Rust 后端
│   ├── src/
│   │   ├── main.rs     # Tauri 入口
│   │   ├── sdk/        # McSDK FFI 绑定
│   │   ├── commands/   # Tauri 命令
│   │   └── state/      # 应用状态
│   └── Cargo.toml
├── src/                # Vue 前端
│   ├── components/     # 通用组件
│   ├── views/          # 页面视图
│   ├── stores/         # 状态管理
│   ├── composables/    # 组合式函数
│   └── assets/         # 静态资源
├── sdk_data/           # McSDK 动态库
└── package.json
```

## 功能预览

### 认证系统

- 离线模式登录
- 微软 OAuth 2.0 设备码登录
- Token 加密存储与自动刷新

### 版本管理

- 版本列表浏览
- 一键下载安装
- 多版本切换

### Mod 管理

- CurseForge/Modrinth 集成
- Mod 搜索与安装
- 依赖自动解析
- 启用/禁用管理

### 皮肤管理

- 皮肤上传
- 皮肤预览
- 披风设置

## 开发指南

### 代码规范

- 前端: ESLint + Prettier
- 后端: Clippy + rustfmt

```bash
# 前端代码检查
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

遵循 [Conventional Commits](https://www.conventionalcommits.org/):

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
- [FFI 接口](sdk_data/FFI_View.md) - McSDK FFI 接口文档

## 路线图

### Phase 1: 基石 (v0.1.0)
- [x] 项目初始化
- [x] 基础 UI 框架
- [x] McSDK 集成
- [x] 离线模式登录

### Phase 2: 版本管理 (v0.2.0)
- [x] 版本列表
- [x] 版本下载
- [x] Java 检测

### Phase 3: 认证系统 (v0.3.0)
- [ ] 微软 OAuth 登录
- [ ] Token 管理

### Phase 4: Mod 管理 (v0.4.0)
- [ ] Mod 列表
- [ ] Mod 搜索
- [ ] Mod 安装

### Phase 5: 高级功能 (v0.5.0)
- [ ] 皮肤管理
- [ ] 服务器列表
- [ ] 整合包支持
- [ ] 多实例管理

### Phase 6: 优化与发布 (v1.0.0)
- [ ] 性能优化
- [ ] 用户体验优化
- [ ] 正式发布

## 许可证

MIT License

## 致谢

- [McSDK](https://github.com/MoTeam-cn/Mc_SDK) - Minecraft 启动器 SDK
- [Tauri](https://tauri.app/) - 桌面应用框架
- [Vue.js](https://vuejs.org/) - 前端框架
- [Headless UI](https://headlessui.com/) - 无样式组件库
- [Tailwind CSS](https://tailwindcss.com/) - CSS 框架

---

*本文档最后更新于 2026-06-26*
