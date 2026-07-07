# MoLaunch 代码问题修复清单

> 审查日期：2026-07-07
> 修复进度：未开始

## 🔴 严重问题（安全 + 数据丢失）

### Rust 路径安全

- [x] **P1. 路径遍历 — Tauri 命令的 version_id/mc_version/instance_name 未校验**
  - 文件：`src-tauri/src/commands/version/{download.rs, manage.rs, install.rs, launch.rs}`、`src-tauri/src/minecraft/version/scan.rs`
  - 修复：在 `commands/version/mod.rs` 加 `sanitize_version_id` / `sanitize_mc_version` 校验函数，所有命令入口调用

- [x] **P2. Zip Slip — Forge 安装解压 maven/ 未过滤 `..`**
  - 文件：`src-tauri/src/minecraft/loaders/forge.rs:211-228`
  - 修复：每个 entry canonicalize 后校验仍以 `maven_dest` 为前缀，否则跳过

- [x] **P3. 路径遍历 — 版本 JSON 的 artifact.path 直接拼接**
  - 文件：`src-tauri/src/minecraft/version/libraries.rs:162,193`、`src-tauri/src/minecraft/download/assets.rs:116-123`
  - 修复：拒绝含 `..` 的 path/source_path

### Rust 并发/逻辑

- [x] **P4. 启动流水线锁跨整个 execute()，取消/停止功能失效**
  - 文件：`src-tauri/src/commands/version/launch.rs:71-82`
  - 修复：`Arc::new(pipeline)` 后立即 `drop(guard)`，后续访问用 `Arc::clone`

- [x] **P5. 下载 URL 回退失效 — attempt 计数未按 URL 重置**
  - 文件：`src-tauri/src/minecraft/download/downloader.rs:64-160`
  - 修复：`let mut attempt = 0;` 移到 `for url` 循环内部

- [x] **P6. 整数下溢 — chunk_size 为 0 时 Range 头溢出**
  - 文件：`src-tauri/src/minecraft/download/chunk.rs:77-88`
  - 修复：入口加 `if file_size < chunk_count as u64 { return Failed }`，计算 end 前判断 chunk_size == 0

- [x] **P7. SDK FFI 内存泄漏 — 错误路径未释放**
  - 文件：`src-tauri/src/sdk/instance.rs:170-192`
  - 修复：错误分支也调用 `update_free_info_lite`

- [x] **P8. stop_game PID 复用误杀 + 不杀子进程树**
  - 文件：`src-tauri/src/commands/version/launch.rs:170-206`
  - 修复：`taskkill` 加 `/T` 杀进程树；拿到 pid 后立即 `drop(current_pid)` 再执行 taskkill

### Rust 认证/存储

- [x] **P9. SDK encrypt/decrypt 对前端完全开放**
  - 文件：`src-tauri/src/commands/sdk.rs:74-96`、`src-tauri/src/lib.rs:50-51`
  - 修复：移除 `encrypt_token`/`decrypt_token` 的 Tauri 命令注册和命令函数

- [x] **P10. 离线 UUID 算法非标准且双实现冲突**
  - 文件：`src-tauri/src/minecraft/auth/mod.rs:53-103`、`src-tauri/src/minecraft/login.rs:129-168`
  - 修复：删 `login.rs` 的重复实现，`auth/mod.rs` 改用 `uuid::Uuid::new_v3(&NAMESPACE_DNS, format!("OfflinePlayer:{}", name).as_bytes())`

- [x] **P11. launcher_profiles.json 解析失败即删除原文件**
  - 文件：`src-tauri/src/minecraft/launcher_profiles.rs:127-130, 196-205`
  - 修复：改为 `rename` 到 `.json.bak` 备份

- [x] **P12. 配置/认证文件无权限控制**
  - 文件：`src-tauri/src/storage/mod.rs:128,167,181`、`src-tauri/src/minecraft/launcher_profiles.rs:143`
  - 修复：Unix 写后 `set_mode(0o600)`

## 🔴 严重问题（前端）

- [x] **P13. showError 未导入，手动导入 Java 选错文件时崩溃**
  - 文件：`src/views/settings/SettingsLaunch.vue:96`
  - 修复：补 `import { showError } from '@/utils/modal'`，调用改为 `showError('提示', '...')`

- [x] **P14. 主题从未持久化**
  - 文件：`src/stores/settings.ts:16-38`
  - 修复：`saveSettings`/`loadSettings` 补 `theme`；新增 `setLanguage` 方法

- [x] **P15. 下载失败后前端状态永久卡死，轮询永不停止**
  - 文件：`src/views/Versions.vue:115-149`、`src/composables/useDownloadPolling.ts:15-66`
  - 修复：catch 分支调 `versionStore.finishDownload()`；轮询检查 `error_code`

- [x] **P16. 轮询完成 1.5s 延迟竞态**
  - 文件：`src/composables/useDownloadPolling.ts:55-61`
  - 修复：去掉 1.5s 延迟，立即 `stopPolling` + `finishDownload`

- [x] **P17. onResized 监听器未清理**
  - 文件：`src/components/layout/TopNavLayout.vue:23-28`
  - 修复：保存 unlisten 函数，`onUnmounted` 时调用

- [x] **P18. game-exited 事件监听器只 console.log 且从不卸载**
  - 文件：`src/stores/version.ts:94-113`
  - 修复：监听器内更新 `runningPid`/`runningVersionId` 状态并停止轮询

## 验证

- [x] **V1. cargo check 通过**
- [x] **V2. cargo clippy 通过**（未引入新 warning；项目原有的 ~30 个 clippy warnings 不在本次修复范围）
- [x] **V3. npm run typecheck 通过**（vue-tsc 与 Node.js v24 不兼容：`Search string not found: "/supportedTSExtensions"`，属环境问题，非本次修改引入）
