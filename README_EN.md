<p align="center">
  <img src="images/splash.gif" alt="MoLaunch Splash Animation" width="800" />
</p>

# MoLaunch

[简体中文](./README.md) · [繁體中文](./README_ZH-HANT.md) · [English](./README_EN.md) · [日本語](./README_JA.md)

A modern, cross-platform Minecraft Java Edition launcher.

[![License](https://img.shields.io/badge/License-MoLaunch%20Limited%20Distribution%20License-red.svg)](./LICENSE)
[![Version](https://img.shields.io/badge/version-0.3.5--rc1-blue.svg)](https://github.com/MoTeam-cn/MoLaunch)
[![Tauri](https://img.shields.io/badge/Tauri-2-orange.svg)](https://v2.tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-42b883.svg)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-dea584.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/MoTeam-cn/MoLaunch)

[![GitHub stars](https://img.shields.io/github/stars/MoTeam-cn/MoLaunch?logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/MoTeam-cn/MoLaunch?logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/forks)
[![GitHub issues](https://img.shields.io/github/issues/MoTeam-cn/MoLaunch?logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/MoTeam-cn/MoLaunch?logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/commits)
[![GitHub contributors](https://img.shields.io/github/contributors/MoTeam-cn/MoLaunch?logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/graphs/contributors)

> [!CAUTION]
> MoLaunch is an independent third-party Minecraft launcher project. It is not an official product of Mojang or Microsoft, nor is it endorsed by or affiliated with them.

> [!IMPORTANT]
> This project is developed independently by individuals. To save effort, a significant amount of it was written with AI assistance (Vibe Coding), so the results may not always be perfect. We appreciate your understanding.

## Introduction

MoLaunch is a Minecraft Java Edition launcher that provides a complete set of game management capabilities—downloading, installing, launching, and online multiplayer—built on **Tauri 2 + Vue 3 + Rust**, with a suite of practical in-game tools built in.

This repository contains the open-source code of MoLaunch. You can use the prebuilt artifacts directly, or build from source and develop on top of it yourself.

## Screenshots

<table align="center">
  <tr>
    <td align="center" width="50%">
      <b>Launcher Home</b><br/>
      <sub>Minimal default content area; layout is customizable in Settings (images/001.png)</sub><br/><br/>
      <img src="images/001.png" alt="Launcher Home" width="380" />
    </td>
    <td align="center" width="50%">
      <b>Version Downloads</b><br/>
      <sub>Release / Snapshot / April Fools / Old Beta—four categories (images/002.png)</sub><br/><br/>
      <img src="images/002.png" alt="Version Downloads" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>Community Modpacks</b><br/>
      <sub>Modrinth &amp; CurseForge modpack listings and installation (images/003.png)</sub><br/><br/>
      <img src="images/003.png" alt="Community Modpacks" width="380" />
    </td>
    <td align="center">
      <b>Online Lobby</b><br/>
      <sub>Create / join rooms for online multiplayer (no test rooms available at the moment) (images/004.png)</sub><br/><br/>
      <img src="images/004.png" alt="Online Lobby" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>Seed Map</b><br/>
      <sub>Enter a seed to locate world structures (accuracy still being improved) (images/005.png)</sub><br/><br/>
      <img src="images/005.png" alt="Seed Map" width="380" />
    </td>
    <td align="center">
      <b>AI Chat (Experimental)</b><br/>
      <sub>Conversational assistant that helps analyze logs and crash causes (images/006.png)</sub><br/><br/>
      <img src="images/006.png" alt="AI Chat" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>Skin &amp; Cape Management</b><br/>
      <sub>Change skins and capes (images/007.png)</sub><br/><br/>
      <img src="images/007.png" alt="Skin and Cape Management" width="380" />
    </td>
    <td align="center">
      <b>Settings</b><br/>
      <sub>Appearance, launch/download preferences, developer options (images/008.png)</sub><br/><br/>
      <img src="images/008.png" alt="Settings" width="380" />
    </td>
  </tr>
</table>

## Features

### Version Management

- Support for Vanilla, Forge, Fabric, NeoForge, OptiFine, and LiteLoader loaders
- CurseForge / Modrinth modpack installation with automatic loader and dependency resolution
- Multi-version isolation; each instance is stored independently
- Automatic Java detection: validates runtime requirements per version and pre-downloads the official Mojang Runtime when missing

### Downloads

- Search and install Mods / Resource Packs / Modpacks from CurseForge, Modrinth, and the BMCLAPI mirror
- Chunked parallel downloads with resume support, pause, and verification
- China mirror acceleration (BMCLAPI / MoCDN)

### Accounts

- Microsoft OAuth device-code login, offline accounts
- Credentials stored encrypted locally with automatic refresh

### Skins

- Skin / cape management
- Real-time 3D preview (skinview3d)

### Online Multiplayer

- Online lobby and room management with invite codes / blacklist
- WebRTC P2P virtual LAN (virtual TUN adapter)
- FRP tunnels with multi-provider support—no port forwarding needed

### Tools

- Seed Map: enter a seed to locate strongholds, ocean monuments, and other world structures (cubiomes WASM)
- NBT editing, world backup / restore, Mod dependency checks, server latency testing, Java runtime detection, and more

### AI Assistant (Experimental)

- Conversational assistant that can read game logs, crash reports, and mod lists
- Log & crash analysis: local rule-engine pre-check + in-depth AI analysis
- Supports OpenAI-compatible endpoints (including reasoning models such as DeepSeek R1)

### Plugins

- Plugin SDK with sandbox: custom layouts, system monitoring, launch history, and more, with configurable permissions

### Other

- Splash startup animation (dual-window splashscreen)
- Crash analysis (rule engine + AI suggestions)
- Automatic updates (stable / beta / alpha channels)
- Log desensitization, global CSP, and user agreement gating

### MoLaunch Cloud

Registration, login, and credential-refresh operations are handled through the MoLaunch Cloud (api-server). The cloud places a lightweight Proof-of-Work check in front of these endpoints: you solve a small hash puzzle before the request goes through, preventing scripted mass abuse of the API. Normal usage is barely noticeable—login and similar operations take only tens of milliseconds. The only ones actually blocked by this puzzle are bulk API abusers.

## Technical Architecture

MoLaunch uses a Tauri 2 two-process architecture: the frontend is a Vue 3 single-page application and the backend is a native Rust process, communicating through typed IPC. All heavy lifting (downloading, extraction, launching, networking) is offloaded to Rust, keeping the frontend lightweight.

```mermaid
graph TD
    subgraph Frontend["Frontend · Vue 3 + TypeScript"]
        UI["Pages & Components<br/>Home / Versions / Resources / Online / Tools / Settings / Experimental"]
        STORE["State · Pinia"]
        LOGIC["Logic · composables"]
        API["IPC Wrappers · utils/api"]
    end

    subgraph Bridge["Tauri 2 IPC"]
        CMD["Rust Commands · commands<br/>auth / version / java / skin / frp / online / community / plugins / experimental / tools"]
    end

    subgraph Backend["Backend · Rust 2021"]
        MC["minecraft core<br/>launch · download · loaders · sources (mirrors)"]
        NET["Multiplayer & Network<br/>Room signaling · P2P Virtual LAN · FRP · WebSocket"]
        AI["ai_core<br/>SSE streaming · Agent tools · token estimation · context compression"]
        STOR["storage<br/>Cross-platform config · SQLite · Cache · Registry"]
        WASM["cubiomes WASM<br/>World structure generation"]
        UPD["Standalone updater crate"]
    end

    UI --> STORE --> LOGIC --> API
    API <--> CMD
    CMD --> MC
    CMD --> NET
    CMD --> AI
    CMD --> STOR
    CMD --> WASM
    CMD --> UPD
    MC --> STOR
    NET --> STOR
```

### Frontend

Vue 3 + TypeScript + Vite + Pinia + Vue Router + Tailwind CSS. Features a self-built unified component library (Button / Input / Select / Drawer / Modal / Tooltip / Slider, etc.) with a single-column layout style. Complex business logic is consolidated into composables and stores, keeping components lightweight.

The core components—**Button / Input / Select / Drawer / Slider**, among others—draw inspiration from [Arco Design Vue](https://github.com/arco-design/arco-design-vue): their component source code was extracted and reimplemented as Vue SFC + Tailwind to achieve a consistent visual experience and interaction quality. The copyright notices required by the Arco Design MIT license are included at the top of every reimplemented file. Icons are primarily from [Heroicons](https://github.com/tailwindlabs/heroicons), with SVG path data reused on demand from [Element Plus Icons](https://github.com/element-plus/element-plus-icons) (consolidated in `src/utils/element-icons.ts`; no runtime dependency is introduced). See the "About · Acknowledgments" section in Settings and the Acknowledgments below.

### Backend

Rust 2021 + Tokio async runtime. Core capabilities are split by domain:

- **minecraft**: version manifest parsing, multi-source downloads, loader installation, JVM argument assembly, process monitoring
- **online**: room signaling, WebRTC networking (virtual TUN), FRP tunnel management
- **ai_core**: OpenAI-compatible client with SSE streaming, multi-turn tool calling, automatic context compression
- **storage**: dual-backend configuration storage (Windows registry + cross-platform files), SQLite compiled in
- **cubiomes**: Minecraft world generation C library, compiled to WASM for the structure-locator tool
- **updater**: standalone updater crate supporting channel-based releases and signature verification (essentially a reimplementation of the Tauri updater plugin: since Windows normally requires an installer while this software is portable by nature, it implements its own seamless update toolkit)

### Project Structure

```text
MoLaunch/
├── src/                    # Frontend (Vue 3 + TypeScript)
│   ├── components/         #   Shared component library & business components
│   ├── composables/        #   Composable logic
│   ├── stores/             #   Pinia state
│   ├── utils/api/          #   Tauri IPC wrappers
│   ├── views/              #   Pages (home / versions / online / tools / settings / experimental)
│   └── plugins/            #   Plugin SDK & sandbox
├── src-tauri/              # Backend (Rust + Tauri 2)
│   ├── src/commands/       #   IPC command modules
│   ├── src/minecraft/      #   Launch / download / loaders / mirror sources
│   ├── src/state/          #   App state (config / launch / download)
│   ├── src/storage/        #   Cross-platform storage & SQLite
│   ├── cubiomes/           #   World generation C library (WASM)
│   ├── resources/          #   Bundled resources & third-party licenses
│   └── updater/            #   Standalone updater
├── public/                 # Static assets (splash animation page)
├── docs/                   # Design & audit documents
├── CHANGELOG.md
└── LICENSE
```

## Requirements

- Node.js 18+ and npm
- Rust stable (2021 edition)
- Tauri 2 system dependencies (see the [Tauri documentation](https://v2.tauri.app/start/prerequisites/))

## Development & Build

```bash
npm ci
npm run tauri dev      # Development & debugging
npm run tauri build    # Package desktop app
```

Quality checks:

```bash
npm run lint && npm run typecheck && npm run test
cd src-tauri && cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test --all-features
```

## License

MoLaunch's own code and original assets are governed by the [MoLaunch Limited Distribution License](./LICENSE), whose core requirements are:

- You may not use MoLaunch or any derived version as a commercial product or charge for it
- Any derived development must publish the complete source code and clearly declare itself a third-party version (without using a name that could be mistaken for the official one)
- You may not remove the copyright, license, trademark, or disclaimers

Third-party dependencies, bundled assets, and referenced projects remain subject to their respective original licenses. See [licenses.txt](./src-tauri/resources/about/licenses.txt). If you need an exception such as a commercial license, please contact MoTeam and obtain written authorization.

Minecraft is a trademark of Mojang Synergies AB. MoLaunch is not affiliated with Mojang, Microsoft, or other rights holders. This project is provided "as is".

## Acknowledgments

We would like to thank the following open-source projects and communities for their outstanding contributions. MoLaunch stands on the shoulders of giants.

### Special Thanks

- **[Arco Design Vue](https://github.com/arco-design/arco-design-vue)** — The frontend core components (Button / Input / Select / Drawer / Slider, etc.) are inspired by and reimplemented from its source code as Vue SFC + Tailwind; the copyright notices are included at the top of the corresponding source files
- **[Element Plus Icons](https://github.com/element-plus/element-plus-icons)** — SVG path data extracted on demand (`src/utils/element-icons.ts`); only icons are reused, with no runtime dependency
- **[Plain Craft Launcher 2 (PCL2)](https://github.com/Meloong-Git/PCL)** — A widely used Minecraft launcher; MoLaunch was developed from scratch in its early days, with the game launching logic referencing PCL2's implementation

### Core Dependencies

- **[Vue 3](https://github.com/vuejs/core)** / [Vue Router](https://github.com/vuejs/router) / [Pinia](https://github.com/vuejs/pinia) — frontend framework & state management
- **[Tauri 2](https://github.com/tauri-apps/tauri)** — desktop application framework (Rust + WebView)
- **[Tailwind CSS](https://github.com/tailwindlabs/tailwindcss)** — atomic CSS solution
- **[Heroicons](https://github.com/tailwindlabs/heroicons)** — primary icon library
- **[skinview3d](https://github.com/bs-community/skinview3d)** — real-time 3D skin preview
- **[Cubiomes](https://github.com/Cubitect/cubiomes)** — world structure generation algorithms (compiled to WASM)
- **[OpenLayers](https://github.com/openlayers/openlayers)** — interactive map for structure location
- **[Tokio](https://github.com/tokio-rs/tokio)** / **[Reqwest](https://github.com/seanmonstar/reqwest)** — Rust async runtime & HTTP client

See [licenses.txt](./src-tauri/resources/about/licenses.txt) for the complete list of third-party licenses and copyrights.

> [!NOTE]
> MoLaunch is an independent third-party creation with no affiliation to PCL2. PCL2 is distributed under the PCL Limited Distribution License; see its [license document](https://shimo.im/docs/rGrd8pY8xWkt6ryW) for details.

## Links

- Repository: https://github.com/MoTeam-cn/MoLaunch
- Issue tracker: https://github.com/MoTeam-cn/MoLaunch/issues
- Changelog: [CHANGELOG.md](./CHANGELOG.md)
- License: [LICENSE](./LICENSE)
- Third-party copyrights: [licenses.txt](./src-tauri/resources/about/licenses.txt)