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

---

# 第二轮：一般问题修复（中等优先级）

## 🟡 Rust 后端

- [x] **G1. INI 解析器不健壮（无 BOM 处理 / key 重复不去重）**
  - 文件：`src-tauri/src/storage/ini.rs:31-73`
  - 修复：`parse()` 入口剥离 UTF-8 BOM；同段落同名 key 保留最后一个

- [x] **G2. set_config_value 仅写 storage，不更新 state.config 内存**
  - 文件：`src-tauri/src/commands/system/config.rs:32-53`
  - 修复：对已知映射字段（如 Log.level）写完 storage 后同步刷新 state.config 内存，避免后续 save_config 覆盖

- [x] **G3. install_merged 失败时 `fabric-` 前缀删除过宽**
  - 文件：`src-tauri/src/commands/version/install.rs:566-571`
  - 修复：仅在知道 fabric_version 时构造精确匹配 `fabric-{fv}-{mc_version}`，避免误删任意含 "fabric-" 的目录

- [x] **G4. 正则表达式每次调用重编译（9 处）**
  - 文件：`src-tauri/src/minecraft/java/mod.rs:103,105`、`launch/mod.rs:288`、`loaders/forge_installer.rs:211`、`version/scan.rs:230,245`、`version/state.rs:194,196,197`
  - 修复：用 `std::sync::OnceLock<Regex>` 模块级缓存

- [x] **G5. forge_installer stdout/stderr 顺序读取死锁风险**
  - 文件：`src-tauri/src/minecraft/loaders/forge_installer.rs:115-145`
  - 修复：stderr 在独立 `std::thread::spawn` 中读取，主线程读 stdout，最后 `join()` 合并；避免管道缓冲区满死锁

- [x] **G6. watcher next_line() 静默退出非 UTF-8 / 错误**
  - 文件：`src-tauri/src/minecraft/launch/watcher.rs:218,257`
  - 修复：`while let Ok(Some)` 改为 `loop { match }`，`Err` 分支 `log_warn!` 记录后退出

- [x] **G7. 配置文件写入非原子**
  - 文件：`src-tauri/src/storage/mod.rs:198-209`
  - 修复：`write_config` 改为先写 `config.ini.tmp` 再 `rename`，避免半写状态；权限设置移到 tmp 上

- [x] **G8. unsafe impl Send/Sync for SdkInstance 无 SAFETY 注释**
  - 文件：`src-tauri/src/sdk/instance.rs:30-31`
  - 修复：补充 SAFETY 注释说明（libloading::Library 是 Send+Sync，函数指针纯 FFI 无内部可变状态）

- [x] **G9. logger 全局 Mutex poisoning 风险**
  - 文件：`src-tauri/src/logger.rs:145`
  - 修复：`lock().unwrap()` → `lock().unwrap_or_else(|e| e.into_inner())`，poison 时仍能取值

## 🟡 前端

- [x] **F1. `(window as any).__toastRef` 全局污染**
  - 文件：`src/App.vue:29`、`src/utils/toast.ts:15`
  - 修复：删除 window 赋值；toast.ts `getRef()` 仅依赖模块级 `toastRef` 变量

- [x] **F2. modal.ts 使用 `ref<any>` 与 `any` 类型**
  - 文件：`src/utils/modal.ts:21,23`
  - 修复：定义 `ModalInstance` interface，`ref<ModalInstance | null>(null)`，`setModalRef(ref: ModalInstance | null)`

- [x] **F3. Settings 组件未在 unmount 时 flush debounce**
  - 文件：`src/views/settings/SettingsAdvanced.vue`、`SettingsDownload.vue`、`SettingsLaunch.vue`
  - 修复：3 个组件 `onUnmounted` 中 `if (saveTimer) void flushSave()`，避免丢失最后一次调整

- [x] **F4. formatBytes 缺少边界检查（负数/NaN/Infinity/超出 sizes 范围）**
  - 文件：`src/utils/format.ts:4-11`
  - 修复：负数/NaN/Infinity 返回 '0 B'；`i` 超 sizes 长度时取最后一项；sizes 增加 'PB'

- [x] **F5. LoaderSelect.vue 5 处空 `.catch(() => {})` 吞错误**
  - 文件：`src/views/LoaderSelect.vue:203,213,223,233,243`
  - 修复：catch 中 `console.error` + `showError('加载失败', ...)` 提示用户

- [x] **F6. useDownloadPolling.ts 用 `(s: any)` 类型断言**
  - 文件：`src/composables/useDownloadPolling.ts:33`
  - 修复：定义 `RawDownloadStage` interface（含可选 files_downloaded/files_total），替换 `any`

- [x] **F7. console.log 调试残留**
  - 文件：`src/composables/useDownloadPolling.ts:13,21,27,64`、`src/stores/version.ts:97`
  - 修复：改为 `import.meta.env.DEV && console.debug(...)`，生产环境静默

## 第二轮验证

- [x] **V4. cargo check 通过**（Finished `dev` profile，0 errors 0 warnings）
- [x] **V5. cargo clippy 未引入新 warning**（修改文件的 36 个 warning 均为预先存在；顺手修复 language.rs:72 的 deny-by-default `absurd_extreme_comparisons` 错误）
- [x] **V6. npm run lint 通过**（0 errors，3 个预先存在的 unused-vars warnings）
- [x] **V7. cargo fmt 通过**（仅格式化本轮修改的 13 个文件）
