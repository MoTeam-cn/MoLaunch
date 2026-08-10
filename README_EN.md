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

[![GitHub stars](https://img.shields.io/github/stars/MoTeam-cn/MoLaunch?style=flat&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZlcnNpb249IjEiIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiI+PHBhdGggZD0iTTggLjI1YS43NS43NSAwIDAgMSAuNjczLjQxOGwxLjg4MiAzLjgxNSA0LjIxLjYxMmEuNzUuNzUgMCAwIDEgLjQxNiAxLjI3OWwtMy4wNDYgMi45Ny43MTkgNC4xOTJhLjc1MS43NTEgMCAwIDEtMS4wODguNzkxTDggMTIuMzQ3bC0zLjc2NiAxLjk4YS43NS43NSAwIDAgMS0xLjA4OC0uNzlsLjcyLTQuMTk0TC44MTggNi4zNzRhLjc1Ljc1IDAgMCAxIC40MTYtMS4yOGw0LjIxLS42MTFMNy4zMjcuNjY4QS43NS43NSAwIDAgMSA4IC4yNVoiIGZpbGw9IiNlYWM1NGYiLz48L3N2Zz4=&labelColor=165dff&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/MoTeam-cn/MoLaunch?style=flat&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiIgdmlld0JveD0iMCAwIDE2IDE2IiBmaWxsPSIjZmZmZmZmIj48cGF0aCBkPSJNNSA1LjM3MnYuODc4YzAgLjQxNC4zMzYuNzUuNzUuNzVoNC41YS43NS43NSAwIDAgMCAuNzUtLjc1di0uODc4YTIuMjUgMi4yNSAwIDEgMSAxLjUgMHYuODc4YTIuMjUgMi4yNSAwIDAgMS0yLjI1IDIuMjVoLTEuNXYyLjEyOGEyLjI1MSAyLjI1MSAwIDEgMS0xLjUgMFY4LjVoLTEuNUEyLjI1IDIuMjUgMCAwIDEgMy41IDYuMjV2LS44NzhhMi4yNSAyLjI1IDAgMSAxIDEuNSAwWk01IDMuMjVhLjc1Ljc1IDAgMSAwLTEuNSAwIC43NS43NSAwIDAgMCAxLjUgMFptNi43NS43NWEuNzUuNzUgMCAxIDAgMC0xLjUuNzUuNzUgMCAwIDAgMCAxLjVabS0zIDguNzVhLjc1Ljc1IDAgMSAwLTEuNSAwIC43NS43NSAwIDAgMCAxLjUgMFoiLz48L3N2Zz4=&labelColor=165dff&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/forks)
[![GitHub issues](https://img.shields.io/github/issues/MoTeam-cn/MoLaunch?style=flat&logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/MoTeam-cn/MoLaunch?style=flat&logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/commits)
[![GitHub contributors](https://img.shields.io/github/contributors/MoTeam-cn/MoLaunch?style=flat&logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/graphs/contributors)

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

MoLaunch covers the entire Minecraft launching workflow, ready to use out of the box:

- **Version Management** — Vanilla / Forge / Fabric / NeoForge / OptiFine loaders, with multi-version isolation
- **Downloads & Installation** — One-click install for Mods, Resource Packs, and Modpacks (CurseForge / Modrinth), with resumable downloads and mirror acceleration in China
- **Accounts & Skins** — Microsoft account login, skin / cape management, and 3D preview
- **Online Multiplayer** — Room lobby, WebRTC P2P virtual LAN, FRP tunnels without port forwarding
- **Utilities** — Seed map, NBT editor, world backup, Mod dependency checks, and more
- **AI Assistant (Experimental)** — Conversational analysis of game logs and crash causes

Cloud operations such as login are handled through the MoLaunch Cloud, which adds a lightweight PoW check in front of its APIs to deter scripted abuse—invisible during normal use.

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

## Contributors

Thanks to all the developers who have contributed code, documentation, and suggestions to MoLaunch.

[![Contributors](https://contrib.rocks/image?repo=MoTeam-cn/MoLaunch)](https://github.com/MoTeam-cn/MoLaunch/graphs/contributors)

![Alt](https://repobeats.axiom.co/api/embed/8769aee202d5829171ef89b4ffa1e9907fab4d7a.svg "Repobeats analytics image")

## Links

- Repository: https://github.com/MoTeam-cn/MoLaunch
- Issue tracker: https://github.com/MoTeam-cn/MoLaunch/issues
- Changelog: [CHANGELOG.md](./CHANGELOG.md)
- License: [LICENSE](./LICENSE)
- Third-party copyrights: [licenses.txt](./src-tauri/resources/about/licenses.txt)