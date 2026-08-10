<p align="center">
  <img src="images/splash.gif" alt="MoLaunch スプラッシュアニメーション" width="800" />
</p>

# MoLaunch

[简体中文](./README.md) · [繁體中文](./README_ZH-HANT.md) · [English](./README_EN.md) · [日本語](./README_JA.md)

モダンでクロスプラットフォームな Minecraft Java Edition ランチャー。

[![License](https://img.shields.io/badge/License-MoLaunch%20Limited%20Distribution%20License-red.svg)](./LICENSE)
[![Version](https://img.shields.io/badge/version-0.3.5--rc1-blue.svg)](https://github.com/MoTeam-cn/MoLaunch)
[![Tauri](https://img.shields.io/badge/Tauri-2-orange.svg)](https://v2.tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3-42b883.svg)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-2021-dea584.svg)](https://www.rust-lang.org/)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](https://github.com/MoTeam-cn/MoLaunch)

[![GitHub stars](https://img.shields.io/github/stars/MoTeam-cn/MoLaunch?style=flat&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiIgdmlld0JveD0iMCAwIDE2IDE2IiBmaWxsPSIjZmZmZmZmIj48cGF0aCBkPSJNOCAuMjVhLjc1Ljc1IDAgMCAxIC42NzMuNDE4bDEuODgyIDMuODE1IDQuMjEuNjEyYS43NS43NSAwIDAgMSAuNDE2IDEuMjc5bC0zLjA0NiAyLjk3LjcxOSA0LjE5MmEuNzUxLjc1MSAwIDAgMS0xLjA4OC43OTFMOCAxMi4zNDdsLTMuNzY2IDEuOThhLjc1Ljc1IDAgMCAxLTEuMDg4LS43OWwuNzItNC4xOTRMLjgxOCA2LjM3NGEuNzUuNzUgMCAwIDEgLjQxNi0xLjI4bDQuMjEtLjYxMUw3LjMyNy42NjhBLjc1Ljc1IDAgMCAxIDggLjI1WiIvPjwvc3ZnPg==&labelColor=165dff&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/MoTeam-cn/MoLaunch?style=flat&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxNiIgaGVpZ2h0PSIxNiIgdmlld0JveD0iMCAwIDE2IDE2IiBmaWxsPSIjZmZmZmZmIj48cGF0aCBkPSJNNSA1LjM3MnYuODc4YzAgLjQxNC4zMzYuNzUuNzUuNzVoNC41YS43NS43NSAwIDAgMCAuNzUtLjc1di0uODc4YTIuMjUgMi4yNSAwIDEgMSAxLjUgMHYuODc4YTIuMjUgMi4yNSAwIDAgMS0yLjI1IDIuMjVoLTEuNXYyLjEyOGEyLjI1MSAyLjI1MSAwIDEgMS0xLjUgMFY4LjVoLTEuNUEyLjI1IDIuMjUgMCAwIDEgMy41IDYuMjV2LS44NzhhMi4yNSAyLjI1IDAgMSAxIDEuNSAwWk01IDMuMjVhLjc1Ljc1IDAgMSAwLTEuNSAwIC43NS43NSAwIDAgMCAxLjUgMFptNi43NS43NWEuNzUuNzUgMCAxIDAgMC0xLjUuNzUuNzUgMCAwIDAgMCAxLjVabS0zIDguNzVhLjc1Ljc1IDAgMSAwLTEuNSAwIC43NS43NSAwIDAgMCAxLjUgMFoiLz48L3N2Zz4=&labelColor=165dff&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/forks)
[![GitHub issues](https://img.shields.io/github/issues/MoTeam-cn/MoLaunch?style=flat&logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/issues)
[![GitHub last commit](https://img.shields.io/github/last-commit/MoTeam-cn/MoLaunch?style=flat&logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/commits)
[![GitHub contributors](https://img.shields.io/github/contributors/MoTeam-cn/MoLaunch?style=flat&logo=github&logoColor=white&color=165dff)](https://github.com/MoTeam-cn/MoLaunch/graphs/contributors)

> [!CAUTION]
> MoLaunch は独立したサードパーティ製 Minecraft ランチャープロジェクトであり、Mojang や Microsoft の公式製品ではありません。また、両社による承認や提携も受けていません。

> [!IMPORTANT]
> 本プロジェクトは個人が独自に開発しており、手間を省くため多くの部分を AI 支援（Vibe Coding）で記述しています。完成度に不満な点があるかもしれませんが、ご容赦ください。

## 概要

MoLaunch は、ダウンロード・インストール・起動・オンラインマルチプレイなど、ゲーム管理に必要な機能を一式備えた Minecraft Java Edition ランチャーです。**Tauri 2 + Vue 3 + Rust** で構築し、実用的なゲームツールも内蔵しています。

本リポジトリは MoLaunch のオープンソースコードです。ビルド済みの成果物をそのまま利用できますし、本リポジトリを元に自前でコンパイル・二次開発することもできます。

## 画面プレビュー

<table align="center">
  <tr>
    <td align="center" width="50%">
      <b>ランチャーホーム</b><br/>
      <sub>デフォルトはシンプルなコンテンツ領域。レイアウトは設定で変更可能（images/001.png）</sub><br/><br/>
      <img src="images/001.png" alt="ランチャーホーム" width="380" />
    </td>
    <td align="center" width="50%">
      <b>バージョン一覧</b><br/>
      <sub>正式版 / スナップショット版 / エイプリルフール版 / レガシー版の 4 種（images/002.png）</sub><br/><br/>
      <img src="images/002.png" alt="バージョン一覧" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>コミュニティ Modパック</b><br/>
      <sub>Modrinth / CurseForge の Modパック一覧とインストール（images/003.png）</sub><br/><br/>
      <img src="images/003.png" alt="コミュニティ Modパック" width="380" />
    </td>
    <td align="center">
      <b>オンラインロビー</b><br/>
      <sub>ルームを作成・参加してオンラインマルチプレイ（現在テスト用ルームはありません）（images/004.png）</sub><br/><br/>
      <img src="images/004.png" alt="オンラインロビー" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>シードマップ</b><br/>
      <sub>シード値を入力してワールド構造を探知（精度は改善中）（images/005.png）</sub><br/><br/>
      <img src="images/005.png" alt="シードマップ" width="380" />
    </td>
    <td align="center">
      <b>AI チャット（実験的）</b><br/>
      <sub>ログやクラッシュ原因の解析を助ける対話型アシスタント（images/006.png）</sub><br/><br/>
      <img src="images/006.png" alt="AI チャット" width="380" />
    </td>
  </tr>
  <tr>
    <td align="center">
      <b>スキン・ケープ管理</b><br/>
      <sub>スキンとケープの変更（images/007.png）</sub><br/><br/>
      <img src="images/007.png" alt="スキン・ケープ管理" width="380" />
    </td>
    <td align="center">
      <b>設定画面</b><br/>
      <sub>外観、起動・ダウンロードの設定、開発者向けオプション（images/008.png）</sub><br/><br/>
      <img src="images/008.png" alt="設定画面" width="380" />
    </td>
  </tr>
</table>

## 機能一覧

### バージョン管理

- バニラ / Forge / Fabric / NeoForge / OptiFine / LiteLoader のローダーに対応
- CurseForge / Modrinth の Modパックインストール。ローダーと依存関係を自動で補完
- バージョンごとに分離保存し、各インスタンスを独立管理
- Java を自動検出：バージョンに応じて実行環境を検証し、不足時は公式の Mojang Runtime を事前ダウンロード

### ダウンロード

- Mod / テクスチャ（リソースパック）/ Modパックの検索・インストール。CurseForge、Modrinth、BMCLAPI ミラーに対応
- 分割並列ダウンロード、レジューム、一時停止、整合性検証
- 中国国内向けミラー高速化（BMCLAPI / MoCDN）

### アカウント

- Microsoft OAuth（デバイスコード）ログイン、オフラインアカウント
- 認証情報はローカルに暗号化保存され、自動リフレッシュ

### スキン

- スキン / ケープの管理
- 3D リアルタイムプレビュー（skinview3d）

### オンラインマルチプレイ

- オンラインロビー・ルーム管理。招待コード / ブラックリスト機能
- WebRTC による P2P 仮想 LAN（仮想 TUN アダプタ）
- FRP トンネル。複数プロバイダに対応し、ポートオープン不要

### ツール

- シードマップ：シード値を入力して要塞・海底神殿などのワールド構造を探す（cubiomes WASM）
- NBT 編集、セーブデータのバックアップ / 復元、Mod 依存関係チェック、サーバー遅延テスト、Java 実行環境の検出など

### AI アシスタント（実験的）

- ゲームログ・クラッシュレポート・Mod 一覧を読み込める対話型アシスタント
- ログ / クラッシュ解析：ローカルルールエンジンによる一次チェック + AI による深い分析
- OpenAI 互換エンドポイントに対応（DeepSeek R1 などの推論モデルを含む）

### プラグイン

- プラグイン SDK とサンドボックス：カスタムレイアウト、システムモニタリング、起動履歴など。権限も設定可能

### その他

- オープニングアニメーション（二重ウィンドウのスプラッシュスクリーン）
- クラッシュ解析（ルールエンジン + AI 提案）
- 自動アップデート（stable / beta / alpha チャンネル）
- ログのマスキング処理、グローバル CSP、利用規約ゲート

### MoLaunch クラウド

登録・ログイン・認証情報のリフレッシュなどの操作は MoLaunch クラウド（api-server）と連携します。クラウドはこれらの API の前に軽量な Proof-of-Work（PoW）認証を挟み、1 問のハッシュ問題を解いてからリクエストを通過させます。これによりスクリプトを使った大量APIアクセスを防いでいます。通常の利用ではほぼ無感覚—ログインなどは数十ミリ秒で完了します。実際にこの検証でブロックされるのは、大量に API を叩く人だけです。

## ライセンス

MoLaunch 独自のコードとオリジナルリソースは [MoLaunch 配布ライセンス（有限許諾）](./LICENSE) に従います。主な要件：

- MoLaunch またはそれを二次開発したバージョンを商用製品として利用・販売してはならない
- 二次開発者は完全なソースコードを公開し、サードパーティ版であることを明確に示さなければならない（公式と誤認されやすい名称は使えない）
- 著作権、ライセンス、商標、免責事項を除去してはならない

サードパーティの依存ライブラリ、同梱リソース、参照プロジェクトはそれぞれのライセンスに従います。詳細は [licenses.txt](./src-tauri/resources/about/licenses.txt) を参照してください。商用ライセンスなどの例外が必要な場合は、MoTeam に連絡して書面による承認を得てください。

Minecraft は Mojang Synergies AB の商標です。MoLaunch は Mojang、Microsoft、その他の権利者とは一切関係ありません。本プロジェクトは「現状のまま（AS IS）」で提供されます。

## 謝辞

以下に挙げるオープンソースプロジェクトとコミュニティの多大な貢献に感謝します。MoLaunch は巨人の肩の上に立っています。

### 特筆すべき感謝

- **[Arco Design Vue](https://github.com/arco-design/arco-design-vue)** — フロントエンドのコアコンポーネント（Button / Input / Select / Drawer / Slider など）の実装を参考に、ソースコードを抽出して Vue SFC + Tailwind 形式に書き直しています。著作権表示は各ソースファイルの先頭に記載
- **[Element Plus Icons](https://github.com/element-plus/element-plus-icons)** — SVG パスデータを抽出して使用（`src/utils/element-icons.ts`）。アイコンのみで実行時依存はなし
- **[Plain Craft Launcher 2 (PCL2)](https://github.com/Meloong-Git/PCL)** — 広く使われている Minecraft ランチャー。MoLaunch は当初ゼロから開発を始めましたが、一部の起動ロジックで参考にさせていただきました

### コア依存

- **[Vue 3](https://github.com/vuejs/core)** / [Vue Router](https://github.com/vuejs/router) / [Pinia](https://github.com/vuejs/pinia) — フロントエンドのフレームワークと状態管理
- **[Tauri 2](https://github.com/tauri-apps/tauri)** — デスクトップアプリケーションフレームワーク（Rust + WebView）
- **[Tailwind CSS](https://github.com/tailwindlabs/tailwindcss)** — ユーティリティファーストな CSS ソリューション
- **[Heroicons](https://github.com/tailwindlabs/heroicons)** — メインのアイコンライブラリ
- **[skinview3d](https://github.com/bs-community/skinview3d)** — スキンの 3D リアルタイムプレビュー
- **[Cubiomes](https://github.com/Cubitect/cubiomes)** — ワールド構造生成アルゴリズム（WASM にコンパイル）
- **[OpenLayers](https://github.com/openlayers/openlayers)** — 構造検索用のインタラクティブマップ
- **[Tokio](https://github.com/tokio-rs/tokio)** / **[Reqwest](https://github.com/seanmonstar/reqwest)** — Rust 非同期ランタイムと HTTP クライアント

サードパーティのライセンスと著作権の完全な一覧は [licenses.txt](./src-tauri/resources/about/licenses.txt) を参照してください。

> [!NOTE]
> MoLaunch は、PCL2 とは一切関係のない独立したサードパーティ製のソフトウェアです。PCL2 は「PCL 配布ライセンス」の条件で配布されており、詳細はその[ライセンス文書](https://shimo.im/docs/rGrd8pY8xWkt6ryW)をご覧ください。

## 貢献者

MoLaunch にコード・ドキュメント・提案を提供してくださったすべての開発者に感謝します。

[![Contributors](https://contrib.rocks/image?repo=MoTeam-cn/MoLaunch)](https://github.com/MoTeam-cn/MoLaunch/graphs/contributors)

## 関連リンク

- リポジトリ：https://github.com/MoTeam-cn/MoLaunch
- 問題のフィードバック：https://github.com/MoTeam-cn/MoLaunch/issues
- 更新ログ：[CHANGELOG.md](./CHANGELOG.md)
- ライセンス：[LICENSE](./LICENSE)
- サードパーティの著作権一覧：[licenses.txt](./src-tauri/resources/about/licenses.txt)