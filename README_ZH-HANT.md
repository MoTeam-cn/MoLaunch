<p align="center">
  <img src="images/splash.gif" alt="MoLaunch 開場動畫" width="800" />
</p>

# MoLaunch

[簡體中文](./README.md) · [繁體中文](./README_ZH-HANT.md) · [English](./README_EN.md) · [日本語](./README_JA.md)

現代化、跨平台的 Minecraft Java 版啟動器。

[![License](https://img.shields.io/badge/License-MoLaunch%20Limited%20Distribution%20License-red.svg)](./LICENSE)
[![Version](https://img.shields.io/badge/version-0.3.5--rc4-blue.svg)](https://github.com/MoTeam-cn/MoLaunch)
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
> MoLaunch 是獨立的第三方 Minecraft 啟動器專案，不是 Mojang 或 Microsoft 的官方產品，也未獲得其核准或與其建立關聯。

> [!IMPORTANT]
> 本專案為個人獨立開發，為了圖省事，不少地方是用 AI 輔助（Vibe Coding）寫的，效果可能不盡如人意，還請大家多多包涵。

## 簡介

MoLaunch 是一款 Minecraft Java 版啟動器，提供下載、安裝、啟動、聯機等完整的遊戲管理能力，基於 **Tauri 2 + Vue 3 + Rust** 構建，並內建一整套實用的遊戲工具。

本倉庫為 MoLaunch 的開源程式碼。你可以直接使用建置產物，也可以基於本倉庫自行編譯、二次開發。

## 介面預覽

<table align="center">
  <tr>
    <td align="center" width="50%">
      <b>啟動器首頁</b><br/>
      <sub>預設簡約內容區，版面可在設定中調整（images/001.png）</sub><br/><br/>
      <img src="images/001.png" alt="啟動器首頁" width="380" />
    </td>
    <td align="center" width="50%">
      <b>版本下載頁</b><br/>
      <sub>正式版 / 快照版 / 愚人節版 / 遠古版四類（images/002.png）</sub><br/><br/>
      <img src="images/002.png" alt="版本下載頁" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>社群整合包</b><br/>
      <sub>Modrinth 與 CurseForge 整合包列表與安裝（images/003.png）</sub><br/><br/>
      <img src="images/003.png" alt="社群整合包" width="380" />
    </td>
    <td align="center">
      <b>聯機大廳</b><br/>
      <sub>建立 / 加入房間聯機（目前暫無測試房間）（images/004.png）</sub><br/><br/>
      <img src="images/004.png" alt="聯機大廳" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>種子地圖</b><br/>
      <sub>輸入種子定位世界結構（準確率仍在最佳化中）（images/005.png）</sub><br/><br/>
      <img src="images/005.png" alt="種子地圖" width="380" />
    </td>
    <td align="center">
      <b>AI 聊天（實驗性）</b><br/>
      <sub>對話式智慧助手，幫你分析日誌與崩潰原因（images/006.png）</sub><br/><br/>
      <img src="images/006.png" alt="AI 聊天" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>皮膚與披風管理</b><br/>
      <sub>皮膚與披風更換（images/007.png）</sub><br/><br/>
      <img src="images/007.png" alt="皮膚與披風管理" width="380" />
    </td>
    <td align="center">
      <b>設定頁面</b><br/>
      <sub>外觀、啟動 / 下載偏好、開發者選項（images/008.png）</sub><br/><br/>
      <img src="images/008.png" alt="設定頁面" width="380" />
    </td>
  </tr>
</table>

## 功能特色

MoLaunch 涵蓋 Minecraft 啟動全流程，開箱即用：

- **版本管理** — 原版 / Forge / Fabric / NeoForge / OptiFine 等載入器，多版本隔離
- **下載安裝** — Mod、資源包、整合包一鍵安裝（CurseForge / Modrinth），斷點續傳 + 中國境內鏡像加速
- **帳戶與皮膚** — Microsoft 帳戶登入、皮膚 / 披風管理與 3D 預覽
- **聯機** — 房間大廳、WebRTC P2P 虛擬區域網路、FRP 隧道免連接埠對應
- **實用工具** — 種子地圖、NBT 編輯、存檔備份、Mod 依賴檢查等
- **AI 助手（實驗性）** — 對話式分析遊戲日誌與崩潰原因

登入等雲端操作對接 MoLaunch 雲端，介面前帶輕量 PoW 驗證防腳本刷介面，正常使用無感。

## 授權許可

MoLaunch 自有程式碼與原創資源遵循 [MoLaunch 分發有限許可](./LICENSE)，核心要求：

- 禁止將 MoLaunch 或其二次開發版本作為商業產品使用或收費，
- 二次開發必須公開完整原始碼，並明確聲明為第三方版本（不得使用易誤認為官方的名稱）
- 不得移除版權、授權、商標與免責聲明

第三方依賴、內嵌資源與引用專案須遵守，其各原始授權，詳見 [licenses.txt](./src-tauri/resources/about/licenses.txt)。如需商業授權等例外，請聯繫 MoTeam 並取得書面授權。

Minecraft 為 Mojang Synergies AB 的商標。MoLaunch 不隸屬於 Mojang、Microsoft 或 其他相關權利人，本專案按「現狀」提供。

## 鳴謝

感謝以下開源專案與社群的傑出貢獻，MoLaunch 站在巨人的肩膀上。

### 特別感謝

- **[Arco Design Vue](https://github.com/arco-design/arco-design-vue)** — 前端核心元件（Button / Input / Select / Drawer / Slider 等）參考其特色與實作，提取原始碼複刻改寫為 Vue SFC + Tailwind 形式，版權聲明註解見各原始檔頂部
- **[Element Plus Icons](https://github.com/element-plus/element-plus-icons)** — 依需求提取 SVG path 資料（`src/utils/element-icons.ts`），僅引入圖示未引入執行期依賴
- **[Plain Craft Launcher 2 (PCL2)](https://github.com/Meloong-Git/PCL)** — 一款被廣泛使用的 Minecraft 啟動器；MoLaunch 前期從零開始開發，為相關啟動邏輯提供了實作參考

### 核心依賴

- **[Vue 3](https://github.com/vuejs/core)** / [Vue Router](https://github.com/vuejs/router) / [Pinia](https://github.com/vuejs/pinia) — 前端框架與狀態管理
- **[Tauri 2](https://github.com/tauri-apps/tauri)** — 桌面應用框架（Rust + WebView）
- **[Tailwind CSS](https://github.com/tailwindlabs/tailwindcss)** — 原子化 CSS 方案
- **[Heroicons](https://github.com/tailwindlabs/heroicons)** — 主圖示庫
- **[skinview3d](https://github.com/bs-community/skinview3d)** — 皮膚 3D 即時預覽
- **[Cubiomes](https://github.com/Cubitect/cubiomes)** — 世界結構產生演算法（編譯為 WASM）
- **[OpenLayers](https://github.com/openlayers/openlayers)** — 結構定址互動式地圖
- **[Tokio](https://github.com/tokio-rs/tokio)** / **[Reqwest](https://github.com/seanmonstar/reqwest)** — Rust 非同步執行環境與 HTTP 用戶端

完整第三方授權與版權清單見 [licenses.txt](./src-tauri/resources/about/licenses.txt)。

> [!NOTE]
> MoLaunch 為獨立第三方創作，與 PCL2 無隸屬或關聯；PCL2 採用《PCL 分發有限許可》，詳情參閱其[授權文件](https://shimo.im/docs/rGrd8pY8xWkt6ryW)。

## 貢獻者

感謝所有為 MoLaunch 貢獻過程式碼、文件與建議的開發者。

[![Contributors](https://contrib.rocks/image?repo=MoTeam-cn/MoLaunch)](https://github.com/MoTeam-cn/MoLaunch/graphs/contributors)

![Alt](https://repobeats.axiom.co/api/embed/8769aee202d5829171ef89b4ffa1e9907fab4d7a.svg "Repobeats analytics image")

## 相關連結

- 儲存庫：https://github.com/MoTeam-cn/MoLaunch
- 問題回饋：https://github.com/MoTeam-cn/MoLaunch/issues
- 更新日誌：[CHANGELOG.md](./CHANGELOG.md)
- 授權：[LICENSE](./LICENSE)
- 第三方著作權清單：[licenses.txt](./src-tauri/resources/about/licenses.txt)