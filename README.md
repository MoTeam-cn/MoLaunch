<p align="center">
  <img src="images/splash.gif" alt="MoLaunch 开屏动画" width="800" />
</p>

# MoLaunch

[简体中文](./README.md) · [繁體中文](./README_ZH-HANT.md) · [English](./README_EN.md) · [日本語](./README_JA.md)

现代化、跨平台的 Minecraft Java 版启动器。

[![License](https://img.shields.io/badge/License-MoLaunch%20Limited%20Distribution%20License-red.svg)](./LICENSE)
[![Version](https://img.shields.io/badge/version-0.3.5--rc8-blue.svg)](https://github.com/MoTeam-cn/MoLaunch)
[![Tauri](https://img.shields.io/badge/Tauri-2-orange.svg)](https://v2.tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-42b883.svg)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-dea584.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/MoTeam-cn/MoLaunch)

[![GitHub stars](https://img.shields.io/github/stars/MoTeam-cn/MoLaunch?style=flat&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZlcnNpb249IjEiIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiI+PHBhdGggZD0iTTggLjI1YS43NS43NSAwIDAgMSAuNjczLjQxOGwxLjg4MiAzLjgxNSA0LjIxLjYxMmEuNzUuNzUgMCAwIDEgLjQxNiAxLjI3OWwtMy4wNDYgMi45Ny43MTkgNC4xOTJhLjc1MS43NTEgMCAwIDEtMS4wODguNzkxTDggMTIuMzQ3bC0zLjc2NiAxLjk4YS43NS43NSAwIDAgMS0xLjA4OC0uNzlsLjcyLTQuMTk0TC44MTggNi4zNzRhLjc1Ljc1IDAgMCAxIC40MTYtMS4yOGw0LjIxLS42MTFMNy4zMjcuNjY4QS43NS43NSAwIDAgMSA4IC4yNVoiIGZpbGw9IiNlYWM1NGYiLz48L3N2Zz4=&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/MoTeam-cn/MoLaunch?style=flat&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiIgdmlld0JveD0iMCAwIDE2IDE2IiBmaWxsPSIjZmZmZmZmIj48cGF0aCBkPSJNNSA1LjM3MnYuODc4YzAgLjQxNC4zMzYuNzUuNzUuNzVoNC41YS43NS43NSAwIDAgMCAuNzUtLjc1di0uODc4YTIuMjUgMi4yNSAwIDEgMSAxLjUgMHYuODc4YTIuMjUgMi4yNSAwIDAgMS0yLjI1IDIuMjVoLTEuNXYyLjEyOGEyLjI1MSAyLjI1MSAwIDEgMS0xLjUgMFY4LjVoLTEuNUEyLjI1IDIuMjUgMCAwIDEgMy41IDYuMjV2LS44NzhhMi4yNSAyLjI1IDAgMSAxIDEuNSAwWk01IDMuMjVhLjc1Ljc1IDAgMSAwLTEuNSAwIC43NS43NSAwIDAgMCAxLjUgMFptNi43NS43NWEuNzUuNzUgMCAxIDAgMC0xLjUuNzUuNzUgMCAwIDAgMCAxLjVabS0zIDguNzVhLjc1Ljc1IDAgMSAwLTEuNSAwIC43NS43NSAwIDAgMCAxLjUgMFoiLz48L3N2Zz4=&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/forks)
[![GitHub issues](https://img.shields.io/github/issues/MoTeam-cn/MoLaunch?style=flat&logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/MoTeam-cn/MoLaunch?style=flat&logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/commits)
[![GitHub contributors](https://img.shields.io/github/contributors/MoTeam-cn/MoLaunch?style=flat&logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/graphs/contributors)

> [!CAUTION]
> MoLaunch 是独立的第三方 Minecraft 启动器项目，不是 Mojang 或 Microsoft 的官方产品，也未获其批准或与其建立关联。

> [!IMPORTANT]
> 本项目为个人独立开发，为了图省事，不少地方是用 AI 辅助（Vibe Coding）写的，效果可能不尽如人意，还请大家多包涵。

## 简介

MoLaunch 是一款 Minecraft Java 版启动器，提供下载、安装、启动、联机等完整的游戏管理能力，基于 **Tauri 2 + Vue 3 + Rust** 构建，并内置一整套实用的游戏工具。

本仓库为 MoLaunch 的开源代码。你可以直接使用构建产物，也可以基于本仓库自行编译、二次开发。

## 界面预览

<table align="center">
  <tr>
    <td align="center" width="50%">
      <b>启动器主页</b><br/>
      <sub>默认简约内容区，布局可在设置中调整（images/001.png）</sub><br/><br/>
      <img src="images/001.png" alt="启动器主页" width="380" />
    </td>
    <td align="center" width="50%">
      <b>版本下载页</b><br/>
      <sub>正式版 / 快照版 / 愚人节版 / 远古版四类（images/002.png）</sub><br/><br/>
      <img src="images/002.png" alt="版本下载页" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>社区整合包</b><br/>
      <sub>Modrinth 与 CurseForge 整合包列表与安装（images/003.png）</sub><br/><br/>
      <img src="images/003.png" alt="社区整合包" width="380" />
    </td>
    <td align="center">
      <b>联机大厅</b><br/>
      <sub>创建 / 加入房间联机（当前暂无测试房间）（images/004.png）</sub><br/><br/>
      <img src="images/004.png" alt="联机大厅" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>种子地图</b><br/>
      <sub>输入种子定位世界结构（准确率仍在优化中）（images/005.png）</sub><br/><br/>
      <img src="images/005.png" alt="种子地图" width="380" />
    </td>
    <td align="center">
      <b>AI 聊天（实验性）</b><br/>
      <sub>对话式智能助手，帮你分析日志与崩溃原因（images/006.png）</sub><br/><br/>
      <img src="images/006.png" alt="AI 聊天" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>皮肤与披风管理</b><br/>
      <sub>皮肤与披风更换（images/007.png）</sub><br/><br/>
      <img src="images/007.png" alt="皮肤与披风管理" width="380" />
    </td>
    <td align="center">
      <b>设置页面</b><br/>
      <sub>外观、启动 / 下载偏好、开发者选项（images/008.png）</sub><br/><br/>
      <img src="images/008.png" alt="设置页面" width="380" />
    </td>
  </tr>
</table>

## 功能特性

MoLaunch 覆盖 Minecraft 启动全流程，开箱即用：

- **版本管理** — 原版 / Forge / Fabric / NeoForge / OptiFine 等加载器，多版本隔离
- **下载安装** — Mod、资源包、整合包一键安装（CurseForge / Modrinth），断点续传 + 国内镜像加速
- **账户与皮肤** — 微软账户登录、皮肤 / 披风管理与 3D 预览
- **联机** — 房间大厅、WebRTC P2P 虚拟局域网、FRP 隧道免端口映射
- **实用工具** — 种子地图、NBT 编辑、存档备份、Mod 依赖检查等
- **AI 助手（实验性）** — 对话式分析游戏日志与崩溃原因

登录等云端操作对接 MoLaunch 云端，接口前带轻量 PoW 验证防脚本刷接口，正常使用无感。

## 许可证

MoLaunch 自有代码与原创资源遵循 [MoLaunch 分发有限许可证](./LICENSE)，核心要求：

- 禁止将 MoLaunch 或其二次开发版本作为商业产品使用或收费
- 二次开发必须公开完整源代码，并明确声明为第三方版本（不得使用易误认为官方的名称）
- 不得移除版权、许可证、商标与免责声明

第三方依赖、内嵌资源与引用项目须遵守其各自原始许可证，详见 [licenses.txt](./src-tauri/resources/about/licenses.txt)。如需商业授权等例外许可，请联系 MoTeam 并取得书面授权。

Minecraft 是 Mojang Synergies AB 的商标。MoLaunch 不隶属于 Mojang、Microsoft 或其他相关权利人，本项目按"现状"提供。

## 鸣谢

感谢以下开源项目与社区的杰出贡献，MoLaunch 站在巨人的肩膀上。

### 特别感谢

- **[Arco Design Vue](https://github.com/arco-design/arco-design-vue)** — 前端核心组件（Button / Input / Select / Drawer / Slider 等）参考借鉴其实现，提取源码复刻改写为 Vue SFC + Tailwind 形式，版权声明注释见各源文件顶部
- **[Element Plus Icons](https://github.com/element-plus/element-plus-icons)** — 按需提取 SVG path 数据复用（`src/utils/element-icons.ts`），仅引入图标未引入运行时依赖
- **[Plain Craft Launcher 2 (PCL2)](https://github.com/Meloong-Git/PCL)** — 一款被广泛使用的 Minecraft 启动器；MoLaunch 前期从零开始开发，启动 Minecraft 的相关逻辑参考了 PCL2 的实现

### 核心依赖

- **[Vue 3](https://github.com/vuejs/core)** / [Vue Router](https://github.com/vuejs/router) / [Pinia](https://github.com/vuejs/pinia) — 前端框架与状态管理
- **[Tauri 2](https://github.com/tauri-apps/tauri)** — 桌面应用框架（Rust + WebView）
- **[Tailwind CSS](https://github.com/tailwindlabs/tailwindcss)** — 原子化样式方案
- **[Heroicons](https://github.com/tailwindlabs/heroicons)** — 主图标库
- **[skinview3d](https://github.com/bs-community/skinview3d)** — 皮肤 3D 实时预览
- **[Cubiomes](https://github.com/Cubitect/cubiomes)** — 世界结构生成算法（编译为 WASM）
- **[OpenLayers](https://github.com/openlayers/openlayers)** — 结构寻址交互式地图
- **[Tokio](https://github.com/tokio-rs/tokio)** / **[Reqwest](https://github.com/seanmonstar/reqwest)** — Rust 异步运行时与 HTTP 客户端

完整第三方许可与版权清单见 [licenses.txt](./src-tauri/resources/about/licenses.txt)。

> [!NOTE]
> MoLaunch 为独立第三方创作，与 PCL2 无隶属或关联关系；PCL2 采用《PCL 分发有限许可》，详情参阅其[许可文档](https://shimo.im/docs/rGrd8pY8xWkt6ryW)。

## 贡献者

感谢所有为 MoLaunch 贡献过代码、文档与建议的开发者。

[![Contributors](https://contrib.rocks/image?repo=MoTeam-cn/MoLaunch)](https://github.com/MoTeam-cn/MoLaunch/graphs/contributors)

![Alt](https://repobeats.axiom.co/api/embed/8769aee202d5829171ef89b4ffa1e9907fab4d7a.svg "Repobeats analytics image")

## 相关链接

- 仓库：https://github.com/MoTeam-cn/MoLaunch
- 问题反馈：https://github.com/MoTeam-cn/MoLaunch/issues
- 更新日志：[CHANGELOG.md](./CHANGELOG.md)
- 许可证：[LICENSE](./LICENSE)
- 第三方版权清单：[licenses.txt](./src-tauri/resources/about/licenses.txt)
