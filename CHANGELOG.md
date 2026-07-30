# 更新日志

本项目的所有重要更改都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增

#### 测试版水印 + 设备 ID 追踪 + DevTools 防护

- 背景：MoLaunch 进入测试阶段，测试版分发给内部测试用户。为防止未授权外传，需要全屏水印 + 设备 ID 追踪 + 禁用右键/DevTools 快捷键，仅在开发者模式开启时才能通过设置页按钮调出 WebView2 DevTools
- 改动：
  - **后端 devtools 控制**（[src-tauri/src/commands/system/developer.rs](src-tauri/src/commands/system/developer.rs)）：新增 `open_devtools` / `close_devtools` / `is_devtools_open` 三个函数，均通过 `require_dev_mode()` 双层校验（`DeveloperUnlocked` && `DeveloperMode`），普通用户即使绕过前端按钮直接调 IPC 也无法触发
  - **后端 IPC 注册**（[src-tauri/src/utils/system_manager.rs](src-tauri/src/utils/system_manager.rs)）：DISPATCHER 注册 3 个 devtools action
  - **Cargo.toml 启用 devtools feature**（[src-tauri/Cargo.toml](src-tauri/Cargo.toml)）：`tauri = { version = "2", features = ["devtools"] }`，让 release 构建也支持 `open_devtools()` 调用
  - **版本号后缀解析**（新建 [src/utils/version.ts](src/utils/version.ts)）：解析 `package.json` version 后缀（beta/alpha/rc/canary），判定是否为测试版；提供 `isPreReleaseBuild()` / `getBuildFingerprint()` 等便捷函数
  - **前端 devtools API**（[src/utils/api/developer.ts](src/utils/api/developer.ts)）：新增 `openDevTools()` / `closeDevTools()` / `isDevToolsOpen()` 三个 IPC 调用
  - **SYSTEM_ACTIONS 扩展**（[src/utils/api/system-manager.ts](src/utils/api/system-manager.ts)）：新增 `OPEN_DEVTOOLS` / `CLOSE_DEVTOOLS` / `IS_DEVTOOLS_OPEN` 三个 action 常量
  - **水印数据 composable**（新建 [src/composables/useWatermarkData.ts](src/composables/useWatermarkData.ts)）：提供水印组件所需的设备 ID（去 `mcsdk-` 前缀）、版本号、屏印哈希（djb2 算法，按小时分桶）、时间标签
  - **DevTools 防护 composable**（新建 [src/composables/useDevToolsGuard.ts](src/composables/useDevToolsGuard.ts)）：capture 阶段拦截 `contextmenu` / `keydown`（F12 / Ctrl+Shift+I/J/C/K / Cmd+Opt+I/J/C/K / Ctrl+U）/ `dragstart`，禁用右键菜单与 DevTools 快捷键
  - **水印组件**（新建 [src/components/common/Watermark.vue](src/components/common/Watermark.vue)）：全屏 45° 斜向重复文字水印，使用 SVG `<pattern>` 实现可重复单元格（280×140px），三行文字（测试版标识+版本号 / 设备ID / 屏印哈希+时间），文字 `rgba(0,0,0,0.06)` 低对比度，`pointer-events: none` 不影响交互；SVG 元素携带 `data-device` / `data-hash` / `data-time` / `data-build` 属性便于追溯
  - **DevTools 子页签**（新建 [src/views/settings/developer/DevToolsTab.vue](src/views/settings/developer/DevToolsTab.vue)）：开发者页面新增 DevTools 子页签，提供「打开 DevTools」/「关闭 DevTools」按钮
  - **SettingsDeveloper 注册子页签**（[src/views/settings/SettingsDeveloper.vue](src/views/settings/SettingsDeveloper.vue)）：在 subTabs 中追加 `devtools` 项
  - **SystemInfoTab 备用解锁**（[src/views/settings/more/SystemInfoTab.vue](src/views/settings/more/SystemInfoTab.vue)）：新增设备 ID 双击 5 次备用解锁入口（4 秒内完成），已解锁状态下双击切换全额显示
  - **App.vue 集成**（[src/App.vue](src/App.vue)）：引入 `Watermark` 组件 + `useDevToolsGuard` composable，全局生效
  - **版本号变更为测试版**（[package.json](package.json) / [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)）：`0.1.0` → `0.1.0-beta.1`
  - **设计文档**（新建 [docs/WATERMARK_AND_DEVTOOLS_DESIGN.md](docs/WATERMARK_AND_DEVTOOLS_DESIGN.md)）：完整设计方案
  - **实现文档**（新建 [docs/WATERMARK_AND_DEVTOOLS_IMPLEMENTATION.md](docs/WATERMARK_AND_DEVTOOLS_IMPLEMENTATION.md)）：最终实现说明
- 用户反馈："看下前端，现在添加测试版水印，因为我要打算发布测试版了，需要水印以及追踪id，也就是设备id，别带 mcsdk- 前缀。充分做好防止被去查水印的情况，同时做好屏印，即使被通过截图或者拍照传出去，也可以迅速追踪解密图片信息获得设备id和时间，然后就是因为是前端实现，做好被通过技术手段去掉水印的反制手段，然后添加右键菜单禁用，后续想通过右键或者快捷键调出 WebView2的开发者等其他模式不允许，然后去 设置页面的开发者侧边栏菜单页面去添加可以通过在那点击按钮弹出来，这样安全方便，然后目前触发开发者模式好像无法触发了，之前我设计的是 双击五次设备id，目前没有了，你设计一个新方案出来我瞅瞅呢"
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过（零错误零警告）

#### 中文搜索本地映射（参考 PCL2 实现）

- 背景：MoLaunch 原样透传中文关键词给 CurseForge / Modrinth 官方 API，两大平台索引不含中文，中文搜索几乎返回空结果。PCL2 通过内置 MC百科（mcmod.cn）本地数据库实现中文搜索，本次参考其思路在 MoLaunch 中实现等价功能
- 改动：
  - **模糊匹配算法**（新建 [src-tauri/src/minecraft/community/fuzzy.rs](src-tauri/src/minecraft/community/fuzzy.rs)）：移植 PCL2 `ModBase.vb:818-946` 的 `SearchSimilarity` / `Search` 算法，基于最长公共子串的相似度，考虑长度加成（`1.4^(3+len) - 3.6`）和位置加成（`1 + 0.3 * max(0, 3-|qp-sp|)`），含 `SearchSource` / `SearchEntry<T>` 泛型类型和单元测试
  - **数据层扩展**（[src-tauri/src/minecraft/community/mcmod.rs](src-tauri/src/minecraft/community/mcmod.rs)）：`Entry` 新增 `popularity` 字段（解析 moddata.txt 最后一行排行数据）；`Database` 新增 `entries: Vec<ChineseSearchEntry>` 反查列表；新增 `search_by_chinese(query) -> RewriteResult` 公开函数，用本地模糊匹配把中文关键词重写为 CurseForge/Modrinth 英文 Slug/单词，并收集 Modrinth Slug 直查列表（最多 100 个）；新增 `extract_words` 单词提取（过滤停用词、单字、纯数字、子串去重）
  - **模块导出**（[src-tauri/src/minecraft/community/mod.rs](src-tauri/src/minecraft/community/mod.rs)）：导出 `fuzzy` 模块
  - **调度层拦截**（[src-tauri/src/minecraft/community/searcher.rs](src-tauri/src/minecraft/community/searcher.rs)）：在 `search()` 入口新增 `is_chinese` 检测（CJK 统一汉字 + 扩展 A + 兼容 ideographs），检测到中文时调 `mcmod::search_by_chinese` 重写查询词；三路并行（CF 搜索 + MR 搜索 + MR Slug 直查）通过 `tokio::join!` 调度，各自独立超时/错误隔离；中文未命中时回退原词透传
  - **Modrinth Slug 直查**（[src-tauri/src/minecraft/community/modrinth/mod.rs](src-tauri/src/minecraft/community/modrinth/mod.rs)）：新增 `get_projects_by_slugs(slugs, rtype) -> Vec<ResourceProject>`，调 `GET /v2/projects?ids=[...]` 批量拉取工程详情（slug 作为 project_id 别名），复用 `convert_project` 转换并写入缓存，失败返回空 Vec 不阻断搜索
  - **实现文档**（新建 [docs/CHINESE_SEARCH_IMPLEMENTATION.md](docs/CHINESE_SEARCH_IMPLEMENTATION.md)）：记录最终代码结构、与设计文档差异、验证方法、性能考量
- 用户反馈："我搜索模组或整合包时使用中文，两个平台都返回空，PCL2 用中文都能搜出来"
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过（零错误零警告）；`cargo test fuzzy` / `cargo test mcmod` 单元测试通过
- 参考：PCL2 `code-libs/Plain Craft Launcher 2/Modules/Resource/ResourceSearcher.vb` 189-290 行

### 修复

#### 资源包转换在版本隔离模式下报 "resourcepacks 目录解析失败: os error 2"

- 背景：`convert` 函数与 `list` 函数路径解析逻辑不一致——`list` 通过 `resolve_packs_dir` 支持版本隔离并对不存在目录做优雅降级；`convert` 硬编码用全局 `game_dir/resourcepacks`，既忽略 `version_id`，又缺少 `exists()` 预检查。用户启用版本隔离、选中具体版本时，列表能正常加载（走版本隔离目录），但点击"转换为文件夹"会因全局 `resourcepacks` 目录不存在而 `canonicalize()` 立即抛出 `os error 2`
- 改动：
  - **后端参数类型**（[src-tauri/src/commands/tools/types.rs](src-tauri/src/commands/tools/types.rs)）：`ResourcePackConvertParams` 新增 `version_id: Option<String>` 字段（`#[serde(default)]`，与 `ResourcePackListParams` 语义一致），保持向后兼容
  - **后端 convert 函数**（[src-tauri/src/commands/tools/resourcepack.rs](src-tauri/src/commands/tools/resourcepack.rs)）：移除"convert 不需要 version_id"错误注释，改用 `resolve_packs_dir(state, params.version_id.as_deref())` 解析基准目录（与 `list` 完全一致）；在 `canonicalize` 之前增加 `exists()` 预检查，目录不存在时返回明确提示"resourcepacks 目录不存在: {path}（请在游戏中放置资源包后再转换）"而非裸 `os error 2`
  - **前端 API**（[src/utils/api/tools.ts](src/utils/api/tools.ts)）：`resourcepackConvert` 新增可选 `versionId?: string` 参数，传递 `version_id: versionId ?? null`
  - **前端组件**（[src/views/tools/data/ResourcePackConverter.vue](src/views/tools/data/ResourcePackConverter.vue)）：`doConvert` 调用 `resourcepackConvert` 时透传 `selectedVersionId.value`，确保转换走与列表相同的版本隔离目录
- 用户反馈："我选择转换为文件夹按钮，他给我个报错？resourcepacks 目录解析失败: 系统找不到指定的文件。 (os error 2)"
- 验证：`cargo check -p mo-launch` 通过（零错误零警告）

### 新增

#### api-server v3 无鉴权更新查询接口

- **`GET /v3/updates/manifest`**（`api-server/src/controllers/v3/updates.rs`）：
  - 新增 v3 简化版更新清单端点，无鉴权（不要求 JWT/CSRF），供 Tauri 客户端直接查询
  - 仅返回基础信息（`version` / `url` / `signature` / `notes`），不含 `pub_date` / `force_update` / `release_url` / `rollout_pct` 等 v1 扩展字段
  - 复用 `UpdatesService::check_for_update` 业务逻辑，`device_pk` 传空字符串（灰度按 `hash("")` 计算，全量发布不受影响）
  - 无可用更新返回 HTTP 204，更新服务未启用返回 503，参数错误返回 400
  - 用户反馈："顺便修复updater接口的问题，目前只提供了 /v1业务接口，也应该提供 /v3业务接口，但是/v3必须比 /v1的updater提供信息少，只返回基础信息就行了"
- **路由注册**（`api-server/src/controllers/v3/mod.rs`）：`/updates` 子模块挂载到 v3 公共路由树
- **OpenAPI 文档**（`api-server/src/docs/registry.rs`）：合并 `V3UpdatesApiDoc`，Swagger UI 可见 `/v3/updates/manifest` 端点

### 修复

#### MoLaunch updater 切换到 v3 无鉴权端点

- `src-tauri/src/commands/system/updater.rs`：
  - `UPDATER_PATH` 从 `/v1/updates/manifest/raw` 改为 `/v3/updates/manifest`（无鉴权端点）
  - 移除 JWT 加载逻辑（`load_creds_with_auto_refresh`）和 `Authorization: Bearer` 请求头，v3 端点无需鉴权
  - 日志不再输出 `(auth: true/false)` 标识
  - 用户反馈："目前updater的返回raw是要鉴权，没有豁免的，到时候修复下apiServer"
- `src-tauri/tauri.conf.json`：
  - `plugins.updater.endpoints[0]` 从 `/v1/updates/manifest/raw` 改为 `/v3/updates/manifest`（macOS/Linux 官方 plugin 读取此配置）

#### 联机设置 Server 地址开发者模式校验 + 组件拆分

- **后端校验**（`src-tauri/src/commands/system/apply_config/apply.rs`）：
  - `apply_online` 函数在更新 `api_server_url` 前调用 `secure::read_developer()` 检查开发者模式状态
  - 开发者模式关闭时静默忽略更新（不写入 config.ini，不报错），与 `ignore_tls` 关闭联动语义一致
  - 用户反馈："后端 apply_config 添加下判断，如果更新 Server 地址，必须检查开发者模式是否打开，关闭状态不准更新，即使更新了 config.ini 的字段也自动无视"
- **前端联动**（`src/views/settings/SettingsOnline.vue` + `src/components/settings/ApiServerCard.vue`）：
  - 新增 `devMode` 状态，从 `useConfigPage` 的 `onLoad` 读取 `cfg.developerMode`
  - 开发者模式关闭时：禁用服务器地址输入框、禁用重置按钮、隐藏测试连通性按钮、显示「需开启开发者模式才能修改服务器地址」提示
  - `watch(apiUrl)` 在 `devMode=false` 时不触发 `markDirty`，避免误传无效 patch
  - 监听 `DevModeToggle.vue` 分发的 `developer-mode-changed` 自定义事件，实时联动可编辑状态
- **组件拆分**（解决 Vue 文件 ≤ 300 行硬约束）：
  - 新建 `src/components/settings/ApiServerCard.vue`（232 行）：包含云端连接状态、服务器地址（开发者模式校验）、测试连通性，自管理加载状态（`useConfigPage` 独立实例，共享全局配置缓存）
  - `SettingsOnline.vue` 从 387 行降至 177 行：仅保留 ICE 服务器配置 + 设备管理，`<ApiServerCard />` 自管理加载占位

#### 诊断工具版本 JSON 编辑器「未保存的修改」误报

- `src/views/tools/data/VersionJsonEditor.vue`：`watch(content, ...)` 添加 `{ flush: 'sync' }` 选项
  - **根因**：默认 `flush: 'pre'`（微任务）在 `loading.value=false` 之后才执行 watcher，导致 `loadJson` 中 `content.value = res.content` 赋值时 `loading` 已为 false，误设 `dirty=true`
  - **修复**：`flush: 'sync'` 确保 watcher 在 `content` 赋值时同步执行，此时 `loading.value` 仍为 true，不误设 dirty

#### 联机设置页面布局调整

- `src/views/settings/SettingsOnline.vue`（已迁移至 `ApiServerCard.vue`）：
  - 测试连通性按钮容器改为 `flex justify-between`，按钮靠右，左侧显示「（默认地址）」提示
  - 连接成功/失败提示改为 `flex justify-end`，右对齐与按钮对齐

#### 检查更新报错 "missing field version" + 不走代理

- `src-tauri/src/commands/system/updater.rs`：
  - **根因**：`tauri-plugin-updater` 内部使用自己的 HTTP 客户端，既不走 `http.rs` 配置的代理，又在服务器返回空 manifest（无 `version` 字段）时报 serde 反序列化错误
  - **修复**：`check_update` 不再依赖 `updater.check()`，改为使用 `crate::http::get_client()`（走用户配置的代理）手动请求 manifest endpoint，解析 JSON 并比较版本
  - 新增 `platform_target()`：构造目标三元组用于 endpoint 模板替换
  - 新增 `is_version_newer()`：简单 semver 比较（major.minor.patch）
  - `UpdaterExt` 导入改为 `#[cfg(not(target_os = "windows"))]`（仅 macOS/Linux 下载安装路径使用）

#### 检查更新继承联机 base_url + 携带 JWT auth

- `src-tauri/src/commands/system/updater.rs`：
  - **base_url**：`UPDATER_ENDPOINT` 常量拆分为 `UPDATER_PATH`（仅路径模板），base_url 改为从 `AppConfig.online.api_server_url` 读取，不再硬编码 `https://api.molaunch.moiu.cn`
  - **鉴权**：`check_update` 签名新增 `&AppState` 参数，调用 `load_creds_with_auto_refresh` 尝试加载设备 JWT，有则携带 `Authorization: Bearer {jwt}` 头；未注册设备时无 auth 请求（服务端后续将添加 raw endpoint 豁免）
  - **日志**：`[Updater] 检查更新` 日志新增 `(auth: true/false)` 标识是否携带鉴权
- `src-tauri/src/utils/system_manager.rs`：`check_update` handler 从 `handler!(_state, app, ...)` 改为 `handler!(state, app, ...)`，传入 `&state` 供读取配置和凭证
  - **target 参数修复**：`platform_target()` 从返回目标三元组（`x86_64-pc-windows-msvc`）改为返回 OS 名称（`windows` / `macos` / `linux`），与服务端校验一致（服务端报错「平台取值非法」）

#### 前置检查场景区分 + 下载按钮分阶段文字 + 下载完成自动撤销

- **场景区分**（`src/composables/useResourceDownload.ts`）：
  - Community 场景（下载页 Mod 搜索弹窗，无 `versionId`）：不再检查前置 Mod，直接调用文件选择对话框让用户选保存位置
  - 版本管理场景（从版本设置 Mod 列表进入，有 `versionId`）：仅当 Mod 类型且有 `dependencies` 时才触发前置检查
  - 用户反馈："直接在那个页面下载的话，不用询问是否补充前置了，除非从版本设置的 mod 列表过来选择 mod 的情况"
- **下载按钮分阶段文字**（`src/composables/useResourceDownload.ts` + `src/components/community/resource-detail/VersionGroupCard.vue`）：
  - 新增 `downloadStage` 响应式状态（`'idle' | 'requesting' | 'waiting' | 'downloading'`），区分下载全流程阶段
  - 按钮文字分阶段显示：请求中（前置检查/准备）→ 等待中（前置弹窗等待用户确认）→ 下载中（实际下载）
  - 通过 `ResourceDetail.vue` 将 `downloadStage` 透传给 `VersionGroupCard`
  - 用户反馈："点击下载按钮，没显示补充前置的过度阶段显示请求中..，然后显示了前置 mod 的弹窗阶段改成 等待中... 然后我在弹窗选择下载后，才显示下载中"
- **下载完成自动撤销**（`src/composables/useResourceDownload.ts`）：
  - `handleDownload`：`downloadResourceToPath` IPC 返回成功后，若 `versionStore.downloading` 仍为 true（WS 因时序未收到 `is_complete`），直接调用 `finishDownload` + `toastSuccess` 兜底
  - `handleDependencyConfirm`：`installDeps` IPC 返回成功后，同样兜底 `finishDownload`
  - `finishDownload` 幂等（仅置 false/null），WS 已完成时无副作用
  - 用户反馈："下载模组完成后，DownloadManager 不会自动撤销，进入页面显示没下，但实际 toast 都提示了"
- **前置列表图片懒加载警告**（`src/components/community/resource-detail/DependencyInlineList.vue`）：移除 `loading="lazy"` 属性，CachedImage 组件已有缓存机制，浏览器原生懒加载触发 Chromium `[Intervention]` 控制台警告

### 重构

#### ResourceDetail.vue 拆分降至 300 行以内

- `src/components/community/ResourceDetail.vue`：从 527 行降至 190 行（项目硬约束 Vue 组件 ≤ 300 行）
  - 下载 + 前置 Mod 检查逻辑抽到 `src/composables/useResourceDownload.ts`（downloading / 弹窗状态 / depsMap / handleDownload / runDependencyCheck / handleDependencyConfirm / handleDependencyClose / handleLoadDeps）
  - 整合包安装逻辑抽到 `src/composables/useResourceModpackInstall.ts`（handleInstallModpack / promptForInstanceName），与联机大厅 `useModpackInstall.ts` 流程一致但入参不同（ResourceProject + ResourceVersion vs ModpackMeta），故独立封装
  - 组件仅保留版本列表加载 watch、useVersionGroups / useSearchProgress 编排、模板
- **响应式修正**：composable 直接接收 `props`（Vue 3 reactive proxy）而非快照对象，避免用户切换资源时 composable 仍看到旧 project
- **复用约定**：两个 composable 均复用 useDependencyCheck / getProjectDetail / formatDownloadFilename / pickSavePath / installModpack / installMerged / showModal / versionStore 现有约定，无新增 API

### 修复

#### Community 场景前置 Mod 检查 + 弹窗层级修复

- **根因**：Community 场景下（下载页 Mod 搜索弹窗）点击下载按钮不弹出前置确认弹窗。`useResourceDownload.ts` 的 `runDependencyCheck` 在 `versionId` 为空时直接 `return false` 跳过检查，而 Community 场景下 `versionStore.selectedVersion` 为空；且后端 `check_mod_dependencies` / `install_mod_with_dependencies` 强制要求 `version_id` 来解析 mods 目录
- **后端改造**（`src-tauri/src/commands/version/mods/dependency_resolver.rs` + `src-tauri/src/utils/version_mods_manager.rs`）：
  - `check_mod_dependencies`：`version_id` 改为 `Option<&str>`，新增 `mods_dir: Option<&str>`。优先用 version_id 解析 mods 目录，其次用传入的 mods_dir，都无则跳过已安装扫描（所有前置返回 missing）
  - `install_mod_with_dependencies`：`version_id` 改为 `Option<&str>`，新增 `target_dir: Option<&str>`。无 version_id 时用 target_dir 作为下载目录
  - 参数结构体 `CheckModDependenciesParams` / `InstallModWithDepsParams` 同步更新
- **前端改造**（`src/composables/useDependencyCheck.ts` + `src/composables/useResourceDownload.ts`）：
  - `CheckDepsParams.versionId` 改为可选，新增 `modsDir?`；`InstallDepsParams.versionId` 改为可选，新增 `targetDir?`
  - `runDependencyCheck`：Community 场景下从 `modVersion.game_versions` 推断 gameVersion（过滤加载器名称）、从 `modVersion.mod_loaders` 取 modLoader，传 `versionId=undefined + modsDir=undefined` 调用检查
  - `handleDependencyConfirm`：Community 场景下先调 `pickDirectory` 选保存文件夹，install 时传 `targetDir`
  - `handleDownload`：开始即设 `downloading = v.id`，检查依赖期间按钮显示 loading 防止重复点击；`finally` 中仅在未进入前置弹窗时清空 loading
  - `handleDependencyClose`：补充清空 `downloading`，修复取消弹窗后按钮仍 loading 的遗漏
- **弹窗层级修复**（`src/components/community/DependencyConfirmDialog.vue`）：z-index 从 `z-[10000]` 提升为 `z-[10003]`（高于顶部 nav 的 `z-[10002]` 和 ResourceDetail 的 `z-[10000]`），解决前置确认弹窗被 nav 遮挡问题

#### 下载管理页交互修复（按钮遮挡 / 阶段文字截断 / 进度自恢复 / 按钮间距）

- `src/components/common/DownloadPanel.vue`：浮动下载按钮 z-index 从 `z-50` 提升为 `z-[10001]`，高于所有弹窗遮蔽罩（z-10000），避免资源详情/前置确认弹窗打开时下载按钮被遮蔽罩遮挡
- `src/views/downloads/DownloadStatsPanel.vue`：当前阶段文字添加 `truncate` 单行截断 + `Tooltip`（block 模式）完整展示，避免阶段名过长撑高布局挤压下方内容
- `src/composables/useDownloadStream.ts`：`initDownloadStream` 启动时主动调用 `isDownloading` 检查后端下载状态，若仍在下载则 `startDownload(raw.version_name)` 恢复状态并触发 watch 建立 WS 连接，解决刷新后进度不自恢复、下载完成无提示问题；watch 回调参数改名 `isDownloading`→`downloading` 避免遮蔽导入的同名函数
- `src/components/common/Button.vue`：mini 尺寸按钮 icon 与文字间距从 4px 调整为 6px，改善版本列表下载按钮内 icon 与「下载」文字视觉间距

#### 前置 Mod 图片缓存 + 下载页 tab 留存 + 搜索历史 + 按钮对齐

- `src/components/community/resource-detail/DependencyInlineList.vue` / `src/components/community/DependencyItem.vue`：前置列表 `<img>` 替换为 `<CachedImage>` 组件，复用后端 `image_cache_manager` 缓存，避免每次渲染发起远程请求
- `src/composables/useTabPersistence.ts`（新建）：抽取 NavSidebar 的 URL query `tab` 同步逻辑为 composable（onMounted 读取 + watch router.replace 写入）
- `src/components/common/NavSidebar.vue`：改用 `useTabPersistence`，移除内联 onMounted + watch，逻辑等价
- `src/views/downloads/DownloadSidebar.vue`：接入 `useTabPersistence`，实现与工具页相同的 tab 留存（刷新保留 + 切换页面回来保留），不再重置为默认 `vanilla`
- `src/composables/useSearchHistory.ts`（新建）：搜索历史 composable，localStorage 持久化（key `molaunch-search-history`），最近 5 条，去重置顶，复用 `safeCallSync` 范式
- `src/components/community/SearchBar.vue`：搜索框 focus 时展示历史下拉，点击历史项填充并搜索；仅主动搜索（回车/搜索按钮/点击历史）记录历史，防抖自动搜索不记录
- `src/components/common/Button.vue`：mini 尺寸 `line-height` 设为 1（文本行高贴近 font-size，与 14px icon 居中后视觉对齐），icon `margin-right` 最终调整为 8px

#### 资源详情弹窗被顶部导航栏遮挡

- `src/components/community/ResourceDetail.vue`：
  - 弹窗 z-index 从 `z-[9999]` 提升为 `z-[10000]`（与 Modal 一致，仍低于 nav 的 z-[10002]）
  - 外层 flex 容器对齐从 `items-center` 改为 `items-start`，padding 改为 `px-4 pt-14 pb-4`，弹窗加 `mt-2`
  - 弹窗最大高度从 `max-h-[85vh]` 改为 `max-h-[calc(100vh-100px)]`，给顶部 nav（高 48px）留出避让空间
- `src/components/community/DependencyConfirmDialog.vue`：
  - 同步应用相同的避让策略（items-start + pt-14 + mt-2 + max-h calc(100vh-100px)）

#### 前置 Mod 检查不弹窗排查辅助

- `src/composables/useResourceDownload.ts`（原 ResourceDetail.vue）：
  - `handleDownload` 前置检查分支入口加 `console.warn` 日志，输出 `resource_type` 确认是否进入前置检查
  - `runDependencyCheck` 所有跳过分支和完成分支的日志从 `console.debug` 改为 `console.warn`，确保在 devtools 默认级别下可见
  - 新增关键日志：`前置检查：mod=X platform=X 依赖数=N game=X loader=X` 和 `前置检查完成：缺失=N 已满足=N`，便于区分"Mod 本身无依赖"与"检查流程未触发"

#### 前置 Mod 列表内联展示（详情页直接查看）

- `src/components/community/resource-detail/DependencyInlineList.vue`：新增组件，展示前置 mod 的 logo + 名称 + 平台标签，支持 loading 状态
- `src/components/community/resource-detail/VersionGroupCard.vue`：
  - 版本条目下方，仅 Mod 类型且有 dependencies 时展示"查看 N 个前置"按钮
  - 点击按钮展开/收起 DependencyInlineList，首次展开 emit `loadDeps` 事件让父组件懒加载查询前置项目详情
  - 新增 `depsMap` / `depsLoadingSet` props 接收父组件缓存的前置详情
- `src/composables/useResourceDownload.ts` `handleLoadDeps`：对 `version.dependencies` 里每个 project_id 调用 `getProjectDetail` 查询详情，存入 `depsMap`，已缓存不重复查询

### 新增

#### Mod 前置依赖自动检查与一键安装

- **后端依赖解析器**：新增 `src-tauri/src/commands/version/mods/dependency_resolver.rs`
  - `check_mod_dependencies` IPC：BFS 递归解析 Mod 版本的 dependencies，限 3 层深度，visited 集合防环
  - `install_mod_with_dependencies` IPC：构造主 Mod + 勾选前置的下载任务，复用 `DownloadSession::start_grouped` 单 stage 并发下载，进度推送下载管理页
  - 版本筛选：按 game_version + mod_loader 过滤，Release > Beta > Alpha，同优先级选最新 release_date
  - 已安装比对：`is_project_installed` 用 slug + id 双重比对本地 jar 元数据
- **CurseForge 文件级 dependencies**：`src-tauri/src/minecraft/community/curseforge/types.rs` 补充 `CfFileDependency` 结构体，`convert.rs` 提取 relationType=3 的 required 依赖（排除 Fabric API 306612 / Quilt API 634179）
- **Modrinth dependencies 排除项**：`src-tauri/src/minecraft/community/modrinth/convert.rs` 过滤 dependency_type="required"，排除 Fabric API `P7dR8mSH` / Quilt API `qvIfYCYJ`
- **前端类型与 composable**：
  - `src/types/community.ts` 新增 `DepType` / `ResolvedDependency` / `DependencyCheckResult` / `DependencyInstallResult`
  - `src/utils/api/version-mods-manager.ts` 新增 `CHECK_MOD_DEPENDENCIES` / `INSTALL_MOD_WITH_DEPENDENCIES` action
  - `src/composables/useDependencyCheck.ts` 封装 IPC 调用与状态管理（checking / installing / missing / upToDate）
- **前端弹窗组件**：
  - `src/components/community/DependencyItem.vue` 单个前置项卡片（复选框 + logo + 名称 + 平台 + 建议版本 + 递归深度标识）
  - `src/components/community/DependencyConfirmDialog.vue` 确认弹窗（缺失列表 + 全选/反选 + 已满足前置折叠区 + "安装主 Mod + N 个前置"按钮）
- **ResourceDetail 集成**：`src/components/community/ResourceDetail.vue` handleDownload 流程改造
  - ModTab 场景（有 modsDir + versionId + gameVersion + Mod 类型）下先调 `getVersionLoaderInfo` 拿 loaderType 转 flag，调 `check_mod_dependencies`
  - 有缺失 → 弹窗等用户确认 → 调 `install_mod_with_dependencies` 一键下载主 Mod + 勾选前置
  - 无缺失或 Community 场景 → 走原 pickSavePath 流程
  - 检查失败不阻断下载，toast 提示后降级到普通下载
- **useModUpdate 集成**：`src/composables/useModUpdate.ts` installSelected 完成后调 `scanMissingDepsAfterInstall` 兜底
  - 有缺失时 toastInfo 提示缺失数量 + 前 3 个名称，引导用户去资源页安装
  - 检查失败不阻断主流程
- **公共工具抽取**：`src/utils/mod-display.ts` 新增 `loaderToFlag` 函数（加载器名 → ModLoaderFlags 位枚举），`useModUpdate.ts` 改为 import 公共版本，避免重复实现

### 修复

#### Forge 安装伪进度卡住 + 前端伪进度补丁

- **问题1**：安装 Forge 时伪进度卡在 47% 不动。根因：`forge/install.rs` 在 `async fn install_modern` 中直接同步调用 `run_forge_installer`（`pub fn`，内部用 `std::process::Command` 阻塞），阻塞 tokio worker 线程，导致 `ticker` 的 `tokio::spawn` 任务无法被调度
  - **修复**：`src-tauri/src/minecraft/loaders/forge/install.rs` 用 `tokio::task::spawn_blocking` 包裹 `run_forge_installer`，释放 worker 线程给 ticker
- **问题2**：伪进度涨到 93% 假完成（实际安装未完成）。根因：`FORGE_TICKER` 曲线 cap 是 100%，42.5 秒就到顶
  - **修复**：`src-tauri/src/commands/version/install/loader_helpers.rs` 调慢曲线 + cap 改为 95%（永不到 100%，真正完成后由调用方跳 100%）
    - Forge：0→30% @1.5%/s, 30→60% @1.0%/s, 60→85% @0.6%/s, 85→95% @0.3%/s（总 ~120 秒）
    - Fabric：0→40% @3%/s, 40→70% @2%/s, 70→90% @1%/s, 90→95% @0.3%/s（总 ~64 秒）
- **问题3**：后端 ticker 到 95% 卡住后前端进度不动。根因：前端直接显示后端真实进度，无补丁
  - **修复**：新增 `src/utils/downloadProgress.ts` 伪进度补丁工具函数（`applyProgressPatch`），对数曲线趋近 99.9%，永不到 100%
  - `src/composables/useDownloadTaskGroups.ts` 加 `now` 参数 + `patchStartTimes` Map，loading 且 progress >= 0.95 的分组应用补丁
  - `src/components/downloads/TaskGroupCard.vue` 加 `now` ref（200ms 定时更新），分组进度和子阶段进度显示改用 `toFixed(1)` 小数点
  - `src/views/Downloads.vue` 总进度 `percentage` 应用补丁，`watch` 监听真实进度设置 `percentageStartTime`（副作用放 watch 不放 computed）

### 重构

#### 代码清理批次 1：download 模块测试分离 + 注释精简

- **测试分离**：将内联 `#[cfg(test)] mod tests` 提取到同级 `_tests.rs` 文件，源文件保持干净
  - `src-tauri/src/minecraft/download/manager.rs`（8 个测试）→ 新建 `manager_tests.rs`，源文件末尾用 `#[cfg(test)] #[path = "manager_tests.rs"] mod tests;` 引入
  - `src-tauri/src/minecraft/download/rate_limiter.rs`（6 个测试）→ 新建 `rate_limiter_tests.rs`，同样方式引入
- **注释精简**：download 目录 10 个文件精简冗余注释
  - 模块文档超 3 行的精简到 1-3 行（session.rs 30 行→3 行、mod.rs 12 行→3 行、config.rs 5 行→3 行、downloader/mod.rs 6 行→2 行、chunk/mod.rs 9 行→3 行、chunk/merge.rs 4 行→3 行）
  - 删除显而易见的内联注释（如 `// 确保目录存在` 紧跟 `create_dir_all`）
- **验证**：`cargo check` 零警告零错误；`cargo test --lib` 91 个测试全部通过
- **不变**：业务逻辑代码零改动，测试用例数量不减

#### 代码清理批次 2：online 模块测试分离 + 注释精简

- **测试分离**：将内联 `#[cfg(test)] mod tests` 提取到同级 `_tests.rs` 文件，源文件保持干净
  - `src-tauri/src/minecraft/online/http_log.rs`（4 个测试）→ 新建 `http_log_tests.rs`
  - `src-tauri/src/minecraft/online/client.rs`（1 个测试）→ 新建 `client_tests.rs`
  - `src-tauri/src/minecraft/online/storage.rs`（2 个测试）→ 新建 `storage_tests.rs`
  - `src-tauri/src/minecraft/online/auth.rs`（2 个测试）→ 新建 `auth_tests.rs`
  - `src-tauri/src/minecraft/online/bridge.rs`（2 个测试）→ 新建 `bridge_tests.rs`
  - `src-tauri/src/minecraft/online/protocol.rs`（11 个测试）→ 新建 `protocol_tests.rs`
  - `src-tauri/src/minecraft/online/tun.rs`（1 个测试）→ 新建 `tun_tests.rs`
  - `src-tauri/src/minecraft/online/crypto.rs`（4 个测试，含 `use ed25519_dalek::Verifier;`）→ 新建 `crypto_tests.rs`
  - `src-tauri/src/minecraft/online/ecies.rs`（2 个测试，含 `use crate::minecraft::online::crypto::X25519StaticKeyPair;`）→ 新建 `ecies_tests.rs`
  - 源文件末尾统一用 `#[cfg(test)] #[path = "<module>_tests.rs"] mod tests;` 引入
- **注释精简**：online 目录 12 个文件精简冗余模块文档（超过 3 行的精简到 1-5 行）
  - `ecies.rs` 24 行协议说明 → 3 行（详细协议在 `api-server/docs/auth.md`）
  - `auth.rs` 23 行注册/登录流程文档 → 3 行
  - `bridge.rs` 31 行架构图 → 4 行
  - `protocol.rs` 44 行帧格式文档 → 5 行
  - `tun.rs` 24 行平台约束/设计/扩展 → 4 行
  - `storage.rs` 22 行存储内容/加密策略 → 7 行
  - `http_log.rs` 14 行日志格式/设计要点 → 3 行
  - `crypto.rs` 11 行算法列表 → 4 行
  - `client.rs` 10 行 → 6 行（保留接口列表）
  - `mod.rs` 23 行子模块清单 → 4 行（子模块清单与 `pub mod` 重复）
  - `signaling.rs` 删除过时的"阶段一仅声明"说明
  - `client_types.rs` 已简洁（5 行），未改动
- **验证**：`cargo check` 零警告零错误；`cargo test --lib` 91 个测试全部通过（online 模块 29 个测试全通过）
- **不变**：业务逻辑代码零改动，测试用例数量不减

#### 代码清理批次 3：分散文件测试分离 + 注释精简

- **测试分离**：9 个文件提取内联测试到同级 `_tests.rs`（共 35 个测试）
  - `src-tauri/src/certs.rs`（2 测试）→ `certs_tests.rs`
  - `src-tauri/src/minecraft/sources.rs`（6 测试）→ `minecraft/sources_tests.rs`
  - `src-tauri/src/minecraft/language.rs`（2 测试）→ `minecraft/language_tests.rs`
  - `src-tauri/src/minecraft/isolation.rs`（5 测试）→ `minecraft/isolation_tests.rs`
  - `src-tauri/src/minecraft/launch/mod.rs`（2 测试）→ `minecraft/launch/mod_tests.rs`
  - `src-tauri/src/minecraft/launch/skin_resourcepack.rs`（4 测试）→ `minecraft/launch/skin_resourcepack_tests.rs`
  - `src-tauri/src/minecraft/version/state.rs`（4 测试）→ `minecraft/version/state_tests.rs`
  - `src-tauri/src/storage/ini.rs`（3 测试）→ `storage/ini_tests.rs`
  - `src-tauri/src/utils/markdown_table.rs`（7 测试）→ `utils/markdown_table_tests.rs`
  - `minecraft/version/setup/mod.rs` 和 `minecraft/java_selector/mod.rs` 已有独立 `tests.rs`，无需改动
- **注释精简**：11 个文件精简模块文档（超 3 行 → 1-3 行）
- **验证**：`cargo check` 零警告零错误；`cargo test --lib` 91 个测试全部通过
- **不变**：业务逻辑代码零改动，测试用例数量不减

#### 代码清理批次 4-6：全项目注释精简

- **批次 4 launch 模块**（18 文件）：launch 目录剩余文件精简模块文档和冗余内联注释
  - `jvm_args.rs` 18 行设计文档→3 行；`watcher/window_title/mod.rs` 16 行→4 行；`pipeline/mod.rs` 12 行→4 行
  - 删除 process_spawn.rs / java_check.rs / natives.rs 中显而易见的内联注释
- **批次 5 community/version/java/loaders 模块**（34 文件）：4 个目录注释精简
  - community/preload/mod.rs 21 行→7 行；java/download/mod.rs 13 行→4 行
  - 删除多处函数清单、模块结构图、装饰分隔线
- **批次 6 utils/state/storage/sdk/ws/commands/顶层**（14 文件）：剩余目录注释精简
  - 删除 commands 目录所有"注：原 N 个分散的 Tauri 命令已聚合为..."冗余注释
  - 精简 http.rs / res_scheme.rs / resources.rs / error_util.rs 过长文档注释
  - state/config.rs 装饰分隔符 `// ===== xxx =====` → `// xxx`
- **验证**：`cargo check` 零警告零错误；`cargo test --lib` 91 个测试全部通过
- **不变**：业务逻辑代码零改动，测试用例数量不减

### 新增

#### DownloadManager 重构阶段 6：加载器 installer 统一 from_config

- **背景**：6 个加载器 installer（forge/fabric/neoforge/liteloader/fabric_api/shared）都用 `DownloadManager::new(1, 0, 0, source_mode)` 硬编码，用户设置的 `max_threads`/`chunk_count`/`speed_limit` 对加载器安装不生效。阶段 6 统一改为 `DownloadManager::from_config(config)`，让用户设置生效
- **`src-tauri/src/minecraft/download/config.rs`**：新增 `DownloadManagerConfig::from_state_for_meta(state)` 方法（读 `config.download.meta_source`，保持 installer 历史行为；与 `from_state` 的唯一区别是读 meta_source 而非 source）
- **6 个 installer 签名改造**（`source_mode: DownloadSourceMode` → `config: &DownloadManagerConfig`）：
  - `src-tauri/src/minecraft/loaders/shared.rs`（`download_mojang_mappings`）
  - `src-tauri/src/minecraft/loaders/fabric.rs`（`install`）
  - `src-tauri/src/minecraft/loaders/fabric_api.rs`（`install`）
  - `src-tauri/src/minecraft/loaders/liteloader.rs`（`install`）
  - `src-tauri/src/minecraft/loaders/neoforge.rs`（`install` + 透传 config 给 shared）
  - `src-tauri/src/minecraft/loaders/forge/install.rs`（`install` + `install_modern` + 透传 config 给 shared）
- **`src-tauri/src/minecraft/loaders/mod.rs`**（`install_loader` 分发器）：移除 `_max_threads` 参数，`source_mode` → `config: &DownloadManagerConfig`；OptiFine 分支传 `config.source_mode`（optifine 签名不变）
- **3 个调用方改造**（用 `from_state_for_meta` 构造 config）：
  - `src-tauri/src/commands/version/install/loader_helpers.rs`（`install_single_loader`）
  - `src-tauri/src/commands/version/install/fabric_api.rs`（`auto_install_fabric_api`）
  - `src-tauri/src/commands/version/loaders.rs`（`install_fabric_api_for_version`）
- **关键决策**：
  - 保持 `meta_source` 语义：installer 历史用 `meta_source`（元数据源），新增 `from_state_for_meta` 方法保持原行为
  - `install_single_loader` 保留 `_max_threads`/`_source_mode` 参数：stages.rs 调用方未列入改造范围，遵循最小修改原则保留签名（前缀 `_` 标记未用）
  - OptiFine 不改签名：`optifine::install` 保留 `source_mode` 参数，分发器调用时传 `config.source_mode`
- **不变**：`install_legacy` 不改（不接收 source_mode 参数）；stages.rs 调用方不改

#### DownloadManager 重构阶段 5：authlib-injector 接入 DownloadManager

- **背景**：`ensure_authlib_injector_jar` 原用 `http::fetch_bytes` 下载 jar 二进制，与项目下载基础设施割裂。阶段 5 将其接入 DownloadManager，统一走项目下载基础设施（获得限速/URL fallback/进度推送能力），同时提取 `download_manager()` 辅助方法消除 validate.rs 中 manager 构造重复
- **`src-tauri/src/minecraft/auth/authlib/client.rs`**（`ensure_authlib_injector_jar`）：加 `manager: &DownloadManager` 参数
  - 下载方式从 `http::fetch_bytes` 改为 `DownloadManager::download_batch`
  - 下载到 cache 路径后读取文件 + 手动 sha256 校验（DownloadManager 的 expected_hash 用 sha1，与 authlib 的 sha256 不兼容）
  - 校验失败删除损坏文件（避免下次缓存命中读到损坏文件）
  - 不传 expected_size（sha256 校验间接保证完整性）
- **`src-tauri/src/minecraft/launch/pipeline/validate.rs`**：
  - 提取 `download_manager(&self)` 私有辅助方法（消除 `validate_and_fix_files` + `build_arguments` 重复构造 11 行 manager 代码）
  - `validate_and_fix_files` 用 `self.download_manager()` 替代内联构造（行为不变，纯重构）
  - `build_arguments` 构造 manager 传入 `ensure_authlib_injector_jar`（用户设置的限速/分片/线程数现在对 authlib-injector.jar 下载也生效）
- **`src-tauri/src/minecraft/download/mod.rs`**：re-export `DownloadManager`（与 `DownloadSession` re-export 风格一致，方便外部 `use crate::minecraft::download::DownloadManager`）
- **关键决策**：
  - 接入 DownloadManager 而非保留 http::fetch_bytes：为了架构一致性，所有下载统一走 DownloadManager
  - sha256 手动校验：DownloadManager 的 expected_hash 用 sha1，与 authlib 的 sha256 不兼容，无法复用
  - urls 只用 `meta.download_url`：fetch_authlib_injector_meta 内部已有 primary + mirror 两个 meta URL，无需再做镜像替换
  - 提取 `download_manager()` 辅助方法：避免在两个方法中复制 manager 构造代码
- **不变**：`crate::http::fetch_bytes` 保留（其他模块可能仍在用）；sha256 校验逻辑不变（仍手动实现）；缓存命中路径不变（cache::exists 检查）

#### DownloadManager 重构阶段 4：Mod 更新原子化

- **背景**：阶段 3 完成 DownloadSession 会话编排层后，Mod 更新流程仍用 3 个 IPC（`getVersionModsDir` + `downloadResourceToPath` + `deleteMod`）拼接，前端负责"下载→删旧"编排，存在两个问题：(1) 下载失败时旧文件可能已被部分覆盖（虽然下载到新文件名，但前端逻辑分散);(2) 前端编排逻辑与后端下载基础设施割裂，无法享受 DownloadSession 的统一进度推送。阶段 4 将"下载新版本 + 删旧版本"封装为后端原子操作 `update_mod`
- **`src-tauri/src/commands/version/mods/update.rs`**（新增）：`update_mod` 命令，封装"下载新版本 → 删旧版本"为原子操作
  - 使用 `DownloadSession::start_grouped` 启动会话（分组"Mod 更新"，2 stages：下载新版本 80% / 替换旧版本 20%）
  - 下载新版本到 mods 目录（用 `sources::cdn_urls` 生成多 URL fallback）
  - **原子性保证**：下载失败时 `mark_failed` 并返回错误，**不删旧文件**；下载成功才删旧文件（仅当 `old_file_name != new_file_name`）
  - 进度通过 DownloadSession 统一推送，前端下载管理页可见
- **`src-tauri/src/commands/version/mods/mod.rs`**：声明 `pub mod update`，更新模块结构说明
- **`src-tauri/src/utils/version_mods_manager.rs`**：新增 `UpdateModParams` 参数结构 + 注册 `update_mod` action 到 DISPATCHER（第 11 个 action）
- **`src-tauri/src/lib.rs`**：更新注释 "10 个 action" → "11 个 action"
- **`src/utils/api/version-mods-manager.ts`**：`VERSION_MODS_ACTIONS` 常量补 `UPDATE_MOD`
- **`src/utils/api/personalization.ts`**：新增 `updateMod` 函数（调用 `versionModsManager('update_mod', ...)`）
- **`src/composables/useModUpdate.ts`**（`installSelected`）：从 3 IPC 降为 1 IPC
  - 移除 `getVersionModsDir` + `downloadResourceToPath` + `deleteMod` 三个调用
  - 改为单一 `updateMod` 调用，后端负责获取 mods 目录 + 下载 + 删旧
  - 移除前端 `version.file_name !== mod.enabled_name` 检查（后端 `old_file_name != new_file_name` 已覆盖所有情况，且原检查会导致 .disabled 旧文件残留不清理）
- **关键决策**：
  - 原子性在后端保证而非前端：前端 3 IPC 拼接在并发/异常场景下可能出现"下载成功但删旧失败"导致状态不一致；后端单 IPC 内部可控
  - 旧文件清理逻辑简化：原前端检查 `version.file_name !== oldFileName && version.file_name !== mod.enabled_name`，第二个条件导致"新文件名等于启用名时不删旧 .disabled 文件"的 bug（旧 .disabled 文件残留）；后端只检查 `old_file_name != new_file_name`，行为更正确
  - `expected_size` 透传：前端传 `version.size`，后端用于下载校验
- **不变**：`update_mod` 通过 `version_mods_manager` IPC 入口分发（该命令已在 lib.rs 注册），无需新增 invoke_handler 注册；`useModUpdate.ts` 的版本列表查询/过滤/选中逻辑不变

#### DownloadManager 重构阶段 3：整合包 + 资源 + 外部下载接入 DownloadSession

- **背景**：阶段 2.5 完成后，5 个调用点仍硬编码 `DownloadManager::new(4, chunk_count, 0, Smart)` 或 `new(max_threads, chunk_count, 0, Smart)`，用户设置的 `speed_limit` / `source_mode` 对整合包 / 资源 / 外部下载全部不生效；同时 5 处 callback 闭包复制粘贴（`sync_stage_from_progress + broadcast_current`），5 处 `reset_stages + flag 重置` 重复。阶段 3 引入 `DownloadSession` 会话编排层，统一消灭这些重复
- **`src-tauri/src/minecraft/download/session.rs`**（新增）：`DownloadSession` 会话编排层，封装 `reset_stages + flag 重置 + manager 构造 + callback 工厂`
  - `start_grouped(state, group, stages)`：顶层入口用（资源 / 外部下载），自动 reset_stages + 重置 flag + 构造 manager
  - `attach(state)`：子流程用（整合包下载 / mods 批量下载），仅构造 manager（stages / flag 已由父函数处理）
  - `make_progress_callback(state, stage_index)`：统一 callback 工厂（消除 5 处闭包复制）
  - `manager()` / `mark_complete` / `mark_failed`
- **`src-tauri/src/minecraft/download/config.rs`**（`DownloadManagerConfig::from_state`）：`chunk_count` 加 `max(1)` 保持与历史调用方一致（防御性，避免 0 值传入分片逻辑）
- **`src-tauri/src/minecraft/download/mod.rs`**：注册 `session` 子模块并 `pub use DownloadSession`
- **`src-tauri/src/commands/community/install/resource.rs`**（`download_resource` / `download_resource_to_path`）：用 `DownloadSession::start_grouped` 替代手工 `reset_stages + flag 重置 + DownloadManager::new + callback 闭包`，删除 2 处闭包复制 + 2 处硬编码 `new(4, chunk_count, 0, Smart)`
- **`src-tauri/src/commands/tools/download.rs`**（`download_file`）：用 `DownloadSession::start_grouped` 替代手工初始化，删除 1 处闭包复制 + 1 处硬编码 `new(4, chunk_count, 0, Smart)`
- **`src-tauri/src/commands/community/install/modpack_stages.rs`**（`download_modpack_archive`）：用 `DownloadSession::attach` 替代手工 `DownloadManager::new + callback 闭包`（stages / flag 已由 `install_modpack` 处理），删除 1 处闭包复制 + 1 处硬编码 `new(4, chunk_count, 0, Smart)`
- **`src-tauri/src/commands/community/install/concurrent.rs`**（`download_files_concurrent`）：用 `DownloadSession::attach` 替代手工 `DownloadManager::new + callback 闭包`，删除 1 处闭包复制 + 1 处硬编码 `new(max_threads, chunk_count, 0, Smart)`；**移除 `max_threads` 参数** —— `DownloadSession::attach` 内部已从 config 读取，避免双重数据源
- **`src-tauri/src/commands/community/install/curseforge.rs`**（`install_cf_mods`）：移除 `max_threads` 参数（透传给 `download_files_concurrent` 的，已无用）
- **`src-tauri/src/commands/community/install/modrinth.rs`**（`install_mr_files`）：移除 `max_threads` 参数（同上）
- **`src-tauri/src/commands/community/install/modpack.rs`**（`install_modpack` / `install_modpack_local`）：移除 2 处 `max_threads` 提取 + 4 处传参
- **关键决策**：
  - `DownloadSession` 提供两种入口：`start_grouped`（顶层，完整初始化）+ `attach`（子流程，仅 manager）—— 后者用于 `install_modpack` 已 reset_stages + flag 重置的场景，避免覆盖父会话状态
  - `max_threads` 参数链彻底移除（4 个文件）—— 之前 `install_modpack` 从 config 读 `max_threads` → 透传给 `install_cf_mods`/`install_mr_files` → 透传给 `download_files_concurrent` → 构造 `DownloadManager::new(max_threads, ...)`；现在 `DownloadSession::attach` 内部从 config 直接读，数据流单一化
  - 用户设置的 `speed_limit` / `source_mode` 现在对**所有下载场景生效**（整合包 / 资源 / 外部 / mods 批量），之前这 5 处硬编码 `speed_limit=0` / `source_mode=Smart`，用户设置不生效
- **不变**：`install_modpack` / `install_modpack_local` 顶层的 `reset_stages + flag 重置` 保留（这些是父会话职责，子流程 `attach` 不应处理）；`DownloadManager` / `download_batch` / chunk / stream 实现完全不动

#### DownloadManager 重构阶段 2.5：补 fix_version_files 接入

- **背景**：阶段 2 改造了 `download_version_full`，但 `fix_version_files`（启动时文件补全 + 手动补全命令）仍用旧签名（4 个独立参数），且 `validate.rs` 中硬编码 `8/4/0/Smart`，用户设置的限速/分片/线程数对启动时文件补全不生效。阶段 2.5 将 fix_version_files 接入统一构造方式
- **`src-tauri/src/minecraft/launch/pipeline/types.rs`**（`LaunchConfig`）：新增 `max_threads`/`chunk_count`/`speed_limit` 三个字段（`#[serde(default)]`，默认值 8/4/0 与历史硬编码一致），用于启动时文件补全构造 DownloadManager
- **`src-tauri/src/commands/version/launch/build_config.rs`**（`build_launch_config`）：从 `config.download` 填充三个新字段
- **`src-tauri/src/minecraft/download/fix.rs`**（`fix_version_files`）：签名收敛 —— 删除 `max_threads`/`chunk_count`/`speed_limit`/`source_mode` 4 个参数，改为接收 `&DownloadManager`（与 `download_client_jar`/`download_libraries`/`download_assets` 签名一致），调用方负责构造 manager
- **`src-tauri/src/commands/version/manage.rs`**（`fix_version_files` 命令）：用 `DownloadManager::from_state(state).await` 构造 manager，替代手工提取 4 个字段
- **`src-tauri/src/minecraft/launch/pipeline/validate.rs`**（`validate_and_fix_files`）：用 `DownloadManager::from_config` + LaunchConfig 中的下载参数构造 manager，替代之前硬编码 `8/4/0/Smart`，用户设置的限速/分片/线程数现在对启动时文件补全也生效；同时使用 `mirror_url`（之前硬编码 None）
- **不变**：`utils/version_list_manager.rs` 调用的是 command 层 `manage::fix_version_files`（签名未变），无需修改

#### DownloadManager 重构阶段 2：MC 本体流程接入

- **背景**：阶段 1 完成基础设施（`DownloadManagerConfig` + `from_state` 工厂），阶段 2 改造 MC 本体下载流程，解决三个核心问题：
  1. `download_version_full` 签名臃肿（11 个参数，其中 6 个用于构造 DownloadManager），调用方重复提取 config
  2. `download_version` 命令独立调用时只 `clear` stages 不重建，`progress_callback` 中 `ds.stages[idx]` 越界（设计文档问题 #10）
  3. `download_version` 命令用手工 `accumulated_bytes`/`accumulated_total`/`speed_window` 累加进度，与 `install_merged` 的 `sync_stage_from_progress` 算法不一致（设计文档问题 #4）
- **`src-tauri/src/minecraft/download/full_download.rs`**：`download_version_full` 签名收敛 —— 删除 `max_threads`/`chunk_count`/`speed_limit`/`source_mode`/`cancel_flag`/`pause_flag` 6 个参数，新增 `state: &AppState`；内部用 `DownloadManager::from_state(state).await.with_cancel_flag(...).with_pause_flag(...)` 一行构造，参数统一从 config 读取
- **`src-tauri/src/commands/version/download.rs`**（`download_version` 命令）：
  - 修复 stages bug：用 `reset_stages` 注册 5 个 MC 本体 stages（版本清单/版本信息/客户端/库文件/资源文件），替代之前只 `clear` 不重建
  - 统一进度同步：删除 `accumulated_bytes`/`accumulated_total`/`speed_window` 三个手工累加变量，改用 `sync_stage_from_progress`（与 `install_merged` 行为一致）
  - 统一阶段切换：`stage_callback` 改用 `set_current_stage`（自动标记前一阶段 Finished）
  - 完成标记：用 `mark_complete()` 替代手工 `is_active=false` + 循环标记 stages
- **`src-tauri/src/commands/version/install/mod.rs`**（`install_merged`）：适配 `download_version_full` 新签名，删除 6 个传参，改为传 `state`；保留 `max_threads` 提取（`install_all_loaders` 仍需要）
- **不变**：`fix_version_files` 及 launch pipeline 调用点暂不改动（涉及 `LaunchPipeline` 无 `AppState` 引用，留到后续阶段）；`download_client_jar`/`download_libraries`/`download_assets` 签名不变

#### DownloadManager 重构阶段 1：基础设施

- **背景**：DownloadManager 实例化在 13 处调用点参数不一致（仅 MC 本体流程读 config，其余硬编码 4/0/Smart），且 3 处重复 `state.config.lock() → extract → drop` 套件。阶段 1 新增统一工厂方法，为后续阶段收敛调用点做准备
- **`src-tauri/src/minecraft/download/config.rs`**（新增）：`DownloadManagerConfig` struct + `from_state()` —— 从 AppConfig 提取 max_threads / chunk_count / speed_limit / source_mode 四字段，统一收敛重复的 lock/extract 逻辑
- **`src-tauri/src/minecraft/download/manager.rs`**：`DownloadManager` 新增 `from_config(&DownloadManagerConfig)` + `from_state(&AppState)` 工厂方法；`reorder_urls` 改用 `sources::is_mirror_url()` 替代硬编码 `contains("bmclapi"/"mirror")`，修复 mocdn URL 被误分类为官方源的 bug
- **`src-tauri/src/minecraft/sources.rs`**：新增 `is_mirror_url(url)` 公共函数，识别 BMCLAPI / mocdn.net / mcimirror.top 三类镜像域名
- **`src-tauri/src/minecraft/download/mod.rs`**：注册 `config` 子模块
- **不变**：现有 13 处 `DownloadManager::new` 调用点暂不改动（阶段 2+ 逐步迁移），对外 API 完全稳定

#### 移除 mod.mcimirror.top CDN 文件下载镜像

- **背景**：`mod.mcimirror.top` 作为 CurseForge/Modrinth CDN 文件下载的兜底镜像，实际请求会 302 重定向到官方 CDN，无加速效果且增加额外跳转延迟，已无存在意义
- **`src-tauri/src/minecraft/sources.rs`**：移除 `CDN_MIRROR` 常量；`apply_cdn_mirrors` 移除所有 mcimirror 兜底分支，仅保留 mocdn.net 镜像；`media.forgecdn.net` 路径因 mocdn 不支持，改为直接走官方源；更新 `cdn_urls` 文档注释
- **`src-tauri/src/commands/community/install/helpers.rs`**：更新 `extract_mr_project_id` / `construct_cf_edge_url` 注释中的镜像源示例（mcimirror → mocdn）
- **保留**：CurseForge/Modrinth 的 **API 镜像**（`CF_MIRROR_BASE` / `MR_MIRROR_BASE`，用于免 API Key 访问）不受影响，仍有实际价值

#### 分片下载断点续传 + 合并前大小校验

- **背景**：分片下载任一 chunk 失败后，`.part` 文件被整体清理，重试必须从 0 重下，慢速网络下体验差；同时若服务端提前断流（`bytes_stream` 提前 `Ok(None)`），`download_chunk` 仍返回 `Ok(downloaded)`，导致部分下载被误合并为损坏的目标文件，启动时才暴露问题
- **`src-tauri/src/minecraft/download/chunk/download.rs`**：入口检测 `.part` 文件已下载字节数实现断点续传 —— `existing == expected` 跳过下载直接返回；`existing > expected` 视为损坏删除重下；`0 < existing < expected` 调整 Range 起点 `actual_start = start + resume_from` 并以 `append` 模式打开文件续传；新增 `expected_chunk_bytes` 与 `chunk_byte_limit`（2 倍冗余）防止被劫持镜像源返回超量数据；失败时 `.part` 文件保留用于重试续传
- **`src-tauri/src/minecraft/download/chunk/mod.rs`**：`all_ok=false` 路径与合并失败路径均不再清理 `.part` 文件，保留用于重试续传；回滚 `file_progress.downloaded_bytes` 避免重试时进度偏高/超过 total
- **`src-tauri/src/minecraft/download/chunk/merge.rs`**：`merge_chunks` 新增 `file_size` 参数，合并前逐个校验每个 `.part` 文件大小与期望值匹配（前 `chunk_count-1` 片为 `file_size / chunk_count`，最后一片为余数），不匹配则返回 `InvalidData` 错误拒绝合并，避免部分下载被误合并为损坏文件

#### 下载进度 WebSocket 推送（替代前端 300ms 轮询）

- **背景**：前端 `useDownloadPolling.ts` 每 300ms 调用 `get_download_progress` IPC 轮询下载进度，devtools 网络面板刷屏且看不到响应内容。改为 WebSocket 服务器推送，devtools 面板干净，可在 WS Frames 面板查看进度消息流
- **`src-tauri/Cargo.toml`**：新增 `tokio-tungstenite = "0.21"` 依赖，tokio 添加 `"net"` + `"io-util"` feature
- **`src-tauri/src/ws/mod.rs`**（新增）：WS 服务器模块——监听 `127.0.0.1:0` 随机端口；每个连接订阅 `progress_tx` broadcast channel；200ms 节流推送（高频进度更新自动合并，避免 UI 刷屏）；支持 Close 帧处理和 lagged 容错
- **`src-tauri/src/state/app.rs`**：`AppState` 新增 `progress_tx: Arc<broadcast::Sender<serde_json::Value>>`（容量 16）和 `ws_port: Arc<OnceLock<u16>>`
- **`src-tauri/src/commands/version/download.rs`**：`progress_callback` / `stage_callback` / 完成处三路推送——`app.emit("download-progress")` + `progress_tx.send(snapshot)`；`build_snapshot` 加 `is_paused: bool` 参数和 `version_name` 字段
- **`src-tauri/src/commands/version/progress.rs`**：`cancel_download` / `pause_download` / `resume_download` 改变 flag 后通过 `broadcast_current` 广播当前状态 snapshot，前端 WS 即时感知控制信号变化
- **`src-tauri/src/utils/system_manager.rs`**：注册 `get_ws_port` action（返回 WS 端口，0 表示未启动）
- **`src-tauri/src/lib.rs`**：`.setup()` 钩子中 `tauri::async_runtime::spawn(ws::start_server)` 启动 WS 服务器
- **`src/utils/api/system-manager.ts`**：`SYSTEM_ACTIONS` 新增 `GET_WS_PORT`
- **`src/utils/api/system.ts`**：新增 `getWsPort()` 函数
- **`src/composables/useDownloadStream.ts`**（新增，替代 `useDownloadPolling.ts`）：监听 `versionStore.downloading` 建立/关闭 WS 连接；`onmessage` 解析 JSON 更新 store；断线 3 秒自动重连；WS 服务器未启动时 500ms 重试获取端口
- **`src/App.vue`** + **`src/views/Downloads.vue`**：`initDownloadPolling()` → `initDownloadStream()`
- **删除**：`src/composables/useDownloadPolling.ts`（已被 WS 方案完全替代）

#### WebSocket 推送修复（连接成功但无消息 + CSP 阻止 + 初始状态丢失）

- **问题**：前端反馈 WS 连接成功但收不到任何消息，且 `version_progress_manager` IPC 被请求两次（`isDownloading` + `getDownloadProgress` 初始状态恢复）
- **`src-tauri/tauri.conf.json`**：CSP `connect-src` 补充 `ws://127.0.0.1:*` + `http://127.0.0.1:*`（`useHttpsScheme: true` 下 https 页面连接 ws 属于混合内容，需显式放行）
- **`src-tauri/src/ws/mod.rs`**：`handle_connection` 连接建立后立即推送一次当前 `download_state` snapshot —— 解决 broadcast 订阅时序问题（subscribe 之前的消息全部丢失，前端进入下载页时能立即收到当前进度，无需等待下一次 callback）；添加关键诊断日志（连接建立 / 初始 snapshot 推送 / 首条广播消息 / 连接关闭）
- **`src/composables/useDownloadStream.ts`**：`initDownloadStream` 添加 `watchRegistered` guard，防止 App.vue + Downloads.vue 重复调用注册多个 watch
- **`src/views/Downloads.vue`**：移除 `initDownloadStream()` 调用（App.vue 已全局初始化），仅保留初始状态恢复逻辑

#### 取消下载时日志刷屏修复

- **问题**：用户取消整合包安装时，`download_files_concurrent` 中所有进行中的文件（如 111 个 mod）都会失败并逐个打印 3 条 INFO 日志（下载失败 + URL + 失败详情），共 333 条日志刷屏
- **`src-tauri/src/commands/community/install/concurrent.rs`**：收集失败前检查 `download_cancel_flag`，若为取消导致的失败：跳过逐个日志，只打印一条总结 `[Community] 下载已取消，N 个文件未完成`，返回简洁错误 `下载已取消`；非取消场景保留原有详细日志便于排障

#### 整合包/资源下载进度 WS 推送缺失修复

- **问题**：MC 本体下载的 `progress_callback` 有 `progress_tx.send()` 广播，但整合包安装（`download_files_concurrent` / `download_modpack_archive`）和资源下载（`download_resource` / `download_resource_to_path`）的 4 个 `progress_callback` 只更新了 `download_state`，**没有广播到 WS**，导致前端 WS 只收到连接时的初始 snapshot，后续进度无推送
- **`src-tauri/src/commands/version/download.rs`**：新增 `pub fn broadcast_current(state: &AppState)` 公共函数 —— 读取 `download_state` + `pause_flag` 构造 snapshot 并 `progress_tx.send()`，供各下载路径的 callback 统一调用
- **`src-tauri/src/commands/version/progress.rs`**：私有 `broadcast_current` 改为委托 `super::download::broadcast_current`，避免重复实现；移除未使用的 `build_snapshot` import
- **`src-tauri/src/commands/community/install/concurrent.rs`**：`download_files_concurrent` 的 `progress_callback` 添加 `broadcast_current` 调用
- **`src-tauri/src/commands/community/install/modpack_stages.rs`**：`download_modpack_archive` 的 `stage0_callback` 添加 `broadcast_current` 调用
- **`src-tauri/src/commands/community/install/resource.rs`**：`download_resource` + `download_resource_to_path` 两处 `progress_callback` 添加 `broadcast_current` 调用

#### 取消下载时弹错误窗修复

- **问题**：用户主动取消下载时，后端返回的错误（如 `下载整合包失败: 下载已取消`）会被前端的 catch 块当作真实失败处理，弹出 `整合包安装失败` / `下载失败` 错误窗，用户体验差（取消是用户主动行为，不应弹错误窗）
- **`src/utils/async.ts`**：新增 `isCancelledError(error: unknown): boolean` 工具函数 —— 检测错误消息是否包含 `下载已取消` 或 `下载被取消`，统一识别取消错误
- **`src/composables/useModpackInstall.ts`** + **`src/composables/useVersionInstallActions.ts`**（install + download 两处）+ **`src/composables/useModUpdate.ts`** + **`src/composables/useExternalDownload.ts`** + **`src/composables/useDragDrop.ts`**（modpack + merged 两处）+ **`src/components/community/ResourceDetail.vue`**：catch 分支新增 `isCancelledError` 检查 —— 取消错误仅 `toastInfo('下载已取消')` + `versionStore.finishDownload()` 退出下载页，不弹错误窗；真实失败保留原有 `showModal` 行为

#### 整合包下载 total_bytes=0 修复（前端显示「计算中...」+「0/1 文件」）

- **问题**：整合包原始包 `DownloadTask.expected_size=0`（依赖运行时探测），`download_batch` 启动时 `total_bytes = sum(expected_size) = 0`。CurseForge CDN 不支持 Range 请求（GET + Range 返回 404），`download_single` 回退单流路径，`stream.rs` 拿到 `content_length` 后只用于返回值和 byte_limit 校验，**不回填 `progress.total_bytes`**，导致：
  - stage 0 的 `bytes_total=0`，`sync_stage_from_progress` 回退按文件数算进度，前端显示「0/1 文件」
  - `sync_stage_from_progress` 累加 `global_bytes_total=0`，全局总大小显示「计算中...」
  - global 加权进度卡 0%
  - 与 chunked 路径行为不一致（`chunk/mod.rs` 第 73-76 行在 `probe_file_size` 后会回填 `total_bytes`）
  - 与 MC 本体下载行为不一致（`expected_size` 来自 version JSON >0，初始 `total_bytes` 就正确）
- **根因分析**：WS 推送与 IPC 轮询读取的数据源完全相同（都是 `state.download_state`），snapshot 字段构造无差异。之前轮询「正常」是错觉——之前测试的整合包源支持 Range（走 chunked 路径回填了 total_bytes），现在 CurseForge 源不支持 Range（走 stream 路径不回填），与传输层无关
- **`src-tauri/src/minecraft/download/downloader/stream.rs`**：
  - 拿到 `response.content_length()` 后立即 `saturating_add` 到 `progress.total_bytes`，与 chunked 路径的回填逻辑对齐，统一两条路径
  - `rollback_progress` 闭包改为 `move` 捕获 `total_size`，失败时回滚 `total_bytes`，避免 `download_single` 的 3 次重试导致 total 翻倍
- **可复用性**：所有走单流路径的下载（CF 整合包归档、CF mod 文件回退单流、资源下载回退单流）都受益，无需在调用方手动探测

#### 整合包解析阶段伪进度动画（消除卡顿无反馈）

- **问题**：解析整合包阶段（打开 zip + 检测格式 + 解析 manifest）是本地同步操作，直接 `set_stage_status(Loading, 0.0)` → 同步操作 → `set_stage_status(Finished, 1.0)` 跳过，无中间进度反馈。大整合包（如 SkyFactory 4）解析需 3s，用户看到卡顿感
- **`src-tauri/src/commands/version/install/loader_helpers.rs`**：重构 `start_progress_ticker` 函数 ——
  - 从对数曲线改为**分段线性曲线**，接受 `segments: &'static [(f64, f64)]` 参数（`[(cap, speed_per_sec), ...]`）
  - 新增 `compute_linear_progress` 分段线性计算函数 + `start_parse_ticker` 整合包解析专用入口
  - Forge/NeoForge 曲线：0→50% @4%/s, 50→80% @3%/s, 80→100% @1%/s（总计 42.5s）
  - Fabric 曲线：0→50% @6%/s, 50→80% @4%/s, 80→100% @2%/s（总计 25.8s，比 Forge 快）
  - 整合包解析曲线：0→90% @5%/s（18s 到顶，解析完成 stop 跳 100%）
  - 每次更新 progress 后调用 `broadcast_current` 广播到 WS
- **`src-tauri/src/commands/version/install/mod.rs`**：`loader_helpers` 模块从 `mod` 改为 `pub(crate) mod`，供 `commands/community/install/modpack.rs` 跨模块调用
- **`src-tauri/src/commands/community/install/modpack.rs`**：两处解析阶段（在线安装 stage 1 + 拖拽安装 stage 0）调用 `start_parse_ticker(state, idx)` —— 分段线性 0→90% @5%/s 缓慢上涨，解析完成后 `store(true)` stop 并跳 100%。错误返回前也 stop ticker 避免泄漏

#### 关于「下载 MOD 阶段总大小持续增长」的说明

- **现象**：stage 2「下载 MOD」的 `bytes_total` 随下载进度缓慢增长（如 296MB → 325MB）
- **原因**：CF API `/mods/files` 批量查询接口对部分文件不返回 `fileLength` 字段（`CfFileEntry.file_length` 带 `#[serde(default)]`，缺失时为 0），`download_batch` 初始化 `total_bytes = sum(expected_size)` 偏小。走单流路径时 `stream.rs` 通过 `content_length` 回填真实大小，`total_bytes` 持续累加
- **结论**：这是 CF API 数据不完整导致的**正确行为**，非 bug。`stream.rs` 的回填是必要的修正——否则 `total_bytes` 会一直为 0（回到修复前的「计算中...」问题）
- **`src/views/downloads/DownloadStatsPanel.vue`**：总大小变化时短暂高亮（`text-primary-500` 600ms）+ Tooltip 提示「部分文件大小由下载时探测，总大小可能会逐步修正」，让用户理解总大小修正是正常行为

#### WebSocket 鉴权（防止本机其他进程窃听下载进度）

- **背景**：WS 服务器监听 `127.0.0.1:0` 随机端口，虽然仅本机可访问，但本机其他进程（如恶意软件、抓包工具）仍可直接连接端口窃取下载进度数据。新增 token 鉴权机制，确保只有持 token 的客户端能接收进度推送
- **`src-tauri/src/state/app.rs`**：`AppState` 新增 `ws_token: Arc<OnceLock<String>>` 字段
- **`src-tauri/src/ws/`**（模块化重构）：
  - `mod.rs`：模块声明 + re-export `start_server` + `broadcast_progress`（精简注释）
  - `auth.rs`（新增）：`generate_ws_token`（OsRng 32 字节 → 64 位十六进制）+ `verify_auth_message` + `auth_ok_message` + `AUTH_TIMEOUT_SECS` 常量
  - `server.rs`（新增）：`start_server` 启动服务器并生成 token 写入 `AppState`；`handle_connection` 鉴权阶段（3 秒超时校验首条消息）→ 鉴权通过后推送初始 snapshot + 200ms 节流推送后续进度
- **`src-tauri/src/utils/system_manager.rs`**：`get_ws_port` 返回值从 `u16` 改为 `{port: u16, token: string}` 对象
- **`src/utils/api/system.ts`**：`getWsPort()` 返回类型从 `number` 改为 `WsPortInfo { port, token }`，新增 `WsPortInfo` 接口
- **`src/composables/useDownloadStream.ts`**：
  - 新增 `authenticated` 标志，鉴权前不处理进度消息
  - `onopen` 后立即发送 `{"type":"auth","token":"<token>"}` 鉴权消息
  - `onmessage` 收到 `auth_ok` 后才标记 `authenticated = true`，后续消息才交给 `handleProgress`
  - `closeStream` / `onclose` 重置 `authenticated`

### 新增

#### TLS 证书管理（内置证书库 + 自定义证书 + 信任源模式）

- **背景**：reqwest 切换到 `rustls-tls` 后端，支持内置证书库（webpki-roots）/ 系统证书库 / 自定义证书三种信任源组合，防止抓包和中间人攻击
- **`src-tauri/Cargo.toml`**：reqwest 添加 `default-features = false` + `rustls-tls` 特性，新增 `rustls-native-certs = "0.6"` 依赖
- **`src-tauri/src/certs.rs`**（新增）：证书管理模块——`cert_dir()` 定位 `%APPDATA%/.Molaunch/certs/` 目录；`list_custom_certs()` / `add_custom_cert()` / `remove_custom_cert()` 增删查自定义 PEM；`load_custom_root_certificates()` / `load_system_root_certificates()` 加载信任根；`parse_pem_meta()` 简易解析 Subject CN 和过期时间
- **`src-tauri/src/state/config.rs`**：新增 `TlsConfig` 结构体（`trust_mode` 字段），默认 `"builtin"`
- **`src-tauri/src/http.rs`**：`build_client` 按信任源模式组合加载根证书，`IgnoreTls` 开启时 `danger_accept_invalid_certs(true)`
- **`src-tauri/src/commands/system/developer.rs`**：新增 `KEY_IGNORE_TLS` 常量和 `is_ignore_tls()` 函数（三重校验：DeveloperUnlocked + DeveloperMode + IgnoreTls）
- **`src-tauri/src/commands/system/apply_config/`**：`ConfigPatch` 新增 `tls_trust_mode` / `ignore_tls` 字段；`ConfigSnapshot` 新增 `TlsSnapshot`；`apply_tls()` + `secure::apply_ignore_tls()` 分别处理 INI 和注册表持久化；trust_mode 变更后热重建 HTTP 客户端
- **`src-tauri/src/utils/system_manager.rs`**：注册 `list_custom_certs` / `add_custom_cert` / `remove_custom_cert` 三个 IPC action
- **`src/utils/api/config.ts`**：`ConfigSnapshot` + `ConfigPatch` 补充 `tlsTrustMode` / `ignoreTls` 字段
- **`src/utils/api/system-manager.ts`**：`SYSTEM_ACTIONS` 补充 `LIST_CUSTOM_CERTS` / `ADD_CUSTOM_CERT` / `REMOVE_CUSTOM_CERT`
- **`src/utils/api/developer.ts`**：新增 `CustomCertInfo` 接口 + `listCustomCerts()` / `addCustomCert()` / `removeCustomCert()` 函数

### 重构

#### 开发者设置页面拆分为子页签（SubTabBar + 5 个子组件）

- **背景**：原 `SettingsDeveloper.vue`（225 行）将实验性功能、日志、缓存、存储、系统信息堆叠在单页，内容过多且无分类。重构后接入 `SubTabBar` 组件（与「更多」页一致），按职责拆分为 5 个子页签
- **`src/views/settings/SettingsDeveloper.vue`**：重构为薄编排层（~90 行），仅管理 SubTabBar 导航 + 共享数据加载（storageDirs / systemInfo），props 下发给子页签
- **`src/views/settings/developer/ExperimentalTab.vue`**（新增）：Modrinth CDN 直连开关
- **`src/views/settings/developer/CertsTab.vue`**（新增）：TLS 信任源模式选择 + 忽略 TLS 开关 + 自定义证书表格（添加 / 删除 .pem 文件）
- **`src/views/settings/developer/LogsTab.vue`**（新增）：HTTP 请求日志 + 应用日志（均为自包含 CollapsibleCard）
- **`src/views/settings/developer/StorageTab.vue`**（新增）：缓存目录 + 存储信息（打开 / 定位按钮）
- **`src/views/settings/developer/SystemTab.vue`**（新增）：系统信息（版本 / OS / 架构 / 内存）

### 修复

#### 所有弹窗不再遮蔽顶部 nav

- **`src/components/layout/TopNavLayout.vue`**：header 添加 `relative z-[10002]`，高于所有弹窗层级（Modal z-10000 / Toast z-10001 / Tooltip z-9999），确保顶部 nav 始终可见
- **`src/components/common/Tooltip.vue`**：边界修正 top 下限从 8px 改为 56px（nav 48px + 8px 间距），避免 tooltip 被夹紧到 nav 区域内

#### 联机房间生命周期治理（僵尸房间 / 参与者泄漏 / 房主在线检测）

- **问题 1：加入方 30s 握手超时退出未通知服务端**
  - **现象**：加入方超时后仅清本地状态，服务端参与者记录仍为 `joined`，大厅显示 `2/4`（实际 `1/4`）
  - **`src/components/online/RoomManager.vue`**：catch 块在 `resetRoomState()` 前先调 `store.guestLeaveRoom()`（`DELETE /rooms/{code}/participants/me`），API 失败不阻塞本地退出
- **问题 2：窗口关闭时无联机清理**
  - **现象**：房主关窗 → 房间僵尸；加入方关窗 → 参与者记录泄漏
  - **`src/components/layout/TopNavLayout.vue`**：`handleClose` 根据 `roomState.role` 分流——房主调 `hostCloseRoom()`，加入方调 `guestLeaveRoom()`，3s 超时保护不卡关窗
- **问题 3：房主关闭程序后房间占用太久**
  - **`src/composables/useRoomHost.ts`**：keepalive 间隔从 5min 缩短到 30s
  - **`api-server/config/default.toml` + `api-server/src/config/settings.rs`**：`keepalive_timeout` 1800s→120s，`cleanup_interval` 300s→60s
  - **容错链**：正常关窗→即时清理；异常退出→最迟 3 分钟回收（120s 失联判定 + 60s 扫描间隔）

#### 房主不能加入自己的房间（join_room 新增 HostCannotJoinSelf 校验）

- **`api-server/src/services/signaling.rs`**：`join_room` 步骤 2.5 新增 `host_device_pk == device_pk` 校验，返回 `HostCannotJoinSelf`（400 Bad Request）
- **`api-server/src/controllers/v1/signaling.rs`**：错误映射函数添加 `HostCannotJoinSelf` 分支

#### 信令结构体字段名大小写全面修复（7 个字段缺失 alias）

- **现象**：mesh 拓扑房主为参与者生成 Offer 后，客户端拉取 `ParticipantOfferResponse` 时 `sdp_offer`/`ice_candidates` 为空；参与者列表中 `host_offer_ready` 永远为 false；大厅列表 `LobbyModpackSummary` 的 `modpack_version`/`file_size`/`file_count` 丢失；大厅房间列表 `LobbyRoomItem.room_code` 为空
- **根因**：api-server 的 `ParticipantInfo`、`ParticipantOfferResponse`、`LobbyModpackSummary`、`LobbyRoomItem` 均无 `#[serde(rename_all = "camelCase")]`，序列化输出 snake_case；客户端同名结构体带 `rename_all = "camelCase"`，反序列化时期望 camelCase，多词字段名不匹配导致静默回退为默认值
- **`src-tauri/src/minecraft/online/signaling.rs`**：
  - `ParticipantInfo.host_offer_ready` 添加 `#[serde(alias = "host_offer_ready")]`
  - `ParticipantOfferResponse.sdp_offer`/`ice_candidates` 添加 `#[serde(alias = "...")]`
  - `LobbyModpackSummary.modpack_version`/`file_size`/`file_count` 添加 `#[serde(alias = "...")]`
  - `LobbyRoomItem.room_code` 添加 `#[serde(alias = "room_code")]`（其余字段此前已有 alias）
- 全面核对客户端与服务端所有共享结构体，确认无其他字段名大小写不匹配

#### 创建房间时整合包元数据反序列化失败（ModpackMeta 字段名大小写不一致）

- **现象**：创建关联整合包的房间时，apiServer 返回 `解密请求信封失败 error=业务数据反序列化失败`
- **根因**：客户端 `ModpackMeta`（`src-tauri/src/minecraft/online/signaling.rs`）标注了 `#[serde(rename_all = "camelCase")]`，序列化后字段名为 `projectId`/`fileId`/`mcVersion`/`modpackVersion`/`loaderVersion`/`fileSize`/`fileCount`/`manifestHash`；服务端 `ModpackMeta`（`api-server/src/models/signaling.rs`）无 `rename_all` 注解，反序列化时期望 `project_id`/`file_id`/`mc_version`/...（snake_case），导致 `serde_json::from_slice::<CreateRoomRequest>` 找不到必需字段而失败。ECIES 解密本身成功，问题在解密后的 JSON 结构不匹配
- **`api-server/src/models/signaling.rs`**：`ModpackMeta` 添加 `#[serde(rename_all = "camelCase")]`，与客户端序列化一致。数据库操作（`row_to_modpack` / `upsert_modpack`）通过 `row.try_get("column_name")` 和 `meta.field_name` 直接访问，不依赖 serde，不受影响

#### 启动时无条件调用 refresh 接口（token 未过期也刷新）

- **现象**：每次启动或切换到联机页时，即使本地 access token 未过期，后端 `auth_init` 仍主动调用 `refresh_credentials` 向云端验证有效性，产生不必要的网络请求
- **根因**：`src-tauri/src/utils/online_manager.rs` 的 `auth_init` action 在 `!creds.is_token_expired()` 分支中，主动调用 `refresh_credentials` 做"云端验证"，注释称用于检测云端撤销 token（如 RSA 密钥变更）。但前端 `onlineManager` 已有 1003 自动重试机制（refresh → login → register 降级链），token 被撤销时业务请求会收到 code=1003 触发无感重认证，无需启动时主动 refresh
- **`src-tauri/src/utils/online_manager.rs`**：`auth_init` 在 access token 未过期时直接返回本地凭证，不再调用 `refresh_credentials`。仅在 token 真正过期时才走 refresh / login 流程，与"刷新接口是给 token 过期准备的"设计意图一致

#### 硬刷新联机页时"云端连接失败"遮罩闪现

- **现象**：在联机页按 Ctrl+Shift+R 硬刷新后，页面短暂显示"云端连接失败"遮罩，然后随 `initAuth` 完成而消失
- **根因**：`App.vue` 的启动流程中，`isRestoring = false`（加载遮罩消失）后 `Online.vue` 先于 `initAuth()` 挂载。此时 `cloudConnected` 默认 `false` 且 `initializing` 默认 `false`，`Online.vue` 的 `v-if="!cloudConnected && !isInRoom"` 判定为 true，显示"云端连接失败"遮罩。待 `initAuth` 完成、`cloudConnected` 变 `true` 后遮罩才消失，形成闪烁
- **`src/stores/online.ts`**：`initializing` 初始值从 `false` 改为 `true`，表示"启动认证尚未完成"，避免 `Online.vue` 在 `initAuth` 完成前误判为连接失败
- **`src/views/Online.vue`**：空状态遮罩的 `v-if` 增加 `!onlineStore.initializing` 条件，仅在 `initAuth` 已完成（`initializing=false`）且 `cloudConnected=false` 时才显示遮罩

#### 联机页面切换后其他页面空白（transition mode="out-in" 不挂载新组件）

- **现象**：从联机页切走后目标页面空白，刷新后直接访问正常；控制台无错误
- **根因**：`App.vue` 的 `<router-view>` 外层使用了 `<transition mode="out-in">`。通过 transition 钩子日志确认：联机页的 `beforeLeave → leave → afterLeave` 完整触发，但新组件的 `beforeEnter` 从未触发，Vue 3 的 transition 在 `mode="out-in"` 下离开动画完成后未挂载新组件
- **`src/views/Online.vue`**：模板存在两个根元素（`v-if`/`v-else` 各一个 `<div>`），Vue 3 Transition 在 `mode="out-in"` 下要求过渡目标为单根组件。用外层 `<div class="h-full">` 包裹两个条件分支使其成为单根组件
- **`src/App.vue`**：移除 `<transition mode="out-in">`（该模式下 transition 在联机页离开后不挂载新组件，是 Vue 3 的已知异常行为），改用默认模式 `<transition name="route">`（同时进出，新组件立即挂载）。添加 `:key="route.path"` 确保路由切换时创建新组件实例。用 wrapper div（`relative h-full`）提供 absolute 定位上下文，`.route-leave-active` 设置 `position: absolute` 让离开组件脱离布局流，避免两个组件垂直排列导致高度溢出。路由过渡与加载遮罩过渡使用独立的 transition name（`route` / `fade`）避免 CSS 互相干扰

#### 联机页面图标导入错误导致路由跳转崩溃

- **`src/views/Online.vue`**：`@heroicons/vue/24/outline` v2 中不存在 `CloudOffIcon` 导出（v2 移除了 CloudOff/CloudSlash 等图标，仅保留 `CloudIcon` / `CloudArrowDownIcon` / `CloudArrowUpIcon`），导致点击顶部 nav 联机按钮时 Vue Router 报 `SyntaxError: does not provide an export named 'CloudOffIcon'` 并阻塞导航。改为使用 `CloudIcon`（配合灰色样式表示未连接状态）。全项目扫描 98 个使用的图标，确认无其他缺失导入

#### 下载菜单切换卡顿（移除无意义的本地版本扫描）

- **`src/views/Versions.vue`**：onMounted 原先并行调用 `versionStore.fetchVersions()` 和 `loadInstalledVersions()`，后者通过 IPC 扫描本地安装目录（`tauri.listInstalledVersionsWithType`），是耗时操作。但返回的已安装版本（如 `Zombie Invade 100 Days` 等自定义整合包）与原版下载列表的版本 id 不匹配，`installedVersions` / `installedVersionLogos` / `installedVersionTypes` 在原版列表中从未命中，标记已安装状态和版本图标均走 fallback 路径，该调用无实际意义。移除 onMounted 中的 `loadInstalledVersions()` 调用，仅保留 `fetchVersions()`。手动刷新（`handleRefresh`）和卸载（`handleUninstall`）时仍会调用 `loadInstalledVersions` 更新列表

#### HTTP 日志 req_id 字段渲染为空（前后端字段名不一致）

- **`src-tauri/src/minecraft/online/http_log.rs`**：`HttpLogEntry` 结构体字段 `req_id`（snake_case）未添加 `#[serde(rename_all = "camelCase")]`，序列化后 JSON 字段名为 `req_id`，但前端 TypeScript 类型定义期望 `reqId`（camelCase），导致 `entry.reqId` 为 `undefined`，表格渲染留空。添加 `#[serde(rename_all = "camelCase")]` 注解，序列化后字段名与前端匹配

#### 路由切换动画优化（添加位移过渡）

- **`src/App.vue`**：路由过渡原先仅有 opacity 渐变，切换时视觉生硬。添加 `transform: translateY(6px)` / `translateY(-6px)` 位移效果（进入组件从下方滑入，离开组件向上滑出），动画时长从 0.2s 调整为 0.25s，过渡更自然流畅

#### call_v1 对 code=1003 未授权响应的正确处理

- 背景：服务端返回 `code=1003`（未授权）时，`call_v1` 仍返回 `Ok(BusinessResult{code:1003})`，调用方无法区分"未授权"和普通业务失败，导致 token 被撤销（如 RSA 密钥变更、同设备多端登录）后联机页面持续报错无法恢复
- 改动：
  - **`src-tauri/src/minecraft/online/client_types.rs`**：新增 `ClientError::Unauthorized { msg, req_id }` 变体，Display 仅展示 msg（req_id 由 HTTP 日志记录，用户可自行翻阅）
  - **`src-tauri/src/minecraft/online/client.rs`**：`call_v1` 检测到 `unified.code == 1003` 时返回 `Err(ClientError::Unauthorized)`（加密信封和明文响应两条路径都处理）
  - **`src/utils/api/online-manager.ts`**：`onlineManager` 函数检测到错误包含 `code=1003` 时，自动静默走降级链重新认证（`auth_refresh` → `auth_login` → `auth_register`），认证成功后重试原请求一次。不再调用 `auth_init`（它仅检查本地过期时间，本地未过期时直接返回旧凭证，无法发现云端撤销）

#### HTTP 日志 req_id 未记录到加密响应

- 背景：`call_v1` 在解密前调用 `extract_req_id(&body_text)` 提取 req_id，但加密信封的 `body_text` 不含 `req_id` 字段（req_id 在解密后的 `unified` 响应体中），导致加密响应的 HTTP 日志 req_id 始终为空
- 改动：**`src-tauri/src/minecraft/online/client.rs`** 将 `log_http_request` 调用从解密前移至解密后，在加密分支和明文分支各自使用 `unified.req_id` 记录日志；JSON 解析失败时用 `extract_req_id` 兜底

#### 程序启动时主动向云端验证 token 有效性

- 背景：`auth_init` 在本地 token 未过期时直接返回旧凭证，不向云端验证，导致 token 被云端撤销（如 RSA 密钥变更）后用户无法感知，首次业务请求才报 1003
- 改动：**`src-tauri/src/utils/online_manager.rs`** `auth_init` 在本地 token 未过期时主动调用 `refresh_credentials` 向云端验证一次，refresh 成功则更新凭证，失败则返回旧凭证（降级由 `onlineManager` 1003 重试机制处理）

#### 开发者页面布局优化

- 背景：HTTP 日志和普通日志使用自定义折叠逻辑（无动画）、原生 `<select>` / `<button>`，与项目 UI 风格不一致
- 改动：
  - **`src/components/common/CollapsibleCard.vue`**：新增 `expand` 事件（展开时触发，支持父组件懒加载）和 `actions` 插槽（标题栏右侧操作按钮区域）；标题栏从 `<button>` 改为 `<div>` 避免嵌套 button
  - **`src/components/settings/HttpLogViewer.vue`**：改用 `CollapsibleCard`（带动画）+ 项目 `Select` + 项目 `Button`，工具栏靠右对齐，首次展开时懒加载
  - **`src/components/settings/LogViewer.vue`**：改用 `CollapsibleCard`（带动画），新增 `defaultOpen` prop（默认 false 收起），展开时懒加载，`onMounted` 不再自动加载日志
  - **`src/views/settings/SettingsDeveloper.vue`**：HTTP 日志移至普通日志上方

### 新增

#### HTTP 请求日志系统（联机 API 调用追踪）

- 背景：联机接口出现问题时难以追踪 `req_id`，需要在开发者模式查看 HTTP 请求历史
- 新增模块 **`src-tauri/src/minecraft/online/http_log.rs`**：
  - `log_http_request(method, path, status, req_id)`：追加写入到 `.Molaunch/logs/http_YYYY-MM-DD.log`
  - 日志格式：`[2026-07-29 19:47:32.123] POST /v3/auth/refresh 200 req_id=xxx`
  - `read_http_logs(date, limit)`：读取并解析为结构化 `HttpLogEntry` 数组
  - `list_http_log_files()`：列出所有 HTTP 日志文件（最新的在前）
  - `extract_req_id(body)`：从响应体 JSON 中提取 `req_id`
  - 4 个单元测试覆盖解析逻辑
- **`src-tauri/src/minecraft/online/client.rs`**：在所有 8 个请求点（`call_v1` / `register` / `login` / `logout` / `refresh` / `get_csrf_token` / `get_server_time` / `get_jwks`）收到响应后调用 `log_http_request`
- **`src-tauri/src/utils/system_manager.rs`**：新增 2 个 IPC action — `read_http_logs`（参数 `date` / `limit`）/ `list_http_log_files`
- **`src/utils/api/system-manager.ts` + `developer.ts`**：新增 `READ_HTTP_LOGS` / `LIST_HTTP_LOG_FILES` action 常量和 `readHttpLogs` / `listHttpLogFiles` API 函数，`HttpLogEntry` 类型定义
- **`src/components/settings/HttpLogViewer.vue`**（新组件，192 行）：可折叠的 HTTP 日志表格，默认收起，展开时才加载日志（避免页面卡顿）；支持选择日期、刷新；表格列：时间 / 方法 / 路径 / 状态码 / req_id，状态码和方法按颜色区分
- **`src/views/settings/SettingsDeveloper.vue`**：在日志查看器下方引入 `HttpLogViewer` 组件

### 变更

#### API 服务端地址切换检测（device.json 凭证一致性）

- 背景：用户在设置页切换 `api_server_url` 后，`device.json` 中的旧凭证（device_pk / refresh_token / 密钥对）对新域名无效，但此前未检测这种不一致，导致请求发到新域名时 401 或 ECDH 派生失败
- 改动：
  - **`src-tauri/src/minecraft/online/storage.rs`**：`DeviceCredentials` 新增 `api_server_url: String` 字段（`#[serde(default)]` 向后兼容旧凭证），`to_storage_json()` 同步写入该字段
  - **`src-tauri/src/utils/online_manager.rs`**：
    - `auth_init`：加载凭证后检查 `creds.api_server_url != api_url`，不一致则丢弃旧凭证走重新注册流程，注册成功后 `new_creds.api_server_url = api_url.clone()` 再 save
    - `refresh_credentials` / `login_fresh`：save 前若 `api_server_url` 为空（旧版凭证）补上当前服务端地址
    - `load_creds_with_auto_refresh`：加一致性检查，不一致时返回错误引导前端重新 `initAuth`

#### 设备凭证文件权限加固

- **`src-tauri/src/minecraft/online/storage.rs`**：`save()` 函数写入文件后，Unix 下显式 `set_permissions(0o600)`，防止其他用户读取私钥 / token

#### 日志脱敏误伤 URL hash 修复

- 背景：`sanitize_sensitive_info()` 中的 `LONG_TOKEN_RE`（`\b[A-Za-z0-9+=_-]{40,}\b`）会把 URL 末尾的 64 字符 hex hash 误判为 token 并替换为 `***`，导致 `[ImageCache] 已缓存: https://textures.minecraft.net/texture/***` 这样的日志无法用于排查
- 改动：**`src-tauri/src/logger/sanitize.rs`** 移除 `LONG_TOKEN_RE` 整条规则（static 定义 + 初始化 + 调用 + 测试用例），只保留 JWT 格式检测和 JSON token 字段检测两条规则。新增 `test_sanitize_preserves_texture_url_with_hex_hash` 测试用例防止回退
- 验证：`cargo test --lib sanitize` 6 个测试全部通过

#### refresh 接口请求格式升级为 MoSign-v1 协议

- 背景：api-server 的 `/v3/auth/refresh` 原请求体仅 `{ "refresh_token": "xxx" }` 明文 JSON，且路由公开无认证门槛——攻击者只要拿到 refresh_token（30 天有效期）即可直接换 access token，无身份绑定、无时间戳防重放、无签名校验。改为与 `/v3/auth/login` 完全一致的 MoSign-v1 协议结构，refresh_token 放在 AES-256-GCM 加密的 content 内
- 改动：
  - **`src-tauri/src/minecraft/online/auth.rs`**：
    - `RefreshRequest` 改为 `{device_pk, v, nonce, signature, content, timestamp}`（与 `LoginRequest` 同结构）
    - 新增 `RefreshPayload<'a>`（content 加密前的明文 JSON）：`{device_pk, timestamp, nonce, refresh_token}`
    - 新增 `pub fn build_refresh_request(creds: &DeviceCredentials) -> Result<RefreshRequest, CryptoError>`，复用 `build_login_request` 的 8 步 ECDH+AES+HMAC 流程，仅 payload 多 `refresh_token` 字段
  - **`src-tauri/src/minecraft/online/client.rs`**：`refresh` 方法签名由 `refresh(&self, refresh_token: &str)` 改为 `refresh(&self, req: &RefreshRequest)`，请求体直接 `.json(req)` 发送
  - **`src-tauri/src/utils/online_manager.rs`**：`refresh_credentials` 函数改为先 `build_refresh_request(&creds)?` 构造请求，再 `client.refresh(&req).await?`；同步导入 `build_refresh_request`
- 复用：完全复用 `build_login_request` 的 ECDH 派生 + HKDF + AES-256-GCM + HMAC-SHA256 流程与 `crypto.rs` 原语；`DeviceCredentials` 已含 ECDH 所需全部字段（`x25519_secret_b64u` / `device_public_key_b64u` / `device_pk` / `refresh_token`），无需改 `storage.rs`
- 兼容性：**破坏性变更**，旧 api-server（明文 refresh_token）与新客户端不兼容；新 api-server 与旧客户端不兼容。必须服务端+客户端同步升级
- 验证：`cargo check --lib` 通过；前端 `auth_refresh` action 不接收前端参数，对前端完全透明无需改动

#### 联机页面云端连接失败降级处理

- 背景：程序启动时静默初始化云端认证（`initAuth`），失败后需自动禁用所有联机功能。此前仅 `TopNavLayout` 和 `SettingsOnline` 消费了 `cloudConnected` 状态，联机主页及子组件未做降级，用户可通过 URL 直接访问 `/apps/online` 绕过顶部导航禁用。
- 改动：
  - **联机主页空状态遮罩**（[src/views/Online.vue](src/views/Online.vue)）：
    - `cloudConnected=false && !isInRoom` 时显示整页空状态遮罩（CloudOffIcon + 错误信息 +「打开联机设置」按钮）
    - `onMounted` 检查 `cloudConnected`，false 时跳过 `refreshStatus` / `detectNat` 避免无意义请求
    - 已在房间时（`isInRoom=true`）不遮罩，保留 P2P 连接，云端 API 失败由 toast 兜底
    - 覆盖所有子组件（LobbyBrowser / CreateRoomForm / RoomManager / OnlineDevicePanel），无需逐个降级
  - **房主轮询定时器暂停**（[src/composables/useRoomHost.ts](src/composables/useRoomHost.ts)）：
    - 提取 `startTimers()` / `stopTimers()` 函数，消除 `onMounted` / `onUnmounted` 中的重复 setInterval/clearInterval 逻辑
    - 新增 `watch(store.cloudConnected)` 监听器：云端断开时 `stopTimers()` 暂停三路轮询（参与者 5s / Answer 5s / 保活 5min），恢复时 `startTimers()` 重启
    - 避免云端断开后轮询持续失败刷屏（虽有 30s 防刷屏 toast，但仍浪费请求）
- 复用：`Online.vue` 已有的 `goSettings()` 函数和 `Cog6ToothIcon` 图标；空状态样式遵循项目约定（icon + text 垂直水平居中）
- 设计决策：
  - 不在 `useWebRTC.fetchOfferAndAnswer` 中检查 `cloudConnected`，因为它是加入房间时的一次性调用（非持续轮询），且引入 store 耦合到底层 composable 不合适
  - 不在 `LobbyBrowser` / `CreateRoomForm` / `RoomManager` / `OnlineDevicePanel` 中单独降级，因为 `Online.vue` 的页面级遮罩已覆盖未在房间的所有场景
  - 已在房间时云端断开的边缘场景（关闭房间 / 踢人 / 退出房间等操作失败）由现有 toast 错误提示兜底

#### updater.exe 实时构建机制（CI + build.rs 双保险）

- 背景：updater.exe 是 Windows 便携版更新器的独立 Cargo 项目编译产物，之前以二进制文件形式内嵌在 `resources/updater/` 目录中。为避免二进制文件污染仓库历史、保证 CI 每次构建使用最新源码，改为实时构建机制。
- 改动：
  - **build.rs 自动构建**（[src-tauri/build_script/updater.rs](src-tauri/build_script/updater.rs)，新增）：
    - 新增 `updater` 模块（`#[cfg(target_os = "windows")]`，非 Windows 平台编译期排除）
    - 增量编译判断：检测 `updater/src/*.rs`、`updater/Cargo.toml`、`updater/build.rs` 是否比 `resources/updater/updater.exe` 新
    - 源码变化或产物不存在时自动调用 `cargo build --release` 编译 updater 项目
    - 编译产物从 `updater/target/release/molaunch_updater.exe` 复制到 `resources/updater/updater.exe`
    - 为每个源文件声明 `cargo:rerun-if-changed`，确保内容修改触发 build.rs 重跑
    - 复用 `cubiomes_wasm.rs` 的增量编译模式（`output()` 捕获 stdout/stderr 避免管道阻塞）
  - **build.rs 注册**（[src-tauri/build.rs](src-tauri/build.rs)）：
    - 在 `main()` 末尾添加 `#[cfg(target_os = "windows")] build_script::updater::build_updater()`
  - **build_script/mod.rs**（[src-tauri/build_script/mod.rs](src-tauri/build_script/mod.rs)）：
    - 新增 `#[cfg(target_os = "windows")] pub mod updater`
  - **CI 工作流**（[.github/workflows/release.yml](.github/workflows/release.yml)）：
    - Windows 平台在 `Build Tauri` 之前新增 `Build updater.exe (Windows only)` 步骤
    - `Swatinem/rust-cache@v2` 新增 `src-tauri/updater` 工作空间，缓存 updater 编译产物加速 CI
    - CI 显式构建可利用 rust-cache 加速且日志清晰，build.rs 作为本地开发双保险
  - **.gitignore 排除规则**（[.gitignore](.gitignore)）：
    - 新增 `src-tauri/resources/updater/updater.exe`（编译产物不入库）
    - 新增 `src-tauri/updater/target/`（updater 独立项目编译产物）
- 复用：增量编译判断逻辑参考 `cubiomes_wasm.rs` 的 `needs_recompile` 实现；`output()` 捕获模式参考 `cubiomes_wasm.rs` 的 emcc 调用
- 验证：`git status` 确认 updater.exe 未被 git 跟踪；build.rs 编译逻辑通过 `cargo check` 验证

#### 联机模块接入 refresh_token 双 token 机制 + 启动静默登录/注册

- 背景：api-server 已完成 refresh_token 改造，`/v3/auth/register` 和 `/v3/auth/login` 响应新增 `refresh_token` 字段，新增 `/v3/auth/refresh` 接口（access_token 1h + refresh_token 30d）。客户端需对接双 token 机制，避免 access_token 过期后用户被强制重新登录，并在启动时静默完成注册/登录/续期，实现「无感联机」。
- 改动：
  - **DeviceCredentials 新增字段**（[src-tauri/src/minecraft/online/storage.rs](src-tauri/src/minecraft/online/storage.rs)）：
    - 新增 `refresh_token: String` + `refresh_expires_at: u64`，均加 `#[serde(default)]` 兼容旧版凭证文件
    - 新增 `is_refresh_token_expired()` 方法（容差 60 秒，`refresh_expires_at == 0` 视为已过期）
    - `to_storage_json()` 纳入新字段，保持不派生 `Serialize` 的安全约束
  - **响应类型新增 refresh_token**（[src-tauri/src/minecraft/online/auth.rs](src-tauri/src/minecraft/online/auth.rs)）：
    - `RegisterData` / `LoginData` 新增 `refresh_token: String`（`#[serde(default)]` 兼容老服务端）
    - 新增 `RefreshRequest` / `RefreshResponse` / `RefreshData` 结构
    - 新增常量 `REFRESH_TOKEN_TTL_SECS = 30 * 24 * 3600`
    - `finalize_credentials_with_register` / `finalize_credentials_with_login` 纳入 refresh_token + refresh_expires_at
    - 新增 `finalize_credentials_with_refresh(creds, data)`：更新 access_token，服务端轮换 refresh_token 时替换本地
  - **OnlineClient 新增 refresh 方法**（[src-tauri/src/minecraft/online/client.rs](src-tauri/src/minecraft/online/client.rs)）：
    - `pub async fn refresh(&self, refresh_token: &str) -> Result<RefreshResponse, ClientError>`
    - 调用 `POST /v3/auth/refresh`，无需 JWT/CSRF 头
  - **online_manager 新增 2 个 action**（[src-tauri/src/utils/online_manager.rs](src-tauri/src/utils/online_manager.rs)）：
    - `auth_refresh`：手动续期 access token（前端「刷新凭证」按钮或内部流程调用）
    - `auth_init`：启动静默登录/注册决策树（无凭证→注册 / token 有效→直接返回 / token 过期+refresh 有效→续期 / 双 token 过期→重新登录）
    - 新增 `AuthInitResult { status, error }` 返回结构
    - 新增 `refresh_credentials` / `login_fresh` / `load_creds_with_auto_refresh` / `build_device_status` 辅助函数，消除各 action 重复构造逻辑
  - **信令 action 自动续期**（[src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs)）：
    - `load_creds` 复用 `online_manager::load_creds_with_auto_refresh`，access token 过期时自动 refresh，无需各信令 action 单独处理
    - 移除不再使用的 `OnlineStorage` 导入
- 复用：`refresh_credentials` 被 `auth_refresh` / `auth_init` / `load_creds_with_auto_refresh` 三处共用；`login_fresh` 封装 ECDH 登录流程供 `auth_init` 降级使用；`build_device_status` 消除 4 处 DeviceStatus 内联构造
- 安全：保持 MoSign-v1 签名验证逻辑不变；保持 DeviceCredentials 不派生 Serialize；保持存储加密机制不变；新字段 `#[serde(default)]` 兼容旧版凭证文件
- 验证：`cargo check` 通过（零错误零警告）

#### Windows 便携版自更新机制（方案 B 落地：updater.exe 子进程替换）

- 背景：Windows 便携版（单 exe 直接运行）与 Tauri 官方 updater plugin 不兼容（官方 plugin Windows 端必须依赖 NSIS installer）。为支持便携版自更新，采用平台分流：Windows 自实现 updater.exe 子进程替换，macOS/Linux 保留官方 plugin。
- 设计文档：[docs/updater/design.md](docs/updater/design.md) §2.3 混合方案
- 改动：
  - **updater.exe 独立 Cargo 项目**（[src-tauri/updater/](src-tauri/updater/)，新增）：
    - 依赖 std + windows crate + ed25519-dalek + base64 + sha2，体积 370KB
    - 模块化拆分：`main.rs`（入口编排）/ `args.rs`（参数解析）/ `platform.rs`（Windows API）/ `verify.rs`（签名校验）/ `log.rs`（日志）
    - 接收 `--old-exe` / `--new-exe` / `--pid` / `--signature` 四个参数
    - 等待主进程退出（OpenProcess + WaitForSingleObject，30s 超时）
    - **minisign 签名校验**：硬编码 Ed25519 公钥（来自 `tauri.conf.json` 的 `plugins.updater.pubkey`），用 ed25519-dalek 验证新 exe 的 SHA-512 prehash 签名，防止篡改
    - 替换 exe（MoveFileExW + MOVEFILE_REPLACE_EXISTING，失败回退备份方案）
    - 启动新 exe 后自身退出
    - **winres 嵌入版本信息**：`build.rs` 使用 winres crate 嵌入 FileDescription / ProductName / LegalCopyright + 图标，exe 详细信息不再为空
    - **README.md**：替代代码注释，记录功能、参数、退出码、模块结构、安全设计
  - **资源注册**（[src-tauri/src/resources.rs](src-tauri/src/resources.rs)）：
    - `embedded_bytes` 新增 `#[cfg(target_os = "windows")] "updater/updater.exe"` 分支
    - 新增 `extract_updater()` 函数，释放到 `%APPDATA%/.Molaunch/updater/updater.exe`（复用 sha256 校验机制）
  - **统一更新命令**（[src-tauri/src/commands/system/updater.rs](src-tauri/src/commands/system/updater.rs)，新增）：
    - `check_update`：所有平台复用 `tauri-plugin-updater` 的 check() 获取 manifest，提取 force_update / url / signature 扩展字段
    - `download_and_install`：Windows 自实现（下载 exe → 释放 updater.exe → 启动子进程并传递 `--signature` → 退出主程序），macOS/Linux 转发官方 plugin
    - 在 `system_manager` 注册 2 个 action（`check_update` / `download_and_install_update`）
  - **前端改造**（[src/utils/updater.ts](src/utils/updater.ts)）：
    - 从直接调用 `@tauri-apps/plugin-updater` 改为调用 `systemManager('check_update')` / `systemManager('download_and_install_update', updateInfo)`
    - 前端不关心平台差异，后端根据 `cfg!(target_os = "windows")` 分流
    - `UpdateDialog.vue` 无需改动（接口未变）
  - **system-manager API**（[src/utils/api/system-manager.ts](src/utils/api/system-manager.ts)）：`SYSTEM_ACTIONS` 新增 `CHECK_UPDATE` / `DOWNLOAD_AND_INSTALL_UPDATE`

#### Windows 便携版自动更新（静默下载 + 退出时替换）
- 背景：此前 Windows 自实现流程需用户手动点击「检查更新 → 下载安装」，体验不连贯。改为后台静默下载新版本到 appdata，用户退出程序时自动替换主 exe，下次启动即为新版本，全程零打扰。
- 设计文档：[docs/updater/design.md](docs/updater/design.md) §4 Windows 便携版 updater
- 改动：
  - **后端新增 2 个命令**（[src-tauri/src/commands/system/updater.rs](src-tauri/src/commands/system/updater.rs)）：
    - `download_update_to_appdata(info)`：使用 `crate::http::get_client()` 下载新版本 exe 到 `%APPDATA%/.Molaunch/last.exe`（确保父目录存在），不走 NSIS installer，复用代理配置
    - `apply_pending_update(app)`：退出时检查 `last.exe` 是否存在——存在则释放 `updater.exe` 并启动子进程（传递 `--old-exe` / `--new-exe` / `--pid`），返回 true 由调用方退出主程序；不存在则返回 false 正常退出
    - 新增 `last_exe_path()` 辅助函数（Windows 平台返回 `%APPDATA%/.Molaunch/last.exe`）
    - 在 `system_manager` 注册 2 个 action（`download_update_to_appdata` / `apply_pending_update`），与既有 `check_update` / `download_and_install_update` 共存
  - **前端定时检查 + 退出触发**（[src/utils/updater.ts](src/utils/updater.ts)）：
    - `initAutoCheck` 平台分流：Windows 10 分钟间隔静默检查 + 自动下载到 appdata；macOS/Linux 6 小时间隔 + 弹窗手动安装
    - 新增 `silentCheckAndDownload()`：Windows 专属，发现新版本后调 `download_update_to_appdata`，记录 `appdataDownloadedVersion` 防止定时重复下载同一版本
    - 新增 `applyPendingUpdate()`：前端退出时调用，转调后端 `apply_pending_update` 命令
    - Dev 模式（`import.meta.env.DEV`）跳过自动检查，避免 dev 版本号低于发布版反复触发更新
  - **退出时触发替换**（[src/components/layout/TopNavLayout.vue](src/components/layout/TopNavLayout.vue)）：
    - `handleClose` 在保存配置 + 联机状态清理后、`appWindow.close()` 前调用 `await applyPendingUpdate().catch(() => false)`
    - 返回 true 时后端已启动 updater.exe 子进程，主程序退出后 updater 接管替换 exe，下次启动即为新版本
  - **system-manager API**（[src/utils/api/system-manager.ts](src/utils/api/system-manager.ts)）：`SYSTEM_ACTIONS` 新增 `DOWNLOAD_UPDATE_TO_APPDATA` / `APPLY_PENDING_UPDATE`
- 用户反馈："tauri 改下，后端默认检查更新还是走 v1... 前端改下，设置10分钟一个检查点，定时请求updater接口获取更新，当然只对windows生效... 检查到新版本后，自动下载文件到appdata目录，命名为 last.exe，后续更新版本自动替换掉... updater.exe在Tauri程序被用户点击右上角退出的时候，自动调用替换"
- 验证：`cargo check -p mo-launch` 通过（零错误零警告）

#### 客户端自动更新能力落地（方案 B 第二阶段：客户端集成）
- 背景：服务端更新服务（api-server `/v1/updates/*` + S3 presigned URL）与 CI/CD 流水线（`.github/workflows/release.yml`）已就绪，客户端需接入 Tauri 官方 updater plugin 完成端到端自动更新闭环
- 设计文档：[docs/updater/design.md](docs/updater/design.md) §4 客户端实现
- 改动：
  - **Tauri 配置**（[src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)）：
    - `plugins.updater.pubkey` 填入 Ed25519 公钥（`npx tauri signer generate` 生成）
    - `endpoints` 修正为 `https://api.molaunch.moiu.cn/v1/updates/manifest/raw`（之前文档占位为 `api.molaunch.net`，实际域名为 `moiu.cn`）
    - `app.security.csp` 的 `connect-src` 追加 `https://api.molaunch.moiu.cn`（查询 manifest）与 `https://download.mocdn.net`（下载 presigned URL）
    - `windows.installMode = passive`（NSIS 进度条小窗口，无需用户交互）
  - **Rust 依赖**（[src-tauri/Cargo.toml](src-tauri/Cargo.toml)）：新增 `tauri-plugin-updater = "2"` + `tauri-plugin-process = "2"`
  - **JS 依赖**（[package.json](package.json)）：新增 `@tauri-apps/plugin-updater` + `@tauri-apps/plugin-process`
  - **Plugin 注册**（[src-tauri/src/lib.rs](src-tauri/src/lib.rs)）：`tauri::Builder` 链追加 `tauri_plugin_updater::Builder::new().build()` + `tauri_plugin_process::init()`
  - **权限配置**（[src-tauri/capabilities/migrated.json](src-tauri/capabilities/migrated.json)）：追加 `updater:default` / `process:default` / `process:allow-relaunch`
  - **更新工具单例**（[src/utils/updater.ts](src/utils/updater.ts)，新增）：
    - 参照 `utils/modal.ts` / `utils/toast.ts` 的 module-level 单例模式
    - 暴露 `updateState`（响应式状态：status / version / notes / forceUpdate / downloaded / total / error / showDialog）
    - 提供 `checkForUpdate({ silent })` / `downloadAndInstall()` / `closeDialog()` / `initAutoCheck()` 四个函数
    - 静默模式（启动 5s + 每 6h 定时）仅在发现更新时弹窗，无更新/检查失败均不打扰用户
    - 手动模式（关于页按钮触发）无论结果都给用户反馈（toast 或弹窗）
    - 强制更新（`manifest.force_update=true`）时禁用关闭按钮、禁用遮罩/ESC 关闭
    - dev 模式（`import.meta.env.DEV`）跳过自动检查，避免 dev 版本号低于发布版本时反复触发更新
  - **更新弹窗组件**（[src/components/about/UpdateDialog.vue](src/components/about/UpdateDialog.vue)，新增，242 行）：
    - 监听 `updateState.showDialog` 自动显示/隐藏
    - 状态切换：检查中 loading → 发现新版本（版本号 + 更新日志 + 按钮）→ 下载进度条 → 安装中 loading（即将重启）→ 错误信息
    - 强制更新时不显示"稍后"按钮，仅"立即更新"
    - 复用项目自定义 `Button.vue`，未使用原生 `<button>`（除关闭按钮 X 图标）
  - **关于页入口**（[src/views/settings/more/AboutTab.vue](src/views/settings/more/AboutTab.vue)）：在 MoLaunch 介绍卡片右上角追加"检查更新"按钮（`outline` 样式 + ArrowPathIcon），点击触发 `checkForUpdate({ silent: false })`
  - **App.vue 集成**（[src/App.vue](src/App.vue)）：`onMounted` 调用 `initAutoCheck()` 注册启动 5s + 每 6h 定时检查；模板挂载 `<UpdateDialog />` 全局弹窗
  - **CI/CD 同步**（[.github/workflows/release.yml](.github/workflows/release.yml)）：版本注册地址修正为 `https://api.molaunch.moiu.cn/v1/admin/updates/releases`
  - **设计文档同步**（[docs/updater/design.md](docs/updater/design.md)）：所有 `api.molaunch.net` 占位域名替换为实际 `api.molaunch.moiu.cn`
- 复用：参照 `utils/modal.ts` 的 module-level 单例模式（`setModalRef` 注入）+ `utils/toast.ts` 的 toast 反馈机制 + 项目自定义 `Button.vue` 组件
- 端到端流程：客户端启动 5s 静默检查 → api-server 查询 `app_releases` 表 → S3 presigned URL 下发 → Tauri plugin 下载并校验 Ed25519 签名 → NSIS passive 模式安装 → `relaunch()` 重启主进程
- 验证：`cargo check` + `npm run typecheck` 待执行（需先安装新依赖 `npm install`）

#### 修复大文件下载被全局 30s timeout 误杀问题
- 背景：用户反馈 132.7 MB 文件分片下载时 chunk 0/1 报 `request or response body error: operation timed out`，8 秒就超时
- 根因：[http.rs](src-tauri/src/http.rs) 全局 HTTP 客户端构建时设置了 `.timeout(Duration::from_secs(30))`，这是 reqwest 的整体超时（连接+响应头+body 读取）。分片下载（33MB/chunk）和单流下载在慢速网络下 30s 下载不完 body 就被误杀
- 修复：
  - [chunk/download.rs](src-tauri/src/minecraft/download/chunk/download.rs) `client.get(url)` 加 `.timeout(Duration::from_secs(86400))`（24h 兜底），覆盖全局 30s。实际超时由现有"无数据流动 15s"机制控制，只有真断流才报错
  - [stream.rs](src-tauri/src/minecraft/download/downloader/stream.rs) 同样加 `.timeout(Duration::from_secs(86400))`，连接阶段仍由 `tokio::time::timeout(5s/10s)` 控制，body 读取阶段由"无数据流动 15s"控制
  - [java/download/files.rs](src-tauri/src/minecraft/java/download/files.rs) Java 运行时下载（50-100MB）加 `.timeout(Duration::from_secs(300))`（5 分钟），覆盖全局 30s
- 复用：沿用现有 `STREAM_IDLE_TIMEOUT_SECS = 15` 和 chunk 15s 无数据流动超时机制，不引入新逻辑
- 排查：全项目搜索 `.send().await` / `.bytes().await`，确认 API 请求和小文件场景 30s 全局 timeout 合理，仅大文件下载场景需覆盖
- 验证：`cargo check` 通过（零错误零警告）

#### 修复日志脱敏误伤 URL 路径问题
- 背景：用户反馈下载日志中 URL 显示为 `https://cdn-modrinth.mocdn.***.0.14-mc-1.20.1-forge.jar`，`net/data/.../physics-mod-3` 被替换为 `***`
- 根因：[logger/sanitize.rs](src-tauri/src/logger/sanitize.rs) `long_token_re` 正则字符集 `[A-Za-z0-9+/=_-]` 包含 `/`，导致 URL 路径段 `net/data/l9m9tuPN/versions/M8j2mfGj/physics-mod-3`（49 字符，含 `/` 和 `-`）被整体当作长 token 替换
- 修复：从字符集移除 `/`，改为 `[A-Za-z0-9+=_-]{40,}`。URL 路径会被 `/` 断开成短段（每段不足 40 字符阈值），真实 JWT/url-safe base64 token 不含 `/`，不受影响
- 测试：新增 `test_sanitize_preserves_urls` 用例验证 URL 不被误伤，5 个测试全部通过
- 验证：`cargo test --lib logger::sanitize` 通过（5 passed; 0 failed）

#### logger 模块拆分（单文件 455 行 → 目录模块 3 文件）
- 背景：[src-tauri/src/logger.rs](src-tauri/src/logger.rs) 单文件 455 行，接近 400 行关注阈值，职责混杂（核心日志器 + 文件查看 API + 脱敏 + 测试），需按职责拆分提升可维护性
- 改动：
  - `logger.rs` 删除，改为 `logger/` 目录模块：[mod.rs](src-tauri/src/logger/mod.rs)（300 行：LogLevel + Logger 结构体 + init/set_level/log/separator + 宏 + strip_to_src_relative）、[viewer.rs](src-tauri/src/logger/viewer.rs)（77 行：日志文件路径查询/列表/读取 API）、[sanitize.rs](src-tauri/src/logger/sanitize.rs)（92 行：敏感信息脱敏 + 4 个测试用例）
  - `mod.rs` 通过 `pub use sanitize::sanitize_sensitive_info;` 和 `pub use viewer::{get_log_path, list_log_files, read_log_file};` 重新导出，保持 `crate::logger::*` 公共路径完全不变
  - `#[macro_export]` 宏内部 `$crate::logger::log` / `$crate::logger::LogLevel` / `$crate::logger::separator` 路径零改动
  - 所有调用方（system_manager.rs / config.rs / apply.rs / lib.rs）零改动
- 复用：沿用原文件全部实现逻辑，仅按职责拆分文件边界，无功能变更
- 验证：`cargo check` 通过（10.61s 零错误零警告）

#### 致谢页新增 MoCDN 条目
- 背景：前序提交已将 `cdn-modrinth.mocdn.net` / `cdn-curseforge.mocdn.net` 自建 CDN 镜像接入下载链路，需在「关于」页致谢列表中补充 MoCDN 项目说明与作者信息
- 改动：
  - [src-tauri/resources/about/acknowledgements.txt](src-tauri/resources/about/acknowledgements.txt) 表格新增一行：`MoCDN | https://mocdn.net | 提供 CurseForge 和 Modrinth 的文件下载加速服务 | mocdn-logo.png | MoTeam:MoTeam.png`
  - 复用已有资源文件 [src/assets/AboutIcon/mocdn-logo.png](src/assets/AboutIcon/mocdn-logo.png)（logo）与 [src/assets/AboutIcon/MoTeam.png](src/assets/AboutIcon/MoTeam.png)（作者头像），无需新增资源
- 复用：沿用 acknowledgements.txt 现有表格格式（`| name | home | desc | logo | authors |`），`include_str!` 嵌入机制无需改动

#### Modrinth CDN 域名替换为 cdn-raw.modrinth.com（绕过中国大陆 cdn-alt 跳转）
- 背景：Modrinth 官方 CDN `cdn.modrinth.com` 对中国大陆用户会 302 跳转到慢速 `cdn-alt.modrinth.com`，导致下载速度极差；`cdn-raw.modrinth.com` 是 Modrinth 官方直连域名，路径结构与原始 CDN 完全一致，仅替换域名即可绕过跳转
- 改动：
  - **sources.rs 新增 rewrite_mr_cdn 函数**：[src-tauri/src/minecraft/sources.rs](src-tauri/src/minecraft/sources.rs) 新增 `MR_CDN_OFFICIAL` / `MR_CDN_RAW` 常量 + `rewrite_mr_cdn(url)` 公开函数（`cdn.modrinth.com` → `cdn-raw.modrinth.com`，`replacen` 仅替换首次匹配）；`apply_cdn_mirrors` 同步匹配 `cdn-raw.modrinth.com`（rewrite 后的域名）+ 防御性兼容原始 `cdn.modrinth.com`；`MR_CDN_DOMAINS` 更新为含两个域名
  - **条件执行（默认关闭）**：`replace_cdn` / `cdn_urls` 入口处仅当 `modrinth_cdn_raw_enabled` 配置为 true 时才调用 `rewrite_mr_cdn`；新增 `get_modrinth_cdn_raw_enabled()` 私有函数直接读 Storage INI（与 `community::get_source_pref` 同模式，避免 mutex）；默认 false，仅在开发者模式解锁后可开启
  - **配置持久化**：[src-tauri/src/state/config.rs](src-tauri/src/state/config.rs) `DownloadConfig` 新增 `modrinth_cdn_raw_enabled: bool`（`#[serde(default)]` 兼容旧配置）；[src-tauri/src/config.rs](src-tauri/src/config.rs) `load_config` / `save_config` 同步读写 `Download/modrinth_cdn_raw_enabled` INI 键；[src-tauri/resources/defaults/config.ini](src-tauri/resources/defaults/config.ini) 模板新增 `modrinth_cdn_raw_enabled=false` + `Proxy/ip_version=any` 默认值
  - **apply_config 类型同步**：[src-tauri/src/commands/system/apply_config/types.rs](src-tauri/src/commands/system/apply_config/types.rs) `DownloadPatch` / `DownloadSnapshot` / `build_snapshot` 新增 `modrinth_cdn_raw_enabled` 字段（前端 camelCase `modrinthCdnRawEnabled`）；[src-tauri/src/commands/system/apply_config/apply.rs](src-tauri/src/commands/system/apply_config/apply.rs) `apply_download` 处理该字段变更
  - **前端类型同步**：[src/utils/api/config.ts](src/utils/api/config.ts) `ConfigSnapshot` / `ConfigPatch` 新增 `modrinthCdnRawEnabled` 字段
  - **开发者页面 UI**：[src/views/settings/SettingsDeveloper.vue](src/views/settings/SettingsDeveloper.vue) 新增「实验性功能」卡片（位于页面顶部），含「Modrinth CDN 直连」开关行（`Select` 组件）；`toggleModrinthCdnRaw` 通过 `applyConfig({ modrinthCdnRawEnabled })` 持久化，失败时回滚 UI；页面 205 行（< 300 行硬约束）
  - **覆盖范围**：所有经过 `cdn_urls()` / `replace_cdn()` 的 Modrinth 下载 URL（整合包安装 / 资源下载 / Mod 更新 / Fabric API 安装），无遗漏调用点
- 复用：
  - 复用 `replacen` 域名替换模式，与现有 `apply_cdn_mirrors` 实现风格一致
  - 复用 `cdn_urls` / `DownloadManager` fallback 机制，rewrite 后的 URL 仍可生成 mocdn + mcimirror 镜像 fallback
  - 复用 `community::get_source_pref` 的 Storage 直读模式，避免在同步函数中 `blocking_lock` mutex
  - 复用 DevModeToggle.vue 现有 `toggleDevMode` 模式（try/applyConfig/catch 回滚）
- 与 CDN 镜像的关系：rewrite 在镜像替换之前执行，`cdn-raw.modrinth.com` 作为"官方 URL"参与 fallback 排序（source=1 时：`[cdn-raw官方, mocdn镜像, mcimirror镜像]`），不影响镜像逻辑
- 验证：`cargo check` 通过（51.28s 零 warning）；`vue-tsc --noEmit` 本次修改文件（config.ts / DevModeToggle.vue）无类型错误

#### CDN 镜像新增自建加速域名（mocdn.net）
- 背景：用户自建了 CDN 加速域名（mocdn.net），需要将社区资源下载 URL 替换为自建 CDN，提升下载速度；现有 `mod.mcimirror.top` 作为兜底保留
- 域名替换规则：
  - **Modrinth**：`cdn.modrinth.com` → `cdn-modrinth.mocdn.net`（优先）+ `mod.mcimirror.top`（兜底）
  - **CurseForge edge**：`edge.forgecdn.net` → `cdn-curseforge.mocdn.net`（优先）+ `mod.mcimirror.top`（兜底）
  - **CurseForge media**：`media.forgecdn.net` → `mod.mcimirror.top`（仅 MCIMirror，mocdn 不支持此域名路径）
- 改动：
  - **sources.rs 多镜像支持**：[src-tauri/src/minecraft/sources.rs](src-tauri/src/minecraft/sources.rs) 新增 `MR_MOCDN_MIRROR` / `CF_MOCDN_MIRROR` 常量；`apply_cdn_mirror`（返回单个 String）重构为 `apply_cdn_mirrors`（返回 `Vec<String>`），按优先级返回所有镜像 URL；`cdn_urls` 从 `[官方, 镜像]` 扩展为 `[官方, mocdn镜像, mcimirror镜像]`（source=1 时）；`replace_cdn` 取第一个镜像（mocdn 优先）
  - **helpers.rs 简化 construct_cf_edge_url**：[src-tauri/src/commands/community/install/helpers.rs](src-tauri/src/commands/community/install/helpers.rs) 移除 `source == 0` 时直接用 `CDN_MIRROR` 的分支，始终用官方 `edge.forgecdn.net` 构造 URL，镜像替换统一由 `cdn_urls()` 处理（避免双重镜像逻辑 + 确保 source=0 时也能获得 mocdn + mcimirror 双 fallback）
- 复用：
  - 复用现有 `cdn_urls` / `DownloadManager` fallback 机制，无需新增下载重试逻辑
  - 复用 `replacen` 域名替换模式，保持与原有 `apply_cdn_mirror` 实现风格一致
- 验证：`cargo check` 通过（43.94s 零 warning）

#### HTTP 客户端新增 IP 协议版本偏好控制（v4 / auto / any）
- 背景：部分域名 IPv6 链路质量差导致下载卡死，需要让用户手动控制客户端使用的 IP 协议版本；同时需支持配置热更新，无需重启应用即可切换
- 三档策略：
  - **`v4`**：强制 IPv4（`local_address = 0.0.0.0`，reqwest 仅解析 A 记录）
  - **`auto`**：自动选择（TCP 连接 Cloudflare DNS `1.1.1.1:443` / `[2606:4700:4700::1111]:443` 测试 v4/v6 连通性与延迟，选更稳定的一方；延迟差异 < 50ms 视为接近，让 OS 决定）
  - **`any`**：随意解析（不设置 `local_address`，跟随 DNS 服务器，默认值）
- 改动：
  - **config.rs ProxyConfig 新增 ip_version 字段**：[src-tauri/src/state/config.rs](src-tauri/src/state/config.rs) `ProxyConfig` 增加 `ip_version: String`（`#[serde(default)]` 兼容旧配置），默认 `"any"`；`AppConfig::default()` 同步初始化
  - **config.rs 持久化修复**：[src-tauri/src/config.rs](src-tauri/src/config.rs) `load_config` / `save_config` 新增 `Proxy/ip_version` INI 键读写（此前 ProxyConfig 已有字段但未持久化，重启后丢失）
  - **http.rs 支持 ip_version 参数**：[src-tauri/src/http.rs](src-tauri/src/http.rs) `init_client` / `build_client` 新增 `ip_version: &str` 参数；新增 `resolve_local_address` 按 v4/auto/any 策略返回 `Option<IpAddr>`；新增 `auto_detect_ip_version` + `test_tcp_connect` 实现 v4/v6 连通性测试（2s 超时）
  - **apply_config 类型同步**：[src-tauri/src/commands/system/apply_config/types.rs](src-tauri/src/commands/system/apply_config/types.rs) `ProxyPatch` / `ProxySnapshot` / `build_snapshot` 新增 `ip_version` 字段（前端 camelCase `ipVersion`）
  - **apply.rs apply_proxy 处理 ip_version 变更**：[src-tauri/src/commands/system/apply_config/apply.rs](src-tauri/src/commands/system/apply_config/apply.rs) `proxy_pending` 从三元组 `(mode, kind, url)` 扩展为四元组 `(mode, kind, url, ip_version)`；`apply_proxy` 检测 `ip_version` 变更并收集到 `proxy_pending`；闭包外副作用阶段调用 `init_client` 传入四元组重建客户端
  - **lib.rs 启动初始化传入 ip_version**：[src-tauri/src/lib.rs](src-tauri/src/lib.rs) `init_client` 调用从 3 参数扩展为 4 参数，启动日志同步打印 `ip_version`
  - **前端类型同步**：[src/utils/api/config.ts](src/utils/api/config.ts) `ConfigSnapshot` / `ConfigPatch` 新增 `ipVersion` 字段（camelCase，与后端对齐）
  - **前端设置页 UI**：[src/views/settings/SettingsAdvanced.vue](src/views/settings/SettingsAdvanced.vue) 代理配置卡片新增「IP 协议版本」独立行（`Select` 组件，三档：强制 IPv4 / 自动选择 / 跟随 DNS），位于代理地址下方；`ipVersion` ref + `onLoad` 赋值 + `watch` markDirty 三件套完整接入 `useConfigPage` 防抖保存链路
- 复用：
  - `proxy_pending` 复用 `log_level_pending` 的「闭包内收集 → 闭包外执行副作用」模式（避免跨 await 持有锁）
  - `auto_detect_ip_version` 复用 `std::net::TcpStream::connect_timeout` 标准库能力，无需引入新依赖
  - 与现有代理热重建机制完全融合，ip_version 变更与代理变更共用同一条重建路径
- 验证：`cargo check` 通过（43.91s 零 warning）；`vue-tsc --noEmit` 本次修改文件（SettingsAdvanced.vue / config.ts）无类型错误（其余报错均为项目中已存在的类型问题，与本次改动无关）；SettingsAdvanced.vue 275 行（< 300 行项目硬约束）

#### 通用缓存图片组件 CachedImage + 社区资源卡片接入
- 背景：用户反馈「社区资源卡片图片加载很卡，要卡好一会」，`ResourceCard` / `ResourceDetailHeader` 直接用远程 URL 渲染 Logo，每次进入列表都重新发起网络请求，未复用后端已有的 `image_cache_manager` 缓存能力
- 改动：
  - **新增 CachedImage.vue 通用组件**：[src/components/common/CachedImage.vue](src/components/common/CachedImage.vue)（103 行）复用 `getCachedImageUrl` + `onImageCached`；`src` 变化时调用 `getCachedImageUrl`，命中缓存直接用 `cache-image://` 本地 URL（零网络请求），未命中先用远程 URL 渲染并监听 `image-cached` 事件，事件匹配后切换为本地 URL；`inheritAttrs: false` + `v-bind="attrs"` 仅绑定到 `<img>`，避免污染 fallback 插槽；支持 `fallback` 插槽（src 为空或加载失败时渲染）
  - **ResourceCard 接入**：[src/components/community/ResourceCard.vue](src/components/community/ResourceCard.vue) Logo `<img>` 替换为 `<CachedImage>`，fallback 插槽为 `CubeIcon`，移除原 `@error` 隐藏逻辑（由组件内置 `fallbackOnError` 处理）
  - **ResourceDetailHeader 接入**：[src/components/community/resource-detail/ResourceDetailHeader.vue](src/components/community/resource-detail/ResourceDetailHeader.vue) 同上替换为 `<CachedImage>` + `CubeIcon` fallback
- 复用：
  - 复用 `@/utils/api/image-cache` 的 `getCachedImageUrl`（已有工具，无需新增 IPC）
  - 复用 `@/composables/useImageCache` 的 `onImageCached`（全局单例 listener，组件卸载自动移除 handler，无 Tauri 2.x unlisten 竞态）
  - 复用 `SkinAvatar.vue` 的事件匹配模式（`pendingRemoteUrl` 标记 → 事件匹配后切换 URL）
- 验证：`vue-tsc --noEmit` 本次修改文件（CachedImage.vue / ResourceCard.vue / ResourceDetailHeader.vue）无类型错误（`src` prop 类型放宽为 `string | null` 以匹配 `ResourceProject.logo_url` 类型）

#### 右下角悬浮按钮布局修复（BackToTop 阈值 + DownloadPanel 动态避让）
- 背景：用户反馈「全局上滑按钮没有出来」+「下载按钮在全局上滑按钮上方，即使对方没出现还是预留位置」。根因：BackToTop 出现阈值 700px 过高（多数场景滚动未达阈值）；DownloadPanel 固定 `bottom-20`（80px）无论 BackToTop 是否可见都预留了上方空位
- 改动：
  - **BackToTop 阈值下调**：[src/components/common/BackToTop.vue](src/components/common/BackToTop.vue) `SHOW_THRESHOLD` 从 700px 降至 400px，滚动约 6-7 个资源卡片高度即触发
  - **悬浮按钮状态协调**：新增 [src/composables/useFloatingButtonState.ts](src/composables/useFloatingButtonState.ts) 模块级共享 `backToTopVisible` ref（无需 Pinia，仅两个组件协调）；BackToTop `watch(visible)` 同步状态；DownloadPanel `computed` 读取状态动态切换 `positionClass`：BackToTop 不可见时贴底 `bottom-6`（24px），可见时上移 `bottom-24`（96px）腾出空间
  - **DownloadPanel 位置动态化**：[src/components/common/DownloadPanel.vue](src/components/common/DownloadPanel.vue) `class` 从固定字符串改为 `:class` 数组绑定，`positionClass` 响应 `backToTopVisible` 变化，CSS `transition-all` 保证位置切换有动画
- 复用：模块级共享 ref 模式（Vue 3 原生 `ref` + ES module 单例），无需引入新依赖或 store
- 验证：`vue-tsc --noEmit` 本次修改文件（BackToTop.vue / DownloadPanel.vue / useFloatingButtonState.ts）无类型错误

#### Modrinth 整合包安装路径校验修复（resourcepacks 等子目录不存在时误判越出实例目录）
- 背景：安装含 `resourcepacks/` 等非 `mods/` 目录文件的 Modrinth 整合包时报错「文件路径校验失败，越出实例目录: resourcepacks/[1.20.1]ProgrammerArtFix-rv3.1.zip」
- 根因：[src-tauri/src/commands/community/install/modrinth.rs](src-tauri/src/commands/community/install/modrinth.rs) `validate_mr_path` 对 `full.parent()` 调用 `canonicalize()`，但下载前目标子目录（如 `resourcepacks/`）尚未创建，`canonicalize` 返回 `Err` → `unwrap_or_default()` 得空 PathBuf → `starts_with(instance_canonical)` 失败 → 误判为路径穿越
- 修复：改为仅 `canonicalize(instance_dir)`（已存在），然后 `instance_canonical.join(path)` 做组件级 `starts_with` 校验；安全性由前置的 `..` 拦截 + 绝对路径拦截保证，不依赖目标目录是否存在
- 验证：`cargo check` 通过（59.67s 零 warning）

#### 创建房间 room_type 值修复（public → lobby）
- 背景：api-server 校验 `room_type` 仅接受 `lobby` / `private`，前端发送 `public` 导致创建公开房间时报 1001 错误
- 改动：
  - [src/types/online.ts](src/types/online.ts) `CreateRoomParams.roomType` 类型从 `'private' | 'public'` 改为 `'private' | 'lobby'`
  - [src/stores/online.ts](src/stores/online.ts) `hostCreateRoom` 参数类型 + `lobbyId` 条件判断 + 注释从 `'public'` 改为 `'lobby'`
  - [src/components/online/CreateRoomForm.vue](src/components/online/CreateRoomForm.vue) `createForm.roomType` 类型 + `publicRoomHint` computed + `handleCreateRoom` lobbyId 传递 + 模板按钮绑定从 `'public'` 改为 `'lobby'`
  - [src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs) + [src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) 注释从 `"public"` 改为 `"lobby"`（代码无逻辑变更，仅透传字符串）
- 验证：`cargo check` 通过（20.49s 零 warning）

#### 联机侧边栏 keep-alive 状态保留 + HTTP 代理热重建
- 背景：用户反馈「侧边栏菜单切换后组件状态丢失（表单输入 / 搜索结果 / 分页位置都重新加载）」，以及「设置页修改 HTTP 代理后实际请求未走代理（启动时初始化一次后不再更新）」
- 改动：
  - **Online.vue keep-alive 缓存**：[src/views/Online.vue](src/views/Online.vue) 内容区从 `v-if`/`v-else-if`/`v-else` 改为 `<keep-alive>` + `<component :is>`，新增 `currentComponent` / `currentProps` computed；切换侧边栏菜单（设备 ↔ 大厅 ↔ 创建/加入/房间详情）时各面板仅 deactivate → activate，不触发 `onUnmounted`，组件级状态（CreateRoomForm 版本选择 / LobbyBrowser 搜索结果 / RoomManager 加入表单）完整保留；RoomManager 在 create/join/room_details 间切换时复用同一缓存实例，`mode` prop 响应式更新
  - **http.rs 代理热重建**：[src-tauri/src/http.rs](src-tauri/src/http.rs) `static HTTP_CLIENT` 从 `OnceLock<reqwest::Client>` 改为 `RwLock<Option<reqwest::Client>>`；`init_client` 从 `set()`（panic on re-init）改为 `write()` + `*guard = Some(client)`（可重复调用覆盖）；`get_client` 从 `get().cloned()` 改为先 read guard 读快照，未初始化时兜底构建无代理默认客户端
  - **apply_config 代理副作用**：[src-tauri/src/commands/system/apply_config/apply.rs](src-tauri/src/commands/system/apply_config/apply.rs) `apply_proxy` 新增 `proxy_pending: &mut Option<(String, String, String)>` 参数（复用 `log_level_pending` 模式，避免跨 await 持有锁），任一代理字段变更即收集完整三元组；闭包外副作用阶段调用 `crate::http::init_client` 重建客户端 + 打印日志
- 复用：
  - keep-alive 是 Vue 3 内置组件，无需引入新依赖
  - `proxy_pending` 复用 `log_level_pending` 的「闭包内收集 → 闭包外执行副作用」模式，保持代码风格一致
- 验证：`cargo check` 通过（5.81s，零 warning）；`vue-tsc --noEmit` 本次修改文件（Online.vue）无类型错误（其余报错均为项目中已存在的类型问题，与本次改动无关）

#### 创建房间高级设置徽章联动修复 + 整合包勾选 Tooltip 提示 + Select 清空按钮
- 背景：用户反馈「勾选关联整合包后高级设置徽章仍显示未启用」（与白名单勾选行为不一致），「未选 MC 版本时关联整合包复选框 disabled 无反馈，用户不知道为什么点不动」，以及「Select 下拉框缺少清空按钮（ArcoDesign 原版有 allow-clear）」
- 改动：
  - **ModpackSelector 新增 enabled-change 事件**：[src/components/online/ModpackSelector.vue](src/components/online/ModpackSelector.vue) 新增 `enabled-change` emit，勾选状态变化时通知父组件（即使版本无元数据也能反映勾选意图）；`onToggle` 拦截未选版本的情况，强制恢复未勾选状态
  - **Tooltip 替代 disabled**：移除 `<input :disabled="!versionId">`，改为用 `Tooltip` 组件包裹 label，未选版本时灰色 + `tooltipText` = "请先选择 MC 版本"（position="top"），已选版本时为空字符串（Tooltip 不显示）
  - **Tooltip 组件增强**：[src/components/common/Tooltip.vue](src/components/common/Tooltip.vue) `v-if="visible"` 改为 `v-if="visible && text"`，text 为空时不显示 tooltip（合理增强，现有 56 个调用都传非空 text，不受影响）
  - **CreateRoomForm 徽章联动改造**：[src/components/online/CreateRoomForm.vue](src/components/online/CreateRoomForm.vue) 新增 `modpackEnabled` ref + `onModpackEnabledChange` 回调；`advancedBadge` / `advancedBadgeActive` 从监听 `modpackMeta` 改为监听 `modpackEnabled`，与白名单勾选行为一致
  - **Select 组件新增 allow-clear 功能**：[src/components/common/Select.vue](src/components/common/Select.vue) 新增 `allowClear` prop + `clear` emit + `hasValue` / `showClearBtn` computed + `handleClear` 函数；模板在箭头之前插入清空按钮（X 图标），hover 触发器时显示清空按钮并用兄弟选择器隐藏箭头；[src/components/common/Select.css](src/components/common/Select.css) 追加 `.select-clear-btn` 样式（12×12 图标 + 20×20 圆形 hover 背景）
  - **CreateRoomForm MC 版本 Select 启用 allow-clear**：`onVersionSelect` 拦截空值（清空时重置版本字段，不再调用解析接口）
- 复用：
  - 复用项目自定义 `Tooltip.vue` 组件（非原生 `title` 属性），符合前端组件复用约定
  - 复用 `enabled-change` 事件模式，与 `WhitelistEditor` 的 `v-model` 双向绑定风格一致
  - Select 清空按钮实现参考 ArcoDesign Vue `allow-clear`（IconClose SVG + CSS 兄弟选择器隐藏箭头 + `@mousedown.stop.prevent` 阻止冒泡）
- 验证：`vue-tsc --noEmit` 本次修改文件（ModpackSelector.vue / CreateRoomForm.vue / Tooltip.vue / Select.vue）无类型错误；CreateRoomForm.vue 298 行 / Select.vue 218 行 / ModpackSelector.vue 188 行（均未超 300 行项目硬约束）

#### 联机大厅阶段 6.2/6.3：封禁列表查询 + 解封操作
- 背景：阶段 6.1 已实现踢人封禁时长选择，但封禁列表查询与解封操作被 api-server 阻塞（无封禁列表查询端点）。本阶段补齐 api-server 端点 + Tauri IPC + 前端 UI，形成完整的封禁管理闭环
- 改动：
  - **api-server 新增封禁列表端点**：[api-server/src/models/signaling.rs](api-server/src/models/signaling.rs) 新增 `ListBansResponse`（含 `bans` + `server_time`，便于客户端计算剩余封禁时长）；[api-server/src/services/signaling.rs](api-server/src/services/signaling.rs) `list_bans` 改返回 `ListBansResponse`；[api-server/src/controllers/v1/signaling.rs](api-server/src/controllers/v1/signaling.rs) 新增 `GET /v1/signaling/rooms/{code}/bans` 路由 + `list_bans` handler + OpenApi 注册（`RoomBan` / `ListBansResponse` schema）
  - **Tauri 后端 IPC**：[src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs) 新增 `RoomBan` / `ListBansResponse` 类型（`rename_all = "camelCase"` + `alias` 兼容 api-server snake_case 响应）+ `signaling_list_bans` 方法；[src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) 新增 `register_list_bans` 注册 `room_list_bans` IPC action
  - **前端类型 + API 封装**：[src/types/online.ts](src/types/online.ts) 新增 `RoomBan` / `ListBansResponse` 接口；[src/utils/api/online-manager.ts](src/utils/api/online-manager.ts) `ONLINE_ACTIONS` 新增 `ROOM_LIST_BANS` 常量 + `listBannedParticipants(roomCode)` 便捷封装
  - **封禁列表 UI 组件**：新增 [src/components/online/BannedList.vue](src/components/online/BannedList.vue)（100 行）：`Card` 容器 + 刷新按钮（`ArrowPathIcon`）+ 空状态（`LockOpenIcon` + text 垂直水平居中）+ 列表项（devicePk 截断显示 + 封禁类型标签：永久=红色 / 临时=橙色 + 封禁时间）+ 解封按钮（`LockOpenIcon`）
  - **useRoomHost 封禁管理逻辑**：[src/composables/useRoomHost.ts](src/composables/useRoomHost.ts) 新增 `bannedList` / `banServerTime` ref + `refreshBans`（按需刷新：挂载时 + 踢人带封禁后 + 解封后）+ `handleUnban`（调用 `unbanParticipant` + 刷新列表）；`handleKick` 在 `banDuration !== null` 时自动触发 `refreshBans`
  - **RoomHostPanel 集成**：[src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) 解构新增 `bannedList` / `banServerTime` / `handleUnban` / `refreshBans`，模板在参与者列表后追加 `<BannedList>` 组件
- 复用：
  - 复用 `unbanParticipant`（[src/utils/api/online-manager.ts](src/utils/api/online-manager.ts)）现有封装，该函数已支持 `devicePk` 参数
  - 复用 `Card` / `Button` / `Tooltip` 项目自定义组件（非原生 HTML）
  - 复用 `ParticipantList.vue` 的列表项布局模式（devicePk 截断 + 图标按钮）
  - 复用空状态 icon + text 垂直水平居中约定（与 `LobbyBrowser` 一致）
- 验证：`cargo check`（api-server + src-tauri）通过；`vue-tsc --noEmit` 本次修改文件无类型错误；BannedList.vue 100 行 / RoomHostPanel.vue 244 行（均未超 300 行项目硬约束）

#### 联机大厅阶段 6.1：踢人封禁时长选择
- 背景：阶段 5 已完成大厅浏览页，阶段 6 补齐房主踢人时的封禁时长选项。原 `handleKick` 硬编码 `banDurationSeconds=null`（不封禁），用户无法通过 UI 触发"踢出并封禁"
- 改动：
  - **新增踢出确认弹窗组件**：新增 [src/components/online/KickConfirmDialog.vue](src/components/online/KickConfirmDialog.vue)（96 行），`teleport to body` + `transition` 动画 + 标题栏 + 参与者信息 + 封禁时长选项卡片（仅踢出 / 10 分钟 / 1 小时 / 永久，2×2 grid 布局）+ 底部按钮栏（取消 / 确认踢出）
  - **useRoomHost handleKick 改造**：[src/composables/useRoomHost.ts](src/composables/useRoomHost.ts) `handleKick` 签名从 `(participantId, devicePk)` 改为 `(participantId, devicePk, banDuration: number | null)`，移除内置 `showConfirm`（由组件层弹窗替代），toast 文案根据封禁时长动态生成（"已踢出" / "已踢出并永久封禁" / "已踢出并封禁 N 分钟"）
  - **RoomHostPanel 接入弹窗**：[src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) 新增 `kickTarget` ref + `onKick` / `onConfirmKick` / `onCloseKick` 方法，`@kick` 绑定从 `handleKick` 改为 `onKick`，模板追加 `<KickConfirmDialog v-if="kickTarget" />`
- 复用：
  - 复用 `kickParticipant`（[src/utils/api/online-manager.ts](src/utils/api/online-manager.ts)）现有封装，该函数已支持 `banDurationSeconds: number | null` 参数，无需改后端
  - 复用 `teleport` + `transition` 弹窗模式，与 `LobbyJoinConfirmDialog.vue` / `Modal.vue` 风格一致
  - 复用项目自定义 `Button.vue` 组件（非原生 `<button>`）
  - 封禁时长选项卡片使用 `<div role="radio">` 而非 `Button.vue`（语义为单选卡片，非动作按钮）
- 阻塞项：6.2 封禁列表查询 UI + 6.3 解封操作 UI 被 api-server 阻塞 — api-server 仅实现 `POST /v1/signaling/rooms/{code}/unban`，无封禁列表查询端点，需 api-server 补充 `GET /v1/signaling/rooms/{code}/bans` 后再实现
- 验证：`vue-tsc --noEmit` 本次修改文件（KickConfirmDialog.vue / useRoomHost.ts / RoomHostPanel.vue）无类型错误；KickConfirmDialog.vue 96 行 / RoomHostPanel.vue 255 行（均未超 300 行项目硬约束）

#### 联机大厅阶段 5：大厅浏览页
- 背景：阶段 1~4 已完成版本上报、公开/私有开关、整合包元数据上报与加入方展示。阶段 5 实现大厅浏览页，用户可搜索公开房间并一键加入
- 改动：
  - **后端 Lobby 类型定义**：[src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs) 新增 `LobbyListQuery` / `LobbyRoomItem` / `LobbyModpackSummary` / `LobbyListResponse` / `LobbyCategory` / `LobbyCategoriesResponse` 类型（`rename_all = "camelCase"` + `alias` 兼容 snake_case 响应）
  - **后端大厅接口方法**：`OnlineClient` 新增 `signaling_list_lobby_rooms(creds, query)` / `signaling_list_lobby_categories(creds)` 方法，手动拼接 query string（`urlencoding::encode` 编码关键词），调用 `GET /v1/signaling/lobby/rooms` / `GET /v1/signaling/lobby/categories`
  - **后端 IPC 注册**：[src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) 新增 `LobbyListParams` 参数结构体（全字段 `Option` + `Default`）+ `register_list_lobby_rooms` / `register_list_lobby_categories` action 注册（`lobby_list_rooms` / `lobby_list_categories`），参数解析使用 `unwrap_or_default()` 容错空 params
  - **前端类型定义**：[src/types/online.ts](src/types/online.ts) 新增 `LobbyListQuery` / `LobbyRoomItem` / `LobbyModpackSummary` / `LobbyListResponse` / `LobbyCategory` / `LobbyCategoriesResponse` 接口
  - **前端 API 封装**：[src/utils/api/online-manager.ts](src/utils/api/online-manager.ts) `ONLINE_ACTIONS` 新增 `LOBBY_LIST_ROOMS` / `LOBBY_LIST_CATEGORIES` 常量 + `listLobbyRooms(query?)` / `listLobbyCategories()` 便捷封装
  - **前端大厅浏览组件**：新增 [src/components/online/LobbyBrowser.vue](src/components/online/LobbyBrowser.vue)（主组件，189 行）+ [src/components/online/LobbyRoomCard.vue](src/components/online/LobbyRoomCard.vue)（卡片组件，112 行）：搜索框（防抖 400ms）+ 加载器过滤 + 刷新 + 分页（复用 `community/Pagination`）+ 空状态（icon + text 垂直水平居中）+ 加入房间流程
  - **前端侧边栏集成**：[src/views/Online.vue](src/views/Online.vue) 侧边栏新增「大厅」分类（`GlobeAltIcon` 图标，与「房间管理」平级），内容区追加 `<LobbyBrowser v-else-if="activeCategory === 'lobby'" />`
  - **前端加入房间流程**：`LobbyBrowser` 点击卡片「加入」→ 无密码直接加入 / 有密码弹 `showPrompt` 输入 → `store.guestJoinRoom` + `guestWebrtc.fetchOfferAndAnswer`（inject 自 `Online.vue` provide）→ `watch(isInRoom)` 自动跳转房间详情
- 复用：
  - 复用 `call_v1` 现有调用模式，与 `signaling_get_room` 等方法风格一致
  - 复用 `handler!` 宏 + `load_creds` / `make_client` 辅助函数，与现有 19 个 signaling action 注册模式一致
  - 复用 `community/Pagination.vue` 分页组件（0-indexed page + `change` 事件），不重复实现
  - 复用项目自定义 `Input.vue` / `Select.vue` / `Button.vue` / `Tooltip.vue` 组件（非原生 HTML）
  - 复用 `resolveIceServers`（[src/utils/online/webrtc-helpers.ts](src/utils/online/webrtc-helpers.ts)）解析 ICE 服务器列表，与 `RoomManager.vue` 加入流程一致
  - 复用 `showPrompt`（[src/utils/modal.ts](src/utils/modal.ts)）弹密码输入框，不引入新弹窗组件
  - 复用 `formatBytes`（[src/utils/format.ts](src/utils/format.ts)）格式化整合包大小
- 验证：`cargo check` 通过（13.92s）；`vue-tsc --noEmit` 本次修改文件（signaling.rs / signaling_manager.rs / types/online.ts / online-manager.ts / LobbyBrowser.vue / LobbyRoomCard.vue / Online.vue）无类型错误；LobbyBrowser.vue 189 行 / LobbyRoomCard.vue 112 行（均未超 300 行项目硬约束）

#### 联机大厅阶段 5.7：加入确认弹窗内嵌整合包校验
- 背景：阶段 5.1~5.6 + 5.8~5.9 已完成大厅浏览页主体功能，5.7 补齐加入前确认环节。当大厅房间关联了整合包时，加入方点击「加入」按钮后先弹确认窗，内嵌 `ModpackRequirementCard` 供加入方校验本地是否已安装同款整合包，未安装时可一键安装后再加入
- 改动：
  - **新增加入确认弹窗组件**：新增 [src/components/online/LobbyJoinConfirmDialog.vue](src/components/online/LobbyJoinConfirmDialog.vue)（65 行），`teleport to body` + `transition` 动画 + 标题栏 + 内容区（内嵌 `ModpackRequirementCard`）+ 底部按钮栏（取消 / 加入房间），点击遮罩或取消按钮关闭弹窗
  - **LobbyRoomCard emit 改造**：[src/components/online/LobbyRoomCard.vue](src/components/online/LobbyRoomCard.vue) `join` 事件从 `[roomCode, hasPassword]` 改为 `[room: LobbyRoomItem]`，传递完整房间对象，便于父组件判断 `room.modpack`
  - **LobbyBrowser 加入流程扩展**：[src/components/online/LobbyBrowser.vue](src/components/online/LobbyBrowser.vue) 新增 `confirmRoom` ref + `handleJoin(room)` 判断 `room.modpack` 有则弹确认窗（`confirmRoom = room`）无则直接 `proceedJoin`；拆分 `proceedJoin(roomCode, hasPassword)` 处理密码弹窗 / 直接加入；`onConfirmJoin` 关闭弹窗后调用 `proceedJoin` 继续流程；模板追加 `<LobbyJoinConfirmDialog v-if="confirmRoom" />`
- 复用：
  - 复用 `ModpackRequirementCard.vue` 三态展示 + 一键安装能力（阶段 4 已实现），弹窗内零重复实现
  - 复用 `showPrompt`（[src/utils/modal.ts](src/utils/modal.ts)）弹密码输入框，弹窗确认后无缝衔接密码流程
  - 复用项目自定义 `Button.vue` 组件（非原生 `<button>`）
  - 复用 `teleport` + `transition` 弹窗模式，与 `Modal.vue` 风格一致
- 验证：`vue-tsc --noEmit` 本次修改文件（LobbyJoinConfirmDialog.vue / LobbyBrowser.vue / LobbyRoomCard.vue）无类型错误；LobbyJoinConfirmDialog.vue 65 行 / LobbyBrowser.vue 218 行 / LobbyRoomCard.vue 112 行（均未超 300 行项目硬约束）

#### 联机大厅阶段 4：房间详情展示整合包（加入方）
- 背景：阶段 3 已让房主创建房间时上报整合包元数据，加入方拉取房间详情后需展示整合包要求并支持一键安装。安全设计：加入方不接收房主的下载链接，通过现有 IPC 反查平台 API 获取
- 改动：
  - **后端 RoomInfoResponse 扩展**：[src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs) `RoomInfoResponse` 新增 `modpack: Option<ModpackMeta>` / `room_type: String` / `host_loader: Option<String>` / `host_loader_version: Option<String>` 字段（均 `#[serde(default, alias = "...")]` 兼容旧服务器 snake_case 响应）
  - **后端 check_local_modpack IPC**：[src-tauri/src/commands/version/list.rs](src-tauri/src/commands/version/list.rs) 新增 `check_local_modpack(state, manifest_hash, source, project_id, file_id)` 扫描所有已安装版本的 `modpack.meta.json`，优先 `manifest_hash` 匹配，回退 `source + project_id + file_id` 三元组匹配，返回 `CheckLocalModpackResult { installed, version_id }`；[src-tauri/src/utils/version_list_manager.rs](src-tauri/src/utils/version_list_manager.rs) 注册 `check_local_modpack` action，顶部注释同步 18 → 19 个 action
  - **前端类型扩展**：[src/types/online.ts](src/types/online.ts) 新增 `CheckLocalModpackResult` 接口；`RoomInfoResponse` 新增 `roomType?` / `hostLoader?` / `hostLoaderVersion?` / `modpack?: ModpackMeta` 字段
  - **前端 API 封装**：[src/utils/api/version-list-manager.ts](src/utils/api/version-list-manager.ts) `VERSION_LIST_ACTIONS` 新增 `CHECK_LOCAL_MODPACK: 'check_local_modpack'` 常量；[src/utils/api/version.ts](src/utils/api/version.ts) 新增 `checkLocalModpack(manifestHash, source, projectId, fileId)` 封装
  - **前端 store 同步**：[src/stores/online.ts](src/stores/online.ts) `RoomState` 新增 `hostModpack: ModpackMeta | undefined` 字段；`refreshRoomInfo` 从 `RoomInfoResponse.modpack` 同步到 `roomState.hostModpack`；`hostCreateRoom` 创建房间时记录关联的 `modpack`
  - **前端一键安装 composable**：新增 [src/composables/useModpackInstall.ts](src/composables/useModpackInstall.ts) 封装安装流程：平台字符串映射（`curseforge` → `CurseForge` / `modrinth` → `Modrinth`）→ 弹窗询问安装名称 → 跳转下载页 → `getProjectVersions` 反查版本列表 → 按 `fileId` 匹配定位 `ResourceVersion` → `installModpack` → `installMerged` → 失败时 `showModal + finishDownload` 遵循项目统一流程
  - **前端整合包要求卡片**：新增 [src/components/online/ModpackRequirementCard.vue](src/components/online/ModpackRequirementCard.vue) 组件，三态展示（已安装绿色 / 可安装蓝色含一键安装按钮 / 不可安装红色）+ 校验中 + 校验失败含重试；`onMounted` 自动调用 `checkLocalModpack` 校验本地是否已装同款
  - **前端嵌入加入方面板**：[src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) 在「房间信息」与「P2P 连接」之间嵌入 `<ModpackRequirementCard v-if="room.hostModpack" :modpack="room.hostModpack" />`，房主未关联整合包时不渲染
- 复用：
  - 复用 `version_list_manager` dispatcher 注册模式新增 `check_local_modpack` action，与原 8 个 list action 风格一致
  - 复用 `installModpack`（[src/utils/api/community.ts](src/utils/api/community.ts)）/ `installMerged`（[src/utils/api/loader.ts](src/utils/api/loader.ts)）/ `showModal` + `versionStore.startDownload/finishDownload` 现有约定，与 `useDragDrop.runModpackInstall` / `ResourceDetail.handleInstallModpack` 流程一致
  - 复用 `formatBytes`（[src/utils/format.ts](src/utils/format.ts)）格式化整合包文件大小
  - 复用项目自定义 `Button.vue` / `Tooltip.vue` 组件（非原生 `<button>` / `title`）
  - 复用 `ModpackSelector.vue` 的 `sourceLabel` 平台显示名映射逻辑，保持组件间一致
- 验证：`vue-tsc --noEmit` 本次修改涉及的文件（version.ts / version-list-manager.ts / ModpackRequirementCard.vue / useModpackInstall.ts / RoomGuestPanel.vue / types/online.ts / stores/online.ts）无类型错误；ModpackRequirementCard.vue 202 行，RoomGuestPanel.vue 295 行（均未超 300 行项目硬约束）

#### 联机大厅阶段 3：上报整合包元数据
- 背景：创建房间时需关联本地已安装整合包，上报 `modpack` 元数据（不含 `download_url`）给 api-server，供加入方校验本地是否已装同款或一键安装。安全设计：不传输下载链接，加入方通过现有 IPC 反查平台 API 获取
- 改动：
  - **后端 ModpackMeta 结构体**：[src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs) 新增 `ModpackMeta` 结构体（字段与 api-server 一致：`source` / `project_id` / `file_id` / `mc_version` / `modpack_version` / `name` / `loader` / `loader_version` / `file_size` / `file_count` / `manifest_hash`），`CreateRoomRequest` 新增 `modpack: Option<ModpackMeta>` 字段（`skip_serializing_if = "Option::is_none"` 兼容旧客户端）
  - **后端 CreateRoomParams 同步**：[src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) `CreateRoomParams` 新增 `modpack: Option<ModpackMeta>` 字段，`register_create_room` 日志扩展 `modpack` 摘要输出（`source(project_id:file_id)` 或 `none`），映射到 `CreateRoomRequest`
  - **后端 modpack.meta.json 持久化**：[src-tauri/src/minecraft/version/modpack_meta.rs](src-tauri/src/minecraft/version/modpack_meta.rs) 新增 `ModpackMetaFile` 结构体（`ModpackMeta` 字段 + `installed_at` 本地记录），实现 `load` / `save` / `to_signaling_meta` 方法；[src-tauri/src/commands/community/install/modpack.rs](src-tauri/src/commands/community/install/modpack.rs) 整合包安装完成时从 `InstallModpackRequest` 构造并写入 `versions/{id}/modpack.meta.json`
  - **后端 read_local_modpack_meta IPC**：[src-tauri/src/commands/version/list.rs](src-tauri/src/commands/version/list.rs) 新增 `read_local_modpack_meta(state, version_id)` 读取 `modpack.meta.json` 返回 `Option<ModpackMetaFile>`；[src-tauri/src/utils/version_list_manager.rs](src-tauri/src/utils/version_list_manager.rs) 注册 `read_local_modpack_meta` action
  - **前端类型扩展**：[src/types/online.ts](src/types/online.ts) 新增 `ModpackMeta` / `ModpackMetaFile` 接口；`CreateRoomParams` 新增 `modpack?: ModpackMeta` 可选字段
  - **前端 API 封装**：[src/utils/api/version-list-manager.ts](src/utils/api/version-list-manager.ts) 新增 `READ_LOCAL_MODPACK_META` 常量；[src/utils/api/version.ts](src/utils/api/version.ts) 新增 `readLocalModpackMeta(versionId)` 封装
  - **前端 store 透传**：[src/stores/online.ts](src/stores/online.ts) `hostCreateRoom` 新增 `modpack?: ModpackMeta` 参数，透传到 `createRoom` 调用
  - **前端整合包选择器**：新增 [src/components/online/ModpackSelector.vue](src/components/online/ModpackSelector.vue) 组件（复选框开关 + 自动读取 meta + 信息卡展示 + 空状态置灰提示）；[src/components/online/CreateRoomForm.vue](src/components/online/CreateRoomForm.vue) 在「高级设置」折叠卡内引入 ModpackSelector，选中版本后自动读取 `modpack.meta.json`，`handleCreateRoom` 透传 `modpackMeta` 给 `hostCreateRoom`
  - **InstallModpackRequest 扩展**：[src/types/community.ts](src/types/community.ts) `InstallModpackRequest` 新增 `projectId` / `fileId` / `modpackVersion` / `fileSize` / `name` 可选字段；[src/components/community/ResourceDetail.vue](src/components/community/ResourceDetail.vue) `handleInstallModpack` 传入这些字段供后端写入 `modpack.meta.json`
- 复用：
  - 复用 `version_list_manager` 现有 dispatcher 注册模式新增 `read_local_modpack_meta` action，与原 7 个 list action 风格一致
  - 复用 `formatBytes`（[src/utils/format.ts](src/utils/format.ts)）格式化整合包文件大小，不重复实现
  - 复用项目 checkbox 惯例（`accent-primary-500`），与 WhitelistEditor / ExportTab 一致，不引入新 Switch 组件
  - 复用 `CollapsibleCard` 折叠卡承载整合包选择器与白名单，保持「高级设置」单一入口
- 验证：`cargo check` 通过（3.80s，零 warning）；`vue-tsc --noEmit` 本次修改涉及的文件（CreateRoomForm.vue / ModpackSelector.vue / stores/online.ts / types/online.ts）无类型错误；CreateRoomForm.vue 286 行（未超 300 行项目硬约束）

#### 联机大厅阶段 2：公开/私有房间开关
- 背景：阶段 1 拆分 MC 版本上报后，还需让房主选择「仅房间码加入」或「加入大厅」以支持后续阶段 5 的大厅浏览页检索。需新增 `room_type` / `lobby_id` 字段并暴露给客户端
- 改动：
  - **后端 CreateRoomRequest 加字段**：[src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs) `CreateRoomRequest` 新增 `room_type: String`（默认 `private`，`skip_serializing_if = "String::is_empty"` 兼容旧客户端）和 `lobby_id: Option<String>`（公开房间必填，当前固定 `global`）字段。注：`CreateRoomRequest` 仅序列化（不反序列化），故 `default` 属性仅作占位，实际默认值由 `signaling_manager::CreateRoomParams` 反序列化时填充
  - **后端 CreateRoomParams 同步**：[src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) `CreateRoomParams` 新增 `room_type: String`（`#[serde(default = "default_room_type")]`，新增本地 `default_room_type` 函数返回 `"private"`）和 `lobby_id: Option<String>` 字段；`register_create_room` 日志扩展 `room_type` / `lobby_id` 输出，映射到 `CreateRoomRequest`
  - **前端类型扩展**：[src/types/online.ts](src/types/online.ts) `CreateRoomParams` 新增 `roomType?: 'private' | 'public'` 和 `lobbyId?: string` 可选字段
  - **前端 store 透传**：[src/stores/online.ts](src/stores/online.ts) `hostCreateRoom` 新增 `roomType` / `lobbyId` 参数（默认 `private` / `undefined`）；`private` 时 `lobbyId` 强制 `undefined`（后端忽略），`public` 时 `lobbyId` 兜底 `global`
  - **前端表单 UI**：[src/components/online/CreateRoomForm.vue](src/components/online/CreateRoomForm.vue) 在「房间密码」字段下新增「房间类型」行，使用项目自定义 `SegmentedButtons.vue` 组件实现私密/公开二选一切换；切换时实时更新提示文案（公开 → "房间将加入全球大厅，其他玩家可在「大厅浏览」中检索并加入"；私密 → "仅凭房间码加入，不会出现在大厅列表中"）；`handleCreateRoom` 透传 `roomType` + `lobbyId`（公开时 `global`）
- 复用：
  - 复用项目自定义 `SegmentedButtons.vue` 组件（替代原生 `<button>` 或新写 Switch 组件），与 SettingsAdvanced / MemorySection 等现有页面风格一致
  - 复用 `signaling_manager.rs` 现有 `default_max_players` 模式新增 `default_room_type`，命名与位置一致
  - `CreateRoomRequest` 仅序列化，故 `default` 属性保留为占位（不触发），避免引入 `#[allow(dead_code)]`
- 验证：`cargo check` 通过（4.45s，零 warning）；`vue-tsc --noEmit` 本次修改涉及的文件无类型错误；CreateRoomForm.vue 298 行（未超 300 行项目硬约束）

#### 联机大厅阶段 1：拆分 MC 版本上报
- 背景：创建房间时原将 `version_id` 直接塞到 `host_mc_version`（如 `1.20.1-forge-47.3.0`），服务端无法按纯版本号 / 加载器类型 / 加载器版本号做大厅筛选与展示。需拆分为三字段：`host_mc_version`（如 `1.20.1`）/ `host_loader`（如 `forge`）/ `host_loader_version`（如 `47.3.0`）
- 改动：
  - **后端新增 `get_version_loader_info` 函数**：[src-tauri/src/commands/version/list.rs](src-tauri/src/commands/version/list.rs) 新增 `get_version_loader_info(state, version_id)` 读取 `versions/{id}/setup.ini` 的 `Type` 字段和对应 `XxxVersion` 字段，返回 `(loader_type, loader_version)` 元组；setup.ini 不存在时回退读版本 JSON 推断类型，无法推断则兜底 `("release", "")`
  - **后端注册 IPC action**：[src-tauri/src/utils/version_list_manager.rs](src-tauri/src/utils/version_list_manager.rs) 新增 `get_version_loader_info` action 注册，返回 `{ loaderType, loaderVersion }` 给前端；顶部注释同步更新 17 → 18 个 action
  - **后端 CreateRoomRequest 加字段**：[src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs) `CreateRoomRequest` 新增 `host_loader: Option<String>` / `host_loader_version: Option<String>` 字段（均 `skip_serializing_if = "Option::is_none"`，兼容旧客户端）；[src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) `CreateRoomParams` 同步新增字段（`#[serde(default)]`），`register_create_room` 映射并扩展日志输出 `loader` / `loader_version`
  - **前端类型扩展**：[src/types/online.ts](src/types/online.ts) `CreateRoomParams` 新增 `hostLoader?: string` / `hostLoaderVersion?: string` 可选字段
  - **前端 API 封装**：[src/utils/api/version-list-manager.ts](src/utils/api/version-list-manager.ts) `VERSION_LIST_ACTIONS` 新增 `GET_VERSION_LOADER_INFO: 'get_version_loader_info'` 常量；[src/utils/api/version.ts](src/utils/api/version.ts) 新增 `getVersionLoaderInfo(versionId)` 封装返回 `{ loaderType, loaderVersion }`
  - **前端 store 透传**：[src/stores/online.ts](src/stores/online.ts) `hostCreateRoom` 新增 `hostLoader` / `hostLoaderVersion` 参数（默认空字符串），空值转 `undefined` 让后端落库为 NULL
  - **前端表单异步解析**：[src/components/online/CreateRoomForm.vue](src/components/online/CreateRoomForm.vue) `onVersionSelect` 改为 async，并行调用 `getVersionGameVersion` + `getVersionLoaderInfo` 解析三字段；新增 `versionResolving` 状态防止解析期间提交；解析失败兜底 `mcVersion = version_id` / `hostLoader = 'release'`；`handleCreateRoom` 透传 `hostLoader` / `hostLoaderVersion` 给 `hostCreateRoom`
- 复用：
  - 复用 `personalization.ts` 中已有的 `getVersionGameVersion`（获取纯 MC 版本号），不重复实现；`version.ts` 仅新增 `getVersionLoaderInfo`，并在注释中说明 `getVersionGameVersion` 已在 `personalization.ts` 实现，避免 `tauri.ts` re-export 命名冲突
  - 复用 `VersionSetup::load`（[src-tauri/src/minecraft/version/setup/load.rs](src-tauri/src/minecraft/version/setup/load.rs)）已实现的 setup.ini 解析能力，无需重写 ini 读取
  - 复用 `VersionType::detect_from_json` 已有的 JSON 加载器探测逻辑作为 setup.ini 缺失时的兜底
  - 复用 `version_list_manager` 现有 dispatcher 注册模式，新 action 与原 6 个 list action 风格一致
- 验证：`cargo check` 通过（8.80s）；`vue-tsc --noEmit` 本次修改涉及的文件（CreateRoomForm / stores/online / types/online / version.ts / version-list-manager.ts / personalization.ts）无类型错误

#### TUN 管理员重启 dev 模式跳过自动重启
- 背景：`npm run tauri dev` 模式下触发 `restart_as_admin`，后端用 `ShellExecuteW("runas")` 启动 `target/debug/molaunch.exe`，但新进程丢失了 cargo run 注入的 dev 环境变量，导致无法连接 Vite dev server，前端加载失败
- 改动：
  - **后端 dev 模式跳过重启**：[src-tauri/src/utils/tun_manager.rs](src-tauri/src/utils/tun_manager.rs) `register_restart_as_admin` 在 `cfg!(debug_assertions)` 为 true 时直接返回 `{ success: false, dev_mode: true, message: "开发模式下无法自动重启，请用管理员权限的终端运行 npm run tauri dev" }`，不调用 `relaunch_as_admin`，不退出当前进程；release 模式保持原自动重启逻辑
  - **前端类型与提示**：[src/utils/api/online-manager.ts](src/utils/api/online-manager.ts) `restartAsAdmin` 返回类型改为 `RestartAsAdminResult`（新增 `dev_mode?: boolean` 和 `message?: string`）；[src/composables/useVirtualLan.ts](src/composables/useVirtualLan.ts) 调用 `restartAsAdmin` 后检查 `result.dev_mode`，若为 true 则 `showInfo` 提示用户用管理员权限终端启动 `npm run tauri dev`
- 复用：
  - 复用现有 `showInfo` 弹窗工具，无需新增组件
  - 复用 `cfg!(debug_assertions)` 编译期判定，无运行时开销
- 验证：`cargo check` + `vue-tsc --noEmit` 通过

#### NavSidebar 支持 disabled 态 + 联机房间子项互斥灰显
- 背景：用户反馈联机页面侧边栏「房间详情」在未创建/加入房间时也可点击，但点击后无内容可显示，体验混乱。期望未在房间时该项灰色不可点击，进入房间后才可正常使用；同时在房间中时「创建房间」「加入房间」应灰显（必须先退出房间）
- 改动：
  - **NavSidebar 扩展 disabled 字段**：[src/components/common/NavSidebar.vue](src/components/common/NavSidebar.vue) `NavCategory` 接口新增可选 `disabled?: boolean` 字段；父项和子项渲染时根据 disabled 添加 `text-gray-300 cursor-not-allowed` 样式（替换原 `cursor-pointer`）；`handleClick` 在 disabled 时直接 return 不切换；子项 `@click` 改为 `!child.disabled && emit(...)`；URL 恢复逻辑同步检查 disabled，避免刷新页面恢复到不可用项
  - **Online.vue 房间子项动态 disabled**：[src/views/Online.vue](src/views/Online.vue) `roomDetailsChild` 从常量改为 `computed`，`disabled: !isInRoom.value`；`categories` 中「创建房间」「加入房间」子项根据 `isInRoom` 设置 `disabled: inRoom`（在房间时灰显），「房间详情」反之。子项 disabled 规则：未在房间时「创建/加入」可用、「房间详情」灰；在房间中时「创建/加入」灰、「房间详情」可用。`watch(isReady)` 的 URL 恢复逻辑增加 `tab === 'room_details' && isInRoom.value` 校验
- 复用：
  - 复用 NavSidebar 现有 `NavCategory` 接口，仅新增可选字段，向后兼容 VersionSettings / Tools / Settings 等其他使用方
  - 复用现有 `isInRoom` computed，无需新增状态判断
- 验证：`vue-tsc --noEmit` 通过（Online.vue / NavSidebar.vue 无类型错误）

#### online/client.rs 模块化 + call_v1 日志降级
- 背景：用户反馈 `online/client.rs` 超过 500 行（566 行），且 `call_v1` 的 INFO 级别日志过于冗长，每次业务请求都打印 4 行 INFO（开始/响应/业务成功/业务失败），刷屏严重且日志中泄露了 `device_pk` 设备标识
- 改动：
  - **新建 client_types.rs**：[src-tauri/src/minecraft/online/client_types.rs](src-tauri/src/minecraft/online/client_types.rs) 从 `client.rs` 拆出类型定义（`UnifiedResponse` / `JwkKey` / `JwksResponse` / `CsrfResponse` / `TimeResponse` / `BusinessResult` / `ClientError`）和 `jwk_to_pem` 函数（改为 `JwkKey::to_pem()` 方法），118 行
  - **client.rs 瘦身**：[src-tauri/src/minecraft/online/client.rs](src-tauri/src/minecraft/online/client.rs) 删除内联类型定义和 `jwk_to_pem`，改用 `use super::client_types::{...}` 导入；通过 `pub use super::client_types::{BusinessResult, ClientError}` 重导出，`signaling.rs` 等外部模块的 `use super::client::{BusinessResult, ClientError, OnlineClient}` 无需改动。从 566 行降至 461 行
  - **mod.rs 注册新模块**：[src-tauri/src/minecraft/online/mod.rs](src-tauri/src/minecraft/online/mod.rs) 追加 `pub mod client_types;`
  - **call_v1 日志降级**：3 处 `log_info!` → `log_debug!`（call_v1 开始 / 响应状态 / 业务成功），同时移除 `device_pk` 字段避免设备标识泄露到 INFO 日志
- 复用：
  - 类型定义和错误类型原样迁移到 `client_types.rs`，无逻辑变更
  - `JwkKey::to_pem()` 方法内部实现与原 `jwk_to_pem()` 函数完全一致，仅改为方法形式
- 验证：`cargo check` 编译通过

#### 存储结构体移除 Serialize derive（方案 C 防御性加固）
- 背景：`switch_ms_account` 等命令返回 `LocalAuthResult`（已 `#[serde(skip)]` 保护），但底层持久化结构体（`StoredMsAccount` / `StoredAuthlibAccount` / `CurrentUser` / `DeviceCredentials` / `MicrosoftLoginResult` 等）仍派生 `Serialize`，存在被未来误用 `serde_json::to_value` 直接返回前端导致 token/密码/私钥泄露的风险
- 改动（采用方案 C：移除 `Serialize` derive，强制编译期阻止 `to_value` 误用）：
  - **auth/storage/types.rs**：[src-tauri/src/minecraft/auth/storage/types.rs](src-tauri/src/minecraft/auth/storage/types.rs) `StoredMsAccount` / `StoredAuthlibAccount` / `CurrentUser` / `PersistedAuthState` 移除 `Serialize` derive，仅保留 `Deserialize`（从注册表加密 JSON 反序列化）；为 `StoredMsAccount` / `StoredAuthlibAccount` 新增 `to_storage_json()` 方法手动构建包含全部字段（含 token/password）的 JSON 供持久化使用
  - **auth/storage/mod.rs**：[src-tauri/src/minecraft/auth/storage/mod.rs](src-tauri/src/minecraft/auth/storage/mod.rs) `save` 方法中 `to_string(&state.ms_accounts)` / `to_string(&state.authlib_accounts)` 改用 `iter().map(|a| a.to_storage_json()).collect::<Vec<_>>()` 手动序列化，`offline_accounts` 无敏感字段保持原样
  - **minecraft/online/storage.rs**：[src-tauri/src/minecraft/online/storage.rs](src-tauri/src/minecraft/online/storage.rs) `DeviceCredentials` 移除 `Serialize` derive，新增 `to_storage_json()` 方法（含 Ed25519 私钥种子 / X25519 私钥 / device_pk / device_token），`save` 方法改用 `to_string(&creds.to_storage_json())`
  - **auth/microsoft/types.rs**：[src-tauri/src/minecraft/auth/microsoft/types.rs](src-tauri/src/minecraft/auth/microsoft/types.rs) `MicrosoftLoginResult` / `OAuthTokenResponse` / `XblTokenResponse` / `XstsTokenResponse` / `MinecraftLoginResponse` / `DeviceCodeResponse` / `MicrosoftLoginError` 移除 `Serialize` derive（仅内部模块间传递，持久化由 `StoredMsAccount` 接管）；`MinecraftProfile` 保留 `Serialize`（`exchange.rs::login_with_xbl` 用 `to_string(&profile)` 构建 `profile_json`，仅含皮肤披风 URL 不含 token）
  - **community/secure_config.rs**：[src-tauri/src/commands/community/secure_config.rs](src-tauri/src/commands/community/secure_config.rs) `CfConfig` 移除 `Serialize` derive（占位结构体，含 `api_key`）
- 安全保障：
  - 移除 `Serialize` 后，`serde_json::to_value(&stored)` 编译失败，强制开发者使用专用 View 结构体（`MsAccountInfo` / `AuthlibAccountInfo` / `LocalAuthResult` / `DeviceStatus`）返回前端
  - View 结构体的敏感字段已标记 `#[serde(skip)]`（`LocalAuthResult.access_token` / `client_token`、`DeviceStatus.device_pk`、`AuthlibLoginResult::NeedSelect.access_token` / `client_token`）
  - 持久化场景通过 `to_storage_json()` 方法显式构建完整 JSON，功能等价于原 `Serialize` derive，但调用点明确可审计
- 复用：
  - `MsAccountInfo` / `AuthlibAccountInfo` / `LocalAuthResult` / `DeviceStatus` 等 View 结构体已存在，本次无需新增
  - `to_storage_json()` 模式参考 `DeviceCredentials` 已有的 `is_registered()` / `is_token_expired()` 方法风格
- 验证：`cargo check` 编译通过


#### 弹窗 Promise 化修复内存优化强力模式 + 插件卸载确认无响应
- 背景：`showConfirm` 为回调式签名返回 `void`，在 async 函数中被误用为 Promise（`await showConfirm(...)` 立即 resolve），导致内存优化「强力模式」复选框点击后无反应、插件卸载确认弹窗点击后无后续动作
- 改动：
  - **modal.ts 新增 `showConfirmAsync`**：[src/utils/modal.ts](src/utils/modal.ts) 包装 `showConfirm` 为 `Promise<boolean>`，适配 `await` 场景
  - **MemoryOptimizer.vue 修复**：[src/views/quick-tools/MemoryOptimizer.vue](src/views/quick-tools/MemoryOptimizer.vue) 强力模式二次确认改用 `showConfirmAsync`
  - **PluginListSection.vue 修复**：[src/views/settings/plugins/PluginListSection.vue](src/views/settings/plugins/PluginListSection.vue) 卸载确认改用 `showConfirmAsync`
- 验证：`vue-tsc --noEmit` 类型检查通过

#### TUN 虚拟网卡权限不足自动提权重启（PCL2 风格）
- 背景：用户反馈联机创建房间后 TUN 接口创建失败（`os error 5` 拒绝访问），原因是 wintun.dll 创建虚拟网卡需要管理员权限。PCL2 的做法是自动退出程序并以管理员权限重新启动
- 改动：
  - **shell.rs 新增 `is_admin()` + `relaunch_as_admin()`**：[src-tauri/src/minecraft/system/shell.rs](src-tauri/src/minecraft/system/shell.rs) 新增管理员权限检测（Windows: `OpenProcessToken` + `GetTokenInformation(TokenElevation)`）和提权重启（Windows: `ShellExecuteW` with verb `"runas"` 触发 UAC 对话框）。参考 PCL2 `ModBase.RunAsAdmin`（`ProcessStartInfo.Verb = "runas"`）实现
  - **tun_start 检测权限错误**：[src-tauri/src/utils/tun_manager.rs](src-tauri/src/utils/tun_manager.rs) `tun_start` action 在 TUN 创建失败时检测 `os error 5` / `拒绝访问` / `Permission denied`，若非管理员则返回 `TUN_PERMISSION_DENIED:` 前缀错误标记
  - **新增 `restart_as_admin` action**：前端确认后调用，后端 `relaunch_as_admin()` 启动提权进程，延迟 500ms 退出当前进程
  - **前端自动弹确认框**：[src/composables/useVirtualLan.ts](src/composables/useVirtualLan.ts) `start()` 检测 `TUN_PERMISSION_DENIED:` 前缀，调 `showConfirmAsync` 弹出「需要管理员权限」确认框，用户确认后调 `restartAsAdmin()` 触发 UAC 提权重启
  - **Cargo.toml 补 Windows API features**：[src-tauri/Cargo.toml](src-tauri/Cargo.toml) `windows` crate 追加 `Win32_Security`（TokenElevation / TOKEN_QUERY）和 `Win32_UI_Shell`（ShellExecuteW）features
- 设计取舍：
  - **不使用 app.manifest requireAdministrator**：PCL2 通过 manifest 始终以管理员运行，但 MoLaunch 不应强制每次启动都弹 UAC。仅在 TUN 创建实际失败时才请求提权，用户体验更好
  - **前端确认而非后端自动重启**：给用户选择权，避免突然退出程序。用户拒绝 UAC 后可手动切回其他功能
  - **延迟 500ms 退出**：给前端留时间收到 IPC 响应，避免 invoke Promise 未 resolve 就退出导致前端报错
- 复用：
  - `ShellExecuteW` 已在 shell.rs `reveal_in_file_manager` 中使用，本次复用相同的 FFI 模式（`#[link(name = "shell32")]` + `to_wide_null` 辅助函数）
  - `showConfirmAsync` 已在 MemoryOptimizer.vue / PluginListSection.vue 中使用，本次联机模块首次复用
- 验证：`cargo check` 编译通过，`vue-tsc --noEmit` 类型检查无新增错误

#### 联机侧边栏新增「房间详情」菜单项
- 背景：用户反馈创建房间后侧边栏没有「房间详情」入口，无法切换到其他菜单（设备/创建/加入）后再回到房间面板，体验固定不灵活
- 改动：
  - **动态追加「房间详情」子项**：[src/views/Online.vue](src/views/Online.vue) 在 `roomState.role !== null`（房主或加入方）时，动态向「房间管理」分类追加 `room_details` 子项（HomeIcon 图标），离开房间后自动移除
  - **activeCategory 扩展**：从 `'device' | 'create' | 'join'` 扩展为 `'device' | 'create' | 'join' | 'room_details'`，支持 URL `?tab=room_details` 恢复
  - **自动切换**：进入房间时自动跳到 `room_details`；离开房间时若停在 `room_details` 自动切回 `create`
  - **RoomManager mode 映射**：`room_details` 模式下根据 role 映射为 `create`（host）或 `join`（guest），RoomManager 内部已有 role 判断逻辑会自动显示对应面板
- 设计取舍：
  - **子项而非独立分类**：房间详情属于房间管理的子功能，放在 room 分类下逻辑清晰，不增加顶级分类数量
  - **动态追加而非始终存在**：未在房间时不显示房间详情菜单项，避免用户误点进入空白页面
  - **mode 映射而非扩展 RoomManager mode 类型**：RoomManager 已有 role 判断逻辑（host → RoomHostPanel / guest → RoomGuestPanel），mode 仅用于未在房间时显示创建/加入表单。映射后 RoomManager 无需改动
- 验证：`vue-tsc --noEmit` 类型检查通过（Online.vue 无错误）

#### 联机房间挂起 + 白名单 mcsdk- 前缀隐藏
- 背景：用户反馈进入房间后切换侧边栏菜单（设备 ↔ 创建 ↔ 加入）会断开 WebRTC 连接，房间详情无法挂着；白名单列表和输入框直接显示 `mcsdk-xxxx-xxxx-xxxx-xxxx` 前缀，视觉冗余
- 改动：
  - **WebRTC 实例提升到页面级**：[src/views/Online.vue](src/views/Online.vue) 在页面 setup 阶段创建 `hostMesh` / `guestWebrtc` 并 `provide`，实例生命周期绑定到 Online.vue。切换侧边栏菜单时 RoomManager 被 v-if 卸载不会触发 `onUnmounted → close()`，房间连接保持不断
  - **RoomManager.vue 改为 inject**：[src/components/online/RoomManager.vue](src/components/online/RoomManager.vue) 移除本地 `useWebRTCMesh()` / `useWebRTC()` 创建和 `provide`，改为 `inject` 获取 Online.vue 的实例引用。`hostMesh` 仅 RoomHostPanel 自行 inject，RoomManager 不再需要
  - **RoomGuestPanel inject key 修复**：[src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) inject key 从 `'guestWebRTC'`（大写 RTC）修正为 `'guestWebrtc'`（小写 rtc），与 Online.vue 的 provide key 对齐。原不匹配导致 inject 返回 undefined
  - **新建 device-id.ts 工具**：[src/utils/online/device-id.ts](src/utils/online/device-id.ts) 提供 `stripMcsdkPrefix` / `ensureMcsdkPrefix` 两个函数，统一处理 `mcsdk-` 前缀的剥离与补全
  - **WhitelistEditor 前缀隐藏**：[src/components/online/WhitelistEditor.vue](src/components/online/WhitelistEditor.vue) 列表展示用 `stripMcsdkPrefix` 去前缀，添加时用 `ensureMcsdkPrefix` 自动补前缀，placeholder 从 `mcsdk-xxxx-xxxx-xxxx-xxxx` 改为 `xxxx-xxxx-xxxx-xxxx`。内部存储与后端交互始终使用完整前缀
- 设计取舍：
  - **实例提升而非 keep-alive**：`<keep-alive>` 会缓存组件 DOM 但 composable 的 `onUnmounted` 仍会触发，无法保持 WebRTC 连接。提升实例到页面级是最小改动方案
  - **displayEntries 而非直接改 raw 数据**：展示用 `{ raw, display }` 二元组，raw 用于内部操作（移除、提交后端），display 用于 UI 渲染。避免去前缀后丢失原始 ID 导致无法匹配
  - **device-id.ts 放 utils/online/**：与 `webrtc-helpers.ts` / `nat-type.ts` / `crypto.ts` / `protocol.ts` 同级，属于联机模块的工具函数
- 复用：
  - `useWebRTC` / `useWebRTCMesh` composable 未改动，仅调整调用位置
  - `stripMcsdkPrefix` / `ensureMcsdkPrefix` 可被未来其他展示设备 ID 的组件复用（如 OnlineDevicePanel）
- 验证：`vue-tsc --noEmit` 类型检查通过（RoomManager.vue / WhitelistEditor.vue / RoomGuestPanel.vue 无错误）

#### 联机创建房间表单改造（MC 版本下拉 + 白名单折叠 + 布局美化）
- 背景：用户反馈创建房间时 MC 版本需要手动输入字符串，容易出错且无法利用启动器已安装版本列表；白名单在创建表单底部平铺展开，即使不启用也占大量视觉空间；整体布局字段标签过窄、间距紧凑，视觉不舒服
- 改动：
  - **新建 CreateRoomForm.vue**：[src/components/online/CreateRoomForm.vue](src/components/online/CreateRoomForm.vue) 从 RoomManager.vue 拆分创建房间表单为独立组件（234 行，符合 300 行约束）。MC 版本输入从 `<Input>` 改为 [Select.vue](src/components/common/Select.vue) 下拉，数据源复用 `listInstalledVersionsWithType()` IPC（已安装版本列表，含 version_type 标识 forge/fabric/neoforge 等）。选择后用 version_id 作为 mcVersion 上报（含 loader 信息，比单纯版本号更有意义，房主和加入方可直接判断兼容性）
  - **白名单改高级设置折叠**：白名单区块从创建表单底部平铺改为独立 [CollapsibleCard.vue](src/components/common/CollapsibleCard.vue)「高级设置」，默认收起，标题栏显示白名单状态徽章（已启用/未启用），点击展开后显示 WhitelistEditor
  - **RoomManager.vue 精简**：[src/components/online/RoomManager.vue](src/components/online/RoomManager.vue) 移除创建表单相关代码（createForm / whitelistForm / maxPlayersHint / createSteps / handleCreateRoom），引用 CreateRoomForm 组件。精简后从 298 行降至 142 行
  - **布局美化**：字段标签宽度从 w-20（80px）加宽到 w-24（96px）适配「MC 版本」等长标签；字段间距从 space-y-3 调整为 space-y-4；标签增加 `shrink-0` 防止窄屏压缩；白名单独立成卡与基础信息卡片视觉分离
- 设计取舍：
  - **mcVersion 字段填 version_id 而非真实版本号**：如 "1.20.1-forge-47.3.0" 而非 "1.20.1"。理由：version_id 包含 loader 信息，对加入方判断兼容性更有意义；后端 rooms 表的 host_mc_version 字段存 version_id 也合理；避免调用 `getVersionGameVersion` 异步解析的真实版本号丢失 loader 上下文。若后续 api-server 阶段四的 host_loader / host_loader_version 字段需要联机客户端上报，再扩展 IPC 字段
  - **不调用 getVersionGameVersion 解析真实版本号**：减少 IPC 调用，version_id 已包含足够信息
  - **房主运行期面板白名单未同步改造**：本次仅改造创建表单的白名单折叠。房主面板（RoomHostPanel.vue）的白名单是运行期管理，已有独立 Card 包裹和状态徽章，保持现状
- 复用：
  - `listInstalledVersionsWithType()` IPC 早已存在，项目内 7 处 Vue 组件复用（VersionSelect / Home / ModDependencyChecker 等），本次联机模块首次复用
  - `Select.vue` / `CollapsibleCard.vue` / `WhitelistEditor.vue` 均为项目已有公共组件，未引入新依赖
- 验证：`vue-tsc --noEmit` 类型检查通过（RoomManager.vue / CreateRoomForm.vue 无错误，项目其他历史遗留类型错误不在本次修改范围）

#### IPC 敏感信息泄露修复（token / device_pk 序列化隔离）
- 背景：用户反馈 `meta_manager` 的 `switch_ms_account` action 返回的 `LocalAuthResult` 携带 `access_token` / `client_token` 明文，经子 agent 全面排查发现项目内存在 4 处同类泄露点（token 通过 Serialize 结构透传到前端 IPC）
- 改动：
  - **LocalAuthResult**：[src-tauri/src/state/auth.rs](src-tauri/src/state/auth.rs) 给 `access_token` / `client_token` 加 `#[serde(skip)]`，序列化到 IPC 时跳过。`profile_json` 保留（前端 `useSkinOperations` / `AccountCard` 解析微软账号皮肤/披风 URL 用于头像显示，不含 token）。启动游戏时 `build_launch_config` 已直接从后端 `auth_storage` 读取 token 注入启动参数，前端无需访问 token 明文
  - **AuthlibLoginResult::NeedSelect**：[src-tauri/src/commands/auth/authlib.rs](src-tauri/src/commands/auth/authlib.rs) 给 `access_token` / `client_token` 加 `#[serde(skip)]`。前端选定 profile 后调用 `authlib_select_profile`，后端从 `state.authlib_pending`（内存暂存）取出 token 完成 refresh，不依赖前端回传
  - **DeviceStatus**：[src-tauri/src/utils/online_manager.rs](src-tauri/src/utils/online_manager.rs) 给 `device_pk` 加 `#[serde(skip)]`。前端无需自己的 device_pk（房间管理 kick/unban 操作中用到的是其他参与者的 device_pk，来自服务器房间状态而非 DeviceStatus）；后端 `build_login_request` 等内部逻辑直接从 `OnlineStorage` 读取 device_pk，不依赖前端回传
- 设计取舍：
  - **CurseForge API Key 保持明文回显**：CurseForge API Key 是用户自己申请的本地数据，Tauri 桌面应用无 XSS 风险，且 SettingsAdvanced.vue 配置页依赖明文回显到输入框（点眼睛查看）的 UX，经用户确认保持现状
  - **MicrosoftLoginResult 不修改**：该结构仅用于内部持久化（serde_json 序列化后 SDK DES 加密落盘），不直接作为 IPC 返回值；`complete_login` 通过 `to_local_auth` 转换为 `LocalAuthResult` 再返回前端。加 `#[serde(skip)]` 会破坏持久化反序列化（token 字段丢失），故保持现状
  - **profile_json 保留序列化**：含 Mojang 返回的角色属性（皮肤/披风 URL），不含 token；前端 `useSkinOperations` / `AccountCard` 依赖此字段解析头像显示
  - **使用 `#[serde(skip)]` 而非 `#[serde(skip_serializing)]`**：LocalAuthResult / DeviceStatus 主要用于 IPC 返回（序列化），无持久化反序列化场景；`skip` 同时阻止 Deserialize 时读取字段（用 Default），避免未来误用
- 复用：
  - `build_launch_config` 早已实现「从后端 auth_storage 直接读取 token 注入启动参数」的安全模式，本次修复仅需阻断 IPC 序列化路径，无需改造启动链路
  - `state.authlib_pending` 早已实现「多角色登录上下文内存暂存」，`authlib_select_profile` 从中取 token，无需前端回传
- 排查范围：子 agent 全面扫描 17 个 IPC manager 入口及其分发子命令，确认 4 处真实泄露点（CRITICAL × 2 / MEDIUM × 1）+ 1 处协议必要暴露（IceServerEntry.credential / room_key，WebRTC 协议要求保持现状）
- 验证：`cargo check` 通过（9.30s，无警告无错误）

#### 请求日志 req_id 落库修复 + UA 解析增强 + 中间件顺序调整
- 背景：用户反馈 `/v1/admin/rooms` 与 `/v1/admin/devices` 分页参数反序列化报错（`invalid type: string "1", expected u32`），同时 `/v1/admin/logs` 接口按 req_id 查询返回空（数据库里 req_id 字段是空字符串），HTTP 日志文件中 MoLaunch 客户端设备类型显示 `Unknown`（UA 含小写 `windows` 未识别）
- 改动：
  - **中间件顺序调整**：[api-server/src/server/mod.rs](api-server/src/server/mod.rs) 将 `request_id` 中间件从最内层移到最外层（最后添加 layer），调整为 `request_id → request_logger → rate_limit → jwt_guard → handler`。原顺序 `request_logger → rate_limit → request_id` 导致 `request_logger` 在 `next.run(req).await` 之前提取 `req.extensions().get::<RequestId>()` 时 `RequestId` 尚未注入，req_id 字段被写为空字符串
  - **device_pk 提取时机修正**：[api-server/src/middlewares/jwt.rs](api-server/src/middlewares/jwt.rs) `jwt_guard` 在 `next.run(req).await` 之后将 `CurrentDevice` 同时注入到 `response.extensions()`；[api-server/src/middlewares/request_logger.rs](api-server/src/middlewares/request_logger.rs) `request_logger` 将 `device_pk` 提取从 `next.run(req).await` 之前移到之后，从 `response.extensions()` 读取。原实现因 `request_logger` 位于 `jwt_guard` 外层无法访问 `req.extensions()` 中的 `CurrentDevice`，导致 `device_pk` 字段恒为 None
  - **UA 解析 - MoLaunch 客户端**：[api-server/src/utils/ua_parser.rs](api-server/src/utils/ua_parser.rs) `detect_device` 在通用匹配之前识别 `MoLaunch/<os> <version>` 格式（`<os>` 来自 `std::env::consts::OS` 全小写），映射 `windows` / `macos` / `linux` / `ios` / `android` 到标准平台名。原实现仅匹配大写 `Windows` 等关键字，导致 `MoLaunch/windows 0.1.0` 识别为 `Unknown`
  - **UA 解析 - Bot 大小写不敏感**：`is_bot` 改为 `to_lowercase()` 后匹配，补充 `sogou` / `bytespider` / `applebot` / `facebookexternalhit` / `twitterbot` / `linkedinbot` / `telegrambot` / `discordbot` / `whatsapp` 关键字。原实现仅匹配固定大小写关键字，`bingbot`（小写）/ `Sogou web spider` 等漏识别
  - **测试覆盖**：新增 5 个测试用例（`test_molaunch_client` 修正为实际 UA 格式、`test_molaunch_client_macos` / `test_molaunch_client_linux` 跨平台、`test_bingbot_case_insensitive` / `test_baiduspider_case_insensitive` / `test_sogou_spider_case_insensitive` 大小写不敏感验证），共 17 个 UA 解析测试全部通过
- 设计取舍：
  - **request_id 放最外层**：确保所有响应（含被限流/封禁的）都带 `Req-ID` 响应头，且 `request_logger` 能在 `next.run(req).await` 之前从 `req.extensions()` 提取 `RequestId`。代价是 `request_logger` 看不到 `device_pk`（由更内层的 `jwt_guard` 注入），故同步修改 `jwt_guard` 把 `CurrentDevice` 也注入到 `response.extensions()` 供外层提取
  - **response.extensions() 传递 CurrentDevice**：`response.extensions()` 不会被序列化到响应体，不影响客户端；仅占少量内存用于跨中间层数据传递
- 复用：
  - `request_logger` 沿用 `RequestId` / `CurrentDevice` Extension 模式，与既有中间件风格一致
  - `is_bot` 沿用关键字匹配策略，仅扩展关键字列表与大小写不敏感
- 验证：api-server `cargo check --all-targets` 通过；`cargo test --lib utils::ua_parser` 17 个测试全部通过

#### 联机大厅 + 整合包云端共享（阶段四）+ 配置热重载 + 多算法密码 Hash
- 背景：阶段四联机大厅允许房主创建公开房间供其他用户浏览加入；整合包云端共享让加入方在加入前感知「这个房间需要什么整合包」并可一键安装。同时修复配置文件热重载日志显示但实际未生效的问题，扩展 admin_guard 支持多种密码哈希算法
- 改动：
  - **数据模型**：新增 [api-server/migrations/sqlite/012_room_lobby_modpack.sql](api-server/migrations/sqlite/012_room_lobby_modpack.sql) 与 [api-server/migrations/postgres/012_room_lobby_modpack.sql](api-server/migrations/postgres/012_room_lobby_modpack.sql)：`rooms` 表新增 `room_type` / `lobby_id` / `modpack_id` / `host_mc_version` / `host_loader` / `host_loader_version` / `host_mc_port` 7 个字段；新建 `room_modpacks` 表存储整合包元数据（PK `modpack_id` + 唯一索引 `(source, project_id, file_id)`）
  - **整合包元数据（不含 URL）**：[api-server/src/models/signaling.rs](api-server/src/models/signaling.rs) 新增 `ModpackMeta` 结构（`source` / `project_id` / `file_id` / `mc_version` / `modpack_version` / `name` / `loader` / `loader_version` / `file_size` / `file_count` / `manifest_hash`），**故意不包含 `download_url` 字段**；加入方通过本地 IPC `getProjectVersions(platform, project_id)` 反查匹配 `file_id` 的 `ResourceVersion` 自行获取下载链接，避免 api-server 成为 URL 分发中心
  - **房间创建扩展**：[api-server/src/services/signaling.rs](api-server/src/services/signaling.rs) `create_room` 新增 `room_type` 校验（`lobby` / `private`）、`modpack` 元数据校验（`source` 白名单 + 必填字段非空校验）；通过 `upsert_modpack` + `link_room_modpack` 关联房间与整合包（重开房间时复用既有 `modpack_id`）
  - **房间详情扩展**：`get_room_info` 响应新增 `room_type` / `host_mc_version` / `host_loader` / `host_loader_version` / `host_mc_port` / `whitelist_enabled` / `ice_servers` / `modpack` 字段；`modpack_id` 非空时反查 `room_modpacks` 表返回完整元数据
  - **大厅列表**：[api-server/src/repositories/signaling.rs](api-server/src/repositories/signaling.rs) 新增 `list_lobby_rooms` 动态拼装 WHERE 条件（`lobby_id` / `has_modpack` / `loader` / `game_version` / `keyword` 过滤），仅返回 `room_type='lobby'` 且 `status IN ('waiting','active')` 且未过期的房间；[api-server/src/services/signaling.rs](api-server/src/services/signaling.rs) `list_lobby_rooms` 将 `LobbyRoomRow` 映射为 `LobbyRoomItem`，列表页仅返回轻量级 `LobbyModpackSummary`（剔除 `manifest_hash` / `loader_version`）
  - **新接口**：[api-server/src/controllers/v1/signaling.rs](api-server/src/controllers/v1/signaling.rs) 新增 `GET /v1/signaling/lobby/rooms`（分页 + 过滤）与 `GET /v1/signaling/lobby/categories`（MVP 阶段仅 `global`，`room_count` 实时统计）
  - **配置热重载修复**：[api-server/src/middlewares/admin_guard.rs](api-server/src/middlewares/admin_guard.rs) / [api-server/src/middlewares/csrf.rs](api-server/src/middlewares/csrf.rs) / [api-server/src/middlewares/request_logger.rs](api-server/src/middlewares/request_logger.rs) 中间件改为持有 `ConfigStore` 而非启动时快照，每次请求读取最新配置；[api-server/src/config/watcher.rs](api-server/src/config/watcher.rs) 移除 `ReloadHook` 机制，简化为单线程文件监听 + `ArcSwap` 配置原子替换
  - **多算法密码 Hash**：[api-server/src/middlewares/admin_guard.rs](api-server/src/middlewares/admin_guard.rs) 扩展支持 `bcrypt` / `SHA1` / `SHA256` 三种哈希算法，按前缀（`$2` / 40 位十六进制 / 64 位十六进制）自动识别并校验；[api-server/Cargo.toml](api-server/Cargo.toml) 新增 `sha1` 依赖
  - **新错误枚举**：`SignalingError` 新增 `InvalidRoomType` / `InvalidModpackSource` / `InvalidModpackFields` 三类业务错误
- 设计取舍：
  - **不传 download_url**：原始用户反馈要求房主创建房间时不应直接传 URL 等敏感参数。后端仅存储平台 + 项目 ID + 文件 ID + 版本信息，加入方通过本地 IPC 反查 URL，避免 api-server 成为 URL 分发中心，同时规避 URL 时效性问题
  - **整合包元数据独立表**：避免 `rooms` 表过宽；多房间复用同一整合包时通过 `(source, project_id, file_id)` 唯一索引 UPSERT 复用记录，仅更新 `room_code` 关联
  - **list_lobby_rooms 参数 9 个**：数据访问层参数较多但都是必需过滤条件，引入查询结构体会增加耦合与代码量，故用 `#[allow(clippy::too_many_arguments)]` 注解保留现状
  - **大厅分类 MVP 仅 global**：未来扩展地区/语言分类时可从配置或数据库读取，当前阶段保持最小实现
- 复用：
  - `LobbyListQuery` 沿用 `IntoParams` 派生宏，与既有查询参数风格一致
  - `ModpackMeta` / `Room` / `RoomParticipant` 沿用 `#[serde(rename_all = "camelCase")]` 命名约定
  - 大厅接口响应加密复用 `EnvelopeService::seal_unified`，与既有信令接口一致
- 验证：api-server `cargo check --all-targets` 通过；`cargo clippy --all-targets` 仅剩既存代码警告（本次新增代码仅 `list_lobby_rooms` 参数过多 1 项已用 `#[allow]` 修复）；`cargo test --lib --no-run` 测试编译通过；既有 77 个测试无 signaling 相关用例受影响
- 文档同步：[docs/online/design.md](docs/online/design.md) 4.3 路由表补充 7 个新路由、4.5 错误码表补充 6 个新错误；[api-server/docs/signaling.md](api-server/docs/signaling.md) 接口列表补充、创建/查询房间接口字段补充、`rooms` 表新字段与 `room_modpacks` 表说明、新增「联机大厅与整合包云端共享」章节含接口详情与一键安装流程；[docs/online/lobby-modpack-share.md](docs/online/lobby-modpack-share.md) 设计文档已在前序步骤同步

#### 联机模块走查修复：并发安全 + Mutex 死锁 + Vue 行数 + Promise 未处理（5 项）
- 背景：联机模块代码走查发现 5 个问题（3 个并发/Mutex 严重），本次一次性修复
- 改动：
  - **Top 1+2 并发安全**：[api-server/src/services/signaling.rs](api-server/src/services/signaling.rs) `join_room` 将 `next_ip_suffix` 递增、已加入检查、人数上限检查、参与者插入合并到单个数据库事务中。使用 `UPDATE rooms SET next_ip_suffix = next_ip_suffix + 1 ... RETURNING` 原子递增并锁定房间行，PostgreSQL 行锁 / SQLite 库锁串行化同房间并发加入，消除虚拟 IP 重复分配与人数超限竞态
  - **Top 3 Mutex 跨 await**：[src-tauri/src/minecraft/online/bridge.rs](src-tauri/src/minecraft/online/bridge.rs) 将 `forward_from_datachannel` 拆分为同步 `decode_from_datachannel` + `write_tx_clone()`；[src-tauri/src/utils/tun_manager.rs](src-tauri/src/utils/tun_manager.rs) `tun_forward_to` 先在 guard 作用域内 clone `write_tx`，释放 guard 后再跨 await 发送，消除死锁风险
  - **Top 4 Vue 行数**：新增 [src/components/online/VirtualIpCard.vue](src/components/online/VirtualIpCard.vue)（46 行）提取虚拟 IP 显示行（图标 + 标签 + IP 代码块 + 复制按钮，复制逻辑自包含）；[src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) 使用该组件，从 304 行降至 291 行
  - **Top 5 Promise 未处理**：[src/composables/useRoomHost.ts](src/composables/useRoomHost.ts) 为 4 处 fire-and-forget Promise（TURN 广播、importRoomKey、MC 端口广播、listen 注册）加 `.catch()`；[src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) importRoomKey 同步加 catch
- 复用：`VirtualIpCard.vue` 复用 `Button.vue` / `Tooltip.vue` / `toastSuccess` / `toastError`，与 RoomGuestPanel 原 copyText 逻辑一致
- 验证：api-server cargo check 通过，src-tauri cargo check 通过，vue-tsc 0 新增错误（既有错误均在未修改文件），eslint 3 个目标文件通过

#### 联机模块 SFU 拓扑评估：保持 mesh + 限制人数 ≤5（阶段三子任务 9）
- 背景：阶段三子任务 9 评估 mesh 拓扑在 5+ 人时是否需要切换 SFU。基于 Minecraft LAN 流量模型（~100 KB/s）测算，5 人房主上行约 0.4 Mbps（家庭宽带舒适），10 人达 0.9 Mbps（多数家庭宽带扛不住）。结合项目定位（轻量启动器，对标 PCL2，2-5 人开黑为主），决策保持 mesh 拓扑 + 限制人数 ≤5，未来 5+ 人刚需时再评估 SFU
- 改动：
  - 后端 [api-server/config/default.toml](api-server/config/default.toml) `[signaling].max_players` 从 `20` 调整为 `5`（mesh 拓扑压力测算安全边界）
  - 后端 [api-server/src/models/signaling.rs](api-server/src/models/signaling.rs) `CreateRoomRequest.max_players` 字段注释补充默认 5
  - 后端 [api-server/docs/signaling.md](api-server/docs/signaling.md) 创建房间接口 `max_players` 字段范围说明更新
  - 后端 [api-server/docs/admin.md](api-server/docs/admin.md) 房间示例与全局设置 `max_players` 示例值从 20 改为 5
  - 前端 [src/components/online/RoomManager.vue](src/components/online/RoomManager.vue) 创建房间表单校验从 `< 2 || > 20` 收紧到 `< 2 || > 5`；最大人数 Input 新增 `maxPlayersHint` / `maxPlayersHintType` 动态 computed：默认态显示「mesh 模式建议 2-5 人，超过请使用专业服务器」，超出 5 或小于 2 时切换 error 态
  - 前端 [src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) 新增 `totalPlayers` / `nearPlayerLimit` computed，运行期总人数 ≥ `maxPlayers - 1` 时在房间信息卡片底部显示橙色（amber-50/700）预警条 + `ExclamationTriangleIcon` 图标，提示房主「mesh 拓扑下房主上行带宽随人数线性增长，继续邀请可能出现卡顿，建议改用专业服务器」
- 新增文档：
  - [docs/online/sfu-evaluation.md](docs/online/sfu-evaluation.md)（178 行）完整 SFU 评估文档：
    - mesh vs SFU vs MCU 三大拓扑对比
    - 2-15 人带宽/CPU 压力测算表（5 人 0.4 Mbps / 10 人 0.9 Mbps / 15+ 不可用）
    - SFU 候选方案对比（mediasoup / livekit / janus / 云 SFU / 混合拓扑）+ 推荐选择（短期 mesh / 中期 mediasoup / 长期 livekit）
    - 切换 SFU 时的协议变更预览（信令扩展 + composable 改造 + 数据分发层复用）
    - 触发 SFU 切换的 4 个条件（带宽不足 / 产品需求 / 商业化 / 竞争压力）
- 设计取舍：
  - 不引入 SFU 服务端进程：mediasoup/livekit 等需独立进程 + UDP 端口段 + 额外运维，违反 P2P 轻量定位
  - 后端 `max_players` 仍保留为可配置项：未来引入 SFU 时只需放开配置 + 新增 topology 字段，无需破坏性改动
  - 前端预警用 amber 色而非红色：接近上限并非错误，仅提示用户权衡
- 复用：
  - `Input` 组件 `hint` / `hint-type` props 复用项目既有约定（参考 RoomManager 房间码输入框）
  - `ExclamationTriangleIcon` 来自 @heroicons/vue/24/outline，与项目其他预警提示风格一致
- 验证：vue-tsc 0 新增错误，eslint 两个目标文件通过，RoomManager.vue 298 行 / RoomHostPanel.vue 226 行，均符合 300 行约束

#### 联机模块 DataChannel AES-GCM 加密：房主/加入方双向加密 + 透明集成（阶段三子任务 8 安全加强）
- 背景：阶段三子任务 8 安全加强 Part B。后端在创建/加入房间时下发 32 字节 AES-256 密钥（Base64Url 编码存于 `rooms.room_key`），前端在 DataChannel 收发前对完整协议帧（含头部）做 AES-GCM 加解密，保证 P2P 链路即便被中间人嗅探也无法解读 IP 包内容。空字符串密钥表示未启用加密（兼容旧服务器），加密层对业务代码完全透明
- 改动：
  - 新增 [src/utils/online/crypto.ts](src/utils/online/crypto.ts)（162 行）：AES-GCM 加解密工具模块。`importRoomKey(base64Key)` 接受 Base64Url/Base64 编码的 32 字节密钥，导入为 Web Crypto API 的 `CryptoKey`（空字符串/格式非法/长度不匹配时返回 null 并 warn）；`encryptFrame(plaintext, key)` 生成 12 字节随机 IV → AES-GCM 加密 → 返回 `IV || ciphertext+tag`；`decryptFrame(encrypted, key)` 切分 IV 与密文 → AES-GCM 解密，认证失败静默返回 null
  - [src/utils/online/webrtc-helpers.ts](src/utils/online/webrtc-helpers.ts) 新增 `wrapHandlersWithDecrypt(handlers, roomKey)` 共享工具：将业务 `onMessage` 包装为「先解密再回调」版本，`roomKey.value` 为 null 时透传；通过闭包捕获 ShallowRef 运行时读取最新值，支持房间运行期动态注入密钥
  - [src/composables/useWebRTCMesh.ts](src/composables/useWebRTCMesh.ts) 新增 `roomKey: ShallowRef<CryptoKey | null>` 内部状态 + `setRoomKey(key)` 方法；`broadcastPacket(raw)` 与 `sendToParticipant(id, raw)` 改为 `Promise<number>` / `Promise<boolean>` 异步方法（roomKey 非空时先 `encryptFrame` 再发送）；`setDataChannelHandlers` 调用 `wrapHandlersWithDecrypt` 包装业务 onMessage；`close()` 清除 roomKey 避免复用残留
  - [src/composables/useWebRTC.ts](src/composables/useWebRTC.ts) 同步新增 `roomKey` + `setRoomKey` + `wrapHandlersWithDecrypt` 集成；新增 `sendPacket(raw): Promise<boolean>` 方法替代业务侧直接 `channel.send`（roomKey 非空时自动加密）；`close()` 清除 roomKey
  - [src/composables/useRoomHost.ts](src/composables/useRoomHost.ts) `onMounted` 在 `lan.start` 之前 `importRoomKey(store.roomState.roomKey)` + `hostMesh.setRoomKey(key)`；`fetchAndBroadcastTurnServers` 与 `mc-port-detected` 监听器中的 `broadcastPacket` 调用改为 `void ...then(sent => log)` 适配异步 API
  - [src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) `onMounted` 新增 `importRoomKey` + `guestWebrtc.setRoomKey`；`useVirtualLan.onTunPacket` 改用 `guestWebrtc.sendPacket(raw)` 走加密通道（替代直接 `channel.send`）
  - [src/stores/online.ts](src/stores/online.ts) `hostCreateRoom` / `guestJoinRoom` 的 `roomState` 赋值补 `roomKey: data.roomKey ?? ''` 字段（之前 RoomState 接口已声明但未填充）
- 设计取舍：
  - 加密层放在 composable 边界（broadcastPacket/sendPacket/sendToParticipant + setDataChannelHandlers），业务层（useRoomHost / RoomGuestPanel 的 onMessage 回调）完全无感知，无需修改协议帧处理逻辑
  - `broadcastPacket` 改为异步：Web Crypto API 的 `crypto.subtle.encrypt` 是 async-only，必须接受这一约束。3 个调用点（useRoomHost × 2 + useVirtualLan onTunPacket × 1）均改为 `void ...then()` 模式，sent 计数仅用于日志，不阻塞主流程
  - IV 明文发送：AES-GCM 标准做法，IV 不需要保密，只需保证同一密钥下不重复。`crypto.getRandomValues` 提供密码学安全随机数
  - 加密整帧（含头部）：避免元数据泄露（type/seq/length 也可被流量分析），且 GCM 认证标签覆盖整帧防篡改
  - 密钥长度校验在 `importRoomKey` 内完成，32 字节非匹配时返回 null 并降级为透传，避免运行时 `crypto.subtle.importKey` 抛错中断流程
- 复用：
  - Web Crypto API（`crypto.subtle`）为浏览器/Tauri webview 原生能力，不引入第三方加密库
  - `wrapHandlersWithDecrypt` 抽到 `webrtc-helpers.ts` 共享，useWebRTC 与 useWebRTCMesh 复用同一份解密包装逻辑，避免重复实现
  - `ShallowRef<CryptoKey | null>` 闭包捕获模式让运行时动态注入/清除密钥成为可能，无需重建 composable
- 验证：`vue-tsc --noEmit` 本次修改 7 个文件 0 新增错误（项目其他预存错误与本次改动无关）；`eslint` 7 个目标文件全部通过
- 后续：部署 api-server v0.1.10+ 后端，端到端实测加密生效（房主/加入方互通正常 + 抓包验证 DataChannel 流量为加密密文）；如需禁用加密可在 api-server 配置 `room_key` 为空字符串

#### 联机模块房主白名单管理：创建表单 + 运行期管理 + 4 个 IPC action（阶段三子任务 8 安全加强）
- 背景：阶段三子任务 8 安全加强项。房主可启用白名单后指定允许加入的设备（按 `device_id` 友好标识），启用且白名单为空 = 拒绝所有人加入（仅房主可进入），便于私密联机。本次完成启动器侧前端 + Tauri 中间件 + Rust 客户端扩展，与已就绪的 api-server 后端（迁移 008 + 仓库层 + 服务层 + 控制器 + OpenAPI）端到端打通
- 改动：
  - [src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs) 新增 `WhitelistEntry` / `WhitelistResponse` / `AddWhitelistRequest` / `SetWhitelistEnabledRequest` 类型；`CreateRoomRequest` 新增 `whitelist_enabled` + `whitelist` 字段；`RoomInfoResponse` 新增 `whitelist_enabled` 字段；`OnlineClient` 新增 4 个客户端方法 `signaling_list_whitelist` / `signaling_add_whitelist` / `signaling_remove_whitelist` / `signaling_set_whitelist_enabled`
  - [src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) 新增 4 个 IPC action 注册（`room_list_whitelist` / `room_add_whitelist` / `room_remove_whitelist` / `room_set_whitelist_enabled`），含 `AddWhitelistParams` / `RemoveWhitelistParams` / `SetWhitelistEnabledParams` 参数结构体；`CreateRoomParams` 新增 `whitelist_enabled` + `whitelist` 字段
  - [src/types/online.ts](src/types/online.ts) `CreateRoomParams` 新增 `whitelistEnabled` + `whitelist` 字段；`RoomInfoResponse` 新增 `whitelistEnabled` 字段；新增 `WhitelistEntry` / `WhitelistResponse` 类型
  - [src/utils/api/online-manager.ts](src/utils/api/online-manager.ts) 新增 4 个 IPC 封装函数 `listWhitelist` / `addWhitelist` / `removeWhitelist` / `setWhitelistEnabled`；`ONLINE_ACTIONS` 常量追加 4 个 action 名
  - [src/stores/online.ts](src/stores/online.ts) `RoomState` 新增 `whitelistEnabled` 字段；`hostCreateRoom` 方法签名追加 `whitelistEnabled` + `whitelist` 两个可选参数；新增 `whitelistEntries` + `whitelistLoading` 两个状态；新增 4 个 store 方法 `refreshWhitelist` / `addWhitelistEntry` / `removeWhitelistEntry` / `updateWhitelistEnabled`；`refreshRoomInfo` 同步 `whitelistEnabled`；`resetRoomState` 清空 `whitelistEntries`
  - 新增 [src/components/online/WhitelistEditor.vue](src/components/online/WhitelistEditor.vue)（288 行）：白名单编辑器子组件，支持两种模式：`create` 模式纯本地 v-model 双向绑定 `{ enabled, deviceIds }`，`runtime` 模式调用后端 API 实时增删；含启用开关 + 输入框 + 列表 + 增删按钮 + 「启用且为空」拒绝所有人警告 + 空状态 icon+text 垂直水平居中
  - [src/components/online/RoomManager.vue](src/components/online/RoomManager.vue) 创建房间表单新增「白名单」section（`WhitelistEditor` create 模式），`handleCreateRoom` 透传 `whitelistForm.enabled` + `whitelistForm.deviceIds` 到 `hostCreateRoom`
  - [src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) 新增「白名单管理」Card（位于「P2P 连接」与「待确认加入请求」之间），使用 `WhitelistEditor` runtime 模式，Card extra 角标显示当前启用状态
- 复用：
  - 复用项目自定义组件 `Input.vue` / `Button.vue` / `Tooltip.vue` / `Card.vue`，遵循「禁止原生 HTML 控件」约定（checkbox 沿用项目惯例 `accent-primary-500`，与 ExportTab / ArchiveManager 一致）
  - `safeCall` / `toastSuccess` / `toastError` 复用既有工具
  - IPC 沿用 `onlineManager(action, params)` 统一入口 + `ONLINE_ACTIONS` 常量，与既有 13 个信令 action + 1 个 TURN action + 2 个 mesh action + 3 个 TUN action 完全一致
  - 类型定义复用 `BusinessResult<T>` 统一响应包装
- 验证：`vue-tsc --noEmit` 本次修改 6 个文件 0 新增错误（项目其他预存错误与本次改动无关）；`eslint` 6 个目标文件全部通过；行数检查 `WhitelistEditor.vue` 288 行、`RoomHostPanel.vue` 212 行、`RoomManager.vue` 277 行，均符合 300 行约束
- 后续：部署 api-server v0.1.10+ 后端，端到端实测白名单创建/加入/动态增删流程；接入加入方被拒后的 UI 提示（房主添加自己到白名单）

#### 联机模块 TURN 中继支持 阶段 F + G：房主拉取系统 TURN + DataChannel 广播 / 加入方接收 + PC 配置更新（阶段三子任务 7）
- 背景：阶段 E（DataChannel TurnServers 控制消息 0x05 协议层）+ H（用户自定义 TURN 配置 UI）+ I（持久化）+ J（编译验证）已就绪，本次推进阶段 F（房主侧：拉取系统 TURN → 合并用户自定义 → DataChannel 广播）与阶段 G（加入方侧：接收 TurnServers 控制消息 → 更新本地 ICE 配置 → `pc.setConfiguration` 热更新），子任务 7 编码部分全部完成
- 改动：
  - [src/composables/useRoomHost.ts](src/composables/useRoomHost.ts) 新增 `fetchAndBroadcastTurnServers()`：调用 `store.fetchTurnServers` 拉取经服务端三层负载过滤后的系统 TURN → 通过 `buildIceServers({ stunServers, customTurnServers, systemTurnServers })` 合并为统一 ICE 列表 → 更新 `store.roomState.iceServers`（影响后续 `generateOfferForParticipant` 为新参与者生成 PC 时的 ICE 配置）→ `encodeTurnServers(turnSeq++, merged)` 编码 + `hostMesh.broadcastPacket` 下发给所有已联通参与者；新增独立 `turnSeq` 计数器（与 `mcPortSeq` / TUN 数据包 seq 独立避免混淆）；`onMounted` 在 `lan.start` 之后触发一次，房间刚创建时参与者尚未联通 broadcastPacket 返回 0 属正常
  - [src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) `watch(dataChannel)` 的 `onMessage` 新增 `CONTROL_SUBTYPE.TURN_SERVERS` 分支：`decodeTurnServersPayload(msg.payload)` 解析房主广播的 ICE 列表 → 更新 `store.roomState.iceServers` → 调用 `pc.setConfiguration({ iceServers, iceTransportPolicy: 'all' })` 热更新当前 PC 配置（已建立连接需 ICE restart 完全生效，此处仅更新配置不主动 restart，避免中断现有连接）；解析失败或空列表时静默丢弃保持现有 PC 不变
- 设计取舍（不主动重建 PC 的原因）：
  - mesh 拓扑下房主为每个参与者生成 Offer，加入方无法单方面触发重新协商
  - 强制 close + 重新 `fetchOfferAndAnswer` 需要房主配合重新生成 Offer，链路过长且会中断现有数据传输
  - TURN 通常在房间初期下发，PC 已建立时 STUN/TURN 已完成 ICE 收集，更新配置主要为后续 ICE restart（如有）预留
- 复用：
  - `buildIceServers` / `encodeTurnServers` / `decodeTurnServersPayload` 均为阶段 D/E 已抽取的共享工具，未新增重复实现
  - `store.fetchTurnServers` / `store.customTurnServers` 为阶段 C 已预留的 store 方法，本次直接调用
  - `pc.setConfiguration` 为 WebRTC 标准接口，未引入额外封装
- 验证：`vue-tsc --noEmit` 两个目标文件 0 新增错误（项目其他预存错误与本次改动无关）；`eslint` 两个文件全部通过；行数检查 `RoomGuestPanel.vue` 290 行（符合 300 行约束）、`useRoomHost.ts` 330 行（composable 无硬性行数约束）
- 后续：子任务 7 编码部分全部完成，下一步为端到端实测（部署 api-server v0.1.10+，Symmetric NAT 环境下验证 TURN 中转生效，PC ICE 收集包含 relay candidate）

#### 联机模块 TURN 中继支持 阶段 E + H + I：DataChannel TurnServers 协议 + 用户自定义 TURN 配置 UI + 持久化（阶段三子任务 7）
- 背景：阶段 A-D 已完成后端配置/迁移/服务/控制器 + Tauri 中间件 + 前端类型/API/Store + webrtc-helpers 抽取。本次推进阶段 E（DataChannel 协议扩展 TurnServers 0x05 控制消息，前后端对齐）、阶段 H（SettingsOnline 用户自定义 TURN 配置 UI）、阶段 I（启动器配置层 OnlineConfig.custom_turn_servers 持久化），为阶段 F（房主拉取系统 TURN 后广播）与阶段 G（加入方接收后重建 PC）铺路
- 改动：
  - [src-tauri/src/minecraft/online/protocol.rs](src-tauri/src/minecraft/online/protocol.rs) `ControlSubtype` 新增 `TurnServers = 0x05`；新增 `turn_servers_message(seq, json_payload)` 便捷构造函数（payload 为 `IceServerEntry[]` 的 JSON UTF-8 字节）；模块文档补充 TurnServers 帧格式说明；新增 2 个单元测试（roundtrip + 空列表）
  - [src/utils/online/protocol.ts](src/utils/online/protocol.ts) `CONTROL_SUBTYPE` 新增 `TURN_SERVERS: 0x05`；新增 `encodeTurnServers(seq, iceServers)` 编码 `IceServerEntry[]` 为 JSON UTF-8 二进制帧；新增 `decodeTurnServersPayload(payload)` 解析 JSON 为 `IceServerEntry[]`（含字段校验，非法结构返回 null）；模块文档同步
  - [src-tauri/src/state/config.rs](src-tauri/src/state/config.rs) `OnlineConfig` 新增 `custom_turn_servers: Vec<IceServerEntry>` 字段（`#[serde(default)]` 兼容旧配置），`Default` 初始化为空 `Vec`
  - [src-tauri/src/commands/system/apply_config/types.rs](src-tauri/src/commands/system/apply_config/types.rs) `OnlinePatch` 新增 `custom_turn_servers: Option<Vec<IceServerEntry>>`（`#[serde(rename = "onlineCustomTurnServers")]`）；`OnlineSnapshot` 新增 `custom_turn_servers: Vec<IceServerEntry>`；`build_snapshot` 同步克隆
  - [src-tauri/src/commands/system/apply_config/apply.rs](src-tauri/src/commands/system/apply_config/apply.rs) `apply_online` 新增 `custom_turn_servers` 字段更新分支（`Some` 即更新，含空数组表示清空）
  - [src/utils/api/config.ts](src/utils/api/config.ts) `ConfigSnapshot` 新增 `onlineCustomTurnServers: IceServerEntry[]`；`ConfigPatch` 新增 `onlineCustomTurnServers?: IceServerEntry[]`
  - 新增 [src/components/online/TurnServerEntryEditor.vue](src/components/online/TurnServerEntryEditor.vue)（119 行）：单个 TURN 服务器条目编辑器子组件，v-model 双向绑定 `IceServerEntry`，支持 URL/username/credential 三字段编辑 + 移除按钮；URL 输入支持逗号/空白分隔多 URL，校验 `turn:/turns:/stun:` 前缀
  - [src/views/settings/SettingsOnline.vue](src/views/settings/SettingsOnline.vue) 新增「ICE 服务器配置」section：v-for 渲染 `customTurnServers` 列表 + 空状态提示（icon + text 垂直水平居中）+ 添加按钮；通过 `useConfigPage` 的 `onLoad` 从配置加载，`watch` deep + `markDirty('onlineCustomTurnServers', v)` 防抖保存，同时同步到 `onlineStore.setCustomTurnServers(v)` 供房主创建房间时读取
- 复用：
  - 后端 `IceServerEntry` 类型直接复用 `minecraft::online::signaling::IceServerEntry`（已实现 Serialize/Deserialize/Clone），不新增类型
  - 前端 `IceServerEntry` 类型复用 `src/types/online.ts` 既有定义
  - `useConfigPage` composable 复用既有加载 + 防抖保存 + loaded 守卫模式（与 `apiUrl` 一致）
  - `onlineStore.setCustomTurnServers` 复用 store 既有方法（阶段 C 已预留）
  - 表单组件复用 `Input.vue`（含 hint/hintType）+ `Button.vue`（ghost/mini）+ `Tooltip.vue`，遵循项目「禁止原生 HTML 控件」约定
  - 单条目编辑抽到 `TurnServerEntryEditor.vue` 子组件，保证 `SettingsOnline.vue` 296 行不超 300 行硬约束
- 验证：`cargo check -p mo-launch` 通过；`cargo test minecraft::online::protocol` 11 个测试全部通过（含新增 2 个 TurnServers 测试）；`vue-tsc --noEmit` 新增/修改文件 0 错误；`eslint` 4 个目标文件全部通过；行数检查 `SettingsOnline.vue` 296 行、`TurnServerEntryEditor.vue` 119 行，均符合 300 行约束
- 后续：阶段 F（房主 useRoomHost 拉取系统 TURN + 合并用户自定义 + DataChannel 广播）、阶段 G（加入方接收 TurnServers 控制消息后重建 PC）

#### 联机模块 TURN 中继支持 阶段 D：webrtc-helpers 抽取 + useWebRTC/useWebRTCMesh 参数类型重构（阶段三子任务 7）
- 背景：阶段 C 已将前端类型/API/Store 与后端 `ice_servers` 字段对齐，本次推进阶段 D：把 ICE 服务器合并/回退逻辑从 store 抽到 `webrtc-helpers.ts` 共享，`useWebRTC` / `useWebRTCMesh` 的 PC 构造方法从 `stunServers: string[]` 改为 `iceServers: IceServerEntry[]`，让 TURN 凭据能一路传到 `RTCIceServer`，为阶段 E（DataChannel TurnServers 控制消息广播）与阶段 F（房主拉取系统 TURN 后广播）铺路
- 改动：
  - [src/utils/online/webrtc-helpers.ts](src/utils/online/webrtc-helpers.ts) 新增 `stunUrlsToIceServers(urls)` / `resolveIceServers(ice, stun)` / `buildIceServers({ stun, custom, system })` 三个工具函数；`createPeerConnection` 重构为接受 `IceServerEntry[]`，按需展开 `urls` / `username` / `credential` 到 `RTCIceServer`，`iceTransportPolicy` 保持 `'all'`（P2P 优先 + relay 兜底）
  - [src/composables/useWebRTC.ts](src/composables/useWebRTC.ts) `ensurePeerConnection` / `setRemoteOfferAndCreateAnswer` / `fetchOfferAndAnswer` 三个方法参数从 `stunServers: string[]` 改为 `iceServers: IceServerEntry[]`；`detectNatType(stunServers?: string[])` 保持 `string[]`（NAT 探测只用 STUN，无需 TURN 凭据）
  - [src/composables/useWebRTCMesh.ts](src/composables/useWebRTCMesh.ts) `createOfferFor(participantId, iceServers: IceServerEntry[])` 参数类型重构
  - [src/composables/useRoomHost.ts](src/composables/useRoomHost.ts) `generateOfferForParticipant` 改为传 `store.roomState.iceServers`（旧房间回退用 `stunUrlsToIceServers(stunServers)`）
  - [src/components/online/RoomManager.vue](src/components/online/RoomManager.vue) `handleJoinRoom` 用 `resolveIceServers(joinResp.iceServers, joinResp.stunServers)` 解析加入方 ICE 服务器列表后传给 `fetchOfferAndAnswer`
  - [src/stores/online.ts](src/stores/online.ts) 删除本地 `stunUrlsToIceServers` / `resolveIceServers`，改从 `webrtc-helpers` 导入；`hostCreateRoom` 改用 `buildIceServers({ stunServers, customTurnServers })` 合并 ICE 列表
- 复用：未引入新组件；`stunUrlsToIceServers` / `resolveIceServers` / `buildIceServers` 三函数集中放 `webrtc-helpers.ts`，被 store + 两个 composable + 两个调用方共用；`createPeerConnection` 沿用既有 `RTCPeerConnection` 构造，仅扩展字段映射
- 验证：`vue-tsc --noEmit` 联机模块 6 个文件 0 新增错误（项目其他预存错误与本次改动无关）；`eslint` 6 个文件全部通过
- 后续：阶段 E 将扩展 DataChannel 协议增加 TurnServers 控制消息（0x05），房主拉取系统 TURN 后通过 DataChannel 广播给所有参与者

#### 联机模块 TURN 中继支持 阶段 C：前端类型 + API 客户端 + Store（阶段三子任务 7）
- 背景：阶段 A（后端配置/迁移/服务/控制器）与阶段 B（Tauri 中间件）已就绪，本次推进阶段 C，将前端类型/API/Store 与后端 `ice_servers` 字段对齐，并把 STUN+TURN 合并逻辑收敛到 store 层，为阶段 D 的 `buildIceServers` 抽取与 `useWebRTC` 参数类型重构铺路
- 改动：
  - [src/types/online.ts](src/types/online.ts) 新增 `IceServerEntry`（`urls` + 可选 `username`/`credential`，对齐浏览器 `RTCIceServer`）与 `TurnServersResponse`（`servers` + `enabled` + `currentTotalLoad` + `loadThreshold`）；`CreateRoomParams` 增加可选 `iceServers`；`RoomInfoResponse` / `JoinRoomResponse` 增加 `iceServers`（保留 `stunServers` 兼容字段，旧房间回退使用）
  - [src/utils/api/online-manager.ts](src/utils/api/online-manager.ts) `ONLINE_ACTIONS` 追加 `ROOM_GET_TURN`；新增 `getTurnServers(roomCode): Promise<BusinessResult<TurnServersResponse>>` 封装（房主独占）
  - [src/stores/online.ts](src/stores/online.ts) `RoomState` 新增 `iceServers: IceServerEntry[]` 字段；新增 `customTurnServers`（用户自定义，由 SettingsOnline 配置）+ `systemTurnServers`（系统下发快照）两个 ref；`hostCreateRoom` 合并 STUN + customTurn 为 `iceServers` 上报后端；`guestJoinRoom` 用 `resolveIceServers` 优先 iceServers 回退 stunServers；`refreshRoomInfo` 同步 iceServers；新增 `fetchTurnServers`（房主独占，调 `room_get_turn`）+ `setCustomTurnServers` 方法
  - 私有工具函数 `stunUrlsToIceServers(urls)` 与 `resolveIceServers(ice, stun)`：前者用于旧客户端兼容转换，后者用于响应解析时的优先级回退；为阶段 D 抽取到 `webrtc-helpers.ts` 做准备
- 复用：未引入新组件；类型与后端 `IceServerEntry` / `TurnServersResponse` 镜像一致；`getTurnServers` 沿用 `onlineManager` + `ONLINE_ACTIONS` 既有模式（参照 `getStunServers` / `listParticipants`）；`safeCall` / `toastSuccess` 等工具复用项目惯例
- 验证：`vue-tsc --noEmit` 联机模块三个文件 0 新增错误（项目其他预存错误与本次改动无关）；`eslint` 三个文件全部通过
- 后续：阶段 D 将抽取 `buildIceServers` 到 `webrtc-helpers.ts`，并重构 `useWebRTC` / `useWebRTCMesh` 接受 `IceServerEntry[]` 参数

#### 联机模块 Minecraft 服务器绑定 UI 引导（阶段三子任务 6 UI 收尾）
- 背景：阶段三子任务 6 已完成 JVM 参数注入（`-Djava.net.preferIPv4Stack=true`）、GameWatcher 自动捕获 MC LAN 端口、HostMcPort 控制消息广播与接收。本次补齐 UI 层引导提示与复制虚拟 IP 快捷操作，让房主与加入方都能直观知道下一步该做什么
- 改动：
  - [src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) 虚拟 IP 行右侧新增「复制虚拟 IP」按钮（`ghost/mini` + `ClipboardDocumentIcon` + `Tooltip`），点击调用 `navigator.clipboard.writeText` 复制并 toast 提示
  - [src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) P2P 连接卡片内新增蓝色引导提示框（`connectedCount > 0` 时显示）：使用 `InformationCircleIcon` + 文案「已联通，请在 Minecraft 内按 Esc → 「开放给局域网」开关。开放后启动器会自动捕获端口并广播给所有参与者，加入方在「多人游戏 → 直接连接」输入你的虚拟 IP 即可加入」
  - [src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) 「我的虚拟 IP」行右侧新增「复制我的虚拟 IP」按钮（同 `ghost/mini` + `ClipboardDocumentIcon` 风格）
  - [src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) 优化「连接已建立」提示：从单行文字升级为带房主虚拟 IP 高亮 code 块 + 「复制房主虚拟 IP」按钮的两行布局，IP 未就绪时显示「（等待房主广播）」占位且按钮禁用
- 复用：剪贴板写入复用项目惯例 `navigator.clipboard.writeText`（见 `ResourceDetailHeader.vue` / `DeviceCodeModal.vue` / `seedmap/format.ts`）；`Button` + `Tooltip` + `Card` 组件沿用 `ParticipantList.vue` 既有风格；新增 `InformationCircleIcon` 来自 `@heroicons/vue/24/outline`，与项目其他蓝色提示框一致
- 验证：`vue-tsc --noEmit` 两个目标文件 0 新增类型错误（项目其他预存错误与本次改动无关）；`eslint` 两个文件全部通过；行数检查 `RoomHostPanel.vue` 164 行、`RoomGuestPanel.vue` 232 行，均符合 300 行约束

#### 联机模块近期代码走查优化（阶段三子任务 5 收尾）
- 背景：子任务 5 主体与数据分发打通后做一轮代码走查，发现 `bridge.rs` 残留无用字段与过时设计注释、`useWebRTCMesh.closeParticipant` 冗余清理 negotiating 标志、`useVirtualLan.onUnmounted` 冗余清理 unlisten、`RoomHostPanel.vue` 405 行违反 300 行约束
- 改动：
  - [src-tauri/src/minecraft/online/bridge.rs](src-tauri/src/minecraft/online/bridge.rs) 移除 `VirtualLanBridge.seq: Arc<AtomicU32>` 字段与 `AtomicU32`/`Ordering` 导入；seq 计数器下沉为读写循环 task 内的局部 `u32` 变量（`wrapping_add` 自增），协议帧 seq 字段保持不变；清理模块职责注释中不存在的 `start_host_bridge()` 与 line 125-131 的过时设计讨论注释；移除失效的 `test_seq_atomic_counter` 测试
  - [src/composables/useWebRTCMesh.ts](src/composables/useWebRTCMesh.ts) `closeParticipant` 不再清理 `negotiating` 标志 —— 若 closeParticipant 在 `createOfferFor` 进行中被调用，`createOfferFor` 的 finally 块会负责清理；其余场景 negotiating 本就为 false。避免与 `createOfferFor` 的 finally 块重复
  - [src/composables/useVirtualLan.ts](src/composables/useVirtualLan.ts) `onUnmounted` 移除冗余的 `unlisten()` 清理 —— `stop()` 内部已统一处理 `running=true/false` 两种分支的 unlisten 释放；保留 `isMounted = false` 与 `void stop()` 即可
  - 拆分 [src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue)（405 → 156 行）：内联的「待确认 Answer 列表」与「参与者列表」模板替换为已有的 [PendingAnswerList.vue](src/components/online/PendingAnswerList.vue) 与 [ParticipantList.vue](src/components/online/ParticipantList.vue) 子组件；信令轮询、Offer 生成、确认/踢出/关闭房间等全部业务逻辑抽到新增 [src/composables/useRoomHost.ts](src/composables/useRoomHost.ts)（279 行），组件只保留 computed 与 UI 渲染
- 复用：`useRoomHost.ts` 通过参数注入 `hostMesh` 与 `lan` 实例，不重新创建 WebRTC/TUN 资源；子组件 `PendingAnswerList`/`ParticipantList` 早在前次拆分时已建立，本次集成进父组件
- 验证：`cargo check --lib` 通过；`vue-tsc --noEmit` 联机模块 7 个文件 0 新增类型错误；`eslint` 7 个文件全部通过

#### wintun.dll 分发方案落地（阶段三子任务 1 待办项）
- 背景：Windows 平台 `tun-rs` 通过 `libloading` 加载 `wintun.dll`，原方案要求用户手动把 dll 放到可执行文件同目录，便携性差且安装版路径不可写。改为编译时嵌入 + 运行时释放到 AppData 全局目录，实现"开箱即用 + 多实例共享"
- 资源嵌入：[src-tauri/src/resources.rs](src-tauri/src/resources.rs) `embedded_bytes` 注册 `wintun/wintun.dll` 逻辑路径，按 `target_arch` 编译时选择对应架构的物理文件（`x86_64`→amd64、`aarch64`→arm64、`x86`→x86、`arm`→arm），全部用 `#[cfg(target_os = "windows")]` 包裹，非 Windows 平台不嵌入
- 资源释放：新增 `extract_wintun()` 函数（`#[cfg(target_os = "windows")]`），复用 `extract_resource` 的 sha256 校验机制，释放到 `%APPDATA%/.MolaLaunch/wintun.dll`（与 `OnlineStorage::appdata_device_path` 同根目录）；首次释放写 dll + sha256 校验文件，后续启动 hash 一致则跳过，主程序更新后嵌入 dll 变了则自动覆盖
- TUN 接口适配：[src-tauri/src/minecraft/online/tun.rs](src-tauri/src/minecraft/online/tun.rs) `VirtualNet::create` 新增 `wintun_dll_path: Option<&Path>` 参数，Windows 下通过 `DeviceBuilder::with(|b| b.wintun_file(path.clone()))` 显式指定 dll 路径，避免依赖默认 DLL 搜索顺序；非 Windows 平台参数被忽略
- 桥接调用：[src-tauri/src/minecraft/online/bridge.rs](src-tauri/src/minecraft/online/bridge.rs) `VirtualLanBridge::start` 在创建 TUN 接口前调 `crate::resources::extract_wintun()` 拿到释放路径，传入 `VirtualNet::create`；释放失败时回退到默认 DLL 搜索（可执行文件同目录等）
- 资源文件：`src-tauri/resources/wintun/{amd64,arm64,x86,arm}/wintun.dll` 4 个架构版本（来源 https://www.wintun.net/，WireGuard 项目，已签名，未修改字节）
- 验证：`cargo check --lib` 通过
- 后续：实际运行时验证需用户在管理员权限下启动联机模块，观察日志 `wintun.dll 已释放` 与 `TUN 接口已创建`

#### TUN 桥接数据分发打通（阶段三子任务 5 数据分发）
- 背景：前端 mesh 拓扑 WebRTC 主体已就绪，本次将后端 `VirtualLanBridge`（TUN 接口 + 读写循环）与前端 `hostMesh.broadcastPacket` / `guestWebrtc.dataChannel.send` 打通，实现 TUN ↔ DataChannel 双向数据转发
- 后端改动：
  - [src-tauri/src/state/app.rs](src-tauri/src/state/app.rs) `AppState` 新增 `virtual_lan_bridge: Arc<TokioMutex<Option<VirtualLanBridge>>>` 字段，房主与加入方共用同一桥接实例
  - 新增 [src-tauri/src/utils/tun_manager.rs](src-tauri/src/utils/tun_manager.rs) 注册 3 个 IPC action：
    - `tun_start`：若已有 bridge 先停止 → 创建 TUN 接口（绑定 ipv4/prefix_len）→ 启动 select! 读写循环 → 返回接口信息
    - `tun_forward_to`：base64 解码 DataChannel 消息 → 协议帧 decode → 写入 TUN（控制消息跳过）
    - `tun_stop`：abort 读写循环 task + 标记 BridgeState::Closed（幂等）
  - [src-tauri/src/utils/online_manager.rs](src-tauri/src/utils/online_manager.rs) `DISPATCHER` 初始化追加 `register_tun_actions` 注册
  - [src-tauri/src/utils/mod.rs](src-tauri/src/utils/mod.rs) 导出 `tun_manager` 模块
  - 二进制传输约定：Tauri IPC 走 JSON，二进制数据用 base64 字符串传递（`message_base64` 字段），IP 包 MTU 1400 字节 base64 后约 1870 字节
- 前端类型改动：
  - [src/types/online.ts](src/types/online.ts) 新增 `TunStartParams` / `TunStartResponse` / `TunForwardResponse` 类型；导出 `EVENT_TUN_PACKET_OUT` 事件名常量与 `TunPacketPayload` 类型（`number[]`）
- 前端 API 客户端改动：
  - [src/utils/api/online-manager.ts](src/utils/api/online-manager.ts) `ONLINE_ACTIONS` 追加 `TUN_START` / `TUN_FORWARD_TO` / `TUN_STOP`；新增 `tunStart(params)` / `tunForwardTo(dataChannelMessage)` / `tunStop()` 三个便捷封装；`tunForwardTo` 内部分块处理 ArrayBuffer → base64（避免 `apply` 参数上限）
- 前端 composable 改动：
  - 新增 [src/composables/useVirtualLan.ts](src/composables/useVirtualLan.ts) 虚拟网卡桥接 composable：`start(selfVirtualIp, subnet)` 解析 CIDR → 调用 `tunStart` → 订阅 `online://tun-packet-out` 事件；`onTunPacket` 回调由调用方注入（房主调 `hostMesh.broadcastPacket`，加入方调 `dataChannel.send`）；`forwardToTun(raw)` 转发 DataChannel 收到的二进制到后端 TUN；`stop()` 停止桥接；onUnmounted 自动清理监听器；导出 `parsePrefixLen(subnet)` 工具函数
- 前端组件改动：
  - [src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) 注入 `useVirtualLan`，`onTunPacket` 回调调 `hostMesh.broadcastPacket(raw)` 下发所有已联通参与者；`generateOfferForParticipant` 在 `createOfferFor` 后通过 `hostMesh.setDataChannelHandlers` 绑定 `onMessage` → `lan.forwardToTun`（`setupDataChannelHandlers` 仅更新传入字段，不影响默认 onOpen/onClose）；`onMounted` 启动 TUN 桥接；`handleCloseRoom` 改为 `lan.stop` → `hostMesh.close` → `store.hostCloseRoom` 顺序释放
  - [src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) 注入 `useVirtualLan`，`onTunPacket` 回调在 DataChannel readyState='open' 时调 `channel.send(raw)`；`watch(guestWebrtc.dataChannel)` 在 DataChannel 就绪时绑定 `onMessage` → `lan.forwardToTun`（`immediate: true` 处理已就绪场景）；`onMounted` 启动 TUN 桥接；`handleLeaveRoom` 改为 `lan.stop` → `guestWebrtc.close` → `store.guestLeaveRoom` 顺序释放
- 数据流：后端 TUN 读包 → `protocol::encode` → emit `online://tun-packet-out` → 前端 listen → `hostMesh.broadcastPacket` 或 `dataChannel.send`；前端 DataChannel.onmessage → ArrayBuffer → base64 → invoke `tun_forward_to` → 后端 base64 decode → `protocol::decode` → 写入 TUN
- 验证：`cargo check --lib` 通过；`vue-tsc --noEmit` 本次修改的 7 个文件 0 新增类型错误；`eslint` 7 个文件全部通过

#### 前端 mesh 拓扑 WebRTC 整套改造（阶段三子任务 5 前端主体）
- 背景：后端 per-participant Offer 接口与前端 API 客户端打底已完成，本次完成前端 WebRTC 层与 Vue 组件的 mesh 拓扑改造，使房主能为每个参与者维护独立 PeerConnection
- 改动：
  - [src/composables/useWebRTC.ts](src/composables/useWebRTC.ts) 改造为加入方专用：移除 `role` 参数与 host 逻辑（`createOffer` / `setRemoteAnswer`）；复用 `webrtc-helpers.ts` 的 `createPeerConnection` / `collectIceCandidates` / `setupDataChannelHandlers`；新增 `fetchOfferAndAnswer(roomCode, participantId, stunServers)` 轮询方法（默认 2s 间隔、30s 超时），内部循环 `fetchParticipantOffer` 直到 `ready=true` 后调用 `setRemoteOfferAndCreateAnswer`
  - 新增 [src/composables/useWebRTCMesh.ts](src/composables/useWebRTCMesh.ts) 房主多 PC 管理器：内部维护 `Map<participantId, {pc, channel}>`，对外暴露 `createOfferFor` / `setRemoteAnswer` / `broadcastPacket` / `sendToParticipant` / `closeParticipant` / `close`；连接状态与 channel open 状态以 `reactive(Map)` 暴露给 UI；onUnmounted 自动关闭所有 PC
  - [src/components/online/RoomManager.vue](src/components/online/RoomManager.vue) 改用 `useWebRTCMesh` 作为房主实例、`useWebRTC` 作为加入方实例；`handleCreateRoom` 不再调用 `hostWebrtc.createOffer`（mesh 模式下 Offer 改为 per-participant 按需生成），创建步骤精简为 stun + create 两步；`handleJoinRoom` 改用 `guestWebrtc.fetchOfferAndAnswer` 完成协商
  - [src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) inject key 改为 `'hostMesh'`；新增 5s `pollParticipants` 轮询：扫描 `status='joined' && !hostOfferReady` 的参与者 → 并发 `hostMesh.createOfferFor` → `uploadParticipantOffer`；`handleConfirm` 接受连接时调用 `hostMesh.setRemoteAnswer(participantId, ...)`，拒绝时调用 `hostMesh.closeParticipant`；踢出时也调用 `closeParticipant` 释放 PC；P2P 状态卡片改为显示「已联通 / 已确认」计数
  - [src/stores/online.ts](src/stores/online.ts) `RoomCreateStep` 类型收窄为 `'stun' | 'create' | null`（移除不再使用的 `'offer'`）
- 复用：底层 PC 创建 / ICE 收集 / DataChannel 设置全部走 `webrtc-helpers.ts`，房主与加入方两侧零重复实现
- 验证：`vue-tsc --noEmit` 本次修改的 6 个文件 0 新增类型错误；`eslint` 6 个文件全部通过

#### 前端 mesh 拓扑 API 客户端打底（阶段三子任务 5 前端先行）
- 背景：子任务 5 后端已完成 per-participant SDP Offer 接口（`PUT/GET /v1/signaling/rooms/{code}/participants/{participant_id}/offer`），前端需新增对应 action 封装，为后续 `useWebRTCMesh.ts` 与 `Room*Panel.vue` 改造打底
- 后端改动：
  - [src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) 新增 `UploadParticipantOfferParams` + `ParticipantOfferParams` 参数结构体；`register_signaling_actions` 追加 `register_upload_participant_offer` + `register_fetch_participant_offer`，分别注册 `room_upload_participant_offer` 和 `room_fetch_participant_offer` 两个 action，调用 `OnlineClient::signaling_upload_participant_offer` / `signaling_fetch_participant_offer`
- 前端类型改动：
  - [src/types/online.ts](src/types/online.ts) `ParticipantInfo` 增加必填字段 `hostOfferReady: boolean`（房主判断是否需要为本参与者生成 Offer）；新增 `UploadParticipantOfferParams` + `ParticipantOfferResponse` 类型
- 前端 API 客户端改动：
  - [src/utils/api/online-manager.ts](src/utils/api/online-manager.ts) `ONLINE_ACTIONS` 追加 `ROOM_UPLOAD_PARTICIPANT_OFFER` / `ROOM_FETCH_PARTICIPANT_OFFER`；新增 `uploadParticipantOffer(roomCode, participantId, sdpOffer, iceCandidates)` 和 `fetchParticipantOffer(roomCode, participantId)` 便捷封装；更新头部注释
- 验证：`cargo check --lib` 通过；`vue-tsc --noEmit` 本次修改的 3 个文件 0 新增类型错误

#### 联机设备凭证迁移至 AppData 全局目录
- 痛点：[src-tauri/src/minecraft/online/storage.rs](src-tauri/src/minecraft/online/storage.rs) 原将 `device.json` 存在 `<exe_dir>/.Molaunch/online/device.json`（启动器目录下），每个启动器实例独立维护一份设备身份，用户复制/移动启动器目录后需重新注册设备，与 api-server 设备表产生重复条目
- 变更：
  - 新路径：Windows `%APPDATA%/.MolaLaunch/online/device.json`，macOS/Linux `~/.config/MolaLaunch/online/device.json`（命名风格沿用 `personalization.rs` 的 `.MolaLaunch` 惯例，确保跨实例共享设备身份）
  - `OnlineStorage` 不再走 `Storage::instance()` 的便携式目录，新增 `appdata_device_path()` 直接解析 AppData 绝对路径；`legacy_device_path()` 仅用于一次性迁移检测
  - `load` 增加自动迁移逻辑：新路径不存在但旧路径存在 → 原样转写（不重新加解密，保留原 DES 密文）→ 删旧文件 → 加载新路径；写入失败时回退从旧路径直接加载，删除失败时仅 WARN，下次启动再尝试（幂等）
  - `save` 写入新路径后同步清理旧路径文件，避免重复迁移
  - `clear` 同时删除新/旧路径文件，确保注销设备时彻底清除
- 兼容性：保留 `OnlineStorage::new(sdk)` / `clear()` 等公共 API 签名不变，调用方 `online_manager.rs` / `signaling_manager.rs` 无需改动
- 影响：`api-server` 设备表无影响（device_pk 不变，仅本地存储位置变更）
- 验证：`cargo check --lib` 通过；`minecraft::online::storage` 模块 2 个单元测试全部通过

#### 移除顶部导航栏 SDK 就绪状态指示器
- 痛点：[src/components/layout/TopNavLayout.vue](src/components/layout/TopNavLayout.vue) 顶部 nav 右侧显示「就绪 / 加载中」小圆点 + 文字，用户反馈不需要
- 变更：删除该状态指示器（圆点 + 文字 + 包裹容器），并清理未使用的 `useSdkStore` 导入与 `sdkStore` 实例
- 影响：顶部 nav 右侧仅保留窗口控制按钮（最小化 / 最大化 / 关闭）

#### 创建房间增加三步进度反馈
- 痛点：用户反馈点击「创建房间」按钮后界面卡顿数秒无任何提示，等待体验差。根因：[src/components/online/RoomManager.vue](src/components/online/RoomManager.vue) 的 `handleCreateRoom` 串行执行 3 个异步操作（获取 STUN 服务器 → `hostWebrtc.createOffer` 含 ICE 收集最长 5 秒 → 调用后端创建房间），期间仅按钮 `loading` 态无阶段性进度反馈
- 变更：
  - [src/stores/online.ts](src/stores/online.ts) 新增 `RoomCreateStep` 类型（`'stun' | 'offer' | 'create' | null`）与 `roomCreateStep` ref，`hostCreateRoom` 接受可选 `preloadedStun` 参数避免 STUN 重复获取，`finally` 中清空 step
  - [src/components/online/RoomManager.vue](src/components/online/RoomManager.vue) 的 `handleCreateRoom` 在每步前设置 `store.roomCreateStep`，模板在按钮下方渲染三步指示器（当前步 primary 高亮 + spinner，已完成步 green 打勾，未完成步 gray 灰化）
- 复用：项目已有的 `ArrowPathIcon` / `CheckCircleIcon`（heroicons vue outline），无新依赖
- 验证：`npx vue-tsc --noEmit` 本次修改的 3 个文件 0 新增类型错误（项目历史遗留错误与本次修改无关）

#### 修复刷新后侧边栏子菜单激活态丢失
- 痛点：用户反馈选择「房间管理 → 加入房间」子菜单后刷新页面，激活态回到「创建房间」或「设备」。根因：[src/views/Online.vue](src/views/Online.vue) 的 `activeCategory` 初始化为 `'device'`，`watch(isReady)` 在登录成功时直接跳 `'create'`，未读取 URL `?tab=`；NavSidebar 自身 `onMounted` 虽读 `route.query.tab`，但 `categories` 依赖 `isReady`，`refreshStatus` 异步完成前 categories 还不含 room 子项，导致 NavSidebar 校验失败不 emit
- 变更：[src/views/Online.vue](src/views/Online.vue) 引入 `useRoute`，`watch(isReady)` 变 true 时优先从 `route.query.tab` 恢复激活项（仅接受 `'create' | 'join'`），URL 无效才默认跳 `'create'`；变 false 时仍强制切回 `'device'`
- 验证：手动测试路径 — 登录后选「加入房间」→ 刷新页面 → 激活态保留在「加入房间」

#### 房主轮询待确认 Answer 失败时显示实际错误 + 30s 防刷屏
- 痛点：用户反馈房主轮询 list_answers 时全部返回 `code=1002, msg="资源不存在"`，但前端 [src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) 的 `pollAnswers` 在 `result.code !== 1` 时仅 `console.warn`，UI 无任何提示，用户无法感知错误，也无法判断是房间不存在、非房主、网络异常、还是 api-server 走了 fallback
- 根因（双重）：
  - 部署侧：用户当前部署的 api-server 是旧版本（响应体仍含 `http_status` 字段、msg 是 generic "资源不存在"），未包含 `signaling_error_to_envelope` 返回具体 msg（如"房间不存在"/"仅房主可执行此操作"）的改动，也未包含 v1/v3 路由前缀修复
  - 前端侧：`pollAnswers` 业务失败时静默不显示 toast，用户只能从浏览器控制台看到 warn 日志
- 修复：[src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) 的 `pollAnswers` 函数在 `result.code !== 1` 时调用新增的 `maybeToastError` 辅助函数，弹 toast 显示实际 `result.msg`（如"资源不存在"/"房间不存在"/"仅房主可执行此操作"），同时打印包含 `code/msg/req_id` 的详细 warn 日志便于排查；异常路径（网络错误等）同样弹 toast 显示异常 message
- 防刷屏：5s 轮询下连续失败会刷屏，新增 `lastAnswerErrorToastAt` ref + `ANSWER_ERROR_TOAST_INTERVAL = 30_000` 常量，30 秒内只弹一次 toast；成功时重置计时器，下次失败可立即弹
- 注意：本次仅前端治标，**根本解决需更新 api-server 部署到最新版本**（包含 `signaling_error_to_envelope` 返回具体 msg + v1/v3 路由前缀修复 + 移除响应体 `http_status` 字段），更新后前端 toast 将显示具体业务错误而非 generic "资源不存在"
- 验证：`npx eslint src/components/online/RoomHostPanel.vue` 0 错误 0 警告

#### 联机模块 /v1 业务接口补充全链路日志 + 注册接口 RSA-3072 适配 + 默认地址 placeholder 修正
- 痛点一：用户点击注册报错 `Failed to [Online] register device: 构造注册请求失败: RSA 加密失败: message too long`。根因：api-server 默认生成 RSA-2048 密钥，注册 content JSON（含 ed25519_pub + x25519_pub + deviceid + timestamp + nonce，约 209B）超过 RSA-2048 + OAEP-SHA256 的最大明文上限 190B
  - 修复：[api-server/src/server/mod.rs](api-server/src/server/mod.rs) 的 `load_or_generate_rsa_keypair` 函数将生成位数从 2048 改为 3072（RSA-3072 + OAEP-SHA256 最大 318B，足够承载注册 content），警告日志同步改为"自动生成 3072 位密钥对"，并补充注释说明 RSA-2048 不足的原因
  - 同步：[api-server/src/utils/mosign.rs](api-server/src/utils/mosign.rs) 的 `test_rsa_oaep_roundtrip` 测试用例从 2048 改为 3072，与生产保持一致避免误导后续开发者
  - 注意：已存在 RSA 密钥文件不会被自动覆盖（`load_or_generate_rsa_keypair` 检测两文件均存在时直接读取），用户需手动删除 api-server 部署目录下的 RSA 私钥/公钥文件后重启服务，才会重新生成 3072 位密钥
- 痛点二：联机模块 /v1 业务接口（信令房间创建/加入/退出等）调用失败时后端无任何日志，无法定位是网络、加密、解密、业务码、还是 CSRF 问题
  - 修复：[src-tauri/src/minecraft/online/client.rs](src-tauri/src/minecraft/online/client.rs) 的 `OnlineClient::call_v1` 方法补充全链路日志：
    - `log_info!`：调用开始（method/path/是否需 CSRF/是否带 body/device_pk）、HTTP 响应（status/body_len）、业务成功（req_id）
    - `log_debug!`：CSRF 获取成功、请求体加密明文长度 + 加密后 payload/key 长度、响应为加密信封解密中、解密成功明文长度、响应为明文
    - `log_warn!`：CSRF 获取失败、HTTP 非 200、业务失败（code/msg/req_id）、明文响应（401/400/500 走明文路径）
    - `log_error!`：响应 JSON 解析失败
- 痛点三：[src/views/settings/SettingsOnline.vue](src/views/settings/SettingsOnline.vue) 输入框 placeholder 仍显示旧地址 `https://api.molaunch.moteam.top`，与实际默认地址 `https://api.molaunch.moiu.cn`（[src-tauri/src/state/config.rs](src-tauri/src/state/config.rs) `OnlineConfig::default`）不一致，对用户产生误导
  - 修复：placeholder 改为 `https://api.molaunch.moiu.cn`，与 `DEFAULT_API_SERVER_URL` 常量及后端默认值统一
- 验证：`cargo check --lib --manifest-path src-tauri/Cargo.toml` 0 错误 0 警告；`cargo check --manifest-path api-server/Cargo.toml` 0 错误 0 警告

#### 修复信令参数/响应字段命名不一致导致"参数解析失败: missing field `room_code`" + 前端字段访问 undefined
- 痛点：用户点击加入房间，后端返回 `参数解析失败: missing field `room_code``。根因：前端 [src/utils/api/online-manager.ts](src/utils/api/online-manager.ts) 发送 camelCase 参数（`{ roomCode, password }`），但 [src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) 的 Params 结构体（`JoinRoomParams` / `RoomCodeParams` / `CreateRoomParams` / `SubmitAnswerParams` / `ConfirmParams` / `KickParams` / `UnbanParams`）未标注 `#[serde(rename_all = "camelCase")]`，serde 默认按 snake_case 反序列化，导致 `roomCode` 无法映射到 `room_code` 字段
- 关联问题：[src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs) 的 6 个响应类型（`CreateRoomResponse` / `RoomInfoResponse` / `JoinRoomResponse` / `PendingAnswer` / `ParticipantInfo` / `KeepaliveResponse`）同样未标注 `rename_all`，api-server 返回 snake_case（如 `room_code` / `host_virtual_ip`），反序列化虽然能成功（字段名与 Rust 字段名一致），但序列化发给前端时输出 snake_case（`room_code`），而前端 [src/types/online.ts](src/types/online.ts) 期望 camelCase（`roomCode`），导致前端访问 `data.roomCode` 得到 `undefined`
- 变更：
  - [src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs)：7 个 Params 结构体统一添加 `#[serde(rename_all = "camelCase")]`
  - [src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs)：6 个响应类型添加 `#[serde(rename_all = "camelCase")]`（让序列化输出 camelCase 给前端）+ 每个字段添加 `#[serde(alias = "snake_case_name")]`（让反序列化接受 api-server 返回的 snake_case），实现"反序列化兼容 snake_case、序列化输出 camelCase"的双向适配
  - 不修改 `StunServersResponse` / `ListAnswersResponse` / `ListParticipantsResponse`（单字段且为单词，无命名差异）
  - 不修改 `DeviceStatus` / `ServerTimeInfo` / `BusinessResult`（前端 [src/types/online.ts](src/types/online.ts) 已使用 snake_case 定义，与后端一致）
- 复用：serde 官方 `rename_all` + `alias` 组合（项目其他模块如 `system_manager.rs` / `version_list_manager.rs` / `skin_manager.rs` 均采用相同模式，仅 Params 单向 camelCase；本次响应类型因 api-server 返回 snake_case 而额外加 alias）
- 验证：`cargo check --lib` 0 错误 0 警告

#### 修复 vite.config.ts import attributes 语法导致 dev 启动失败
- 痛点：`npm run dev` 启动报错 `Bundling with import attributes is not currently supported`，根因是 [vite.config.ts](vite.config.ts) 第 4 行 `import pkg from './package.json' with { type: 'json' }` 使用了 ES2024 import attributes 语法，当前 esbuild 版本不支持
- 变更：改为普通 import `import pkg from './package.json'`（[tsconfig.json](tsconfig.json) 第 10 行已启用 `resolveJsonModule: true`，esbuild 默认支持 JSON 导入，无需 import attributes）
- 验证：项目内全局搜索 `with { type: 'json' }` 无其他匹配

#### 修复 dev 启动后依赖预构建扫描 code-libs 报错
- 痛点：修复 import attributes 后首次启动 dev 触发 `Re-optimizing dependencies because vite config has changed`，vite 默认扫描项目根目录下所有 .ts/.vue 文件，扫到了 [code-libs/arco-design-vue-main/packages/arco-vue-docs/](code-libs/arco-design-vue-main/packages/arco-vue-docs/) 目录里的 170+ 个文档源码文件，这些文件引用了 `vue-i18n` / `@web-vue/components/*` / `@arco-design/arco-vue-docs-navbar` / `@stackblitz/sdk` 等未安装的依赖，导致依赖预构建报错 `The following dependencies are imported but could not be resolved`
- 根因：`code-libs/arco-design-vue-main` 是 Arco Design Vue 源码副本，仅作查阅参考用（已在 `.gitignore` 第 62 行排除），不应参与 vite 依赖预构建扫描
- 变更：[vite.config.ts](vite.config.ts) 新增 `optimizeDeps.entries: ['index.html', 'src/main.ts']`，显式指定扫描入口为应用真实入口，避免 vite 默认全项目扫描
- 复用：vite 官方 `optimizeDeps.entries` 配置项（无需引入额外插件）
- 验证：项目入口 `src/main.ts` 已确认存在

#### 修复信令 action 未注册导致"未知操作: room_join" + signaling_handler 重命名为 signaling_manager
- 痛点：用户点击加入房间，后端返回 `未知操作: room_join`。根因：[src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs) 中定义了 `register_signaling_actions` 函数（注册 12 个信令 action：`get_stun_servers` / `room_create` / `room_info` / `room_close` / `room_join` / `answer_submit` / `answers_list` / `participant_confirm` / `room_keepalive` / `room_leave` / `participant_kick` / `participant_unban` / `participants_list`），但 [src-tauri/src/utils/online_manager.rs](src-tauri/src/utils/online_manager.rs) 的 DISPATCHER 中**从未调用这个函数**，导致所有信令 action 都没有注册到 dispatcher，前端调用时统一回落到 `未知操作: {action}` 错误分支
- 命名问题：原文件名 `signaling_handler.rs` 不符合项目 `xxx_manager.rs` 命名惯例（utils 目录下 14 个业务模块全部叫 `xxx_manager.rs`，无 `xxx_handler.rs`），重命名为 `signaling_manager.rs`
- 变更：
  - 重命名 `signaling_handler.rs` → `signaling_manager.rs`，文件顶部注释由"信令 action 处理器"改为"信令 action 管理器"，并补充"命名遵循项目 `xxx_manager.rs` 惯例"说明
  - [src-tauri/src/utils/mod.rs](src-tauri/src/utils/mod.rs)：新增 `pub mod signaling_manager;` 模块声明（原文件存在但未在 mod.rs 中声明，导致 `crate::utils::signaling_manager` 路径无法解析）
  - [src-tauri/src/utils/online_manager.rs](src-tauri/src/utils/online_manager.rs)：在 DISPATCHER 的 Lazy 闭包末尾、`d` 返回前，调用 `crate::utils::signaling_manager::register_signaling_actions(&mut d);` 注册全部信令 action
  - [src-tauri/src/minecraft/online/client.rs](src-tauri/src/minecraft/online/client.rs)：`BusinessResult<T>` 加 `Serialize` derive（signaling_manager 中 `serde_json::to_value` 调用需要 `T: Serialize`）
  - [src-tauri/src/minecraft/online/signaling.rs](src-tauri/src/minecraft/online/signaling.rs)：8 个响应/嵌套类型加 `Serialize` derive（CreateRoomResponse / RoomInfoResponse / JoinRoomResponse / PendingAnswer / ListAnswersResponse / ParticipantInfo / ListParticipantsResponse / KeepaliveResponse，原仅 Deserialize）
  - [src-tauri/src/utils/signaling_manager.rs](src-tauri/src/utils/signaling_manager.rs)：清理未使用的导入（`tauri::AppHandle` 和 6 个 Response 类型，仅 `CreateRoomRequest` 在 handler 中实际使用）
- 复用：
  - `signaling_manager::register_signaling_actions` 函数（阶段二已实现，本次只是补上调用）
  - `Dispatcher::register` 机制（与 auth_* action 一致的注册方式）
- 验证：`cargo check --lib` 0 错误 0 警告

#### Input.vue 补充 hint 提示渲染（参考 Arco FormItemMessage）
- 痛点：[src/components/common/Input.vue](src/components/common/Input.vue) 之前已定义 `hint` / `hintType` 两个 props，但 template 完全没有渲染，导致传入 hint 后输入框下方无任何提示显示。用户反馈"这输入框下也没显示错误提示啊"
- 调研：阅读 `code-libs/arco-design-vue-main/packages/web-vue/components/input/input.tsx` 与 `form/form-item-message.vue` + `form/style/index.less`，确认 Arco 原始 Input 组件本身不渲染提示文字，提示统一由 FormItem 的 FormItemMessage 子组件渲染（min-height 20px 防抖动 + form-blink 透明度动画 + 错误色 form-color-tip-text_error）
- 变更：
  - [src/components/common/Input.vue](src/components/common/Input.vue)（365 行，含 200+ 行历史样式；本次净增 6 行）：
    - template 外层包裹 `<span class="input-root">`（display: inline-block; width: 100%），承载原 input-wrapper + 下方提示文字
    - 新增 `<transition name="input-hint">` 包裹的 `<div class="input-hint" :class="`input-hint-${hintType}`" role="alert">` 渲染 hint 文字
    - 新增 `.input-hint` 样式：margin-top 4px、min-height 20px、font-size 12px、line-height 20px（与 Arco form-font-error-text-size 一致）
    - 颜色三态：`.input-hint-error` 红色（#f53f3f Arco red-6）、`.input-hint-success` 绿色（#00b42a Arco green-6）、`.input-hint-default` 灰色（#86909c Arco gray-7）
    - 出现/消失动画：`opacity 0 → 1` 过渡 0.2s ease（参考 Arco form-blink）
  - [src/components/online/RoomManager.vue](src/components/online/RoomManager.vue)（183 行，300 限制内）：
    - 新增 `roomCodeHint` / `roomCodeHintType` 两个 computed，根据 `joinForm.roomCode` 长度动态切换：
      - 空时：default 提示"请输入 6 位房间码（数字 + 大写字母）"
      - 1-5 位：error 提示"还需输入 N 位"
      - 6 位：success 提示"房间码格式正确"
      - 超过 6 位：error 提示"房间码不能超过 6 位"
    - 房间码 Input 绑定 `:hint="roomCodeHint"` 和 `:hint-type="roomCodeHintType"`
    - 房间码行布局由 `items-center` 改为 `items-start`，label 加 `pt-2 shrink-0`，避免 hint 出现时 label 被居中推到 input 与 hint 中间
- 复用：
  - Arco FormItemMessage 的设计模式（min-height 防抖动 + 透明度动画 + 错误色编码）
  - 项目已有的 `hint` / `hintType` props 定义（无需新增 prop）
- 验证：ESLint 对 2 个变更文件 0 错误 0 警告（Input.vue 的 maxlength warning 为历史问题）；vue-tsc 类型检查 2 个文件无错误

#### 联机主页改用侧边栏布局（与设置页一致）+ NavSidebar 支持子菜单
- 痛点：
  1. 联机主页原为纯 web 风格的纵向单列卡片堆叠，与启动器整体风格（[Settings.vue](src/views/Settings.vue) 侧边栏布局）不一致，用户反馈"在纯 web 页面还行，但我这是启动器"
  2. 创建/加入房间原来是 RoomManager 内部的水平 tab，用户希望改成「房间管理」下的子菜单（在最外部的侧边栏上）
  3. 加入房间表单的 `maxlength="6"` 传字符串导致 Vue 警告 `Invalid prop: type check failed`
  4. 房间码输入 123（不足 6 位）也能请求后端，缺少长度校验
- 变更：
  - 扩展 [src/components/common/NavSidebar.vue](src/components/common/NavSidebar.vue)（181 行，300 限制内）支持二级子菜单：
    - `NavCategory` 接口新增可选 `children?: NavCategory[]` 字段（向后兼容，[Settings.vue](src/views/Settings.vue) 等无 children 的调用方行为不变）
    - 父项点击：有 children 则 toggle 展开/收起，无 children 则 emit id
    - 子项点击：emit id；父项高亮条件包含「子项选中」
    - `watch(modelValue)` 自动展开包含选中子项的父项
    - 子菜单展开/收起动画：`grid-template-rows: 0fr → 1fr` + `opacity` + ChevronDown 图标 `rotate-180` 旋转，三者同步 `transition-all duration-200`
  - 重构 [src/views/Online.vue](src/views/Online.vue)（178 行，300 限制内）：改用与 [Settings.vue](src/views/Settings.vue) 一致的 `flex h-full rounded-xl overflow-hidden bg-white shadow-sm` + `NavSidebar` + 右侧标题栏 + 内容区布局
    - 左侧分类：「设备」（始终显示）+「房间管理」（仅 `isReady` 时显示，含 children: 「创建房间」「加入房间」）
    - 右侧顶部：分类 label + desc（子项优先）+ 状态徽章（未注册/需登录/已就绪）+ 联机设置入口
    - 右侧内容区：`OnlineDevicePanel` / `RoomManager(:mode="activeCategory")` 切换
    - 状态联动：`watch(isReady)` 登录成功自动跳到「创建房间」子项，JWT 过期自动切回「设备」
  - 新增 [src/components/online/OnlineDevicePanel.vue](src/components/online/OnlineDevicePanel.vue)（190 行，300 限制内）：从原 [Online.vue](src/views/Online.vue) 拆出的设备面板，包含加载占位 / 注册引导 / 登录卡片 / NAT 类型检测 / 设备信息；NAT 检测直接调用 `detectNatTypeWithStun`（无需 useWebRTC 实例）
  - 重构 [src/components/online/RoomManager.vue](src/components/online/RoomManager.vue)（183 行，300 限制内）：
    - 移除 NAT 检测卡片（已移到 OnlineDevicePanel）
    - 移除内部子菜单（已移到外部 NavSidebar）
    - 新增 `mode: 'create' | 'join'` prop，由父组件传入；role=null 时根据 mode 显示创建/加入表单
    - role=host/guest 时无论 mode 都显示对应面板（保证房间内切换子菜单不丢连接）
    - 修复 `maxlength="6"` → `:maxlength="6"`（字符串改数字，消除 Vue 警告）
    - 房间码校验从「非空」改为「长度===6」，不足 6 位直接 toastError 不请求后端
- 复用：
  - `NavSidebar`：扩展后的公共组件，与 [Settings.vue](src/views/Settings.vue) 共用，自动同步 `?tab=` 到 URL（含子项 id）
  - `Card` / `Button` / `Input` / `Tooltip`：项目自定义组件
  - `useOnlineStore`：与 [RoomManager.vue](src/components/online/RoomManager.vue) 共享 `deviceStatus` 与房间状态
  - `detectNatTypeWithStun` / `NAT_TYPE_META` / `getNatFeasibilityColorClass`：[src/utils/online/nat-type.ts](src/utils/online/nat-type.ts) 工具函数
  - `formatTimestamp`：阶段一新增的公共工具函数
- 验证：ESLint 对 6 个变更文件（NavSidebar.vue / Online.vue / OnlineDevicePanel.vue / RoomManager.vue / RoomHostPanel.vue / RoomGuestPanel.vue）0 错误 0 警告

#### 联机功能阶段二：信令 + WebRTC + NAT 检测 + 房间管理 UI
- 背景：阶段一完成认证 + 路由 + 设置面板后，[Online.vue](src/views/Online.vue) 房间管理位置仍是"功能开发中"占位。阶段二补齐信令 IPC、WebRTC composable、NAT 检测与房主/加入方双面板 UI
- 范围决策：全量二阶段（信令 + WebRTC + 房间管理）；WebRTC PeerConnection 放在前端；本次不做虚拟网卡（仅做 P2P SDP/ICE 交换，不创建虚拟网卡）
- 后端：
  - 新增 [src-tauri/src/utils/signaling_handler.rs](src-tauri/src/utils/signaling_handler.rs)：12 个信令 action 处理器（`get_stun_servers` / `room_create` / `room_info` / `room_close` / `room_join` / `answer_submit` / `answers_list` / `participant_confirm` / `room_keepalive` / `room_leave` / `participant_kick` / `participant_unban` / `participants_list`），每个处理器带 INFO 开始日志 + DEBUG 关键步骤日志 + ERROR 错误分支日志
  - 修改 [src-tauri/src/utils/online_manager.rs](src-tauri/src/utils/online_manager.rs)：在 `handle_action` 中按 action 名分发到 signaling_handler，保持原有认证 action 不变
- 前端类型与 IPC：
  - 修改 [src/types/online.ts](src/types/online.ts)：新增 `BusinessResult<T>`（code/data/msg/time/req_id）、`StunServersResponse`、`CreateRoomParams` / `CreateRoomResponse`、`RoomInfoResponse` / `JoinRoomResponse`、`PendingAnswer` / `ListAnswersResponse`、`ParticipantInfo` / `ListParticipantsResponse` / `KeepaliveResponse`、`NatType` 联合类型与 `NatDetectionResult`
  - 修改 [src/utils/api/online-manager.ts](src/utils/api/online-manager.ts)：新增 12 个信令 IPC wrapper（`getStunServers` / `createRoom` / `getRoomInfo` / `closeRoom` / `joinRoom` / `submitAnswer` / `listAnswers` / `confirmParticipant` / `keepaliveRoom` / `leaveRoom` / `kickParticipant` / `unbanParticipant` / `listParticipants`）
- 前端 WebRTC 与 NAT 检测：
  - 新增 [src/composables/useWebRTC.ts](src/composables/useWebRTC.ts)：WebRTC composable，封装 `RTCPeerConnection` 生命周期，暴露 `createOffer` / `setRemoteOfferAndCreateAnswer` / `setRemoteAnswer` / `detectNatType` / `close` / `connectionState` / `detectingNat` 等
  - 新增 [src/utils/online/nat-type.ts](src/utils/online/nat-type.ts)：NAT 类型检测与 tooltip 元数据
    - `NAT_TYPE_META`：6 种 NAT 类型（Open / FullCone / RestrictedCone / PortRestrictedCone / Symmetric / Blocked / Unknown）的 label、color、feasibility、tooltip 文案，tooltip 说明该类型的联机可行性（高/中/低）与适用场景
    - `detectNatTypeWithStun`：创建临时 PeerConnection（不污染主连接），通过 ICE candidate 类型组合判定 NAT 类型
    - `getNatFeasibilityColorClass`：根据 feasibility 返回 Tailwind 颜色类（high=green、medium=yellow、low=red）
- 前端状态管理与 UI：
  - 修改 [src/stores/online.ts](src/stores/online.ts)：新增 `RoomState` 接口与 `roomState` ref，新增 `hostCreateRoom` / `hostCloseRoom` / `guestJoinRoom` / `guestLeaveRoom` / `refreshParticipants` / `keepalive` / `kickParticipant` 等方法
  - 新增 [src/components/online/RoomManager.vue](src/components/online/RoomManager.vue)（254 行，300 限制内）：房间管理主控制器，根据 `roomState.role` 切换 NAT 检测卡片 + 房主/加入方面板 + 创建/加入房间表单；通过 `provide` 把 `useWebRTC` 实例注入子组件
  - 新增 [src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue)（260 行，300 限制内）：房主面板，包含房间信息（房间码/最大人数/剩余有效期/子网/STUN/MC 版本）、P2P 连接状态、待确认连接列表（接受/拒绝）、参与者管理（踢出/封禁）、5 秒轮询 answers + 10 秒轮询 participants + 5 分钟 keepalive 定时器
  - 新增 [src/components/online/RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue)（156 行，300 限制内）：加入方面板，包含房间信息、P2P 连接状态（已连接时显示"在 MC 中直接连接到房主虚拟 IP"提示）、退出房间
  - 修改 [src/views/Online.vue](src/views/Online.vue)：`isReady` 分支由"功能开发中"占位替换为 `<RoomManager />`
- 复用：
  - `Card` / `Button` / `Input` / `Tooltip`：项目自定义组件（[src/components/common/](src/components/common/)），符合"禁止使用原生 HTML 元素"约定
  - `showConfirm` / `toastSuccess` / `toastError`：全局 Modal/Toast 服务（[src/utils/modal.ts](src/utils/modal.ts)）
  - `useOnlineStore`：与 [Online.vue](src/views/Online.vue) 共享 deviceStatus 与房间状态
  - `formatTimestamp`：阶段一新增的公共工具函数
- 修复：3 个 Room*.vue 文件末尾被误追加的 `</content></invoke>` 标签导致 `vue/no-parsing-error`；[src/components/online/RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) 中 `lastKeepalive` / `lastPollAnswers` ref 只赋值不读取，删除死代码消除 `vue/no-ref-as-operand`
- 验证：ESLint 对 9 个变更文件 0 错误 0 警告；`cargo check --lib` 7.40s，0 警告 0 错误

#### 联机模块日志补全 + 默认 api-server 地址变更 + RSA 长度诊断
- 痛点：
  1. 联机模块后端几乎无日志，注册/登录失败时无法定位问题环节
  2. 默认 api-server 地址需切换到新域名 `api.molaunch.moiu.cn`
  3. 注册报错 `RSA 加密失败: message too long`，根因是 api-server 使用 RSA-2048（最大明文 190B）而注册 content JSON 约 209B，超出限制
- 变更：
  - [src-tauri/src/state/config.rs](src-tauri/src/state/config.rs)：`OnlineConfig::default` 默认地址由 `https://api.molaunch.moteam.top` 改为 `https://api.molaunch.moiu.cn`
  - [src-tauri/resources/defaults/config.ini](src-tauri/resources/defaults/config.ini)：`[Online] api_server_url` 同步更新为新域名
  - [src/views/settings/SettingsOnline.vue](src/views/settings/SettingsOnline.vue)：`DEFAULT_API_SERVER_URL` 常量同步更新
  - [src-tauri/src/minecraft/online/crypto.rs](src-tauri/src/minecraft/online/crypto.rs)：`rsa_oaep_encrypt` 增加 RSA 位数/明文长度/max_allowed 日志（DEBUG），明文超长时返回清晰错误提示（指明需 RSA-3072 或 RSA-4096）
  - [src-tauri/src/minecraft/online/auth.rs](src-tauri/src/minecraft/online/auth.rs)：`build_register_request` / `build_login_request` 增加 content 长度、deviceid/device_pk、nonce 日志（DEBUG）
  - [src-tauri/src/minecraft/online/client.rs](src-tauri/src/minecraft/online/client.rs)：
    - `get_server_time` / `get_jwks` / `register` / `login` / `logout` 五个 HTTP 方法全部增加请求发起日志（DEBUG/INFO）、响应状态码+body 长度日志、非 200/异常分支 WARN/ERROR 日志
    - `fetch_server_rsa_pem` 从 JWK `n` 字段直接计算 RSA 位数并日志输出（INFO），位数 < 3072 时主动 WARN 提示将触发 "message too long"
  - [src-tauri/src/utils/online_manager.rs](src-tauri/src/utils/online_manager.rs)：6 个 action 处理器全部补充开始日志（INFO）、关键步骤日志（DEBUG）、错误分支 ERROR 日志，便于从日志即可定位失败环节
- RSA 问题结论：客户端无法绕过（协议规定 content 用 RSA-OAEP 加密），**需在 api-server 端重新生成 RSA-3072 或 RSA-4096 密钥**（修改 [api-server/examples/generate_keys.rs](api-server/examples/generate_keys.rs) 的 `RSA_BITS` 后重新执行）
- api-server 端修复：[api-server/examples/generate_keys.rs](api-server/examples/generate_keys.rs) `RSA_BITS` 由 `2048` 改为 `3072`，注释说明位数选择理由（注册 content 约 209B，RSA-3072 + OAEP-SHA256 最大 318B 足够）。**部署步骤**：在 api-server 目录执行 `cargo run --example generate_keys` 重新生成 `config/keys/private.pem` 与 `public.pem`，重启服务后注册接口可正常加密
- 验证：`cargo check --lib` 4.66s，0 警告 0 错误

#### 联机功能阶段一：前端接入（路由 + 顶导 + 设置 Tab）
- 背景：联机功能后端骨架（MoSign-v1 认证 + `online_manager` IPC + `OnlineConfig`）已就绪，[Online.vue](src/views/Online.vue) 主页骨架已创建，但未接入路由/导航/设置，且 [Online.vue](src/views/Online.vue) 引用的 `formatTimestamp` 在 [src/utils/format.ts](src/utils/format.ts) 中不存在
- 变更：
  - [src/utils/format.ts](src/utils/format.ts)：新增 `formatTimestamp(unixSeconds)` 函数，格式化 Unix 秒为 `YYYY-MM-DD HH:mm:ss`（本地时区），修复 [Online.vue](src/views/Online.vue) 的断裂导入
  - [src/router/index.ts](src/router/index.ts)：注册 `/apps/online` 路由，懒加载 [Online.vue](src/views/Online.vue)
  - [src/components/layout/TopNavLayout.vue](src/components/layout/TopNavLayout.vue)：顶导菜单在「下载」与「工具」之间新增「联机」入口（`UserGroupIcon`）
  - [src/views/Settings.vue](src/views/Settings.vue)：侧栏 `baseCategories` 在「进阶设置」之后新增「联机」分类（`GlobeAltIcon`），并接入 `SettingsOnline` 组件；`?tab=online` 跳转由现有 [NavSidebar.vue](src/components/common/NavSidebar.vue) 自动处理（无需额外代码）
  - 新增 [src/views/settings/SettingsOnline.vue](src/views/settings/SettingsOnline.vue)（248 行，300 限制内）：
    - api-server 地址输入（防抖 800ms 自动保存，走统一 `applyConfig`，复用 `useConfigPage`）
    - 重置为默认按钮（一键还原 `https://api.molaunch.moteam.top`）
    - 测试连通性按钮（调用 `auth_get_server_time`，展示服务器时间/时区/UTC 偏移/本地时间差）
    - 设备登出 / 清除凭证按钮（复用 `useOnlineStore`，清除前经 `showConfirm` 全局 Modal 二次确认）
- 复用：
  - `useConfigPage`：与 [SettingsAdvanced.vue](src/views/settings/SettingsAdvanced.vue) 一致的加载 + 防抖保存 + `loaded` 守卫
  - `useOnlineStore`：与 [Online.vue](src/views/Online.vue) 共享 `deviceStatus` 与 `logout/clear` 状态
  - `showConfirm`：全局 Modal 服务（[src/utils/modal.ts](src/utils/modal.ts)），替代 `window.confirm`
  - `formatTimestamp`：新增的公共工具函数，供 [Online.vue](src/views/Online.vue) 与 [SettingsOnline.vue](src/views/settings/SettingsOnline.vue) 共用
- 验证：ESLint 对 6 个变更文件 0 错误 0 警告

#### 修复联机后端模块 6 处编译警告
- 痛点：`cargo check --lib` 报 6 处警告（unused imports × 4 + private_interfaces × 1 + dead_code × 1）
- 修复：
  - [src-tauri/src/minecraft/online/crypto.rs](src-tauri/src/minecraft/online/crypto.rs)：移除未使用的 `Verifier`（ed25519_dalek）与 `EphemeralSecret`（x25519_dalek）导入
  - [src-tauri/src/utils/online_manager.rs](src-tauri/src/utils/online_manager.rs)：移除未使用的 `Deserialize`（serde）与 `DeviceCredentials`（storage）导入
  - [src-tauri/src/minecraft/online/client.rs](src-tauri/src/minecraft/online/client.rs)：
    - `JwkKey` 结构体加 `#[allow(dead_code)]`（`kid`/`alg`/`use_` 为 JWKS 规范标准字段，阶段二接入 JWT 签名验证时使用）
    - `get_jwks` 由 `pub async fn` 改为 `async fn`（仅模块内 `verify_jwt` 调用，无需对外暴露，消除 `private_interfaces` 警告）
- 验证：`cargo check --lib` 31.23s，0 警告 0 错误

#### 统一应用图标配置（favicon + Tauri 窗口图标）
- 痛点：原 [index.html](index.html) 使用 Vite 默认的 `/vite.svg` 作为 favicon，且 Tauri 动态创建的窗口（插件窗口、微软登录窗口）未设置图标，标题栏显示系统默认图标
- 修复：
  - [index.html](index.html)：favicon 改为引用 [src/assets/logo.svg](src/assets/logo.svg)
  - [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)：`bundle.icon` 数组新增 `Images/logo.png`，作为 Tauri 应用打包图标与默认窗口图标来源
  - [src-tauri/src/commands/plugins/window.rs](src-tauri/src/commands/plugins/window.rs)：插件子窗口创建时通过 `app.default_window_icon()` 复用默认图标
  - [src-tauri/src/commands/auth/microsoft.rs](src-tauri/src/commands/auth/microsoft.rs)：微软登录窗口创建时同样复用默认图标
- 效果：所有 Tauri 创建的窗口（主窗口、插件窗口、登录窗口）统一显示 `logo.png` 图标，favicon 显示 `logo.svg`

#### 优化 Vite 构建的产物路径分类与文件名清理
- 痛点：原构建产物所有文件混在 `dist/assets/` 根目录，且 Vue 组件 chunk 文件名带丑陋后缀（如 `AlertV2.vue_vue_type_script_setup_true_lang-xxx.js`）
- 修复：在 [vite.config.ts](vite.config.ts) 的 `build.rollupOptions.output` 中配置：
  - `entryFileNames: 'assets/js/[name]-[hash].js'` —— 入口 JS 统一输出到 `assets/js/`
  - `chunkFileNames` 函数 —— 清理 Vue 组件的 `.vue_vue_type_script_setup_true_lang` 后缀，仅保留组件名
  - `assetFileNames` 函数 —— 按扩展名分类：`.css` → `assets/css/`，`.js` → `assets/js/`，其他 → `assets/common/`
- 效果：`dist/assets/` 下分类为 `js/`、`css/`、`common/` 三个子目录，文件名干净

#### 禁用 Vite 资源内联（base64）避免污染 JS chunk
- 痛点：Vite 默认 `assetsInlineLimit: 4096`，小于 4KB 的资源会被内联为 base64 到 JS，导致 JS chunk 体积膨胀且无法被浏览器缓存
- 修复：在 [vite.config.ts](vite.config.ts) 设置 `assetsInlineLimit: 0`，所有资源一律输出为独立文件
- 收益：Tauri 应用走本地文件系统加载，无 HTTP 请求开销，独立文件更利于缓存与调试

#### 解决 Vite 5 资源双输出导致的冗余文件问题
- 痛点：Vite 5.4.x 处理 webp 等资源时，无论用 `import.meta.glob` 还是静态 import，都会同时输出到 `assets/`（默认）和 `assetFileNames` 指定的子目录（如 `assets/common/`），导致同一份文件存在两份。JS 实际引用子目录下的文件，`assets/` 根目录下的文件无人引用
- 根因：Vite 5.4 的 asset plugin 在处理 `import.meta.glob` 的 `?url` 模式导入的资源时，会在 `assetFileNames` 指定路径和默认 `assets/` 路径各 emit 一次。CSS/JS 走不同流程不受影响，仅图片等资源会双输出
- 修复：在 [vite.config.ts](vite.config.ts) 的 `assetFileNames` 函数中，将图片等普通资源的输出路径设为 `assets/` 根目录（与默认输出位置一致），仅 CSS 输出到 `assets/css/`、JS 输出到 `assets/js/`。这样双输出位置相同，文件名带 hash 完全一致，第二份输出会覆盖第一份，最终只有一份文件
- 效果：构建后 `dist/assets/` 根目录下 25 个 webp、25 个 png、1 个 jpg、1 个 svg，无冗余文件

#### 优化 Vite 构建的 chunk 分离策略
- 痛点：原构建产物中 `Home.js` 580KB、`Tools.js` 517KB、`index.js` 249KB，单个业务 chunk 堆积了大量第三方依赖（ol/skinview3d/three 等），导致首屏加载慢、缓存命中率低
- 根因：`vite.config.ts` 未配置 `build.rollupOptions.output.manualChunks`，Rollup 默认把所有动态导入共享的依赖合并到入口 chunk
- 修复：在 [vite.config.ts](vite.config.ts) 的 `build.rollupOptions.output.manualChunks` 中按「稳定性 + 用途」拆分第三方依赖：
  - `vendor-vue`：Vue 框架核心（vue / vue-router / pinia / vue-demi / @vue/*）—— 109KB，几乎不变
  - `vendor-tauri`：Tauri JS 桥接层（@tauri-apps/*）—— 20KB
  - `vendor-ol`：OpenLayers 地图库及依赖（ol / rbush / quickselect）—— 324KB，仅 Tools 页加载
  - `vendor-skinview3d`：3D 皮肤预览库及依赖（skinview3d / three / skinview-utils）—— 504KB，仅皮肤管理加载
  - `vendor-heroicons`：Heroicons 图标库（650+ 图标文件）—— 48KB
  - `vendor-misc`：其他第三方依赖兜底
- 效果（生产构建对比）：
  | Chunk | 修改前 | 修改后 |
  |-------|-------|-------|
  | Home.js | 580 KB | 75 KB（-87%）|
  | Tools.js | 517 KB | 183 KB（-65%）|
  | index.js | 249 KB | 108 KB（-57%）|
- 收益：
  - 首屏（Home 页）只加载 vendor-vue + vendor-tauri + vendor-heroicons + Home.js，不再下载 ol/skinview3d/three
  - ol 和 skinview3d 改为按需加载（进入 Tools / 皮肤管理才下载）
  - 第三方依赖独立 chunk，业务代码更新不会让浏览器重新下载 vendor

#### 修复自定义布局底部"数据每 3 秒自动刷新"重复显示问题
- 痛点：用户反馈主页右侧自定义布局内容区底部出现两个"数据每 3 秒自动刷新"提示
- 根因：JSON/XML 示例文件已包含 `{ "type": "text", "content": "数据每 3 秒自动刷新" }` 的 text section，但 [CustomLayoutPanel.vue](src/plugins/custom-layout/CustomLayoutPanel.vue) 在 sections 渲染容器末尾又硬编码了一个相同的 `<p>数据每 3 秒自动刷新</p>`，导致重复
- 修复：移除渲染面板中硬编码的提示行，让用户通过布局文件自行控制是否显示该提示（示例 JSON/XML 已示范）
- 验证：删除一行 `<p>`，无逻辑变更，渲染流程不变

#### 自定义布局支持一键填入示例模板
- 痛点：用户反馈设置 - 个性化 - 自定义主页，要使用示例模板需先「导出文件 → 打开文件 → 复制内容 → 粘贴到内联编辑器」四步，繁琐不便
- 修复：
  - [CustomLayoutSection.vue](src/views/settings/personal/CustomLayoutSection.vue) 新增 `onFillTemplate` 函数：直接从后端读取当前格式的示例布局内容，填入内联编辑器并同步到 store
  - 原「导出示例」按钮区改为「示例模板」区，包含两个按钮：
    - 「填入模板」（仅 inline 模式显示）：直接填入到内联编辑器
    - 「导出文件」：保留原导出文件功能
  - 保护逻辑：
    - 来源为 URL 时提示先切换到内联模式（URL 模式下内联编辑器不可见，不显示「填入模板」按钮）
    - 内联编辑器已有内容时弹窗确认避免覆盖
- 验证：ESLint 0 错误；文件 286 行（300 行限制内）

#### 自定义布局新增 datetime 格式（修复最近启动时间未格式化问题）
- 痛点：用户反馈设置 - 个性化 - 自定义主页右侧内容区，默认 JSON/XML 示例的"最近启动"列表直接显示后端返回的 RFC3339 时间字符串（如 `2026-07-25T23:13:05.728551100+08:00`），无法阅读
- 根因：示例 JSON/XML 中 `launch_time` 字段使用 `format: "text"`，直接原样输出字符串；`formatValue` 函数也不支持时间格式化
- 修复：
  - [types.ts](src/plugins/custom-layout/types.ts)：`ValueFormat` 类型新增 `'datetime'`
  - [parser.ts](src/plugins/custom-layout/parser.ts)：`VALID_FORMATS` 集合新增 `'datetime'`，使 JSON/XML 解析器接受该格式
  - [datasource.ts](src/plugins/custom-layout/datasource.ts)：`formatValue` 新增 `datetime` 分支，调用新增的 `formatDateTime` 函数将 RFC3339 转为 `MM-DD HH:mm`（与 `LaunchHistoryPanel.vue` 的 `formatTime` 风格一致）
  - [layout-sample.json](src-tauri/resources/samples/layout/layout-sample.json) / [layout-sample.xml](src-tauri/resources/samples/layout/layout-sample.xml)：`launch_time` 字段 `format` 从 `text` 改为 `datetime`
- 验证：ESLint 0 错误

#### 修复首页账号类型显示错误（外置登录误显示为离线账号）
- 痛点：用户反馈首页启动面板顶部账号类型胶囊指示器，外置登录账号也显示为"离线账号"，且图标是手动 SVG 绘制
- 根因：[LaunchPanel.vue](src/components/home/LaunchPanel.vue) 的 `accountTypeLabel` 只判断了 `Microsoft` 和"其他"，把 `AuthlibInjector` 也归到了"离线账号"；图标使用内联 SVG 而非项目统一的 Heroicons 组件库
- 修复：
  - 将 `accountTypeLabel` 重构为 `accountTypeMeta`，三分流返回 `{ label, icon, color }`
    - Microsoft 正版账号：`ShieldCheckIcon`，`text-primary-600`，"正版账号"
    - AuthlibInjector 外置账号：`ServerStackIcon`，`text-purple-600`，"外置账号"
    - Legacy 离线账号：`UserIcon`，`text-gray-400`，"离线账号"
    - 未登录：`UserIcon`，`text-gray-400`，"未登录"
  - 模板用 `<component :is="accountTypeMeta.icon">` 渲染 Heroicons 图标，移除两段内联 SVG
- 验证：ESLint 0 错误

#### 彻底修复 Tauri callback 丢失警告（全局单例 listener 方案）
- 痛点：上一轮修复（`isMounted` 检查 + `cancelPreloadModsDetail` abort 后台 task）后，用户反馈 Mod 管理 ↔ 设置 tab 切换仍触发 `[TAURI] Couldn't find callback id xxx` 警告
- 根因：Tauri 2.x `unlisten` 的固有竞态无法通过前端 `isMounted` 检查消除
  - `_unlisten` 先同步删除前端 callback（`callbacks.delete(id)`），再异步通知 Rust 删 listener
  - 两步之间 Rust 的 `emit` 会调用已删除的 callback id → 警告
  - `image-cached` 事件是最主要触发源：`image_cache::spawn_download` 是独立 `tokio::spawn`，不受 `cancelPreloadModsDetail` 控制，ModTab 卸载后图片下载 task 仍在运行并 emit
  - `mods-preload-update`/`mods-preload-done`/`mods-dir-changed` 同样存在竞态窗口
- 修复方案（全局单例 listener，彻底绕开 `unlisten` 竞态）：
  - 新增 [useGlobalTauriEvent.ts](src/composables/useGlobalTauriEvent.ts)：为每个事件名维护一个全局单例 Tauri listener（永不 `unlisten`），组件通过 `onGlobalEvent` 注册 handler 到本地 `Set`，`onUnmounted` 仅从 Set 移除 handler，Tauri listener 保留 → 无 callback 删除竞态
  - [useImageCache.ts](src/composables/useImageCache.ts)：`onImageCached` 改用 `onGlobalEvent`，`image-cached` 事件的单例 listener 全局存活
  - [useModsPreload.ts](src/composables/useModsPreload.ts)：`mods-preload-update`/`mods-preload-done` 改用 `onGlobalEvent`，移除 `listen`/`unlisten`/`isMounted` 逻辑，`startListener`/`stopListener` 保留为 no-op 兼容旧调用方
  - [useModList.ts](src/composables/useModList.ts)：`mods-dir-changed` 改用 `onGlobalEvent`，移除 `useTauriEvent` 依赖
  - `useTauriEvent.ts` 保留但不再被任何文件引用（适合不需要全局单例的场景）
- 效果：Tauri listener 永不 unlisten → Rust `emit` 永远能找到 callback → 警告彻底消除

#### 修复 Mod 管理 / 设置 tab 来回切换触发 Tauri callback 丢失警告
- 痛点：用户反馈在版本设置页 Mod 管理和设置两个侧边菜单来回切换，会触发 `[TAURI] Couldn't find callback id xxx. This might happen when the app is reloaded while Rust is running an asynchronous operation.` 警告
- 根因分析（Tauri 2.x 官方 `_unlisten` 实现存在固有竞态 + 项目 listener 泄漏）：
  - Tauri 2.x 的 `_unlisten` 是"先同步删前端 callback、再异步通知 Rust 删 listener"，两步之间 Rust 端的 `emit` 会调用已删除的 callback id 触发警告
  - 项目 `preload_mods_detail_cmd` 内部 `tokio::spawn` 后台 task 持续 emit `mods-preload-update`/`mods-preload-done`，`watch_mods_dir` 的防抖线程持续 emit `mods-dir-changed`
  - `useModsPreload` 完全没有 `onUnmounted` 自动清理，依赖外部手动调用 `stopListener`，容易遗漏
  - `useTauriEvent` 和 `useModsPreload` 的 `start()` 是 async 但调用方 fire-and-forget，如果 `await listen` 期间组件已卸载，`stop()` 看到 `unlisten === null` 直接返回，listener 泄漏
  - `useModList.init()` 异步链中 fire-and-forget 调用 `watchModsDir`/`preloadModsDetail`，组件卸载后仍会触发 `tokio::spawn` 后台 task
- 修复方案（前端 P0 最小修复）：
  - [useTauriEvent.ts](src/composables/useTauriEvent.ts)：新增 `isMounted` 标志，`start()` 的 `await listen` 完成后检查 `isMounted`，若已卸载立即 `unlisten()` 新句柄避免泄漏；handler 内部也检查 `isMounted` 跳过卸载后的回调
  - [useModsPreload.ts](src/composables/useModsPreload.ts)：补上 `onUnmounted(() => stopListener())` 自动清理（之前完全依赖外部调用，是最大漏洞）；同样加 `isMounted` 检查，两个 `await listen` 之间窗口期也能正确 unlisten 新句柄
  - [useModList.ts](src/composables/useModList.ts)：`init()` 在每个 `await` 后检查 `isMounted`，组件卸载后不再继续触发 fire-and-forget invoke（特别是 `preloadModsDetail` 会 spawn 后台 task 持续 emit）；`onUnmounted` 中设置 `isMounted = false`
- 修复方案（后端 P1 根治，彻底切断 emit 源）：
  - [preload.rs](src-tauri/src/commands/version/preload.rs)：新增全局 `CURRENT_PRELOAD: OnceLock<Mutex<Option<AbortHandle>>>` 保存当前 spawn task 的 AbortHandle；`preload_mods_detail_cmd` spawn 前 `abort_current_preload()` 取消旧 task（避免多个 task 并发 emit）；新增 `cancel_preload_mods_detail_cmd` 命令供前端卸载时调用
  - [version_install_manager.rs](src-tauri/src/utils/version_install_manager.rs)：注册 `cancel_preload_mods_detail_cmd` action（无参数无 state，handler 用 `_state, _app, _params`）
  - [version-install-manager.ts](src/utils/api/version-install-manager.ts)：新增 `CANCEL_PRELOAD_MODS_DETAIL_CMD` action 常量
  - [personalization.ts](src/utils/api/personalization.ts)：新增 `cancelPreloadModsDetail()` 函数封装
  - [useModList.ts](src/composables/useModList.ts)：`onUnmounted` 调用 `tauri.cancelPreloadModsDetail()` abort 后台 task，task 在下一个 await 点终止，不再 emit 给已注销的 listener
- 验证：`cargo check` 0 错误 0 警告；`eslint` 0 错误（修改文件无新增警告）

#### 导出功能添加进度条 + 固定底栏导出按钮
- 痛点：用户反馈导出过程只有按钮转圈，看不到具体进度；且导出按钮在页面底部，需要滚动到底才能点击，体验不佳
- 后端改动（进度事件推送）：
  - [types.rs](src-tauri/src/commands/version/export/types.rs)：新增 `ExportStage` 枚举（init/scan/network/zip/done/failed）和 `ExportProgress` 结构体（stage/percent/message/versionId），含 `ExportProgress::new` 构造函数（percent 自动 clamp 到 0-100）
  - [mod.rs](src-tauri/src/commands/version/export/mod.rs)：定义 `EXPORT_PROGRESS_EVENT = "export-progress"` 常量，新增 `emit_progress(app, version_id, stage, percent, message)` 函数；`export_modpack` 在各阶段调用 emit：Init(1%)/Scan(3%→10%)/Network(12%→50%)/Zip(52%→95%)/Done(100%)/Failed(0%)，并传递 `app: &AppHandle` 给 `build_modpack_zip`
  - [zip.rs](src-tauri/src/commands/version/export/zip.rs)：新增 `emit_zip_progress(app, version_id, current, total)` 函数（50-95% 区间按文件数线性插值）；6 个 builder（Modrinth/CurseForge/HMCL/MultiMC/MCBBS/Compress）全部加 `app: &AppHandle` 参数，在文件写入循环中按 i+1/total 调用 emit，确保大文件打包时进度实时更新
- 前端改动（监听 + UI）：
  - [version-export-manager.ts](src/utils/api/version-export-manager.ts)：新增 `EXPORT_PROGRESS_EVENT` 常量、`ExportStage` 类型联合、`ExportProgress` 接口（与后端 serde camelCase 对齐）
  - [useExportTab.ts](src/composables/useExportTab.ts)：新增 `exportProgress`/`exportStage`/`exportMessage` 三个 ref；新增 `startProgressListener`/`stopProgressListener`/`resetProgress`；在 `onMounted` 启动 `listen<ExportProgress>('export-progress')` 监听，按 `versionId` 过滤避免切换版本时残留旧事件；`onUnmounted` 自动 unlisten；`handleExport` 开始前 `resetProgress()`，结束 3 秒后重置（保留完成/失败状态显示）
  - [ExportTab.vue](src/views/version-settings/ExportTab.vue)：参考 [LoaderSelect.vue](src/views/LoaderSelect.vue) 改为三段式 flex 列布局：中段 `flex-1 overflow-y-auto` 放基本信息 + 导出选项表单（`max-w-2xl` 居中），底段 `shrink-0 border-t` 固定操作栏（左状态提示 + 右导出按钮 + 上方进度条）；导出按钮使用项目自定义 [Button.vue](src/components/common/Button.vue) 组件（`type="primary"` + `:loading="exporting"`），图标全部改用 Heroicons（`ArrowUpTrayIcon`/`CheckIcon`/`XMarkIcon`/`DocumentArrowDownIcon`/`DocumentArrowUpIcon`）；导出中按钮文案显示"扫描文件 23%"（动态阶段+百分比），上方显示 1px 水平进度条；完成显示对勾图标+"导出完成"，失败显示叉号图标+"导出失败"；底栏左侧 `bottomHint` 显示当前格式/进度/状态文案
  - [VersionSettings.vue](src/views/VersionSettings.vue)：将 export tab 加入 `flex flex-col` 分支（与 mod tab 一致），让 ExportTab 自管内部滚动 + 固定底栏布局
- 验证：`cargo check` 0 错误 0 警告；`eslint` 0 错误（vue-tsc 环境预存问题不影响）

#### 重构导出功能依赖解析，复用 VersionSetup 消除重复造轮子
- 痛点：用户指出 zip.rs 中自造的 MC 版本/加载器检测逻辑（`extract_fabric_version` / `extract_loader_version` / `extract_loader_from_libraries` / `extract_mc_version_from_id`）是重复造轮子，且按 version_id 字符串名字提取加载器版本不可靠（如 RLCraft 的 id 是 "RLCraft 1.12.2 - Release v2.9.3"，不含 `forge-` 关键字）
- 复用方案：项目已有 `crate::minecraft::version::setup::VersionSetup::from_version_json(version_dir, version_id)` 返回完整 `LoaderInfo`（含 `version_type` + 6 种加载器版本号 + `original_version`），与启动游戏、扫描版本列表走同一套检测逻辑（`state.rs::detect_version_type` 关键字优先 + `libraries[]` maven 坐标兜底，正确区分 Fabric/Quilt/Forge/NeoForge/OptiFine/LiteLoader）
- 改动：[zip.rs](src-tauri/src/commands/version/export/zip.rs) 删除 4 个自造函数，新增 `parse_loader_info(instance_dir, version_id) -> (VersionType, Option<String>)` 和重写 `parse_dependencies`，全部委托给 `VersionSetup::from_version_json`；`build_cf_loader_id` / `build_mmc_zip` / `build_mcbbs_zip` 改为基于 `VersionType` 枚举 match 分发，避免字符串 key 查找；`build_hmcl_zip` 也复用 `parse_dependencies` 获取 mc 版本（替代 `extract_mc_version_from_id`）
- 影响范围：所有 6 种导出格式的依赖解析统一走 `VersionSetup`，与启动游戏 / 扫描版本列表 / Java 检查等模块完全一致；Quilt 现在能正确识别（之前 `scan/loaders.rs::detect_loaders` 把 Quilt 误判为 Fabric，但 `VersionSetup::from_version_json` 走 `state.rs::detect_loader_from_json` 能正确区分）
- 验证：`cargo check` 0 错误 0 警告

#### 修复导出扫描漏文件和加载器检测失败
- 痛点：用户测试 CurseForge 格式导出 RLCraft 实例，结果 zip 内只有 resourcepacks 文件夹和 2 个 txt 文件，mods/ 目录下的所有 jar 全部漏扫；同时 manifest.modLoaders 为空（未检测到 Forge 加载器）
- 根因 1：[scan.rs](src-tauri/src/commands/version/export/scan.rs) 的 `glob_to_regex` 将规则 `mods/` 转成 regex `^mods/$`，只匹配字面量字符串 `mods/`，不匹配 `mods/xxx.jar`。规则 `mods/` 的语义是"匹配 mods 目录下所有内容"，应等价于 `mods/*`
- 修复 1：在 `compile_rules` 中对以 `/` 结尾的 pattern 自动追加 `*`，使其生成 `^mods/.*$` 正确匹配目录下所有文件。影响范围：所有目录前缀规则（`mods/`/`coremods/`/`config/`/`resourcepacks/`/`shaderpacks/`/`saves/` 等）现在都能正确扫描
- 根因 2：[zip.rs](src-tauri/src/commands/version/export/zip.rs) 的 `parse_dependencies` 仅从 version json 的 `id` 字段用正则 `forge-([\d.]+)` 提取加载器版本，但 RLCraft 的 id 是 "RLCraft 1.12.2 - Release v2.9.3"（不含 `forge-`），且 1.12.2 老版 Forge 的 version json 用 `net.minecraftforge:forge:1.12.2-14.23.5.2860` 这种 maven 坐标，id 字段不含加载器版本号
- 修复 2：新增 `extract_loader_from_libraries` 函数，优先从 version json 的 `libraries[]` 数组中按 maven 坐标前缀（`net.minecraftforge:forge:` / `net.neoforged:neoforge:` / `net.fabricmc:fabric-loader:` / `org.quiltmc:quilt-loader:`）提取加载器版本。老版 Forge 坐标 `1.12.2-14.23.5.2860` 取 `-` 后的部分作为加载器版本。原 id/mainClass/content 兜底逻辑保留作为 fallback
- 验证：`cargo check` 0 错误 0 警告

#### 扩展导出功能支持 6 种整合包格式（与导入格式对齐）
- 痛点：之前导出只支持 Modrinth 格式（.mrpack），用户无法导出 CurseForge/HMCL/MMC/MCBBS/普通压缩包格式，与导入支持的 7 种格式（除 LauncherPack 外）不对齐
- 后端改动：
  - [types.rs](src-tauri/src/commands/version/export/types.rs)：新增 `ExportFormat` 枚举（6 个变体：Modrinth/Curseforge/Hmcl/Mmc/Mcbbs/Compress），含 `extension()` 和 `requires_online_check()` 方法；`ExportModpackParams` 新增 `format` 字段（默认 Modrinth）；`ModDownloadInfo` 新增 `project_id`/`file_id` 字段（CF 格式导出用）
  - [zip.rs](src-tauri/src/commands/version/export/zip.rs)：重构 `build_modpack_zip` 为 dispatcher，按 format 分发到 6 个 builder：
    - `build_modrinth_zip`（已有逻辑）：modrinth.index.json + overrides/
    - `build_curseforge_zip`：manifest.json + modlist.html + overrides/，联网获取到 projectID/fileID 的 mod 写入 files[]，未获取到的直接打包
    - `build_hmcl_zip`：modpack.json + minecraft/，所有文件直接打包
    - `build_mmc_zip`：mmc-pack.json + instance.cfg + .minecraft/，按 version json 解析 components[]
    - `build_mcbbs_zip`：mcbbs.packmeta + overrides/，按 version json 解析 addons[]
    - `build_compress_zip`：直接打包 .minecraft/，无 manifest
    - 抽取 `create_zip_writer`/`write_file_entry`/`write_string_entry` 共用工具，复用 `parse_dependencies` 解析 minecraft + 加载器版本
  - [mod.rs](src-tauri/src/commands/version/export/mod.rs)：`export_modpack` 加 `format.requires_online_check()` 守卫，非联网格式即使误传 `check_hosted_assets=true` 也强制跳过联网
  - [network.rs](src-tauri/src/commands/version/export/network.rs)：`merge_cf_results` 把 CF 返回的 `project_id`/`file_id` 填入 `ModDownloadInfo`，供 CF 格式 manifest 使用
- 跨模块改动：[community/types.rs](src-tauri/src/minecraft/community/types.rs) 的 `FileDownloadInfo` 新增 `project_id`/`file_id` 字段；[curseforge/mod.rs](src-tauri/src/minecraft/community/curseforge/mod.rs) 的 `fingerprint_search_with_downloads` 从 `exactMatches[i].id`（modId）和 `file.id` 填充这两个字段；[modrinth/mod.rs](src-tauri/src/minecraft/community/modrinth/mod.rs) 设置为 None
- 前端改动：
  - [version-export-manager.ts](src/utils/api/version-export-manager.ts)：新增 `ExportFormat` 类型联合、`ExportFormatOption` 接口、`EXPORT_FORMAT_OPTIONS` 常量（6 个格式元信息，含 label/description/extension/supportsOnlineCheck）、`findExportFormat` 工具函数；`ExportModpackParams` 新增 `format` 字段
  - [useExportTab.ts](src/composables/useExportTab.ts)：新增 `exportFormat` ref（默认 'modrinth'）、`currentFormatMeta`/`supportsOnlineCheck`/`formatOptions` 计算属性；`handleExport` 根据格式选择扩展名（.mrpack / .zip）和文件对话框标题；非联网格式强制 `finalCheckHostedAssets=false` 避免无效联网请求
  - [ExportTab.vue](src/views/version-settings/ExportTab.vue)：顶部添加"导出格式" Select 下拉（含格式描述），联网检查和仅 Modrinth 选项仅在 `supportsOnlineCheck` 为 true 时显示；Tooltip 文案动态显示当前格式
- 决策记录：参考 PCL2 仅支持 Modrinth 格式导出，但用户要求"支持什么格式导入就支持什么格式导出"，故实现 6 种格式（除 LauncherPack，因 MoLaunch 不带启动器分发）；CurseForge 联网查不到 projectID/fileID 的 mod 按用户选择直接打包到 overrides
- 验证：`cargo check` 0 错误 0 警告；`eslint` 0 错误

#### 完成版本设置 → 导出子页前端实现（Modrinth 格式整合包导出）
- 痛点：版本设置页左侧导航已有"导出"Tab，但此前为占位"功能开发中"。后端 `version_export_manager` 已实现 4 个 action（get_export_options / export_modpack / save_export_config / load_export_config）和 ~20 个静态选项 + 资源包/存档/光影包动态子选项扫描、Modrinth+CurseForge 联网检查、`modrinth.index.json` 生成、zip 打包等完整能力，但前端缺少 UI 入口。
- 前端新增 3 个文件：
  - [utils/api/version-export-manager.ts](src/utils/api/version-export-manager.ts)：API 层，定义 `versionExportManager` 入口、`VERSION_EXPORT_ACTIONS` 常量、4 个数据类型接口（`ExportOption` / `ExportModpackParams` / `ExportModpackResult` / `SaveConfigParams` / `LoadConfigResult`）和 4 个强类型高层 API 函数（`getExportOptions` / `exportModpack` / `saveExportConfig` / `loadExportConfig`），与后端 `utils::version_export_manager::DISPATCHER` 注册的 action 一一对应
  - [composables/useExportTab.ts](src/composables/useExportTab.ts)：业务逻辑 composable，封装状态管理（packName / packVersion / checkHostedAssets / modrinthUploadMode / exportOptions）+ 4 个 handler（loadOptions / toggleOption / handleSaveConfig / handleLoadConfig / handleExport），`applyConfigOverride` 复刻后端 `config.rs::apply_config_to_options` 的必选项保护逻辑（enabled=false 的选项不允许取消勾选）
  - [views/version-settings/ExportTab.vue](src/views/version-settings/ExportTab.vue)：主页面组件（145 行，远低于 300 行硬约束），布局为单列 max-w-2xl：基本信息卡片（整合包名称/版本/联网检查/仅 Modrinth）+ 导出选项卡片（含读取/保存配置按钮）+ 操作按钮区
  - [views/version-settings/export-tab/ExportOptions.vue](src/views/version-settings/export-tab/ExportOptions.vue)：选项列表组件（98 行），按 `parent` 字段分组渲染顶层/子选项，必选项（enabled=false）显示"必选"标签并禁用勾选框，可见性由 `visible` 字段控制，子选项缩进 + 左侧分隔线，空状态垂直水平居中
- 修改 [views/VersionSettings.vue](src/views/VersionSettings.vue)：替换原占位 `<div v-else>功能开发中</div>` 为 `<ExportTab v-else-if="activeCategory === 'export'" />`，import 新组件
- 设计原则：复用项目自定义组件（Input / Button / Tooltip），不引入新依赖；composable 模式与 `useVersionOverviewActions` 一致；选项状态原地修改 `checked` 字段避免整树重渲染；文件对话框走 `pickSavePath` / `pickFile`（基于 @tauri-apps/plugin-dialog）
- 验证：`eslint` 0 错误（仅 1 个预存在的 `_id` 未使用 warning，与本次修改无关）；`cargo check` 0 错误 0 警告（后端未改动，仅验证未引入回归）

#### 修复 Mod 管理页顶部筛选 tag 文字换行问题
- 痛点：版本设置 → Mod 管理页中，当 mods 文件数量较多（数字 badge 变宽）时，顶部"全部/已启用/已禁用"筛选 tag 组被 flex 默认 shrink 行为压缩，导致 tag 内中文文字（"全部"/"已启用"/"已禁用"）折行显示，视觉拥挤。
- 修改 [src/views/version-settings/mod-tab/ModToolbar.vue](src/views/version-settings/mod-tab/ModToolbar.vue)：3 处 Tailwind 类调整
  - 筛选 tag 组容器：`flex items-center gap-1.5 rounded-lg bg-gray-100 p-1` → `flex flex-shrink-0 items-center gap-1.5 rounded-lg bg-gray-100 p-1`（整个 tag 组不被父容器压缩）
  - 单个按钮：`flex items-center gap-1.5 rounded-md px-3 py-1 text-xs font-medium transition-colors` → 加 `whitespace-nowrap`（文字强制不换行）
  - 计数 badge：`rounded-full px-1.5 py-0.5 text-[10px] leading-none` → 加 `whitespace-nowrap tabular-nums`（数字不换行 + 等宽数字避免多位数抖动）
- 验证：`eslint` 0 错误；窗口窄时搜索框优先让出空间，tag 组保持完整宽度

#### 完成 IPC dispatcher 迁移收尾：set_game_dir 聚合 + plugins/sdk.ts 走 manager
- 痛点：IPC dispatcher 迁移收尾阶段发现两处遗漏：(1) `set_game_dir` 后端有 `#[tauri::command]` 标注但未在 `lib.rs` 注册，前端 `setGameDir()` 调用会失败；(2) `plugins/sdk.ts` 中 7 处 `invoke('xxx')` 仍走裸 IPC 命令，未通过 13 个 manager 入口，与统一 dispatcher 架构不一致。
- 后端：
  - 修改 [commands/system/game_dir.rs](src-tauri/src/commands/system/game_dir.rs)：`set_game_dir` 去掉 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`，与 `open_game_dir` / `get_game_dir` 等同级函数签名一致；模块头注释从"6 个 Tauri 命令 + set_game_dir 保留独立"改为"7 个 Tauri 命令全部聚合"
  - 修改 [utils/system_manager.rs](src-tauri/src/utils/system_manager.rs)：新增 `SetGameDirParams` 结构体（`#[serde(rename_all = "camelCase")]` game_dir 字段），DISPATCHER 注册 `set_game_dir` action（handler 用 `state, _app, params`，调用 `set_game_dir(&state, p.game_dir)`）；文件头注释从"17 个 action"改为"18 个 action"，game_dir 分组从"6 个"改为"7 个"
- 前端：
  - 修改 [utils/api/system-manager.ts](src/utils/api/system-manager.ts)：`SYSTEM_ACTIONS` 新增 `SET_GAME_DIR: 'set_game_dir'`，文件头注释从"17 个 action"改为"18 个 action"，game_dir 分组从"6 个"改为"7 个"
  - 修改 [utils/api/system.ts](src/utils/api/system.ts)：`setGameDir` 改为 `systemManager<void>(SYSTEM_ACTIONS.SET_GAME_DIR, { gameDir })`，移除 `import { invoke } from '@tauri-apps/api/core'`（本文件已无裸 invoke 调用）；文件头注释从"8 个原 Tauri 命令 + set_game_dir 保留独立"改为"9 个原 Tauri 命令全部聚合"
  - 修改 [plugins/sdk.ts](src/plugins/sdk.ts)：7 处 `invoke<XXX>('yyy')` 改为通过对应 manager + ACTIONS 常量调用，移除 `import { invoke }`，改为导入 4 个 manager（configManager / systemManager / versionLaunchManager / versionListManager）+ 4 个 ACTIONS 常量
    - `getConfig` → `configManager(CONFIG_ACTIONS.GET_CONFIG, { keys: null })`
    - `listInstalledVersions` → `versionListManager(VERSION_LIST_ACTIONS.LIST_INSTALLED_VERSIONS)`
    - `listInstalledVersionsWithType` → `versionListManager(VERSION_LIST_ACTIONS.LIST_INSTALLED_VERSIONS_WITH_TYPE)`
    - `listLaunchHistory` → `versionLaunchManager(VERSION_LAUNCH_ACTIONS.GET_LAUNCH_HISTORY)`
    - `getSystemMemory` → `systemManager(SYSTEM_ACTIONS.GET_SYSTEM_MEMORY)`
    - `getRunningGamePid` → `versionLaunchManager(VERSION_LAUNCH_ACTIONS.GET_RUNNING_GAME)`
    - `getCacheStats` → `systemManager(SYSTEM_ACTIONS.GET_CACHE_STATS)`
- 收益：前端再无裸 `invoke('xxx')` 调用（除 13 个 manager 入口本身的 `invoke('xxx_manager')`），所有 IPC 调用统一走 `XxxManager(XXX_ACTIONS.YYY, params)` 模式，IPC 命令收敛为 13 个 manager 入口；`set_game_dir` 不再是"未注册的死代码"，前端可正常通过 `setGameDir()` 调用
- 验证：`cargo check` 0 错误 0 警告；`eslint` 0 错误（4 个原有 `_xxx` 未使用 warning 与本次修改无关）；`tsc --noEmit --skipLibCheck` 通过

#### 聚合 13 个 community IPC 命令为 1 个 community_manager（参照 image_cache_manager 模式）
- 痛点：`commands::community` 下 4 个子模块（search 2 个 / detail 3 个 / install::resource 5 个 / install::modpack 3 个）共 13 个独立 Tauri 命令分散注册，与 `image_cache_manager` / `meta_manager` / `tools_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/community_manager.rs](src-tauri/src/utils/community_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 13 个 action，7 个参数结构体（`ResourceTypeParams` / `ProjectVersionsParams` / `McmodUrlParams` / `DownloadToPathParams` / `FormatFilenameParams` / `ResourceInstallPathParams` / `FilePathParams`）均使用 `#[serde(rename_all = "camelCase")]`；对于 `req: SomeRequest` 类型参数（search_resources / get_project_detail / download_resource / install_resource / install_modpack / install_local_modpack）直接将 params 反序列化为对应 Request 类型，避免冗余包裹结构体
    - `search_resources` / `get_category_tags` / `get_project_detail` / `get_project_versions` / `get_mcmod_url` / `preview_local_modpack` 不需要 state（handler 用 `_state, _app`）
    - `download_resource` / `format_download_filename` / `install_resource` / `get_resource_install_path` / `install_modpack` / `install_local_modpack` 仅需 state（handler 用 `state, _app`）
    - `download_resource_to_path` 需要 state 和 app（handler 用 `state, app`，原 `_app: AppHandle` 改为 `_app: &AppHandle`）
  - 改造 4 个原命令文件去掉 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`、`AppHandle` → `&AppHandle`：
    - [commands/community/search.rs](src-tauri/src/commands/community/search.rs)（search_resources / get_category_tags，无 state/app 参数，签名不变）
    - [commands/community/detail.rs](src-tauri/src/commands/community/detail.rs)（get_project_detail / get_project_versions / get_mcmod_url，无 state/app 参数，签名不变）
    - [commands/community/install/resource.rs](src-tauri/src/commands/community/install/resource.rs)（download_resource / download_resource_to_path / format_download_filename / install_resource / get_resource_install_path，`State<'_, AppState>` → `&AppState`，`_app: AppHandle` → `_app: &AppHandle`）
    - [commands/community/install/modpack.rs](src-tauri/src/commands/community/install/modpack.rs)（install_modpack / install_local_modpack / preview_local_modpack，`State<'_, AppState>` → `&AppState`，移除 `use tauri::State`）
  - 新增 [commands/community/mod.rs](src-tauri/src/commands/community/mod.rs) 的 `#[tauri::command] pub async fn community_manager(state, app, req)` 转发入口
- 前端：
  - 新增 [utils/api/community-manager.ts](src/utils/api/community-manager.ts)：`communityManager(action, params)` 入口和 `COMMUNITY_ACTIONS` 常量（13 个 action 全部大写蛇形命名，值为小写下划线），`CommunityAction` 类型
  - 修改 [utils/api/community.ts](src/utils/api/community.ts)：12 个 `invoke('xxx')` 改为 `communityManager(COMMUNITY_ACTIONS.XXX, ...)`（searchResources / getCategoryTags / getProjectDetail / getProjectVersions / downloadResource / downloadResourceToPath / formatDownloadFilename / getResourceInstallPath / installModpack / installLocalModpack / previewLocalModpack / getMcmodUrl），保留所有类型定义和导出
- 收益：IPC 命令从 13 个收敛为 1 个，与 `image_cache_manager` / `meta_manager` / `tools_manager` 模式一致，降低注册和维护成本；后续新增社区资源相关命令只需在 `community_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 聚合 17 个 system + logger IPC 命令为 1 个 system_manager（参照 image_cache_manager 模式）
- 痛点：`commands::system` 下 4 个子模块（game_dir 6 个 / config 2 个 / developer 5 个 / about 1 个）+ crate 顶层 `logger` 模块（3 个）共 17 个独立 Tauri 命令分散注册，与 `image_cache_manager` / `meta_manager` / `config_manager` / `tools_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/system_manager.rs](src-tauri/src/utils/system_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 17 个 action，3 个参数结构体（`PathParams` / `WriteTextFileParams` / `ReadLogFileParams`）均使用 `#[serde(rename_all = "camelCase")]`
    - `open_game_dir` / `get_game_dir` / `save_config_to_file` 需要 state（handler 用 `state, _app`）
    - `open_path` / `reveal_in_explorer` / `write_text_file` / `read_log_file` 需要参数（handler 用 `_state, _app, params`）
    - `is_developer_unlocked` 返回 bool（非 Result）、`get_storage_dirs` 返回 `StorageDirs`、`get_system_info` 返回 `SystemInfo`、`get_log_path` 返回 `String`、`list_log_files` 返回 `Vec<String>`，handler 内用 `Ok(serde_json::to_value(r).map_err(|e| e.to_string())?)` 包装
  - 改造 5 个原命令文件去掉 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`：
    - [commands/system/game_dir.rs](src-tauri/src/commands/system/game_dir.rs)（open_game_dir / open_path / reveal_in_explorer / get_game_dir / write_text_file / get_system_memory；`set_game_dir` 保留独立 Tauri 命令，版本切换流程内部调用）
    - [commands/system/config.rs](src-tauri/src/commands/system/config.rs)（get_config_path / save_config_to_file；其他 `get_config_value` / `set_config_value` / `config_manager` 已由其他 agent 迁移）
    - [commands/system/developer.rs](src-tauri/src/commands/system/developer.rs)（is_developer_unlocked / unlock_developer_mode / get_storage_dirs / get_system_info / get_cache_stats）
    - [commands/system/about.rs](src-tauri/src/commands/system/about.rs)（get_about_data）
    - [logger.rs](src-tauri/src/logger.rs)（get_log_path / list_log_files / read_log_file；logger 在 crate 顶层模块，不在 commands/ 下）
  - 新增 [commands/system/mod.rs](src-tauri/src/commands/system/mod.rs) 的 `#[tauri::command] pub async fn system_manager(state, app, req)` 转发入口
- 前端：
  - 新增 [utils/api/system-manager.ts](src/utils/api/system-manager.ts)：`systemManager(action, params)` 入口和 `SYSTEM_ACTIONS` 常量（17 个 action 全部大写蛇形命名，值为小写下划线），`SystemAction` 类型
  - 修改 [utils/api/system.ts](src/utils/api/system.ts)：8 个 `invoke('xxx')` 改为 `systemManager(SYSTEM_ACTIONS.XXX, ...)`（openGameDir / openPath / revealInExplorer / getGameDir / writeTextFile / getSystemMemory / getConfigPath / saveConfigToFile），`setGameDir` 保留 `invoke('set_game_dir')`，下载进度相关函数已被其他 agent 迁移到 `versionProgressManager` 保持原样
  - 修改 [utils/api/developer.ts](src/utils/api/developer.ts)：8 个 `invoke('xxx')` 改为 `systemManager(SYSTEM_ACTIONS.XXX, ...)`（isDeveloperUnlocked / unlockDeveloperMode / getStorageDirs / getSystemInfo / getCacheStats / getLogPath / listLogFiles / readLogFile），保留 `StorageDirs` / `SystemInfo` / `CacheStat` / `CacheStatsResult` 类型定义和导出
  - 修改 [utils/api/about.ts](src/utils/api/about.ts)：`getAboutData` 由 `invoke('get_about_data')` 改为 `systemManager(SYSTEM_ACTIONS.GET_ABOUT_DATA)`，移除未使用的 `invoke` 导入，保留 `Author` / `AcknowledgementItem` / `DependencyItem` / `LicenseItem` / `AboutData` 类型定义
- 收益：IPC 命令从 17 个收敛为 1 个，与 `image_cache_manager` / `meta_manager` / `config_manager` 模式一致，降低注册和维护成本；后续新增 system / logger 相关命令只需在 `system_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 聚合 11 个 version download/install/loaders/preload IPC 命令为 1 个 version_install_manager（参照 image_cache_manager 模式）
- 痛点：`commands::version` 下 4 个子模块（download 1 个 / install 1 个 / loaders 8 个 / preload 1 个）共 11 个独立 Tauri 命令分散注册，与 `image_cache_manager` / `version_list_manager` / `version_mods_manager` / `version_launch_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/version_install_manager.rs](src-tauri/src/utils/version_install_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 11 个 action，5 个参数结构体（VersionIdParams / McVersionParams / InstallMergedParams / ValidateLoadersParams / InstallFabricApiParams）均使用 `#[serde(rename_all = "camelCase")]`；`download_version` / `install_merged` / `preload_mods_detail_cmd` 同时需要 state 和 app（handler 用 `state, app`），`validate_loaders` / `list_fabric_api_versions` 不需要 state（handler 用 `_state`），其余 loaders 命令仅需 state（handler 用 `_app`）
  - 改造 4 个原命令文件去掉 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`、`AppHandle` → `&AppHandle`：
    - [commands/version/download.rs](src-tauri/src/commands/version/download.rs)（download_version）
    - [commands/version/install/mod.rs](src-tauri/src/commands/version/install/mod.rs)（install_merged，保留 `#[allow(clippy::too_many_arguments)]`）
    - [commands/version/loaders.rs](src-tauri/src/commands/version/loaders.rs)（8 个函数；list_forge_versions / list_neoforge_versions / list_optifine_versions 返回类型从 `Result<String, String>` 改为 `Result<Vec<serde_json::Value>, String>`、list_fabric_versions 改为 `Result<serde_json::Value, String>`、list_liteloader_versions 改为 `Result<Vec<String>, String>`，由 dispatcher 直接序列化为 JSON Value，前端不再需要 JSON.parse）
    - [commands/version/preload.rs](src-tauri/src/commands/version/preload.rs)（preload_mods_detail_cmd，`tokio::spawn` 内的 `app` 改为 `app.clone()` 因为参数变为 `&AppHandle`）
  - 新增 [commands/version/mod.rs](src-tauri/src/commands/version/mod.rs) 的 `#[tauri::command] pub async fn version_install_manager(state, app, req)` 转发入口
- 前端：
  - 新增 [utils/api/version-install-manager.ts](src/utils/api/version-install-manager.ts)：`versionInstallManager(action, params)` 入口和 `VERSION_INSTALL_ACTIONS` 常量（11 个 action 全部大写蛇形命名，值为小写下划线），`VersionInstallAction` 类型
  - 修改 [utils/api/version.ts](src/utils/api/version.ts)：`downloadVersion` 由 `invoke('download_version', { versionId })` 改为 `versionInstallManager(VERSION_INSTALL_ACTIONS.DOWNLOAD_VERSION, { versionId })`，移除未使用的 `invoke` 导入（其他函数仍走 `versionListManager`）
  - 修改 [utils/api/loader.ts](src/utils/api/loader.ts)：9 个函数（listForgeVersions / listNeoforgeVersions / listFabricVersions / listOptifineVersions / listLiteloaderVersions / validateLoaders / installMerged / listFabricApiVersions / installFabricApiForVersion）由 `invoke('xxx', {...})` 改为 `versionInstallManager(VERSION_INSTALL_ACTIONS.XXX, {...})`，移除 `JSON.parse` 包装（dispatcher 直接返回 JSON Value），保留 `FabricApiVersion` 类型定义和导出
  - 修改 [utils/api/personalization.ts](src/utils/api/personalization.ts)：仅 `preloadModsDetail` 由 `invoke('preload_mods_detail_cmd', { versionId })` 改为 `versionInstallManager(VERSION_INSTALL_ACTIONS.PRELOAD_MODS_DETAIL_CMD, { versionId })`，其他函数（个性化 / 选中版本 / 文件补全 / mod 管理 / 脚本导出）保持原样
- 收益：IPC 命令从 11 个收敛为 1 个，与 `image_cache_manager` / `version_list_manager` / `version_mods_manager` / `version_launch_manager` 模式一致，降低注册和维护成本；后续新增版本下载/安装/加载器/预加载相关命令只需在 `version_install_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 聚合 12 个 plugins IPC 命令为 1 个 plugins_manager（参照 image_cache_manager 模式）
- 痛点：`commands::plugins` 模块的 12 个命令（sandbox 3 个 + install 2 个 + spawn 1 个 + window 1 个 + layout 1 个 + export 2 个 + personalization 2 个）独立注册为 Tauri 命令，与 `image_cache_manager` / `meta_manager` / `tools_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/plugins_manager.rs](src-tauri/src/utils/plugins_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 12 个 action，10 个参数结构体（PluginIdParams / ReadExternalPluginFileParams / SourceDirParams / ZipPathParams / PluginSpawnProcessParams / PluginCreateWindowParams / LoadCustomLayoutParams / ReadLayoutSampleParams / ExportPluginSampleParams / WritePersonalizationParams）均使用 `#[serde(rename_all = "camelCase")]`；所有 action 均不需要 `AppState`，handler 内用 `_state` 忽略；`plugin_create_window` 需要 `&app` 用于创建 WebviewWindow
  - 改造 7 个原命令文件去掉 `#[tauri::command]` 标注：
    - [commands/plugins/sandbox.rs](src-tauri/src/commands/plugins/sandbox.rs)（list_external_plugins / read_external_plugin_file / uninstall_external_plugin，3 个函数均无 state/app 参数，签名不变）
    - [commands/plugins/install.rs](src-tauri/src/commands/plugins/install.rs)（install_external_plugin_from_dir / install_external_plugin_from_zip，签名不变）
    - [commands/plugins/spawn.rs](src-tauri/src/commands/plugins/spawn.rs)（plugin_spawn_process，签名不变）
    - [commands/plugins/window.rs](src-tauri/src/commands/plugins/window.rs)（plugin_create_window，参数 `app: AppHandle` 改为 `app: &AppHandle`，`WebviewWindowBuilder::new(&app, ...)` 改为 `WebviewWindowBuilder::new(app, ...)` 适配 `Manager` trait bound）
    - [commands/plugins/layout.rs](src-tauri/src/commands/plugins/layout.rs)（load_custom_layout，签名不变）
    - [commands/plugins/export.rs](src-tauri/src/commands/plugins/export.rs)（read_layout_sample / export_plugin_sample，签名不变）
    - [commands/plugins/personalization.rs](src-tauri/src/commands/plugins/personalization.rs)（read_personalization / write_personalization，签名不变）
  - 新增 [commands/plugins/mod.rs](src-tauri/src/commands/plugins/mod.rs) 的 `#[tauri::command] pub async fn plugins_manager(state, app, req)` 转发入口
- 前端：
  - 新增 [utils/api/plugins-manager.ts](src/utils/api/plugins-manager.ts)：`pluginsManager(action, params)` 入口和 `PLUGINS_ACTIONS` 常量（12 个 action 全部大写蛇形命名，值为小写下划线），`PluginsAction` 类型
  - 修改 [utils/api/plugins.ts](src/utils/api/plugins.ts)：7 个 `invoke('xxx', {...})` 改为 `pluginsManager(PLUGINS_ACTIONS.XXX, {...})`，函数签名、类型定义（`ExternalPluginManifest` / `ExternalPluginEntry`）和外部调用点保持不变
  - 修改 [utils/pluginInstaller.ts](src/utils/pluginInstaller.ts)：`loadPersonalizationData` / `savePersonalizationData` / `fetchCustomLayoutContent` 3 个函数由 `invoke('xxx', {...})` 改为 `pluginsManager(PLUGINS_ACTIONS.XXX, {...})`，移除未使用的 `invoke` 导入
  - 修改 [plugins/sandbox/PluginSandbox.vue](src/plugins/sandbox/PluginSandbox.vue)：`spawnProcess` / `createWindow` 两个特殊处理分支由 `invoke('plugin_spawn_process', {...})` / `invoke('plugin_create_window', {...})` 改为 `pluginsManager(PLUGINS_ACTIONS.PLUGIN_SPAWN_PROCESS, {...})` / `pluginsManager(PLUGINS_ACTIONS.PLUGIN_CREATE_WINDOW, {...})`，移除未使用的 `invoke` 导入
- 收益：IPC 命令从 12 个收敛为 1 个，与 `image_cache_manager` / `meta_manager` / `tools_manager` / `sdk_manager` / `skin_manager` / `version_mods_manager` / `java_manager` / `config_manager` 模式一致，降低注册和维护成本；后续新增 plugins 相关命令只需在 `plugins_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 聚合 17 个 version 子模块 IPC 命令为 1 个 version_list_manager（参照 image_cache_manager 模式）
- 痛点：`commands::version` 下 4 个子模块（list 6 个 / folder 5 个 / manage 4 个 / personalization 2 个）共 17 个独立 Tauri 命令分散注册，与 `image_cache_manager` / `version_mods_manager` / `version_launch_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/version_list_manager.rs](src-tauri/src/utils/version_list_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 17 个 action，7 个参数结构体（VersionIdParams / AddMcFolderParams / McFolderPathParams / RenameMcFolderParams / RenameVersionParams / SetSelectedVersionParams / UpdatePersonalizationParams）均使用 `#[serde(rename_all = "camelCase")]`；`fix_version_files` 需要 `AppHandle`（emit `version-fix-progress` 事件），handler 用 `&app` 调用
  - 改造 4 个原命令文件去掉 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`、`AppHandle` → `&AppHandle`：
    - [commands/version/list.rs](src-tauri/src/commands/version/list.rs)（list_versions / list_installed_versions / list_installed_versions_with_type / uninstall_version / get_version_effective_dir / get_version_game_version）
    - [commands/version/folder.rs](src-tauri/src/commands/version/folder.rs)（list_mc_folders / add_mc_folder / remove_mc_folder / switch_mc_folder / rename_mc_folder）
    - [commands/version/manage.rs](src-tauri/src/commands/version/manage.rs)（fix_version_files：`app_handle: tauri::AppHandle` → `app_handle: &tauri::AppHandle`，保留 `Emitter` trait；rename_version / get_selected_version / set_selected_version）
    - [commands/version/personalization.rs](src-tauri/src/commands/version/personalization.rs)（get_version_personalization / update_version_personalization）
  - 新增 [commands/version/mod.rs](src-tauri/src/commands/version/mod.rs) 的 `#[tauri::command] pub async fn version_list_manager(state, app, req)` 转发入口
- 前端：
  - 新增 [utils/api/version-list-manager.ts](src/utils/api/version-list-manager.ts)：`versionListManager(action, params)` 入口和 `VERSION_LIST_ACTIONS` 常量（17 个 action 全部大写蛇形命名，值为小写下划线），`VersionListAction` 类型
  - 修改 [utils/api/version.ts](src/utils/api/version.ts)：10 个 `invoke('xxx')` 改为 `versionListManager(VERSION_LIST_ACTIONS.XXX, ...)`（listVersions / listInstalledVersions / listInstalledVersionsWithType / listMcFolders / addMcFolder / removeMcFolder / switchMcFolder / renameMcFolder / uninstallVersion / getVersionEffectiveDir），保留 `downloadVersion` 走原 `invoke('download_version')` 调用，保留 `InstalledVersionInfo` / `McFolder` 类型定义
  - 修改 [utils/api/personalization.ts](src/utils/api/personalization.ts)：7 个 `invoke('xxx')` 改为 `versionListManager(VERSION_LIST_ACTIONS.XXX, ...)`（getVersionPersonalization / updateVersionPersonalization / fixVersionFiles / renameVersion / getSelectedVersion / setSelectedVersion / getVersionGameVersion），保留 `VersionPersonalization` / `PersonalizationUpdate` 类型定义，保留 mod 管理 / 脚本导出 / 预加载等其他命令原样不动；因本文件不再直接调用 `invoke`，移除未使用的 `invoke` 导入
- 收益：IPC 命令从 17 个收敛为 1 个，与 `image_cache_manager` / `version_mods_manager` / `version_launch_manager` 模式一致，降低注册和维护成本；后续新增版本列表/文件夹/管理/个性化相关命令只需在 `version_list_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 聚合 5 个 sdk + 7 个 skin IPC 命令为 sdk_manager / skin_manager 两个入口（参照 image_cache_manager 模式）
- 痛点：`commands::sdk` 模块的 5 个命令（get_platform_info / get_sdk_version / is_sdk_initialized / get_device_id / check_update_lite）和 `commands::skin` 模块的 7 个命令（get_skin_cape_info / get_skin_url / get_cape_url / upload_skin / equip_cape / unequip_cape / download_url_to_file）独立注册为 Tauri 命令，与 `image_cache_manager` / `meta_manager` / `tools_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/sdk_manager.rs](src-tauri/src/utils/sdk_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 5 个 action，5 个 action 均无参数（handler 内用 `_params` 忽略）；`get_platform_info` 不需要 state（handler 内用 `_state` / `_app`），其余 4 个用 `&state` 访问 `state.sdk` 锁
  - 新增 [utils/skin_manager.rs](src-tauri/src/utils/skin_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 7 个 action，4 个参数结构体（GetSkinUrlParams / UploadSkinParams / EquipCapeParams / DownloadUrlToFileParams）均使用 `#[serde(rename_all = "camelCase")]`；`download_url_to_file` 不需要 state（handler 内用 `_state` / `_app`），3 个图片缓存相关 action 用 `&app` 调 `image_cache::get_image_url`
  - 修改 [commands/sdk.rs](src-tauri/src/commands/sdk.rs)：去掉 5 个函数的 `#[tauri::command]`，`State<'_, AppState>` 改为 `&AppState`，新增 `#[tauri::command] pub async fn sdk_manager(state, app, req)` 转发入口，保留 `SdkStatus` 结构体导出
  - 修改 [commands/skin.rs](src-tauri/src/commands/skin.rs)：去掉 7 个函数的 `#[tauri::command]`，`State<'_, AppState>` 改为 `&AppState`、`AppHandle` 改为 `&AppHandle`（`get_skin_url` / `get_cape_url` 内的 `Some(app)` 改为 `Some(app.clone())`），新增 `#[tauri::command] pub async fn skin_manager(state, app, req)` 转发入口，保留 `invalidate_skin_cache` / `invalidate_cape_cache` 私有辅助函数
- 前端：
  - 新增 [utils/api/sdk-manager.ts](src/utils/api/sdk-manager.ts)：导出 `sdkManager(action, params)` 函数和 `SDK_ACTIONS` 常量（5 个 action，大写蛇形键 + 小写下划线值），`SdkAction` 类型
  - 新增 [utils/api/skin-manager.ts](src/utils/api/skin-manager.ts)：导出 `skinManager(action, params)` 函数和 `SKIN_ACTIONS` 常量（7 个 action），`SkinAction` 类型
  - 修改 [utils/api/sdk.ts](src/utils/api/sdk.ts)：`getPlatformInfo` / `getSdkVersion` 由 `invoke('xxx')` 改为 `sdkManager(SDK_ACTIONS.XXX)`，保留 `SdkStatus` 类型导入
  - 修改 [utils/api/skin.ts](src/utils/api/skin.ts)：7 个函数由 `invoke('xxx', {...})` 改为 `skinManager(SKIN_ACTIONS.XXX, {...})`，保留 `SkinInfo` / `CapeInfo` / `SkinCapeInfo` 类型定义和导出
  - 修改 [utils/api/java.ts](src/utils/api/java.ts)：`getDeviceId` 由 `invoke('get_device_id')` 改为 `sdkManager(SDK_ACTIONS.GET_DEVICE_ID)`（`get_device_id` 属于 SDK 命令，已随 sdk_manager 迁移；其他 Java 命令仍走 `javaManager`），移除未使用的 `invoke` 导入
- 收益：sdk/skin 模块的 12 个分散 Tauri 命令聚合为 2 个 IPC 入口（`sdk_manager` / `skin_manager`），与 `image_cache_manager` / `meta_manager` / `tools_manager` 模式统一，lib.rs 注册项减少 12 行；前端 `java.ts` 的 `getDeviceId` 一并迁移到 `sdkManager`，避免运行时找不到 `get_device_id` 命令

#### 聚合 10 个 version::mods IPC 命令为 1 个 version_mods_manager（参照 image_cache_manager 模式）
- 痛点：`commands::version::mods` 模块的 10 个 mod 管理命令（list/manage/install/watcher 四个子模块各 2-4 个）独立注册为 Tauri 命令，与 `image_cache_manager` / `meta_manager` / `version_progress_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/version_mods_manager.rs](src-tauri/src/utils/version_mods_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 10 个 action（list.rs 2 个 + manage.rs 2 个 + install.rs 4 个 + watcher.rs 2 个），5 个参数结构体（VersionIdParams / ToggleModParams / DeleteModParams / InstallModParams / RevealModFileParams）均使用 `#[serde(rename_all = "camelCase")]`
  - 改造 4 个原命令文件去掉 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`、`AppHandle` → `&AppHandle`：
    - [commands/version/mods/list.rs](src-tauri/src/commands/version/mods/list.rs)（is_version_modable / list_mods）
    - [commands/version/mods/manage.rs](src-tauri/src/commands/version/mods/manage.rs)（toggle_mod / delete_mod）
    - [commands/version/mods/install.rs](src-tauri/src/commands/version/mods/install.rs)（install_mod / open_mods_dir / reveal_mod_file / get_version_mods_dir）
    - [commands/version/mods/watcher.rs](src-tauri/src/commands/version/mods/watcher.rs)（watch_mods_dir / unwatch_mods_dir；unwatch_mods_dir 无参数无 state，handler 用 `_state, _app, _params`）
  - 同步 [commands/version/mods/helpers.rs](src-tauri/src/commands/version/mods/helpers.rs)：`get_mods_dir` 参数从 `&State<'_, AppState>` 改为 `&AppState`；preload.rs 和 loaders.rs 中现有调用通过 `State::Deref<Target = AppState>` 的 deref coercion 保持编译
  - 新增 [commands/version/mods/mod.rs](src-tauri/src/commands/version/mods/mod.rs) 的 `#[tauri::command] pub async fn version_mods_manager(state, app, req)` 转发入口
- 前端：
  - 新增 [utils/api/version-mods-manager.ts](src/utils/api/version-mods-manager.ts)：`versionModsManager(action, params)` 入口和 `VERSION_MODS_ACTIONS` 常量（10 个 action 全部大写蛇形命名，值为小写下划线）
  - 修改 [utils/api/personalization.ts](src/utils/api/personalization.ts)：Mod 管理区段的 10 个 `invoke('xxx')` 改为 `versionModsManager(VERSION_MODS_ACTIONS.XXX, ...)`，函数签名和外部调用点保持不变；区段内其他命令（个性化 / 选中版本 / 文件补全 / 脚本导出 / 预加载）保持原样
- 收益：IPC 命令从 10 个收敛为 1 个，与 `image_cache_manager` / `meta_manager` / `version_progress_manager` 模式一致，降低注册和维护成本；后续新增 mod 管理相关命令只需在 `version_mods_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 聚合 6 个 java IPC 命令为 1 个 java_manager（参照 image_cache_manager 模式）
- 痛点：`commands::java` 模块的 6 个命令（`detect_java` / `list_java` / `select_java_for_mc` / `get_java_requirements` / `check_java_compatible` / `download_java`）独立注册为 Tauri 命令，与 `image_cache_manager` / `meta_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/java_manager.rs](src-tauri/src/utils/java_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 6 个 action；4 个带参数 action 定义 `#[serde(rename_all = "camelCase")]` 参数结构体（`SelectJavaForMcParams` / `GetJavaRequirementsParams` / `CheckJavaCompatibleParams` / `DownloadJavaParams`），`detect_java` / `list_java` 无额外参数用 `handler!(state, _app, _params, { ... })`
  - 改造 [commands/java.rs](src-tauri/src/commands/java.rs)：去掉 6 个原函数的 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`、`AppHandle` → `&AppHandle`；新增 `#[tauri::command] pub async fn java_manager(state, app, req)` 转发入口
- 前端：
  - 新增 [utils/api/java-manager.ts](src/utils/api/java-manager.ts)：`javaManager(action, params)` 入口和 `JAVA_ACTIONS` 常量（6 个 action 全部大写蛇形命名）
  - 修改 [utils/api/java.ts](src/utils/api/java.ts)：6 个 `invoke('xxx')` 改为 `javaManager(JAVA_ACTIONS.XXX, ...)`，函数签名、类型定义和外部调用点保持不变
- 收益：IPC 命令从 6 个收敛为 1 个，与 `image_cache_manager` / `meta_manager` 模式一致，降低注册和维护成本；后续新增 Java 相关命令只需在 `java_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 聚合 4 个 config IPC 命令为 1 个 config_manager（参照 image_cache_manager 模式）
- 痛点：`commands::system::apply_config`（2 个：`get_config` / `apply_config`）和 `commands::system::config`（2 个：`get_config_value` / `set_config_value`）共 4 个独立 Tauri 命令，与 `image_cache_manager` / `meta_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/config_manager.rs](src-tauri/src/utils/config_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 4 个 action；4 个 action 均定义 `#[serde(rename_all = "camelCase")]` 参数结构体（`GetConfigParams` / `ApplyConfigParams` / `GetConfigValueParams` / `SetConfigValueParams`）
  - 改造 [commands/system/apply_config/mod.rs](src-tauri/src/commands/system/apply_config/mod.rs)：去掉 `get_config` / `apply_config` 的 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`，移除未使用的 `use tauri::State`
  - 改造 [commands/system/apply_config/apply.rs](src-tauri/src/commands/system/apply_config/apply.rs)：`apply_config_inner` 参数 `State<'_, AppState>` → `&AppState`，移除未使用的 `use tauri::State`
  - 改造 [commands/system/config.rs](src-tauri/src/commands/system/config.rs)：去掉 `get_config_value` / `set_config_value` 的 `#[tauri::command]` 标注，`set_config_value` 参数 `State<'_, AppState>` → `&AppState`；新增 `#[tauri::command] pub async fn config_manager(state, app, req)` 转发入口；`get_config_path` / `save_config_to_file` 不在本次聚合范围，保持原样
- 前端：
  - 新增 [utils/api/config-manager.ts](src/utils/api/config-manager.ts)：`configManager(action, params)` 入口和 `CONFIG_ACTIONS` 常量（4 个 action 全部大写蛇形命名）
  - 修改 [utils/api/config.ts](src/utils/api/config.ts)：4 个 `invoke('xxx')` 改为 `configManager(CONFIG_ACTIONS.XXX, ...)`，移除未使用的 `invoke` 导入，函数签名、类型定义和外部调用点保持不变
- 收益：IPC 命令从 4 个收敛为 1 个，与 `image_cache_manager` / `meta_manager` 模式一致，降低注册和维护成本；后续新增配置相关命令只需在 `config_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 聚合 7 个 version::launch + version::script_export IPC 命令为 1 个 version_launch_manager（参照 image_cache_manager 模式）
- 痛点：`commands::version::launch`（6 个：`launch_game` / `get_launch_progress` / `cancel_launch` / `stop_game` / `get_running_game` / `get_launch_history`）和 `commands::version::script_export`（1 个：`export_launch_script`）共 7 个独立 Tauri 命令，与 `image_cache_manager` / `meta_manager` / `version_progress_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/version_launch_manager.rs](src-tauri/src/utils/version_launch_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 7 个 action；`launch_game` 和 `export_launch_script` 定义 `#[serde(rename_all = "camelCase")]` 参数结构体（`LaunchGameParams` / `ExportLaunchScriptParams`），其余 5 个无参数 action 用 `handler!(state, _app, _params, { ... })`
  - 改造 [commands/version/launch/mod.rs](src-tauri/src/commands/version/launch/mod.rs)：去掉 6 个原函数的 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`、`AppHandle` → `&AppHandle`；`spawn_exit_watcher` 调用处改为 `app_handle.clone()`（借用转 owned）；新增 `#[tauri::command] pub async fn version_launch_manager(state, app, req)` 转发入口
  - 改造 [commands/version/launch/build_config.rs](src-tauri/src/commands/version/launch/build_config.rs) 和 [failure.rs](src-tauri/src/commands/version/launch/failure.rs)：helper 函数参数 `&State<'_, AppState>` → `&AppState`，移除未使用的 `use tauri::State`
  - 改造 [commands/version/script_export/mod.rs](src-tauri/src/commands/version/script_export/mod.rs)：去掉 `export_launch_script` 的 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`
- 前端：
  - 新增 [utils/api/version-launch-manager.ts](src/utils/api/version-launch-manager.ts)：`versionLaunchManager(action, params)` 入口和 `VERSION_LAUNCH_ACTIONS` 常量（7 个 action 全部大写蛇形命名）
  - 修改 [utils/api/launch.ts](src/utils/api/launch.ts)：6 个 `invoke('xxx')` 改为 `versionLaunchManager(VERSION_LAUNCH_ACTIONS.XXX, ...)`，函数签名、类型定义和外部调用点保持不变
  - 修改 [utils/api/personalization.ts](src/utils/api/personalization.ts)：`exportLaunchScript` 的 `invoke('export_launch_script')` 改为 `versionLaunchManager(VERSION_LAUNCH_ACTIONS.EXPORT_LAUNCH_SCRIPT, ...)`；其他命令保持原样
- 收益：IPC 命令从 7 个收敛为 1 个，与 `image_cache_manager` / `meta_manager` / `version_progress_manager` 模式一致，降低注册和维护成本；后续新增启动相关命令只需在 `version_launch_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 聚合 6 个 version::progress IPC 命令为 1 个 version_progress_manager（参照 image_cache_manager 模式）
- 痛点：`commands::version::progress` 模块的 6 个下载进度命令（`get_download_progress` / `is_downloading` / `reset_download_progress` / `cancel_download` / `pause_download` / `resume_download`）独立注册为 Tauri 命令，与 `image_cache_manager` / `meta_manager` 的 dispatcher 聚合模式不一致，注册和维护成本高。
- 后端：
  - 新增 [utils/version_progress_manager.rs](src-tauri/src/utils/version_progress_manager.rs)：用 `static DISPATCHER: Lazy<Dispatcher>` 注册 6 个 action，6 个 handler 均为 `handler!(state, _app, _params, { ... })`（仅需 state，无额外参数）
  - 改造 [commands/version/progress.rs](src-tauri/src/commands/version/progress.rs)：去掉 6 个原函数的 `#[tauri::command]` 标注，参数 `State<'_, AppState>` → `&AppState`；新增 `#[tauri::command] pub async fn version_progress_manager(state, app, req)` 转发入口
- 前端：
  - 新增 [utils/api/version-progress-manager.ts](src/utils/api/version-progress-manager.ts)：`versionProgressManager(action, params)` 入口和 `VERSION_PROGRESS_ACTIONS` 常量（6 个 action 全部大写蛇形命名）
  - 修改 [utils/api/system.ts](src/utils/api/system.ts)：下载进度查询区段的 6 个 `invoke('xxx')` 改为 `versionProgressManager(VERSION_PROGRESS_ACTIONS.XXX)`，函数签名和外部调用点保持不变；区段内其他系统命令保持原样
- 收益：IPC 命令从 6 个收敛为 1 个，与 `image_cache_manager` / `meta_manager` 模式一致，降低注册和维护成本；后续新增下载进度相关命令只需在 `version_progress_manager.rs` 的 DISPATCHER 中追加 register 即可

#### 修复外置账号未在皮肤站设置皮肤时弹窗 3D 预览空白且无提示
- 痛点：外置登录账号（AuthlibInjector）在皮肤站未设置皮肤时，yggdrasil API 返回 `textures: {}`，后端 `parse_skin_cape_info` 解析后 `skin_url=None`、`cape_url=None`。前端 `useSkinOperations.loadInfo()` 直接把 null 赋给 `skinUrl`，导致 SkinManager 弹窗的 3D 预览（SkinModel3D）显示"暂无皮肤"空状态，且无任何提示告知用户原因。
- 修改 [utils/default-skin.ts](src/utils/default-skin.ts)：新增 `STEVE_SKIN_URL` 常量导出，用于外置账号未设置皮肤时的顶替（与 yggdrasil 协议"未设置皮肤按 Steve 处理"一致，离线/微软账号仍按 UUID hash 分配）
- 修改 [composables/useSkinOperations.ts](src/composables/useSkinOperations.ts)：
  - 新增 `authlibUsingDefaultSkin` ref 标志，标识外置账号是否正在用默认皮肤顶替
  - `loadInfo` 外置账号分支：`data.skin_url` 为 null 时用 `STEVE_SKIN_URL` 顶上 `skinUrl`，并置 `authlibUsingDefaultSkin = true`；披风保持 null（不顶默认披风）
  - 每次 `loadInfo` 开头重置 `authlibUsingDefaultSkin = false`，避免状态残留
- 修改 [components/common/SkinManager.vue](src/components/common/SkinManager.vue)：
  - 解构 `authlibUsingDefaultSkin`，新增 AlertV2 提示（info 类型）：`当前账号未在皮肤站设置皮肤，已显示默认 Steve 皮肤。上传皮肤后将替换为此账号的形象。`
  - "删除皮肤"按钮在 `authlibUsingDefaultSkin=true` 时禁用，避免误删（当前 skinUrl 是默认皮肤而非服务器皮肤）
- 修改 [components/common/SkinAvatar.vue](src/components/common/SkinAvatar.vue)：外置账号分支无皮肤时回退到 `STEVE_SKIN_URL`（原为 UUID 哈希默认皮肤，与皮肤站"默认 Steve"行为不一致），保持头像与弹窗 3D 预览一致
- 收益：外置账号未设置皮肤时，弹窗 3D 预览与账号头像一致显示 Steve，顶部 info 提示告知用户原因，删除按钮禁用避免误操作

#### 降低 Java 搜索和 shell 调用日志级别为 debug
- 痛点：Java 搜索过程日志（候选数、步骤1-5、有效 Java）和 shell 调用日志（`[Shell] run_executable: javaw.exe -version`）默认 info 级别下刷屏，影响日志可读性。
- 修改 [minecraft/java/search.rs](src-tauri/src/minecraft/java/search.rs)：7 处 `log_info!` 改为 `log_debug!`（搜索开始/候选数/步骤1-5/有效Java）
- 修改 [minecraft/system/shell.rs](src-tauri/src/minecraft/system/shell.rs)：`run_executable_output` 的 `[Shell] run_executable` 日志由 `log_info!` 改为 `log_debug!`（Java 二进制探测等 shell 调用属于内部实现细节）
- 修改 [logger.rs](src-tauri/src/logger.rs)：`separator` 函数日志级别由 `Info` 改为 `Debug`，`========== Java Search ==========` 等分割线不再默认显示
- 收益：默认日志级别下 Java 搜索过程和 shell 调用日志不再刷屏，需要排查时切换到 debug 模式查看

#### 实现 yggdrasil 协议皮肤管理 API，修复外置账号皮肤按钮走离线 API 的 bug
- 痛点：外置登录账号（AuthlibInjector）点击皮肤按钮时，SkinManager 用 `isMicrosoft` 二值判断将其归入离线分支，调用 `setOfflineSkin` / `saveCustomSkin`，后端在 `offline_accounts` 中找不到 uuid 报错"离线账号不存在"。SkinAvatar 同样将外置账号当作离线账号，显示 UUID 哈希默认皮肤而非从 yggdrasil 服务器拉取。
- 后端（yggdrasil 皮肤管理 5 个端点）：
  - [authlib/types.rs](src-tauri/src/minecraft/auth/authlib/types.rs)：新增 `ProfileProperty`、`ProfileInfo`、`TexturesPayload`、`SkinCapeInfo` 等结构体，用于解析角色属性和材质信息（Base64 解码 textures property）
  - [authlib/client.rs](src-tauri/src/minecraft/auth/authlib/client.rs)：新增 `fetch_profile`（GET /sessionserver/session/minecraft/profile/{uuid}）、`upload_skin`、`delete_skin`、`upload_cape`、`delete_cape` 5 个 yggdrasil API 端点封装，以及 `parse_skin_cape_info` 从 ProfileInfo 解析皮肤/披风信息
  - [commands/auth/authlib.rs](src-tauri/src/commands/auth/authlib.rs)：新增 `authlib_get_skin_info`、`authlib_upload_skin`、`authlib_delete_skin`、`authlib_upload_cape`、`authlib_delete_cape` 5 个命令，统一从 `auth_storage.get_authlib_account` 取 access_token，PNG 文件由后端读取校验（避免前端引入 `@tauri-apps/plugin-fs` 依赖）
  - [utils/meta_manager.rs](src-tauri/src/utils/meta_manager.rs)：注册 5 个皮肤 action 到 dispatcher
- 前端（三分流重构：微软 / 外置 / 离线）：
  - [composables/useSkinOperations.ts](src/composables/useSkinOperations.ts)：重构为三分流，`loadInfo` / `pickAndUpload` 根据 `loginType` 分别调用 `authlibGetSkinInfo` / `authlibUploadSkin` / `authlibDeleteSkin` / `authlibUploadCape` / `authlibDeleteCape`（外置）、`getSkinCapeInfo` / `uploadSkin`（微软）、`getLocalSkinName` / `saveCustomSkin`（离线）；新增 `canUploadSkin` / `canUploadCape` computed 据 `uploadableTextures` 动态启用上传按钮
  - [components/common/SkinManager.vue](src/components/common/SkinManager.vue)：模板三分流，外置账号显示皮肤/披风上传删除面板（据 `uploadableTextures` 动态显示），不支持上传时提示"此 yggdrasil 服务器不支持上传皮肤或披风"
  - [components/common/SkinAvatar.vue](src/components/common/SkinAvatar.vue)：新增 `serverUrl` prop，`loadAvatar` 新增 `AuthlibInjector` 分支调用 `authlibGetSkinInfo` 从 yggdrasil 服务器拉取皮肤 URL，失败或无皮肤时回退到 UUID 哈希默认皮肤
  - [components/home/account-selector/AccountCard.vue](src/components/home/account-selector/AccountCard.vue)：修复 `loginType` 映射，将 '外置' 正确映射为 'AuthlibInjector'（原代码 `card.loginType === '正版' ? 'Microsoft' : 'Offline'` 将外置账号误归为离线），传递 `serverUrl` prop 供 SkinAvatar 调用 yggdrasil API
  - [utils/api/authlib.ts](src/utils/api/authlib.ts)：新增 `authlibGetSkinInfo` / `authlibUploadSkin` / `authlibDeleteSkin` / `authlibUploadCape` / `authlibDeleteCape` 5 个 API 封装
  - [types/auth.ts](src/types/auth.ts)：新增 `AuthlibSkinCapeInfo` 类型定义（snake_case 字段名匹配后端 Serialize 输出）
  - [utils/api/meta-manager.ts](src/utils/api/meta-manager.ts)：新增 5 个 `META_ACTIONS` 常量
  - [stores/auth.ts](src/stores/auth.ts)：`currentUser` 类型由 `AuthResult | null` 收紧为 `LocalAuthResult | null`，使 SkinManager 中 `authStore.currentUser?.server_url` 访问有类型保障（后端所有登录方法均返回 `LocalAuthResult`，结构兼容赋值不受影响）
- 收益：外置账号可查看/上传/删除皮肤和披风（据服务器 `uploadableTextures` 权限动态显示），账号卡片头像正确从 yggdrasil 服务器拉取皮肤，微软/离线账号功能不受影响

#### 抽象通用 action 分发器（dispatcher），meta_manager 和 tools_manager 共用
- 痛点：meta_manager 和 tools_manager 各自维护冗长的 match 语句，新增 action 要改两处逻辑，且无法复用分发机制。
- 新增 [utils/dispatcher.rs](src-tauri/src/utils/dispatcher.rs)：
  - `ActionRequest`：统一请求体（替代原 `MetaRequest` / `ToolsRequest`）
  - `Dispatcher`：注册式分发器，`register(action, handler)` + `dispatch(state, app, req)`
  - `Handler`：统一签名，用 owned `AppState` + `AppHandle` 参数避免 HRTB 复杂性
  - `handler!` 宏：自动包装 `Box::pin(async move { ... })`，简化注册代码
- 配套 [state/app.rs](src-tauri/src/state/app.rs)：`AppState` 派生 `Clone`（所有字段均为 `Arc<...>`，克隆开销极低），让 dispatcher 能以 owned 参数接收 state
- 重构 [utils/meta_manager.rs](src-tauri/src/utils/meta_manager.rs)：23 个 auth action 改为 `static DISPATCHER: Lazy<Dispatcher>` + register 调用
- 重构 [commands/tools/mod.rs](src-tauri/src/commands/tools/mod.rs)：25+ 个 tools action 改为注册式
- 删除 `MetaRequest` 和 `ToolsRequest`，统一使用 `ActionRequest`
- 收益：新增 action 只需一行 register，无需改 match；分发机制复用，降低维护成本

#### 聚合 23 个 auth IPC 命令为 1 个 meta_manager（参照 tools_manager 模式）
- 痛点：auth 相关 IPC 命令多达 23 个（离线 6 + 微软 9 + authlib 6 + 会话 2），注册和维护成本高。
- 后端：
  - 新增 [utils/meta_manager.rs](src-tauri/src/utils/meta_manager.rs)：统一分发逻辑，接收 `MetaRequest { action, params }`，按 action 分发到各子模块函数。Params 结构体统一用 `#[serde(rename_all = "camelCase")]`，与前端约定一致
  - 新增 [commands/auth/mod.rs](src-tauri/src/commands/auth/mod.rs) 的 `meta_manager` IPC 命令，只负责转发请求到 utils 工具
  - 去掉原有 23 个命令的 `#[tauri::command]` 标注，改为普通 `pub async fn` 供 dispatch 调用；参数类型 `State<'_, AppState>` → `&AppState`、`AppHandle` → `&AppHandle`（涉及 offline.rs / microsoft.rs / authlib.rs / account/{ms,offline,session}.rs，以及 microsoft.rs 的 complete_login 辅助函数和 authlib.rs 的 authlib_relogin_with_password 辅助函数）
  - [lib.rs](src-tauri/src/lib.rs) 删除 23 个原命令注册，只注册 `commands::auth::meta_manager` 一个命令
- 前端：
  - 新增 [utils/api/meta-manager.ts](src/utils/api/meta-manager.ts)：统一 API 入口 `metaManager(action, params)` 和 `META_ACTIONS` 常量
  - 迁移所有调用点：[utils/api/auth.ts](src/utils/api/auth.ts)（17 个 invoke）和 [utils/api/authlib.ts](src/utils/api/authlib.ts)（6 个 invoke）全部改用 `metaManager`，其他业务文件通过 `@/utils/tauri` 中转调用，无需改动
- 收益：IPC 命令从 23 个收敛为 1 个，与 tools_manager 模式一致，降低注册和维护成本；后续新增 auth 命令只需在 `meta_manager.rs` 的 dispatch 中追加分支即可

#### 修复 authlib 登录报错 "missing field access_token"
- 问题：外置登录时服务器返回 200 但反序列化失败，报 `missing field access_token at line 1 column 1382`。
- 根因：[authlib/types.rs](src-tauri/src/minecraft/auth/authlib/types.rs) 的 `AuthResponse` 等 6 个结构体
  缺少 `#[serde(rename_all = "camelCase")]` 标注。yggdrasil 协议规范要求字段为 camelCase
  （如 `accessToken`），但 serde 默认按 Rust 字段名 snake_case 查找（`access_token`），
  导致 required 字段找不到触发反序列化错误。
- 修复：给 `AuthenticateRequest`、`RefreshRequest`、`ValidateRequest`、`AuthResponse`、
  `ServerMetadata` 5 个结构体添加 `#[serde(rename_all = "camelCase")]`。
  `Profile`、`ProfileId`、`Agent` 字段名无下划线不受影响，无需标注。
- 对比：项目其他模块（curseforge、microsoft、launcher_profiles）均已正确使用 rename_all，
  唯独 authlib 模块遗漏，本次修复对齐项目约定。

#### 外置登录服务器地址自动补全（用户只需输入域名或皮肤站页面地址）
- 痛点：原先用户必须输入完整路径 `https://littleskin.cn/api/yggdrasil`，太麻烦。
- 新增 [utils/authlib-url.ts](src/utils/authlib-url.ts)：yggdrasil 服务器 URL 规范化工具，按以下规则自动补全：
  1. 去除首尾空白和首尾斜杠
  2. 若无 `http://` 或 `https://` 协议头，自动补 `https://`
  3. 若路径以 `/api/yggdrasil` 结尾，直接使用
  4. 若路径为空或为皮肤站页面路径（`/user`、`/index`、`/register`、`/login`、`/skin` 等），
     去掉原路径，替换为 `/api/yggdrasil`（解决用户粘贴皮肤站页面地址的问题）
  5. 其它非标准路径保留用户输入（可能是自建服务器）
- 修改 [ExternalLoginPanel.vue](src/components/common/ExternalLoginPanel.vue)：
  - placeholder 改为 `如 littleskin.cn 或 https://littleskin.cn`
  - `fetchMeta` 和 `handleLogin` 使用 `computed` 的 `normalizedUrl` 而非原始输入
  - 输入框下方实时显示"将使用：https://xxx/api/yggdrasil"提示，让用户知道最终请求地址
  - 底部说明改为"输入域名即可，自动补全协议和 /api/yggdrasil 路径"
- 支持的输入形式示例：
  - `littleskin.cn` → `https://littleskin.cn/api/yggdrasil`
  - `https://littleskin.cn` → `https://littleskin.cn/api/yggdrasil`
  - `littleskin.cn/api/yggdrasil` → `https://littleskin.cn/api/yggdrasil`
  - `https://littleskin.cn/user` → `https://littleskin.cn/api/yggdrasil`（自动去掉 /user）
  - `https://littleskin.cn/index` → `https://littleskin.cn/api/yggdrasil`（自动去掉 /index）
  - `https://littleskin.cn/skin/edit` → `https://littleskin.cn/api/yggdrasil`（前缀匹配）
  - `https://example.com/custom` → 保留为 `https://example.com/custom`（非标准路径保留）

#### 修复登录页 Tab 切换动画僵硬 + 切换外置登录后内容空白
- 问题 1：登录页切换登录方式时内容瞬间替换，无过渡动画，体验僵硬。
- 问题 2：切换到外置登录 Tab 后，再切回离线/微软 Tab 内容空白不显示。
- 根因：[ExternalLoginPanel.vue](src/components/common/ExternalLoginPanel.vue) 内部包含
  `<Teleport>`（ProfileSelectModal 弹窗），`<Transition mode="out-in">` 的 leave 钩子
  会等待 CSS transitionend 事件，Teleport 可能干扰事件触发，导致 leave 卡住、
  新内容不 enter，表现为切换后空白。
- 修复：
  - [SubTabBar.vue](src/components/common/SubTabBar.vue) 指示线改为所有 Tab 均渲染 span，
    通过 `opacity` + `scaleX` + `transition-all` 实现平滑淡入淡出 + 缩放过渡（200ms ease-out），
    替代原先 `v-if` 瞬间 mount/unmount。
  - [Login.vue](src/views/Login.vue) 放弃 `<Transition>` 组件，改用 `:key="activeTab"` 触发
    Vue 重新挂载 + CSS `@keyframes` animation 自动播放。不依赖 transitionend 事件，
    不受 Teleport 干扰，稳定可靠。动画效果为 opacity + translateY 8px（200ms ease-out），
    与指示线动画同步。同时尊重 `prefers-reduced-motion` 设置。
  - [ExternalLoginPanel.vue](src/components/common/ExternalLoginPanel.vue) 用外层
    `<div class="space-y-3">` 包裹主表单和 ProfileSelectModal，形成单一根节点，
    消除 Vue Transition 的多根节点警告。

#### 修复 network.rs 延迟测试绕过全局 HTTP 客户端
- 问题：[commands/tools/network.rs](src-tauri/src/commands/tools/network.rs) 的 `latency_test` 函数
  自行用 `Client::builder().timeout(10s).build()` 构造客户端，未走 `crate::http::get_client()`，
  导致用户配置的代理和项目统一 User-Agent 对测速功能不生效。
- 修复：改用 `crate::http::get_client()` 获取全局客户端，在每个请求上附加
  `.timeout(Duration::from_secs(10))` 保留测速超时设定。删除 `use reqwest::Client` 和
  `use std::sync::Arc`（不再需要手动 Arc 包裹，reqwest::Client 内部已是 Arc）。
- 收益：测速功能现在自动应用用户配置的代理和统一 User-Agent，与其它 HTTP 请求行为一致。

#### 重构 authlib 请求层：URL 收口到 sources.rs，请求统一走 http.rs
- 背景：[authlib/client.rs](src-tauri/src/minecraft/auth/authlib/client.rs) 原先在文件内
  硬编码 `https://authlib-injector.yushi.moe/artifact/latest.json` 等 URL，
  并自行实现了 `fetch_text`、`download_bytes` 等请求函数，与项目约定不符
  （[sources.rs](src-tauri/src/minecraft/sources.rs) 顶部明确要求"所有远程 URL 必须在此文件定义常量"）。
- 变更：
  - [sources.rs](src-tauri/src/minecraft/sources.rs) 新增 `AUTHLIB_INJECTOR_OFFICIAL`、
    `AUTHLIB_INJECTOR_LATEST_PATH`、`AUTHLIB_INJECTOR_BMCLAPI` 常量，
    及 `authlib_injector_meta_url_official()` / `authlib_injector_meta_url_mirror()` 构造函数。
  - [http.rs](src-tauri/src/http.rs) 新增 3 个通用函数：
    `get_text_with_status`（GET 返回状态码+文本）、
    `post_json_with_status`（POST JSON 返回状态码+文本）、
    `fetch_bytes`（GET 二进制）。
  - [authlib/client.rs](src-tauri/src/minecraft/auth/authlib/client.rs) 删除自实现的
    `client()`、`fetch_text`、`download_bytes` 函数，所有请求改走 `crate::http` 模块；
    URL 改用 `sources` 模块常量。`YggdrasilError` 业务错误转换保留不变。
- 收益：URL 与请求逻辑收口到统一模块，便于后续维护和镜像源切换；

#### 再次修复主页「添加账号」按钮点击无反应（深层原因）
- 问题：上一轮仅修复了路由守卫层面，实际仍存在多个深层原因导致点击无反应：
  1. [useSwipeNavigation.ts](src/composables/useSwipeNavigation.ts) `onPointerDown` 使用
     `target.closest('button')` 检测交互元素，但 SVG 子元素（如 `<path>`）的 `closest`
     在 Tauri WebView 中可能不跨越 SVG-HTML 边界，导致点击「添加账号」按钮内的图标时
     被错误识别为拖拽起点，触发 `setPointerCapture` 劫持 pointer 事件。
  2. [useAccountCards.ts](src/composables/useAccountCards.ts) `watch(cards)` 逻辑缺陷：
     当 `currentIndex` 指向「添加账号」卡片（末尾）时，`currentCard` 为 `undefined`，
     `!currentCard?.isActive` 为 `true`，导致账号列表异步加载触发 cards 变化时，
     `currentIndex` 被强制重置回 active 卡片，用户刚切到「添加账号」卡片就被拉回。
  3. `switchAccount`/`removeAccount` 未处理 `'外置'` 类型，外置账号无法切换/删除。
  4. `onMounted` 未调用 `loadAuthlibAccounts`，外置登录账号不显示在卡片栏。
- 修复：
  - `onPointerDown` 改用 `e.composedPath()` 遍历事件路径，按 `tagName` 检测交互元素，
    可靠跨越 SVG-HTML 边界。
  - `watch(cards)` 增加 `currentIndex === newCards.length` 提前返回，避免「添加账号」
    卡片被误重置。
  - `switchAccount`/`removeAccount` 新增 `'外置'` 分支，通过 `serverUrl + uuid` 双键定位。
  - `onMounted` 补充 `authStore.loadAuthlibAccounts()` 调用。
  - `addAccount` 添加 `try/catch` 捕获 `NavigationFailure`，便于排查静默失败。

#### 修复版本选择页 FolderSidebar 调用未定义 invoke 报错
- 问题：[FolderSidebar.vue](src/views/version-select/FolderSidebar.vue) 第 38、104 行
  直接调用 `invoke<string>('get_game_dir')`，但未导入 `invoke`，
  导致进入版本选择页时前端报错 `ReferenceError: invoke is not defined`，文件夹列表加载失败。
- 修复：改用项目已有的封装 `tauri.getGameDir()`（来自 [utils/api/system.ts](src/utils/api/system.ts)），
  与 [useVersionSettings.ts](src/composables/useVersionSettings.ts) 中的调用方式保持一致。

#### 修复主页「添加账号」按钮点击无反应
- 问题：[AccountSelector.vue](src/components/home/AccountSelector.vue) `addAccount` 调用
  `router.push('/login')`，但 [router/index.ts](src/router/index.ts) 路由守卫第 85 行
  `to.path === '/login' && authStore.isLoggedIn` 会把已登录用户重定向回 `/apps`，
  导致点击「添加账号」按钮视觉上"没反应"。
- 修复：`addAccount` 改为 `router.push({ path: '/login', query: { add: '1' } })`，
  路由守卫识别 `query.add === '1'` 时跳过重定向，放行进入登录页。

#### 修复种子地图 hover 检测不响应问题
- 问题：[useSeedMap.ts](src/views/tools/data/useSeedMap.ts) `pointermove` 节流实现有 bug：
  throttle 期间新位置被丢弃，超时回调使用闭包捕获的首次事件 `e.pixel`（已过期），
  导致用户快速移动鼠标并停下时，停下的位置不被检测，hover 无反应，必须再动一下才能触发。
- 修复：始终更新 `lastPixel` 变量，超时回调改用 `lastPixel` 而非闭包捕获的 `e.pixel`，
  保证用户停下时的最新位置被检测。

#### 修复 seedmap constants.ts 使用废弃 glob 语法触发 Vite 警告
- 问题：[constants.ts](src/utils/seedmap/constants.ts) 使用了 Vite 2-4 的 `as: 'url'` 语法，
  Vite 5 已废弃该语法，每次进入工具页面都会在后端控制台弹出
  `The glob option "as" has been deprecated in favour of "query"` 警告。
- 修复：移除三层 fallback（as:'url' / query+import / query）冗余写法，
  统一使用 Vite 5 推荐语法 `query: '?url', import: 'default'`，直接返回 url 字符串。

#### 整合包安装新增 LauncherPack 与 Compress 格式支持
- 新增：[types.rs](src-tauri/src/commands/community/install/types.rs)
  `ModpackFormat` 枚举新增 `LauncherPack`（带启动器整合包）和 `Compress`（普通压缩包兜底）两个变体，
  与业界同类整合包分发格式对齐。
- 检测：[concurrent.rs](src-tauri/src/commands/community/install/concurrent.rs)
  `detect_modpack_format` 在常规 manifest 文件扫描未命中后，依次扫描：
  1. 根目录/一级子目录的 `modpack.zip` 或 `modpack.mrpack` → 识别为 `LauncherPack`；
  2. 含 `.minecraft/` 目录前缀 → 识别为 `Compress`，前缀作为 overrides 解压依据。
- 递归安装：[modpack.rs](src-tauri/src/commands/community/install/modpack.rs)
  `install_local_modpack` 入口预检测到 `LauncherPack` 时，提取内层整合包到
  `.tmp_launcher_extract/` 临时目录后将 `archive_path` 替换为内层路径继续走主流程，
  避免递归调用 async fn（`E0733`）和 `download_state` 重复重置。函数返回前清理临时文件。
- Compress 解压：[concurrent.rs](src-tauri/src/commands/community/install/concurrent.rs)
  `build_overrides_prefixes` 为 `Compress` 格式返回 `.minecraft/` 前缀，
  `extract_overrides` 去掉该前缀后将内容解压到 instance 目录。
- 前端：[community.ts](src/types/community.ts) `ModpackFormat` 类型新增 `'launcherpack' | 'compress'`；
  [useDragDrop.ts](src/composables/useDragDrop.ts) `formatToLabel` 新增"带启动器整合包"和"普通压缩包"标签。

#### 显式拒绝 .rar 格式整合包
- 后端：[helpers.rs](src-tauri/src/commands/community/install/helpers.rs)
  新增 `validate_modpack_extension`，显式拒绝 `.rar`（无开源解压库支持），
  提示用户解压后重新压缩为 `.zip`。在 `install_modpack` 和 `install_local_modpack` 入口校验。
- 前端：[useDragDrop.ts](src/composables/useDragDrop.ts)
  拖拽 `.rar` 文件时显示"RAR 格式不支持"错误弹窗，提示解压后重新压缩为 zip。

#### 清理代码中 PCL2 相关注释
- 清理：移除 [useDragDrop.ts](src/composables/useDragDrop.ts)、
  [MoLaunchIntro.vue](src/components/about/MoLaunchIntro.vue)、
  [DragOverlay.vue](src/components/common/DragOverlay.vue)、
  [auth/mod.rs](src-tauri/src/minecraft/auth/mod.rs)、
  [language.rs](src-tauri/src/minecraft/language.rs)、
  [build_config.rs](src-tauri/src/commands/version/launch/build_config.rs)、
  [skin_resourcepack.rs](src-tauri/src/minecraft/launch/skin_resourcepack.rs)、
  [chunk/probe.rs](src-tauri/src/minecraft/download/chunk/probe.rs)、
  [tools/memory.rs](src-tauri/src/commands/tools/memory.rs)、
  [install/mmc.rs](src-tauri/src/commands/community/install/mmc.rs)
  中代码注释里对 PCL2 的引用，仅保留 [CreditsTab.vue](src/views/settings/more/CreditsTab.vue)
  鸣谢页面与 [licenses.txt](src-tauri/resources/about/licenses.txt) 第三方许可声明。
- 补充：进一步将版本目录下的 `PCL/` 子目录重命名为 `MoLaunch/`，
  Logo 字段从 `PCL\Logo.png` 改为 `MoLaunch\Logo.png`。
  涉及 [modpack_stages.rs](src-tauri/src/commands/community/install/modpack_stages.rs)
  `migrate_modpack_config` 与 `copy_external_logo`、
  [types.rs](src-tauri/src/commands/community/install/types.rs)、
  [mmc.rs](src-tauri/src/commands/community/install/mmc.rs)、
  [community.ts](src/types/community.ts) 中残留的 PCL 路径引用全部清除。

#### 优化版本选择页分组卡片为可折叠展开动画
- 优化：[VersionSelect.vue](src/views/VersionSelect.vue) 版本分组卡片改为可折叠，
  默认全部收起，点击标题栏展开/收起。标题栏带分组类型图标（草方块/铁砧/Fabric 等，
  复用 `typeMetaMap.icon`）+ 分组标题 + 数量徽标 + ChevronDown 旋转箭头。
  内容区使用 `grid-template-rows 0fr→1fr` 平滑高度过渡动画，
  与 [MoLaunchIntro.vue](src/components/about/MoLaunchIntro.vue) 的展开动画一致。
- 自动展开：进入页面时自动展开当前选中版本所在的分组，用户无需手动查找。
- 选中标识：选中项用左侧 2px primary 色边框 + 浅色背景标识（替代原整行浅色背景）。
- 样式调整：分组标题栏图标从 `h-4 w-4` 放大为 `h-5 w-5` 与标题更搭；
  padding 从 `px-5 py-4` 收紧为 `px-4 py-3`，卡片更紧凑不臃肿。
- 背景：原实现每个分组默认展开全部版本，版本多时列表过长。改为默认收起后，
  用户按需展开感兴趣的分组，列表更紧凑。

#### 修复整合包安装 stage_index 硬编码导致进度显示错位
- 背景：本地拖拽安装整合包时，mod 下载进度错误显示在"复制配置文件"阶段，
  而"下载 MOD"阶段一直为 0%。根因是 stage_index 硬编码，未区分在线/本地两种 stage 结构。
- 问题：[curseforge.rs](src-tauri/src/commands/community/install/curseforge.rs)
  `install_cf_mods` 硬编码 `stage_index=2`、[modrinth.rs](src-tauri/src/commands/community/install/modrinth.rs)
  `install_mr_files` 硬编码 `stage_index=2`、[concurrent.rs](src-tauri/src/commands/community/install/concurrent.rs)
  `extract_overrides` 硬编码 `set_stage_bytes(3, ...)`。
  这组值只对"在线安装"（4 stages：0下载包/1解析/2下载MOD/3复制配置）正确，
  对"本地拖拽"（3 stages：0解析/1下载MOD/2复制配置）错位一位。
- 修复：三个函数都改为接受 `stage_index: usize` 参数，由调用方传入。
  [modpack.rs](src-tauri/src/commands/community/install/modpack.rs)
  在线安装传 2/3，本地拖拽传 1/2。

#### 修复 CF 批量查询 mod info 用 GET /mods 返回 EOF 导致触发保底 URL 拼接
- 背景：整合包安装时 `batch_get_mod_slugs` 调用 `GET /v1/mods?modIds=...`
  批量查询 mod slug，CF 官方 API 返回空响应（`EOF while parsing a value at line 1 column 0`），
  导致所有 mod 拿不到 slug，文件名无法应用 `community_filename_format` 翻译格式，
  触发 `construct_cf_edge_url` 保底 URL 拼接（拼接出的 URL 直接 404）。
- 根因：CF 官方 API `GET /v1/mods` 的 modIds 查询参数对数量敏感，
  即使分批 50 个也偶发返回空响应；CF 官方推荐的批量查询接口是 `POST /v1/mods`，
  请求体 `{"modIds":[...]}`，与 `POST /v1/mods/files` 一致，支持大批量 ID。
- 修复：[mod.rs](src-tauri/src/minecraft/community/curseforge/mod.rs)
  `batch_get_mod_slugs` 从 `cf_get("/mods?modIds=...")` 改为 `cf_post("/mods", {"modIds":[...]})`，
  与 `fingerprint_search` 使用相同的 POST 接口。批次大小从 50 提升到 250，
  平衡请求数与单次请求体大小。部分批次失败时只记录 warn 日志，不阻断整体查询。

#### 优化整合包安装 mod 文件名处理与镜像源 Key 检查
- 优化 1：[helpers.rs](src-tauri/src/commands/community/install/helpers.rs)
  `apply_filename_format` 过滤译名中 Windows 文件名非法字符（`< > : " / \ | ? *` 及控制字符），
  替换为下划线。修复 mcmod 译名含 `:` 等字符导致 `std::fs::File::create` 报 os error 的问题。
- 优化 2：[curseforge.rs](src-tauri/src/commands/community/install/curseforge.rs)
  与 [modrinth.rs](src-tauri/src/commands/community/install/modrinth.rs)
  移除 mod 文件名重命名日志（每个 mod 一条日志过于刷屏）。
- 优化 3：[modpack.rs](src-tauri/src/commands/community/install/modpack.rs)
  `install_modpack` 与 `install_local_modpack` 的 CF API Key 前置检查：
  `source=0` 强制镜像时跳过检查（镜像站 mod.mcimirror.top 自带 Key 请求 CF，
  用户无需配置自己的 Key 即可使用 /mods/files 等需要 Key 的接口）。
  错误提示增加"或将下载源切换为「尽量镜像」使用镜像站"的引导。

#### 安全兜底：镜像源域名强制不附加 CF API Key
- 背景：用户担心 CurseForge API Key 泄露给镜像站。虽然上层 `get_cf_config` 在
  source=0 强制镜像时已返回 `api_key=None`，source=1 回退镜像时也显式传 `None`，
  但缺乏底层兜底，未来改动可能误把 key 带到镜像 URL。
- 修复：[http.rs](src-tauri/src/minecraft/community/curseforge/http.rs)
  `build_cf_request` 与 `build_cf_post_request` 在附加 `x-api-key` header 前检查
  URL 域名：包含 `mcimirror.top` 时强制不附加 Key。镜像站自带 Key 请求 CF，
  不需要用户 Key。即使上层逻辑误传 key 给镜像 URL，底层也会兜底剥离。

#### 修复 CF CDN 分片下载 Range 404（supports_range 改用 GET + Range 检测）
- 背景：CF CDN（edge.forgecdn.net）对 Range 请求返回 404，但 `supports_range`
  用 HEAD 请求检测 `accept-ranges` header，CF CDN HEAD 虚假返回 `accept-ranges: bytes`，
  导致代码误判支持分片，实际 GET + Range 返回 404，分片必然失败。
  日志显示每个 mod 4 个 chunk 全部 404，浪费近 2 分钟才回退单流。
- 根因：HEAD 请求的 `accept-ranges` header 不能反映服务端对实际 Range 请求的响应。
  PCL2 不用 HEAD 预检，而是首线程不带 Range 拿 FileSize，后续线程带 Range 校验
  ContentLength，Range 失败时切换源或回退单线程。
- 修复：[probe.rs](src-tauri/src/minecraft/download/chunk/probe.rs)
  `supports_range` 从 HEAD + `accept-ranges` 改为 GET + `Range: bytes=0-0`，
  检查 HTTP 206 Partial Content 状态码。206 = 支持 Range，200/404/其他 = 不支持。
  与 PCL2 的 GET + Range 动态检测策略一致，准确反映服务端真实行为。
  CF CDN GET + Range 返回 404 时 `supports_range` 返回 false，直接走单流下载，
  避免分片 404 浪费时间。

#### 修复分片下载 404 后重试又走分片浪费近 2 分钟
- 背景：CF CDN 对部分文件的 Range 请求返回 404，但完整 GET 请求返回 200。
  分片 404 失败后回退单流，但单流也失败（见下条），重试时又走分片又 404，
  浪费近 2 分钟才凑巧成功。日志显示 "尝试 1/3" → "尝试 2/3" 间隔 1分49秒。
- 修复：[single.rs](src-tauri/src/minecraft/download/downloader/single.rs)
  分片返回 404 时设置 `chunk_disabled = true`，后续重试直接跳过分片探测，
  走单流下载。日志输出 "分片返回 404，禁用分片改走单流" 便于排查。

#### 修复单流下载整体超时 5s 对大文件不够导致回退失败
- 背景：分片 404 后回退单流，但 `stream.rs` 用 reqwest `.timeout(5s)` 设置整体超时，
  89.8MB 文件 5 秒下载不完，单流也超时失败。导致大文件需要多次重试凑巧成功。
- 根因：reqwest 的 `.timeout()` 是整个请求的超时，包括响应体流式读取。
- 修复：[stream.rs](src-tauri/src/minecraft/download/downloader/stream.rs)
  - 连接 + 响应头阶段：用 `tokio::time::timeout(timeout, send())` 包裹，
    保持 5s/10s 短超时快速失败触发 URL 回退
  - body 流式读取阶段：改用"无数据流动 15s 超时"（与 chunk 下载一致），
    大文件慢速网络不受影响，只有真断流才会失败

#### 修复 CurseForge 整合包 mods 不下载的关键 bug
- 背景：用户反馈 CF 整合包（如 RLCraft 2.9.3）"安装完成"但 mods 目录为空。
  对比 PCL2 源码（`ModModpack.vb` InstallPackCurseForge + `ResourceVersion.vb`
  FromPlatformJson）后发现四个关键 bug。
- Bug 1：[curseforge.rs](src-tauri/src/commands/community/install/curseforge.rs)
  `CfManifestFile` serde 字段映射错误：
  - `#[serde(rename_all = "camelCase")]` 把 Rust `file_id` 映射到 JSON `fileId`（小写 d），
    但 CF 官方 manifest.json 用 `fileID`（大写 ID），`fileId` ≠ `fileID` 导致反序列化失败，
    所有 `file_id`/`project_id` 变成 None，`filter_map` 过滤后空 Vec，
    触发"CF manifest 无有效 file_id，跳过依赖下载"日志。
  - 修复：去掉 `rename_all = "camelCase"`，改用 `#[serde(alias = "fileID", alias = "fileId")]`
    和 `#[serde(alias = "projectID", alias = "projectId")]` 兼容大写 ID（CF 官方）
    和小写 id（部分第三方工具）两种写法。
- Bug 2：[curseforge.rs](src-tauri/src/commands/community/install/curseforge.rs)
  `CfFileEntry` serde 字段映射错误：
  - CF API `/v1/mods/files` 返回的 file id 字段名是 `id`（不是 `fileId`），
    参考 PCL2 `ResourceVersion.FromPlatformJson` 中 `Data("id")`。
    原 `rename_all = "camelCase"` 把 `file_id` 映射到 `fileId`，不匹配 `id`，
    导致反序列化失败 `missing field fileId`。
  - 修复：`#[serde(rename = "id")]` 把 `file_id` 映射到 JSON `id`。
- Bug 3：[helpers.rs](src-tauri/src/commands/community/install/helpers.rs)
  `construct_cf_edge_url`（downloadUrl 为空时的 CDN 直链兜底）：
  - 原 `split_at(len-4)` 拆分方向反了，应为 `split_at(4)`（PCL2 Substring(0,4)/Substring(4)）。
    例如 fileId=2725062 原逻辑拼成 `files/272/5062`（错），正确应为 `files/2725/62`。
  - 原格式串漏掉 `file_name`，拼出的 URL 指向目录而非文件，下载必失败。
  - 修复：`split_at(4)` + 余位 `parse::<i64>()` 去前导 0（与 PCL2 CInt 等价）
    + 补上 `file_name`，最终格式 `{base}/files/{前4位}/{余位去0}/{file_name}`。
- Bug 4：[curseforge.rs](src-tauri/src/commands/community/install/curseforge.rs)
  `install_cf_mods` 对 `batch.data` 为空时静默成功：
  - `download_files_concurrent` 对空 `files` 列表直接返回 `Ok(())`（[concurrent.rs:28-32](src-tauri/src/commands/community/install/concurrent.rs#L28-L32)），
    导致镜像源（mod.mcimirror.top）不支持 `/mods/files` 批量查询返回空 data 时，
    `install_cf_mods` "成功"但 0 个 mod 下载，整合包"安装完成"而 mods 目录为空。
  - 修复：在 `cf_post` 返回后增加空 data 校验，`batch.data.is_empty()` 时返回
    `Err`，提示用户切换下载源到「缓慢时换镜像」或「尽量官方」（镜像源可能不支持
    `/mods/files` POST 批量查询，需走官方 API）。
- 参考 PCL2：PCL2 用同样的 `POST /v1/mods/files` 批量查询，`downloadUrl` 为空时
  用 fileId 拼 `edge.forgecdn.net/files/{前4}/{余}/{FileName}` 兜底，并生成多个
  CDN 域名变体（edge/media/mediafilez/overwolf 互换）+ MCIM 镜像源顺序尝试。
- 验证：cargo check 0 errors，tsc 0 errors。需测试 RLCraft 2.9.3 等标准 CF 整合包
  安装后 mods 目录是否有 170+ 个 jar 文件。

#### 修复分片下载取消后仍重试的问题
- 问题：用户在下载管理页暂停后取消任务，分片下载的 chunk 返回"下载已取消"错误，
  但 [single.rs](src-tauri/src/minecraft/download/downloader/single.rs) 的重试循环
  把它当作普通失败继续重试 max_retries=3 次，直到重试次数用完才停止，浪费时间
  且日志刷屏 `[WARN] chunk N 失败: 下载已取消`。
- 修复（[single.rs](src-tauri/src/minecraft/download/downloader/single.rs)）：
  1. `while attempt < max_retries` 循环开头检查 `cancel_flag`，已取消时
     `break 'url_loop` 跳出整个 URL 循环，不再重试。
  2. 最终返回前检查 `cancel_flag`，已取消时返回 `error: "下载已取消"` 而非
     `"所有下载源均失败"`，让上层准确判断取消状态。
- 验证：cargo check 0 errors。需测试下载管理页暂停→取消后日志是否立即停止，
  不再出现重试警告。

#### 修复 CurseForge 整合包 manifest.json 解析失败（missing field projectId）
- 问题：部分 CF 整合包 manifest.json 的 files 项缺失 `projectID` 字段，
  导致 `CfManifestFile.project_id: i64` 反序列化失败，报错
  `missing field projectId at line 21 column 5`。
- 修复（[curseforge.rs](src-tauri/src/commands/community/install/curseforge.rs)）：
  - `CfManifestFile.project_id` 改为 `Option<i64>` + `#[serde(default)]`，兼容缺失字段。
  - `install_cf_mods` 中 `project_ids` 用 `filter_map(|f| f.project_id)` 过滤 None。
  - `file_translated` 构造改为 `project_id.and_then(...)` 链式调用，None 时跳过 slug 查询，
    译名留空（仍正常下载，仅文件名不翻译）。
- 参考 PCL2 ModModpack.vb InstallPackCurseForge：仅校验 `projectID` 和 `fileID` 存在，
  不强制要求 projectID（部分老整合包仅 fileID）。
- 验证：cargo check 0 errors 0 warnings。需测试缺失 projectID 的 CF 整合包安装。

#### 修复安装失败时下载管理页卡在 0% 不退出的问题
- 根因：后端 `install_modpack` / `install_local_modpack` / `install_merged` /
  `download_version` 命令在 `?` 错误传播时没有重置 `download_state`，`is_active`
  保持 true；前端 `isDownloading()` 轮询返回 true，Downloads.vue 的 watch 无法
  触发 `router.back()`。
- 后端修复（统一错误处理）：
  - [modpack.rs](src-tauri/src/commands/community/install/modpack.rs)：`install_modpack`
    和 `install_local_modpack` 将核心逻辑包在 `async { ... }.await` 中，外层
    `if let Err(e) = result { ds.mark_failed(0); return Err(e); }` 统一处理错误，
    确保任何阶段失败都重置 `is_active=false`。
  - [install/mod.rs](src-tauri/src/commands/version/install/mod.rs)：`install_merged`
    中 `download_version_full` 失败的 `map_err` 中添加 `ds.mark_failed(0)`。
  - [download.rs](src-tauri/src/commands/version/download.rs)：`download_version`
    中 `download_version_full` 失败的 `map_err` 中添加 `ds.mark_failed(0)`。
- 前端修复（用户点击确定后自动退出下载页）：
  - [useDragDrop.ts](src/composables/useDragDrop.ts)：`runModpackInstall` 的两个
    catch 分支改用 `showModal({ type: 'error', ..., onConfirm: () => versionStore.finishDownload() })`，
    用户点击确定后触发 `finishDownload()`，Downloads.vue 的 watch 自动 `router.back()`。
  - [ResourceDetail.vue](src/components/community/ResourceDetail.vue)：`handleInstallModpack`
    的 catch 分支同样改用 `showModal + onConfirm + finishDownload`。
  - [useVersionInstallActions.ts](src/composables/useVersionInstallActions.ts)：
    `onInstallRequest` 和 `handleDownload` 的 catch 分支同样改用
    `showModal + onConfirm + finishDownload`。
  - 设计原则：先显示错误弹窗（模态），用户点击确定后才调用 `finishDownload()`，
    避免 watch 在 nextTick 触发 `router.back()` 导致弹窗一闪而过。
- 验证：cargo check 0 errors 0 warnings，tsc 0 errors。需测试 CurseForge 整合包
  在未配置 API Key 时拖拽安装，验证错误弹窗显示后点击确定能自动退出下载管理页。

#### 统一所有下载场景的失败处理（showModal + 点击确定后退出）
- 背景：之前只修复了整合包安装（`install_modpack` / `install_local_modpack`）、
  版本下载（`download_version` / `install_merged`）的错误处理，但资源下载
  （`download_resource_to_path`）、Mod 更新（`useModUpdate`）、外部下载
  （`useExternalDownload`）仍用 `toastError + finishDownload`，不符合用户要求
  "只要安装涉及到下载管理的地方都应该点击确定弹窗后退出"。
- 前端统一修复（所有 `versionStore.startDownload` 调用点的 catch 分支）：
  - [ResourceDetail.vue](src/components/community/ResourceDetail.vue)：`handleDownload`
    （资源下载到本地）catch 改用 `showModal + onConfirm: finishDownload`。
  - [useModUpdate.ts](src/composables/useModUpdate.ts)：`installSelected`
    （Mod 更新下载）catch 改用 `showModal + onConfirm: finishDownload`，
    移除不再使用的 `toastError` 导入，新增 `showModal` 导入。
  - [useExternalDownload.ts](src/composables/useExternalDownload.ts)：`startDownload`
    （外部 URL 下载）catch 改用 `showModal + onConfirm: finishDownload`，
    新增 `showModal` 导入。
- 轮询冲突修复（[useDownloadPolling.ts](src/composables/useDownloadPolling.ts)）：
  - 问题：轮询检测到 `error_code != 0` 时会立即 `finishDownload + toastError`，
    抢先于调用方 catch 的 `showModal`，导致用户被 `router.back()` 带走，看不到错误弹窗。
  - 修复：`error_code` 路径改为只 `stopPolling`，不 `finishDownload`、不 `toastError`，
    让调用方 catch 统一处理 `showModal + onConfirm: finishDownload`。
    所有调用方（`useDragDrop` / `useVersionInstallActions` / `ResourceDetail` /
    `useModUpdate` / `useExternalDownload`）都有 catch 处理，无遗漏。
- 验证：tsc 0 errors。需测试资源下载、Mod 更新、外部下载失败时弹窗显示后点击确定能退出。

#### 拓展拖拽安装整合包支持：HMCL / MMC / MCBBS 三种新格式
- 背景：用户反馈 `hmcl`、`mmc`、`mcbbs` 这些类型的整合包无法拖拽安装，
  而其他启动器（PCL2）均支持。分析 `code-libs/Plain Craft Launcher 2/Modules/Minecraft/ModModpack.vb`
  发现 PCL2 支持 7 种格式（CurseForge / HMCL / MMC / MCBBS / Modrinth / LauncherPack / Compress），
  而 MoLaunch 此前仅支持 CurseForge 和 Modrinth 两种。
- 数据结构扩展（[src-tauri/src/commands/community/install/types.rs](src-tauri/src/commands/community/install/types.rs)）：
  - `ModpackFormat` 枚举新增 `Hmcl` / `Mmc` / `Mcbbs` 三个变体。
  - `ModpackInfo` 中间结构新增 `archive_base_folder`（关键文件层级前缀）、
    `hmcl_manifest` / `mmc_pack` / `mcbbs_manifest` 三个 Option 字段。
  - 移除原 `overrides_prefix` 单字段，改为由 `build_overrides_prefixes(format, base)`
    动态构造前缀列表（CF/MR 双前缀，HMCL/MMC/MCBBS 单前缀）。
- 新增三个数据结构模块（与现有 `curseforge.rs` / `modrinth.rs` 拆分约定一致）：
  - [hmcl.rs](src-tauri/src/commands/community/install/hmcl.rs)：`HmclManifest { game_version, name }`
  - [mmc.rs](src-tauri/src/commands/community/install/mmc.rs)：`MmcPack { components: Vec<MmcComponent> }`
  - [mcbbs.rs](src-tauri/src/commands/community/install/mcbbs.rs)：`McbbsManifest { addons, name }`
- 格式识别重写（[concurrent.rs](src-tauri/src/commands/community/install/concurrent.rs)）：
  - 新增 `DetectedModpack` 结构体，包含 `format` / `archive_base_folder` /
    `manifest_content` / `index_content` / `hmcl_content` / `mmc_content`。
  - `detect_modpack_format` 改为返回 `DetectedModpack`，按 PCL2 优先级顺序扫描
    关键文件：`mcbbs.packmeta` > `mmc-pack.json` > `modrinth.index.json` >
    `manifest.json`（有 addons → Mcbbs，无 → Curseforge）> `modpack.json`。
  - 两遍扫描：第一遍根目录，第二遍一级子目录（`archive_base_folder` 自动填充
    `"subfolder/"` 前缀，与 PCL2 的 ArchiveBaseFolder 一致）。
  - 新增 `build_overrides_prefixes` 函数：按 format 构造 overrides 前缀列表
    （CF/MR：`overrides/` + `client-overrides/`；HMCL：`minecraft/`；MMC：`.minecraft/`；MCBBS：`overrides/`）。
  - `extract_overrides` 改为接受 `prefixes: &[String]` 参数，按前缀列表匹配并去掉前缀。
- 解析逻辑扩展（[modpack_stages.rs](src-tauri/src/commands/community/install/modpack_stages.rs)）：
  - `parse_modpack_info` 改为接受 `&DetectedModpack` 引用，新增 HMCL/MMC/MCBBS 三个分支：
    - HMCL：从 `modpack.json` 的 `gameVersion` 提取游戏版本；不解析加载器（与 PCL2 一致）。
    - MMC：从 `mmc-pack.json` 的 `components[]` 按 uid 提取
      `net.minecraft`（game）/ `net.minecraftforge`（forge）/
      `net.neoforged`（neoforge）/ `net.fabricmc.fabric-loader`（fabric）；
      跳过 `org.lwjgl.*`（与 PCL2 一致）。
    - MCBBS：从 `mcbbs.packmeta` 或带 `addons` 的 `manifest.json` 的 `addons[]` 按 id 提取
      `game` / `forge` / `neoforge` / `fabric` / `optifine`；遇到 `quilt` 直接报错
      （PCL2 也不支持 Quilt）。
- 安装流程调整（[modpack.rs](src-tauri/src/commands/community/install/modpack.rs)）：
  - `install_modpack` 和 `install_local_modpack` 的 `match info.format` 新增
    `Hmcl | Mmc | Mcbbs` 分支：跳过依赖 mods 下载（这些格式 mods 已打包在 overrides 中），
    直接进入 Stage 3 解压 overrides。
  - `extract_overrides` 调用改为传入 `build_overrides_prefixes(info.format, &info.archive_base_folder)`。
- 前端类型扩展（[src/types/community.ts](src/types/community.ts)）：
  `ModpackFormat` 类型扩展为 `'curseforge' | 'modrinth' | 'hmcl' | 'mmc' | 'mcbbs'`。
- 行为对齐 PCL2：HMCL/MMC/MCBBS 整合包不下载依赖 mods，仅解压 overrides + 安装游戏本体。
- 验证：cargo check 0 errors 0 warnings，tsc 0 errors。需测试三种新格式整合包
  的拖拽安装流程，特别是 overrides 目录前缀正确性（HMCL 的 `minecraft/`、
  MMC 的 `.minecraft/`、MCBBS 的 `overrides/`）。

#### 新增拖拽全局遮蔽层 DragOverlay，提升拖拽体验
- 背景：用户反馈拖拽整合包/Mod 时直接弹出实例名输入框过于生硬，缺乏其他启动器
  （如 PCL2/HMCL）的全屏遮蔽层 + 图标 + 提示文案的视觉反馈。
- 新增组件 [src/components/common/DragOverlay.vue](src/components/common/DragOverlay.vue)：
  - 全屏 `fixed inset-0 z-[10001]` 半透明黑色背景 + backdrop-blur
  - 中央虚线大卡片，根据拖拽类型显示不同图标和主标题：
    整合包（ArchiveBoxIcon，primary 色）/ Mod（CubeIcon，emerald 色）/
    批量 Mod（CubeIcon，emerald 色）/ 不支持的文件（ExclamationCircleIcon，amber 色）
  - 卡片下方显示动态提示文案（如"松开以安装整合包"）与"将文件拖到此处释放"辅助提示
  - 使用 Vue Transition 实现 0.18s opacity + scale 进入/离开动画，视觉柔和
- 改造 [src/composables/useDragDrop.ts](src/composables/useDragDrop.ts)：
  - 新增模块级单例 `dragState`（reactive），暴露 `active` / `hint` / `kind` 三字段
  - 新增 `useDragDropState()` 返回 readonly 状态供 DragOverlay 订阅
  - 新增 `classifyDrag(paths)` 函数，按扩展名预判拖拽类型与提示文案
  - `onDragDropEvent` 事件处理改为 switch 分发：enter 显示遮蔽层，over 保持显示，
    leave/drop 隐藏遮蔽层，drop 后异步分发到对应处理函数
- [src/App.vue](src/App.vue) 顶层渲染 `<DragOverlay />`，随根组件生命周期自动管理。
- 验证：tsc 通过，cargo check 通过。需测试拖拽 enter/leave/drop 各状态下遮蔽层
  显示与隐藏是否平滑，不同文件类型图标和提示是否正确。

#### 修复 CurseForge 整合包 manifest 中 files 项缺失 projectID 导致解析失败
- 背景：用户拖入某些 CurseForge 整合包时报错
  `解析 manifest.json 失败: missing field projectId at line 21 column 5`，
  导致 Stage 1 直接中断。
- 根因：`CfManifestFile.project_id` 原为必填 `i64` 字段，但部分第三方 CF 整合包
  manifest 的 files 项仅含 `fileID`（无 `projectID`），强类型反序列化直接失败。
  PCL2 ModModpack.vb 使用动态 JObject 解析，对缺失字段做跳过处理。
- 修复（[src-tauri/src/commands/community/install/curseforge.rs](src-tauri/src/commands/community/install/curseforge.rs)）：
  - `CfManifestFile.project_id` 改为 `Option<i64>` + `#[serde(default)]`，缺失时为 None。
  - `install_cf_mods` 中 `project_ids` 改用 `filter_map` 过滤 None，缺失项跳过 slug 查询。
  - `file_translated` 构造改为 `project_id.and_then(...)` 链式调用，缺失时译名直接为 None，
    下载仍正常进行（仅文件名不应用 community_filename_format 译名重命名）。
- 行为对齐 PCL2：缺失 projectID 不阻断安装，仅影响译名查询。
- 验证：cargo check 通过。需测试缺失 projectID 的 CF 整合包能否正常解析并安装。

#### 新增拖拽安装整合包与 Mod 功能（参考 PCL2 FormMain.FileDrag 路由分发）
- 背景：MoLaunch 此前仅支持从社区资源页在线下载整合包，无法处理用户从本地
  拖入的 .zip / .mrpack 整合包文件或 .jar / .litemod Mod 文件。参考 PCL2
  的拖拽路由思路，为 MoLaunch 增加 CurseForge / Modrinth 两种格式的本地整合包
  与 Mod 拖拽安装能力。
- 后端（[src-tauri/src/commands/community/install/](src-tauri/src/commands/community/install/)）：
  - 新增 `install_local_modpack` 命令（[modpack.rs](src-tauri/src/commands/community/install/modpack.rs)），
    接收 `InstallLocalModpackRequest { file_path, instance_name }`，跳过 Stage 0
    下载阶段，直接复用 Stage 1-3（解析 manifest → 下载依赖 mods → 解压 overrides）。
    命令已在 [lib.rs](src-tauri/src/lib.rs) 的 invoke_handler 列表注册。
  - 新增 `InstallLocalModpackRequest` 结构体（[types.rs](src-tauri/src/commands/community/install/types.rs)），
    并在 [mod.rs](src-tauri/src/commands/community/install/mod.rs) re-export。
  - 命令内部流程：校验文件存在 → 创建 instance 目录 → 重置 download_state
    （3 个 stages：解析 / 下载 MOD / 复制配置）→ 打开 zip → detect_modpack_format
    → CF 格式校验 API Key → parse_modpack_info → install_cf_mods / install_mr_files
    → extract_overrides → 返回 InstallModpackResult。
  - 清理死文件：删除 hmcl.rs / mmc.rs / mcbbs.rs（这三个文件定义了 HmclManifest /
    MmcPack / McbbsManifest 结构体，但从未被 mod.rs 声明，完全未编译。当前
    detect_modpack_format 只支持 CF / MR 两种格式，未来扩展时再添加）。
- 前端：
  - 新增 composable [src/composables/useDragDrop.ts](src/composables/useDragDrop.ts)，
    使用 Tauri v2 `getCurrentWebview().onDragDropEvent` 监听拖拽事件，按扩展名
    路由：`.zip`/`.mrpack` → 整合包安装，`.jar`/`.litemod`/`.disabled`/`.old`
    → Mod 安装，`.rar` → 提示解压后重试，其他 → 提示无法识别。
  - 整合包流程：弹窗输入实例名（默认取文件名去扩展名）→ `installLocalModpack`
    → `installMerged` 安装游戏本体，进度通过 `download_state` 推送至 DownloadPanel。
  - Mod 流程：弹窗选择目标版本（列出已安装版本）→ `installMod`，支持多文件
    批量安装到同一版本。
  - 在 [src/App.vue](src/App.vue) 根组件 setup 顶层调用 `useDragDrop()` 注册
    全局监听，随根组件生命周期自动卸载。
  - 扩展类型与 API：[src/types/community.ts](src/types/community.ts) 增加
    `InstallLocalModpackRequest`；
    [src/utils/api/community.ts](src/utils/api/community.ts) 新增
    `installLocalModpack` 封装。
- 验证：cargo check 通过（0 warning），tsc 通过。需测试：
  (1) 拖入 CurseForge 整合包能否正确识别并安装（含缺失 projectID 的 manifest）；
  (2) 拖入 Modrinth .mrpack 整合包能否正确安装；
  (3) 拖入 .jar Mod 能否弹窗选版本并复制到 mods 目录；
  (4) 拖入 .rar 能否给出友好提示；
  (5) 多文件批量 Mod 安装的成功/失败统计。

#### 种子地图新增半成品警告提示
- 背景：1.16 版本 WASM 内存越界问题经四轮修复仍未彻底解决，需在 UI 上明确告知
  用户当前为半成品状态，避免误导。
- 变更：[src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue) 顶部
  AlertV2 区域新增一条 error 提示："本项目仍为半成品，不保证完全可用，部分版本
  （如 1.16）可能存在 WASM 内存越界问题，推荐等待后续更新。"
- 原 cubiomes 致谢提示保留不变，两条 AlertV2 用 space-y-2 间距分隔。

#### 修复 1.16 mapOceanMix 动态扩展导致共享 cache 越界（改用独立 malloc）
- 背景：上述 NULL 检查 + getMaxArea 精确估算 + allocCache padding 三轮修复后，
  1.16 拖动地图仍偶发 "memory access out of bounds"，26.2 正常。
- 根因（最终定位）：`mapOceanMix`（layers.c）的 `lw`/`lh` 会根据 warm/frozen
  ocean 位置动态扩展（最多 `w+17`、`h+17`），**无法被 `getMaxArea` 静态估算**：
  1. 函数先扫描 `out[0..w*h)` 找 warm/frozen ocean，动态确定 `lx0/lx1/lz0/lz1`。
  2. 然后用 `land = out + w*h` 作为共享 cache buffer 调用 `l->p->getMap`，
     写入 `land[0..lw*lh)`。当 lw/lh 扩展时，`lw*lh` 可能超过 cache 尾部余量。
  3. 内层 land chain（mapZoom 等）还会在 `land + lw*lh` 后叠加临时 buffer，
     进一步突破 `getMaxArea` 估算的 `siz`。
  4. native 路径 calloc 的对齐/分页余量偶发掩盖此问题，WASM 严格内存检查下
     立即 trap。
- 修复（[src-tauri/cubiomes/layers.c](src-tauri/cubiomes/layers.c) `mapOceanMix`）：
  - 将 `land` 从共享 cache buffer (`out + w*h`) 改为独立 `malloc` 分配。
  - 大小用 `getMinLayerCacheSize(l->p, lw, lh)` 精确计算 land chain 所需 buffer，
    覆盖内层所有临时 buffer 叠加。
  - malloc 失败返回 -1 走错误处理路径，getMap 失败 free(land) 后返回 err，
    函数返回前 free(land) 释放。
  - 添加 `#include "generator.h"` 以使用 `getMinLayerCacheSize` 函数。
- 修复（[src-tauri/cubiomes/generator.c](src-tauri/cubiomes/generator.c) `allocCache`）：
  - WASM padding 从 `len + len/2 + 1024` 减小为 `len + 16`（仅边界对齐余量）。
  - 原因：mapOceanMix 已不共享 cache buffer，`getMaxArea` 的 p2/zoom 估算已修正，
    不再需要大 padding 兜底，避免内存浪费。
  - 更新 `getMaxArea` 注释，说明 mapOceanMix 已改用独立 malloc。
- 验证：cargo check 通过；WASM 已重新编译（cubiomes.wasm 大小 772345 → 772437）。
  需测试 1.16 地图加载与拖动（z=5/6/7/8 各级别）、26.2 结构加载（出生点/要塞）。

#### 修复 1.16 拖动地图偶发 WASM "memory access out of bounds"
- 背景：渲染 1.16 地图本身正常，但拖动加载新 tile 时偶发
  "memory access out of bounds"，specials（出生点/要塞）也偶发失败；
  1.18+ 版本不出现此问题。
- 根因（两层，均与 WASM 内存分配偶发返回 NULL 有关）：
  1. cubiomes 内部 `allocCache` 返回 NULL 未检查：`mapApproxHeight`
     （generator.c）和 `locateBiome`（finders.c）调用 `allocCache` 后
     未检查 NULL，`genBiomes(cache=NULL)` → `genArea` 内 `memset(NULL,...)`
     触发 WASM OOB。1.18+ 走 biome noise 分支（不调 allocCache），不受影响。
  2. cubiomes 内部 `malloc`/`calloc` 返回 NULL 未检查：`mapApproxHeight` 的
     `double *depth = malloc(...)`、`checkForBiomesAtLayer` 的
     `ids = calloc(...)`、`checkForTemps` 的 `area = calloc(...)` 均未检查
     NULL。WASM 内存碎片化或接近 MAXIMUM_MEMORY 上限时偶发返回 NULL，
     后续解引用触发 OOB trap，导致 tile load 和结构校验路径偶发失败。
- 修复（cubiomes submodule 内 14 处 NULL 检查 + emcc 参数）：
  - [src-tauri/cubiomes/generator.c](src-tauri/cubiomes/generator.c)
    `genBiomes` 入口加 `if (!cache) return -1;`
    `genArea` 加 `if (!layer || !out) return -1;` 兜底
    `mapApproxHeight` 的 allocCache 后加 NULL 检查 + free(depth) 防泄漏
    `mapApproxHeight` 的 depth malloc 后加 NULL 检查，返回 -1 走错误码 4
    `mapOceanMixMod` 的 otyp malloc 后加 NULL 检查，返回 -1 走错误处理路径
    `getBiomeAt` 的 allocCache 后加 NULL 检查，返回 none 安全降级
  - [src-tauri/cubiomes/finders.c](src-tauri/cubiomes/finders.c)
    `locateBiome` 的 allocCache 后加 NULL 检查，返回 out + passes=0 安全降级
    `areBiomesViable` 的 allocCache 后加 NULL 检查，跳 L_no（已有 if(ids) free 保护）
    `checkForBiomesAtLayer` 的 ids calloc 后加 NULL 检查，返回 0（不匹配）降级
    `checkForTemps` 的 area calloc 后加 NULL 检查，返回 0（不通过）降级
    `mapEndIslandHeight` 的 ids malloc 后加 NULL 检查，返回 0 降级（修复 -Wreturn-mismatch）
    `floodFillGen` 的 queue malloc 后加 NULL 检查，返回 0 降级
    `getBiomeCenters` 的 ids malloc 后加 NULL 检查，返回 0 降级
    `checkForBiomes` MC_B1_7 分支 allocCache 后加 NULL 检查，返回 0 降级
    `checkForBiomes` 主分支 allocCache 后加 NULL 检查，返回 0 降级
    `checkForBiomes` 主分支 buf malloc 后加 NULL 检查，跳 L_end 用已填充 ids 匹配
    `getBiomeCenters` 1.17- 分支 cache allocCache 后加 NULL 检查，跳 L_end 降级
    `getParaRange` 的 skip malloc 后加 NULL 检查，置 err=-1 跳 L_end 降级
    `getLargestRec` 的 meta calloc 后加 NULL 检查，返回 0 降级
  - [src-tauri/cubiomes/biomenoise.c](src-tauri/cubiomes/biomenoise.c)
    `mapEndBiome` 的 hmap malloc 后加 NULL 检查，返回 -1 走错误处理路径
    `mapEnd` 的 buf malloc 后加 NULL 检查，返回 -1 走错误处理路径
    `mapEndSurfaceHeight` 的 buf malloc 后加 NULL 检查，返回 -1 走错误处理路径
  - [src-tauri/build_script/cubiomes_wasm.rs](src-tauri/build_script/cubiomes_wasm.rs)
    emcc 加 `-s MAXIMUM_MEMORY=512MB`，显式设置内存上限，降低 calloc 失败概率
- 影响：1.16 拖动时偶发 OOB 消除；26.2 结构加载失败消除；malloc/calloc 失败时
  以错误码/空结果降级，JS 端重试机制（generatorWorker.ts 已有 MAX_RETRIES=2）接管，
  不再 trap 中断 Worker。
- 注意：cubiomes submodule 文件已修改，需提交到 fork 仓库 MoTeam-cn/cubiomes。
  需重新编译 WASM（cargo run 时 build.rs 自动调 emcc）。

#### 修复 1.16 layer stack cache 大小估算不足导致完全无法加载贴图
- 背景：上述 NULL 检查修复后，1.16 仍完全无法加载贴图（所有 tile 报
  "memory access out of bounds"），而 26.2 正常。说明根因不仅是偶发 NULL，
  而是 cache 大小本身不足。
- 根因：`getMinCacheSize` → `getMaxArea` 对 layer stack 的临时 buffer 估算不足：
  1. `mapZoom`/`mapZoomFuzzy` 内部 `int *buf = out + pW*pH`，buf 大小 = newW*newH
     = (2*pW)*(2*pH) = 4*pW*pH，但 `getMaxArea` 只累加 `areaX*areaZ ≈ pW*pH`
     （差 4 倍）。
  2. `mapOceanMix`/`mapRiverMix` 多层嵌套调用时，`out + w*h` 作为 land buffer,
     内部 `land + lw*lh` 又作为下一层 buf，临时 buffer 嵌套叠加未被 `siz` 累加
     完全覆盖。
  3. native 环境下 calloc 通常返回对齐/分页的额外余量偶发掩盖此问题，WASM 内存
     严格限制下立即触发 "memory access out of bounds"。
- 修复（[src-tauri/cubiomes/generator.c](src-tauri/cubiomes/generator.c) `allocCache`）：
  - WASM 编译路径（`#ifdef __EMSCRIPTEN__`）额外分配 200% padding + 4KB 固定空间
    （`len * 3 + 1024`），作为 layer stack cache 边界 case 的兜底。
  - native 编译路径不受影响，保持原 `calloc(len, sizeof(int))`。
- 验证：WASM 已重新编译（cubiomes.wasm 大小 772190 → 772211）。需测试 1.16
  地图加载与拖动是否正常，26.2 是否回归。

#### 修复 1.16 拖动地图仍报 OOB（getMaxArea 对 p2 layer 估算少一倍）
- 背景：上述 `allocCache` padding 修复后，1.16 拖动地图仍持续报
  "memory access out of bounds"（z=5/6/8 各种 tile 坐标），26.2 正常。
  说明 `len * 3` padding 仍不足以覆盖 layer stack 的真实临时 buffer 需求。
- 根因（精确分析各 layer 的实际 buffer 占用）：
  1. **p2 layer（mapHills edge=2 / mapRiverMix edge=0 / mapOceanMix edge=17）**：
     parent1 写入 `out[0..area)`，parent2 写入 `out+area` 后的 `buf[0..area)`，
     总占用 `2*area`；但 `getMaxArea` 只累加 `area`，**差 100%**。
     - mapHills: `riv = out + pW*pH`，p2 写入 riv[0..pW*pH]，总 2*(w+2)*(h+2)
     - mapRiverMix: `buf = out + w*h`，p2 写入 buf[0..w*h]，总 2*w*h
     - mapOceanMix: `land = out + w*h`，p1 写入 land[0..lw*lh]（lw/lh 动态扩展
       最多 ±8），总 w*h + (w+16)*(h+16) ≈ 2*w*h
  2. **zoom=2 layer（mapZoom/mapZoomFuzzy edge=3）**：`buf = out + pW*pH` 后接
     4*pW*pH，总占用 5*pW*pH ≈ 5/4*area，原版累加 area，**差 25%**。
  3. **zoom=4 layer（mapVoronoi edge=3）**：`src = out + w*h` 后接 pw*ph，
     总占用 w*h + pw*ph ≈ 17/16*area，原版差 6%。
  4. 一个 layer stack 经过 6+ 层 zoom 和 2-3 层 p2 layer，100% 差距叠加后
     `len * 3` padding 不够。
- 修复（[src-tauri/cubiomes/generator.c](src-tauri/cubiomes/generator.c) `getMaxArea`）：
  - WASM 路径（`#ifdef __EMSCRIPTEN__`）按 layer 类型准确累加：
    - p2 + zoom==1: `area * 2`
    - zoom==2: `area * 5 / 4 + 16`
    - zoom==4: `area * 17 / 16 + 16`
  - native 路径保持原 `area`（calloc 对齐余量掩盖此 bug，不影响）。
  - `allocCache` 的 WASM padding 从 `len * 3 + 1024` 减小为 `len + len/2 + 1024`
    （50% + 4KB），作为 mapOceanMix lw/lh 动态扩展等边界 case 兜底。
- 修复（[src-tauri/build_script/cubiomes_wasm.rs](src-tauri/build_script/cubiomes_wasm.rs)
  `needs_recompile`）：
  - 为每个 .c/.h 源文件单独声明 `cargo:rerun-if-changed=<file>`。
  - 原因：`cargo:rerun-if-changed=cubiomes` 只检查目录本身时间戳（文件增删），
    不检查目录内文件内容修改。导致 generator.c 修改后 build.rs 不重跑，
    WASM 不重新编译，修复不生效。
- 验证：WASM 已重新编译（cubiomes.wasm 大小 772211 → 772345）。需测试 1.16
  地图加载与拖动（z=5/6/8 各级别）是否正常，26.2 是否回归。

#### 模块化 build.rs 拆分 emsdk 相关逻辑
- 背景：build.rs 单文件 334 行，其中 emcc 查找、环境变量配置、WASM 编译命令
  构建等 emsdk 相关逻辑占 312 行，主入口逻辑被淹没，难以维护。
- 改造：将 emsdk 相关代码拆分为 build_script/ 子模块：
  - [src-tauri/build_script/mod.rs](src-tauri/build_script/mod.rs)：模块入口
  - [src-tauri/build_script/emsdk.rs](src-tauri/build_script/emsdk.rs)：emcc
    可执行文件查找（EMSCRIPTEN_ROOT / PATH / 常见 emsdk 路径）与环境变量配置
  - [src-tauri/build_script/cubiomes_wasm.rs](src-tauri/build_script/cubiomes_wasm.rs)：
    WASM 编译入口、源文件清单、增量编译判断
- 结果：[src-tauri/build.rs](src-tauri/build.rs) 精简至 20 行，仅负责调用
  tauri_build::build() 和 compile_cubiomes_wasm()。
- 命名说明：模块目录使用 build_script/ 而非 build/，避免与 build.rs 文件名
  冲突导致 cargo 报 "file for module `build` found at both" 错误。
- 验证：cargo check 通过。

#### 修复 build.rs 构建日志被误报为 warning
- 背景：cargo run 时输出 "warning: Compiling cubiomes to WebAssembly via emcc..."
  和 "warning: cubiomes WASM compiled: ..."，让用户误以为构建有问题。
- 根因：build.rs 用 println!("cargo:warning=...") 输出构建日志，cargo 会把
  所有 cargo:warning 前缀的消息当 warning 标黄显示。
- 修复：[src-tauri/build.rs](src-tauri/build.rs) 改用 eprintln! 直接输出到
  stderr，不经过 cargo 的 warning 系统。日志仍可见但不被标黄。
- 影响：构建日志清晰显示，不再误导读者和 IDE。

#### 修复 1.17 及以下版本地图加载崩溃（SurfaceNoise NULL 解引用）
- 背景：选择 MC 1.16 加载种子时地图一直显示加载中，26.2 正常。
- 根因：cubiomes generator.c 的 mapApproxHeight 对 < MC_1_18 版本走旧 biome
  深度算法分支，内部访问 sn->octdepth 和 sampleSurfaceNoise(sn,...)。但
  cubiomes_wrapper.c 的两个 gen_biomes_with_height 函数对所有版本都传 sn=NULL，
  1.17 及以下主世界/末地会 NULL 解引用导致 WASM 崩溃，Worker 卡死前端一直 loading。
  1.18+ 走 NP_DEPTH 分支不访问 sn，所以正常。
- 修复：[src-tauri/cubiomes/cubiomes_wrapper.c](src-tauri/cubiomes/cubiomes_wrapper.c)
  的 cubiomes_gen_biomes_with_height 和 cubiomes_gen_biomes_with_height_static
  对 mc < MC_1_18 版本调用 initSurfaceNoise(&sn, dim, seed) 初始化后传入，
  1.18+ 保持传 NULL。
- 注意：需重新编译 WASM（cargo run 时 build.rs 自动调 emcc）。
- 影响：1.7~1.17 所有旧版本地图加载恢复正常。

#### 修复 MC 版本枚举映射错误导致旧版本加载失败
- 背景：选择 MC 1.16 时地图一直显示加载中，但 26.2 最新版正常。
- 根因：cubiomes/biomes.h 的 MCVersion 枚举从 MC_1_3_2=0 递增，
  实际值 MC_1_16=14、MC_26_2=28=MC_NEWEST。但 useSeedMap.ts 中
  SEEDMAP_MC_VERSIONS 的 value 比实际值大 6（如 1.16 标 20 实际应为 14，
  26.2 标 34 实际应为 28）。26.2 value=34 超出枚举范围被 cubiomes 容错
  回退到 MC_NEWEST 所以"正常"，1.16 value=20 实际对应 MC_1_21_1，
  1.21.1 需要 1.18+ 的 noise generator 配置，与传入参数不匹配导致 genBiomes 失败。
- 修复：[src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)
  修正所有 SEEDMAP_MC_VERSIONS value 为实际 cubiomes 枚举值（1.7=4 ~ 26.2=28），
  默认 mcVersion 从 34 改为 28，文件头注释补充完整枚举值对照表。
  [src/views/tools/data/LoadSaveModal.vue](src/views/tools/data/LoadSaveModal.vue)
  默认 mcVersion 从 34 改为 28。
- 影响：所有 MC 版本（1.7~26.2）均能正确加载对应地形。

#### 修复超大种子加载失败（WASM strtoll 精度问题）
- 背景：种子 `-4335919219098812575`（绝对值 4.3e18，超过 JS Number 安全范围 2^53）
  无法加载，地图一直显示加载中。`12345` 和 `-12345` 等小数字种子正常。
- 根因：cubiomes_wrapper.c 的 parse_seed 使用 strtoll 解析十进制种子，
  Emscripten WASM 环境下 strtoll 内部可能用 double 累加，对超大 i64 丢精度，
  导致种子值错误、cubiomes genBiomes 无法生成对应地形。
- 修复：[src-tauri/cubiomes/cubiomes_wrapper.c](src-tauri/cubiomes/cubiomes_wrapper.c)
  的 parse_seed 改为手动逐字符解析（uint64_t 累加），不依赖 strtoll，
  确保十进制和十六进制种子的 64 位精度。负数种子通过 `(uint64_t)(-(int64_t)n)`
  转为补码 u64。
- 注意：此修改需要重新编译 WASM。运行 `cargo run` 时 build.rs 会自动调用 emcc
  重新编译 cubiomes.{js,wasm}，前端通过 res:// 协议加载新文件。
- 影响：所有 i64 范围内的十进制/十六进制种子（含负数）均能正确加载。

#### 种子地图从存档加载功能 + 文案修正
- 背景：用户希望直接选择启动器已安装版本和本地存档，自动提取 level.dat 种子加载
  到种子地图，免去手动查找种子。同时修正 AlertV2 文案"不保准"→"不保护"。
- 复用：archiveList（存档列表）、listInstalledVersionsWithType（版本列表）、
  getVersionGameVersion（版本号解析）、fastnbt（NBT 解析，与 nbt.rs 共用）、
  mapMcVersionToCubiomes（新增版本映射函数）、Select/Button/Tooltip 自定义组件、
  DeviceCodeModal 的 Teleport 弹窗模式。
- 变更：
  - [src-tauri/src/commands/tools/archive.rs](src-tauri/src/commands/tools/archive.rs)：
    新增 extract_save_seed 函数，读 level.dat 解析 WorldGenSettings.seed（1.16+）
    或 RandomSeed（1.15 及更早），返回十进制字符串避免 JS 精度丢失。
  - [src-tauri/src/commands/tools/types.rs](src-tauri/src/commands/tools/types.rs)：
    新增 ExtractSaveSeedParams / ExtractSaveSeedResult 类型。
  - [src-tauri/src/commands/tools/mod.rs](src-tauri/src/commands/tools/mod.rs)：
    注册 extract_save_seed action。
  - [src/utils/api/tools.ts](src/utils/api/tools.ts)：新增 extractSaveSeed API。
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    导出 mapMcVersionToCubiomes 函数，将 MC 版本号字符串映射到最近 cubiomes 枚举
    （精确匹配优先，降级取 ≤ 目标的最大版本，无则取最老版本）。
  - [src/views/tools/data/LoadSaveModal.vue](src/views/tools/data/LoadSaveModal.vue)：
    新增从存档加载弹窗组件：选版本→自动拉取 saves→选存档→提取种子→映射版本→emit。
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：控制栏
    "加载"按钮旁新增"从存档"按钮（FolderOpenIcon + Tooltip），打开 LoadSaveModal；
    AlertV2 文案"不保准"→"不保护"。
- 影响：用户可在种子地图页面直接从本地存档加载种子，自动匹配 MC 版本；文案修正
  提升表达准确性。

#### 种子地图鸣谢补充 + 测试警告提示
- 背景：种子地图工具核心依赖 cubiomes 算法库和 OpenLayers 渲染引擎，需在设置-更多-鸣谢
  中体现；同时地图仍在测试阶段，应在界面明确提示用户准确率待验证。
- 复用：项目已有的 about 资源文件机制（include_str! 嵌入 + markdown 表格解析）、
  AlertV2 组件（灰底简洁风格提示框）。
- 变更：
  - [src-tauri/resources/about/acknowledgements.txt](src-tauri/resources/about/acknowledgements.txt)：
    新增 Cubiomes（上游 Cubitect/cubiomes，本项目用 MoTeam-cn/cubiomes 分支，logo 和
    作者头像均为 Cubitect.png）和 OpenLayers（logo 为 openlayers.png）两条鸣谢记录。
  - [src-tauri/resources/about/frontend-deps.txt](src-tauri/resources/about/frontend-deps.txt)：
    新增 OpenLayers ^10.9.0 前端运行时依赖。
  - [src-tauri/resources/about/licenses.txt](src-tauri/resources/about/licenses.txt)：
    新增 OpenLayers（BSD-2-Clause）和 Cubiomes（MIT）许可声明。
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    顶部控制栏下方新增 AlertV2 error 警告（醒目位置），提示地图测试中不保准确率，
    并感谢 cubiomes 算法支持；底部新增 SeedMapIntro 收缩框容器。
  - [src/views/tools/data/SeedMapIntro.vue](src/views/tools/data/SeedMapIntro.vue)：
    新增种子地图实现原理介绍组件（约 300 字），风格与 MoLaunchIntro.vue 一致
    （grid-template-rows 0fr→1fr 平滑过渡），涵盖 OpenLayers 渲染引擎、cubiomes
    WASM 算法、Worker 串行队列、region 遍历与分块查找、性能优化等实现细节。
- 影响：设置-更多-鸣谢页面展示 cubiomes 和 OpenLayers 项目信息与许可；种子地图
  顶部明确标注测试状态管理用户预期，底部提供实现原理展开查看。

#### 种子地图结构刷新实时性 + 群系校验开关 + 图标居中根因修复
- 背景：用户反馈拖动地图到新区域后标记不显示（需反复拖动才出现），希望群系校验
  过滤可由开关控制，且非 webp 图标（heroicons svg）在按钮中靠左不居中。
- 根因：
  1. 标记不实时：`refreshStructures` 并发控制 `if (structRefreshInProgress) return`
     直接丢弃新请求，用户拖到新区域时若上次查找未完成，新区域查找被跳过且不再触发
     （moveend 只在拖动结束时触发一次）。
  2. 图标靠左：Button.vue 的 `.btn-size-mini > svg { margin-right: 4px }` 用于
     图标+文字按钮的间距，但 `:empty` 选择器因 Vue slot 注释节点不匹配，导致图标-only
     按钮的 svg 仍有 margin-right，在 flex 居中时向左偏移 4px。
- 复用：项目自研 Button.vue（useSlots 检测）、Tooltip.vue、ShieldCheckIcon。
- 变更：
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - `refreshStructures` 并发控制改为 pending 机制：查找期间有新请求时标记
      `structPendingRefresh=true`，查找完成后自动补偿触发，避免新区域被遗漏。
    - 新增 `showNonViable` ref（默认 false），控制是否显示未通过群系校验的候选位置。
    - `renderStructures` 根据 `showNonViable` 决定是否过滤 viable=false。
    - 新增 `watch(showNonViable)` 用已缓存数据重新渲染（无需重新查找）。
  - [src/components/common/Button.vue](src/components/common/Button.vue)：
    - 用 `useSlots()` 检测 default slot 是否有文本内容，动态添加 `btn-icon-only` class。
    - CSS 中 `:empty` 选择器（因 Vue slot 注释节点失效）替换为 `btn-icon-only` class，
      图标-only 按钮的 svg 和 spinner 的 margin-right 归零，flex 居中时真正居中。
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    - 筛选栏新增"群系校验"开关按钮（ShieldCheckIcon），点击切换 showNonViable。
    - StructPopup 传入 `:show-viable="showNonViable"`，仅开启时显示校验状态。
  - [src/views/tools/data/StructPopup.vue](src/views/tools/data/StructPopup.vue)：
    - 新增 `showViable` prop，仅 showViable=true 时显示"已通过/未通过群系校验"提示。
- 影响：拖动地图后新区域标记自动刷新（不再需要反复拖动），用户可按需开启群系校验
  开关查看所有候选位置，所有图标-only 按钮中 svg 图标真正居中显示。

#### 种子地图弹窗复制按钮 + 村庄间隔 + Tooltip 统一
- 背景：用户反馈村庄间隔过近不符合实际游戏体验，点击村庄弹窗出现"未通过群系校验"
  黄色字体让人困惑，复制按钮需改为复制坐标 + 复制 TP 命令，部分地图按钮用原生
  `title` 而非自研 Tooltip 组件。
- 根因（村庄间隔近）：cubiomes 按 region 返回候选位置（Village 每 32 chunks=512
  blocks 一个 region 最多一个候选），但实际生成受 biome 限制。代码此前显示了所有
  候选位置（包括 `viable=false` 未通过群系校验的），导致标记过密且弹窗出现黄色
  "未通过群系校验"提示。ravine/fossil 等启发式结构 viable 始终为 true 不受影响。
- 复用：项目自研 Tooltip.vue（替换原生 title）、Button.vue、format.ts 的
  copyToClipboard、useSeedMap.ts 已有的 yCoord ref（前往坐标面板的 Y 输入）。
- 变更：
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    `renderStructures` 默认过滤 `viable=false` 的结构，仅显示通过群系校验的
    真实生成位置，避免候选位置过密。ravine/fossil 等启发式结构不受影响。
  - [src/views/tools/data/StructPopup.vue](src/views/tools/data/StructPopup.vue)：
    - 新增 `yCoord` prop（继承前往坐标面板的 Y 值，默认 64）。
    - 替换单个"复制"按钮为两个：复制坐标（`x z` 不带 y）+ 复制 TP（`/tp x y z`）。
    - 移除"已通过/未通过群系校验"提示行（过滤后均为 viable=true，提示冗余）。
    - min-width 200px→220px 适配两个并排按钮。
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    - 3 处原生 `title` 替换为 Tooltip 组件（复制坐标、大型群系、前往坐标）。
    - 右下角缩放控件（放大/缩小/重置视图）补充 Tooltip 提示。
    - StructPopup 传入 `:y-coord="yCoord"`，让 TP 命令继承面板 Y 值。
- 影响：村庄标记间隔更接近实际游戏，弹窗不再出现"未通过群系校验"黄色提示，
  用户可一键复制坐标或 TP 命令，所有图标按钮统一使用自研 Tooltip。

#### 种子地图村庄不显示根因修复 + 图标居中
- 背景：用户反馈默认只勾选村庄时，地图上逛了一大圈都看不到村庄标记。
  同时反馈图标-only 按钮中图标偏左不居中。
- 根因（村庄不显示）：`generatorWorker.ts` 中 `regionSize` 是 cubiomes 返回的
  chunk 单位（如 Village=32 chunks=512 blocks），但代码直接用 `block/regionSize`
  计算 region 坐标，未 ×16 转换。导致 region 数量膨胀 16²=256 倍，轻易超过
  `REGION_TRAVERSE_LIMIT=5000` 而跳过 Village 查找。
  修复前 zoom 6 下 Village 的 region 数 ≈60×60=3600（接近上限），
  修复后 ≈4×4=16（远低于上限），正常缩放范围内均能查找到村庄。
- 变更：
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    `regionSize` 乘以 16 转为 block 单位（`regionSizeBlocks = regionSize * 16`），
    region 坐标计算用 `block / regionSizeBlocks`。更新 REGION_TRAVERSE_LIMIT 注释。
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    所有图标-only 按钮添加 `!flex !justify-center !items-center`，图标居中显示。
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    `selectedStructureTypes` 默认改回 `['Village']`（避免全部勾选时标记过密）。
- 影响：村庄在 zoom 4~10 范围内正常显示；所有图标-only 按钮内容居中。

#### 种子地图损坏图标文件修复
- 背景：用户反馈遗迹废墟、紫水晶洞、掠夺者前哨站、林地府邸、海底神殿等结构
  图标不显示。检查文件头发现 9 个 .webp 文件实际是 HTML 内容（404/重定向页面
  被错误保存为 .webp），文件头为 `<!DOCTYPE html>` 而非 webp 的 `RIFF`。
- 变更：
  - 从 docs/Map/minecraftsearch.com/images/structures 复制正确的原站图标，
    文件名映射：amethyst_geode→geode、woodland_mansion→mansion、
    ocean_monument→monument、pillager_outpost→outpost、buried_treasure→treasure、
    trail_ruin→trail_ruins、ruined_portal→ruined_portal_n。
  - 删除 bastion.webp 和 fortress.webp（原站无对应图标，保留色块 fallback）。
- 影响：26 个 webp 图标文件全部有效（RIFF 格式），筛选栏和地图上的结构标记
  正确显示 webp 图标。堡垒遗迹和下界要塞使用 Circle 色块 fallback。

#### 种子地图筛选栏 UI 重构 + 要塞/出生点 hover 提示
- 背景：用户反馈筛选栏图标+文字太占空间（多行换行），要塞/出生点红点无提示
  导致不知道是什么。用户要求改为图标-only + tooltip 悬停显示文字，节省空间
  单行排列；要塞和出生点在地图上 hover 时也显示提示。
- 复用：项目自研 Tooltip.vue（Teleport + 自动边界检测 + Select 避让）；
  useSeedMap.ts 已有的几何 hit detection 模式（遍历 source + 像素容差计算）。
- 变更：
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    - 引入 Tooltip 组件，筛选栏所有按钮改为图标-only（`!w-7 !h-7 !p-0`），
      外包 Tooltip 悬停显示结构中文名。
    - 出生点/要塞按钮也改为图标-only + Tooltip。
    - 新增 hoverMarker 悬浮提示（右上角），显示出生点/要塞名称和坐标。
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - `findStructAtPixel` 重构为 `findFeatureAtPixel`，返回 `HitResult`（含
      type/label/x/z），遍历 struct/spawn/stronghold 三个 source。
    - 新增 `hoverMarker` ref（`{ label, x, z } | null`），pointermove 中根据
      hit result 设置 hoverStruct 或 hoverMarker。
    - singleclick 中对 spawn/stronghold 也标记坐标（无 popup 数据）。
- 影响：筛选栏单行排列节省空间；hover 出生点显示"出生点 (x, z)"，
  hover 要塞显示"要塞 (x, z)"，不再需要猜测红点含义。

#### 种子地图结构标记不显示根因修复
- 背景：经多轮修复（图标加载、缩小限制、Worker 健康恢复）后，地图标记仍为红点。
  分析日志发现 `renderStructures` 中 `filtered=0`：`selectedStructureTypes` 默认
  仅选中 `['Village']`，而当前视图返回的结构为 `Mansion`/`Ravine` 等，筛选后
  无匹配项导致不渲染任何结构标记，用户只看到粉色要塞圆点（`getMarkerStyle('#E91E63')`），
  误以为"图标变红点"。
- 复用：`getStructuresForVersion(mcVersion, dimension)` 已有的版本/维度结构过滤逻辑，
  与 `structureListForVersion` computed 和 `watch([mcVersion, dimension, largeBiomes])`
  清理无效选中项的逻辑一致。
- 变更：
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    `selectedStructureTypes` 初始值从 `new Set(['Village'])` 改为全选默认版本/维度
    （MC_26_2/主世界）所有可用结构（排除 stronghold，由独立按钮控制），确保地图
    返回的结构默认全部渲染。清理调试日志（refreshStructures/renderStructures）。
  - [src/utils/seedmap/constants.ts](src/utils/seedmap/constants.ts)：
    移除 `getStructStyle` 的 styleDebugCount 调试日志。
- 影响：默认加载种子后所有结构类型标记均可见（webp 图标），用户可按需在筛选栏
  取消勾选不需要的结构类型。

#### 种子地图缩小黑屏 + 图标红点修复
- 背景：用户反馈缩小到一定程度全屏变黑、结构标记仍为红点而非 webp 图标。
  根因：
  1. 缩小黑屏：MIN_ZOOM=0 允许过度缩小，zoom 0~1 时单 tile 覆盖 8K~16K 方块，
     生成耗时且 viewport 可视 tile 极少（2~4 个），大量空 bitmap 导致观感全黑。
  2. 红点：`import.meta.glob` 在 Vite 5.0.x 不同环境（dev/build/Tauri webview）
     返回类型不一致（`{ default: url }` 或 `url` 字符串），原代码仅处理 `{ default }` 形式，
     部分环境下 `iconUrlMap` 为空导致全部退化为 Circle 色点。
- 变更：
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    MIN_ZOOM 从 0 提升到 2，防止过度缩小。MAX_ZOOM 从 12 回退到 10（与原站对齐），
    RESOLUTIONS 移除末尾两级（0.125, 0.0625），避免极端放大 height buffer 退化黑屏。
  - [src/utils/seedmap/constants.ts](src/utils/seedmap/constants.ts)：
    - `iconUrlMap` 加载逻辑用 `typeof mod === 'string'` 兼容 Vite 两种返回类型，
      `iconUrlMap` 为空时打印 console.warn 便于诊断。
    - OL Icon scale 从 0.4 提升到 0.6（高亮 0.8），显式指定 `anchorXUnits`/`anchorYUnits`
      为 fraction，添加 `crossOrigin: 'anonymous'`。
  - [src/utils/seedmap/terrainShading.ts](src/utils/seedmap/terrainShading.ts)：
    移除已失效的 "zoom 12 时 hw=hh=1" 注释，改为通用防御性边界说明。
- 影响：缩小最小到 zoom 2（64 像素/方块），放大最大到 zoom 10（4 像素/方块），
  均与原站对齐。结构标记在所有环境下正确加载 webp 图标。

#### 计算工具页面交互优化
- 背景：用户反馈坐标计算工具的"交换 A/B"按钮为 icon-only ghost 样式，
  视觉不清晰、难以点击；调色板染料色块使用原生 `<button>` 违反项目
  "必须用自研组件"约束；CalcPage 曾误引入顶部 SubTabBar 菜单栏，
  与"单列堆叠（参考 PCL2）"的 UI 偏好冲突。
- 复用：项目自研 Button.vue（type="outline" + 文字标签）、Input.vue
  （width prop 适配窄输入框）、Tooltip.vue；ColorPicker.vue 中预设色块
  的 `<div role="button" tabindex="0">` 模式（无文字纯色块的标准实现）。
- 变更：
  - [src/views/tools/calc/CalcPage.vue](src/views/tools/calc/CalcPage.vue)：
    移除 SubTabBar 顶部菜单栏，恢复单列垂直堆叠布局（CoordCalculator +
    ColorPalette 两个工具卡片依次排列）。
  - [src/views/tools/calc/CoordCalculator.vue](src/views/tools/calc/CoordCalculator.vue)：
    交换按钮从 `type="ghost"` icon-only 改为 `type="outline"` + 文字标签
    "交换"，外包 Tooltip 提示"交换 A 和 B"，提升可点击性与视觉清晰度。
  - [src/views/tools/calc/ColorPalette.vue](src/views/tools/calc/ColorPalette.vue)：
    染料预设色块从原生 `<button>` 改为 `<div role="button" tabindex="0">`
    + `@keydown.enter`，与 ColorPicker.vue 预设色板实现一致；补充 cursor-pointer
    和键盘可访问性。

#### 种子地图黑屏 + 红点 + 无法继续加载修复（根因修复）
- 背景：之前修复（分块查找 + 边界检查 + Worker 健康恢复）未解决根本问题。
  经深入分析定位到三个独立根因：
  1. 黑屏/无法继续加载：findStructures 在大范围（低 zoom）下遍历数百万 region，
     每个 region 调 _malloc+_free，耗时数分钟，阻塞 Worker 串行队列导致 tile 生成饿死。
  2. 红点：import.meta.glob 配合 `import:'default'` 在 Vite 5.0.x + Tauri 环境下
     可能返回空对象，导致 getStructIconUrl 全量返回空字符串，触发 Circle fallback。
- 复用：generatorWorker.ts 已有的 SLIME_CHUNK_LIMIT 范围限制模式；useSeedMap.ts 的
  structRequestId 结果忽略机制；Vite 的 import.meta.glob 不带 import:'default' 的形式。
- 变更：
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    新增 REGION_TRAVERSE_LIMIT=5000 常量，handleFindStructures 的 region 遍历前
    检查 region 总数，超过上限时跳过该结构类型（类比 SLIME_CHUNK_LIMIT）。
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - 新增 STRUCT_MIN_ZOOM=4 常量和 updateStructLayerVisibility 函数，
      zoom < 4 时隐藏结构图层并跳过 findStructures（避免大范围遍历阻塞 Worker）。
    - moveend 事件中调用 updateStructLayerVisibility，loadSeed 中也调用。
    - refreshStructures 添加 structRefreshInProgress 并发控制：上一次查找未完成时
      跳过新请求，避免多个 findStructures 累积占用所有 Worker。
  - [src/utils/seedmap/constants.ts](src/utils/seedmap/constants.ts)：
    import.meta.glob 移除 `import:'default'`（Vite 5.0.x 已知组合 bug），
    改用 `{ default: url }` 模块对象手动取 .default，构建 iconUrlMap 直接映射，
    getStructIconUrl 改为 O(1) 查表而非每次遍历 Object.entries。
  - [src/utils/seedmap/workerPool.ts](src/utils/seedmap/workerPool.ts)：
    pickWorker 返回类型改为 `WorkerHandle | null`，所有 Worker terminated 时
    返回 null 而非 workers[0]，enqueue 对应 reject 而非向死 Worker postMessage。

#### 种子地图缩放结构丢失修复
- 背景：缩放种子地图时控制台刷屏 "[cubiomes] ravine/mega_ravine/
  underwater_ravine/mega_underwater_ravine/fossil/fossil_diamond 范围过大，跳过"
  警告，ravine/fossil 系列结构整片消失，地图标记退化为红点。
  根因：`callChunkFinder` 单次 WASM 调用范围超过 sizeLimit 时直接跳过，
  未做分块处理；`renderStructures` 中 `feat.setStyle` 静态设置样式绕过了
  layer 动态 style 函数，导致 Icon 图标 URL 不生效。
- 复用：layer 已配置的 `style: (feature) => getStructStyle(stype, highlighted)`
  动态样式函数（含 hover/click 高亮 + Icon 图标逻辑）；`callFinderOnce`
  单次调用辅助函数（已有 buffer 分配 + 结果读取 + 释放流程）。
- 变更：
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    `callChunkFinder` 改为分块查找模式：当 totalX/totalZ 超过 sizeLimit 时，
    将大范围切分为 `sizeLimit × sizeLimit` 子块，逐个调用 `callFinderOnce`，
    合并所有子块结果返回。ravine/nether_fossil/fossil 系列共用此逻辑。
    移除 nether_fossil 范围过大的 `console.warn`（分块查找已确保大范围也能处理）。
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    `renderStructures` 移除 `feat.setStyle(getStructStyle(...))` 调用，
    让 layer 动态 style 函数接管样式（确保 Icon 图标 URL 正确加载 +
    hover/click 高亮生效）。

#### 种子地图边界加载 + 极端缩放黑屏 + Worker 健康恢复
- 背景：加载一定数量 tile 后边界地图无法继续加载（滚动无响应）；
  缩放到极端级别（zoom=12，hsx=hsz=1）页面变黑；Worker 偶发错误后被
  永久标记为 unhealthy，任务调度异常。
- 复用：workerPool.ts 已有的 `pickWorker` 评分机制（pending 越少分越高）；
  useSeedMap.ts 的 `emptyBitmap` 空白 tile 工厂；terrainShading.ts 的
  clamp 索引模式。
- 变更：
  - [src/utils/seedmap/workerPool.ts](src/utils/seedmap/workerPool.ts)：
    - `WorkerHandle` 新增 `terminated` 标志，避免向已 terminate 的 Worker 派发任务。
    - `onMessage` 收到非 error 消息时重置 `healthy=true` + `errorCount=0`，
      让 Worker 能从瞬时错误中恢复（避免一次偶发错误永久 unhealthy）。
    - `pickWorker` 改为双轨选择：优先 healthy && idle Worker，无健康 Worker 时
      fallback 到最空闲的非 terminated Worker（不再盲目选 workers[0]）。
    - `onError` 累计 errorCount > MAX_ERRORS_PER_WORKER 时才 terminate。
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    `loadBiomeTile` 添加防御性边界检查：`constrainOnlyCenter:true` 时 OL 可能
    请求超出 EXTENT 的 tile，直接返回 emptyBitmap 避免无效 Worker 调用。
  - [src/utils/seedmap/terrainShading.ts](src/utils/seedmap/terrainShading.ts)：
    `cx`/`cz` 计算改为 `Math.min(Math.max(bx, 0), Math.max(0, hsx - 2))`，
    确保 `hsx=1` 时 `hsx-2=-1` 不再产生负索引（修复极端缩放黑屏）。

#### NBT 解析器改用 fastnbt（修复 KNOWN_ISSUES P1）
- 背景：`docs/KNOWN_ISSUES.md` P1 记录 NBT 解析为手动实现（约 296 行），
  维护成本高、边界情况（嵌套 TAG_List / 空 compound / 超大数组）易出 bug。
  原因是早期试 `simdnbt` 依赖 nightly（portable_simd）失败后转手动，
  未尝试 stable 兼容的 `fastnbt`。
- 复用：`fastnbt = "2"`（stable 兼容，serde 设计，社区验证）；`flate2` 保留（gzip 解压）。
- 变更：
  - [src-tauri/Cargo.toml](src-tauri/Cargo.toml)：
    新增 `fastnbt = "2"`，更新 simdnbt 注释说明改用 fastnbt。
  - [src-tauri/src/commands/tools/nbt.rs](src-tauri/src/commands/tools/nbt.rs)：
    重写（296 → 166 行）。`parse` 函数改用 `fastnbt::from_bytes` 解析，
    新增 `read_root_name`（fastnbt::Value 不保留根名称，手动从字节流提取），
    新增 `convert_nbt` 递归将 fastnbt::Value 转 NbtNode（保持前端 IPC 协议不变）。
    ByteArray 转 Vec<u8>（0-255，与原手动实现一致）。TAG_End 空根提前返回明确错误。
    移除手动 NbtReader / parse_root / parse_payload / TAG_* 常量。
- 收益：代码量减半，可靠性提升（社区验证），支持嵌套 List / 空 compound / 超大数组。

#### 缺失结构图标补充
- 背景：Mineshaft / Slime_Chunks / Ravine / Fossil / Nether_Fossil 等结构无 webp 图标，
  此前用 OL Circle + STRUCTURE_ICONS.color 几何形状 fallback。
- 变更：
  - [src/assets/structures/](src/assets/structures/)：
    新增 6 个 webp 图标：
    mineshaft / slime_chunks / ravine / fossil / fossil_diamond / nether_fossil。
  - [src/utils/seedmap/constants.ts](src/utils/seedmap/constants.ts)：
    `getStructIconUrl` 新增 `ICON_NAME_ALIASES` 别名映射：
    Mega_Ravine / Underwater_Ravine / Mega_Underwater_Ravine → ravine.webp
    （canyon 系列共用同一图标）。

#### 结构 popup 浮窗 + 坐标格式化 + 悬停群系名
- 背景：原 SeedMap.vue 点击结构仅显示一行 "选中: XXX (x, z) 已通过群系校验"，
  无交互能力（无法复制坐标、无法快速前往、无法关闭）。鼠标悬停时也无群系名提示，
  玩家无法判断当前 tile 的群系。本次将选中详情替换为 OL Overlay popup 浮窗，
  并新增鼠标悬停时异步查询群系名（debounce 300ms）。
- 复用：format.ts 的 formatCoord/copyToClipboard（项目已有惯例 navigator.clipboard.writeText，
  见 ResourceDetailHeader.vue / ColorPalette.vue）；constants.ts 的 getStructIcon/getStructIconUrl；
  toast.ts 的 toastSuccess/toastError；项目自定义 Button.vue（避免原生 button）。
- 变更：
  - [src/utils/seedmap/format.ts](src/utils/seedmap/format.ts)：
    新建。导出 `formatCoord(x, z)`（格式化 "X / Z"）+ `copyToClipboard(text)`（包装 navigator.clipboard），
    从 SeedMap.vue 拆出避免组件超 300 行。
  - [src/utils/seedmap/biomeNames.ts](src/utils/seedmap/biomeNames.ts)：
    新建。`BIOME_NAMES` 记录 60+ 个 biome ID → 中文名映射（参考 cubiomes/biomes.h 枚举），
    覆盖主世界 0~50、1.7 mutate 变种（id+128）、1.16 下界 170~173、1.17 洞穴 174~175、
    1.18 山地 177~182、1.19 183~184、1.20 185、1.21 186、26.2 187。导出 `getBiomeName(id)` 未知返回 '未知群系'。
  - [src-tauri/cubiomes/cubiomes_wrapper.c](src-tauri/cubiomes/cubiomes_wrapper.c)：
    新增 `cubiomes_get_biome_at_point(seed_str, mc, dim, large_biomes, scale, x, y, z)`。
    内部 setupGenerator + applySeed + getBiomeAt，返回 biome ID（< 0 出错）。
    坐标系约定：scale=1 时 x/y/z 为方块坐标，scale>1 时为 scale 坐标系（方块/scale），
    与 `cubiomes_gen_biomes_with_height_static` 对齐。
  - [src-tauri/build.rs](src-tauri/build.rs)：
    EXPORTED_FUNCTIONS 新增 `_cubiomes_get_biome_at_point`。
  - [src/utils/seedmap/types.ts](src/utils/seedmap/types.ts)：
    新增 `BiomeAtPointMsg` / `BiomeAtPointResultMsg` / `BiomeAtPointParams` 类型，
    MainToWorkerMsg 联合加 BiomeAtPointMsg，WorkerToMainMsg 联合加 BiomeAtPointResultMsg。
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    handleMessage switch 加 `case 'biome_at_point'`，新增 `handleBiomeAtPoint(msg)`：
    writeSeedString → `_cubiomes_get_biome_at_point` → post biome_at_point_result。
  - [src/utils/seedmap/workerPool.ts](src/utils/seedmap/workerPool.ts)：
    新增 `getBiomeAtPoint(params)` 方法（enqueue 'biome_at_point'），
    onMessage switch 加 biome_at_point_result 分支，enqueue type 联合加 'biome_at_point'。
  - [src/views/tools/data/StructPopup.vue](src/views/tools/data/StructPopup.vue)：
    新建（76 行）。OL Overlay 内容组件，props=struct，emits=goto/close。
    显示：结构图标 + 名称 + formatCoord 坐标 + 群系校验状态 + 复制/前往/关闭按钮（全部用 Button.vue，
    heroicons 图标，不用 Emoji / 原生 HTML）。
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    引入 format.ts / biomeNames.ts / ol/Overlay。
    新增状态：`popupData` (含 struct + OL 坐标)、`mouseBiomeName`、`popupContainer` ref、`popupOverlay`。
    initMap 创建 Overlay（OL v10 API：autoPan 传 PanIntoViewOptions 对象，含 animation.duration + margin）。
    singleclick 结构时设置 popupData + overlay.setPosition，点击空白时 closePopup。
    pointermove mouseBlock 变化时调 `scheduleBiomeQuery`（debounce 300ms，scale=4，
    x/z=block/4，y=yCoord/4），成功后 mouseBiomeName=getBiomeName(biomeId)。
    新增函数：copyCoord（toast 反馈）、goToStruct（animate zoom 8）、closePopup（清空 + setPosition(undefined)）。
    onMounted 加 `await nextTick()` 确保 popupContainer ref 已挂载。移除 selectedStruct（被 popupData 替代），
    保留 hoverStruct。
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    引入 StructPopup.vue + formatCoord。从 useSeedMap 解构 popupData/mouseBiomeName/popupContainer/copyCoord/goToStruct/closePopup。
    移除底部 "选中详情" div。mapContainer 内加 popupContainer div（始终挂载，OL Overlay 通过 element 引用），
    内部 StructPopup 用 v-if=popupData 控制。mouseBlock 悬浮提示加 mouseBiomeName 显示。
    所有 `({{ x }}, {{ z }})` 改用 formatCoord（hoverStruct / lastClickBlock / mouseBlock）。
    lastClickBlock 加复制按钮（复用 Button.vue type=ghost）。

#### 实施扩展结构方案 A（id 201-223 子集，7 个结构）
- 背景：cubiomes 不原生支持部分扩展结构（id 201-223），业界有通过 fork cubiomes 实现的方案。
  本次调研后采用方案 A：仅实现 cubiomes 原生精确支持（ravine 系列 4 个）+ biome 检查启发式
  （nether_fossil/fossil/fossil_diamond 3 个），共 7 个结构。跳过 dungeon_zombie/skeleton/spider
  （RNG 不可精确预测）、cheese_cave 系列（3D 噪声性能差）、lava_lake/lava_flooded_cave（同前）、
  enchanted_golden_apple（loot 表依赖）、sulfur_spring（需 fork cubiomes）。
- 变更：
  - [src-tauri/cubiomes/cubiomes_wrapper.c](src-tauri/cubiomes/cubiomes_wrapper.c)：
    新增 `cubiomes_find_ravines`（CANYON_CARVER/UNDERWATER_CANYON_CARVER + checkCanyonStart，
    mega 通过 carveCanyon poses.size 阈值区分，阈值 200）、`cubiomes_find_nether_fossils`
    （soul_sand_valley 中心启发式，4 邻居 ≥2 匹配）、`cubiomes_find_fossils`
    （desert/swamp/mangrove_swamp 中心启发式，diamondMode 额外要求深层 deep_dark）三个查找函数。
    所有函数硬上限 64x64 chunks 防内存爆炸，biome ID 取自 cubiomes/biomes.h
    （soul_sand_valley=170, desert=2, swamp=6, mangrove_swamp=184, deep_dark=183）。
  - [src-tauri/build.rs](src-tauri/build.rs)：
    EXPORTED_FUNCTIONS 新增 `_cubiomes_find_ravines` / `_cubiomes_find_nether_fossils` / `_cubiomes_find_fossils` 3 个导出。
  - [src/utils/seedmap/structures.ts](src/utils/seedmap/structures.ts)：
    `StructureQueryMode` 扩展 7 个新类型（'ravine' / 'mega_ravine' / 'underwater_ravine' /
    'mega_underwater_ravine' / 'nether_fossil' / 'fossil' / 'fossil_diamond'）；
    OVERWORLD_STRUCTURES 末尾追加 Ravine(212) / Mega_Ravine(213) / Underwater_Ravine(214) /
    Mega_Underwater_Ravine(215) / Fossil(221) / Fossil_Diamond(222) 6 条；
    NETHER_STRUCTURES 末尾追加 Nether_Fossil(204) 1 条。
  - [src/utils/seedmap/constants.ts](src/utils/seedmap/constants.ts)：
    STRUCTURE_ICONS 追加 7 个图标（颜色取自 prompt-structures.md：Ravine #7A6B5A、
    Mega_Ravine #5A4B3A、Underwater_Ravine #4A7A8B、Mega_Underwater_Ravine #3A6A7B、
    Nether_Fossil #D4C4A8、Fossil #E8D5B0、Fossil_Diamond #5DCED1）。
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    `handleFindStructures` 按 queryMode 分支调用 3 个新 WASM 函数；
    提取 `callChunkFinder` 公共闭包模式（chunk 范围计算 → buffer 分配 → WASM 调用 → 结果读取 → 释放），
    复用于 ravine/nether_fossil/fossil 三类查找；性能保护：mega 模式 chunk 范围上限 32x32
    （carveCanyon 较慢），非 mega 64x64，超出则 console.warn 跳过。

#### 实现 slime_chunks 查找（prompt-structures.md）
- 背景：cubiomes 提供 isSlimeChunk API（finders.h:317 纯算法函数，仅需 seed + chunk 坐标），
  原前端未接入，无法在地图上标记史莱姆区块。
  prompt-structures.md §findSlimeChunks 要求前端遍历可视范围内的 chunks 调 isSlimeChunk。
- 变更：
  - [src-tauri/cubiomes/cubiomes_wrapper.c](src-tauri/cubiomes/cubiomes_wrapper.c)：
    - 新增 `cubiomes_is_slime_chunk(seed_str, chunk_x, chunk_z)` 返回 1/0
    - 内部调 cubiomes isSlimeChunk（无需 Generator，纯算法）
  - [src-tauri/build.rs](src-tauri/build.rs)：
    - EXPORTED_FUNCTIONS 新增 `_cubiomes_is_slime_chunk`
  - [src/utils/seedmap/structures.ts](src/utils/seedmap/structures.ts)：
    - StructureQueryMode 新增 `'slime'` 类型
    - OVERWORLD_STRUCTURES 末尾添加 `Slime_Chunks`（id=-3 文档约定特殊值，queryMode='slime'）
  - [src/utils/seedmap/constants.ts](src/utils/seedmap/constants.ts)：
    - STRUCTURE_ICONS 添加 `Slime_Chunks`（shape='circle', color='#44FF44', label='史莱姆区块'）
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    - handleFindStructures 新增 queryMode='slime' 分支：
      遍历可视范围 chunk（block/16）调 `_cubiomes_is_slime_chunk`，是则添加到 structs
    - 性能优化：可视范围 chunk 数 > 10000 时跳过，避免卡死
    - 坐标用 chunk 中心方块（chunk*16 + 8）
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - structureListForVersion 注释补充说明 slime_chunks 不被过滤（在结构列表中勾选）
    - 现有过滤 `s.queryMode !== 'stronghold'` 已天然保留 slime，无需修改逻辑

#### 实现多要塞遍历（prompt-structures.md）
- 背景：原 specials 流程仅返回首座要塞（cubiomes_first_stronghold），
  无法显示完整要塞分布（cubiomes 默认 MC_1_9+ 共 128 座）。
  prompt-structures.md §findStrongholds 要求基于 nextStronghold 迭代器
  一次性返回可视范围内的多座要塞。
- 变更：
  - [src-tauri/cubiomes/cubiomes_wrapper.c](src-tauri/cubiomes/cubiomes_wrapper.c)：
    - 新增 `cubiomes_find_strongholds(seed_str, mc, max_count, out_buffer, out_len)`
    - 内部 setupGenerator + applySeed（nextStronghold 需 Generator 做群系校验）
    - initFirstStronghold 初始化迭代器，循环 nextStronghold 写入 x/z 到 out_buffer
    - max_count 强制上限 128（cubiomes 默认要塞总数），避免无限循环
  - [src-tauri/build.rs](src-tauri/build.rs)：
    - EXPORTED_FUNCTIONS 新增 `_cubiomes_find_strongholds`
  - [src/utils/seedmap/types.ts](src/utils/seedmap/types.ts)：
    - SpecialsResult / SpecialsResultMsg 的 `firstStronghold` 改为
      `strongholds: {x,z}[]` 数组（兼容性注释标注原字段已废弃）
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    - handleSpecials 改调 `_cubiomes_find_strongholds(max_count=128)`
    - 从 HEAP32 读取要塞坐标数组，返回 strongholds: {x,z}[]
  - [src/utils/seedmap/workerPool.ts](src/utils/seedmap/workerPool.ts)：
    - specials_result 解析改为返回 `{ spawn, strongholds }`
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - refreshSpecials 清空 strongholdSource 后遍历 strongholds 数组添加多个 Feature
    - OL 自动渲染所有 Feature（数量上限 128）

#### 结构系统 queryMode 字段化（prompt-structures.md）
- 背景：prompt-structures.md 要求按 `queryMode: "region"/"stronghold"/"mineshaft"`
  区分结构查找逻辑，原 structures.ts 仅按 cubiomesType 遍历 region，
  未标注 Stronghold（cubiomes getStructurePos 不适用）和 Mineshaft（按 chunk 查找）的语义差异。
- 变更：
  - [src/utils/seedmap/structures.ts](src/utils/seedmap/structures.ts)：
    - 新增 `StructureQueryMode` 类型与 `StructureTypeConfig.queryMode` 字段
    - 补充 Stronghold 到 OVERWORLD_STRUCTURES（id=25, queryMode='stronghold'）
    - 给 Mineshaft 标注 queryMode='mineshaft'（前端语义，cubiomes 内部统一处理）
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    - handleFindStructures 跳过 queryMode='stronghold' 的结构，
      避免与 specials 流程（cubiomes_first_stronghold）重复绘制首座要塞
    - region/mineshaft 沿用 cubiomes_get_structure_pos 遍历 region（cubiomes 内部统一）
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - structureListForVersion 过滤 queryMode='stronghold'，由独立"要塞"按钮控制
    - 版本/维度切换清理逻辑同步过滤 stronghold
- 未实现：
  - 文档中 id 201-223 的扩展结构（dungeon_zombie/skeleton/spider、fossil、cheese_cave_*、
    ravine、underground_lava_lake 等）：cubiomes 库本身不支持，需 fork 扩展
  - slime_chunks：需新增 cubiomes_is_slime_chunk C 端 API
  - 多要塞遍历：需 cubiomes_next_stronghold 迭代器支持

#### 优化 tile 加载策略（边缘空白 + 中间空缺）
- 背景：用户反馈地图顶部边缘加载缓慢、中间出现空缺需缩放才触发重载。
  原因：preload=0 不预加载相邻 zoom 级别；tile 加载失败后返回 emptyBitmap
  被 OL 视为成功，不重试；cacheSize=2048 偏小。
- 变更：
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - TileLayer 新增 `preload: 1`：预加载相邻 zoom 级别，减少拖拽/缩放时的边缘空白
    - cacheSize 从 2048 增大到 4096，避免大范围浏览时 tile 被过早清除
    - DataTile `transition` 从 150 改为 0：禁用淡入动画，tile 生成后立即显示
    - loadBiomeTile 新增重试机制：Worker 失败时重试 2 次（间隔 200ms），
      避免偶发错误（WASM init 未完成/内存增长）导致永久空缺

#### 修复 emcc 编译失败（exit code 9009）+ build.rs 管道卡死
- 背景：build.rs 执行 emcc 时返回 exit code 9009（命令未找到）。
  根本原因：emsdk 的 `.emscripten` 配置用 `os.getenv('EM_CONFIG')` 定位 emsdk 根目录，
  build.rs 只设置了 EMSDK/EMSCRIPTEN_ROOT/EM_CACHE，缺少 EM_CONFIG 和 PATH（node/python/llvm），
  导致 emcc.exe 无法找到依赖。另外 status() 模式下 emcc 的 stderr 输出会撑满 cargo 管道缓冲区，
  导致 build.rs 永久挂起。
- 变更：
  - [src-tauri/build.rs](src-tauri/build.rs)：
    - find_emcc 第3步（common_emsdk_paths）新增 EM_CONFIG 环境变量（指向 .emscripten）
    - 新增 find_subdir_bin 函数：自动扫描 emsdk 的 node/python 版本目录（如 22.16.0_64bit/bin）
    - 构建 PATH：emscripten + upstream/bin + node/bin + python + 原 PATH（跨平台分隔符）
    - emcc 执行改用 output() 代替 status()，捕获 stdout/stderr 避免管道缓冲区满卡死
    - 失败时打印 emcc stderr 前 20 行帮助诊断
  - WASM 已重新编译：resources/wasm/cubiomes.{js,wasm} 包含全部 pointer 模式 API

#### 从 WASM 读取 cubiomes 内置 biome 颜色表（prompt-cubiomes.md）
- 背景：前端硬编码的 BIOME_COLORS 仅覆盖部分 biome ID（约 80 个），
  未覆盖的 biome 会显示为 DEFAULT_COLOR 灰色。cubiomes 内置 initBiomeColors
  覆盖全部 256 个 biome ID，颜色与官方 viewer 一致。
- 变更：
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    - 新增模块级变量 `wasmBiomeColors: Uint8Array | null`
    - handleInit 中调用 `_cubiomes_init_biome_colors()` + `_cubiomes_get_all_biome_colors()`，
      复制 256×3 RGB 到独立 Uint8Array（避免 WASM 内存增长导致 HEAPU8 视图失效）
    - 渲染循环优先用 wasmBiomeColors，fallback 到 BIOME_COLORS（try-catch 保护）
    - WASM 未更新时（emcc 编译失败）自动回退到前端硬编码，不影响功能
- 依赖：需 emcc 重新编译 WASM 才能生效（当前 emcc exit code 9009 待解决）

#### 地形渲染 UI 控件 + applyTerrainShading 调用修复（prompt-frontend.md）
- 背景：terrainShading.ts 已实现 doContour（等高线）和 ymax（最大渲染高度）选项，
  但 generatorWorker.ts 调用时仍用旧签名（传 `false` 布尔），导致类型不匹配且新功能未启用。
  本次修复调用签名，并在 UI 暴露 Y 坐标/等高线/限高三个控件。
- 变更：
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    - 修复 applyTerrainShading 调用：改传 `TerrainShadingOptions` 对象
      `{ scale, pixelsPerCell: TILE_SIZE / sx, doContour: msg.doContour, ymax: msg.ymax }`
    - 从 GenerateTileMsg 读取 doContour/ymax，支持 UI 动态控制
  - [src/utils/seedmap/types.ts](src/utils/seedmap/types.ts)：
    - GenerateTileMsg 和 GenerateTileParams 新增可选 `doContour`、`ymax` 字段
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - 新增状态：`yCoord`（Y 坐标，默认 64）、`doContour`（等高线开关）、`ymaxLimit`（限高，0=不限）
    - generateTile 调用透传 y/doContour/ymax 参数
    - 新增 watch：yCoord/doContour/ymaxLimit 变化时刷新 biomeSource
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    - 坐标面板扩展为两行：第一行 X/Z/前往，第二行 Y/等高线/限高
    - 复用 showCoordPanel 控制显示/隐藏，不新增按钮

#### cubiomes C 库 pointer 模式 API + 地下群系支持（prompt-cubiomes.md）
- 背景：prompt-cubiomes.md 要求给 cubiomes C 库添加 pointer 模式 API（内部 buffer 管理）、
  指定 Y 高度的群系生成（地下群系查看）、biome 颜色表导出等函数。
  原 out_buffer 模式每次调用需 JS 端 _malloc/_free 多个 buffer，代码冗余；
  pointer 模式由 C 端管理内部 buffer，JS 端通过 _get_*_pointer 读取，简化 Worker 代码。
- 变更：
  - [src-tauri/cubiomes/cubiomes_wrapper.c](src-tauri/cubiomes/cubiomes_wrapper.c)：
    - 新增内部静态 buffer（g_biome_data/g_height_data/g_height_dims/g_biome_colors），
      用 realloc 动态扩容，避免固定大小限制
    - 新增 `cubiomes_gen_biomes_static` — 生成 biome 到内部 buffer（pointer 模式）
    - 新增 `cubiomes_gen_biomes_with_height_static` — 生成 biome + height 到内部 buffer，
      接受 y 参数（方块 Y 坐标，用于地下群系）
    - 新增 `cubiomes_gen_biomes_at_y` — 指定 Y 高度生成 biome（地下群系查看）
    - 新增 `cubiomes_gen_biomes_at_y_with_height` — 指定 Y + 高度图
    - 新增 `cubiomes_get_biome_data_pointer` / `_size` — 返回 biome buffer 指针/大小
    - 新增 `cubiomes_get_height_data_pointer` / `_size` — 返回 height buffer 指针/大小
    - 新增 `cubiomes_get_height_grid_dims` — 返回 height 维度 int[2]
    - 新增 `cubiomes_init_biome_colors` — 初始化 cubiomes 内置颜色表（256×3 RGB）
    - 新增 `cubiomes_get_all_biome_colors` — 返回颜色表指针
    - 新增 `cubiomes_get_image_dimensions` — 根据 pixelsPerCell 计算图片尺寸
    - 新增 `cubiomes_free_static_buffers` — 释放内部 buffer（dispose 时调用）
    - Y 坐标转换：scale=1 时 y 直接用方块 Y；scale>1 时转 1:4 坐标（y/4）
    - 新增 `#include "util.h"`（initBiomeColors 声明）和 `#include <math.h>`（sqrt）
  - [src-tauri/build.rs](src-tauri/build.rs)：
    - EXPORTED_FUNCTIONS 新增 13 个 pointer 模式函数导出
  - [src/utils/seedmap/types.ts](src/utils/seedmap/types.ts)：
    - GenerateTileMsg 和 GenerateTileParams 新增可选 `y` 字段（方块 Y 坐标，默认 64）
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    - handleGenerate 改用 pointer 模式 API：
      `_cubiomes_gen_biomes_with_height_static` / `_cubiomes_gen_biomes_static`
      替代原 out_buffer 模式的 `_cubiomes_gen_biomes_with_height` / `_cubiomes_gen_biomes`
    - 移除 biome/height/hw/hh 的 _malloc/_free 逻辑（C 端内部管理）
    - 通过 `_cubiomes_get_biome_data_pointer` / `_cubiomes_get_height_data_pointer`
      / `_cubiomes_get_height_grid_dims` 读取数据
    - 新增 y 参数传递（默认 64=海平面）

#### 种子地图结构图标版本过滤 + 缺失图标 fallback
- 背景：不同 MC 版本可用的结构不同（如 Ancient_City 仅 1.19+，Trial_Chambers 仅 1.21+），
  旧版本前端筛选栏不应显示这些结构的按钮；部分结构（如 Mineshaft）无 webp 图标资源，
  原 getStructStyle 直接用空 src 创建 Icon 导致地图上无可见标记。
- 变更：
  - [src/utils/seedmap/structures.ts](src/utils/seedmap/structures.ts)：
    - StructureTypeConfig 新增 `javaSinceValue` 字段（cubiomes MC 枚举值，与 biomes.h 对齐）
    - 每个结构标注 Java 版引入版本（如 Village=1.0→10, Monument=1.8→11, Ancient_City=1.19→23, Trial_Chambers=1.21→26）
    - 新增 `getStructuresForVersion(mcVersion, dim)`：按 MC 版本 + 维度过滤结构清单，
      供前端筛选栏动态显示；Worker 仍用 `getStructuresByDimension`（全量遍历），
      因 cubiomes_get_structure_pos 对旧版本返回 0（无结构），安全跳过
  - [src/utils/seedmap/constants.ts](src/utils/seedmap/constants.ts)：
    - STRUCTURE_ICONS 新增 `Mineshaft` 定义（square, #6B5B3A, '废弃矿井'）
    - 修复 `Ruined_Portal_N` label 为 '废弃传送门（下界）'，`End_Gateway` label 为 '末地折跃门'，`Treasure` label 为 '埋藏宝藏'
    - `getStructStyle` 增加 Shape fallback：当结构无 webp 图标时，用 OL Circle + STRUCTURE_ICONS.color
      渲染几何形状（highlighted 时半径加大 + 白色描边），确保所有结构都有可见标记
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - `structureListByDimension` 改为 `structureListForVersion`，用 `getStructuresForVersion(mcVersion, dimension)` 动态过滤
    - watch [mcVersion, dimension] 中清理已选但当前版本不可用的结构类型，避免筛选栏残留无效选中
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    - 结构筛选按钮适配新变量名 `structureListForVersion`
    - 无 webp 图标的结构按钮显示彩色小圆点 fallback（`<span>` + backgroundColor），避免 broken img

#### 种子地图 tile 边界根本修复（block→scale 坐标转换）+ 大 scale 跳过 height
- 背景：上一版 tile 边界仍不连续，放大后区块内容与全局不一致。参考
  cubiomes-viewer 源码（docs/Map/cubiomes-viewer/src/world.cpp:436）发现根本原因：
  cubiomes `Range.x/z` 期望 **scale 坐标**（每单位 = scale 个方块），
  但 generatorWorker 直接把方块坐标传给 Range.x/z，导致 cubiomes 按 scale 倍偏移生成，
  tile 内容完全错位、相邻 tile 边界不连续。
- 变更：
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    - **核心修复**：handleGenerate 中将主线程传入的 blockX/blockZ（方块坐标）
      转换为 rangeX/rangeZ = blockX/scale, blockZ/scale 后再传给
      `_cubiomes_gen_biomes_with_height` / `_cubiomes_gen_biomes` 的 Range.x/z
    - 新增 scale 整数与整除校验（blockX/Z 必须是 scale 整数倍，否则抛错），
      确保 tile 边界与 cubiomes scale 网格对齐
    - scale > 16 时改用 `_cubiomes_gen_biomes`（仅 biome，不生成 height），
      避免 scale=64/256 时 height buffer 膨胀到 4MB/64MB；
      远观级别阴影细节本就不可见，跳过 applyTerrainShading
    - 保留 Z 轴翻转（OL top-left origin 下，tile y=0 顶部 = 北方 = max Z，
      cubiomes gz=0 = min Z = 南方，需翻转使 py=0↔gz=sz-1）
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - loadBiomeTile 注释更新：明确 blockX/blockZ 始终是方块坐标，
      block→scale 转换由 worker 负责；EXTENT_HALF 与 blocksPerTile 都是 scale 整数倍，
      保证 startBlockX/Z 可被 scale 整除
    - 文件头 zoom 体系注释更新为 13 级 + cubiomes scale 适配说明

#### 种子地图 UI 重构 + tile 边界修复 + 图标迁移到 assets
- 背景：图标应放在 src/assets 而非 public；地图图标过大；tile 边界不连续；
  默认进入页面应自动加载地图；坐标输入和大型群系应移到地图容器 overlay
- 变更：
  - [src/assets/structures/](src/assets/structures/)：
    22 个 webp 图标从 public/structures/ 迁移至此，与其他 assets 保持一致
  - [src/utils/seedmap/constants.ts](src/utils/seedmap/constants.ts)：
    新增 `getStructIconUrl`（通过 `import.meta.glob` 预加载 assets 图标 URL）；
    图标 scale 从 1.0/1.2 缩小到 0.4/0.5（约 13px，与原圆形标记相近）
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    - EXTENT_HALF 从 29_999_872 改为 29_999_104（16384 的整数倍，确保所有 zoom 级别 tile 对齐）
    - TileGrid origin 从 bottom-left 改为 top-left（标准 OL XYZ 方案），修复 OL 发送负 y 导致 startBlockZ 错误
    - startBlockZ 公式改为 `EXTENT_HALF - (y+1) * blocksPerTile`
    - 默认种子 12345 自动加载（onMounted），seedInput 保持空让用户自行输入
    - loadSeed 接受可选 seedOverride 参数
    - 新增 showCoordPanel 状态控制坐标输入面板展开/收起
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    - 移除"我的坐标"行和"大型群系"checkbox，改为地图容器 overlay 按钮
    - 左下角：大型群系切换按钮（Squares2X2 图标）+ 坐标输入按钮（MapPin 图标）
    - 坐标输入面板展开时显示 X/Z Input + 前往按钮
    - 缩放按钮使用 Plus/Minus/AdjustmentsHorizontal 图标替代文字
    - 筛选按钮使用 type=primary/outline 区分选中状态（替代不存在的 active prop）
    - 图标 img src 改用 getStructIconUrl(s.name)

#### 种子地图结构筛选 + 图标替换
- 背景：默认只显示村庄和出生点，其他结构需要手动开启；结构图标比方块形状更直观
- 变更：
  - [public/structures/](public/structures/)：
    新增 22 个结构 webp 图标（主世界 18 + 下界 3 + 末地 2）
  - [src/utils/seedmap/constants.ts](src/utils/seedmap/constants.ts)：
    `getStructStyle` 从形状（圆/方/三角/菱）改为使用 webp 图标，移除未使用的 `RegularShape`/`blackStroke`
  - [src/utils/seedmap/structures.ts](src/utils/seedmap/structures.ts)：
    新增 `OVERWORLD_STRUCTURES`/`NETHER_STRUCTURES`/`END_STRUCTURES` 数组导出，供筛选 UI 使用
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    出生点和要塞拆分为独立 layer（spawnLayer/strongholdLayer），支持单独显示/隐藏；
    新增 `selectedStructureTypes`（默认仅 Village）、`structureListByDimension`、
    `toggleStructureType`、`isStructureSelected`、`renderStructures`；
    切换结构类型时重新过滤渲染
  - [src/views/tools/data/SeedMap.vue](src/views/tools/data/SeedMap.vue)：
    地图下方新增结构筛选栏（button 形式），包含出生点/要塞开关 + 各维度结构按钮；
    选中状态高亮，未选中图标半透明；删除原有图例区块

#### 种子地图回退到 fork cubiomes + terrainShading 地形阴影渲染
- 背景：放弃其他修改版 cubiomes 方案，改用 fork 仓库
  [MoTeam-cn/cubiomes](https://github.com/MoTeam-cn/cubiomes)（原生支持 MC_26_2），
  并基于项目根目录 `terrainShading.js` 实现地形阴影渲染（hillshade + terrace + contour）
- 变更：
  - [src-tauri/cubiomes/](src-tauri/cubiomes/)：
    替换为 fork 仓库 clone，支持 MC_1_21_5(29) ~ MC_26_2(34) 完整版本枚举
  - [src-tauri/cubiomes/cubiomes_wrapper.c](src-tauri/cubiomes/cubiomes_wrapper.c)：
    新增 `cubiomes_gen_biomes_with_height` 函数，一次返回 biome IDs + 高度数组（调用 `mapApproxHeight`，
    1.18+ 传 NULL 给 SurfaceNoise），用于 terrainShading 渲染
  - [src-tauri/build.rs](src-tauri/build.rs)：
    恢复 `compile_cubiomes_wasm()` 自动编译，`sources` 新增 `terrainnoise.c`/`xradv.c`，
    `EXPORTED_FUNCTIONS` 新增 `_cubiomes_gen_biomes_with_height`
  - [scripts/build-wasm.ps1](scripts/build-wasm.ps1)：
    同步更新源文件清单和导出函数，支持手动构建
  - [src/utils/seedmap/terrainShading.ts](src/utils/seedmap/terrainShading.ts)：
    新增，从 `terrainShading.js` 转换为 TypeScript，实现 hillshade（左上光源 azimuth=315°/altitude=30°）+
    terrace（方块边界台阶线）+ contour（等高线，可选）
  - [src/utils/seedmap/generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)：
    回滚到 `createCubiomesModule` API，调用 `_cubiomes_gen_biomes_with_height` 获取 biome + 高度，
    应用 `applyTerrainShading` 渲染地形阴影，生成 ImageBitmap 返回主线程
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：
    版本映射改用 fork 真实枚举值（26.2=34, 26.1=33, 1.21.9=31, 1.21.6=30, 1.21.5=29），
    默认 mcVersion/currentMc 改为 34（MC_26_2 = MC_NEWEST）

#### 种子地图 generatorWorker 适配修改版 cubiomes API
- 背景：WASM 文件已替换为修改版（`CubiomesModule` 工厂名 + 状态化 API），
  旧版 `createCubiomesModule` + `_cubiomes_*` 封装函数 API 不再可用
- 变更（[generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)）：
  - 工厂函数名：`createCubiomesModule` → `CubiomesModule`
  - 种子参数：字符串指针 `_malloc` → `BigInt` 直接传入 `_apply_seed(BigInt, dimension)`
  - 群系生成：`_cubiomes_gen_biomes` → `_init_generator` + `_apply_seed` + `_generate_biome_image_rgba`（优先）或 `_generate_biome_range`（回退）
  - 结构查找：region 遍历 `_cubiomes_get_structure_pos` → 批量 `_find_structures_in_area` + `_get_structure_results_pointer/count`
  - 出生点：`_cubiomes_estimate_spawn` → `_get_spawn` + `_get_spawn_result_pointer`
  - 新增种子状态缓存（`ensureSeed`）：参数未变时跳过 `_init_generator`/`_apply_seed`，避免重复初始化
  - 移除旧 seed 指针管理（`cachedSeedPtr`/`getSeedPtr`），该 API 直接传 BigInt
  - 要塞查找暂返回 null（该 API 无对应函数）

#### 种子地图 cubiomes WASM 构建方式调整（禁用 build.rs 自动编译，改手动构建）
- 背景：上游 cubiomes（Cubitect/cubiomes master）最高仅 `MC_1_21_WD=28`（1.21.4），
  修改版 cubiomes（内部值 29 = 1.21.5+/26.x）尚未开源。
  待该方案更新后替换源码，届时手动构建一次即可。
- 变更：
  - [build.rs](src-tauri/build.rs)：注释掉 `compile_cubiomes_wasm()` 调用与 `rerun-if-changed`，保留函数定义（加 `#![allow(dead_code)]`）以备将来恢复。现有 `resources/wasm/cubiomes.{js,wasm}`（版本 28）继续使用
  - 新增 [scripts/build-wasm.ps1](scripts/build-wasm.ps1)：手动构建脚本，参数与原 build.rs emcc 调用一致（源文件清单、导出函数、优化选项等）
  - [package.json](package.json)：新增 `build:wasm` 脚本，运行 `powershell -ExecutionPolicy Bypass -File scripts/build-wasm.ps1`
  - 将来替换该 cubiomes 源码后，运行 `npm run build:wasm` 构建一次即可，无需 build.rs 自动编译

#### 种子地图 MC 版本列表扩充（支持 26.2/26.1/1.21.9/1.21.6）
- 问题：版本下拉最大仅 1.21.5，业界同类工具已支持到 26.2
- 根因（分析参考实现的版本映射表）：
  1. 修改版 cubiomes 内部值 29 对应 1.21.5/1.21.6/1.21.9/26.1/26.1.2/26.2（共享 worldgen）
  2. cubiomes 上游 master（Cubitect/cubiomes）最高仅 `MC_1_21_WD=28`（Winter Drop = 1.21.4），尚未支持 29
  3. 参考实现注释明确 "most releases are worldgen-identical to 1.21.5"，28 与 29 对大多数种子结果接近
- 修复（[useSeedMap.ts](src/views/tools/data/useSeedMap.ts)）：
  - 扩展 `SEEDMAP_MC_VERSIONS`，新增 26.2/26.1.2/26.1/1.21.9/1.21.6/1.21.4 标签对齐业界 Latest 列表
  - 1.21.5+ 暂时复用 cubiomes `MC_1_21_WD(28)` 的 worldgen（cubiomes 上游支持 29 后再单独映射）
  - 默认版本仍为 28（对应最新标签 26.2）
  - 注释说明版本对齐策略与业界差异

#### 种子地图 Worker WASM HEAPU8 undefined 根因修复
- 问题：加载地图时报 `Cannot read properties of undefined (reading 'set')`，所有 tile/结构/出生点操作均失败
- 根因（分析 cubiomes.js 工厂函数返回逻辑确认）：
  1. **`instantiateWasm` 回调返回 undefined 导致竞态（核心根因）**：Emscripten 工厂函数结尾为 `wasmExports = await createWasm(); await run(); return Module`，而 `createWasm` 执行 `return Module["instantiateWasm"](imports, receiveInstance)` —— 返回的是我们回调的返回值。之前的 `instantiateWasm` 回调没返回 Promise，`await createWasm()` 立即 resolve，`receiveInstance`（内含 `updateMemoryViews` 赋值 `HEAPU8`）尚未执行，导致 `await factory()` 返回时 `Module.HEAPU8` 为 undefined。后续 WASM 调用内部 `HEAPU8.set` 触发 `Cannot read properties of undefined`
  2. 并发 WASM 内存操作（辅助因素）：`async onmessage` 可能导致多条消息并发执行
- 修复（[generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)）：
  - **`instantiateWasm` 返回 Promise（核心修复）**：在 `cb(r.instance)` 执行后（即 `receiveInstance` → `updateMemoryViews` → `HEAPU8` 赋值完成后）才 `resolve`，确保 `await createWasm()` 真正等待 HEAPU8 就绪
  - 消息串行化：所有消息进入单一 `queue`，`drainQueue()` 一次只处理一条，杜绝并发 WASM 内存操作
  - `new Function` 执行胶水代码 + `ensureHeap()` 安全访问作为加固

#### 种子地图群系生成一致性修复
- 问题：相同版本+种子+坐标，参考实现显示冰河，我们显示草原
- 根因：
  1. `scale` 使用了 cubiomes 不支持的值（如 2, 8, 32），导致群系采样错误
  2. `mcVersion` 映射错误：1.21.5 用了 29，但 cubiomes `MC_1_21_WD=28`
- 修复（[useSeedMap.ts](src/views/tools/data/useSeedMap.ts)）：
  - `scale` 改为从 `SUPPORTED_SCALES = [1, 4, 16, 64, 256]` 中选不超过 bpp 的最大值
  - `mcVersion` 默认值从 29 改为 28（cubiomes MC_1_21_WD=28）
  - `SEEDMAP_MC_VERSIONS` 映射表注释标注 cubiomes 枚举值来源

#### 种子地图 init 协议修复
- 问题：init 消息无 jobId，Worker 内 catch 不会 postError（因 `'jobId' in msg` 对 init 为 false），init 失败被吞掉导致 WorkerPool 永久 hang
- 修复（[types.ts](src/utils/seedmap/types.ts) + [workerPool.ts](src/utils/seedmap/workerPool.ts)）：
  - `InitMsg` 和 `InitCompleteMsg` 加 `jobId` 字段
  - `WorkerPool.init()` 生成 jobId 并用作 pending key，`onMessage` 用 `msg.jobId` 查找

#### 种子地图最大放大级别扩展 + 平滑缩放
- 问题：最大 zoom 不够，无法看清方块细节；zoom 级别离散跳跃，看不到层级过渡
- 修复（[useSeedMap.ts](src/views/tools/data/useSeedMap.ts)）：
  - RESOLUTIONS 扩展 2 级：增加 0.125 和 0.0625（8 像素、16 像素一个方块）
  - MAX_ZOOM 从 10 改为 12
  - `constrainResolution: false` 允许滚轮停在非整数 zoom，tile 用最近级别拉伸（配合 image-rendering: pixelated 保持像素清晰）

#### 种子地图缩放平滑度与默认 zoom 调整
- 问题：滚轮一次缩放跳多级 zoom，导致内容突变（草原变沙漠）；默认 zoom 太低看不到方块细节
- 修复（[useSeedMap.ts](src/views/tools/data/useSeedMap.ts)）：
  - `MouseWheelZoom` 加 `maxDelta: 1, duration: 250`，限制单次滚轮缩放幅度，避免跳多级
  - 默认 zoom 从 4（16 bpp）改为 6（4 bpp），scale=4 采样更精细，接近参考实现的方块边界效果
  - loadSeed/resetView 的默认 zoom 同步改为 6

#### 种子地图滚轮缩放锚点修复
- 问题：滚轮放大后视口中心偏移，鼠标位置的内容在放大后变成旁边的内容
- 修复（[useSeedMap.ts](src/views/tools/data/useSeedMap.ts)）：
  - 显式配置 `MouseWheelZoom({ useAnchor: true })`，确保滚轮缩放围绕鼠标位置
  - 用 `defaultInteractions({ mouseWheelZoom: false }).extend([new MouseWheelZoom(...)])` 保留其他默认交互（DragPan/DragZoom/KeyboardPan 等）
- 注意：不同 zoom 级别 scale 不同，cubiomes 采样网格不重合是固有行为（参考实现也有此现象），无法完全消除

#### 种子地图 tile 边界对齐修复（cubiomes Range 坐标系）
- 问题：相邻 tile 内容不连续（海洋旁突然变沙漠），边界对不齐
- 根因（深入分析 cubiomes 源码 biomenoise.c:1577-1580）：
  - cubiomes `genBiomeNoise3D` 中 `xi = (r.x+i)*scale + mid`，`zj = (r.z+j)*scale + mid`
  - **`r.x`/`r.z` 是相对于 scale 的网格坐标，不是方块坐标**（scale>1 时）
  - 之前直接传方块坐标 `blockX` 给 `r.x`，导致采样位置 = `blockX * scale + mid`，放大了 scale 倍
  - scale=1 时 `r.x` 恰好是方块坐标，所以低 zoom 没问题；scale>1 时采样位置完全错位
- 修复（[generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)）：
  - 传给 cubiomes 前 `rangeX = floor(blockX / scale)`，`rangeZ = floor(blockZ / scale)`
  - 统一处理 scale=1 和 scale>1 两种情况
- 修复（[useSeedMap.ts](src/views/tools/data/useSeedMap.ts)）：
  - `EXTENT_HALF` 从 30_000_000 改为 29_999_872（256 × 117187），确保是所有 scale（最大 256）的整数倍
  - 这样 tile 边界 `startBlockX = -EXTENT_HALF + x * blocksPerTile` 也是 scale 整数倍
  - cubiomes 采样网格与 tile 边界完美对齐

#### 种子地图 tile 边界对齐修复（Z 轴方向）
- 问题：相邻 tile 内容不连续（海洋旁突然变沙漠），边界对不齐
- 根因：MC Z 轴向南递增，OL Y 轴向上递增，直接映射导致方向冲突；之前尝试 Y 翻转但 cubiomes 采样方向不匹配
- 修复（[useSeedMap.ts](src/views/tools/data/useSeedMap.ts)）：
  - origin 改为左下角 `[-EXTENT_HALF, -EXTENT_HALF]`（MC 西南角）
  - `startBlockZ = -EXTENT_HALF + y * blocksPerTile`（不翻转，直接映射）
  - blocksPerTile 是 scale 整数倍，相邻 tile 采样网格自动对齐
  - 副作用：南北方向颠倒（屏幕上方=南方），后续可通过 View rotation 修复

#### 种子地图 zoom/tile 体系对齐业界方案
- 问题：地图只加载约 4 个区块、放大后内容与缩略图不一致
- 根因分析（分析业界同类实现）：
  1. TILE_SIZE=256（参考实现 64）→ 单 tile 覆盖方块多 16 倍，可视区 tile 数极少
  2. EXTENT ±50000（参考实现 ±3e7）→ 范围太小，超出变空白
  3. zoom 体系 -6~7（参考实现 0~10）→ RESOLUTIONS 数组错位
  4. scale 二值选择 `bpp>=4?4:1`（参考实现连续 `res<=1?1:round(res)`）→ 放大时采样不足
  5. Y 轴未翻转（参考实现 `blockZ = origin[1] - tileZ × blocksPerTile`）→ tile 位置错位
  6. mcVersion 映射错误（1.21 用 21，参考实现 1.21.5→29, 1.21.1→26）
  7. origin 在左下角（参考实现左上角 `[-3e7, 3e7]`）
- 修复（[useSeedMap.ts](src/views/tools/data/useSeedMap.ts)）：
  - TILE_SIZE=64, EXTENT_HALF=30_000_000, RESOLUTIONS=[256,128,64,32,16,8,4,2,1,0.5,0.25]
  - MIN_ZOOM=0, MAX_ZOOM=10, 默认 zoom 4（16 bpp）
  - scale 连续选择 `bpp<=1?1:round(bpp)`
  - Y 翻转：`startBlockZ = EXTENT_HALF - y * blocksPerTile`
  - origin 改为左上角 `[-EXTENT_HALF, EXTENT_HALF]`
  - SEEDMAP_MC_VERSIONS 对齐参考实现 JAVA_MC_VERSION_MAP（1.21.5→29, 1.21.4→28, 1.21.1→26, 1.20→25 等）
  - TileLayer 加 cacheSize: 2048
  - loadSeed/resetView 默认 zoom 改 4，goToUserCoord 用 zoom 8
- 修复（[generatorWorker.ts](src/utils/seedmap/generatorWorker.ts)）：
  - TILE_SIZE=64（与前端一致，避免 tile 尺寸不匹配）

#### 修复 "Cannot read properties of undefined (reading 'set')" — WASM HEAPU8 未导出
- 根因：Emscripten 新版默认不把 HEAP 视图（HEAPU8/HEAPU32/HEAP32）暴露到 `Module` 对象，`cubiomes.js` 内 `HEAPU8` 是闭包局部变量；Worker 内 `Module.HEAPU8.set(bytes, ptr)` 因 `Module.HEAPU8` undefined 抛错
- 修复：[build.rs](src-tauri/build.rs) 的 `EXPORTED_RUNTIME_METHODS` 从 `ccall,cwrap` 改为 `ccall,cwrap,HEAPU8,HEAPU32,HEAP32`，强制 Emscripten 把 HEAP 视图挂到 `Module`
- 验证：重新编译后 `cubiomes.js` 中确认存在 `Module.HEAPU8` 赋值
- [generatorWorker.ts](src/utils/seedmap/generatorWorker.ts) `handleInit` 加 `Module.HEAPU8` 二次校验，未就绪则抛明确错误（避免后续 `set` 调用产生不可读的 undefined 报错）

#### 修复 res:// 协议 404 + Canvas2D willReadFrequently 警告
- res:// 404 根因：[res_scheme.rs](src-tauri/src/res_scheme.rs) 的 `parse_res_path` 重复拼接 `web-common/` 前缀，返回 `web-common/web-common/wasm/cubiomes.js`，而 `embedded_bytes` 注册的 key 是 `wasm/cubiomes.js`（不带前缀），导致 `read_resource_bytes` 找不到资源
- 修复：重写 `parse_res_path`，从 `/web-common/` 起始位置直接截取到 query string 前，去掉前导 `/`，不再重复拼接 `RES_ROOT`
- 验证：`cargo test res_scheme` 7 个测试全部通过
- Canvas2D willReadFrequently 根因：OL `Select` 交互的 `forEachFeatureAtPixel` 内部调 `getImageData` 做 hit detection，每帧 benchmark 触发浏览器性能警告
- 修复：[useSeedMap.ts](src/views/tools/data/useSeedMap.ts) 移除 `Select` 交互，改用 `map.on('pointermove'/'singleclick')` + 自实现 `findStructAtPixel`（遍历 `structSource.forEachFeature` 计算 Point 几何距离，完全绕过 OL hit canvas 路径）
  - HIT_TOLERANCE_PX=6（与 OL Select 默认 hitTolerance 一致）
  - pointermove 加 50ms 节流避免高频遍历
  - hover/click 高亮通过 `hoverFeat`/`clickFeat` 变量驱动 structLayer 的 style 回调

#### 修复 Worker "Cannot use import statement outside a module" 报错
- 问题：Vite 默认把 `new Worker(new URL('./generatorWorker.ts', import.meta.url))` 打包成 classic worker，但 `generatorWorker.ts` 用了 ES module 的 `import` 语法，classic worker 不支持
- 修复：
  - [workerPool.ts](src/utils/seedmap/workerPool.ts) Worker 构造加 `{ type: 'module' }`，让 Vite 生成 ESM worker bundle
  - [generatorWorker.ts](src/utils/seedmap/generatorWorker.ts) `importScripts`（classic worker 专属 API）改为 `fetch` + `new Function` 执行 Emscripten 胶水代码，兼容 module worker（参考业界做法）

#### Toast 非 success 方法同步打印控制台日志
- 动机：方便追踪 error/warning/info 类 Toast 的触发来源与上下文
- [toast.ts](src/utils/toast.ts) `toastError` → `console.error`、`toastWarning` → `console.warn`、`toastInfo` → `console.info`，`toastSuccess` 保持静默

#### 种子地图架构迁移：后端 FFI → 前端 WASM WorkerPool
- 动机：原方案在后端 Rust 通过 FFI 调 cubiomes C 库生成群系图，每张 tile 都要走 IPC 传递 RGBA 数据，开销大且无法多线程并行；与业界纯前端 WASM + Worker 架构差距明显
- 新方案：build.rs 调 emcc 自动将 cubiomes_wrapper.c 编译为 WASM，前端通过 Web Worker 池并行调用，零 IPC 开销
- 新增 `res://` 自定义 URI scheme（`src-tauri/src/res_scheme.rs`）：
  - 协议格式 `res://web-common/{type}/{filename}`（Windows: `https://res.localhost/`，macOS/Linux: `res://localhost/`）
  - 路径白名单校验防遍历，附 CORS + `application/wasm` MIME（支持 `compileStreaming`）
- 新增 WASM 加载工具 `src/utils/wasm-loader.ts`：`resUrl()` / `loadWasmModule()` / `loadWasmBytes()` / `prefetchWasmUrl()`，平台探测改用 `navigator.userAgent`（不依赖 `@tauri-apps/plugin-os`，项目未引入）
- 新增前端 WorkerPool 模块 `src/utils/seedmap/`：
  - `types.ts`：主线程↔Worker 消息协议（init/prepare_seed/generate/find_structures/specials/obsolete/dispose）
  - `structures.ts`：cubiomes StructureType 枚举与维度映射（主世界 17 类/下界 3 类/末地 2 类）
  - `generatorWorker.ts`：Worker 实现，`importScripts` 加载 Emscripten 胶水，`createImageBitmap` + transferable 零拷贝回传 tile
  - `workerPool.ts`：调度层，Worker 数量 `clamp(4, floor(0.75 * hardwareConcurrency), 16)`，低配降级到 2；优先派发 idle Worker，错误超 5 次 terminate
- 重写 `src/views/tools/data/useSeedMap.ts`：移除 `seedmapBiomes/Structures/Specials` IPC 调用与 `acquireIpcSlot` 并发锁，改用 `pool.generateTile/findStructures/getSpecials`
- 删除 `src/utils/api/tools.ts` 中 seedmap* API 函数与类型（约 90 行）
- 架构对齐 docs/Map/map.md 分析文档（WorkerPool 消息协议、tile 生成流程、坐标系、版本映射表）

#### 新增 NavSidebar 公共导航侧边栏组件（三页面复用 + tab 路由同步）
- 动机：Settings/VersionSettings/Tools 三个页面左侧分类侧边栏代码高度重复（aside 类名、菜单项类名、选中态、图标尺寸几乎一致），且刷新页面后选中项丢失
- 新建 `src/components/common/NavSidebar.vue`（67 行）：
  - props: `modelValue`（v-model）、`categories`（id/label/icon/desc）
  - 切换菜单时 `router.replace({ query: { ...route.query, tab: id } })` 同步到 URL
  - 页面加载时从 `route.query.tab` 恢复选中项（刷新页面保留打开的分类）
  - 保留其他 query 参数（如 VersionSettings 的 `id`），不冲突
- 改造三个页面（删除手写 aside，替换为 `<NavSidebar v-model="activeCategory" :categories="categories" />`）：
  - `src/views/Settings.vue`：删除 L83-104 手写 aside
  - `src/views/VersionSettings.vue`：删除 L116-133 手写 aside（与已有 `route.query.id` 同步共存）
  - `src/views/Tools.vue`：删除 L128-145 手写 aside，`switchCategory` 改为 `watch(activeCategory)` 触发 TOC 刷新（含从 URL 恢复时）

#### 修复种子地图区块无法拼接 + 块状放大丑陋（对齐业界渲染方案）
- 问题：各 tile 边缘无法拼接，放大后是块状色块而非精细群系图
- 根因 1：旧实现用 `cellPx` 把 1 个群系格画成 N×N 像素方块，tile 边界对不齐
- 根因 2：旧 zoom 体系只有 6 级且 res 不连续，与参考实现 14 级（地图 zoom 8~21）不符
- 修复方案（对齐 docs/Map/map.md 分析）：
  - zoom 体系改为 14 级：OL zoom -6~7 对应地图 zoom 8~21，bpp = 2^(3-OL_zoom)
  - 移除 cellPx 概念：每像素 1 个群系值，sx = TILE_SIZE × bpp / scale
  - scale 选择：bpp≥4 用 scale=4（粗采样快），bpp<4 用 scale=1（精细）
  - 像素绘制：sx×sz 群系下采样到 TILE_SIZE×TILE_SIZE 像素，1 像素 1 群系值
  - CSS 加 `image-rendering: pixelated` 让 OL canvas 用 nearest neighbor 放大，群系边界清晰
- 文件变更：`src/views/tools/data/useSeedMap.ts`（zoom 配置 + loadBiomeTile 重写）、`SeedMap.vue`（pixelated CSS）

#### 修复 OL "Rendering array data is not yet supported" 报错
- 问题：点击加载种子后抛 `Uncaught Error: Rendering array data is not yet supported`，地图不渲染
- 根因：OL 10 的 `DataTile` loader 返回 `Uint8ClampedArray` 时走 array 渲染路径，但 CanvasTileLayerRenderer 不支持 array 数据渲染
- 修复：loader 改为返回 `ImageBitmap`（OL `ImageLike` 类型），通过 `document.createElement('canvas')` 绘制 cell 块后 `createImageBitmap(canvas)` 转换
- 附带优化：直接用 Canvas `fillRect` 绘制 cell 块，比先构造 RGBA array 再转 ImageBitmap 更快（省一次像素遍历）

#### willReadFrequently 警告说明
- 现象：控制台输出 `Canvas2D: Multiple readback operations using getImageData are faster with the willReadFrequently attribute set to true`
- 原因：OL `Select` 交互的 `forEachFeatureAtPixel` 内部调 `getImageData` 做命中检测，浏览器提示性能优化
- 处理：属 OL 默认行为，不影响功能；保留 hover 高亮能力，不强制禁用 hitDetection

#### 修复种子地图进入即卡死（OL extent 过大 + IPC 并发失控）
- 问题：进入种子地图工具页直接卡死，控制台大量 Tauri callback 警告
- 根因 1：投影 extent 设为 ±500 万方块，OL TileGrid 在此范围下计算 tile 范围矩阵爆炸
- 修复 1：缩小 extent 到 ±50000 方块（10 万×10 万），足够覆盖种子地图查看范围
- 根因 2：currentSeed 为空时 biomeLayer 仍可见，OL 疯狂请求 tile 走 IPC 导致队列堆积
- 修复 2：biomeLayer 初始 `visible: false`，loadSeed 时才 `setVisible(true)`
- 根因 3：OL 同时发起多个 tile 请求，每个走 Tauri invoke，IPC 队列堆积卡死前端
- 修复 3：新增 IPC 并发锁（`acquireIpcSlot`/`releaseIpcSlot`），同时最多 3 个 cubiomes 调用，超出排队等待
- 修复 4：View 加 `constrainResolution: true` + `resolutions` 配置，限制 zoom 为整数避免插值导致 tile level 不匹配
- 修复 5：moveend 事件防抖 300ms，避免连续拖拽触发多次 refreshStructures
- 关于 Tauri callback 警告：属 dev 环境热重载正常现象（前端重载时后端有未完成 IPC），重启 dev server 即可消除

#### 种子地图改用 OpenLayers 渲染引擎（替换手写 Canvas）
- 动机：业界同类工具普遍使用 OpenLayers 作为地图渲染引擎，自带 tile 缓存、拖拽缩放、图层管理；手写 Canvas 是重复造轮子
- 新增依赖：`ol@10.9.0`（OpenLayers），`src/main.ts` 引入 `ol/ol.css`
- 架构变更：
  - 群系图层：`DataTile` source + 自定义 `loader`，调后端 cubiomes 获取群系 ID 转 RGBA；OL 自动按 `(z,x,y)` 缓存 tile，已加载区块不再重新请求
  - 结构/特殊点图层：`VectorLayer` + `Feature` + `RegularShape`/`Circle` 样式，按形状区分类型
  - 交互：OL 内置拖拽/滚轮缩放/惯性；`Select` 交互处理 hover/click
  - 投影：自定义 `mc` 投影，1 单位 = 1 方块，extent ±500 万
  - zoom level → (resolution, cellPx, scale) 映射：6 级缩放，res 从 8（最远）到 0.125（最近）
- 删除的手写逻辑（约 300 行）：
  - `onWindowMouseMove`/`onWindowMouseUp` 拖拽监听
  - `draw()`/`drawNow()`/`getViewRect()` Canvas 渲染
  - `isCacheValid()`/`biomeCache` 手动缓存管理
  - `scheduleLoad()` 防抖加载
  - `onCanvasMouseMove()` 手动 hover 命中检测
  - `ResizeObserver` 响应式 Canvas 尺寸
- 文件变更：
  - `src/utils/seedmap/constants.ts`：移除 canvas draw 函数，新增 OL Style 工厂（`getStructStyle`/`getMarkerStyle`/`getClickMarkerStyle`）
  - `src/views/tools/data/useSeedMap.ts`：完全重写为 OL 版（DataTile + VectorLayer + Select 交互）
  - `src/views/tools/data/SeedMap.vue`：canvas 替换为 `<div ref="mapContainer">`，144 行
  - `src/main.ts`：新增 `import 'ol/ol.css'`

#### 种子地图交互重构（修复拖动 + 图标标注 + 坐标输入 + 拆分组件）
- 问题 1：无法拖动地图——`onMouseMove`（拖动逻辑）未绑定到 canvas，只有 `onCanvasMouseMove`（hover 逻辑）绑了
- 修复 1（拖动）：改用 `window` 级 `mousemove`/`mouseup` 监听（`onCanvasMouseDown` 时添加，`onWindowMouseUp` 时移除），确保拖出 canvas 仍可移动/释放
- 问题 2：结构标注看不懂——之前用纯色圆点，无形状区分
- 修复 2（图标标注）：每种结构用不同形状（方形=建筑、三角=神殿/前哨、圆=海洋/水、菱=宝藏/传送门）+ 颜色 + 黑色描边，hover 显示中文名+坐标，点击选中显示详情
- 新增 1（点击坐标）：点击空白处标记该点坐标（黄色圆圈），控制栏显示"点击坐标：(X, Z)"
- 新增 2（用户坐标输入）：控制栏第二行加"我的坐标 X/Z"输入框 + "前往"按钮，输入后视图中心跳转到该坐标并重新加载
- 修复 3（加载按钮右侧）：控制栏用 `ml-auto` 将加载按钮推到右侧
- 修复 4（最小缩放）：`MIN_CELL_PX=2`，防止 cellPx=1 时请求量过大导致加载失败
- 拆分（300 行约束）：从 SeedMap.vue 拆出 3 个文件
  - `src/utils/seedmap/constants.ts`（153 行）：群系调色板 + 结构图标定义 + draw 函数
  - `src/views/tools/data/useSeedMap.ts`（448 行）：组合式逻辑（状态+交互+加载+渲染）
  - `src/views/tools/data/SeedMap.vue`（146 行）：纯模板 + composable 调用

#### 种子地图性能优化（响应式 Canvas + 预加载 + 防抖异步加载）
- 问题 1：Canvas 固定 800×600 像素，超出外层 `max-w-3xl`（约 768px）容器边框
- 问题 2：每次拖动 `onMouseUp` 或滚轮 `onWheel` 立即触发 IPC 加载，缓存只覆盖当前可视区域，拖动一点点就缓存失效显示"加载中"，交互卡顿
- 修复 1（响应式 Canvas）：`containerRef` + `ResizeObserver` 动态计算 Canvas 宽度，高度按 5:3 比例（限制 320~640px），Canvas 用 `class="block w-full"` 适配父容器
- 修复 2（预加载）：群系请求区域 = 可视区域 × 2（四周各预加载 50%），拖动时只要在预加载范围内就直接从缓存偏移绘制
- 修复 3（防抖异步加载）：拖动 `onMouseMove` 只 `draw()` 不加载；`onMouseUp` 后防抖 300ms 调 `scheduleLoad()`，仅当 `isCacheValid()` 返回 false 时才重新请求群系
- 修复 4（视觉无中断）：无数据区域显示深灰背景（`#1a1a1a`）而非"加载中"文字；加载遮罩仅在首次加载（`biomeCache === null`）时显示，后续静默加载不遮挡视图
- 新增 `isCacheValid()` 检查当前可视区域是否完全在缓存范围内（留 10% 余量避免边缘抖动）

#### 种子地图独立为单独分组
- 原问题：种子地图位于"游戏资源"分组下，与截图管理、资源包转换并列，但种子地图是高频实用工具，埋在分组内不便快速访问
- 修复：从 `GameResourcePage.vue` 移除 SeedMap，新建 `src/views/tools/seedmap/SeedMapPage.vue` 单独承载
- `Tools.vue` categories 数组新增"种子地图"分类（10 个分类），图标用 `MapIcon`，置于分类列表末尾便于快速定位
- "游戏资源"分组现剩 2 个工具（截图管理、资源包转换）

#### 修复种子地图 allocCache 失败（wrapper.c Range 字段顺序错误）
- 根因：`cubiomes_wrapper.c` 中 `Range r = { scale, x, y, z, sx, sy, sz }` 的字段顺序错误，cubiomes `Range` 结构体（见 biomenoise.h）实际字段顺序为 `{ scale, x, z, sx, sz, y, sy }`，导致 y 被赋给 z、z 被赋给 sx，sx 变成负数，`getMinCacheSize` 中 `(size_t)sx * sz * sy` 因负数转无符号溢出为巨大值，`calloc` 失败返回 NULL
- 修复：改为 `Range r = { scale, x, z, sx, sz, y, sy }` 严格按 cubiomes Range 字段顺序初始化
- 教训：FFI wrapper 中初始化 C 结构体时必须核对头文件字段顺序，C99 复合字面量按声明顺序赋值

#### 工具页重新分组（每分组 ≤ 3 个工具，重要工具单独一栏）
- 原问题："数据工具"分组下堆积 8 个工具（Java管理/崩溃分析/版本JSON/NBT查看/截图管理/数据导出/资源包转换/种子地图），远超合理数量
- 拆分方案：将原"数据工具"分组的 8 个工具按功能域重新归类
  - **Java 管理**（新分组，单独一栏）：Java 管理（启动游戏核心依赖，作为重要工具独立）
  - **诊断工具**（新分组）：崩溃分析、版本 JSON 编辑、NBT 查看
  - **游戏资源**（新分组）：截图管理、资源包转换、种子地图
  - **数据导出**迁入"便捷工具"：与清理垃圾、内存优化同属实用工具
- 新建 `src/views/tools/java/JavaPage.vue`、`src/views/tools/diagnostic/DiagnosticPage.vue`、`src/views/tools/game-resource/GameResourcePage.vue` 三个编排层组件
- 删除 `src/views/tools/data/DataPage.vue`（工具已全部分散到新分组）
- 修改 `Tools.vue` categories 数组：9 个分类（外部下载/便捷工具/存档管理/Mod工具/网络工具/计算工具/Java管理/诊断工具/游戏资源），每个 ≤ 3 个工具
- 修改 `QuickTools.vue`：新增 DataExporter 组件
- 新增图标：CommandLineIcon（Java管理）、BugAntIcon（诊断工具）、SwatchIcon（游戏资源）

#### 种子地图重构为 C wrapper 封装层（彻底修复 allocCache 失败 + STATUS_ACCESS_VIOLATION）
- 根因：原方案在 Rust 端声明 cubiomes `Generator` 联合体（LayerStack + BiomeNoise + BetaBiomeNoise + NetherNoise + EndNoise）的 `_opaque: [u8; N]` 缓冲，无论 N 取 4KB / 64KB 都不可靠——Rust 端无法精确模拟 C 联合体的对齐/大小/字段布局，`setupGenerator` 仍可能写越界
- 最终修复：新增 `src-tauri/cubiomes/cubiomes_wrapper.c` C 封装层，由 C 编译器在栈上分配 `Generator g;` 真实结构体，C 端完成 `setupGenerator` / `applySeed` / `allocCache` / `genBiomes` / `free` 全生命周期管理，Rust 端只通过 FFI 传参数和接收结果
- wrapper 暴露 6 个函数：`cubiomes_gen_biomes` / `cubiomes_get_structure_pos` / `cubiomes_is_viable` / `cubiomes_get_region_size` / `cubiomes_estimate_spawn` / `cubiomes_first_stronghold`
- `build.rs` 的 sources 数组添加 `"cubiomes/cubiomes_wrapper.c"`（之前遗漏导致 wrapper 符号找不到，链接的还是旧二进制）
- 重写 `seedmap.rs`：删除 Generator/LayerStack/Range/Pos 等 Rust FFI 结构体声明，改为只声明 wrapper 函数签名；MC 版本常量严格按 cubiomes/biomes.h 枚举顺序计数（含 MC_1_16_1/MC_1_19_2/MC_1_21_1/MC_1_21_3/MC_1_21_WD 等中间版本）
- 教训：Rust FFI 调 C 库时，若 C 结构体含联合体/复杂布局，应优先写 C wrapper 而非在 Rust 端手动声明结构体

#### 工具页版本选择 Select 在全局隔离 All(4) 时默认选中第一个版本
- 问题：用户开启"隔离所有版本"后，截图/资源包/存档管理工具的版本 Select 仍默认选"全局（不隔离）"，导致扫描走 `<game_dir>/screenshots/` 等全局目录而非版本隔离目录
- 根因：前端 Select 默认值 `''` 表示全局，即使后端 `resolve_shots_dir` / `resolve_packs_dir` / `resolve_saves_dir` 已支持 `version_id` 参数，前端不传就走了全局路径
- 修复：`ScreenshotManager.vue` / `ResourcePackConverter.vue` / `ArchiveManager.vue` 三组件 `onMounted` 中读取全局 `isolationMode`，当为 `4`（All）且已安装版本列表非空时，默认选中第一个版本，让用户直接看到版本隔离目录
- 中间隔离模式（1/2/3）仍保留"全局"默认，因为这些模式下部分版本仍共享全局目录

#### 修正 cubiomes 版本枚举值（allocCache 失败根因）
- 根因：cubiomes/biomes.h 的 `enum MCVersion` 含中间版本（MC_1_16_1、MC_1_19_2、MC_1_21_1、MC_1_21_3、MC_1_21_WD），导致 1.16–1.21 的常量值全部偏移
- 修正：MC_1_16 19→20，MC_1_17 20→21，MC_1_18 21→22，MC_1_19 23→24，MC_1_20 24→25，MC_1_21 27→28
- 加详细注释列出完整枚举顺序，防止后续再次猜错
- allocCache 调用前加调试日志输出 mc/dim/scale/range 实际值

#### 存档管理按版本隔离配置扫描
- 原行为：直接扫 `<game_dir>/saves/`，无视版本隔离配置
- 修复：`archive::list` / `archive::backup` / `archive::restore` 三函数接收 `version_id` 参数，复用 `resolve_isolation_mode` + `get_effective_game_dir` 解析路径
- types.rs 新增 `ArchiveListParams`，`ArchiveBackupParams` / `ArchiveRestoreParams` 加 `version_id` 字段
- 前端 `ArchiveManager.vue` 标题栏加版本选择 Select（全局 + 已安装版本列表），backup/restore 调用时传入当前选中版本
- `api/tools.ts` 的 `archiveList` / `archiveBackup` / `archiveRestore` 三函数增加可选 `versionId` 参数

#### 新增种子地图工具（基于 cubiomes C 库）
- 工具位置：数据工具页 → 种子地图
- 输入种子（支持十进制 / 0x 十六进制 / 文本自动 hash）+ 选择 MC 版本（1.7–1.21）+ 维度（主世界 / 下界 / 末地）+ 大型生物群系 flag
- 三组 IPC 命令（统一走 `tools_manager` 入口）：
  - `seedmap_biomes`：查询指定区域群系（返回 `biomes[z][x]` 二维数组）
  - `seedmap_structures`：查询区域内的结构标记（村庄/神殿/要塞/海底神殿/远古城市/试炼密室等 20+ 种，带 `viable` 群系校验标志）
  - `seedmap_specials`：查询出生点 + 首座要塞近似位置
- 引入 `cc` build-dependency 编译 cubiomes C 源码（MIT，https://github.com/Cubitect/cubiomes）到 Rust 静态库，FFI 调用 `setupGenerator` / `applySeed` / `genBiomes` / `getStructurePos` / `isViableStructurePos` / `initFirstStronghold` / `estimateSpawn` 等
- 前端 `SeedMap.vue`：800×600 Canvas 渲染群系图（预置调色板覆盖 100+ 群系 ID），鼠标拖拽平移 + 滚轮缩放（以鼠标位置为中心）+ 结构标记点击看详情 + 出生点/要塞十字标记 + 悬浮坐标显示
- 复用约束：Tauri 官方 plugin 优先用、`spawn_blocking` 跑 CPU 密集任务、Button/Select/Input 用项目自定义组件、原生 checkbox 加 `accent-primary-500` 与项目其他工具一致

#### 修复种子地图 STATUS_ACCESS_VIOLATION 崩溃
- 根因：cubiomes `Generator` 联合体（LayerStack + BiomeNoise + BetaBiomeNoise + NetherNoise + EndNoise）实际尺寸约 8–16KB，原 FFI 声明用 `[u8; 4096]` 缓冲导致 `setupGenerator` 写越界
- 修复：扩大 `_opaque` 缓冲到 `[u8; 65536]`（64KB，留足余量），加详细注释说明尺寸依据

#### 截图管理 / 资源包转换按版本隔离配置扫描
- 原行为：直接扫 `<game_dir>/screenshots/` 和 `<game_dir>/resourcepacks/`，无视项目默认 `isolation_mode=4`（隔离所有版本）配置，导致扫到 `target/debug/.minecraft/...` 而非真实版本目录
- 修复：复用 `crate::commands::version::list::resolve_isolation_mode` + `crate::minecraft::isolation::get_effective_game_dir`，按版本隔离配置解析有效游戏目录
- 后端类型新增 `ScreenshotListParams` / `ScreenshotDeleteParams.version_id` / `ResourcePackListParams`（`version_id: Option<String>`）
- `screenshot::list` / `screenshot::delete` / `resourcepack::list` 三个函数接收 params，路径校验基于解析后的实际目录
- 前端 `ScreenshotManager.vue` 和 `ResourcePackConverter.vue` 标题栏增加版本选择 Select（选项：全局 + 已安装版本列表），默认全局，切换时自动重新加载
- `api/tools.ts` 的 `screenshotList` / `screenshotDelete` / `resourcepackList` 三函数增加可选 `versionId` 参数

#### 移除后端冗余的文件选择对话框命令
- 删除 `src-tauri/src/commands/system/game_dir.rs` 中的 `select_folder` / `select_file` / `save_file` 三个 Tauri 命令及 `FileFilter` 结构体（这些命令本质只是把 `tauri-plugin-dialog` 又包了一层 Rust 边界，与前端 `@tauri-apps/plugin-dialog` 功能完全重叠）
- 删除 `src-tauri/src/lib.rs` 中三个命令的 invoke_handler 注册
- 删除 `src/utils/api/system.ts` 中的 `selectFolder` / `selectFile` / `saveFile` 三个前端封装函数
- 保留 `write_text_file` 命令（非对话框，是写文本辅助命令，会自动创建父目录）
- 统一使用 `src/utils/fileDialog.ts`（基于 `@tauri-apps/plugin-dialog`）的 `pickFile` / `pickDirectory` / `pickSavePath` 三个函数
- 迁移 9 个文件 14 处调用点：CrashDialog.vue / FolderSidebar.vue / SettingsPlugins.vue / JavaCustomMode.vue / JavaPathSelector.vue / CustomLayoutSection.vue / ResourceDetail.vue / useExternalDownload.ts / useModList.ts / useVersionOverviewActions.ts / useSkinOperations.ts
- CrashDialog.vue 的写文件方式从 `invoke('plugin:fs|write_text_file', ...)` 改为统一调用 `writeTextFile`（自动创建父目录 + 后端日志）

#### 工具页文件选择器与 TOC 导航优化
- 新增 `src/utils/fileDialog.ts`：封装 Tauri dialog 插件的 `pickFile` / `pickDirectory` / `pickSavePath` 三个函数，统一文件/文件夹选择对话框调用
- 为 ArchiveManager（备份输出路径 + 恢复 zip 路径）、NbtViewer（NBT 文件路径）、DataExporter（导出输出路径）的输入框右侧添加文件选择器图标按钮（Input 的 append 插槽 + FolderOpenIcon），用户无需手动输入路径
- 新增 `src/components/common/ToolToc.vue`：工具页右侧 TOC 导航条组件，自动扫描页面内 `[data-toc-card]` 元素生成图标方格条
  - 每个方格显示工具标题前 2 字，hover 时用 Tooltip 组件显示完整标题
  - 点击方格滚动跳转到对应工具卡片（scrollIntoView smooth）
  - 滚动时自动高亮当前可见项（scroll 事件 + getBoundingClientRect 计算）
  - 工具数 < 3 时自动隐藏
- 修改 `Tools.vue` 布局：右侧内容区改为双栏（内容 + w-14 TOC 侧栏），滚动容器加 `.tools-scroll-container` class，分类切换时递增 `tocRefreshKey` 触发 TOC 重新扫描
- 为所有 Page 组件（DataPage / CalcPage / ModToolsPage / NetworkPage / ArchivePage）的工具卡片包裹 div 加 `data-toc-card` + `data-toc-title` + `id` 锚点属性
- 新增依赖：`@tauri-apps/plugin-dialog@^2`（前端 JS 包，后端 Rust 插件已就绪）

#### 存档管理工具（存档管理分类）
- 后端：`archive.rs` 的 `list` 扫描 `{game_dir}/saves/` 子文件夹（递归计算大小、检测 level.dat）；`backup` 将存档打包为 zip（可选排除 playerdata/ 目录用于导出分享包）；`restore` 从 zip 解压到 saves/ 目录（目标已存在则失败，路径安全校验）
- 前端：列出存档（名称、大小、修改时间、有效标志），点击备份弹出对话框填写输出路径 + 排除玩家数据选项，底部恢复区填写 zip 路径 + 存档名称，备份/恢复走 showConfirm 回调式
- 后端类型：`ArchiveListResult` / `ArchiveItem` / `ArchiveBackupParams` / `ArchiveBackupResult` / `ArchiveRestoreParams` / `ArchiveRestoreResult`
- 组件：`src/views/tools/archive/ArchiveManager.vue`，后端：`src-tauri/src/commands/tools/archive.rs`

#### 网络延迟测试工具（网络工具分类）
- 后端：`network.rs` 的 `latency_test` 用 reqwest 并发测试多个 URL 的 HTTP 延迟（10 秒超时），返回每个 URL 的延迟、状态码、错误信息
- 前端：textarea 输入 URL 列表（每行一个），提供官方源/BMCLAPI/MCBBS 预设按钮一键填充，测试结果按延迟颜色分级（绿/黄/橙/红）
- 后端类型：`NetworkLatencyTestParams` / `NetworkLatencyResult` / `LatencyItem`
- 组件：`src/views/tools/network/NetworkLatencyTester.vue`，后端：`src-tauri/src/commands/tools/network.rs`

#### 服务器状态检测工具（网络工具分类）
- 后端：`network.rs` 的 `server_ping` 纯 Rust 手写 MC SLP 协议（1.7+ 版本），TCP 连接 → Handshake（VarInt 协议版本 -1 + 地址 + 端口 + next state=1）→ Status Request → 读取 JSON 响应 → Ping/Pong 计算延迟
- MOTD 提取支持字符串和对象两种形式（含 extra 数组递归拼接），去除 §格式化代码；支持 Favicon base64 返回
- VarInt 编码修复：对负数（如协议版本 -1）转 u32 位移避免无限循环
- 前端：输入 host + port（默认 25565）→ 展示 Favicon + MOTD + 在线人数/版本/延迟三栏信息卡，延迟按颜色分级
- 后端类型：`ServerPingParams` / `ServerPingResult`
- 组件：`src/views/tools/network/ServerPinger.vue`

#### NBT 数据查看工具（数据工具分类）
- 后端：`nbt.rs` 手动实现 NBT 解析器（simcdnbt 需要 nightly Rust 不可用），用 flate2 gzip 解压 + 大端二进制解析，覆盖全部 13 种标签类型（TAG_End ~ TAG_Long_Array），含负长度校验和越界保护
- 前端：输入 NBT 文件路径 → 递归树形展示（NbtTreeNode.vue 递归组件），每个节点显示类型徽章（按类型着色）+ 名称 + 值，支持展开/折叠，默认展开根节点
- 后端类型：`NbtParseParams` / `NbtParseResult` / `NbtNode`
- 组件：`src/views/tools/data/NbtViewer.vue` + `src/views/tools/data/NbtTreeNode.vue`（递归树组件），后端：`src-tauri/src/commands/tools/nbt.rs`
- 依赖变更：移除 `simdnbt`（nightly-only），新增 `flate2 = "1"`（gzip 解压）

#### 崩溃日志分析工具（数据工具分类）
- 后端：`crash_analyzer.rs` 的 `analyze` 对 log_text 做大小写不敏感的子串/正则匹配，识别 6 类崩溃模式（java_version / missing_mod / memory / driver / mod_conflict / other）
- 每条分析结果含分类、严重级别（error/warning/info）、标题、匹配行片段、中文修复建议
- 前端：粘贴崩溃日志文本 → 调 `crashAnalyze` → 展示识别出的崩溃原因条目，6 类分类标签 + 3 级严重级别样式
- 后端类型：`CrashAnalyzeParams` / `CrashAnalyzeResult` / `CrashAnalysisItem`
- 组件：`src/views/tools/data/CrashAnalyzer.vue`，后端：`src-tauri/src/commands/tools/crash_analyzer.rs`

#### 截图批量管理工具（数据工具分类）
- 后端：`screenshot.rs` 的 `list` 枚举 `{game_dir}/screenshots/` 下所有文件按修改时间降序排序；`delete` 批量删除截图，删除前校验每个 path 规范化后以 screenshots 目录为前缀（路径安全）
- 前端：列举截图文件，支持多选与批量删除，删除走 `showConfirm` 回调式（项目规范），全选/反选、已选数量与总大小统计、空状态提示
- 后端类型：`ScreenshotListResult` / `ScreenshotItem` / `ScreenshotDeleteParams` / `ScreenshotFailedItem` / `ScreenshotDeleteResult`
- 组件：`src/views/tools/data/ScreenshotManager.vue`，后端：`src-tauri/src/commands/tools/screenshot.rs`

#### 资源包转换工具（数据工具分类）
- 后端：`resourcepack.rs` 的 `list` 枚举 `{game_dir}/resourcepacks/` 顶层条目（.zip 文件 → format=zip；目录 → format=folder）；`convert` 支持 folder → zip / zip → folder 双向转换，用 zip crate，路径校验
- 前端：列举资源包，支持在 zip ↔ folder 两种格式间转换，转换走 `showConfirm` 回调式，目标已存在时后端返回失败提示
- 后端类型：`ResourcePackListResult` / `ResourcePackItem` / `ResourcePackConvertParams` / `ResourcePackConvertResult`
- 组件：`src/views/tools/data/ResourcePackConverter.vue`，后端：`src-tauri/src/commands/tools/resourcepack.rs`

#### 版本 JSON 编辑工具（数据工具分类）
- 后端：`version_json.rs` 的 `read` 读取 `{game_dir}/versions/{version_id}/{version_id}.json`；`save` 先用 `serde_json::from_str::<serde_json::Value>` 校验合法性，校验通过后写入文件，version_id 校验不含 `..` / 路径分隔符
- 前端：选择已安装版本 → 读取 JSON → 编辑 → 保存，保存走 `showConfirm` 回调式二次确认，dirty 状态追踪、文件路径提示、未保存警告
- 后端类型：`VersionJsonReadParams` / `VersionJsonReadResult` / `VersionJsonSaveParams` / `VersionJsonSaveResult`
- 组件：`src/views/tools/data/VersionJsonEditor.vue`，后端：`src-tauri/src/commands/tools/version_json.rs`

#### Java 版本管理工具（数据工具分类）
- 复用 `stores/java.ts` + `utils/api/java.ts`，列出系统检测到的 Java 运行时
- 单击列表项设为默认 Java，支持「自动选择」模式（由后端启动流水线按版本需求匹配）
- 列表项展示完整路径（Tooltip 组件）、版本号、Java 大版本徽章
- 重新检测按钮触发 `store.refreshJava()`，空状态提示引导用户检测
- 组件：`src/views/tools/data/JavaManager.vue`

#### Mod 依赖检测工具（Mod 工具分类）
- 后端：`mod_tools.rs` 的 `mod_dependency_check` 扫描 `versions/{id}/mods/` 下所有 .jar，读取每个 mod 的 dependencies（fabric.mod.json 的 depends / mods.toml 的 [[dependencies]]），与已安装 mod_id 集合比对，排除 minecraft/java/fabricloader/fabric-api/quilt_loader/quilted_fabric_api/forge/neoforge 内置依赖
- 前端：选版本 → 调 `modDependencyCheck` → 按 required_by 分组展示缺失依赖列表，空状态提示「依赖均已满足」
- 为 `ModMetadata` 新增 `dependencies: Vec<String>` 字段，`MetaBuilder` 新增 `add_dependencies()` 累积合并去重
- 后端类型：`ModDependencyCheckParams` / `ModDependencyResult` / `MissingDep` / `ConflictDep`
- 组件：`src/views/tools/mod-tools/ModDependencyChecker.vue`，后端：`src-tauri/src/commands/tools/mod_tools.rs`

#### Mod 文件去重工具（Mod 工具分类）
- 后端：`mod_tools.rs` 的 `mod_dedup_scan` 按 slug 分组，找出有多个版本的 mod，slug 为空的 mod 不参与去重，版本号空时回退到文件名
- 前端：选版本 → 调 `modDedupScan` → 展示重复 mod 列表（mod_id + 多版本条目，含版本号徽章、文件名 Tooltip、文件大小）
- 后端类型：`ModDedupScanParams` / `ModDedupResult` / `DuplicateMod` / `DuplicateVersion`
- 组件：`src/views/tools/mod-tools/ModDedupScanner.vue`

#### 启动器数据导出工具（数据工具分类）
- 后端：`data_export.rs` 的 `export_launcher_data` 将 config / versions / accounts 打包为 zip
- 账号脱敏：微软账号仅保留 username/uuid（不含 expires_at/is_expired），离线账号不含 skin 字段，current_user 仅含 name/uuid/login_type
- 前端：勾选导出项 → 填写输出路径（默认预填下载目录下 molaunch-export.zip）→ 调 `exportLauncherData` → 展示导出结果（路径/大小/包含项）
- 后端类型：`ExportLauncherDataParams` / `ExportResult`
- 组件：`src/views/tools/data/DataExporter.vue`，后端：`src-tauri/src/commands/tools/data_export.rs`

#### 坐标距离计算工具（计算工具分类）
- 输入两组 XYZ 坐标，实时计算欧氏距离、曼哈顿距离、切比雪夫距离
- 地狱门坐标换算：主世界↔下界 1:8 比例双向换算
- 交换 A/B 坐标按钮
- 组件：`src/views/tools/calc/CoordCalculator.vue`

#### 游戏内调色板工具（计算工具分类）
- RGB 滑块 + HEX 输入框双向同步，实时预览颜色
- RGB / HEX / HSL 三种格式互转
- 16 种 Minecraft 染料色预设（点击切换）
- 16 种 Minecraft 格式化代码（§0~§f，点击复制到剪贴板）
- 组件：`src/views/tools/calc/ColorPalette.vue`

#### 工具页侧边栏新增 5 个分类骨架
- 在 `Tools.vue` 左侧侧边栏新增 5 个一级分类：存档管理 / Mod 工具 / 网络工具 / 计算工具 / 数据工具
- 为每个分类新建页面组件（编排层）：`tools/archive/ArchivePage.vue`、`tools/mod-tools/ModToolsPage.vue`、`tools/network/NetworkPage.vue`、`tools/calc/CalcPage.vue`、`tools/data/DataPage.vue`
- 为 14 个工具创建占位组件（显示"即将实现"），后续批次逐步替换为完整实现
- 从 `QuickTools.vue` 移除 `upcomingTools` 数组与"更多工具"section（已迁移到各分类页面）
- 默认选中分类改为「便捷工具」（原为「外部下载」）
- 新增实现计划文档 `docs/QUICK_TOOLS_IMPL_PLAN.md`

#### 清理游戏垃圾 UI 重构
- **文件树分组展示**：扫描结果按"全局 / 各版本"分组，每组可折叠（grid-rows 0fr↔1fr 动画）、可全选，display_name 带 " - {version}" 后缀的归入对应版本组，无后缀的归入"全局"组
- **高度限制 + 滚动**：中部扫描结果区限制 `max-h-[400px]` + `overflow-y-auto`，顶部标题栏与底部操作栏固定，避免列表过长一直下拉
- **Tooltip 组件替代原生 title**：所有 `:title="item.path"` 改用 `<Tooltip>` 组件（项目自定义组件约定）
- **showConfirm 回调误用修复**：`executeCleanup` 中 `await showConfirm(...)` 当 Promise 用导致永远 return，已改为回调式用法
- **拆分 CleanupGroupList.vue 子组件**：CleanupTool.vue 从 415 行降至 255 行，分组计算与渲染逻辑内聚到子组件（194 行），主组件只负责状态管理与编排
- **路径溢出修复**：Tooltip 组件默认 `display: inline-flex` 宽度收缩到内容，导致内部 `truncate` 失效、长路径撑破父容器与右侧"X 个文件"重叠。改用 `block` prop 让 trigger 撑满父容器宽度，truncate 正确生效
- **默认折叠状态**：扫描完成后所有分组默认折叠，用户先看到分组概览（组名 + 总大小 + 文件数 + 已选数），按需展开查看明细
- **扫描完成 toast 提示**：扫描发现内容时 toast 提示"发现 N 项可清理内容，共 X"

### 修复

#### 清理游戏垃圾扫描返回空结果
- **根因 1：natives 目录命名规则错误**：代码拼接 `versions/<ver>/natives`，但 MoLaunch 实际命名约定是 `versions/<ver>/<ver>-natives`（带版本前缀），导致扫描全部落空。已改为 `version_path.join(format!("{}-natives", version_name))`
- **根因 2：未适配版本隔离模式**：MoLaunch 默认启用版本隔离（isolation_mode=4），日志、崩溃报告、Fabric 缓存等实际位于 `versions/<ver>/` 下而非 `.minecraft` 根目录，原 `SCAN_DIRS` 只扫描根目录导致这些目录全部被跳过
- **修复**：新增 `VERSION_SCAN_DIRS` 配置，对每个版本目录追加扫描 `logs` / `crash-reports` / `.mixin.out` / `.fabric/processedMods` / `.fabric/remappedJars`，display_name 带 " - {version}" 后缀以区分
- `build_allowed_parents` 同步更新，保证 execute 阶段安全检查与 scan 阶段路径完全一致
- 保留 `ROOT_SCAN_DIRS` 兼容非版本隔离布局（isolation_mode=0）

#### 下载工具三处 Bug 修复
- **分片下载显示多个文件**：`list_downloads` 过滤 `.partN` 临时分片文件（DownloadManager 分片下载时创建的 `file.zip.part0` ~ `file.zip.partN` 临时文件不再出现在"已下载文件"列表中），新增 `is_chunk_part_file` helper 判断文件名后缀
- **删除按钮无 IPC 调用**：`useExternalDownload.ts` 中 `deleteFile` / `cancelDownloadTask` / `resetDownloadDir` 三处误用 `await showConfirm(...)` 当 Promise，但 `showConfirm` 是回调式（`showConfirm(title, msg, onConfirm, onCancel?)` 返回 void），`await undefined` 后 `if (!confirmed) return` 永远退出，导致三个操作全部失效。已改为回调式用法，确认后执行实际逻辑
- **下载目录设置位置**：`ExternalDownload.vue` 的「下载目录」section 从页面顶部移到最下方，符合"先使用后配置"的交互习惯
- **DownloadedFileList.vue 原生 `<button>` 改用 `<Button>` 组件**：遵循项目约定（必须用项目自定义组件而非原生 HTML）

### 变更

#### 设置页面结构调整：拆分「其他」分类
- 删除侧边栏「其他」分类（SettingsOther.vue 已删除）
- **日志级别** 迁移到「进阶设置」页（SettingsAdvanced.vue）的「系统」卡片，通过 useConfigPage 接入配置读写
- **应用版本 + 开发者模式解锁入口 + SDK 信息** 迁移到「更多 → 系统信息」子页签（新建 SystemInfoTab.vue）
- 「更多」顶部子菜单新增「系统信息」页签（关于 / 系统信息 / 鸣谢 / 教程）
- 删除「配置文件路径」展示（不再需要）
- 侧边栏「高阶配置」重命名为「进阶设置」，desc 同步更新为「日志、代理、CurseForge、社区资源等」
- DevModeToggle.vue 与 developer.ts 注释同步：解锁触发点从「其他」改为「系统信息子页签」
- Settings.vue 关闭开发者模式时的 fallback 路径从「其他」改为「更多」

### 重构

#### 代码质量 V2 - 阶段 5.1：AppConfig struct 嵌套化
- 新增 5 个子 struct：`ProxyConfig`（mode/kind/url）、`DownloadConfig`（source/meta_source/max_speed/max_threads/chunk_count/mirror_*/mirror_mode）、`MemoryConfig`（mode/min/max）、`CommunityConfig`（source/filename_format/mod_local_name_style/ignore_quilt）、`LaunchAdvancedConfig`（disable_jlw/disable_lua/use_dedicated_gpu）
- AppConfig 从 30 平铺字段变为 10 通用 + 5 分组，`proxy_type` 改名 `proxy.kind`（避开 Rust 关键字 `type`）
- INI 手动映射代码（load_config/save_config）更新：section/key 名完全不变，用户配置文件零影响
- 27 个文件的访问点迁移（如 `config.proxy_mode` → `config.proxy.mode`、`config.download_source` → `config.download.source`）
- 编写 INI 迁移工具函数预留（`storage/ini.rs` 的 `merge_missing_from` 已支持旧 key 补全）

#### 代码质量 V2 - 阶段 5.2：VersionSetup struct 嵌套化
- 启用 4 个已建子 struct（`LoaderInfo` / `DisplayConfig` / `JavaConfig` / `AdvancedConfig`），移除 `#[allow(dead_code)]`
- VersionSetup 从 30 平铺字段变为 4 嵌套（loader/display/java/advanced）
- AdvancedConfig 字段去掉 `advance_` 前缀（如 `advance_jvm_args` → `jvm_args`），因已在 `advanced` 子 struct 中前缀冗余
- setup.ini 读写代码更新：section/key 名完全不变
- 10 个文件访问点迁移（如 `setup.java_path` → `setup.java.java_path`、`setup.advance_jvm_args` → `setup.advanced.jvm_args`）
- PersonalizationUpdate 保持平铺（IPC 补丁 DTO，前端 camelCase JSON 传参），仅更新 apply 到 VersionSetup 时的访问路径

#### 代码质量 V2 - 阶段 5.3：ConfigPatch / ConfigSnapshot struct 嵌套化
- 新增 10 个子 struct：`ProxyPatch` / `DownloadPatch` / `MemoryPatch` / `CommunityPatch` / `LaunchAdvancedPatch` + 对应 5 个 Snapshot
- 使用 `#[serde(flatten)]` 将子 struct 字段展平到父 struct 的 JSON 序列化中，配合 `#[serde(rename = "camelCaseKey")]` 保持前端 JSON key 完全不变（如 `proxyMode` / `downloadSource` / `mirrorUrl`）
- `Vec<ConfigEntry>` 扁平传输格式不变，`get_config` / `apply_config` 的 IPC 逻辑零改动，前端零改动
- build_snapshot 更新：从 AppConfig 嵌套字段映射到 ConfigSnapshot 嵌套子 struct
- apply_config_inner 更新：从 ConfigPatch 嵌套字段映射到 AppConfig 嵌套字段
- 3 个文件修改（types.rs + apply.rs + validate.rs）

#### 代码质量 V2 - 阶段 1.6：F5 console.error 吞错模式迁移
- 将 53 处"吞错模式"（catch 块仅 `console.error`，无 toast/rethrow/状态清理）迁移到 `utils/async.ts` 的 `safeCall`/`safeCallSync` 高阶函数
- 覆盖 30 个文件（14 个 TS + 16 个 Vue），包括 stores（version/settings/sdk/plugins/java/auth）、composables（useConfigPage/useDownloadPolling/useLaunchState/useVersionSettings 等）、views（Downloads/Settings/SettingsOther 等）
- 21 处非吞错模式（catch 有 toast/rethrow/状态清理/回退赋值）保留原 try/catch
- 5 处有 finally 的简化为 `await safeCall(...)` + 平铺 cleanup（因 safeCall 已吞异常，cleanup 必然执行）
- `sandbox-bootstrap.ts` 因字符串模板注入 iframe，无法 import 父级 safeCallSync，内联一份最小化实现

#### 代码质量 V2 - 阶段 1.9：B3 log_err 样板迁移
- 将 61 处 `.map_err(|e| e.to_string())` 样板迁移到 `error_util::log_err("语义化 label")`，覆盖 22 个 Rust 文件
- `log_err` 在返回值不变（仍为 `e.to_string()`）的前提下，额外通过 `log_error!` 记录带 label 的错误日志，便于问题定位
- 72 处 `.map_err(|e| format!(...))` 保留不迁移：这些调用刻意构造中文错误文案，前端 toast 直接展示，迁移到 log_err 会改变返回值破坏前端展示
- `log_err_with`（带 context 参数版本）仍为零调用，保留供未来需要附加上下文（如版本号、路径）的场景使用

### 新增

#### 内存优化双模式（轻量 / 强力）
- 后端 `src-tauri/src/commands/tools/memory.rs` 重写为 `NtSetSystemInformation` 方案（与 PCL2 一致）：
  - 通过 FFI 声明 `ntdll.dll` 的 `NtSetSystemInformation` 未公开 API，配合 `SystemMemoryListInformation`（class 80）+ `SYSTEM_MEMORY_LIST_COMMAND` 枚举执行系统级内存操作
  - 轻量模式（light）：仅调用 `MemoryEmptyWorkingSets`，一次系统调用清空所有进程工作集，释放几十~几百 MB，响应快、几乎无副作用
  - 强力模式（strong）：依次执行 `MemoryFlushModifiedList` → `MemoryPurgeLowPriorityStandbyList` → `MemoryPurgeStandbyList` → `MemoryEmptyWorkingSets`，清空 standby list 可释放数 GB
  - 移除原遍历进程 + `SetProcessWorkingSetSize` 方案，移除 `Cargo.toml` 中不再使用的 `Win32_System_Diagnostics_ToolHelp` feature
  - Linux / macOS 保持原有 `malloc_trim` / `malloc_zone_pressure_relief` 实现
- 修改 `src-tauri/src/commands/tools/types.rs`：新增 `MemoryOptimizeParams { mode: String }`，`MemoryOptimizeResult` 新增 `mode` 字段并统一字段单位为字节（`_bytes` 后缀）
- 修改 `src-tauri/src/commands/tools/mod.rs`：`memory_optimize` 分支解析 `MemoryOptimizeParams` 并传递 mode 参数
- 修改 `src/utils/api/tools.ts`：新增 `MemoryOptimizeMode` 类型（`'light' | 'strong'`），`MemoryOptimizeResult` 字段更新为 `_bytes` + `mode`，`memoryOptimize(mode)` 接受模式参数（默认 `light`）
- 修改 `src/views/QuickTools.vue`：
  - 新增轻量 / 强力模式选择器（复用 `Button` 组件 primary/outline 切换，移除 `SegmentedButtons`）
  - 优化按钮移至右侧 flex 布局，与模式选择器并排显示
  - 标题旁加问号图标 + `Tooltip`，多行说明两种模式的差异与副作用
  - 选中强力模式时显示 `AlertV2` warning 提示，警告清空 standby list 会导致已缓存应用下次启动变慢
  - 强力模式点击优化时弹出二次确认对话框，防止用户误操作
  - 优化结果展示本次使用的模式标签，按钮文案随模式动态切换

#### 插件系统基础框架（前端）
- 新增插件系统类型定义 `src/types/plugin.ts`：
  - `PluginCapabilities`：插件能力声明（homePanel 主页右侧内容区组件 / settingsPanel 插件设置组件）
  - `PluginLifecycleHooks`：生命周期钩子（onEnable / onDisable / onGameLaunch / onGameExit / onDownloadComplete）
  - `PluginManifest`：插件清单（id / name / description / version / author / capabilities / hooks）
  - `PluginRuntimeState`：运行时状态（enabled / builtin / lastError）
  - `HomePanelMode`：主页右侧内容区显示模式（`'default' | 'plugin:${string}'`）
- 新增插件 SDK `src/plugins/sdk.ts`：提供有限的后端 API 包装（getConfig 过滤敏感字段 / listInstalledVersions / emit 强制 `plugin:` 前缀 / log），仅暴露安全的只读 API
- 新增内置示例插件 `src/plugins/quick-stats/`：
  - `index.ts`：插件清单，声明 homePanel 能力
  - `QuickStatsPanel.vue`：显示已安装版本数量的简单面板
- 新增插件注册中心 `src/plugins/index.ts`：维护 `builtinPlugins` 数组与 `findPlugin()` 查找函数，预留外部插件动态加载扩展点
- 新增插件状态管理 store `src/stores/plugins.ts`：
  - 双轨制持久化：前端 localStorage（首屏前同步读取避免闪烁）+ 后端 INI `[Plugin]` 节（跨设备同步）
  - 提供 `setPluginEnabled` / `setHomePanelMode` / `syncFromBackend` / `notifyGameLaunch` / `notifyGameExit` 等 action
  - 禁用当前 homePanelMode 对应插件时自动回退到 `default`
- 新增插件管理页面 `src/views/settings/SettingsPlugins.vue`：
  - 列表展示所有已注册插件（名称、描述、版本、作者、能力标识、启用开关）
  - 内置插件标识（不可卸载，仅可禁用）
  - 外部插件加载入口预留（当前置灰，后续版本支持）
  - 空状态提示（图标 + 文字垂直水平居中）
- 修改 `src/views/Settings.vue`：侧边栏在「个性化」与「高阶配置」之间新增「插件」菜单项（PuzzlePieceIcon 图标）
- 修改 `src/views/settings/SettingsPersonal.vue`：新增「主页」配置区块，提供「右侧内容区」Select（默认启动日志 / 已启用且提供 homePanel 的插件）
- 修改 `src/views/Home.vue`：右侧内容区改为动态组件渲染，根据 `homePanelMode` 决定渲染 LaunchLog 或插件 homePanel 组件（插件被禁用或不存在时回退到 LaunchLog）
- 修改 `src/main.ts`：mount 前初始化 pluginStore（从 localStorage 同步加载），mount 后异步调用 `syncFromBackend()` 从后端 INI 同步配置（失败静默回退）

#### 插件系统配置持久化（后端）
- 修改 `src-tauri/src/commands/system/config.rs`：扩展 `is_valid_config_key` 白名单，放行 `[Plugin]` 节下的 `homePanelMode` 键与 `enabled_<id>` 键（id 为 kebab-case 插件 ID），支持插件配置通过 `set_config_value` 持久化到 INI 文件

#### 业务插件扩展（内置）
- 新增内置插件 `src/plugins/launch-history/`：启动历史面板
  - `LaunchHistoryPanel.vue`：展示最近 50 条启动记录（版本名 / 启动时间 / 退出码），实时监听 `plugin:game-launch` / `plugin:game-exit` 事件刷新列表，退出状态以图标区分（成功 CheckCircleIcon / 失败 XCircleIcon）
  - `index.ts`：插件清单，id=`launch-history`，声明 homePanel 能力
- 新增内置插件 `src/plugins/system-monitor/`：系统状态监控面板
  - `SystemMonitorPanel.vue`：内存使用进度条（>=80% 红色 / >=60% 黄色 / 否则绿色）+ 游戏进程状态 + SDK 初始化状态，每 3 秒轮询刷新
  - `index.ts`：插件清单，id=`system-monitor`，声明 homePanel 能力
- 新增内置插件 `src/plugins/version-stats/`：版本统计图表
  - `VersionStatsPanel.vue`：按加载器分类横向条形图（vanilla / forge / fabric / neoforge / optifine / liteloader）+ 按主版本号分布统计
  - `index.ts`：插件清单，id=`version-stats`，声明 homePanel 能力
- 修改 `src/plugins/quick-stats/index.ts`：补充 `builtin: true` 字段以区分内置 / 外部插件
- 修改 `src/plugins/index.ts`：注册 4 个内置插件（quick-stats / launch-history / system-monitor / version-stats）
- 修改 `src/types/plugin.ts`：`PluginManifest.capabilities` 改为可选字段（`capabilities?`），新增 `builtin: boolean` 字段；新增 `ExternalPluginManifest` 接口（含 entry / permissions 字段）
- 修改 `src/plugins/sdk.ts`：扩展 `PluginSdk` 接口，新增 `listInstalledVersionsWithType()` / `listLaunchHistory()` / `getSystemMemory()`（返回 total/used/available/usage_percent）/ `getRunningGamePid()` 四个方法
- 修改 `src/utils/api/launch.ts`：新增 `LaunchHistoryEntry` 接口与 `getLaunchHistory()` 函数
- 后端新增命令 `get_launch_history`（`src-tauri/src/commands/version/launch.rs`）：返回 `AppState.launch_history` 中最近 50 条记录（按时间倒序），并在 `lib.rs` 注册
- 调用方适配：`Home.vue` / `SettingsPersonal.vue` / `SettingsPlugins.vue` 中 `manifest.capabilities()` 改为 `manifest.capabilities?.()` 以兼容外部插件不实现该字段的情况

#### 外部插件沙箱加载机制
- 后端新增外部插件管理模块 `src-tauri/src/commands/plugins/mod.rs`，提供 5 个 IPC 命令并在 `lib.rs` 注册：
  - `list_external_plugins`：扫描 `<base_dir>/plugins/<plugin_id>/` 目录，读取每个插件的 `manifest.json`，要求 manifest.id 与目录名一致
  - `read_external_plugin_file`：读取外部插件文件内容，使用 `canonicalize` + `starts_with` 双重校验防止 `../` 路径遍历攻击
  - `install_external_plugin_from_dir`：递归复制源目录到 `plugins/<id>/`，安装前校验插件 ID 合法性（kebab-case：小写字母 + 数字 + 连字符，不以连字符开头 / 结尾）
  - `install_external_plugin_from_zip`：从 ZIP 文件路径安装插件，支持扁平结构和单根目录结构两种 ZIP 格式，带 Zip Slip 路径遍历防护（canonicalize 父目录后校验目标在 dst 内），跨盘符 rename 失败时自动回退到递归复制
  - `uninstall_external_plugin`：二次 canonicalize 校验后删除插件目录
- 后端新增 `src-tauri/src/commands/mod.rs`：声明 `pub mod plugins;`
- 前端新增外部插件 API 封装 `src/utils/api/plugins.ts`：导出 `ExternalPluginManifest` / `ExternalPluginEntry` 类型与 `listExternalPlugins` / `readExternalPluginFile` / `installExternalPluginFromDir` / `installExternalPluginFromZip` / `uninstallExternalPlugin` 五个函数，并在 `src/utils/tauri.ts` re-export
- 前端新增沙箱引导脚本 `src/plugins/sandbox/sandbox-bootstrap.ts`：
  - `SANDBOX_BOOTSTRAP_SCRIPT`：注入到 iframe 的引导脚本字符串，暴露 `window.molaunch` 全局对象（含 SDK 方法 + `onEvent` 事件订阅）
  - 基于 postMessage 的通信协议（request / response / event / ready 四种消息类型）
  - `buildSandboxHtml(pluginHtml, pluginId)`：构造完整 HTML 文档，在 `</body>` 前注入引导脚本
- 前端新增沙箱代理组件 `src/plugins/sandbox/PluginSandbox.vue`：
  - 使用 `<iframe sandbox="allow-scripts" :srcdoc="sandboxHtml">` 加载外部插件 HTML（无 `allow-same-origin`，确保 iframe 无法访问父窗口 DOM / cookie / localStorage）
  - `handleMessage()`：监听 iframe postMessage，根据 manifest.permissions 白名单转发到 pluginSdk（`ALWAYS_ALLOWED = new Set(['emit', 'log'])` 始终放行）
  - 桥接 `plugin:game-launch` / `plugin:game-exit` 事件到 iframe
  - `onUnmounted` 时拒绝所有 pending 请求，避免内存泄漏
- 前端完全重写 `src/stores/plugins.ts`：
  - 新增 `externalManifestToPluginManifest()` 转换函数，使用 `markRaw(defineComponent({...}))` 为外部插件构造 PluginSandbox 包装组件作为 homePanel
  - 同步外部插件 `permissions` 字段到 PluginManifest，供插件管理页展示
  - 新增 `externalPluginsRaw` ref 保留原始外部插件清单
  - 新增 `builtinPluginList` / `externalPluginList` 计算属性
  - `initRuntimeStates()` 改为保留已存在状态，内置默认启用、外部默认禁用
  - 新增 `loadExternalPlugins()`：扫描后端目录并合并清单
  - 新增 `installFromDir(sourceDir)`：从文件夹安装后重新加载
  - 新增 `installFromZip(zipPath)`：从 ZIP 文件路径安装后重新加载
  - 新增 `uninstallExternal(pluginId)`：卸载后重新加载，若卸载的是当前 homePanelMode 对应插件则回退到 default
  - `syncFromBackend()` 现在先调用 `loadExternalPlugins()` 确保外部插件能被后端 INI 配置覆盖
  - `notifyGameLaunch` / `notifyGameExit` 现在同时派发 window CustomEvent 桥接到沙箱
  - 将动态 `import('@tauri-apps/api/core')` 改为静态 import，消除 vite 构建的 mixed-import 警告
- 前端扩展 `src/types/plugin.ts`：`PluginManifest` 新增可选 `permissions?: string[] | null` 字段（内置插件为 null 表示无沙箱限制，外部插件为数组）
- 前端完全重写 `src/views/settings/SettingsPlugins.vue`：
  - 修复 Tooltip 空 bug：`Tooltip` 组件 prop 是 `text` 不是 `content`，原代码 `:content="..."` 导致悬停时显示空 tooltip
  - 列表展示所有插件（内置 + 外部），区分来源标识（内置灰色 / 外部蓝色），内置在前按 ID 排序
  - 每个插件卡片新增「已声明权限」展示区：内置插件显示绿色「全部（无沙箱限制）」tag，外部插件显示蓝色权限 tag + 灰色始终允许权限 tag（emit / log 带 * 后缀）
  - 顶部统计区新增内置 / 外部数量分类
  - 外部插件卸载按钮（仅外部插件显示，`v-if="!manifest.builtin"`）
  - 「从文件夹安装」按钮 + 「从 ZIP 文件安装」按钮（调用 `selectFile()` 选择 .zip 文件 + `pluginStore.installFromZip()`）
  - 刷新按钮重新扫描外部插件
  - 可用权限展示从纯文本改为 tag 列表（蓝色可用权限 + 灰色始终允许权限 + 「带 * 号的权限始终允许」说明）

#### 缓存统计统一 IPC 端口
- 后端新增 `src-tauri/src/utils/cache_stats.rs` 统一缓存统计工具：
  - `CacheStat` 结构：name / category / subDir / path / fileCount / totalSize / ttlHours
  - `CacheStatsResult` 结构：按类别分组（cache / cacheTemp / cacheApp）
  - `collect_all()`：递归统计所有缓存子目录的文件数和占用大小，附带 TTL 信息
  - 统计范围：cache 下 4 个子目录（images / forge_installer / preload_mods / launch，均 24h TTL）、cacheTemp 下 2 个子目录（TaskTemp 24h TTL / sdk 不清理）、cacheApp 下 runtime/{component} 每个 Java Runtime 单独统计（不清理）
  - 不存在或为空的 runtime/ 目录返回占位项便于 UI 展示路径
  - `walk_dir()` 递归遍历辅助函数，失败时静默跳过
  - `root_dirs()` 返回三个缓存根目录路径列表（供 UI 展示父目录用）
- 后端 `src-tauri/src/utils/mod.rs`：注册 `cache_stats` 子模块
- 后端新增 `get_cache_stats` IPC 命令（`src-tauri/src/commands/system/developer.rs`）：在 `spawn_blocking` 中执行 `cache_stats::collect_all()`，避免阻塞主线程；在 `lib.rs` 注册
- 前端 `src/utils/api/developer.ts`：新增 `CacheStat` / `CacheStatsResult` 类型与 `getCacheStats()` 函数
- 前端 `src/views/settings/SettingsDeveloper.vue` 接入缓存统计展示：
  - 新增「缓存统计」卡片，展示总文件数、总占用大小
  - 列表展示每个子目录：名称 + TTL 标识（黄色 24h 自动清理 / 灰色不清理）+ 类别 tag + 文件数 + 占用大小 + 完整路径 + 打开按钮
  - 刷新按钮（旋转图标动画）
  - 原有「缓存目录」卡片保留，仅展示父目录路径便于整体定位
  - `onMounted` 并行加载 storageDirs / systemInfo / cacheStats

#### 通用工具函数提取（版本号解析 / 文件名校验 / 时间解析）
- 排查后端各模块内的私有工具函数，将 3 个具有通用性的函数提取到 `utils` 下统一管理，消除 5 处重复实现
- 新增 `src-tauri/src/utils/version.rs`：版本号解析工具，提供 `parse_number(version: &str) -> Vec<u32>`（如 "1.20.1" -> [1, 20, 1]）
  - 从 `minecraft/loaders/utils.rs` 提取（原 `parse_version_number`）
  - `minecraft/version/libraries.rs`：`compare_versions_ge` 中手写的 `a.split('.').filter_map(|p| p.parse().ok()).collect()` 重复实现改用 `utils::version::parse_number`
- 新增 `src-tauri/src/utils/path.rs`：路径与文件名安全工具，提供 `sanitize_file_name(name: &str) -> Result<(), String>`（拒绝空字符串、路径分隔符、路径遍历 `..`、空字节）
  - 从 `commands/version/mods/helpers.rs` 提取（原 `pub(super) fn sanitize_file_name`）
- 新增 `src-tauri/src/utils/datetime.rs`：时间解析与格式化工具，提供 `parse_utc(s: &str) -> Option<DateTime<Utc>>` 和 `format_utc_to_local(s: &str) -> Option<String>`
  - 从 `minecraft/loaders/utils.rs` 提取（原 `parse_utc_to_local`，重命名为 `format_utc_to_local`）
  - 支持 4 种格式：RFC3339、naive datetime（T 分隔）、naive datetime（空格分隔）、纯日期
  - `commands/version/list.rs`：`parse_timestamp` 中手写的 RFC3339 + naive datetime 解析改用 `utils::datetime::parse_utc`
  - `minecraft/fools.rs`：`parse_april_fools_date` 中手写的时间解析改用 `utils::datetime::parse_utc`
  - `minecraft/version/state.rs`：`is_old_version` 中手写的 RFC3339 + naive datetime 解析改用 `utils::datetime::parse_utc`
- `src-tauri/src/utils/mod.rs`：注册 `version` / `path` / `datetime` 三个子模块
- 调用方改造：
  - `minecraft/loaders/forge.rs`：`utils::parse_version_number` → `crate::utils::version::parse_number`，`utils::parse_utc_to_local` → `crate::utils::datetime::format_utc_to_local`
  - `minecraft/loaders/neoforge.rs`：同上
  - `minecraft/loaders/forge_html.rs`：同上
  - `minecraft/launch/skin_resourcepack.rs`：`crate::minecraft::loaders::utils::parse_version_number` → `crate::utils::version::parse_number`
  - `commands/version/mods/mod.rs`：`helpers::sanitize_file_name` → `crate::utils::path::sanitize_file_name`
- 文件删除：
  - `minecraft/loaders/utils.rs`：**整个文件删除**，两个函数均已迁移到 `utils::version` 和 `utils::datetime`
  - `minecraft/loaders/mod.rs`：移除 `pub mod utils;` 声明
- 函数清理：
  - `commands/version/mods/helpers.rs`：删除 `sanitize_file_name` 函数（已迁移到 `utils::path`），保留 `get_mods_dir`（业务专属）
  - `commands/version/mods/mod.rs`：模块结构文档注释更新

#### 字节数格式化统一工具函数
- 后端此前有 3 处重复实现 `format_bytes` / `format_speed` 函数（cache_cleanup.rs、download/chunk/util.rs、community/install/helpers.rs），逻辑相同但实现略有差异（小数位数、是否支持 GB 等），违反 DRY 原则。本次改造统一收口到 `utils/format` 模块
- 新增 `src-tauri/src/utils/format.rs`：提供 `bytes()` / `bytes_with()` / `speed()` / `speed_with()` 四个自由函数，支持 B/KB/MB/GB/TB 五档单位，可通过 `decimals` 参数指定小数位数（默认 1 位）
- 实现细节：
  - 通过循环确定单位档位，避免使用 `f64::log` 等不稳定的浮点对数计算
  - 边界处理：0 字节返回 "0 B"，超出 TB 范围夹紧到 TB
  - 速度格式化复用字节数格式化逻辑，追加 "/s" 后缀
- `src-tauri/src/utils/mod.rs`：注册 `format` 子模块
- 调用方改造（全部改用 `crate::utils::format`）：
  - `src-tauri/src/utils/cache_cleanup.rs`：删除本地 `format_bytes` 函数，改用 `format::bytes_with(bytes, 2)`（缓存统计用 2 位小数）
  - `src-tauri/src/minecraft/download/chunk/util.rs`：**整个文件删除**，原 `format_bytes` / `format_speed` 两个函数已迁移到 `utils::format`
  - `src-tauri/src/minecraft/download/chunk/mod.rs`：移除 `pub mod util;` 和 `use self::util::{format_bytes, format_speed};`，改用 `use crate::utils::format;`，所有调用点改为 `format::bytes()` / `format::speed()`
  - `src-tauri/src/minecraft/download/chunk/probe.rs`：改用 `use crate::utils::format;`
  - `src-tauri/src/minecraft/download/chunk/download.rs`：改用 `use crate::utils::format;`
  - `src-tauri/src/commands/community/install/helpers.rs`：删除本地 `format_bytes` 函数，模块文档注释更新
  - `src-tauri/src/commands/community/install/modrinth.rs`：`super::helpers::format_bytes` → `crate::utils::format::bytes`
  - `src-tauri/src/commands/community/install/curseforge.rs`：同上
  - `src-tauri/src/commands/community/install/modpack_stages.rs`：`use super::helpers::format_bytes` → `use crate::utils::format`，调用点改为 `format::bytes()`
  - `src-tauri/src/commands/community/install/mod.rs`：模块结构文档注释移除 `format_bytes` 描述

#### 缓存定期清理机制（24h 自动清理）
- 新增 `src-tauri/src/utils/cache_cleanup.rs`：自动清理超过 24h 的不重要缓存文件，避免磁盘占用无限增长
- 清理范围：
  - `.Molaunch/cache/images/`：图片缓存（皮肤、披风、头像），可重新下载
  - `.Molaunch/cache/forge_installer/`：Forge 安装器注入资源，可重新释放
  - `.Molaunch/cache/preload_mods/`：社区资源预加载缓存（已有 6h TTL，物理文件随本机制一并清理）
  - `.Molaunch/cache/launch/`：嵌入 jar 释放（lwjgl-unsafe-agent、java-wrapper），可重新释放
  - `<temp>/MoLaunch/TaskTemp/`：Forge/NeoForge 安装包临时下载
- 不清理（重要资源）：
  - `<temp>/MoLaunch/sdk/`：SDK 动态库，有 sha256 校验机制，运行中清理会导致加载失败
  - `%APPDATA%/.minecraft/runtime/`：Java Runtime，下载耗时长，跨游戏目录共享
- 触发时机：
  - **启动时**：立即执行一次清理（清理上次运行遗留的过期文件）
  - **定时任务**：每 1h 检查一次（避免频繁 IO，又能在合理时间内清理过期文件）
- 实现细节：
  - `run_cleanup()`：同步阻塞函数，遍历所有需清理目录，删除 mtime > 24h 的文件和空目录
  - `spawn_cleanup_task()`：通过 `tauri::async_runtime::spawn_blocking` 在独立线程执行，避免阻塞 async 运行时
  - 清理结果通过日志输出（删除文件数、目录数、释放空间大小、错误数、耗时）
  - `CleanupResult` 结构体统计清理结果，`format_bytes()` 格式化字节数为人类可读字符串
  - 过期判断基于文件 mtime，遇到系统时间异常（mtime 在未来）时跳过，避免误删
- `src-tauri/src/utils/mod.rs`：注册 `cache_cleanup` 子模块
- `src-tauri/src/lib.rs`：在 CurseForge 初始化后、Tauri Builder 构造前调用 `spawn_cleanup_task()`

#### 缓存访问统一收口（utils 层 + storage 层分离）
- 后端三种缓存位置此前散落在各业务模块中，通过直接拼接 `std::env::temp_dir()` / `std::env::var("APPDATA")` 或直接 `use crate::storage::cache::Cache` 访问，路径生成逻辑分散且无统一入口。本次改造将所有缓存访问收口到 `utils` 层自由函数，`storage` 层保留底层单例实现，业务模块不再直接依赖 `storage::cache*` 或手动拼接环境变量
- 新增 `src-tauri/src/storage/cache_temp.rs`：系统临时目录缓存底层实现，管理 `<temp>/MoLaunch/` 目录，提供 `CacheTemp` 单例（`task_temp_dir()` / `ensure_task_temp_dir()` / `sdk_dir()` / `ensure_sdk_dir()` / `sdk_library_path()`），覆盖 Forge/NeoForge 安装包临时下载（TaskTemp）和 SDK 动态库释放（sdk）
- 新增 `src-tauri/src/storage/cache_app.rs`：AppData 缓存底层实现，管理 `%APPDATA%/.minecraft/runtime/` 目录，提供 `CacheApp` 单例（`runtime_dir()` / `ensure_runtime_dir()`），覆盖 Java Runtime 存储（Mojang 官方位置，跨游戏目录共享）
- 新增 `src-tauri/src/utils/cache.rs`：运行路径缓存工具（`.Molaunch/cache/`），包装 `storage::cache::Cache` 单例为自由函数（`dir` / `path` / `ensure_dir` / `exists` / `read` / `read_bytes` / `write` / `write_bytes` / `remove` / `list` / `clear_dir`）
- 新增 `src-tauri/src/utils/cache_temp.rs`：系统临时目录缓存工具，包装 `storage::cache_temp::CacheTemp` 单例为自由函数
- 新增 `src-tauri/src/utils/cache_app.rs`：AppData 缓存工具，包装 `storage::cache_app::CacheApp` 单例为自由函数
- `src-tauri/src/storage/mod.rs`：注册 `cache_app` / `cache_temp` 子模块
- `src-tauri/src/utils/mod.rs`：注册 `cache` / `cache_app` / `cache_temp` 子模块，文档注释中补充三种缓存位置对照表
- 调用方改造（全部改用 `utils::cache*` 自由函数）：
  - `src-tauri/src/minecraft/image_cache.rs`：图片缓存读写（`Cache::instance().path/exists/write_bytes/remove/clear_dir` → `cache::path/exists/write_bytes/remove/clear_dir`）
  - `src-tauri/src/minecraft/loaders/forge_installer.rs`：Forge 安装器注入资源释放（`Cache::instance().ensure_dir` → `cache::ensure_dir`）
  - `src-tauri/src/minecraft/launch/embedded.rs`：嵌入资源 jar 释放（`Cache::instance().exists/path` → `cache::exists/path`）
  - `src-tauri/src/minecraft/community/preload/cache.rs`：社区资源预加载缓存读写（`Cache::instance().read/write` → `cache::read/write`）
  - `src-tauri/src/minecraft/loaders/forge.rs` / `neoforge.rs`：安装包临时下载（`std::env::temp_dir().join("MoLaunch").join("TaskTemp")` + `std::fs::create_dir_all` → `utils::cache_temp::ensure_task_temp_dir`）
  - `src-tauri/src/sdk/mod.rs`：SDK 动态库路径（`std::env::temp_dir().join("MoLaunch").join("sdk").join(filename)` → `utils::cache_temp::sdk_library_path`）
  - `src-tauri/src/resources.rs`：SDK 释放目标路径（同上）
  - `src-tauri/src/minecraft/java/download/files.rs`：Java Runtime 目录（`std::env::var("APPDATA").join(".minecraft").join("runtime").join(component)` → `utils::cache_app::runtime_dir`）
  - `src-tauri/src/minecraft/java/search.rs`：Java 搜索 Step 5 搜索 APPDATA runtime 目录（`std::env::var("APPDATA").join(".minecraft").join("runtime")` → `utils::cache_app::runtime_base_dir`）
- 开发者页扩展展示三种缓存位置：
  - `src-tauri/src/commands/system/developer.rs`：`StorageDirs` 新增 `cache_temp` / `cache_app` 字段（serde 序列化为 `cacheTemp` / `cacheApp`），`get_storage_dirs` 命令返回所有缓存路径
  - `src/utils/api/developer.ts`：`StorageDirs` 接口同步新增 `cacheTemp` / `cacheApp` 字段
  - `src/views/settings/SettingsDeveloper.vue`：缓存卡片条目从 2 条扩展为 4 条（运行路径缓存 / 运行路径临时 / 系统临时缓存 / AppData 缓存），标签更清晰

#### 统一 User-Agent 标识
- `src-tauri/src/http.rs`：所有外部 HTTP 请求统一附加 UA 头，格式 `MoLaunch/<os> <version>`（如 `MoLaunch/windows 0.1.0`），覆盖皮肤/披风下载、头像缓存、BMCLAPI 镜像源、MC 文件下载、Java 下载、微软账号 OAuth 登录、社区资源下载等所有走 `http::get_client()` / `http::build_client()` 的请求；`<os>` 运行时取 `std::env::consts::OS`（windows/macos/linux），`<version>` 编译时通过 `env!("CARGO_PKG_VERSION")` 从 Cargo.toml 注入；替换原 reqwest 默认 UA `reqwest/<version>`，避免被部分 WAF/CDN 识别为爬虫返回 403

#### 浏览器环境拦截提示
- `src/main.ts`：在最早入口处检测 `window.__TAURI_INTERNALS__`（Tauri 2 在 WebView 中注入的全局对象），若不存在则判定为浏览器环境，直接渲染友好提示页（SVG 警告图标 + "小朋友，此页面默认给 Tauri 客户端使用，请勿使用浏览器直接打开呦？！"），并阻止 Vue app 挂载，避免 `@tauri-apps/api` 的 `getCurrentWindow()` 在浏览器中抛 "Cannot read properties of undefined (reading 'metadata')" 导致 TopNavLayout setup 崩溃刷屏；Tauri WebView 中正常走原挂载流程

#### 主题色自定义（Arco Design 风格颜色选择器 + 后端持久化）
- `src/utils/color.ts`：新建颜色工具模块，提供 HEX ↔ RGB ↔ HSL 互转、`generateColorScale` 由主色生成 50~950 共 11 档色阶（基于 HSL 调整 L 值，极亮/极暗档微调饱和度防发灰）、`applyPrimaryColor` 把色阶写入 `:root` 的 CSS 变量（同时输出 HEX 形式 `--color-primary-{n}` 与 RGB 空格分隔形式 `--color-primary-rgb-{n}`，供 Tailwind `rgb(var(...) / <alpha>)` 使用，并打印诊断日志便于排查）、`PRESET_COLORS` 12 个预设色板
- `src/components/common/ColorPicker.vue`：新建自研颜色选择器组件，参考 Arco Design Vue `<color-picker>` 视觉风格（文件头部按项目规范添加 Arco Design Vue 版权声明）；触发器 32px 高（色块 + HEX 文本 + 下拉箭头），与项目自研 Select 风格一致；下拉面板含预设色板（6 列 × 2 行 = 12 色，参考 Arco 默认色板）+ 自定义 HEX 输入框（实时校验，3 位缩写自动扩展为 6 位，无效输入显示红框 + 错误提示并回退原值）；弹层定位逻辑复用 Select.vue 实现（视口空间不足自动向上展开 + scroll 事件关闭）并补充横向边界夹紧（右侧/左侧空间不足时左移到 `viewportW - margin - dropdownW`，避免下拉面板被窗口边框截断）；"自定义颜色"标题与上方色板通过 `:not(:first-child) { margin-top: 16px }` 拉开距离；scaleY + opacity 弹出动画；select 函数添加诊断日志
- `src/stores/settings.ts`：新增 `primaryColor` 字段（默认 `"#165dff"` Arco 蓝）与 `setPrimaryColor()` 方法；`loadSettings()` 末尾立即调用 `applyPrimaryColor()` 注入 CSS 变量；存储双轨制——前端 localStorage（首屏前同步读取避免闪烁）+ 后端 INI（跨设备同步）；新增 `syncPrimaryColorFromBackend()` 启动后从后端拉取覆盖前端，后端无值时把前端默认值同步到后端；`setPrimaryColor()` 同时做四件事：立即注入 CSS 变量 + 写 localStorage + 异步 `applyConfig({ primaryColor })` 写后端 INI + toastSuccess 提示用户操作已生效
- `src/main.ts`：在 `app.mount()` 之前先调用 `useSettingsStore(pinia)` 触发 `loadSettings()`，确保 CSS 变量在 Vue 渲染前就注入到 `:root`，避免首屏蓝色 → 用户自定义色的闪烁
- `src/App.vue`：`initApp()` 中异步触发 `settingsStore.syncPrimaryColorFromBackend()`，不阻塞主流程
- `tailwind.config.js`：`primary` 调色板 11 档从硬编码 HEX 改为 `rgb(var(--color-primary-rgb-{n}) / <alpha-value>)` 形式，让所有 `text-primary-*` / `bg-primary-*` / `border-primary-*` 工具类自动跟随 CSS 变量
- `src/assets/styles/main.css`：在 `:root` 定义完整 11 档色阶默认值（Arco 蓝 `#165dff` 系列），含 HEX 与 RGB 双形式；`--color-primary` 兼容旧变量指向 500 档 RGB；所有 `.btn-primary` / `.btn-outline` / `.btn-text` / `.btn-ghost:hover` / `.input:focus` 中的硬编码 `#165dff` / `#4080ff` / `#0e42d2` / `#94bfff` 改为 `var(--color-primary-{n})`；`body` 背景色从硬编码 `#e0ecff` 改为 `rgb(var(--color-primary-rgb-100) / 0.25)` 跟随主题色
- `src/components/layout/TopNavLayout.vue`：主内容区背景从内联 `style="background-color: #e0ecff"` 改为 `bg-primary-100/30` Tailwind 类，跟随主题色
- `src/components/common/Input.vue`：`.input-wrapper:focus-within` 边框色从 `#165dff` 改为 `var(--color-primary-500)`
- `src/components/common/Select.vue`：`.select-trigger.active` 边框色 + `.select-check-icon` 颜色从 `#165dff` 改为 `var(--color-primary-500)`
- `src/components/common/BackToTop.vue`：渐变背景从 `linear-gradient(135deg, #3b82f6, #2563eb)` 改为 `linear-gradient(135deg, var(--color-primary-500), var(--color-primary-600))`；阴影 rgba 改为 `rgb(var(--color-primary-rgb-600) / 0.4)`
- `src/components/home/LaunchPanel.vue`：启动按钮 hover 态从混用 `border-blue-500 text-blue-500` 改为统一 `border-primary-500 text-primary-500`
- `src/views/settings/SettingsPersonal.vue`：移除原"主题"Select（浅色/深色/跟随系统，本就未生效）；新增"主题色"ColorPicker，绑定 `settingsStore.primaryColor`，描述"控制菜单栏、按钮、选中态等所有主色区域"
- 后端配置链路（双轨制持久化）：
  - `src-tauri/src/state/config.rs`：`AppConfig` 新增 `primary_color: String` 字段，默认 `"#165dff"`
  - `src-tauri/src/config.rs`：INI 加载/保存新增 `primary_color` 键（`[General]` 段下）
  - `src-tauri/src/commands/system/apply_config/types.rs`：`ConfigPatch` 新增 `primary_color: Option<String>`；`ConfigSnapshot` 新增 `primary_color: String`；`build_snapshot` 补充字段映射
  - `src-tauri/src/commands/system/apply_config/apply.rs`：`apply_launcher` 函数新增 `primary_color` 字段处理
- `src/utils/api/config.ts`：`ConfigSnapshot` 新增 `primaryColor: string`；`ConfigPatch` 新增 `primaryColor?: string`
- 受影响区域（全部跟随主题色变化）：顶栏背景 `bg-primary-600`、主内容区背景 `bg-primary-100/30` + body `rgb(var(--color-primary-rgb-100) / 0.25)`、设置/版本设置/下载/文件夹选择侧栏选中态 `bg-primary-50 text-primary-700 border-primary-500`、SubTabBar / SegmentedButtons 选中态、所有 `.btn-primary` / `.btn-outline` / `.btn-text` 按钮、所有 Input / Select 聚焦边框、BackToTop 渐变按钮、LaunchPanel 启动按钮、社区资源/版本选择/账号卡片等约 100 处 `primary-*` 使用点
- 不受影响区域：47 处 `text-blue-*` / `bg-blue-*` 硬编码（Alert info 提示框、Toast info、Java 下载进度条等"信息提示"语义场景）保留原色，符合 Arco Design 中 info 蓝与 primary 蓝分离的设计规范
- 用户操作反馈：选择颜色后立即注入 CSS 变量生效 + toastSuccess 提示"主题色已更新为 #XXXXXX" + 后端 INI 持久化（跨设备同步）

#### 设置页个性化补充：游戏默认界面语言 + 启动器语言固定简体中文
- `src-tauri/src/state/config.rs`：`AppConfig` 新增 `game_language: String` 字段，默认值 `"zh_cn"`（启动器语言固定简体中文，无需"跟随启动器"选项），支持 `"none"`（不设置）与 MC 标准语言代码（`zh_cn` / `en_us` / `ja_jp` / `ko_kr` / `fr_fr` / `de_de` / `ru_ru` 等）；`"auto"` 作为旧配置兼容值保留处理
- `src-tauri/src/config.rs`：INI 加载/保存新增 `game_language` 字段（`[General]` 段下 `game_language` 键），持久化到本地配置文件
- `src-tauri/src/commands/system/apply_config/types.rs`：`ConfigPatch` 新增 `game_language: Option<String>`（可选更新）；`ConfigSnapshot` 新增 `game_language: String`（快照返回）；`build_snapshot` 补充字段映射
- `src-tauri/src/commands/system/apply_config/apply.rs`：`apply_launcher` 函数新增 `game_language` 字段处理，写入 AppConfig 并通过 `apply_config` IPC 命令统一更新
- `src-tauri/src/minecraft/language.rs`：完全重写 `set_game_language` 函数，签名从 `(game_dir, version_id, mc_version)` 改为 `(game_dir, version_id, mc_version, target_lang)`，从硬编码中文改为接受任意目标语言；新增 `adjust_lang_case` 函数根据 MC 版本自动调整大小写（1.0~1.10 用 `zh_CN` 大写后缀，1.11+ 用 `zh_cn` 小写，26+ 用小写）；新增 `to_upper_suffix` 辅助函数；每个分支补充 `[Language]` 前缀日志；保留老用户保护机制（saves/ 存在时不覆盖已有语言）
- `src-tauri/src/minecraft/launch/arguments.rs`：`build_launch_arguments` 新增 `game_language: Option<&str>` 参数；仅当 game_language 非空且非 `"none"` 时才调用 `set_game_language`；修复原代码 bug——`set_game_language` 的 `mc_version` 参数原本误传 `version_id`（如 `"1.20.1-forge"`），改为调用 `detect_version_and_loader` 获取真实 MC 版本号（如 `"1.20.1"`），避免 `adjust_lang_case` 解析版本号失败；同时修复 `helpers` 私有模块访问错误，改用 `crate::minecraft::version::setup::detect_version_and_loader` 公共再导出路径
- `src-tauri/src/minecraft/launch/pipeline/types.rs`：`LaunchConfig` 新增 `#[serde(default)] pub game_language: Option<String>` 字段，支持从前端启动请求透传
- `src-tauri/src/minecraft/launch/pipeline/validate.rs`：`build_arguments` 方法透传 `self.config.game_language.as_deref()` 到 `build_launch_arguments`
- `src-tauri/src/commands/version/launch.rs`：新增 `resolve_game_language` 辅助函数（`none` → `None`；`auto` 旧配置兼容 → 跟随启动器语言映射 `zh-CN` → `zh_cn`、`en-US` → `en_us`；其他 → 直接返回）；`LaunchConfig` 构造时填充 `game_language` 字段
- `src-tauri/src/commands/version/script_export.rs`：`build_launch_arguments` 调用新增 `None` 参数（导出脚本时不设置游戏语言，避免副作用）
- `src/utils/api/config.ts`：`ConfigSnapshot` 新增 `gameLanguage: string`；`ConfigPatch` 新增 `gameLanguage?: string`
- `src/views/settings/SettingsPersonal.vue`：完全重写——启动器语言固定仅简体中文（移除 English 选项，Select 仅含 `简体中文` 一项）；新增「游戏」分组含「默认界面语言」Select（8 个选项：简体中文 / English / 日本語 / 한국어 / Français / Deutsch / Русский / 不设置，默认简体中文）；通过 `getConfigMap` / `applyConfig` IPC 读写后端配置，`watch` 自动保存；`loadGameLanguage` 兼容旧配置读到 `auto` 时回退为 `zh_cn`

#### options.txt 语言设置逻辑优化（5 分支处理）
- `src-tauri/src/minecraft/language.rs`：`set_game_language` 优化为 5 分支处理逻辑：
  1. **options.txt 不存在**：创建文件并写入 `lang:<target>`，不写入其他字段
  2. **文件存在，lang 字段不存在**：补充 lang 字段到文件末尾（不创建新文件）
  3. **文件存在，lang 已是目标语言**：跳过，不写入（避免无意义 IO）
  4. **文件存在，lang 是其他语言且 saves/ 不存在**：覆盖为目标语言（先写 `-` 触发缓存清空，再写目标值，PCL2 风格）
  5. **文件存在，lang 是其他语言且 saves/ 已存在**：跳过，尊重老用户手动选择的语言
- 补充 `#[cfg(test)]` 单元测试覆盖 `adjust_lang_case` 与 `to_upper_suffix`：MC 1.0~1.10 大写后缀、1.11+ 小写、26+ 小写、无下划线代码原样返回

#### 关于页新增 MoLaunch 实现原理介绍
- `src/components/about/MoLaunchIntro.vue`：新增组件，默认折叠，点击标题栏展开 200 字实现说明，内容涵盖技术栈选型（Tauri 2 + Vue 3 + Rust）、启动器核心实现（版本管理、Java 检测、游戏启动）、联机模块（FRP 隧道 SDK 动态库嵌入与释放）、UI 设计理念（参考 PCL2 / Arco Design）、数据存储与安全（设备 ID 派生密钥加密）
- `src/views/settings/SettingsMore.vue`：在「关于」子页签的 MoLaunch 介绍卡片与技术栈卡片之间插入 `<MoLaunchIntro />` 组件

#### 窗口尺寸固定不可缩放
- `src-tauri/tauri.conf.json`：`resizable` 改为 `false`，移除 `minWidth`/`minHeight`，窗口固定为 1096×592 不可拖拽缩放
- `src-tauri/src/lib.rs`：移除 `setup` 钩子中的 `set_min_size` 调用，移除 `on_window_event` 中 `Resized` 事件的夹紧逻辑（不再需要）

#### 开发者模式开关与设备 ID 显示优化
- `src/components/settings/DevModeToggle.vue`：开启/关闭双按钮组改为 Select 平行布局（与设置页其他选择器风格一致）
- `src/views/settings/SettingsOther.vue`：设备 ID 默认打码显示（前 4 位 + `****` + 后 4 位），双击切换全额显示；打码状态下点击图标 Tooltip 提示"双击切换全额显示 / 打码"；全额显示时下方常驻 Alert 警告"设备 ID 已全额显示，本 ID 用于本地数据加密存储，请勿截图外传或泄露给他人"；切到其他设置页时随组件卸载自动隐藏提示

#### 设置页选择器统一改为自定义 Select 组件
- `src/views/settings/SettingsDownload.vue`：版本列表源、文件下载源两处 SegmentedButtons 改为 Select 组件，布局调整为标题左/Select 右的平行布局（`flex items-center justify-between`，Select 固定宽 `w-40`）
- `src/views/settings/SettingsPersonal.vue`：外观的主题（浅色/深色/跟随系统，补充深色选项）、语言（简体中文/English）两处按钮组改为 Select 平行布局
- `src/views/settings/SettingsAdvanced.vue`：代理模式（不使用代理/系统代理/自定义代理）、代理类型（HTTP/HTTPS/SOCKS5）、CurseForge API Key 启用开关（已启用/未启用）三处 SegmentedButtons 改为 Select 平行布局；移除 SegmentedButtons 组件导入
- 整体规范：所有设置页选择器统一使用 32px 高的 Arco Design 风格 Select 组件，与左侧标题/描述平行展示，描述文案下移至同行下方

#### 游戏启动 Java 路径改用 Select 组件
- `src/views/settings/settings-launch/JavaPathSelector.vue`：移除原 watch + `handleDocumentClick` + `showJavaList` + `javaSelectorRef` 自定义下拉逻辑，改为复用项目自研 Select 组件（`customOption` prop + `#selected`/`#option`/`#empty` slot）；触发器展示版本徽章（自动/Java 17 等）+ 路径，下拉项展示主版本徽章 + 完整路径 + 完整版本号两行布局，未检测到 Java 时显示空状态提示；保留自动检测/手动导入按钮不变
- 修复原代码 bug：原实现使用了 `javaList` 中不存在的 `is_64bit` / `is_jre` 字段（实际类型为 `{ executable, version, major_version }`），新代码只展示真实存在的字段

#### 内存分配模式改用 Button 组件 + Tooltip 与 Select 下拉框协调优化
- `src/views/settings/settings-launch/MemoryAllocation.vue`：「分配模式」原两个原生 `<button>`（自动配置 / 自定义）改为复用项目自研 Button 组件（`type="primary"` 选中 / `type="outline"` 未选中，`size="small"` + `flex-1` 等宽），与「高阶配置 → 社区资源 → Mod 管理样式」按钮组风格统一
- `src/views/version-settings/MemorySection.vue`：「分配模式」原 `SegmentedButtons` 组件（跟随全局 / 自动配置 / 自定义）改为三个 Button 组件（同上风格），移除 `SegmentedButtons` 导入与 `modeButtons` 数组
- `src/components/common/Tooltip.vue`：新增与 Select 下拉框的方向协调——显示时若拟放置位置与任何 `.select-dropdown` 矩形重叠，且当前方向为 `top`/`bottom`，自动切换到反方向重算位置（反方向仍重叠则保留原方向）；新增 `MutationObserver` 监听 body 子节点变化（select-dropdown 通过 teleport 加入/移除 body），下拉框打开/关闭时实时重新计算 Tooltip 位置，避免下拉框向上展开时与 Tooltip 重叠导致无法框选选项；新增 `overlapsSelectDropdown` 矩形相交检测函数与 `calcByDirection` 方向化位置计算函数，原 `calcPosition` 重构为先按 `props.position` 计算、再视情况避让、最后边界修正的三阶段流程

#### 下载配置页下载源改用 Select 组件
- `src/views/settings/SettingsDownload.vue`：将"版本列表源"与"文件下载源"两处 SegmentedButtons 替换为项目自定义 Select 组件（32px 高、Arco Design 风格、下拉面板 4px 圆角 + 阴影），符合项目"复用自定义组件而非浏览器原生组件"的规范

#### 特别鸣谢 logo 圆形展示优化与重生头像修复
- `src-tauri/resources/about/acknowledgements.txt`：MC 百科作者重生补充头像文件名 `chongsheng.jpg`
- `src/views/settings/SettingsMore.vue`：项目 logo 容器从 12x12 放大到 14x14；背景从 `bg-gray-50` 改为 `bg-white` 与方形 logo 白边融合；图片样式从 `object-contain` 改为 `object-cover` 填满圆形容器，方形 logo 四角白边被圆形裁剪不再突兀

#### 特别鸣谢作者信息与头像展示
- `src-tauri/resources/about/acknowledgements.txt`：补充三位作者信息——BMCLAPI(bangbang93)、MC 百科(重生)、MCIM API(z0z0r4)；BMCLAPI 项目 logo 从 `bangbang93.png` 改为 `bmclapi-qun.png`；authors 字段格式升级为 `姓名:头像文件名`，支持作者头像展示
- `src-tauri/src/commands/system/about.rs`：新增 `Author` 结构体（`name` + `avatar: Option<String>`），`AcknowledgementItem.authors` 类型从 `Vec<String>` 改为 `Vec<Author>`；`parse_authors` 函数支持 `name:avatar` 格式解析，冒号可省略表示无头像
- `src/utils/api/about.ts`：新增 `Author` 接口，`AcknowledgementItem.authors` 类型从 `string[]` 改为 `Author[]`
- `src/views/settings/SettingsMore.vue`：特别鸣谢项目 logo 容器从方框（`rounded-lg`）改为圆形（`rounded-full`）+ 灰色环边；作者标签改为圆形头像 + 姓名 组合，有头像时显示图片，无头像时显示姓名首字圆形占位（primary 主题色背景）

#### 关于页面数据迁移至后端（markdown 表格 txt + IPC 命令）
- `src-tauri/resources/about/`：新建目录，存放 5 个 markdown 表格格式的 txt 数据文件：`acknowledgements.txt`（特别鸣谢，含 authors 字段）、`frontend-deps.txt`（前端运行时依赖）、`frontend-dev-deps.txt`（前端开发工具链）、`backend-deps.txt`（后端依赖）、`licenses.txt`（许可与版权声明）。修改数据只需更新 txt 文件并重新编译后端，无需改动业务代码
- `src-tauri/src/utils/markdown_table.rs`：新建通用 markdown 表格解析工具模块，支持注释行（`#`）、转义竖线（`\|`）、对齐分隔行（`:---:`）、缺失列填充、多余列忽略；含 7 个单元测试覆盖各种边界场景
- `src-tauri/src/utils/mod.rs`：新建顶级 utils 模块并注册 `markdown_table` 子模块
- `src-tauri/src/commands/system/about.rs`：新建 about 命令模块，定义 `AcknowledgementItem`/`DependencyItem`/`LicenseItem`/`AboutData` 数据结构，提供 `get_about_data` IPC 命令一次性返回关于页面所需的全部数据
- `src-tauri/src/commands/system/mod.rs`：注册 about 子模块并 pub use
- `src-tauri/src/resources.rs`：在 `embedded_text` 中注册 5 个 about txt 资源（include_str! 嵌入二进制）
- `src-tauri/src/lib.rs`：注册顶级 utils 模块；在 invoke_handler 中注册 `get_about_data` 命令
- `src/utils/api/about.ts`：新建前端 API 封装，定义 `AboutData`/`AcknowledgementItem`/`DependencyItem`/`LicenseItem` 类型与 `getAboutData()` 调用函数
- `src/views/settings/SettingsMore.vue`：移除全部硬编码的 `acknowledgements`/`frontendDeps`/`frontendDevDeps`/`backendDeps`/`licenses` 数组（约 250 行数据），改为 `onMounted` 异步调用 `getAboutData()` 加载；技术栈和许可列表区域增加加载中/加载失败状态；logo 改用 `import.meta.glob` 预加载 AboutIcon 目录构建 文件名→URL 映射表，根据后端返回的 logo 文件名动态解析；官网按钮文案从 `molaunch.moiu.cn` 改为 `点我前往`；特别鸣谢每项增加"作者"展开按钮（ChevronDownIcon 旋转 180°），展开后显示作者标签列表，作者为空时显示"暂未提供作者信息"，展开/收起附带 200ms 平滑过渡动画

#### 关于页面技术栈补全前端开发工具链
- `src/views/settings/SettingsMore.vue`：新增 `frontendDevDeps` 数组并拆出"前端开发工具"独立子组（与"前端"/"后端 (Rust)"并列展示），将原混入 `frontendDeps` 的 Vite / TypeScript 移至新组，并补充 9 个直接 devDependencies：vue-tsc、@vitejs/plugin-vue、Vitest、@vue/test-utils、ESLint、Prettier、PostCSS、Autoprefixer、@tauri-apps/cli；同步在"许可与版权声明"列表中追加 7 个新条目（Vue Language Tools、@vue/test-utils、Vitest、ESLint、Prettier、PostCSS、Autoprefixer），@vitejs/plugin-vue 与 @tauri-apps/cli 因分别跟随 Vite / Tauri 2 已有条目不再重复

#### 关于页面补充 Element Plus Icons 借用声明
- `src/views/settings/SettingsMore.vue`：在"鸣谢 → 法律信息 → 特别说明"中追加"关于 Element Plus Icons"段落，说明 Heroicons Vue 图标集不足时从 Element Plus Icons 提取 SVG path 写入 `src/utils/element-icons.ts` 复用、未引入运行时依赖、版权声明已添加；同时在"许可与版权声明"列表中追加 Element Plus Icons 条目（MIT License，含来源网站与许可文档链接）
- `src/utils/element-icons.ts`：补全顶部 MIT 许可证完整文本（替换原占位注释"MIT License full text will be added here"），明确标注 Copyright (c) 2021-present Element Plus Team 及完整权限与免责条款

#### 启动流程日志增强（定位启动后 16 秒空白期）
- `src-tauri/src/lib.rs`：在 Tauri Builder 构建前、`register_uri_scheme` 前后、`builder.run()` 前、`setup()` 钩子入口各补一条 `[Startup]` 日志，便于定位从 `CF enabled` 到首个 IPC 之间的耗时区间（涵盖 plugin 注册、URI scheme 注册、context 构建、webview/窗口创建、setup 钩子）
- `src-tauri/src/commands/sdk.rs`：`get_platform_info` / `get_sdk_version` / `get_device_id` 入口补 `[Startup][IPC]` 日志，定位前端首波 SDK 查询到达后端的时间点
- `src-tauri/src/commands/auth/account.rs`：`get_login_status` / `get_ms_accounts` / `get_offline_accounts` 入口补 `[Startup][IPC]` 日志，定位 `authStore.restoreSession()` 三次 IPC 到达时间点
- `src/App.vue`：`onMounted` 与 `initApp` 各阶段（detectJava 触发、fetchPlatformInfo+fetchDeviceId 完成、restoreSession 完成、initApp 完成）补 `[Startup][Frontend]` console.log 与 ISO 时间戳，配合后端日志可定位 16 秒空白期究竟花在 Tauri 框架启动、WebView 初始化还是前端 JS 加载
- 说明：日志显示 `[20:04:21.518] CF enabled=false` 到 `[20:04:37.211] Listing all Java runtimes` 之间约 15.7 秒空白，主要由 Tauri Builder 启动 + WebView 创建 + 前端 bundle 加载 + Vue 应用挂载占用；新增日志可逐一拆解各阶段耗时

#### 日志格式增强：等级加方括号 + 调用路径
- `src-tauri/src/logger.rs`：日志宏（`log_info!` / `log_warn!` / `log_error!` / `log_debug!` / `log_trace!`）改为传递 `file!()` + `line!()`，替代原 `module_path!()`；`Logger::log` 与公共 `logger::log` 签名改为 `(level, file, line, message)`；输出格式从 `[time] [LEVEL] message` 改为 `[time] [src/path.rs:line] [LEVEL] message`（等级置于调用路径之后），控制台同样输出路径段（灰色）；新增 `strip_to_src_relative` 辅助函数将 `file!()` 返回的路径统一裁剪到 `src/` 开头；`separator` 函数改用占位路径 `logger.rs:0`
- `src/main.ts`：前端入口补三条 `[Startup][Frontend]` console.log（main.ts 入口、Vue app 创建、mount 调用），配合后端 setup 钩子时间戳可精确定位 dev 模式启动 10 秒空白期究竟花在 WebView2 加载、JS bundle 解析还是 Vue 应用挂载
- 卡顿定位结论：日志显示 setup() hook 完成到首个 IPC 到达之间约 10 秒，由 Vite dev server 启动 + WebView2 加载 localhost:1420 + JS bundle 解析 + Vue 挂载占用；release 构建会快 3–5 倍

#### 插件子进程执行权限（spawnProcess）
- 后端 `src-tauri/src/commands/plugins/mod.rs`：
  - 新增 `ProcessPermissions` 结构（`allowed_commands` 命令白名单 / `timeout_ms` 单次超时默认 30s 最大 5min / `max_concurrent` 最大并发默认 1 最大 5）
  - `ExternalPluginManifest` 新增 `process_permissions: Option<ProcessPermissions>` 字段，仅当 `permissions` 含 `spawnProcess` 时生效
  - 新增 `ProcessResult` 结构（exit_code / stdout / stderr / timed_out / duration_ms）
  - 新增 `plugin_spawn_process` IPC 命令：权限校验 → 命令白名单匹配（canonicalize 后比对，Windows 忽略大小写与 `.exe` 后缀）→ `tokio::process::Command` 非 shell 执行（防注入）→ 超时控制（`tokio::time::timeout` 包裹 `child.wait()`，超时调用 `child.kill()`）→ stdout/stderr 管道异步读取各截断到 1MB（在 UTF-8 字符边界切割）
  - 新增辅助函数 `read_plugin_manifest` / `is_command_allowed` / `truncate_output`
- 后端 `src-tauri/src/lib.rs`：注册 `plugin_spawn_process` 命令
- 前端 `src/plugins/sdk.ts`：新增 `SpawnProcessOptions` / `ProcessResult` 类型，`PluginSdk` 接口新增 `spawnProcess()` 方法（内置插件实现直接抛错——内置插件有直接后端访问能力，不需要此方法；外部插件由 PluginSandbox 拦截处理）
- 前端 `src/plugins/sandbox/sandbox-bootstrap.ts`：`window.molaunch` 暴露 `spawnProcess(command, args, options)` 方法
- 前端 `src/plugins/sandbox/PluginSandbox.vue`：`handleMessage` 新增 `spawnProcess` 特殊桥接——权限校验通过后注入 `props.pluginId` 上下文（沙箱内 iframe 无 same-origin 无法获知自身 pluginId），直接 `invoke('plugin_spawn_process')` 调用后端命令

#### 缓存统计 SDK 暴露 + 示例插件展示
- 前端 `src/plugins/sdk.ts`：新增 `CacheStatEntry` / `CacheStatsResult` 类型，`PluginSdk` 接口新增 `getCacheStats()` 方法，`PluginSdkImpl` 实现 `getCacheStats()` 调用 `get_cache_stats` 命令；作为普通只读权限，内置与外部插件均可用
- 前端 `src/plugins/sandbox/sandbox-bootstrap.ts`：`window.molaunch` 暴露 `getCacheStats()` 方法
- 前端 `src/plugins/system-monitor/SystemMonitorPanel.vue` 完全重写：新增「缓存占用」卡片（总文件数 + 总大小 + 三分类明细：运行缓存 / 临时缓存 / AppData），手动刷新按钮（不轮询，避免 IPC 重复读取），`loadAll()` 现在并行加载 `getCacheStats()`

#### 缓存管理独立页面（普通用户可见）
- 前端新增 `src/views/settings/SettingsCache.vue`：缓存管理页面，普通用户可见（不需要开发者模式）
  - 顶部三卡片总览：总占用 / 可自动清理 / 重要资源
  - 详细列表：每个子目录含名称 + TTL 标识（黄色 24h 自动清理 / 灰色不清理）+ 类别 tag + 文件数 + 占用大小 + 路径 + 打开按钮
  - 数据来源：`tauri.getCacheStats()` IPC 命令
- 前端 `src/views/Settings.vue`：`baseCategories` 新增 `{ id: 'cache', label: '缓存管理', icon: CircleStackIcon }` 子菜单项，模板新增 `<SettingsCache v-else-if="activeCategory === 'cache'" />`
- 前端 `src/views/settings/SettingsDeveloper.vue` 完全重写：移除缓存统计相关代码（cacheStats / cacheStatsLoading / loadCacheStats / cacheStatsEntries / cacheTotalSize / cacheTotalFiles / ArrowPathIcon 导入），`onMounted` 改为只加载 `loadStorageDirs()` + `loadSystemInfo()`，模板移除「缓存统计」卡片保留「缓存目录」（仅展示父目录路径）。缓存统计迁移到独立页后普通用户可见，开发者页仅保留路径定位功能

#### 插件权限元信息表 + 插件管理页布局重写
- 前端新增 `src/plugins/permissions.ts`：权限元信息单一数据源
  - `PermissionMeta` 接口：name / description / useCase / risk（low/medium/high）/ alwaysAllowed / requiresExtraConfig
  - `PERMISSION_REGISTRY` 数组：10 项权限（emit / log 始终允许，getConfig / listInstalledVersions / listInstalledVersionsWithType / listLaunchHistory / getSystemMemory / getRunningGamePid / getCacheStats 7 项普通权限，spawnProcess 高级权限）
  - 导出 `ALWAYS_ALLOWED_PERMISSIONS` / `NORMAL_PERMISSIONS` / `ADVANCED_PERMISSIONS` / `RISK_STYLES`（风险等级 → 颜色样式映射）/ `getPermissionMeta()` 查询函数
- 前端 `src/views/settings/SettingsPlugins.vue` 完全重写：
  - 新增插件系统运行逻辑展示区（5 步流程图：扫描目录 → 解析清单 → 加载插件 → 权限校验 → 事件桥接，带箭头连接）
  - 权限 tag 现在带 Tooltip，悬停显示 `${description} — ${useCase}`
  - 高风险权限 tag 显示为红色 + 警告图标
  - 新增可展开/收起的「可用权限说明」区域，分三组：
    - 始终允许（灰色背景，emit / log）
    - 普通权限（蓝色背景，含风险等级 tag）
    - 高级权限（红色背景，含「需额外配置字段」提示，spawnProcess 需要 `processPermissions` 配置）
  - manifest.json 示例更新，包含 `processPermissions` 配置（allowed_commands / timeout_ms / max_concurrent）

#### 缓存监控内置插件 + 插件管理页布局优化
- 前端新增内置插件 `src/plugins/cache-monitor/`：缓存监控面板，专用于主页右侧展示缓存磁盘占用
  - `index.ts`：插件清单，id=`cache-monitor`，声明 homePanel 能力
  - `CacheMonitorPanel.vue`：顶部三卡片概览（总占用 / 文件总数 / 可自动清理大小）+ 按分类分组明细（运行缓存 / 临时缓存 / AppData），每个子目录显示名称 + TTL 标识（黄色 24h / 灰色不清理）+ 文件数 + 占用大小 + 完整路径，手动刷新按钮（不轮询，避免 IPC 重复读取）
  - 与 system-monitor 区分：system-monitor 综合展示内存/进程/SDK 状态，cache-monitor 专注缓存磁盘占用明细
- 前端 `src/plugins/index.ts`：注册 cache-monitor 内置插件（现共 5 个内置插件）
- 前端 `src/views/settings/SettingsPlugins.vue` 流程图布局优化：
  - 5 个步骤方框改为 `flex-1` 等宽 + `items-stretch` 等高布局，消除原 `min-w-[140px]` 导致的方框大小不一致问题
  - 箭头改为独立 flex 元素（`flex-none`），不参与伸缩，保证方框等宽
  - 容器加 `min-w-[760px]` + `overflow-x-auto`，窄屏可横向滚动
- 前端 `src/views/settings/SettingsPlugins.vue` 权限说明列表对齐优化：
  - 三组权限（始终允许 / 普通 / 高级）的每项布局从 `flex` 改为 `grid grid-cols-[180px_1fr]`，权限名占固定 180px 列宽
  - 所有描述文本左边缘对齐，不再因权限名长度不同导致描述参差不齐
  - 普通权限和高级权限的风险等级 tag 改为 `flex-wrap`，窄屏可换行不溢出

#### 默认模式右侧时钟卡片（HomeClockCard）
- 新增 `src/components/home/HomeClockCard.vue`：默认模式下主页右侧渲染时钟卡片
  - 顶部固定显示大时钟（HH:MM + 秒数 primary 色高亮）+ 日期 + 星期
  - 底部轮播信息卡片，每 6 秒自动翻页切换：
    - 内存使用（带进度条，≥80% 红色 / ≥60% 黄色 / 其他绿色）
    - 已安装版本数（含最近版本 ID）
    - 最近一次启动（版本 ID + 时间 + 退出状态）
    - 缓存占用（总大小 + 文件数）
  - 数据源通过 `Promise.all` 并行加载，单个失败跳过不阻塞轮播
  - 翻页动画：`translateY ±12px` + `opacity` 淡入淡出（0.4s）
  - 底部指示点支持点击切换，切换后重新计时
- 修改 `src/views/Home.vue`：
  - 新增 `showLaunchProgress` 状态（含 600ms 延迟隐藏），启动中渲染 LaunchLog，结束后切换到时钟卡片/插件/自定义布局
  - `homePanelComponent` computed 默认模式回退从 LaunchLog 改为 HomeClockCard
  - 插件未找到/未启用时回退到 HomeClockCard 而非 LaunchLog

#### customLayoutConfig 配置读写重构
- 移除 `customLayoutConfig` 后端 INI 持久化逻辑：
  - `src/stores/plugins.ts`：删除 `persistCustomLayoutConfig` 方法，`setCustomLayoutConfig` / `refreshCustomLayoutCache` 仅保存到 localStorage
  - `syncFromBackend` 不再从后端 INI 读取 `customLayoutConfig`，仅从前端 localStorage 恢复
  - URL 来源的 cachedContent 通过独立的 `load_custom_layout` 命令单独获取（命中本地缓存文件 `.Molaunch/cache/custom_layout/<sha256>.txt`）
- 理由：`customLayoutConfig` 包含 `cachedContent` 大字段不适合存 INI；URL 内容已有独立缓存文件，无需重复持久化
- 移除 `src-tauri/src/commands/system/config.rs` 中 `("Plugin", "customLayoutConfig")` 白名单项（不再需要）

#### 主页右侧内容区自定义布局模式（JSON / HTML / XML）
- 新增 `src/plugins/custom-layout/` 自定义布局引擎模块，支持三种格式：
  - **JSON**：结构化布局，启动器提供组件库（stat-grid 统计网格 / list 数据列表 / progress 进度条 / text 文本块 / divider 分割线），用户配置页面信息，支持 `{{dataSource.field}}` 值表达式插值
  - **XML**：结构化布局，使用浏览器内置 `DOMParser` 解析，解析后统一转为 `LayoutSchema` 复用 JSON 渲染组件
  - **HTML**：直接渲染 HTML，复用 `sandbox-bootstrap.ts` 的 `buildSandboxHtml` 注入 `window.molaunch` SDK，通过 `<iframe sandbox="allow-scripts">` 加载，与 PluginSandbox 区别为无权限白名单（用户自定义内容）但禁用 spawnProcess
- 新增 `src/plugins/custom-layout/types.ts`：自定义布局 Schema 类型定义（`LayoutSection` 联合类型、`StatItem`、`ListField`、`LayoutSchema`、`ParseResult`）
- 新增 `src/plugins/custom-layout/parser.ts`：JSON 和 XML 解析器，统一输出 `LayoutSchema`，包含校验逻辑（VALID_SECTION_TYPES / VALID_FORMATS / VALID_COLORS / VALID_VARIANTS）
- 新增 `src/plugins/custom-layout/datasource.ts`：数据源加载与值解析，`loadDataContext()` 通过 `Promise.allSettled` 并行获取 cache/system/versions/history 数据，单个失败不阻塞其他；`resolveValue()` 解析 `{{key}}` 插值；`formatValue()` 支持 bytes/number/percent/text 格式化
- 新增 `src/plugins/custom-layout/CustomLayoutPanel.vue`：JSON/XML 结构化布局渲染器，每 3 秒轮询数据源（不重新解析布局），支持图标映射（chart-bar / circle-stack / cpu-chip / clock）
- 新增 `src/plugins/custom-layout/HtmlLayoutPanel.vue`：HTML 直接渲染组件，`BLOCKED_METHODS = new Set(['spawnProcess'])`
- 新增 `src/plugins/custom-layout/index.vue`：自定义布局入口，根据 format 分发到 CustomLayoutPanel 或 HtmlLayoutPanel
- 扩展 `src/types/plugin.ts`：
  - `HomePanelMode` 类型扩展为 `'default' | \`plugin:${string}\` | 'custom'`
  - 新增 `LayoutFormat = 'json' | 'html' | 'xml'` / `LayoutSource = 'inline' | 'url'`
  - 新增 `CustomLayoutConfig` 接口（format / source / inlineContent / url / cachedContent / cachedAt）
- 扩展 `src/stores/plugins.ts`：
  - 新增 `DEFAULT_CUSTOM_LAYOUT` 常量与 `customLayoutConfig` ref
  - `loadFromStorage` / `saveToStorage` 支持 customLayoutConfig
  - 新增 `setCustomLayoutConfig(partial)` / `persistCustomLayoutConfig()` / `refreshCustomLayoutCache()` 方法
  - `syncFromBackend` 读取后端 `customLayoutConfig` JSON 字符串解析；若 source=url 且无缓存自动调用 `load_custom_layout` 加载
  - `refreshCustomLayoutCache()` 传入 `forceRefresh: true` 强制忽略本地缓存重新下载
- 重写 `src/views/settings/SettingsPersonal.vue` 主页配置区：
  - 顶层模式选择：默认 / 插件模式 / 自定义模式，通过 `panelMode` computed 派生
  - 插件模式：条件渲染插件选择 Select（仅显示已启用且提供 homePanel 的插件）
  - 自定义模式：条件渲染格式选择（JSON/HTML/XML）+ 来源选择（内联/URL）+ 内联编辑器（textarea 防抖 500ms 同步）/ URL 加载（含刷新按钮）
  - 内联编辑器占位文本移至 `inlinePlaceholder` computed，避免模板内联 JS 表达式中转义引号导致 Vue 编译器解析失败
  - 显示缓存时间 `cachedTimeText`
- 修改 `src/views/Home.vue`：
  - `homePanelComponent` computed 新增 `if (mode === 'custom') return CustomLayout` 分支
  - 新增 `homePanelProps` computed：custom 模式返回 `{ config: pluginStore.customLayoutConfig }`，其他模式返回 `{}`
  - 模板 `<component :is="homePanelComponent" v-bind="homePanelProps" />` 支持 props 传递

#### 缓存监控与缓存设置页面布局优化（顶部固定 + 底部滑动）
- 修改 `src/plugins/cache-monitor/CacheMonitorPanel.vue`：
  - 标题栏改为 `flex flex-none`
  - 概览卡片改为 `flex-none grid grid-cols-3 gap-3 mb-4`
  - 分类明细改为 `flex-1 space-y-4 overflow-y-auto pr-1`
  - 实现「顶部固定 + 底部滑动」布局
- 重写 `src/views/settings/SettingsCache.vue` 模板结构：
  - 外层改为 `flex h-full flex-col gap-4 p-6`
  - Alert + 总览卡片组 `flex-none grid grid-cols-3 gap-4`
  - 详细列表区域 `flex-1 min-h-0 ... flex flex-col`，内部列表 `flex-1 overflow-y-auto`
- 修改 `src/views/Settings.vue`：容器 div 的 class 改为动态绑定，cache 页面 `!p-0` 去掉外层 padding 让子组件自管理滚动

#### 自定义布局 URL 加载后端命令（load_custom_layout）
- 后端新增 `load_custom_layout` IPC 命令（`src-tauri/src/commands/plugins/mod.rs`）：
  - 接收 `url: String` 和可选 `force_refresh: Option<bool>` 参数
  - URL 协议校验（仅允许 http/https，拒绝 file://、data: 等）
  - 缓存文件路径使用 URL 的 sha256 哈希（64 字符十六进制），避免文件名冲突和路径注入
  - 缓存位置：`.Molaunch/cache/custom_layout/<sha256>.txt`
  - 非强制刷新时优先读取本地缓存，缓存不存在或读取失败时发起 HTTP 请求
  - 响应大小上限 5MB（`MAX_CUSTOM_LAYOUT_BYTES`），超过则报错
  - 响应必须为合法 UTF-8 文本，否则报错
  - 写入缓存失败不阻塞返回内容（仅 warn 日志）
  - 返回布局内容文本字符串，前端直接写入 `customLayoutConfig.cachedContent`
- `src-tauri/src/commands/plugins/mod.rs`：新增 `use sha2::{Digest, Sha256}` 导入和 `hash_url()` 辅助函数
- `src-tauri/src/lib.rs`：注册 `commands::plugins::load_custom_layout` 命令
- `src-tauri/src/commands/system/config.rs`：`is_valid_config_key` 白名单新增 `("Plugin", "customLayoutConfig")` 项，支持自定义布局配置通过 `set_config_value` 持久化到 INI 文件
- `src-tauri/src/utils/cache_cleanup.rs`：清理范围新增 `.Molaunch/cache/custom_layout/`（24h TTL），与 images/forge_installer/preload_mods/launch 一致
- `src-tauri/src/utils/cache_stats.rs`：统计范围新增 `custom_layout` 子目录（24h TTL），供缓存管理页面展示

#### 插件创建子窗口权限（createWindow）
- 后端新增 `plugin_create_window` IPC 命令（`src-tauri/src/commands/plugins/mod.rs`）：
  - 接收 `plugin_id` / `label` / `url` / `title` 参数，创建独立 WebviewWindow
  - 权限校验：manifest 必须声明 `createWindow` 权限 + `window_permissions` 配置
  - 域名白名单：URL 域名必须匹配 `window_permissions.allowed_domains`（支持 `*.` 通配符，如 `*.github.io`）
  - URL 协议校验：仅允许 http/https，拒绝 file://、data: 等
  - 窗口数量限制：每个插件最多 5 个窗口（`MAX_PLUGIN_WINDOWS`），label 格式 `plugin-<id>-<label>` 避免与内置窗口冲突
  - 新增 `WindowPermissions` 结构体（allowed_domains / width / height / resizable，含 serde 默认值函数）
  - 新增 `extract_domain` / `is_domain_allowed` 辅助函数（简单字符串解析，不依赖 url crate）
  - `ExternalPluginManifest` 新增 `window_permissions: Option<WindowPermissions>` 字段
- 后端 `src-tauri/src/lib.rs`：注册 `commands::plugins::plugin_create_window` 命令
- 前端 SDK 扩展 `src/plugins/sdk.ts`：
  - 新增 `CreateWindowOptions` 接口（label / url / title）
  - `PluginSdk` 接口新增 `createWindow(options: CreateWindowOptions): Promise<void>`
  - `PluginSdkImpl` 新增 `createWindow` 实现，内置插件调用直接抛错（内置插件有直接后端访问能力，不需开窗口）
- 前端权限注册表 `src/plugins/permissions.ts`：`PERMISSION_REGISTRY` 新增 `createWindow` 权限项（high risk, requiresExtraConfig: 'windowPermissions'）
- 前端沙箱引导 `src/plugins/sandbox/sandbox-bootstrap.ts`：`window.molaunch` 新增 `createWindow` 方法转发
- 前端沙箱代理 `src/plugins/sandbox/PluginSandbox.vue`：新增 `createWindow` 消息处理（类似 `spawnProcess`，注入 pluginId 调用后端 `plugin_create_window`）
- 前端 HTML 布局面板 `src/plugins/custom-layout/HtmlLayoutPanel.vue`：`BLOCKED_METHODS` 新增 `'createWindow'`（自定义 HTML 布局不允许创建窗口）

#### 通用文本文件写入命令（write_text_file）
- 后端新增 `write_text_file` IPC 命令（`src-tauri/src/commands/system/game_dir.rs`）：接收 `path` / `content` 参数，自动创建父目录后写入文本文件，用于前端导出示例文件场景
- 后端 `src-tauri/src/lib.rs`：注册 `commands::system::write_text_file` 命令

#### JSON/XML 布局新增 html section 类型（支持内联 JS/CSS）
- 扩展 `src/plugins/custom-layout/types.ts`：`LayoutSection` 联合类型新增 `html` 类型，包含 `content`（HTML 内容）/ `script`（内联 JS）/ `style`（内联 CSS）/ `height`（iframe 高度，默认 200）字段
- 扩展 `src/plugins/custom-layout/parser.ts`：
  - `VALID_SECTION_TYPES` 新增 `'html'`
  - JSON 解析：新增 `parseHtmlJson` 函数，校验 content 必须为字符串，script/style/height 可选
  - XML 解析：新增 `parseHtmlXml` 函数，要求通过 `<content>` 子节点提供 HTML 内容（避免与 `<script>`/`<style>` 文本混淆），`<script>`/`<style>` 子节点提供 JS/CSS，`height` 属性设置 iframe 高度
  - 文档注释更新：补充 JSON 和 XML 格式的 html section 示例
- 扩展 `src/plugins/custom-layout/CustomLayoutPanel.vue`：
  - 新增 `buildHtmlSrcDoc` 函数：将 content + style + script 组装为完整 HTML 文档字符串
  - 模板新增 `html` section 渲染分支：通过 `<iframe sandbox="allow-scripts" :srcdoc="...">` 渲染（不含 `allow-same-origin`，iframe 运行在 null origin，无法访问父窗口 DOM/cookie/localStorage）

#### 示例文件导出功能（插件页面 + 个性化自定义模式）
- 示例文件存储于 `src-tauri/resources/samples/` 目录，通过 `include_str!` 嵌入二进制，前端通过 IPC 命令读取，不再硬编码在前端代码中：
  - `samples/plugin/manifest.json`：插件示例清单（含全部可选权限配置字段）
  - `samples/plugin/index.html`：插件示例入口（演示 SDK 调用 getConfig / getCacheStats）
  - `samples/layout/layout-sample.json`：JSON 布局示例（含全部 section 类型，含 html section）
  - `samples/layout/layout-sample.xml`：XML 布局示例（含 html section，CDATA 包裹 HTML 内容）
  - `samples/layout/layout-sample.html`：HTML 布局示例（通过 window.molaunch 调用 SDK 加载数据，每 3 秒刷新）
- 后端新增两个 IPC 命令（`src-tauri/src/commands/plugins/mod.rs`）：
  - `read_layout_sample(format)`：根据格式从嵌入资源读取示例布局内容
  - `export_plugin_sample(dest_path, as_zip)`：导出插件示例模板，支持文件夹（直接写入 manifest.json + index.html）和 ZIP（使用 zip crate 打包）两种方式
- 后端 `src-tauri/src/resources.rs`：`embedded_text` 注册 5 个示例文件路径
- 后端 `src-tauri/src/lib.rs`：注册 `read_layout_sample` 和 `export_plugin_sample` 命令
- 插件管理页 `src/views/settings/SettingsPlugins.vue`：
  - 移除前端硬编码的 SAMPLE_MANIFEST / SAMPLE_INDEX_HTML 常量
  - 导出按钮改为卡片样式（虚线边框，与个性化页一致），支持「文件夹」和「ZIP 文件」双选
  - 文件夹导出调用 `selectFolder` + `export_plugin_sample(asZip=false)`
  - ZIP 导出调用 `saveFile` + `export_plugin_sample(asZip=true)`，后端使用 zip crate 现场打包
- 个性化页 `src/views/settings/SettingsPersonal.vue`：
  - 移除前端硬编码的 SAMPLE_JSON / SAMPLE_XML / SAMPLE_HTML 常量
  - `onExportSampleLayout` 改为调用 `read_layout_sample` 从后端获取示例内容，再通过 `saveFile` + `write_text_file` 写入

### 修复

#### 沙箱 iframe 中 Tauri 内部脚本崩溃 + bootstrap 注入顺序
- `src/plugins/sandbox/sandbox-bootstrap.ts` 的 `buildSandboxHtml`：
  - 注入 `window.__TAURI_INTERNALS__` 桩（`{ plugins: {}, invoke: ... }`），防止 Tauri 2 的内部 IPC 初始化脚本在 `sandbox="allow-scripts"`（无 `allow-same-origin`）的 iframe 中因 `window.__TAURI_INTERNALS__` 为 undefined 而抛出 `Cannot read properties of undefined (reading 'plugins')` 错误
  - bootstrap 注入位置从 `</body>` 前改为 `<head>` 开头，确保 `window.molaunch` 在用户内联脚本执行前已定义（此前用户 HTML 中的 `<script>` 在 bootstrap 之前执行，导致 `window.molaunch` 为 undefined）
- `src/plugins/custom-layout/CustomLayoutPanel.vue` 和 `HtmlLayoutPanel.vue`：
  - iframe sandbox 从 `allow-scripts` 改为 `allow-scripts allow-same-origin`，使 Tauri 2 的 IPC 桥接脚本在所有 frame 中正常初始化（Tauri 2 通过 WebView2 的 `AddScriptToExecuteOnDocumentCreated` 在 `document_start` 向所有 frame 注入初始化脚本，无 `allow-same-origin` 时 IPC 不可用导致 `__TAURI_INTERNALS__` 为 undefined，后续脚本访问 `.plugins` 崩溃）
  - 移除 html section 的 `__TAURI_INTERNALS__` 桩（`allow-same-origin` 后 Tauri 自行初始化，桩不再需要）

#### progress section 新增 format 字段 + html section 内置设计系统 CSS
- `src/plugins/custom-layout/types.ts`：progress section 联合类型新增 `format?: ValueFormat` 字段
- `src/plugins/custom-layout/parser.ts`：`parseProgressJson` 和 `parseProgressXml` 均支持读取 `format` 属性
- `src/plugins/custom-layout/CustomLayoutPanel.vue`：
  - 进度条值显示使用 `section.format || 'text'` 进行格式化（此前硬编码 `'text'`，导致 bytes 值显示为原始数字）
  - 无 label 时也显示当前值/最大值（右侧对齐）
  - `buildHtmlSrcDoc` 注入内置设计系统 CSS（`DESIGN_SYSTEM_CSS`），提供与启动器主界面一致的视觉风格
  - 可用类名：`.btn` / `.btn-primary` / `.btn-sm` / `.card` / `.card-title` / `.stat` / `.stat-label` / `.stat-value` / `.grid` / `.grid-2` / `.grid-3` / `.progress-bar` / `.progress-fill` / `.badge` / `.badge-primary` / `.badge-green` / `.badge-red` / `.badge-gray` / `.text-muted` / `.text-sm` / `.text-lg` / `.text-bold` / `.flex` / `.items-center` / `.justify-between` / `.gap-2` / `.gap-4` / `.mt-2` / `.mt-4` / `.mb-2` / `.mb-4`

#### 示例布局字段名修正 + format 字段 + 设计系统类名演示
- `src-tauri/resources/samples/layout/layout-sample.json` / `layout-sample.xml` / `layout-sample.html`：
  - 修正字段名以匹配 `datasource.ts` 实际实现：`versions.installedCount` → `versions.count`、`system.memoryUsagePercent` → `system.usagePercent`
  - progress section 新增 `format: "bytes"`，内存使用值正确格式化为 `8.0 GB` 等
  - html section content 改用内置设计系统类名（`.card` / `.btn-primary` / `.badge` / `.progress-bar` 等）演示可用组件
  - HTML 示例也改用设计系统类名（`.grid grid-3` / `.card` / `.stat-value` / `.progress-bar` / `.progress-fill`）

#### 插件管理页导出区域布局优化
- `src/views/settings/SettingsPlugins.vue`：导出插件示例模板卡片移至 manifest.json 示例上方，两者之间添加 `mt-4` 间距

#### html section 内置前端组件 API（替代 alert/confirm/prompt）
- `src/plugins/custom-layout/CustomLayoutPanel.vue`：
  - `buildHtmlSrcDoc` 注入 `UI_API_SCRIPT`，提供 `window.molaunch.toast(type, text)` / `window.molaunch.alert(title, msg)` / `window.molaunch.confirm(title, msg)` / `window.molaunch.prompt(title, msg, default)` 四个 UI 组件 API
  - iframe 内通过 postMessage 调用父窗口的前端组件（Toast / Modal），不再使用浏览器原生 alert/confirm/prompt
  - 新增 `handleUiRequest` 消息处理器，onMounted 时注册、onUnmounted 时移除
  - toast 类型支持 info / success / error / warning，对应启动器 Toast 组件
  - confirm / prompt 返回 Promise，支持异步等待用户操作
- `src-tauri/resources/samples/layout/layout-sample.json` 和 `layout-sample.xml`：html section 按钮从 `onclick="alert(...)"` 改为 `onclick="window.molaunch.toast('success', ...)"`

#### 个性化配置常驻化存储到 AppData（全系统共享）
- 后端 `src-tauri/src/commands/plugins/mod.rs`：新增 `read_personalization` / `write_personalization` IPC 命令
  - 存储路径：`%APPDATA%/.MolaLaunch/personalization.json`
  - 独立于游戏目录（game_dir），确保不同 game_dir 的启动器实例加载同一份配置
  - JSON 格式存储 enabledMap / homePanelMode / customLayoutConfig 全部个性化数据
  - `personalization_path()` 辅助函数处理路径解析和目录创建
- 后端 `src-tauri/src/lib.rs`：注册 `read_personalization` / `write_personalization` 命令
- 后端 `src-tauri/src/commands/system/config.rs`：移除 [Plugin] 段 INI 白名单（`homePanelMode` 和 `enabled_<id>`），不再通过 INI 存储插件配置
- 前端 `src/stores/plugins.ts`：
  - 移除 `STORAGE_KEY` 常量、`loadFromStorage()` / `saveToStorage()` / `persistPluginEnabled()` / `persistHomePanelMode()` 方法
  - 新增 `persistToBackend()` 方法：全量收集 enabledMap + homePanelMode + customLayoutConfig，调用 `write_personalization` 写入 AppData
  - `syncFromBackend()` 从 `read_personalization` 命令读取配置（替代原 `get_config_value` 逐键读取 INI）
  - `setPluginEnabled` / `setHomePanelMode` / `setCustomLayoutConfig` / `refreshCustomLayoutCache` / `uninstallExternal` 均改为调用 `persistToBackend()`
  - store 初始化仅调用 `initRuntimeStates()`（不再从 localStorage 读取），实际启用状态由 `syncFromBackend` 异步加载

#### 插件安装按钮状态泄漏
- `src/views/settings/SettingsPlugins.vue`：原单个 `installing` ref 被文件夹和 ZIP 两个按钮共用，点击「文件夹」时「ZIP 文件」按钮也显示「安装中」。修复为拆分 `installingFolder` / `installingZip` 独立 ref + `installingAny` computed 互斥（点击一个按钮时禁用另一个，但仅被点击的按钮显示 loading 态）

#### 移除 community/install/mod.rs 未使用的 Emitter 导入
- `src-tauri/src/commands/community/install/mod.rs`：`use tauri::{AppHandle, Emitter, State};` 改为 `use tauri::{AppHandle, State};`，修复 `warning: unused import: Emitter`

#### 整合包下载面板不显示名字
- `src-tauri/src/commands/community/install/mod.rs`：`install_modpack` 中 `reset_stages` 后补充 `ds.version_name = req.instance_name.clone()`，修复前端从后端恢复下载状态时拿到空字符串覆盖正确名字的问题（普通版本下载无此问题）

#### 整合包下载暂停按钮无效
- `src-tauri/src/commands/community/install/modpack_stages.rs`：整合包原始包下载的 `DownloadManager` 补充 `.with_cancel_flag()` 和 `.with_pause_flag()`
- `src-tauri/src/commands/community/install/concurrent.rs`：整合包 mods 下载的 `DownloadManager` 补充 `.with_cancel_flag()` 和 `.with_pause_flag()`
- `src-tauri/src/commands/community/install/mod.rs`：`install_modpack` 开头重置 `download_cancel_flag` 和 `download_pause_flag`，防止上次残留导致新下载卡住

#### 社区资源下载走 DownloadManager + 下载管理页面展示
- `src-tauri/src/commands/community/install/mod.rs`：`download_resource` 和 `download_resource_to_path` 两个命令全部从单流 reqwest 直连改为走 `DownloadManager`（支持多 URL fallback + 分片 + 暂停/取消），进度通过 `download_state` 统一通道写入，前端在下载管理页面展示
- `src/components/community/ResourceDetail.vue`：移除 `useCommunityDownload` 和 `DownloadProgressOverlay`，改用 `versionStore.startDownload` 接入统一下载状态；下载开始时 `toastInfo` 提示
- `src/views/version-settings/mod-tab/ModUpdateDialog.vue`：Mod 更新下载也接入 `versionStore.startDownload`，下载开始时 `toastInfo` 提示
- `src/composables/useDownloadPolling.ts`：下载完成时 `toastSuccess` 提示，下载失败时 `toastError` 提示
- 所有社区资源下载完成后右下角 `DownloadPanel` 浮动按钮自动显示进度环，点击可进入下载管理页面查看详情

#### CurseForge + Modrinth CDN 下载镜像替换（统一由来源策略控制）
- `src-tauri/src/minecraft/sources.rs`：重写 CDN 替换为统一函数，同时支持 CurseForge（`edge.forgecdn.net` 等）和 Modrinth（`cdn.modrinth.com`）域名替换为 `mod.mcimirror.top`；由现有 `community_source` 配置控制：source=0 直接用镜像、source=1 官方+镜像双 URL fallback、source=2 用官方
- `src-tauri/src/minecraft/community/curseforge/convert.rs`：撤销源头 CDN 替换，改为在使用时替换
- `src-tauri/src/commands/community/install/helpers.rs`：`construct_cf_edge_url` fallback 构造根据 source 策略选择域名
- `src-tauri/src/commands/community/install/modpack_stages.rs`：整合包下载使用统一 `cdn_urls()` 构造 URL 列表
- `src-tauri/src/commands/community/install/curseforge.rs`：CF mods 下载使用统一 `cdn_urls()`
- `src-tauri/src/commands/community/install/modrinth.rs`：MR mods 下载对每个 URL 应用 `cdn_urls()` 扩展镜像 fallback
- `src-tauri/src/commands/community/install/mod.rs`：`download_resource` 和 `download_resource_to_path` 使用统一 `replace_cdn()`
- `src-tauri/src/minecraft/loaders/fabric_api.rs`：移除硬编码的 MCIM 镜像替换，复用统一 `cdn_urls()` 函数
- 移除上一版新增的 `community_cf_cdn_mirror` 独立配置项（不再需要，统一由来源策略控制）

#### CurseForge 批量查询绕过 source 策略
- `src-tauri/src/minecraft/community/curseforge/mod.rs`：`http` 模块改为 `pub(crate)` 暴露 `cf_post`
- `src-tauri/src/commands/community/install/curseforge.rs`：`install_cf_mods` 中的 `/mods/files` 批量查询从硬编码官方 API 改为 `cf_post`，走 source 策略（source=0 强制镜像，source=1 回退，source=2 官方）

#### 返回顶部按钮临界点闪烁 + 弹层误触发 + 路由残留
- `src/components/common/BackToTop.vue`：
  - 引入迟滞阈值：未显示时需 `scrollTop > 700` 才出现，已显示时仅在 `scrollTop ≤ clientHeight`（一屏高度）时隐藏，避免在临界点反复闪烁
  - 过滤逻辑从遍历祖先检查 `getComputedStyle().position`（开销大且误杀正常容器）改为 `el.closest('main')` 单次查询，仅响应主内容区内的滚动容器，Teleport 弹层 / 下拉框自动被过滤
  - 新增路由切换刷新：监听 `router.afterEach` 重置 `visible` 与 `activeEl`，并在 fade 过渡结束后（450ms 延迟）扫描新页面已滚动容器恢复按钮状态
  - `scrollToTop` 使用 `activeEl` 前增加 `isConnected` 检查，避免指向已卸载元素导致报错
  - 移除原 `scrollHeight ≤ clientHeight * 1.5 || scrollHeight < 600` 内容长度判断（与基于绝对像素的迟滞阈值重复）
- `src/components/common/DownloadPanel.vue`：浮动下载按钮位置从 `bottom-6`（24px）上移至 `bottom-20`（80px），与 `BackToTop`（bottom: 24px / 高度 44px）垂直错开 12px 间隙，避免两者同时可见时重叠

#### 设置 - 更多页面无法下滑
- `src/views/Settings.vue`：`about` 分类容器原先同时缺少 `p-6` 与 `overflow-y-auto`（仅 `overflow-hidden`），但 `SettingsMore` 子组件仅自带 `p-6` 内边距、并不自管理滚动，导致内容超出时无法下滑。修复为：仅 `cache` 分类由子组件自管理滚动并被排除 `overflow-y-auto`，其余分类（含 `about`）统一由外部容器提供纵向滚动

#### 插件页权限说明折叠区改用公共 CollapsibleCard 组件
- `src/views/settings/SettingsPlugins.vue`：「可用权限说明」面板原先自行用 `ref(false) + v-if` 实现展开/折叠，**无展开动画**且代码冗余。改造为使用项目已有的 `CollapsibleCard` 公共组件，获得 `grid-template-rows: 0fr→1fr` 平滑高度过渡动画，并移除 `permissionsExpanded` ref 与手写 SVG 箭头（改用 `ChevronDownIcon`），减少约 20 行代码

#### 首页加载慢：版本显示与开始游戏按钮需等待数秒
- `src/views/Home.vue` onMounted 改造为三阶段并行加载：
  - **阶段1**（首屏快速显示）：并行执行 `restoreSession` + `detectJava` + 新增的 `restoreSelectedVersionFast`，后者仅一次 IPC 读 config（约 1ms）即乐观设置 `selectedVersion`，用户立刻看到版本名 + 开始游戏按钮变蓝，无需等待磁盘扫描或网络请求
  - **阶段2**（后台并行）：`checkRunningGame` 与 `listInstalledVersionsWithType` 两个互不依赖的操作改为 `Promise.all` 并行（原先串行 4 个 await）
  - **阶段3**（校验刷新）：磁盘扫描完成后调用 `validateSelectedVersion` 校验版本是否仍存在（不存在则清空持久化并回退到第一个已安装版本），并刷新版本类型映射缓存
- **移除 `fetchVersions` 阻塞**：首页 onMounted 原先 `await versionStore.fetchVersions()` 拉取 Mojang 完整版本清单（1~3s 网络请求），但首页的 `VersionSelector` 与 `LaunchPanel` 根本不使用 `versions` 数组（仅版本下载页 `Versions.vue` 使用）。该调用已移除，由 `Versions.vue` 与 `useVersionInstallActions.ts` 在进入下载页时按需 lazy-load（已有 `if (versions.length === 0)` 去重 guard）
- `src/stores/version.ts` 新增两个方法：
  - `restoreSelectedVersionFast()`：仅读 config 不校验，用于首屏快速显示
  - `validateSelectedVersion(installedList)`：校验当前 selectedVersion 是否仍存在，不存在则清空持久化并自动回退到第一个已安装版本
- `src/stores/sdk.ts` `fetchPlatformInfo` 新增 `if (initialized.value) return` guard，避免 `App.vue` 与 `Home.vue` 重复发起 `get_platform_info` + `get_sdk_version` 两个 IPC（与 `javaStore.detectJava` 的 `javaLoaded` guard 模式一致）
- `src/views/Home.vue`：移除未使用的 `sdkStore` import 与声明（fetchPlatformInfo 已由 App.vue 触发，Home.vue 不再调用）

#### 版本选择页样式对齐 Settings 规范
- `src/views/VersionSelect.vue`：
  - 根容器从 `flex h-full` 改为 `flex h-full rounded-xl overflow-hidden bg-white shadow-sm`（与 Settings 根容器一致，提供圆角白底卡片化外观）
  - 顶部栏 padding 从 `px-4 py-3` 改为 `px-6 py-4`，标题从 `<h1 class="text-base font-semibold text-gray-800">` 改为 `<h2 class="text-lg font-semibold text-gray-900">`
  - 主体内边距从 `p-4` 改为 `p-6`
  - 版本分组卡片完全对齐 Settings 卡片规范：`rounded-xl border-gray-200` → `rounded-lg border-gray-300`、卡片头从 `bg-gray-50/60 px-4 py-2.5` 改为 `px-5 pt-5 pb-3`（去灰底）、卡片头标题从 `text-gray-700` 改为 `text-gray-900`、列表行从 `px-4 py-3` 改为 `px-5 py-4`、卡片间距从 `space-y-4` 改为 `space-y-6`
  - 列表项 hover 从 `hover:bg-primary-50/40`（40% 透明度）改为 `hover:bg-gray-50`（满色，与 Settings 一致）
  - 选中态打勾图标颜色从 `text-primary-600` 改为 `text-primary-500`（与主色变量一致）
  - 空状态卡片圆角从 `rounded-2xl` 改为 `rounded-lg`
  - 所有内联 SVG 图标替换为 Heroicons：返回（`ArrowLeftIcon`）、刷新（`ArrowPathIcon`）、选中打勾（`CheckIcon`）、下载（`ArrowDownTrayIcon`）、空状态（`ArchiveBoxIcon`）
- `src/views/version-select/FolderSidebar.vue`：
  - 侧边栏宽度从 `w-64`（256px）改为 `w-48`（192px），与 Settings 侧边栏一致
  - 滚动区内边距从 `px-3 pt-5` 改为 `py-4`（按钮自带 `px-4`，与 Settings 一致）
  - 选中态高亮条从左侧绝对定位（`absolute left-0 h-5 w-0.5 bg-primary-500`）改为右侧 border（`border-r-2 border-primary-500`），与 Settings 侧边栏一致
  - 选中态背景从 `bg-primary-50/70`（70% 透明度）改为 `bg-primary-50`（满色）
  - 文件夹按钮 padding 从 `pl-3 pr-2 py-2.5` 改为 `px-4 py-2.5`
  - 文件夹图标从内联 SVG（`h-4 w-4 mr-2.5`）替换为 Heroicons `FolderIcon`（`w-5 h-5 mr-3`），与 Settings 侧边栏图标规范一致
  - 移除按钮图标替换为 `XMarkIcon`，添加按钮图标替换为 `PlusIcon`
  - 移除"文件夹列表"和"添加或导入"两个分组小标题（Settings 侧边栏为扁平列表无分组标题），改为用一条 `border-t border-gray-100` 分隔线区分文件夹列表与添加按钮

#### 个性化布局编辑器改用公共 Input 组件
- `src/views/settings/SettingsPersonal.vue`：JSON/XML/HTML 内容编辑器原先使用原生 `<textarea>` 元素（带 `font-mono text-xs` 等宽字体小字号样式），违反项目"必须复用公共组件"规范。改为使用 `Input.vue` 公共组件的 textarea 模式（`<Input textarea :rows="16" resize="vertical">`），并通过 scoped `:deep(.textarea-inner)` 注入等宽字体与 12px 字号，保持与原原生 textarea 一致的代码输入体验

#### 新增工具菜单 + 外部下载工具
- 顶部菜单新增"工具"项（`WrenchScrewdriverIcon`），位于"下载"和"设置"之间，点击导航到 `/apps/tools` 工具列表页
- `src/components/layout/TopNavLayout.vue`：navItems 数组新增工具菜单项，active 高亮逻辑新增 `/apps/tools` 前缀匹配（子页面访问时工具按钮也高亮）
- `src/router/index.ts`：注册 `/apps/tools`（工具列表页）和 `/apps/tools/external-download`（外部下载工具页）两条路由
- 新增 `src/views/Tools.vue`：工具列表页，卡片布局对齐 Settings 规范（`rounded-lg border-gray-300` + `px-5 py-4`），目前包含"外部下载工具"一个入口，点击进入子页面
- 新增 `src/views/ExternalDownload.vue`：外部下载工具页面，功能包括：
  - URL + 文件名输入表单（URL 变化时自动从末段推断文件名）
  - 协议白名单校验（仅允许 http/https）
  - 下载进度展示（百分比 + 已下载/总字节 + 速度），300ms 轮询
  - 暂停/恢复/取消操作（复用全局 `pause_download` / `resume_download` / `cancel_download` IPC）
  - 已下载文件列表（文件名 + 大小 + 修改时间），支持删除和打开下载目录
  - 页面恢复机制（用户切回此页时检测正在进行的下载并恢复进度显示）
  - 下载开始/完成/失败通过 Toast 提示

#### 后端新增外部下载 IPC 命令
- `src-tauri/src/storage/mod.rs`：新增 `download_dir()` 方法返回 `.Molaunch/Download/` 路径，`init()` 中自动创建该目录
- `src-tauri/src/commands/system/download.rs`（原占位文件）：实现 4 个 IPC 命令：
  - `download_external_file(url, file_name)`：校验 http/https 协议 + 文件名安全性后，通过 `DownloadManager` 下载到 `.Molaunch/Download/`，进度写入全局 `download_state`（分组"外部下载"），复用 `download_cancel_flag` / `download_pause_flag` 支持暂停/取消
  - `get_external_download_dir()`：返回下载目录路径
  - `list_external_downloads()`：列举已下载文件（名称/大小/修改时间），按修改时间倒序
  - `delete_external_download(file_name)`：删除指定文件（含文件名安全校验）
- `src-tauri/src/commands/system/mod.rs`：`pub use download::*` 导出新命令
- `src-tauri/src/lib.rs`：`invoke_handler!` 注册 4 个新命令
- 安全设计（参考 PCL2 百宝箱 `StartCustomDownload`）：
  - 协议白名单：仅允许 `http://` 和 `https://`，拒绝 `file://`、`ftp://` 等
  - 文件名安全校验：拒绝空字符串、含 `/` `\` `..` `\0` 的文件名，防路径遍历
  - 外部 URL 不经过 `cdn_urls` 镜像策略，直接使用原始 URL

#### 工具页改为侧边栏布局 + 外部下载支持自定义目录
- `src/views/Tools.vue`：从卡片列表布局重写为与 Settings.vue 一致的侧边栏布局（左侧 `w-48` 菜单 + 右侧标题栏 + `v-if` 切换子组件），移除路由跳转改为组件内切换
- `src/views/ExternalDownload.vue`：从独立路由页面改为 Tools 子组件（移除外层 `rounded-xl` 容器和返回按钮），新增下载目录选择器区块：
  - 显示当前生效目录（只读 Input + 打开目录按钮）
  - "选择目录"按钮调用系统文件夹选择对话框，通过 `applyConfig({ externalDownloadDir })` 持久化到 AppConfig
  - "恢复默认"按钮（仅自定义目录时显示）清空配置回退到 `.Molaunch/Download/`
  - 自定义/默认状态标签提示
- `src/router/index.ts`：移除 `/apps/tools/external-download` 独立路由（ExternalDownload 已改为 Tools 子组件）
- `src-tauri/src/state/config.rs`：AppConfig 新增 `external_download_dir: Option<String>` 字段（None 或空则用默认目录）
- `src-tauri/src/config.rs`：`load_config` / `save_config` 新增 `[ExternalDownload] dir` 读写
- `src-tauri/src/commands/system/download.rs`：提取 `resolve_external_download_dir` 公共 helper，`download_external_file` / `get_external_download_dir` / `list_external_downloads` / `delete_external_download` 四个命令统一从 config 读取自定义目录，为空则 fallback 到 `Storage::download_dir()`
- `src-tauri/src/commands/system/apply_config/types.rs`：`ConfigPatch` 新增 `external_download_dir: Option<Option<String>>`（双层 Option 语义：None 不更新 / Some(None) 清空 / Some(Some(dir)) 设置），`ConfigSnapshot` 新增 `external_download_dir: Option<String>`，`build_snapshot` 镜像该字段
- `src-tauri/src/commands/system/apply_config/apply.rs`：新增 `apply_external_download` 域子函数，在 `apply_config_inner` 闭包内统一调用
- `src/utils/api/config.ts`：`ConfigSnapshot` / `ConfigPatch` 前端类型同步新增 `externalDownloadDir` 字段

#### 工具模块化重构 + 便捷工具 + 自动获取文件名
- **后端 tools 模块化**：新建 `src-tauri/src/commands/tools/` 文件夹，包含 6 个模块文件：
  - `mod.rs`：模块声明 + `tools_manager` 统一 IPC 命令，通过 `ToolsRequest.action` 字段分发到 8 个子操作（download_file / get_download_dir / list_downloads / delete_download / fetch_filename / cleanup_scan / cleanup_execute / memory_optimize）
  - `types.rs`：12 个请求/响应类型定义（ToolsRequest、DownloadFileParams、FetchFilenameResult、CleanupItem、MemoryOptimizeResult 等）
  - `download.rs`：从原 `system/download.rs` 迁移的外部下载逻辑，函数改为普通 `pub async fn` 接收 typed params，保留 `resolve_external_download_dir` 公共 helper
  - `filename.rs`：HEAD 优先 → 失败回退 GET with `Range: bytes=0-0`，解析 `Content-Disposition`（先 RFC 5987 `filename*=UTF-8''xxx` 用 `urlencoding::decode`，再 `filename="xxx"`），都没有则从 URL path 提取；同时取 `Content-Length` 作为 file_size
  - `cleanup.rs`：扫描 `.minecraft` 下的 `logs`/`crash-reports`/`.mixin.out`/`screenshots` 四个目录（screenshots 标为"可选"），`execute` 用 `is_path_safe` 基于 canonicalize 做路径遍历防护，自底向上删除文件再删目录
  - `memory.rs`：Windows 调用 `SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX)` 释放进程工作集，sysinfo 0.29 API 取 `available_memory()` 前后差值返回 freed_kb
- **旧代码清理**：删除 `src-tauri/src/commands/system/download.rs`，`system/mod.rs` 移除 `mod download; pub use download::*;`，`lib.rs` 移除 4 个旧命令注册替换为 `commands::tools::tools_manager`
- `src-tauri/src/commands/mod.rs`：新增 `pub mod tools;`
- **前端统一 API**：新建 `src/utils/api/tools.ts`，提供 `toolsManager<T>(action, params)` 泛型封装 + 8 个类型安全的具名函数（downloadFile / getDownloadDir / listDownloads / deleteDownload / fetchFilename / cleanupScan / cleanupExecute / memoryOptimize）
- **ExternalDownload.vue 重新设计**：
  - 下载目录 UI 从只读 Input + 挤压按钮重设计为灰底信息条（文件夹图标 + 路径文字 + 状态标签）+ 独立操作按钮行（选择目录 / 打开目录 / 恢复默认）
  - 粘贴链接后自动获取文件名：watch URL 防抖 500ms → 调用 `fetch_filename` → 自动补全文件名输入框，期间输入框禁用并显示旋转加载图标；用户手动编辑文件名后停止自动补全
  - 集成 versionStore 替代本地轮询：`startDownload` 时调用 `versionStore.startDownload(fileName)` 触发 `useDownloadPolling` 全局轮询，下载进度在下载管理页（Downloads.vue）以"外部下载"分组可见；watch `versionStore.downloading` 检测完成后自动刷新文件列表
- **QuickTools.vue 新建**：便捷工具子组件，含三个区块：
  - 清理游戏垃圾：扫描 → 勾选（非可选项默认选中）→ 确认清理 → 展示结果（清理大小/文件数/失败项），支持重新扫描
  - 内存优化：一键优化按钮，展示前后可用内存对比和释放量
  - 更多工具敬请期待：6 个占位卡片（存档备份恢复/Mod依赖检测/游戏日志分析/Java版本管理/服务器状态检测/世界存档管理），点击提示"敬请期待"
- `src/views/Tools.vue`：侧边栏新增"便捷工具"菜单项（WrenchScrewdriverIcon），右侧内容区 v-if 切换 ExternalDownload / QuickTools

#### 细化清理游戏垃圾扫描范围
- `src-tauri/src/commands/tools/cleanup.rs` 扩展扫描目录：
  - 固定子目录新增 `assets/cache`（资源索引缓存）、`.fabric/remapCache`（Fabric 重映射缓存），均为安全可清理内容
  - 新增通配符扫描 `versions/*/natives/`：遍历 `.minecraft/versions/` 下每个版本目录的 `natives` 子目录（原生库提取目录，每次启动游戏重新提取），每个版本单独作为一个清理项（显示名 "原生库 - <版本名>"），按版本名排序保证展示稳定
- 新增 `build_allowed_parents` 公共函数：scan 与 execute 共用，构建所有允许清理的目录列表（固定目录 + 所有 versions/*/natives），确保安全检查与扫描结果完全一致，避免路径遍历
- 原有 `logs`/`crash-reports`/`.mixin.out`/`screenshots` 保持不变

#### iframe sandbox 方案回退（保留 allow-same-origin）
- `src/plugins/custom-layout/CustomLayoutPanel.vue` 与 `HtmlLayoutPanel.vue` 的 iframe sandbox 保留 `allow-scripts allow-same-origin`
- 原因：Tauri 2 通过 WebView2 的 `AddScriptToExecuteOnDocumentCreated` 在所有页面 `<script>` 之前注入 IPC 初始化脚本，需同源才能正确设置 `__TAURI_INTERNALS__`；去掉 `allow-same-origin` 会导致 "Cannot read properties of undefined (reading 'plugins')" 报错（桩 `<script>` 来不及在 Tauri 脚本之前执行）
- 自定义布局内容为用户可信配置，`allow-same-origin` 的安全风险可接受；sandbox 警告为 WebView 安全提示，不影响功能

#### 内存优化跨平台 + 示例布局移除 html section + 内存显示格式优化
- `src-tauri/src/commands/tools/memory.rs`：内存优化新增 Linux 和 macOS 平台支持：
  - Windows：`SetProcessWorkingSetSize(GetCurrentProcess(), -1, -1)` 裁剪工作集（不变）
  - Linux：FFI 调用 glibc `malloc_trim(0)` 归还堆碎片给 OS
  - macOS：FFI 调用 `malloc_zone_pressure_relief(NULL, 0)` 释放所有 malloc zone 空闲内存（用 opaque `*mut c_void` 指针避免声明复杂的 `malloc_zone_t` 结构体）
  - 提取 `release_process_memory()` 公共函数，用 `#[cfg(target_os)]` 分平台编译，sleep 从 100ms 调整为 150ms 让 OS 充分回收
- `src-tauri/resources/samples/layout/layout-sample.json` 和 `layout-sample.xml`：恢复 html section（含自定义 HTML + 按钮 + 徽章 + 进度条示例 + `console.log` 脚本），验证 shadow DOM 渲染方案
- `src/views/QuickTools.vue`：内存优化结果显示从单行改为两行布局——首行突出显示"已释放 X"，次行显示"系统可用内存 before → after"；移除 `formatMemoryKb`，直接用 `formatBytes` 格式化字节值

#### 修复内存数据单位错误（16GB 物理内存显示 5.57TB 可用）
- 根因：sysinfo 0.29.11 在 Windows 上 `available_memory()` 实际返回**字节**（而非文档声明的 KB），代码误将字节值标记为 KB 字段名，前端又 × 1024 导致放大 1024 倍
- `src-tauri/src/commands/tools/memory.rs`：新增 `get_available_memory_bytes()` 函数，通过 `total_memory()` 量级判断 sysinfo 返回单位（>10亿视为字节，否则视为 KB × 1024），统一返回字节
- `src-tauri/src/commands/tools/types.rs`：`MemoryOptimizeResult` 字段名从 `freed_kb`/`before_kb`/`after_kb` 改为 `freed_bytes`/`before_bytes`/`after_bytes`，语义准确
- `src/views/QuickTools.vue`：更新字段名引用，用 `formatBytes()` 直接格式化（不再 × 1024）

#### iframe → shadow DOM 渲染方案（彻底消除 sandbox 安全警告）
- `src/plugins/custom-layout/CustomLayoutPanel.vue`：html section 从 `<iframe sandbox="allow-scripts allow-same-origin">` 改为 shadow DOM 渲染
  - 新增 `renderHtmlShadow(container, section)` 函数：创建 shadow root → 注入设计系统 CSS + 用户样式 → innerHTML 插入用户 HTML → `new Function` 执行用户脚本
  - 新增 `setupMolaunchApi()` 函数：直接在主窗口上下文定义 `window.molaunch`（toast/alert/confirm/prompt），无需 postMessage 桥接
  - 移除 `buildHtmlSrcDoc`、`UI_API_SCRIPT`、`handleUiRequest` 和 message 监听器（iframe 专属逻辑）
  - 用内容指纹（`dataset.renderedKey`）避免相同内容重复渲染
- `src/plugins/custom-layout/HtmlLayoutPanel.vue`：整体从 iframe 改为 shadow DOM
  - 新增 `renderHtml()` 函数：创建 shadow root → innerHTML 插入 HTML → 提取 `<script>` 标签并 `new Function` 执行（innerHTML 插入的 script 不自动执行）
  - `window.molaunch` 通过 Proxy 代理到 `pluginSdk`（拦截 `spawnProcess`/`createWindow` 等危险方法）
  - 移除 `buildSandboxHtml` 调用、`handleMessage` 和 message 监听器
  - 用 `watch(props.content)` + `nextTick` 响应内容变化重新渲染
- 优势：无 iframe → 无 sandbox 警告；无 Tauri IPC 注入 → 无 "Cannot read properties of undefined (reading 'plugins')" 报错；shadow DOM → CSS 隔离；`new Function` → JS 可执行

#### 内存优化改为枚举所有进程裁剪工作集（与 PCL2 一致，释放量从几十 MB 提升到数 GB）
- 根因：原实现仅对启动器自身进程调用 `SetProcessWorkingSetSize`，只释放了启动器自己的工作集（几十 MB）；PCL2 枚举系统所有进程逐个裁剪工作集，释放整个系统的物理内存（数 GB）
- `src-tauri/src/commands/tools/memory.rs`：`release_process_memory` 改为遍历系统进程快照：
  - 用 `CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)` 创建进程快照
  - 用 `Process32FirstW` / `Process32NextW` 遍历所有进程
  - 对每个进程用 `OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_SET_QUOTA, ...)` 打开
  - 调用 `SetProcessWorkingSetSize(handle, -1, -1)` 裁剪工作集
  - 对打开失败 / 设置失败的进程（如受保护的系统进程）静默跳过
  - 统计成功 / 失败进程数并打印日志
- `src-tauri/Cargo.toml`：windows crate 新增 `Win32_System_Diagnostics_ToolHelp` feature（进程快照 API）
- sleep 从 150ms 调整为 500ms 让 OS 充分回收多进程工作集

### 优化

#### 窗口最小尺寸调整
- `src-tauri/tauri.conf.json`：窗口最小尺寸从 900×600 调整为 1090×592，避免界面元素拥挤

#### Mod 管理界面优化
- `src/views/version-settings/mod-tab/ModToolbar.vue`：搜索框宽度减半（`w-56` → `w-28`），筛选按钮组增加内边距和按钮间距（`p-0.5` → `p-1`、`px-2.5` → `px-3`、`gap-1` → `gap-1.5`），搜索框与操作栏保持在同一行不换行
- `src/views/version-settings/mod-tab/ModEmptyState.vue`：空状态改为上下左右居中显示（`flex h-full min-h-[400px] items-center justify-center`），图标增加圆角灰色背景容器，字体和信息层级优化（标题 15px font-semibold、副标题 13px），无匹配时增加"试试调整筛选条件或搜索关键词"提示
- `src/views/version-settings/mod-tab/ModListItem.vue`：列表项字体优化（标题 `text-[13px] font-semibold`、详情行 `text-[11px] font-medium`），分隔符改为 `|`，文件名截断长度从 28 字符增至 32 字符

#### 启动初始化并行化（减少 Java 搜索等待时间）
- `src/App.vue`：`initApp()` 中 `detectJava()` 提前并行启动（不再等待 SDK 和认证恢复完成）；`fetchPlatformInfo()` 和 `fetchDeviceId()` 改为 `Promise.all` 并行获取；路由修正后再 `await javaPromise` 确保完成

### 修复

#### 下载缓存图片时 cache-image.localhost 连接失败
- `src-tauri/src/minecraft/image_cache.rs`：新增三个公共函数：`is_cache_url()` 判断 URL 是否为 Tauri WebView 内部虚拟 URL、`read_cache_by_url()` 从虚拟 URL 直接读取本地缓存文件内容、`cache_path_by_url()` 返回缓存文件路径；`get_image_url()` 入口增加防御性检查，误传 cache-image 虚拟 URL 时直接返回避免 reqwest 请求虚拟 URL
- `src-tauri/src/commands/skin.rs`：`download_url_to_file` 命令改用公共函数 `read_cache_by_url` 识别虚拟 URL 并从本地缓存读取，不再用 reqwest 发起无法连接的 HTTP 请求
- 已排查确认启动流程（`launch.rs` / `script_export.rs` / `minecraft/launch/`）不存在同样问题，启动 IPC 参数不包含 URL 字段，后端 reqwest 调用全部使用后端硬编码 URL 或 profile_json 远程 URL
- 已排查确认前端其他 URL→后端调用点（`download_resource_to_path` / `install_modpack`）接收的是 CurseForge/Modrinth 平台 jar 文件远程 URL，不经过 image-cache 系统，确定安全；`<img>` src 等 WebView 内部场景均不会回传给后端 invoke

#### Button 组件除 primary 外所有类型按钮无边框/背景丢失
- `src/components/common/Button.vue`：将类名从动态模板字符串 `` `btn-${type}` `` / `` `btn-size-${size}` `` 改为静态 switch 映射（`typeClass`/`sizeClass`）。原因：Tailwind purge 扫描器只能静态识别源码中出现的完整类名字符串，无法推断模板字符串展开结果，导致 `main.css` 的 `@layer components` 中 `btn-outline`/`btn-secondary`/`btn-ghost`/`btn-text` 等自定义类被判定为"未使用"而整体 purge，这些类型按钮丢失 border/background/color 只剩文字。改用静态映射后 Tailwind 可在源码中识别到完整类名，保留对应样式
- `src/composables/useSwipeNavigation.ts`：`onPointerDown` 在 `pointerdown` 起源于交互元素（button/a/input 等）时提前返回，跳过 `setPointerCapture` 指针捕获，避免拖拽逻辑劫持点击事件导致皮肤/登出按钮的 `click` 无法派发、弹窗不弹出的问题

### 优化

#### 账号卡片拖拽体验优化
- `src/composables/useSwipeNavigation.ts`：重写拖拽逻辑，新增边界阻尼（首尾卡片拖拽阻力减半）、`isAnimating` 状态标记、左键/触摸判断；切换阈值从 60px 降为 40px 更灵敏
- `src/composables/useSwipeNavigation.ts`：`onPointerDown` 调用 `setPointerCapture` 捕获指针，`onPointerUp` 调用 `releasePointerCapture` 释放，修复指针移出容器外部后丢失拖拽状态的问题
- `src/components/home/AccountSelector.vue`：`switchTo` 视觉索引立即更新不再被 switching 锁阻塞，账号切换异步进行；卡片容器增加 `cursor-grab/grabbing` 光标反馈、`will-change-transform` GPU 加速、`select-none` 防止拖动选中文字

#### 离线账号皮肤接入启动流程（PCL2 方案 A + 方案 B）
- `src-tauri/src/minecraft/auth/mod.rs`：新增 `adjust_uuid_for_skin_variant()` 函数，通过递增 UUID 末位让 MC 离线模式哈希到目标皮肤模型（Steve=classic / Alex=slim），算法参考 PCL2 的 `McSkinSex` 函数
- `src-tauri/src/minecraft/launch/skin_resourcepack.rs`：新增独立模块实现方案 B（资源包替换），包含 pack_format 版本映射、1.19.3+ 9 角色路径处理、zip 生成、options.txt resourcePacks 字段修改；支持默认皮肤和自定义皮肤（`custom:` 前缀）；含 4 个单元测试
- `src-tauri/src/minecraft/launch/mod.rs`：添加 `pub mod skin_resourcepack` 模块声明
- `src-tauri/src/resources.rs`：注册 9 个离线皮肤 PNG 文件（从 `src/assets/Skins/` 复制到 `src-tauri/resources/skins/`），新增 `get_embedded_resource()` 公共接口供 `skin_resourcepack` 模块直接读取嵌入资源
- `src-tauri/src/commands/version/launch.rs`：方案 A（UUID 调整）+ 方案 B（资源包生成）协同工作；非离线账号启动时自动清理残留资源包
- `src-tauri/src/commands/version/script_export.rs`：导出启动脚本时同步应用 UUID 调整
- `src-tauri/src/commands/auth/account.rs`：新增 `save_custom_skin` IPC 命令，将用户选择的 PNG 文件复制到 `<app_data>/custom_skins/<uuid>.png`，验证 PNG 文件头和大小（<1MB），写入 `custom:<path>|<variant>` 格式的 skin 字段
- `src-tauri/src/lib.rs`：注册 `save_custom_skin` 命令
- `src/utils/api/auth.ts`：新增 `saveCustomSkin` API 封装
- `src/utils/default-skin.ts`：新增 `isVersion1193Plus`、`getDefaultSkinsForVersion`、`parseSkinUrl`、`isCustomSkin`、`parseSkinVariant` 工具函数；`getDefaultSkin` 和 `getDefaultSkinEntry` 支持自定义皮肤 URL 解析（通过 `convertFileSrc`）
- `src/components/common/skin-manager/SkinLocalSelector.vue`：根据 MC 版本过滤可选皮肤（1.19.3+ 显示 9 个，旧版只显示 Steve/Alex）；新增自定义皮肤上传按钮区
- `src/composables/useSkinOperations.ts`：`loadInfo` 和 `onSelectLocalSkin` 支持自定义皮肤 URL 解析；新增 `onUploadCustomSkin` handler
- `src/components/common/SkinManager.vue`：引入 `useVersionStore` 获取当前 MC 版本传给 `SkinLocalSelector`；传递 `onUploadCustomSkin` handler

#### SDK 加载逻辑与日志优化
- `src-tauri/src/sdk/mod.rs`：重写 `get_sdk_resource_dir()`，优先从发布模式资源目录（`<exe_dir>/resources/sdk_data/`）查找 SDK 库，开发模式路径用词法规范化去掉 `../`，macOS 额外查找 `.app/Contents/Resources/sdk_data/`
- 新增 `normalize_path()` 函数：词法规范化路径组件，避免日志显示 `src-tauri/../sdk_data` 这类不美观路径
- `src-tauri/src/state/app.rs`：去掉重复的 `Loaded config from file` 日志（`config.rs` 中已有 `Config loaded from storage`）

#### SDK 动态库改为嵌入二进制释放到临时目录
- `src-tauri/src/resources.rs`：`embedded_bytes` 中按平台条件编译注册 SDK 动态库（`cfg(target_os = "windows/macos/linux")`），新增 `extract_sdk()` 函数释放 SDK 到 `<temp>/MoLaunch/sdk/` 目录
- `src-tauri/src/sdk/mod.rs`：删除 `get_sdk_resource_dir()` 和 `normalize_path()`，`check_sdk_library()` 改为调用 `resources::extract_sdk()` 确保释放后返回临时目录路径，`get_sdk_library_path()` 改为返回临时目录路径
- `src-tauri/tauri.conf.json`：移除 `../sdk_data/*` 资源打包配置（SDK 不再从外部文件加载）
- 释放策略复用 `extract_resource` 的 sha256 校验机制：SDK 热更新（手动替换临时目录文件）不会触发覆盖，主程序更新后嵌入版本变化自动覆盖旧版，临时目录被清理后自动重新释放

#### 全局替换原生 tooltip 为 Tooltip 组件 + 账号卡片按钮统一
- `src/components/home/account-selector/AccountCard.vue`：皮肤按钮和退出按钮统一为 `Button type="outline" size="mini"`，退出按钮用 `!text-red-500` 等类保留红色危险操作样式，两个按钮均用 `Tooltip` 组件替换原生 `title` 属性
- 9 个文件原生 `title` 属性替换为 `Tooltip` 组件：`InstalledList.vue`、`BackToTop.vue`、`TaskGroupCard.vue`（2处）、`MultiSelectBar.vue`（4处）、`MemorySection.vue`（3处）、`SettingsAdvanced.vue`、`MemoryAllocation.vue`（3处）、`FabricApiInfoCard.vue`、`SkinCapeList.vue`

#### 离线皮肤弹窗提示版本不支持
- `src/utils/element-icons.ts`：新增 Element Plus Icons SVG path 数据文件，包含 info/warning/error/success/debug 5 种图标，带 MIT 版权注释
- `src/components/common/Alert.vue`：扩展提示框组件，新增 error（红色）和 debug（青色）类型，原有 info/warning/success 保留，共支持 5 种类型（v1 Arco 风格：白底左色条）
- `src/components/common/AlertV2.vue`：新增第二版提示框组件（v2 灰底简洁风格），图标改用 Element Plus Icons（从 `element-icons.ts` 引入），`leading-relaxed` 确保文字与图标对齐
- `src/components/common/SkinManager.vue`：离线账号皮肤弹窗顶部用 AlertV2（type=info）替换 Alert，内容区 max-h 从 70vh 调为 80vh 避免加提示后触发滚动条
- `src/components/common/skin-manager/SkinLocalSelector.vue`：移除内联提示，改由 SkinManager 弹窗顶部的 AlertV2 组件统一显示
- `src/assets/styles/main.css`：html/body 新增 `overflow: hidden; height: 100%; margin: 0`，修复 App 容器外部出现滚动条的问题

### 重构

#### 项目文档全面重构
- `README.md`：基于项目真实技术栈重写，修正前端为 Tailwind CSS + @heroicons/vue + skinview3d + vue-virtual-scroller，后端为纯 Rust minecraft 模块（非 McSDK C FFI），项目结构按实际目录重写，删除过时路线图，更新致谢与文档链接
- `AI_AGENT_GUIDELINES.md`：精简为 AI 协作行为约束，聚焦 CHANGELOG 规则、Git 提交规范（`!c` 后缀 + 常规泛化描述约束）、修改前/后检查清单、禁止/必须事项
- `DEVELOPMENT_GUIDELINES.md`：完整开发规范，修正日志宏为项目自定义宏（log_info!/log_warn! 等），Tauri 命令模板使用 State + lock/drop 模式，Vue 模板使用 script setup + Composition API + 自定义组件，新增 Arco Design 风格 UI 规范
- `DEVELOPMENT_BLUEPRINT.md`：基于真实架构重写，覆盖后端 commands/minecraft/state/storage 分层、前端 components/composables/stores 结构、IPC 数据流、下载状态轮询、启动流水线、安全规范、z-index 层级（9999/10000/10001）

### 新增

#### 整合包安装支持自定义安装名称
- `src/components/community/ResourceDetail.vue`：点击整合包下载后，弹窗询问安装名称（默认填入整合包译名/原名/文件名），用户可自定义；取消则中止安装
- 新增 `promptForInstanceName` 辅助函数：将 callback 风格的 `showPrompt` 包装为 Promise，便于在 async 流程中 await

#### Modal 全局弹窗层级与尺寸优化
- `src/components/common/Modal.vue`：z-index 从 9999 提升至 10000，确保覆盖所有业务弹窗（ResourceDetail/ModUpdateDialog 等 9999 层级），修复整合包安装名称询问框被版本详情弹窗遮挡的问题
- 弹窗宽度从 max-w-sm 放大到 max-w-md，内边距 p-5→p-6，标题字号 text-sm→text-base，图标 w-5→w-6，输入框 py-2→py-2.5，按钮栏 py-3→py-3.5，整体视觉更协调

#### Toast 全局提示置顶
- `src/components/common/Toast.vue`：z-index 从 9998 提升至 10001，确保 Toast 始终显示在所有弹窗之上（Modal 10000 / 业务弹窗 9999），修复弹窗遮挡 Toast 提示的问题

#### 资源详情 MC 百科按钮按需显示
- `src/components/community/resource-detail/ResourceDetailHeader.vue`：「转到 MC百科」按钮改为异步查询 mcmod 数据库直链，查到才显示按钮，查不到直接不显示（原逻辑是查不到回退到搜索页）
- 新增 `mcmodUrl` ref + watch：project 变化时异步查询，`openMcmod` 简化为直接打开已查到的直链

#### 全局替换 Emoji 为图标组件
- `src/views/Community.vue`：搜索页空结果 🔍 Emoji 替换为放大镜 SVG 图标，空状态改为 `h-full` 撑满容器实现上下左右居中
- `src/components/community/ResourceCard.vue`：资源卡片无 Logo 时 📦 Emoji 替换为 CubeIcon 图标
- `src/components/common/DeviceCodeModal.vue`：登录步骤完成标记 ✓ Emoji 替换为 CheckIcon 图标
- `src/views/version-settings/setup-tab/JavaCustomMode.vue`：Java 兼容性标记 ✓/✗ 替换为「兼容/不兼容」文字
- `src/components/version-settings/AdvanceFieldsPanel.vue`：安全警告 ⚠️ Emoji 替换为【安全警告】文字标记
- `src/composables/useDownloadPolling.ts`：调试日志 ⚠️ Emoji 替换为 [WARN] 文字标记

#### 搜索栏新增独立搜索按钮 + Select 变化触发搜索
- `src/components/community/SearchBar.vue`：第一行新增「搜索」主按钮（带放大镜图标），与「重置」按钮并列
- 三个 Select（来源/加载器/分类）选项变化时通过 `selectAndUpdate` 辅助函数同步更新 v-model 并立即触发 search 事件，无需手动在搜索框输入才生效
- 游戏版本输入框新增 `@keydown.enter` 回车触发搜索

#### 下载管理页无任务时显示极简占位画面
- `src/views/Downloads.vue`：3 秒重试检查期间显示"正在检查下载任务..."加载动画，避免页面空白
- 检查完毕仍无任务时显示"暂无下载任务，即将返回上一页..."极简空状态，停留 1.5 秒后自动返回，避免突兀跳转

### 重构

#### 阶段 1：重复代码整合（参考 docs/CODE_QUALITY_REPORT.md）

##### 1.1 useConfigPage composable（消除 4 处 Settings 页样板）
- 新建 `src/composables/useConfigPage.ts`：抽象 onMounted 加载配置 + useDebouncedSave('patch', applyConfig) + watch+markDirty+loaded 守卫三件套
- 重构 `SettingsDownload.vue`、`SettingsAdvanced.vue`、`CommunityConfigCard.vue`、`SettingsLaunch.vue` 使用 useConfigPage，每处减少 15-25 行样板代码

##### 1.2 useMemoryVisualizer composable（消除内存可视化重复）
- 新建 `src/composables/useMemoryVisualizer.ts`：封装系统内存轮询 + 6 个 computed（totalMemoryMB/usedMemoryMB/gameMemoryMB/otherMemoryMB/usedPercent/gamePercent）+ applyAutoMemory
- 重构 `SettingsLaunch.vue` 和 `MemorySection.vue` 使用 useMemoryVisualizer，消除两文件间逐字相同的内存可视化逻辑

##### 1.6 + 1.7 后端 state helper（消除 12 处 lock/clone/drop 套件）
- 在 `state/mod.rs` 新增 `resolve_mirror_and_source(state)` 和 `resolve_game_dir_from_state(state)` 两个 async helper
- 重构 `loaders.rs`（5 处 mirror_and_source）、`personalization.rs`（2 处 game_dir）、`manage.rs`（1 处 game_dir）、`list.rs`（4 处 game_dir）使用新 helper

##### 1.9 VersionPersonalization serde rename 修复（消除 snakeMap workaround）
- 后端 `commands/version/personalization.rs`：给 `VersionPersonalization` 加 `#[serde(rename_all = "camelCase")]`，与 `PersonalizationUpdate` 保持一致
- 前端 `utils/api/personalization.ts`：`VersionPersonalization` 接口所有字段从 snake_case 改为 camelCase
- 前端 `SetupTab.vue`：删除 `snakeMap` 转换表（11 行），`savePersonalField`/`saveAdvanceSwitch` 直接用字段名同步共享状态
- 前端 `OverviewTab.vue`、`ModTab.vue`、`MemorySection.vue`、`setup-tab/JavaModeSelector.vue`：所有 `p.xxx_yyy` 访问改为 `p.xxxYyy`

##### 2.1 LoaderSelect.vue 拆分（422 → 283 行，低于 300 行限制）
- 新建 `src/composables/useFabricApi.ts`：封装 Fabric API 版本查询状态管理（state/latest/error refs + fetchFabricApi + retry + watch selected 触发）
- 新建 `src/components/install/FabricApiInfoCard.vue`：纯展示组件，接收 state/latest/error props + emit retry，内部使用 formatBytes/formatDate
- `src/utils/format.ts`：新增 `formatDate`（ISO 日期 → YYYY-MM-DD）
- 重构 `src/views/LoaderSelect.vue`：使用 useFabricApi composable + FabricApiInfoCard 子组件，移除内联 Fabric API 状态/格式化代码（约 130 行）

##### 3.1 install/mod.rs 拆分（612 → 253 行 + 4 个子模块）
- 新建 `stages.rs`（115 行）：`install_all_loaders` 批量加载器安装
- 新建 `post_install.rs`（156 行）：`merge_and_rename_version` JSON 合并 + 目录重命名
- 新建 `setup_persist.rs`（53 行）：`save_setup_and_create_isolation` setup.ini 保存 + 隔离目录创建
- 新建 `fabric_api.rs`（117 行）：`auto_install_fabric_api` Fabric API 自动安装
- 重构 `mod.rs`（253 行）：保留 MC 下载编排 + 阶段管理 + 取消/错误处理，调用 4 个子模块函数

##### 1.3-1.5 + 1.8 工具函数整合
- `utils/format.ts`：新增 `formatSpeedCompact`（紧凑速度格式，用于下载进度条）
- `utils/toast.ts`：新增 `toastSuccess`/`toastError`/`toastWarning`/`toastInfo` 推荐函数名，保留 `showSuccess` 等为兼容别名，消除与 `modal.ts` 的命名冲突
- 新建 `utils/async.ts`：`safeCall`/`safeCallSync` 高阶函数，统一 try/catch + console.error 样板
- 新建后端 `error_util.rs`：`log_err`/`log_err_with` 辅助函数，统一 `.map_err(|e| { log_error!(...); e.to_string() })` 样板
- `DownloadProgressOverlay.vue`：移除本地 `formatSpeed`，改用 `formatSpeedCompact`
- `Versions.vue`：toast 引入改用 `toastSuccess`/`toastInfo`/`toastWarning` 前缀

#### 阶段 2：前端超长文件拆分

##### 2.2 Downloads.vue 拆分（364 → 139 行）
- 新建 `composables/useDownloadTaskGroups.ts`：下载任务分组逻辑
- 新建 `components/downloads/TaskGroupCard.vue`：任务卡片子组件

##### 2.3 + 2.4 SettingsLaunch + SetupTab 共建 ToggleRow
- 新建 `components/settings/ToggleRow.vue`：公共开关行组件（Tooltip + Toggle）
- 新建 `views/settings/settings-launch/MemoryAllocation.vue`：内存分配子组件
- 新建 `components/version-settings/AdvanceFieldsPanel.vue`：高级选项面板
- `SettingsLaunch.vue` 334 → 135 行，`SetupTab.vue` 340 → 232 行

##### 2.5 Versions.vue 拆分（327 → 211 行）
- 新建 `composables/useVersionInstallActions.ts`：安装/卸载/下载 handler 下沉
- 新建 `views/downloads/DownloadSidebar.vue`：侧边栏子组件

##### 2.6 ModTab.vue 拆分（309 → 104 行）
- 新建 `composables/useModOperations.ts`：8 个 handler + filteredMods 下沉

##### 2.7 SkinManager.vue 拆分（304 → 144 行）
- 新建 `composables/useSkinOperations.ts`：7 个异步操作 + image-cache 监听下沉
- 新建 `components/common/skin-manager/SkinPreviewPanel.vue`：预览面板子组件

##### 2.8 OverviewTab.vue 拆分（303 → 177 行）
- 新建 `composables/useVersionOverviewActions.ts`：8 个 handler 下沉

#### 阶段 3：后端超长文件拆分

##### 3.2 java/download.rs 拆分（554 → 121 行 mod.rs + 7 子模块）
- 拆为 `download/{mod,constants,types,match,fetch,files,verify,progress}.rs`

##### 3.3 community/preload.rs 拆分（404 → 125 行 mod.rs + 5 子模块）
- 拆为 `preload/{mod,types,cache,hash,jar_metadata,online_query}.rs`

##### 3.4 launch/watcher/analyzer.rs 拆分（728 → 88 行 mod.rs + 5 子模块）
- 拆为 `analyzer/{mod,crit1,stack,crit3,collect,util}.rs`
- `analyze_crit1` 函数（221行）拆为 4 个子函数

##### 3.5 version/setup/mod.rs 拆分（585 → 21 行 mod.rs + 5 子模块）
- 拆为 `setup/{mod,types,save,load,update,helpers,tests}.rs`
- 新增 4 个分组子 struct（LoaderInfo/DisplayConfig/JavaConfig/AdvancedConfig）

##### 3.6 apply_config.rs 拆分（454 → 87 行 mod.rs + 4 子模块）
- 拆为 `apply_config/{mod,types,validate,apply,secure}.rs`
- `apply_config_inner` 按域拆为 6 个子函数

##### 3.7 launch/mod.rs 拆分（615 → 64 行 mod.rs + 5 子模块）
- 拆为 `launch/{mod,arguments,classpath,jvm_args,game_args,embedded}.rs`
- `build_jvm_args` 函数（146行）拆为 5 个子函数

##### 3.8 java_selector.rs 拆分（511 → 26 行 mod.rs + 6 子模块）
- 拆为 `java_selector/{mod,rules,compat,weight,select,installer,tests}.rs`

##### 3.9 java/mod.rs 拆分（493 → 28 行 mod.rs + 3 子模块）
- 拆为 `java/{mod,detect,search,select}.rs`
- `search_java_with_paths`（138行）拆为 4 个函数 + CandidateCollector 结构体

##### 3.10 download/mod.rs 拆分（549 → 30 行 mod.rs + 5 子模块）
- 拆为 `download/{mod,version_list,full_download,stages,fix,util}.rs`

##### 3.11 launch/pipeline.rs 拆分（521 → 103 行 mod.rs + 3 子模块）
- 拆为 `pipeline/{mod,types,execute,validate}.rs`

##### 3.12 download/chunk.rs 拆分（440 → 243 行 mod.rs + 4 子模块）
- 拆为 `chunk/{mod,probe,download,merge,util}.rs`

#### 阶段 4：lib.rs 优化

##### 4.1 图片缓存协议抽离
- `minecraft/image_cache.rs` 新增 `register_uri_scheme(builder)` 函数
- `lib.rs` 中 54 行内联协议注册闭包抽离为一行调用
- 消除 3 处重复的空响应构造

##### 4.2 版本域命令分组重组
- 47 个 version 命令按子域分组并添加注释头（list/folder/download/install/loaders/manage/personalization/mods/preload/progress/launch/script_export）

#### 修复：cleanup_failed_install dead_code 警告
- `cleanup_failed_install` 函数在 Phase 3 重构（任务 3.1）拆分 `install/mod.rs` 时丢失了调用点
- 在 `install/mod.rs` 的两个失败路径恢复清理调用：
  - MC 本体下载失败时（`download_version_full` 返回 Err）
  - 加载器安装失败时（`loader_errors` 非空）
- 用户取消安装的路径不触发清理（保留部分下载以便后续恢复）

### 优化

#### 开发者模式配置统一到 get_config/apply_config
- 将开发者模式开关的获取/修改从独立 IPC（`is_developer_mode`/`set_developer_mode`）统一到 `get_config`/`apply_config`，与项目约定一致
- 后端改动：
  - `commands/system/apply_config.rs`：`ConfigSnapshot` 新增 `developerMode`/`developerUnlocked` 字段，`ConfigPatch` 新增 `developerMode` 字段；`get_config` 从注册表读取填充，`apply_config` 分流写入注册表（含解锁校验）
  - `commands/system/developer.rs`：移除 `is_developer_mode`/`set_developer_mode` 命令，`KEY_DEV_UNLOCKED`/`KEY_DEV_MODE` 常量改为 `pub` 供 apply_config 引用
  - `lib.rs`：注销 `is_developer_mode`/`set_developer_mode` 命令
- 前端改动：
  - `utils/api/config.ts`：`ConfigSnapshot` 新增 `developerMode`/`developerUnlocked`，`ConfigPatch` 新增 `developerMode`
  - `utils/api/developer.ts`：移除 `isDeveloperMode`/`setDeveloperMode` 函数
  - `components/settings/DevModeToggle.vue`：改用 `getConfigMap`/`applyConfig` 读写开发者模式
  - `views/Settings.vue`：改用 `getConfigMap` 读取 `developerMode` 决定侧边菜单显隐

### 新增

#### 下载暂停/取消功能
- 新增下载进度页面的暂停和取消按钮，用户可在安装过程中随时暂停或终止下载任务
- 后端改动：
  - `state/app.rs`：`AppState` 新增 `download_pause_flag: Arc<AtomicBool>` 字段
  - `minecraft/download/manager.rs`：`DownloadManager` 新增 `pause_flag` 字段和 `with_pause_flag()` 方法，`download_batch` 循环中检查暂停信号并等待
  - `minecraft/download/mod.rs`：`download_version_full` 新增 `pause_flag` 参数
  - `commands/version/progress.rs`：新增 `cancel_download` / `pause_download` / `resume_download` 三个 Tauri 命令
  - `commands/version/install/mod.rs`：`install_merged` 在开始时重置取消/暂停标志，在 MC 下载后、加载器安装前、Fabric API 安装前检查取消信号
  - `commands/version/download.rs`：`download_version` 传入 cancel/pause 标志
  - `commands/version/types.rs`：`DownloadStageSnapshot` 新增 `group` 和 `is_paused` 字段
- 前端改动：
  - `utils/api/system.ts`：新增 `cancelDownload` / `pauseDownload` / `resumeDownload` API 封装
  - `types/download.ts`：`RawDownloadStage` 新增 `is_paused` 字段，`DownloadProgress` 新增 `isPaused` 字段
  - `composables/useDownloadPolling.ts`：从 stage 的 `is_paused` 推导全局暂停状态
  - `views/Downloads.vue`：卡片头部新增暂停/恢复和取消按钮，暂停时进度图标切换为暂停状态

#### Mod 多选与版本更新功能（参考 PCL2 PageInstanceMod）
- 版本设置 Mod 管理页新增多选模式与版本更新/更改功能
- 多选交互（复刻 PCL2 PageInstanceMod + MyLocalModItem）：
  - **点击列表项即切换选中**（PCL2 也是点击触发，非长按）
  - Shift+点击 范围选择，ESC 清空选中
  - 批量操作：启用、禁用、更新、删除、全选、反选
  - **批量操作完成后自动清空选中**（参考 PCL2 第 465、678 行 `ChangeAllSelected(False)`）：启用/禁用、删除操作成功后无条件调用 `clearSelection()`，退出多选状态
- 按钮智能禁用（复刻 PCL2 PageInstanceMod.xaml.vb 第 202-216 行）：
  - 选中项中没有已启用的 mod 时，"禁用"按钮禁用（`hasEnabledSelected`）
  - 选中项中没有已禁用的 mod 时，"启用"按钮禁用（`hasDisabledSelected`）
  - 选中项中没有可更新的 mod 时，"更新"按钮禁用（`hasUpdatableSelected`）
  - "删除"按钮始终可用（只要有选中项）
  - `batchActions` 改为 `computed`，根据选中状态响应式更新 `disabled` 属性
- 选中状态指示（复刻 PCL2 MyLocalModItem.RectCheck，.vb 第 280-286 行）：
  - **不使用复选框图标**（太突兀），也**不覆盖原有启用/禁用状态色条**
  - 在列表项左边缘外侧挂一条 5px 宽的蓝色圆角竖条（`-left-1` 向左探出 4px），与 PCL2 `Margin=(-3,6,0,6)` 一致
  - 未选中：竖条不渲染，完全不影响原有状态色条
  - 选中：竖条上下留 6px（`top-1.5 bottom-1.5`），用 `transform: scaleY` 弹性动画（`cubic-bezier(0.34, 1.56, 0.64, 1)` 先冲到 1.15 再回弹到 1，对应 PCL2 AniEaseOutBack）
  - 选中时标题颜色变为主题强调色（`text-blue-600`，对应 PCL2 ColorBrush2）
  - 原有的启用/禁用状态色条保持不变，两者位置独立、互不干扰
- 多选操作栏布局（复刻 PCL2 CardSelect，第 59-77 行）：
  - **浮动在视口底部中央**（fixed bottom-6 left-1/2），不占据列表布局空间
  - 卡片分上下两部分：上方居中"已选择 X 项"文字，下方水平排列操作按钮
  - 入场动画：从下方滑入 + 淡入（对应 PCL2 的 TranslateTransform Y="-10" + Opacity）
  - 通过 `teleport to="body"` 确保浮在最上层，z-40 不遮挡弹窗（z-9999）
- 版本更新/更改功能：
  - 单个 Mod 列表项新增"更新"按钮（仅关联了平台工程的 Mod 显示）
  - 弹出版本选择对话框，查询 CurseForge/Modrinth 平台版本列表
  - 按游戏版本和加载器过滤，自动选中当前版本对应的筛选条件
  - 选择版本后下载安装到 mods 目录，自动删除旧版本文件（文件名不同时）
- 通用化设计（可复用于其他列表场景）：
  - 新建 `composables/useMultiSelect.ts`：泛型多选 composable，管理选中集合/Shift 范围选择/全选反选/ESC 清空，不涉及业务逻辑
  - 新建 `components/common/MultiSelectBar.vue`：通用批量操作栏组件，通过 `actions` prop 配置按钮（key/label/icon/variant），emit `action` 事件由调用方分发
- 新增文件：
  - `composables/useMultiSelect.ts`：通用多选 composable
  - `components/common/MultiSelectBar.vue`：通用批量操作栏组件
  - `views/version-settings/mod-tab/ModUpdateDialog.vue`：版本更新对话框（teleport + transition 自承载弹窗）
- 修改文件：
  - `views/version-settings/mod-tab/ModListItem.vue`：新增 `hasSelection`/`selected` props、`select` emit，点击切换选中，新增更新按钮
  - `views/version-settings/ModTab.vue`：集成通用 MultiSelectBar 和 ModUpdateDialog，定义 actions 配置 + action 事件分发
  - `composables/useModOperations.ts`：内部使用 useMultiSelect composable 管理选中状态，新增批量业务 handler（batchToggle/batchDelete/batchUpdate/openUpdateDialog）

#### Mod 详情版本 tag 自动切换修复
- 修复从 Mod 列表点击"详情"打开 ResourceDetail 时，不会自动切换到 mod 所在整合包的游戏版本 tag 的问题
- 根因：后端命令 `get_version_game_version` 已在 `commands/version/list.rs` 定义并标注 `#[tauri::command]`，但**未在 `lib.rs` 的 `invoke_handler` 中注册**，导致前端调用时报 `Command get_version_game_version not found`，`versionGameVersion` 始终为 `null`，传给 ResourceDetail 的 `gameVersion` 为 `undefined`，watch 中 `if (props.gameVersion)` 判断为 false，不执行 tag 自动切换
- 修复：在 `src-tauri/src/lib.rs` 的 `invoke_handler` 列表中添加 `commands::version::list::get_version_game_version` 注册

#### Mod 图标机制重构 + Mods 目录文件监听（参考 PCL2）
- **放弃 jar 解包提取 logo**，改用平台工程 `logo_url` + `image_cache` 缓存机制（与皮肤/披风一致），实现「几秒后图标自动加载出来」的体验
- 图标缓存机制（复用皮肤/披风 `image_cache::get_image_url`）：
  - 预加载查到 CF/MR 工程后，调用 `image_cache::get_image_url(project.logo_url, app)` 处理 logo URL
  - 命中缓存：返回 `cache-image://{hash}.png`，零网络请求，前端立即渲染
  - 未命中：返回远程 URL，后端异步下载，完成后 emit `image-cached` 事件通知前端刷新
  - 前端 `useModOperations` 监听 `image-cached` 事件，按 `cached_logo_url === remote_url` 匹配 mod 并原地替换为本地缓存 URL
  - 持久化缓存命中时从 `project.logo_url` 重新计算 `cached_logo_url`（image_cache 状态可能已变化）
- Mods 目录文件监听（参考 PCL2 PageInstanceMod FileSystemWatcher）：
  - 新增 `notify = "8"` crate 依赖
  - 新建 `commands/version/mods/watcher.rs`：`watch_mods_dir` / `unwatch_mods_dir` 命令
  - 使用 `notify::RecommendedWatcher` 监听 mods 目录文件创建/修改/删除
  - 500ms 静默期防抖：收到事件后等待无新事件才 emit，避免文件还在写入时过早刷新
  - 全局 `Mutex<Option<RecommendedWatcher>>` 持有当前 watcher，同一时间只有一个监听
  - watcher drop 时 channel 关闭，防抖线程自动退出
  - 通过 `mods-dir-changed` 事件通知前端
- 前端改动：
  - `composables/useModOperations.ts`：
    - 新增 `image-cached` 事件监听，异步下载完成后自动刷新 mod 图标 URL
    - 新增 `mods-dir-changed` 事件监听，文件变化时静默重载 mod 列表（`loadMods(true)` 不显示 spinner）+ 重新触发预加载
    - `loadMods(silent)` 重构：按 `enabled_name` 保存当前预加载数据（project / cached_logo_url / translated_name 等），重载后合并回去，避免文件变化触发重载时丢失已加载的工程信息
    - `init()` 中调用 `watchModsDir(selectedId)` 启动文件监听
    - `onUnmounted` 调用 `unwatchModsDir()` 停止监听，避免资源泄漏
  - `utils/api/personalization.ts`：新增 `watchModsDir` / `unwatchModsDir` API 封装
  - `views/version-settings/mod-tab/ModListItem.vue`：图标 `src` 从 `mod.logo_data` 改为 `mod.cached_logo_url || defaultModLogo`
  - `views/version-settings/mod-tab/ModUpdateDialog.vue`：同上，并新增 `@error` fallback 到默认图
- 后端改动：
  - `commands/version/mods/types.rs`：`ModInfo` / `ModMetadata` / `ModMeta` 移除 `logo_data` / `icon_path` / `logo_file` 字段
  - `commands/version/mods/metadata.rs`：删除 `extract_logo_data_url` / `guess_mime` 函数，移除 jar 内 logo 提取逻辑
  - `minecraft/community/preload/types.rs`：`PreloadUpdate` 的 `logo_data` 字段改为 `cached_logo_url`
  - `minecraft/community/preload/online_query.rs`：CF/MR 结果 emit 时填充 `cached_logo_url`（通过 `image_cache::get_image_url`）
  - `minecraft/community/preload/cache.rs`：`CachedMod` 的 `logo_data` 字段改为 `cached_logo_url`
  - `minecraft/community/preload/mod.rs`：缓存命中时从 `project.logo_url` 重新计算 `cached_logo_url`
  - `minecraft/community/preload/jar_metadata.rs`：JAR 元数据 emit 时 `cached_logo_url` 为 None（logo 仅来自平台工程）
  - `commands/version/mods/mod.rs`：新增 `pub mod watcher;` 模块声明
  - `lib.rs`：注册 `watch_mods_dir` / `unwatch_mods_dir` 命令

#### Mod 更新对话框底部状态栏优化
- 修复"发现新版本："标签无内容价值的问题（原逻辑仅做字符串相等比较，且只重复显示表格中已有的版本号）
- 新建 `utils/version.ts`：抽出 `compareVersion` 和 `versionChangeType` 工具函数（从 `useVersionGroups.ts` 抽出共享，消除重复代码）
- `useVersionGroups.ts`：移除内联 `compareVersion`，改为从 `@/utils/version` 导入
- `ModUpdateDialog.vue` 底部状态栏改为显示有价值的版本变化信息：
  - **升级**（绿色 ↑）：`1.2.0 → 1.3.0`，选中版本高于当前版本
  - **降级**（琥珀色 ↓）：`1.3.0 → 1.2.0`，选中版本低于当前版本（用户主动降级）
  - **当前版本**（灰色 ✓）：选中版本与当前版本相同
  - **已选择**（灰色）：当前 mod 版本未知时回退显示
  - 附加下载量信息（`formatDownloads`，如 `· 12.3 万次下载`），表格中未显示的有用信息
- 使用语义化版本比较（`compareVersion` 按 `.` 分段数字比较）替代字符串相等，正确识别 `1.2.0` 与 `1.2` 为同版本
- 后端版本号 fallback 链：JAR 元数据 → MANIFEST.MF → 文件名提取
  - `commands/version/mods/metadata.rs`：`finalize_metadata` 接收 `path` 参数，当 JAR 元数据 version 为空时调用 `extract_version_from_filename` 从文件名提取版本号
  - 新增 `extract_version_from_filename` 函数：去掉扩展名后，按 `+` 分隔符或最后匹配策略提取版本号
    - 有 `+` 时取 `+` 前面的第一个版本号（如 `alltheleaks-1.1.1+1.20.1-forge.jar` → `1.1.1`）
    - 无 `+` 时取最后一个版本号（如 `create-1.20.1-6.0.4.jar` → `6.0.4`）
  - 解决 Forge mod 的 `mods.toml` 中 `${file.jarVersion}` 占位符无法解析且 MANIFEST.MF 缺失时 version 为空的问题
- 前端 `ModUpdateDialog.vue`：当 `mod.version` 仍为空时显示"当前版本未知 → 选中版本"，明确告知用户无法判断升降级的原因

#### Mod 版本号识别链完整复刻 PCL2
- **根因**：之前只按顺序短路返回第一个找到的来源，且缺少 `fml_cache_annotation.json` 来源，导致部分 Forge mod 无法获取版本号
- **完整复刻 PCL2 `LocalResourceFile.LoadMetadataFromJar`**（`code-libs/PCL-main/.../LocalResourceFile.vb`）的 4 来源累积合并策略：
  1. `mcmod.info`（Forge 1.12-）
  2. `fabric.mod.json`（Fabric/Quilt，必须包含 `schemaVersion` 才视为有效）
  3. `META-INF/mods.toml`（Forge 1.13+/NeoForge）
  4. **`META-INF/fml_cache_annotation.json`（Forge 1.7-1.12 注解缓存，新增）**——查找 `@Mod` 注解，从 `values.version.value` 获取版本号
- **累积合并不覆盖策略**（参考 PCL2 的 Display/Description/Version setter）：
  - `MetaBuilder` 封装"已有有效值不覆盖"逻辑
  - `slug`：第一个非空值优先
  - `description`：第一个长度>2的值优先
  - `version`：第一个有效版本号（只含数字、点、减号）优先，占位符（包含 "version" 字样，如 `${file.jarVersion}`）标记为 `"version"`
- **`${file.jarVersion}` 占位符统一处理**：标记为 `"version"` 后，最后从 `META-INF/MANIFEST.MF` 的 `Implementation-Version` 解析（参考 PCL2 Finished: 标签第 314-329 行）
- **版本号有效性校验**：版本号必须包含 `.` 或 `-`，否则视为无效（参考 PCL2 第 330 行）
- 拆分为目录结构（文件超过 500 行按项目约定拆分）：
  - `metadata/mod.rs`：主入口 + `MetaBuilder` 合并器 + `finalize_metadata` + `extract_version_from_filename`
  - `metadata/sources.rs`：4 个来源的 `merge_*` 函数 + `read_manifest_version`

#### CurseForge 版本列表版本号修复
- **根因**：`curseforge/convert.rs` 的 `convert_version` 直接写 `version: String::new()`，注释"CurseForge 无版本号字段"，导致 CF 版本列表的 `ResourceVersion.version` 全为空字符串，前端 `versionChange` 计算永远走 `unknown` 分支
- **参考 PCL2**：阅读 `MyLocalModItem.xaml.vb` 第 298 行 `If(Entry.ProjectVersion.Version, Entry.ProjectVersion.Display)`，PCL2 对 CF 也是 `Version = Nothing`，但用 `Display`（即 `displayName`）作为 fallback
- **修复**：新建 `minecraft/community/version_extract.rs` 共享工具，从 `display_name` 提取版本号
  - CurseForge 的 `displayName` 通常类似 `jei-1.20.1-15.2.0.27.jar`，提取出 `15.2.0.27`
  - 有 `+` 分隔符时取 `+` 前面的版本号（如 `alltheleaks-1.1.1+1.20.1-forge.jar` → `1.1.1`）
  - 无 `+` 时取最后一个版本号（mod 版本号通常在 MC 版本号后面）
- `curseforge/convert.rs`：`convert_version` 调用 `version_extract::extract_version_from_name(&file.display_name)` 填充 `version` 字段
- `mods/metadata/mod.rs`：移除内联 `extract_version_from_filename`，改用共享的 `version_extract::extract_version_from_name`（消除重复代码）
- Modrinth 的 `version` 来自 API 的 `version_number` 字段，不需要 fallback

#### 版本变化提示视觉重做
- **问题**：底部状态栏的小徽章（text-xs）视觉上不够突出，和操作按钮挤在一起
- **改进设计**：参考 npm / VS Code 扩展更新的版本对比设计，从底部小徽章提升为独立的"双卡片+大箭头"对比区域
- 新建 `VersionChangeBadge.vue` 组件（100 行）：
  - 左侧：当前版本卡片（白底灰边，`当前` 标签 + 等宽字体版本号）
  - 中间：大箭头图标（w-5 h-5）+ 状态标签（升级↑/降级↓/同版本✓/切换→）
  - 右侧：选中版本卡片（彩色底+彩色边，颜色随状态变化：升级绿/降级琥珀/同版本灰/未知蓝）
  - 右侧：下载量（如有）
- 位置从底部状态栏移到 mod 信息卡片下方、过滤器上方，成为视觉焦点
- 底部状态栏简化为只保留取消/安装按钮（`justify-end`），更简洁
- `ModUpdateDialog.vue` 从 442 行降至 391 行（版本变化逻辑抽到独立组件）
- 移除不再使用的 `hasUnknownVersion` 计算属性和 `formatDownloads`/`ArrowUpIcon`/`ArrowDownIcon`/`CheckCircleIcon` 导入

#### 版本列表简化 + 回退版本变化 UI
- **版本列表表格简化**：隐藏"版本号"列，改为只显示"文件名"列（文件全称）
  - 之前同时显示 `ver.version`（版本号）和 `ver.file_name`（文件名），信息冗余
  - 现在只显示 `ver.file_name`，超长时 truncate 并保留原生 `title` tooltip
- **回退 VersionChangeBadge 双卡片设计**：用户反馈"还不如原来左下角那个"
  - 删除 `VersionChangeBadge.vue` 组件
  - 恢复左下角小徽章版本变化提示（彩色徽章 + 图标 + `当前版本 → 选中版本` + 下载量）
  - 简化未知版本的文案逻辑：`mod.version` 有值显示"已选择"，无值显示"当前版本未知"

#### 版本列表自定义 Tooltip 恢复 + 徽章视觉优化
- **恢复自定义 Tooltip**：版本列表文件名回退为原生 `title` 的问题修复
  - 重新导入 `Tooltip` 组件
  - 文件名长度超过 28 字符时才用 Tooltip（短文件名不需要，避免过度触发）
  - 添加 `cursor-help` 样式提示可悬停
- **左下角徽章视觉优化**：从矩形 `rounded-md` 改为胶囊式 `rounded-full`，更精致
  - 统一为单个胶囊容器，通过 `:class` 动态切换背景色和边框色（升级绿/降级琥珀/同版本灰/未知蓝）
  - 旧版本号用删除线 + 灰色（表示将被替换），新版本号用彩色加粗高亮
  - 同版本时不显示旧版本和箭头，只显示 `✓ + 版本号`
  - 未知状态用小圆点替代图标，视觉更轻量

#### 移除版本更新弹窗的筛选器
- **问题**：更新/更换 mod 版本时，游戏版本和模组加载器由当前整合包决定，用户不可能切换，筛选框是多余的
- **移除**：删除模板中的游戏版本 Select、加载器 Select、"全部"复选框
- **保留自动筛选逻辑**：`filteredVersions` 仍按当前整合包的 MC 版本（`props.mcVersion`）和加载器（`modLoaderType`）自动筛选
- 移除不再使用的 `Select` 组件导入、`gameVersionOptions`、`loaderOptions`、`showAllVersions`
- 简化 `loadVersions` 中的自动选中逻辑：直接用 `props.mcVersion` 和 `modLoaderType`，不再检查 `gameVersionOptions.includes`

#### Fabric/Forge 启动 ClassNotFoundException 修复
- **根因**：`build_classpath` 只读取当前版本 JSON 的 `libraries`，不递归合并父版本的 libraries
  - Fabric 版本 JSON 有 `inheritsFrom`，其 `libraries` 只包含 Fabric Loader 相关库
  - 原版库（lwjgl、netty 等）来自父版本 JSON
  - 不递归合并导致 classpath 缺失 Fabric Loader 库，启动时报 `ClassNotFoundException: net.fabricmc.loader.impl.launch.knot.KnotClient`
- **修复**：参考 PCL2 `McLibListGet`，新增 `collect_libraries_recursive` 递归合并父版本 libraries
  - 遍历 `inheritsFrom` 链，把所有层级的 libraries 合并到一起
  - 子版本 libraries 排在前面（优先级更高）
  - 循环继承检测（防止 `inheritsFrom` 成环导致死循环）
  - 使用 `HashSet` 去重，避免同一 jar 被重复加入 classpath
- **崩溃分析器增强**：`crit1.rs` 新增 `ClassNotFoundException` 规则
  - 提取缺失的类名（支持跨行匹配）
  - 根据类名智能判断具体原因：
    - 类名含 `fabric`/`knot` → "Fabric 加载器库缺失"
    - 类名含 `forge`/`fml`/`modlauncher` → "Forge 加载器库缺失"
    - 类名含 `neoforge` → "NeoForge 加载器库缺失"
    - 其他 → "Java 类缺失" + 具体类名
  - 给出"重新安装该版本的加载器"的建议

#### 启动时文件检查速度优化（60 秒 → 0.5 秒）
- **根因**：`find_missing_libs` 对每个 lib **串行**调用 `FileChecker.is_valid`，其中哈希校验会读取整个文件并计算 SHA1。73 个 lib（约 200MB）串行计算哈希非常慢，导致每次启动卡 1 分钟
- **参考 PCL2**：阅读 `ModLaunch.vb` 第 1705 行，PCL2 启动时构建 classpath 只调用 `McLibListGet` 获取路径列表，**不做任何文件校验和哈希检查**。文件校验和下载在安装阶段做，启动时不重复校验
- **优化方案**：新增 `quick_check` 参数区分两种场景：
  - **快速检查模式**（`quick_check = true`，启动时）：只检查文件存在 + 大小匹配，不计算 SHA1
    - 用于 `fix_version_files` 经 `validate_and_fix_files` 调用
    - 文件下载时已经做过完整校验，正常情况下不会损坏
  - **完整校验模式**（`quick_check = false`，下载时）：计算 SHA1 哈希，确保文件完整性
    - 用于 `download_version_full` 的版本安装/修复
- **并行化**：使用 `std::thread::scope` 并行检查多个库文件
  - 按 CPU 核心数分线程（`available_parallelism`），按索引取模分配库文件
  - 使用 `Mutex<Vec<LibEntry>>` 收集结果，最后按原顺序排序
- `find_missing_libs` 新增 `quick_check` 参数
- `download_libraries` 新增 `quick_check` 参数透传
- `fix.rs`（启动时）传 `true`，`full_download.rs`（下载时）传 `false`
- 日志增加模式标识：`[Libraries] Total: 73, Missing: 7 (mode: quick)`
- **预期效果**：73 个库的检查时间从约 60 秒降至约 0.5 秒（120 倍提速）

#### Assets 文件检查速度优化（启动卡顿的真正元凶）
- **根因**：上一次优化只修了 `find_missing_libs`，但 `find_missing_assets` 仍然是**串行 + 完整哈希校验**。Assets 通常有几百上千个文件（音效、纹理等），串行计算 SHA1 是启动卡 44 秒的真正元凶
- **优化**：对 `find_missing_assets` 应用与 `find_missing_libs` 相同的优化方案
  - 新增 `quick_check` 参数，快速检查模式只检查文件存在 + 大小匹配
  - 使用 `std::thread::scope` 并行检查，按 CPU 核心数分线程
  - 新增 `quick_check_asset` 辅助函数
- `download_assets` 新增 `quick_check` 参数透传
- `fix.rs`（启动时）传 `true`，`full_download.rs`（下载时）传 `false`
- 日志增加模式标识：`[Assets] Total: 580, Missing: 12 (mode: quick)`
- **预期效果**：assets 检查时间从约 40 秒降至约 1 秒（40 倍提速）

#### Fabric 启动 ClassNotFoundException 真正根因修复：主 jar 未下载
- **根因**：`fix_version_files`（启动时文件补全）只调用了 `download_libraries` 和 `download_assets`，**缺少 `download_client_jar`**！导致主 jar 文件（如 `26.2-Fabric0.19.3.jar`）不存在，启动时 classpath 缺失主 jar
- **日志证据**：`[Classpath] Warning: Main jar not found: ...26.2-Fabric0.19.3.jar`
- **修复**：在 `fix.rs` 中添加 `download_client_jar` 调用，作为文件补全的第 1 步
  - 用 `merged_json` 获取 `downloads.client` 字段（支持 inheritsFrom，父版本的 client jar 信息会合并进来）
  - 主 jar 下载失败不中断流程（某些特殊版本可能没有 client jar，用 `log_info` 记录后继续）
- **`download_client_jar` 修复**：用 `find_original_version` 确定主 jar 的正确路径
  - 之前直接用传入的 `version_id` 构造路径，对于有 `inheritsFrom` 的版本会下载到错误位置
  - 有 `inheritsFrom` 时主 jar 应在父版本目录下（如 Fabric 版本的主 jar 在原版目录 `versions\26.2\26.2.jar`）
  - 无 `inheritsFrom` 时主 jar 在当前版本目录下
  - `find_original_version` 改为 `pub(crate)`，`classpath` 模块改为 `pub(crate) mod`
- **关于"卡 41 秒"的澄清**：这不是检查慢，而是下载 7 个缺失库的网络耗时
  - 检查本身是秒过的（quick 模式生效，日志 `mode: quick`）
  - `[Libraries] Total: 73, Missing: 7` 打印后，到 `[Assets]` 之间的 36 秒是在下载那 7 个缺失的库
  - 下载耗时取决于网络速度和文件大小，属正常现象

#### 崩溃分析结果前端无提示修复 + PCL2 风格崩溃弹窗
- **根因**：`launch` 命令在 `pipeline.execute().await` 返回 `Err` 时（如 `ClassNotFoundException` 致命错误），通过 `?` 直接返回 Err，**后面的 `tokio::spawn` 监听 `exit_rx` 退出事件的任务永远不会被创建**。所以 `game-exited` 事件永远不会发送，前端收不到崩溃信息
- **后端修复**：`launch.rs` 捕获 `LaunchProcess` 阶段的失败，等待 watcher 完成崩溃分析后手动发送 `game-exited` 事件
  - 只对 `LaunchProcess` 阶段的失败做崩溃分析（`GetJava`/`Login` 等阶段失败不需要）
  - 等待 `exit_rx` 最多 15 秒，避免无限等待
  - 如果崩溃分析无结果，构造基本的 `CrashInfo`（用 `launch_err.message` 作为 reason）
  - 清理启动状态后发送 `game-exited` 事件，让前端展示崩溃对话框
- **前端优化**：`CrashDialog.vue` 参考 PCL2 `MyMsgText` 风格优化
  - 参考 PCL2 `MyMsgText.xaml`：
    - 浅灰白底 `#FBFBFB`（PCL2 的 `Background="#FBFBFB"`）
    - 圆角 `rounded-lg`（PCL2 的 `CornerRadius="7"`）
    - 标题下方加 2px 分割线（PCL2 的 `ShapeLine`，与标题同色）
    - 遮罩半透明黑色 `bg-black/40`（PCL2 的 `RGBA(90,0,0,0)`）
  - 参考 PCL2 `MyMsgText.xaml.vb` 进入动画：
    - 透明度 0→1（120ms）
    - Y 偏移 40→0（300ms，回弹缓动 `cubic-bezier(0.34, 1.56, 0.64, 1)`）
    - 关闭时下沉 20px + 淡出
  - Transition 名从 `modal` 改为 `crash-modal`，添加 scoped 样式

#### Fabric 库下载失败根因修复 + CrashDialog 报错修复 + PCL2 风格重做
- **Fabric 库 size/sha1 读取修复**（`libraries.rs` `parse_libraries`）
  - **根因**：Fabric 版本 JSON 的库格式与 Mojang 不同：
    ```json
    { "name": "org.ow2.asm:asm:9.10.1", "sha1": "...", "size": 126151, "url": "https://maven.fabricmc.net/" }
    ```
    size 和 sha1 在**根级别**，不在 `downloads.artifact` 里
  - 之前没有 `downloads.artifact` 时，else 分支直接设 `size=0, sha1=None`，导致：
    - `find_missing_libs` 快速检查时 `size=0` 只检查文件存在，文件不存在就标记为缺失
    - 下载时 `expected_size=0` 无法校验
    - 每次启动都缺失 7 个 Fabric 库（fabric-loader、asm、sponge-mixin 等）
  - **修复**：else 分支从根级别读取 `size` 和 `sha1`
- **下载日志可见性修复**（`downloader.rs`）
  - 之前关键日志用 `log_debug`，默认不输出，导致看不到下载过程
  - 改为 `log_info`：开始下载时输出 `local_path`、`size`、`urls`
  - 跳过已存在文件时输出日志
  - 下载失败时输出尝试的 URL 数量
- **CrashDialog `log_tail` undefined 报错修复**（`types.rs`）
  - **根因**：`CrashInfo.log_tail` 有 `#[serde(skip_serializing_if = "Vec::is_empty")]`，空时序列化跳过该字段，前端收到 `undefined`
  - **修复**：移除 `skip_serializing_if`，只保留 `default`，确保字段始终序列化
  - `log_lines` 也加 `default` 防御
  - 前端 `CrashDialog.vue` 加防御性处理：`crashInfo.value?.log_lines ?? []`
- **CrashDialog 重做为 PCL2 风格**（严格参考 `MyMsgText.xaml`）
  - 标题字号 23px（PCL2 `LabTitle FontSize=23`）
  - 标题下方 2px 分割线（PCL2 `ShapeLine`，与标题同色 `bg-gray-700/80`）
  - 内容字号 15px（PCL2 `LabCaption FontSize=15`）
  - 文字颜色 `#5C5C5C`（PCL2 `LabCaption Foreground="#FF5C5C5C"`）
  - 去掉"崩溃原因""建议"等小标题卡片，改为 PCL2 风格的纯文本段落（参考 `GetAnalyzeResult` 输出）
  - 浅灰白底 `#FBFBFB`，圆角 `rounded-lg`（PCL2 `CornerRadius="7"`）
  - 按钮 3 个右对齐：查看输出 / 导出错误报告 / 确定（PCL2 `PanBtn`）
  - 进入动画：透明度 0→1（120ms）+ Y 偏移 40→0（300ms 回弹缓动）

#### Fabric 库 URL 拼接修复 + CrashDialog 严格复刻 PCL2 配色
- **URL 拼接缺斜杠修复**（`libraries.rs` `root_url` 构造）
  - **根因**：`format!("{}{}", u.trim_end_matches('/'), path)` 把 URL 结尾的 `/` 去掉后直接拼接，导致 `https://maven.fabricmc.net/` + `org/ow2/asm/...` 变成 `https://maven.fabricmc.netorg/ow2/asm/...`（缺少斜杠）
  - **修复**：改为 `format!("{}/{}", u.trim_end_matches('/'), path)`，用 `/` 连接
- **parse_libraries 读取 Fabric 格式 size/sha1**（之前修改未保存，重新修复）
  - Fabric 版本 JSON 的库格式：`{ "name": "...", "sha1": "...", "size": 126151, "url": "..." }`
  - size 和 sha1 在根级别，不在 `downloads.artifact` 里
  - else 分支从根级别读取 `library["size"]` 和 `library["sha1"]`
- **CrashDialog 严格复刻 PCL2 配色**（参考 `MyMsgText.xaml` + `Application.xaml`）
  - 在 `tailwind.config.js` 添加 PCL2 颜色系：
    - `pcl-1`=`#343d4a`（深灰蓝，正文/默认文字/阴影）
    - `pcl-2`=`#0b5bcb`（主蓝，标题/Highlight 按钮）
    - `pcl-3`=`#1370f3`（亮蓝，悬停态边框）
    - `pcl-7`=`#e0eafd`（按钮悬停背景）
    - `pclmsg-bg`=`#FBFBFB`（弹窗背景）
    - `pclmsg-caption`=`#5C5C5C`（正文文字，写死不随主题变）
  - 弹窗配色严格对应 PCL2：
    - 标题 `text-pcl-2`（`#0b5bcb`），字号 23px
    - 分割线 `bg-pcl-2`（与标题同色，高 2px）
    - 正文 `text-pclmsg-caption`（`#5C5C5C`），字号 15px，行高 18px
    - 背景 `bg-pclmsg-bg`（`#FBFBFB`）
    - 阴影 `shadow-[0_4px_20px_rgba(52,61,74,0.5)]`（PCL2 DropShadowEffect）
    - 遮罩 `bg-black/35`（PCL2 `rgba(0,0,0,0.353)`）
  - 按钮配色参考 PCL2 MyButton 三态：
    - 确定按钮（Highlight 态）：边框 `border-pcl-2`，文字 `text-pcl-2`，hover 变 `pcl-3` + 背景 `pcl-7`
    - 查看输出/导出按钮（Normal 态）：边框 `border-pcl-1`，文字 `text-pcl-1`，hover 同上
    - 按钮背景 `bg-white/30`（PCL2 `ColorBrushHalfWhite #55ffffff`）
    - 圆角 `rounded`（PCL2 `CornerRadius=3`）
    - 过渡 `duration-100`（PCL2 颜色过渡 100ms）

#### Mod 更新对话框 UI 重做
- **版本变化徽章视觉突出**：底部状态栏从纯文字改为彩色徽章卡片样式（参考 `Alert` 组件的色块结构）
  - 升级：绿色徽章（`bg-green-50 border-green-200`）+ ↑ 图标 + `1.2.0 → 1.3.0` 等宽字体
  - 降级：琥珀色徽章（`bg-amber-50 border-amber-200`）+ ↓ 图标 + 版本号
  - 同版本：灰色徽章 + ✓ 图标 + 版本号
  - 未知/已选择：蓝色徽章 + `→` 箭头 + 选中版本号
  - 版本号使用 `font-mono` 等宽字体，对齐数字和 `.` 便于阅读
- **按钮样式重做**（对齐项目 `Modal` / `DownloadPanel` 组件的配色约定）：
  - 取消按钮：`text-gray-600 hover:bg-gray-200`（移除 border，与 Modal 次按钮一致）
  - 安装按钮：`bg-primary-600 hover:bg-primary-700`（从 `bg-blue-500` 改为项目主色 `primary-600`）
  - 禁用状态添加 `disabled:hover:bg-primary-600` 防止 hover 覆盖禁用样式
- **自定义 Tooltip 替换原生 title**：
  - 版本列表中文件名的 `:title="ver.file_name"` 改为 `<Tooltip>` 组件
  - 添加 `cursor-help` 样式提示可悬停
  - 使用 200ms 延迟避免快速划过时频繁触发

### 修复

#### 下载阶段重复显示
- `state/download.rs`：`DownloadState::default()` 不再预填充 5 个阶段，改为空列表
- 原因：`install_merged` 的 `append_stages` 会在默认阶段之上追加，导致出现两组重复的阶段（第一组永远停留在 waiting 状态，图标不更新）

#### 阶段快照缺少 group 字段
- `commands/version/types.rs`：`DownloadStageSnapshot` 新增 `group: Option<String>` 字段
- `commands/version/progress.rs` 和 `download.rs`：快照映射时填充 `group` 字段
- 原因：前端 `useDownloadPolling.ts` 读取 `s.group` 但后端未返回，导致所有阶段显示为独立项而非按分组折叠

#### Fabric API 安装导致前端提前退出
- `commands/version/install/mod.rs`：将 `install-complete` 事件从加载器安装后移至 `mark_complete()` 之后
- 原因：原代码在 Fabric API 安装前就发出 `install-complete` 事件并调用 `mark_complete()`，导致前端轮询检测到 `is_complete=true` 后关闭进度面板

#### 启动高级选项（参考 PCL2 PageSetupLaunch）
- 新增 3 个启动高级选项，位于"启动设置"页面底部：
  - **禁用 Java Launch Wrapper**：JLW 用于修复 Java 18- 在中文路径下可能无法正常启动的问题
  - **禁用 LWJGL Unsafe Agent**：LUA 用于修复 LWJGL 3.4.1 的性能问题，通过 `-javaagent` 参数注入 `lwjgl-unsafe-agent.jar`
  - **使用高性能显卡**：自动在 Windows 设置中将 Java 改为使用独立显卡
- 从 PCL2 资源文件夹复制 `lwjgl-unsafe-agent.jar` 到 `src-tauri/resources/`，注册为嵌入资源
- 后端改动：
  - `state/config.rs`：`AppConfig` 新增 `launch_disable_jlw` / `launch_disable_lua` / `launch_use_dedicated_gpu` 三个字段
  - `commands/system/apply_config.rs`：`ConfigPatch` 和 `ConfigSnapshot` 同步新增三个字段
  - `config.rs`：INI 加载/保存支持 `[Launch]` 段的三个字段
  - `resources/defaults/config.ini`：新增 `[Launch]` 段默认值
  - `minecraft/launch/mod.rs`：
    - 新增 `resolve_lwjgl_agent()` 函数，从缓存目录释放 `lwjgl-unsafe-agent.jar`
    - `build_jvm_args()` 接入 `disable_lua` 参数，未禁用时添加 `-javaagent` 参数
  - `minecraft/launch/pipeline.rs`：`LaunchConfig` 新增 `disable_jlw` / `disable_lua` 字段
  - `commands/version/launch.rs`：从全局配置读取 `disable_jlw` / `disable_lua` 传入 `LaunchConfig`
- 前端改动：
  - `utils/api/config.ts`：`ConfigSnapshot` 和 `ConfigPatch` 新增三个字段（camelCase）
  - `views/settings/SettingsLaunch.vue`：新增"高级选项"卡片，3 个开关切换器，watch 自动保存

#### 通用图片缓存组件（方案 C：混合缓存 + 自定义 URI scheme）
- 背景：前端直接用 `<img :src="remoteUrl">` 加载远程图片，Tauri webview 的 HTTP 缓存仅会话内有效，重启应用后需重新下载
- **安全设计**：不使用 Tauri 的 asset protocol（会暴露完整本地文件路径，存在恶意读取风险），改用自定义 URI scheme `cache-image://{hash}.png`，前端只能通过 hash 请求，后端验证 hash 合法性后返回文件内容
- 新增 `src-tauri/src/minecraft/image_cache.rs` 通用图片缓存组件：
  - 首次加载：返回远程 URL，前端立即渲染；后端 `tokio::spawn` 异步下载到本地缓存
  - 二次加载：返回自定义 URI scheme URL（`cache-image://{hash}.png`），零网络请求
  - 缓存 key：URL 的 SHA1 hash，URL 变化时自动失效
  - 下载完成：emit `image-cached` 事件通知前端刷新
  - 并发去重：`IN_FLIGHT` Mutex<HashSet> 避免同一 URL 重复下载
  - `parse_hash_from_request()`：从 URI 解析 hash 并验证格式（40 位十六进制）
  - `find_cache_by_hash()`：根据 hash 查找缓存文件路径
  - 提供 `get_image_url()` / `invalidate()` / `clear_all()` 三个公开方法
- `lib.rs` 注册 `register_uri_scheme_protocol("cache-image", ...)` 处理器：
  - 从请求 URI 提取 hash，验证格式（防止路径遍历）
  - 查找缓存文件，返回 `image/png` 内容
  - 无效请求返回 403，文件不存在返回 404
  - 响应头设置 `Cache-Control: public, max-age=86400` 减少重复请求
- `storage/cache.rs` 新增 `read_bytes()` / `write_bytes()` 二进制读写方法
- `minecraft/mod.rs` 注册 `pub mod image_cache`
- Tauri 配置更新：
  - `tauri.conf.json` CSP 放行 `cache-image:` scheme（不含 asset: 和 asset.localhost）
  - 不启用 `assetProtocol`（避免暴露本地文件路径）
  - 不添加 `protocol-asset` feature（无需 asset protocol）
- 前端新增 `src/composables/useImageCache.ts`：
  - `useImageCache(targetRef, expectedRemoteUrlRef)`：自动监听事件并刷新指定 ref
  - `onImageCached(callback)`：回调式监听，适合自定义刷新逻辑
  - 组件卸载时自动 unlisten
- 皮肤/披风管理接入图片缓存：
  - `commands/skin.rs`：`get_skin_url` / `get_cape_url` 返回 `CachedImage` 结构体（含 url 和 cached 字段），上传皮肤/装备取消披风后显式失效旧缓存
  - `skin.ts`：`getSkinUrl` / `getCapeUrl` 返回 `CachedImage | null`
  - `SkinManager.vue`：监听 `image-cached` 事件刷新 skinUrl/capeUrl
  - `SkinAvatar.vue`：记录 `currentRemoteUrl`，事件匹配后重新加载头像
  - `SkinCapeList.vue`：直接使用 `cape.cached_url` 加载图片（由 `getSkinCapeInfo` 返回时填充），无需额外调用 `getCachedImageUrl`，减少一轮 IPC 调用
- 通用图片缓存命令（独立组件，可复用于任意远程图片场景）：
  - 新增 `src-tauri/src/commands/image_cache.rs`：
    - `get_cached_image_url(url)`：接受任意远程 URL，返回 `CachedImage`
    - `invalidate_cached_image(url)`：失效指定 URL 缓存
    - `clear_image_cache()`：清空所有图片缓存
  - 新增 `src/utils/api/image-cache.ts`：前端 API 封装（`getCachedImageUrl` / `invalidateCachedImage` / `clearImageCache`）
  - `tauri.ts` 导出 `image-cache` 模块
  - 披风列表 `SkinCapeList.vue` 改用 `getCachedImageUrl` 获取缓存 URL 再裁剪图标，二次加载零网络

### 重构

#### 新增 storage/cache.rs 缓存组件，统一缓存路径管理
- 背景：缓存文件散落在各处，`forge_installer.rs` 使用 `std::env::temp_dir().join("MoLaunch").join("Cache")`，`preload.rs` 手动拼接 `"cache/preload_mods/"` 前缀调用 `Storage.read_file/write_file`，路径管理不统一
- 新增 `src-tauri/src/storage/cache.rs`：
  - `Cache` 结构体（全局单例 + OnceLock 懒加载），缓存根目录由 `Storage::cache_dir()` 提供
  - 提供 `dir()` / `path()` / `ensure_dir()` / `exists()` / `read()` / `write()` / `remove()` / `list()` / `clear_dir()` 共 9 个方法
  - 所有方法接受相对于缓存根目录的路径，自动拼接，调用方无需手动构造 `"cache/"` 前缀
- `storage/mod.rs` 注册 `pub mod cache;`
- 迁移调用方：
  - `forge_installer.rs`：删除 `get_cache_dir()` 函数，改用 `Cache::instance().ensure_dir("forge_installer")`，资源文件从系统临时目录迁移到 `.Molaunch/cache/forge_installer/`
  - `preload.rs`：`load_file_cache` / `save_file_cache` 从 `Storage::instance().read_file("cache/preload_mods/...")` 改为 `Cache::instance().read("preload_mods/...")`，去掉手动 `cache/` 前缀
  - `developer.rs`：`get_storage_dirs` 命令的 `cache` 字段从 `storage.cache_dir()` 改为 `Cache::instance().dir()`

### 优化

#### 皮肤/披风管理：后端改为返回 URL，前端直接加载图片
- 背景：原实现中后端 `download_skin_png` 和 `download_cape_png` 命令会下载 PNG 二进制数据并 base64 编码后返回前端，存在不必要的 base64 编解码开销和 IPC 传输浪费
- 后端改动（`src-tauri/src/commands/skin.rs`）：
  - `get_skin_url` 新增可选 `uuid` 参数，支持查询非当前登录用户的皮肤 URL（原 `download_skin_png` 的 uuid 查找逻辑合并至此）
  - 新增 `get_cape_url` 命令，返回当前已装备披风的下载 URL（替代原 `download_cape_png`）
  - 删除 `download_skin_png` 和 `download_cape_png` 命令（不再需要后端下载+base64）
  - `save_data_url_to_file` 改为 `download_url_to_file(url, path)`，后端直接从 URL 下载并写入文件，避免 base64 中转
- 后端注册表更新（`src-tauri/src/lib.rs`）：移除 `download_skin_png`/`download_cape_png`/`save_data_url_to_file`，注册 `get_cape_url`/`download_url_to_file`
- 前端 API 层（`src/utils/api/skin.ts`）：
  - `getSkinUrl` 新增可选 `uuid` 参数
  - 新增 `getCapeUrl` 函数
  - 删除 `downloadSkinPng`、`downloadCapePng`、`saveDataUrlToFile`
  - 新增 `downloadUrlToFile(url, path)` 函数
- 前端组件改动：
  - `SkinManager.vue`：`skinDataUrl`/`capeDataUrl` 重命名为 `skinUrl`/`capeUrl`，调用改为 `getSkinUrl()`/`getCapeUrl()`，保存皮肤改用 `downloadUrlToFile`
  - `SkinAvatar.vue`：微软账号皮肤加载改用 `getSkinUrl(uuid)` 获取 URL，直接传给 `Image` 加载（离线账号默认皮肤仍使用 Vite 本地 URL，无 CORS 问题）
  - `SkinCapeList.vue`：披风列表从 SVG 占位图标改为显示真实披风 PNG 图片（详见下方「披风列表显示真实图片」条目）

#### 披风列表显示真实披风图标
- 背景：原 `SkinCapeList.vue` 对所有披风使用统一的 SVG 占位图标（对勾/方块），无法直观区分不同披风；MC 服务器返回的披风 PNG 是完整正背面纹理图，直接显示会过大且包含无关内容
- 新增工具函数（`src/utils/cape-icon.ts`）：
  - `getCapeIcon(capeUrl)`：根据 skinview3d 的 CapeObject UV 映射，裁剪披风外侧可见图案（front 面，纹理坐标 (1,1) 起 10x16 区域）作为图标
  - 参考 skinview3d 源码（`node_modules/skinview3d/libs/model.js` 的 `setCapeUVs` 函数）确认正确的纹理坐标
  - 输出 canvas 尺寸 10x16（原始尺寸，由 CSS 控制显示大小）
  - 支持高清披风纹理（根据图片宽度自动计算 scale，标准 64x32 scale=1，高清 128x64 scale=2）
  - 使用 `imageSmoothingEnabled = false` 保持像素风格
- 改动（`src/components/common/skin-manager/SkinCapeList.vue`）：
  - 引入 `getCapeIcon`，为每个披风加载图标 dataURL 存入 `iconMap`
  - `watch` 监听 `capes` 变化自动重新加载图标（`immediate: true, deep: true`）
  - 加载失败的披风 id 记入 `failedIds`，回退到灰色占位 SVG
  - 布局从横排改为纵排卡片（图标在上、名称在下），网格 3-6 列
  - 图标使用 `image-rendering: pixelated` 保持 Minecraft 像素风格
  - 已装备披风保留对勾角标 + 主题色高亮边框

### 新增

#### 开发者模式功能
- 触发方式（两者结合）：
  1. 在「其他」页的应用版本号上连续点击 5 次（1.5 秒内）解锁开发者模式
  2. 解锁后「高阶配置」顶部显示「开发者模式」开关卡片（关闭/开启 2 按钮样式）
  3. 开关开启后「设置」侧边菜单末尾追加「开发者」菜单项
- 存储位置：Windows 注册表 `HKCU\Software\MoLaunch` 下的两个布尔值
  - `DeveloperUnlocked`：是否已解锁（决定开关卡片是否显示）
  - `DeveloperMode`：开关是否开启（决定侧边菜单 developer 项是否显示）
- 开发者页（SettingsDeveloper.vue）4 个卡片：
  - 日志：日志文件下拉选择 + 黑底等宽字体内容预览（最高 384px 滚动）+ 刷新按钮 + 打开日志目录按钮
  - 缓存：缓存目录/临时目录路径展示 + 各带「打开」按钮
  - 存储信息：数据根目录/配置文件/日志目录路径展示 + 各带「打开」按钮
  - 系统信息：应用版本/操作系统/架构/位数/总内存/已用内存/可用内存/内存使用率

#### 后端：抽象注册表模块到 storage 子目录
- 现象：注册表操作（reg_key/reg_get/reg_set/reg_delete）原本位于 `minecraft/auth/storage/registry.rs` 内为 `pub(super)`，仅 AuthStorage 可用，新开发者模式无法复用
- 修复：创建 `src-tauri/src/storage/registry.rs` 子模块，包含 `pub(crate)` 可见性的 `reg_key/reg_get/reg_set/reg_delete` + 新增高层便捷 API `reg_get_bool/reg_set_bool`（布尔值存取）；`minecraft/auth/storage/registry.rs` 仅保留认证专用键名常量；`minecraft/auth/storage/mod.rs` 改用 `crate::storage::registry::{reg_key, reg_get, reg_set, reg_delete}`；`storage/mod.rs` 新增 `pub mod registry;` 声明
- 非 Windows 平台桩实现保留（保证跨平台编译通过）

#### 后端：新增 commands/system/developer.rs
- 6 个 Tauri 命令：
  - `is_developer_unlocked` - 查询开发者模式是否已解锁
  - `unlock_developer_mode` - 解锁开发者模式（连续点击版本号 5 次后调用）
  - `is_developer_mode` - 查询开发者模式是否开启
  - `set_developer_mode` - 设置开关（仅在已解锁时可生效）
  - `get_storage_dirs` - 返回 `{ base, config, logs, cache, temp }` 路径
  - `get_system_info` - 返回应用版本/OS/架构/内存等系统信息

#### 后端：暴露日志命令（logger.rs）
- 原函数 `get_log_path` / `list_log_files` / `read_log_file` 已存在但未加 `#[tauri::command]`，前端无法调用
- 修复：原函数重命名为 `_inner` 后缀，新增 3 个 `#[tauri::command]` 包装：
  - `get_log_path` - 返回今日日志文件完整路径（String）
  - `list_log_files` - 返回日志文件名列表（最新的在前）
  - `read_log_file` - 读取指定日志文件内容（带路径遍历防护：禁止 `/` `\` `..`，仅允许 `.log` 后缀）

#### 前端：新增 src/utils/api/developer.ts
- 封装 9 个 invoke 调用 + `StorageDirs` / `SystemInfo` TypeScript 类型
- 通过 `src/utils/tauri.ts` re-export，复用现有 `import * as tauri from '@/utils/tauri'` 入口

#### 前端：版本号注入与开发者模式触发
- `vite.config.ts` 新增 `define: { __APP_VERSION__: JSON.stringify(pkg.version) }`，从 package.json 读取版本号注入
- `src/vite-env.d.ts` 新增 `declare const __APP_VERSION__: string` 类型声明
- `SettingsOther.vue` 在「配置信息」卡片末尾新增「应用版本」行（可点击），连续点击 5 次解锁开发者模式，Toast 提示剩余次数
- 版本号行使用项目自定义 Tooltip 组件（position=right, delay=200ms）显示「连续点击 5 次解锁开发者模式」提示，替代原生 title 属性

#### 前端：SettingsAdvanced.vue 开发者模式开关卡片
- 仅在 `devUnlocked === true` 时显示（`v-if="devUnlocked"`），位于「高阶配置」顶部
- 「已开启/已关闭」2 按钮样式（与 CurseForge API Key 启用开关一致）
- 切换时调用 `setDeveloperMode`，成功后派发 `window.dispatchEvent(new CustomEvent('developer-mode-changed', { detail: v }))` 通知父组件

#### 前端：Settings.vue 侧边菜单条件渲染
- `baseCategories` 数组（5 项基础菜单）+ `developerCategory`（开发者菜单项）
- `categories` computed：`devModeEnabled` 为 true 时追加 developer 项到末尾
- `onMounted` 时读取 `isDeveloperMode()` 初始化状态 + 监听 `developer-mode-changed` 事件实时更新
- 关闭开发者模式时若当前停留在 developer 分类，自动切回「其他」避免空白页
- `onUnmounted` 清理事件监听

#### 前端：新建 SettingsDeveloper.vue
- 4 个卡片（重构后日志卡片已抽到 LogViewer.vue，详见下方「前端重构」条目）：
  - 日志：日志文件 Select 下拉（复用项目自定义组件）+ 黑底等宽字体 pre 内容预览 + 刷新按钮 + 打开日志目录按钮
  - 缓存：缓存目录/临时目录路径展示 + 各带「打开」按钮
  - 存储信息：数据根目录/配置文件/日志目录路径展示 + 各带「打开」按钮
  - 系统信息：应用版本/操作系统/架构/位数/总内存/已用内存/可用内存/内存使用率 8 项
- 字节数格式化（KB/MB/GB）+ 操作系统/架构友好显示名（windows→Windows / x86_64→x64 (64-bit) 等）

### 重构

#### 前端重构：拆分 SettingsDeveloper.vue / SettingsAdvanced.vue（300 行规范）
- 现象：开发者模式功能落地后两个 Vue 文件超过项目 300 行规范——
  - `SettingsDeveloper.vue` 385 行（含 formatBytes 重复实现 / osDisplay / archDisplay / LogLine 接口 / parseLogLines / logLineClass 等可复用函数内联在组件中）
  - `SettingsAdvanced.vue` 303 行（含开发者模式开关卡片的模板与脚本逻辑与代理/CurseForge 配置混杂）
- 修复一：抽取可复用纯函数到 TS 工具文件（参考现有 `utils/format.ts` / `utils/mod-display.ts` 模式）
  - 新建 `src/utils/system-display.ts`（23 行）：`osDisplay` / `archDisplay`，将后端原始 os/arch 字符串映射为本地化显示名
  - 新建 `src/utils/log-display.ts`（50 行）：`LogLine` 接口 + `parseLogLines`（按 `[LEVEL]` 正则解析为带行号与级别的行数组）+ `logLineClass`（按业界惯例返回级别文字颜色 class，ERROR→红/WARN→黄/INFO→绿/DEBUG→青/TRACE→暗灰）
  - `formatBytes` 已在 `src/utils/format.ts` 存在（LaunchLog.vue 已在用），原 SettingsDeveloper.vue 内的重复实现删除，改为 `import { formatBytes } from '@/utils/format'`
- 修复二：抽取日志查看卡片为自包含子组件
  - 新建 `src/components/settings/LogViewer.vue`（197 行）：包含日志文件 Select 下拉 + RecycleScroller 虚拟滚动渲染 + 黑底深灰滚动条样式，接收 `logsDir` prop 用于「打开目录」按钮
  - SettingsDeveloper.vue 删除日志查看相关状态（logFiles / selectedLogFile / logContent / logLoading / loadLogFiles / loadLogContent / onLogSelect / refreshLogs / logFileOptions / logLines）与 RecycleScroller / parseLogLines / logLineClass 等 import，改为 `<LogViewer :logs-dir="storageDirs?.logs" />`
- 修复三：抽取开发者模式开关卡片为自包含子组件
  - 新建 `src/components/settings/DevModeToggle.vue`（85 行）：包含 devUnlocked / devMode 状态 + toggleDevMode 函数 + onMounted 自行加载注册表状态 + 派发 `developer-mode-changed` window 事件，内部 `v-if="devUnlocked"` 自行控制显隐
  - SettingsAdvanced.vue 删除 devUnlocked / devMode ref + toggleDevMode 函数 + onMounted 中读取开发者模式状态的代码 + 模板中开发者模式卡片整段（~49 行模板），改为 `<DevModeToggle />`
- 修复四：SettingsDeveloper.vue 模板去重
  - 缓存/存储信息/系统信息三张卡片原本各包含手写重复的目录行/信息行模板（每行 ~10 行 × 13 条 = 130 行模板），改为 `cacheEntries` / `storageEntries` / `systemEntries` 三个 computed 数组 + `v-for` 渲染，每张卡片只需 1 个通用行模板
- 结果：
  - `SettingsDeveloper.vue`：385 → 153 行（减少 232 行，主要为模板去重 + 抽取日志查看器）
  - `SettingsAdvanced.vue`：303 → 252 行（减少 51 行，主要为抽取开发者模式开关卡片）
  - `LogViewer.vue`：197 行（新文件）
  - `DevModeToggle.vue`：85 行（新文件）
  - `system-display.ts`：23 行（新文件）
  - `log-display.ts`：50 行（新文件）
- 验证：`npm run build` 通过

#### 前端：日志查看器会话分隔标记着色 + 跳过开头空行
- 现象：后端每次会话开始时打印 `\n=== MoLaunch Started ===`，前端渲染时该行显示为默认白色（与其他原始输出无法区分），且开头多出一个空行
- 修复（仅改 `src/utils/log-display.ts`）：
  - 新增 `session` 级别：`=== ... ===` 模式的行匹配为 `session`，对应颜色 `text-indigo-400`（靛蓝，醒目区分启动边界，区别于 INFO 的绿色 / DEBUG 的青色）
  - `parseLogLines` 跳过开头连续空行（后端 `\n===` 前缀导致的开头空行），中间空行保留（作为日志段落分隔）；行号保留原始文件行号，便于与实际文件对照
- 验证：`npm run build` 通过

### 修复

#### 代码重构阶段 4.9：拆分 commands/community/install.rs 为 7 个子模块
- 现象：`commands/community/install.rs` 1117 行，单一文件混合「6 个对外数据结构（DownloadRequest / DownloadResult / CommunityDownloadProgress / InstallModpackRequest / ModpackFormat / InstallModpackResult）」「7 个纯工具函数（format_bytes / apply_filename_format / resolve_install_dir / parse_cf_loader_id / parse_mr_loader / extract_mr_project_id / construct_cf_edge_url）」「3 个 zip/下载操作（download_files_concurrent / extract_overrides / detect_modpack_format）」「CF 整合包处理（6 个 manifest 数据结构 + install_cf_mods）」「MR 整合包处理（2 个 index 数据结构 + install_mr_files）」「6 个 #[tauri::command] 命令（download_resource / format_download_filename / download_resource_to_path / install_resource / get_resource_install_path / install_modpack）」6 块关注点，其中 install_modpack 单函数达 252 行
- 修复：将 `install.rs` 升级为 `install/` 目录，拆为 7 个子模块：
  - `install/types.rs`（106 行）：DownloadRequest / DownloadResult / CommunityDownloadProgress / InstallModpackRequest / ModpackFormat / InstallModpackResult（均 pub）+ ModpackInfo（pub(super)，install_modpack Stage 1 解析中间结构，跨 CF/MR 格式统一）
  - `install/helpers.rs`（129 行）：format_bytes / apply_filename_format / resolve_install_dir / parse_cf_loader_id / parse_mr_loader / extract_mr_project_id / construct_cf_edge_url 共 7 个纯函数（均 pub(super)）
  - `install/concurrent.rs`（183 行）：download_files_concurrent（多文件并发下载，进度汇总到 download_state 指定 stage）+ extract_overrides（解压 overrides / client-overrides 到 instance 目录）+ detect_modpack_format（检测 manifest.json / modrinth.index.json）
  - `install/curseforge.rs`（141 行）：CfManifest / CfMinecraft / CfModLoader / CfManifestFile / CfFilesBatchResponse / CfFileEntry 共 6 个 CF manifest 数据结构 + install_cf_mods（POST /v1/mods/files 批量查询 → 批量查 slug → 应用 filename_format → 并发下载）
  - `install/modrinth.rs`（121 行）：MrIndex / MrFile 共 2 个 MR index 数据结构 + install_mr_files（遍历 files[] 直接下载，mods/ 路径下文件应用 filename_format）
  - `install/modpack_stages.rs`（156 行）：download_modpack_archive（Stage 0，下载原始整合包）+ parse_modpack_info（Stage 1，解析 manifest/index 得到 ModpackInfo）—— 从 install_modpack 中抽取的两个独立阶段，降低 install_modpack 自身行数
  - `install/mod.rs`（379 行）：6 个 `#[tauri::command]` 命令（download_resource / format_download_filename / download_resource_to_path / install_resource / get_resource_install_path / install_modpack）
- 关键设计：install_modpack 原 252 行通过抽取 download_modpack_archive + parse_modpack_info 两个阶段辅助函数降至 ~150 行，mod.rs 总行数控制在 379 行（6 个命令 + 1 个 install_modpack 主体）
- 关键约束（继承 Phase 4.8）：`#[tauri::command]` 宏在定义处生成 `__cmd__` 符号，命令函数不能移到子模块后用 pub use 重导出，故 6 个命令必须留在 mod.rs
- 重导出策略：`pub use types::{DownloadRequest, DownloadResult, CommunityDownloadProgress, InstallModpackRequest, ModpackFormat, InstallModpackResult};` 保持 `commands::community::install::X` 路径完全向后兼容，`community/mod.rs` 的 `pub mod install;` + `pub use install::{download_resource, ...}` 共 2 处声明 + `lib.rs` invoke_handler 注册均无需修改
- 验证：`cargo check` 通过

#### 代码重构阶段 4.8：拆分 commands/version/mods.rs 为 4 个子模块
- 现象：`commands/version/mods.rs` 813 行，单一文件混合「3 个数据结构（ModInfo / ModMetadata / ModMeta）」「2 个共享辅助函数（get_mods_dir + sanitize_file_name）」「jar 内 mod 元数据读取流水线（read_mod_metadata + 8 个内部辅助：finalize_metadata / extract_logo_data_url / guess_mime / read_fabric_mod_meta / read_forge_mods_toml_meta / read_manifest_version / read_mcmod_info_meta / parse_toml_kv / lookup_translated）」「8 个 #[tauri::command] 命令（is_version_modable / list_mods / toggle_mod / delete_mod / install_mod / open_mods_dir / get_version_mods_dir / reveal_mod_file）+ infer_loader_type」4 块关注点
- 修复：将 `mods.rs` 升级为 `mods/` 目录，拆为 4 个子模块：
  - `mods/types.rs`（78 行）：`ModInfo`（pub，前端 invoke 返回类型）+ `ModMetadata`（pub(crate)，jar 元数据最终结构）+ `ModMeta`（pub(super)，jar 元数据中间结构）
  - `mods/helpers.rs`（37 行）：`get_mods_dir`（pub(crate)，按隔离模式解析 effective_dir/mods）+ `sanitize_file_name`（pub(super)，路径遍历防护）
  - `mods/metadata.rs`（~280 行）：`read_mod_metadata`（pub(crate)，jar 元数据读取流水线入口）+ 8 个内部辅助函数（finalize_metadata / extract_logo_data_url / guess_mime / read_fabric_mod_meta / read_forge_mods_toml_meta / read_manifest_version / read_mcmod_info_meta / parse_toml_kv / lookup_translated）
  - `mods/mod.rs`（402 行）：8 个 `#[tauri::command]` 命令（is_version_modable / list_mods / toggle_mod / delete_mod / install_mod / open_mods_dir / get_version_mods_dir / reveal_mod_file）+ 私有 `infer_loader_type`
- 关键约束：`#[tauri::command]` 宏在函数定义处生成 `__cmd__` / `__tauri_command_name_` 辅助符号，`lib.rs` 的 `invoke_handler` 通过 `commands::version::mods::__cmd__X` 路径查找。**命令函数不能移到子模块后用 `pub use` 重导出**（否则 `__cmd__` 符号留在子模块，路径查找失败）。故 8 个命令必须留在 `mod.rs`，仅类型 / 辅助函数 / 元数据读取流水线可拆分
- 重导出策略：
  - `pub use types::ModInfo;`（ModInfo 为 pub，前端可见）
  - `pub(crate) use types::ModMetadata;`（ModMetadata 为 pub(crate)，crate 内可见，preload 复用）
  - `pub(crate) use helpers::get_mods_dir;`（commands/version/preload.rs 的 `use super::mods::get_mods_dir` 依赖此重导出）
  - `pub(crate) use metadata::read_mod_metadata;`（minecraft/community/preload.rs 的 `crate::commands::version::mods::read_mod_metadata` 依赖此重导出）
  - 注意：ModMetadata 在 metadata.rs 中是私有 `use super::types::ModMetadata` 引入的，不能通过 `pub(crate) use metadata::{read_mod_metadata, ModMetadata}` 重导出（编译器报 E0603 private），必须从 `types` 直接重导出
- `commands::version::mods::{is_version_modable, list_mods, toggle_mod, delete_mod, install_mod, open_mods_dir, reveal_mod_file, get_version_mods_dir, ModInfo, ModMetadata, read_mod_metadata, get_mods_dir}` 路径保持完全向后兼容，`lib.rs` 的 invoke_handler 注册 + `commands/version/preload.rs` + `minecraft/community/preload.rs` 共 3 处外部引用均无需修改
- 验证：`cargo check` 通过

#### 代码重构阶段 4.7：拆分 minecraft/community/modrinth.rs 为 4 个子模块
- 现象：`minecraft/community/modrinth.rs` 819 行，单一文件混合「7 个 MR API 响应数据结构（MrHit / MrProject / MrVersion 等）+ 基地址常量」「响应到统一资源模型的转换（convert_hit / convert_project / convert_version + build_facets 查询参数构造）」「HTTP 请求层（pick_base + mr_get / mr_post + source 策略回退镜像 + 404 优雅处理）」「公共 API（version_files_search / search / get_project / get_versions / batch_get_project_slugs）」4 块关注点
- 修复：将 `modrinth.rs` 升级为 `modrinth/` 目录，拆为 4 个子模块：
  - `modrinth/types.rs`（110 行）：`MR_OFFICIAL_BASE` / `MR_MIRROR_BASE` 常量 + `MrSearchResponse` / `MrHit` / `MrProject` / `MrVersion` / `MrFile` / `MrHashes` / `MrDependency` 共 7 个 MR API 响应数据结构（`pub(crate)` 可见性，仅模块内部使用）
  - `modrinth/convert.rs`（248 行）：`convert_hit`（搜索命中 → ResourceProject）+ `convert_project`（工程详情 → ResourceProject，含 mcmod 中文译名 + 加载器标志位聚合）+ `convert_version`（版本 → ResourceVersion，取 primary 文件 + 提取 required 依赖）+ `build_facets`（构造 MR facets 查询参数 `[["project_type:mod"],["categories:'forge'"]]`，ignore_quilt 过滤）
  - `modrinth/http.rs`（257 行）：`pick_base`（source 策略：0=强制镜像 / 1=缓慢时换镜像 / 2=尽量官方）+ `mr_get` / `mr_post`（source=1 时官方失败自动回退镜像，但 404 不重试因镜像也是 404，官方 20s 超时，含嵌套 `parse_resp` / `parse_post_resp` 处理 404/非 2xx/响应解析）
  - `modrinth/mod.rs`（234 行）：`version_files_search`（参考 PCL2 LocalResourceOnlineLoad 步骤 1-3：version_files 用 SHA1 查 → project_id → /projects 批量查询 + sha1 一致性校验防错位）+ `search` + `get_project` + `get_versions` + `batch_get_project_slugs`（整合包文件名格式化用）
- 同步清理：移除 `modrinth.rs` 中的私有 `urlencode_params` 函数（与 `curseforge.rs` 重复），改用 Phase 4.6 抽取的 `super::common::urlencode_params`。至此 `urlencode_params` 重复定义问题完全解决
- `modrinth::{search, get_project, get_versions, version_files_search, batch_get_project_slugs}` 公共 API 路径保持完全向后兼容，`community/mod.rs` 已有的 `pub mod modrinth;` 声明 + 5 处外部调用（preload / searcher / detail / community/install）均无需修改
- 验证：`cargo check` 通过

#### 代码重构阶段 4.6：拆分 minecraft/community/curseforge.rs 为 4 个子模块
- 现象：`minecraft/community/curseforge.rs` 786 行，单一文件混合「9 个 CF API 响应数据结构（CfModEntry / CfFile / CfSearchResponse 等）」「响应到统一资源模型的转换（convert_project / convert_version / parse_cf_download_url）」「HTTP 请求层（get_cf_config + cf_get / cf_post + source 策略回退镜像）」「公共 API（fingerprint_search / search / get_project / get_versions / batch_get_mod_slugs + 私有 curseforge_loader_type）」4 块关注点
- 修复：将 `curseforge.rs` 升级为 `curseforge/` 目录，拆为 4 个子模块：
  - `curseforge/types.rs`（111 行）：`CfSearchResponse` / `CfPagination` / `CfModEntry` / `CfLogo` / `CfLinks` / `CfCategory` / `CfFile` / `CfHash` / `CfFilesResponse` 共 9 个 CF API 响应数据结构（`pub(crate)` 可见性，仅模块内部使用）
  - `curseforge/convert.rs`（140 行）：`convert_project`（CF 工程条目 → ResourceProject，含 tags 翻译 + mcmod 中文译名 + 加载器标志位聚合）+ `convert_version`（CF 文件 → ResourceVersion）+ `parse_cf_download_url`（参考 PCL2 ParseCurseForgeDownloadUrls，构造 edge.forgecdn.net 回退 URL）
  - `curseforge/http.rs`（259 行）：`CF_OFFICIAL_BASE` / `CF_MIRROR_BASE` 常量 + `get_cf_config`（source 策略：0=强制镜像 / 1=缓慢时换镜像 / 2=尽量官方）+ `build_cf_request` / `build_cf_post_request`（附加 x-api-key header）+ `cf_get` / `cf_post`（source=1 时官方失败自动回退镜像重试，官方请求 10s/15s 超时）
  - `curseforge/mod.rs`（305 行）：`fingerprint_search`（参考 PCL2 LocalResourceOnlineLoad 步骤 1-3：fingerprints/432 → modId → /mods 批量查询）+ `search` + `get_project`（数字 modId 走 /mods/{id}，slug 走 /mods/search）+ `get_versions` + `batch_get_mod_slugs`（整合包文件名格式化用）+ 私有 `curseforge_loader_type`
- 同步修复：发现 `urlencode_params` 函数在 `curseforge.rs` 与 `modrinth.rs` 中重复定义，已抽取到 `community/common.rs` 作为 `pub fn urlencode_params`（与已有 `fmt_elapsed` 共置），`curseforge/mod.rs` 改用 `super::common::urlencode_params`。`modrinth.rs` 中的私有 `urlencode_params` 待 Phase 4.7 拆分时一并清理
- `curseforge::{search, get_project, get_versions, fingerprint_search, batch_get_mod_slugs}` 公共 API 路径保持完全向后兼容，`community/mod.rs` 已有的 `pub mod curseforge;` 声明 + 5 处外部调用（preload / searcher / detail / community/install）均无需修改
- 验证：`cargo check` 通过

#### 代码重构阶段 4.5：拆分 commands/version/install.rs 为 4 个子模块
- 现象：`commands/version/install.rs` 694 行，单一文件混合「install_merged 主入口（参数校验 + 版本名唯一化 + MC 本体下载 + 多加载器顺序安装 + 进度 ticker + 失败清理）」「加载器单次安装逻辑（install_single_loader + 进度 ticker）」「版本目录命名冲突解决（resolve_unique_instance_name + find_loader_version_dir）」「失败清理（cleanup_failed_install）」4 块关注点
- 修复：将 `install.rs` 升级为 `install/` 目录，拆为 4 个子模块：
  - `install/loader_helpers.rs`（136 行）：`install_single_loader`（pub(crate)）+ `start_progress_ticker`（pub(crate)），加载器单次安装与进度推送逻辑
  - `install/version_naming.rs`（83 行）：`resolve_unique_instance_name`（pub(crate)，处理 `(1)`/`(2)` 后缀冲突 + 复用 modpack 半成品目录）+ `find_loader_version_dir`（pub(crate)，按 Forge/NeoForge/Fabric/OptiFine/LiteLoader 前缀匹配版本目录）
  - `install/cleanup.rs`（54 行）：`cleanup_failed_install`（pub(crate)），删除原 MC 目录 + 加载器创建目录（fabric 用精确 `fabric-{fabric_version}-{mc_version}` 模式避免误删）
  - `install/mod.rs`（471 行）：`install_merged` 主入口 #[tauri::command]（参数校验 + 唯一实例名解析 + MC 本体下载 + 多加载器顺序安装委托 + 进度 ticker + 失败清理委托）
- `commands::version::install::install_merged` 路径保持完全向后兼容，`commands/version/mod.rs` 已有的 `pub mod install;` 声明 + `lib.rs` 的 invoke_handler 注册均无需修改
- 验证：`cargo check` 通过

#### 代码重构阶段 4.4：拆分 minecraft/version/setup.rs 为 3 个子模块
- 现象：`minecraft/version/setup.rs` 692 行，单一文件混合「PersonalizationUpdate + VersionSetup 结构体定义」「INI/Maven 解析辅助 + 版本号检测自由函数」「impl VersionSetup（构造 + save/load + ensure_complete + load_or_create + update_personalization + from_version_json）+ tests」3 块关注点
- 修复：将 `setup.rs` 升级为 `setup/` 目录，拆为 3 个子模块：
  - `setup/types.rs`（~110 行）：`PersonalizationUpdate` + `VersionSetup` 结构体定义
  - `setup/helpers.rs`（~85 行）：`parse_ini` / `extract_maven_version`（私有）/ `read_setup_version_and_loader` / `read_mc_version_from_json` / `detect_version_and_loader` 自由函数
  - `setup/mod.rs`（~540 行）：`impl VersionSetup` 全部方法（new/file_path/empty/exists/save/save_full/save_with_options/ensure_complete/load_or_create/update_personalization/load/from_version_json）+ tests
- `pub use` 保持 `crate::minecraft::version::setup::{VersionSetup, PersonalizationUpdate, detect_version_and_loader, read_setup_version_and_loader, read_mc_version_from_json}` 路径完全向后兼容，外部 8 处调用无需修改
- 验证：`cargo check` 通过 + `cargo test --lib setup` 2 个单元测试通过

#### 代码重构阶段 4.3：拆分 minecraft/launch/watcher.rs 为 4 个子模块
- 现象：`minecraft/launch/watcher.rs` 690 行，单一文件混合「8 个数据结构」「日志行解析+加载进度检测」「崩溃分析（运行时日志+崩溃报告文件）」「GameWatcher 结构体+start_monitoring 流程」4 块关注点
- 修复：将 `watcher.rs` 升级为 `watcher/` 目录，拆为 4 个子模块：
  - `watcher/types.rs`（~115 行）：`GameState` / `ExitInfo` / `CrashInfo` / `CrashCategory` / `LogLevel` / `LogEntry` / `LoadProgress` + impl `name()`
  - `watcher/log_parser.rs`（~70 行）：`parse_log_line` / `extract_log_level`（私有）/ `detect_load_progress` 纯函数（原 GameWatcher 静态方法）
  - `watcher/analyzer.rs`（~260 行）：`analyze_crash` / `analyze_stack_for_mod`（私有）/ `analyze_crash_report`（pub，外部调用）/ `parse_crash_report`（私有）
  - `watcher/mod.rs`（~250 行）：`GameWatcher` 结构体 + new/state/load_progress/recent_logs/exit_receiver/start_monitoring/stop
- `pub use` 保持 `crate::minecraft::launch::watcher::X` 路径完全向后兼容，`launch/mod.rs` 已有的 `pub use watcher::{CrashCategory, CrashInfo, ExitInfo, GameState, GameWatcher, LoadProgress}` re-export 无需修改
- 原 `Self::parse_log_line`/`Self::detect_load_progress`/`Self::analyze_crash` 调用改为 `parse_log_line`/`detect_load_progress`/`analyzer::analyze_crash` 自由函数调用
- 验证：`cargo check` 通过

#### 代码重构阶段 4.2：拆分 minecraft/auth/storage.rs 为 4 个子模块
- 现象：`minecraft/auth/storage.rs` 626 行，单一文件混合「数据结构」「注册表常量+低层操作」「AuthStorage 结构体+加解密+load/save」「11 个高层操作方法」4 块关注点
- 修复：将 `storage.rs` 升级为 `storage/` 目录，拆为 4 个子模块：
  - `storage/types.rs`（~67 行）：`StoredMsAccount` + impl From<&MicrosoftLoginResult> + `StoredOfflineAccount` + `PersistedAuthState` + `CurrentUser`
  - `storage/registry.rs`（~85 行）：11 个注册表键名常量 + `ALL_KEYS` 数组 + 4 个自由函数 `reg_key`/`reg_get`/`reg_set`/`reg_delete`（原 AuthStorage 静态方法）
  - `storage/operations.rs`（~185 行）：`impl AuthStorage` 独立 impl 块，封装 11 个高层操作：`save_ms_login`/`save_offline_login`/`set_offline_skin`/`remove_offline_account`/`get_offline_account`/`clear_current_user`/`remove_ms_account`/`get_ms_account`/`get_current_refresh_token`/`update_ms_token`（仅依赖 `self.load()`/`self.save()`，与注册表细节解耦）
  - `storage/mod.rs`（~280 行）：`AuthStorage` 结构体 + `new()` + `encrypt`/`decrypt` + `reg_set_encrypted`/`reg_get_decrypted` + `load`/`invalidate`/`save` 核心方法
- `pub use types::*` 保持 `crate::minecraft::auth::storage::X` 路径完全向后兼容，外部 5 处调用无需修改
- 利用 Rust 多 impl 块特性，将 11 个高层操作分散到独立文件，主文件聚焦核心 load/save
- 验证：`cargo check` 通过

#### 代码重构阶段 4.1：拆分 state/mod.rs 为 5 个子模块
- 现象：`state/mod.rs` 433 行，混合「AppState 聚合」「认证结构」「下载阶段/状态」「AppConfig + McFolder + 路径解析」「LaunchHistory」5 块关注点
- 修复：按关注点拆分到 `state/` 下的 5 个子模块：
  - `state/auth.rs`（28 行）：`LocalAuthResult` + `AuthState`
  - `state/launch.rs`（18 行）：`LaunchHistory`
  - `state/download.rs`（209 行）：`StageStatus` 枚举 + `DownloadStage` + impl (new/new_grouped) + `DownloadState` + impl Default + 8 个进度同步方法（reset_stages/append_stages/set_current_stage/set_stage_status/set_stage_bytes/sync_stage_from_progress/mark_complete/mark_failed）
  - `state/config.rs`（120 行）：`AppConfig` + impl Default + `McFolder` + `get_default_game_dir` + `resolve_game_dir`
  - `state/app.rs`（82 行）：`AppState` 聚合结构体 + impl Default + impl new()（加载配置 + 加载 SDK + 创建 auth_storage 共享 Arc）
- `state/mod.rs` 缩减至 22 行：仅声明 5 个子模块 + `pub use` 统一 re-export 所有公开类型/函数
- 通过 `pub use` 保持 `crate::state::X` 路径完全向后兼容，外部 65 处调用无需修改
- 验证：`cargo check` 通过

#### 代码重构阶段 3.13：拆分 utils/api/system.ts 为 config 模块
- 现象：`utils/api/system.ts` 322 行，混合「系统操作」「目录选择」「下载进度查询」「全局配置读写（getConfig/applyConfig/refreshConfig + ConfigSnapshot/ConfigPatch/ConfigEntry 类型 + configCache 缓存）」4 块关注点，其中配置相关逻辑独占约 230 行
- 修复：抽出 `utils/api/config.ts`（239 行）：
  - 类型：`ConfigSnapshot`（全量快照）/`ConfigPatch`（部分更新）/`ConfigEntry`（扁平 key-value）
  - 缓存：`configCache`/`configPromise` 模块级状态（首次请求后缓存，切换侧栏直接读缓存）
  - 读写：`getConfig(keys?, force?)`（扁平数组格式 + 缓存 + 并发请求合并）/`getConfigMap(force?)`（对象格式 + 缓存）/`applyConfig(patch)`（统一更新入口 + 乐观缓存同步）/`refreshConfig()`（清空缓存）
  - 直写 INI：`getConfigValue(section, key)`/`setConfigValue(section, key, value)`（保留用于调试/迁移）
- 父文件 `system.ts` 保留：系统操作（openGameDir/openPath/revealInExplorer/getGameDir/selectFolder/selectFile/saveFile/setGameDir/getSystemMemory/getConfigPath/saveConfigToFile）+ 下载进度查询（getDownloadProgress/isDownloading/resetDownloadProgress）
- `tauri.ts` 增加 `export * from './api/config'` 保持 `import * as tauri` 用法向后兼容
- 修复 2 处直接 import：`composables/useDebouncedSave.ts`（ConfigPatch 类型）和 `views/settings/SettingsOther.vue`（getConfigMap/applyConfig）改为从 `@/utils/api/config` 导入
- `system.ts` 缩减至 92 行

#### 代码重构阶段 3.12：拆分 stores/version.ts 为 useLaunchState composable
- 现象：`stores/version.ts` 335 行，单一 store 混合「版本列表」「下载状态」「启动流程」「Java 下载进度」「游戏退出监听」「进度轮询」6 块关注点，其中启动相关逻辑独占约 175 行
- 修复：抽出 `composables/useLaunchState.ts`（200 行）：
  - 封装：launching/launchingVersionId/runningPid/runningVersionId/launchProgress/javaDownloadProgress 状态
  - 封装：launchGame/stopGame/cancelLaunch/checkRunningGame 函数
  - 封装：startProgressPolling/stopProgressPolling（每 200ms 轮询后端启动进度）
  - 封装：startJavaDownloadListener/stopJavaDownloadListener（监听 Java 自动下载进度事件）
  - 封装：setupGameExitListener/cleanupGameExitListener（游戏退出事件监听 + 提示）
  - 封装：launchStageName computed（10 个阶段枚举 → 中文显示）
  - 与版本列表本身解耦：launchGame 直接接收完整 params，不依赖 versions 数组
- 父 store 保留：版本列表状态 + fetchVersions/refreshVersions、下载状态 + startDownload/updateProgress/finishDownload、selectedVersion 持久化、loaderVersionsCache + getLoaderCache/setLoaderCache、getVersionById/getReleaseVersions/getSnapshotVersions
- 父 store 通过 `const { ... } = useLaunchState()` 委托启动状态，并 re-export 给调用方（保持 store API 完全向后兼容，其他文件 import useVersionStore 无需修改）
- `stores/version.ts` 缩减至 160 行

#### 代码重构阶段 3.11：拆分 ModTab.vue 为 3 子组件 + 1 composable + 1 util
- 现象：`ModTab.vue` 710 行，混合「不可安装提示」「工具栏」「列表项」「3 种空状态」5 块 UI + 详情按钮 3 级 fallback 逻辑（70 行）+ 显示辅助函数
- 修复：抽出 3 子组件 + 1 composable + 1 util：
  - `views/version-settings/mod-tab/ModListItem.vue`（164 行）：单个 Mod 行 UI，props 接收 mod/detailLoadingFor/modLocalNameStyle，emit toggle/delete/show-info/open-wiki/open-file；包含 5 个操作按钮 + 状态色条 + 图标 + 信息区
  - `views/version-settings/mod-tab/ModToolbar.vue`（94 行）：工具栏（从文件安装/打开文件夹/刷新按钮 + 过滤按钮组 + 搜索输入），使用 `defineModel` 双向绑定 modFilter/modSearch
  - `views/version-settings/mod-tab/ModEmptyState.vue`（82 行）：4 种空状态统一组件（not-modable/loading/empty/no-match），props 接收 variant + modsCount，emit go-download/go-select/install
  - `composables/useModDetailQuery.ts`（162 行）：详情按钮 3 级 fallback 逻辑（零延迟预加载就绪 → 等待预加载 → 并发请求 CF+MR → 本地信息弹窗）；封装 detailVisible/detailProject/detailLoadingFor 状态 + handleShowInfo + handleOpenWiki
  - `utils/mod-display.ts`（60 行）：纯函数 modTitle/modSubtitle/loaderVisual/stripModVersion，消除父子组件间的逻辑重复
- 父组件保留：mods 列表加载/过滤、handleToggleMod（原地更新避免列表闪烁）、handleDeleteMod/handleInstallMod/handleOpenModsDir/handleOpenFile、onMounted 编排
- `ModTab.vue` 缩减至 297 行

#### 代码重构阶段 3.10：拆分 SetupTab.vue 为 JavaModeSelector + JavaCustomMode 子组件
- 现象：`SetupTab.vue` 534 行，其中 Java 选择模式（4 模式：auto/auto_version/folder/custom）独占约 330 行，包含状态、5 个 computed、6 个函数、4 套模式 UI
- 修复：抽出 2 个子组件到 `views/version-settings/setup-tab/`：
  - `JavaModeSelector.vue`（219 行）：4 模式下拉框 + auto/auto_version/folder 三种模式 UI；保留状态（javaMode/javaVersionMin/Max/customJavaPath/refreshingJava/javaReqs）、javaReqDesc/javaVersionRangeTip/hasCompatibleJava/pickDefaultJavaPath、handleSaveJavaMode/handleSaveJavaVersionRange、watch(personalization)
  - `JavaCustomMode.vue`（148 行）：custom 模式 UI（Java 列表 Select + 刷新按钮 + 空状态 + 不兼容警告）；props 接收 customJavaPath/refreshingJava（v-model）+ javaReqs；内部管理 javaOptionsForCustom/customJavaWarning/handleSelectJavaFromList/handleImportJava/handleRefreshJavaList
- 额外修复：将 `isJavaCompatible` 抽取为 `utils/api/java.ts` 中的纯函数（参数为 majorVersion + reqs），消除父子组件间的逻辑重复
- 父子组件通过 `useVersionSettings()` 共享状态（模块级单例），无需 props 透传 selectedId/personalization
- 父组件 `SetupTab.vue` 保留：启动选项（版本隔离/窗口标题/自定义信息）、内存分配（MemorySection 子组件）、服务器、高级选项
- `SetupTab.vue` 缩减至 187 行

#### 代码重构阶段 3.9：拆分 ResourceDetail.vue 为 3 个子组件
- 现象：`ResourceDetail.vue` 481 行，模板独占 232 行，混合「头部+操作按钮」「版本分组卡片」「下载进度浮层」3 块独立 UI + 各自的 helper 函数
- 修复：抽出 3 个子组件到 `components/community/resource-detail/`：
  - `ResourceDetailHeader.vue`（~108 行）：Logo + 标题 + 平台标签 + 操作按钮行（转到平台/MC百科/复制名称），`openMcmod`/`copyName` 内部处理
  - `VersionGroupCard.vue`（~115 行）：版本分组卡片，props 接收 title/versions/expanded/mounted/downloading/isModpack，emit `toggle`/`download`/`install`；`releaseColor`/`loaderNames` 移入内部
  - `DownloadProgressOverlay.vue`（~40 行）：下载进度浮层，props 接收 progress，`formatSpeed`/`downloadPercent` 移入内部
- 父组件保留：版本加载 watch、handleDownload、handleInstallModpack、composable 编排
- `ResourceDetail.vue` 缩减至 221 行

#### 代码重构阶段 3.8：提取 useLoaderData composable 消除 5 处重复 fetch 模式
- 现象：`LoaderSelect.vue` 468 行，其中 `onMounted` 独占 107 行，5 种加载器（Forge/NeoForge/Fabric/OptiFine/LiteLoader）的 `fetch → 赋值 → catch → finally` 模式逐字重复
- 修复：新建 `composables/useLoaderData.ts`（~180 行）：
  - 提取通用 `fetchLoader<T>` 泛型函数消除 5 处重复的 promise 链
  - 包含：原始版本数据 refs、加载状态 refs、5 个 computed 版本项列表（forgeItems/neoforgeItems/fabricItems/optifineItems/liteloaderItems）、`fetchAll()` 函数（缓存检查 + 独立请求 + 完成后缓存）
  - 导出 `LoaderItem`/`ForgeVersion`/`NeoforgeVersion`/`FabricVersion`/`OptifineVersion` 类型供外部复用
- 父组件保留：MC 版本类型判断、选中状态管理、兼容性检查、实例名生成、模板
- `LoaderSelect.vue` 缩减至 291 行

#### 代码重构阶段 3.7：拆分 SkinManager.vue 为 4 个子组件 + 提取 runWithRefresh 消除重复
- 现象：`SkinManager.vue` 443 行，混合「3D 预览」「上传皮肤」「离线皮肤选择」「披风列表」「动画选择」「账号管理」6 块 UI
- 修复：抽出 4 个子组件到 `components/common/skin-manager/`：
  - `SkinAnimationSelector.vue`（~55 行）：动画状态选择器，`v-model` 双向绑定，`AnimationType` 类型从此处 `export`
  - `SkinCapeList.vue`（~55 行）：披风列表，props 接收 capes/activeCape/uploading，emit `equip`/`unequip`
  - `SkinUploadPanel.vue`（~80 行）：微软账号上传皮肤 + 账号管理快捷入口（修改密码/用户名，内部 `open` 外部链接）
  - `SkinLocalSelector.vue`（~40 行）：离线账号本地皮肤选择网格，emit `select`
- 额外修复：提取 `runWithRefresh` 工具函数消除 `pickAndUpload`/`onEquipCape`/`onUnequipCape` 三处重复的 `uploading=true → 执行 → showSuccess → loadInfo → bumpSkinVersion → uploading=false` 模式
- 父组件保留：3D 预览区（SkinModel3D + SkinAvatar + 下载按钮）、状态管理、loadInfo、saveSkinToLocal、onSelectLocalSkin
- `SkinManager.vue` 缩减至 265 行

#### 代码重构阶段 3.6：拆分 SettingsLaunch.vue 为 JavaPathSelector 子组件
- 现象：`SettingsLaunch.vue` 431 行，混合「Java 路径选择」「内存分配」「版本隔离」「游戏目录」4 块逻辑，其中 Java 选择器独占约 155 行（按钮 + 下拉列表 + 点击外部收起 + 自动检测/手动导入）
- 修复：抽出 Java 选择器为 `views/settings/settings-launch/JavaPathSelector.vue`（约 175 行）：
  - 自包含：管理 `showJavaList`/`detectingJava`/`javaSelectorRef` 状态、`handleDocumentClick` 外部点击监听、`handleAutoDetectJava`/`handleManualImportJava` 函数
  - 模板含：版本徽章、下拉输入框、自动选项 + 已安装 Java 列表
- 父组件保留：内存配置（自动/自定义 + 可视化条 + 滑动条）、版本隔离（Select 下拉）、游戏目录只读展示
- `SettingsLaunch.vue` 缩减至 279 行，并移除不再需要的 `useJavaStore`、`ArrowPathIcon`/`DocumentPlusIcon`、`showInfo`/`showSuccess`/`showError` 等导入和 click-outside watcher

#### 代码重构阶段 3.5：拆分 AccountSelector.vue 为 4 个子模块
- 现象：`AccountSelector.vue` 440 行，混合「未登录提示」「指示器」「卡片轮播」「拖动/滚轮导航」4 块独立逻辑
- 修复：抽出 4 个子模块到 `components/home/account-selector/` 和 `composables/`：
  - `LoginPrompt.vue`（~25 行）：未登录时的图标 + 登录按钮（内部 router 跳转）
  - `AccountIndicator.vue`（~50 行）：圆点指示器 + 计数，emit `switch` 切换
  - `AccountCard.vue`（~80 行）：单个账号卡片（头像/用户名/操作按钮），emit `skin`/`logout`
  - `useSwipeNavigation.ts`（~85 行）：拖动/滚轮导航 composable，参数为 `totalCards`/`currentIndex`/`onSwitch` 回调
- `AccountCardData` 接口在 `AccountCard.vue` 中 `export`，`AccountIndicator.vue` 和父组件通过 `import type` 复用，避免重复定义
- 父组件保留：cards computed、switchTo/prev/next、switchAccount/removeAccount、watch currentIndex 同步逻辑
- `AccountSelector.vue` 缩减至 289 行

#### 代码重构阶段 3.4：拆分 Downloads.vue 为空状态 + 统计面板子组件
- 现象：`Downloads.vue` 367 行，混合「空状态」「统计面板」「任务列表」三块独立 UI
- 修复：抽出两个子组件到 `views/downloads/`：
  - `DownloadEmptyState.vue`（~95 行）：无下载任务时的空状态卡片，含「浏览版本」按钮（内部用 router 跳转）
  - `DownloadStatsPanel.vue`（~75 行）：左侧统计面板，接收 6 个 props（currentStageName/percentage/speed/bytesDownloaded/bytesTotal/filesRemaining）
- 父组件保留：进度计算（taskGroups computed）、任务列表模板、折叠/展开状态
- `Downloads.vue` 缩减至 286 行；同时移除父组件不再需要的 `useRouter`、`useRouter`-related `goToVersions`、`formatSpeed` 等导入

#### 代码重构阶段 3.3：拆分 VersionSelect.vue 为 FolderSidebar 子组件
- 现象：`VersionSelect.vue` 345 行，混合「左侧文件夹管理」和「右侧版本列表」两块相互独立的逻辑
- 修复：抽出左侧文件夹列表为 `views/version-select/FolderSidebar.vue`（179 行，含切换/添加/移除文件夹逻辑）
- 父子通信：
  - 子组件 `defineExpose({ loadFolders })` 暴露刷新接口
  - 子组件 `emit('switched', path)` 通知父组件重新加载版本列表（父组件直接绑定 `@switched="loadInstalled"`）
- `VersionSelect.vue` 缩减至 208 行，并移除不再需要的 `invoke`/`showConfirm`/`showPrompt`/`showSuccess`/`showWarning`/`showError` 导入和 `McFolder` 接口

#### 代码重构阶段 2.5：新增 SegmentedButtons.vue 组件
- 现象：3 按钮选择组（`<div class="flex gap-2"><button :class="active ? ... : ..."/></div>`）在 5 处重复，类名字符串 `border-primary-500 bg-primary-50 text-primary-700` 在 6 个组件中重复
- 修复：新建 `components/common/SegmentedButtons.vue`，支持 `v-model`（直接赋值）和 `@select`（自定义回调）两种模式
- 应用迁移：
  - `SettingsAdvanced.vue` 的代理模式 + 代理类型（2 处 3 按钮组）改用 `SegmentedButtons`
  - `SettingsDownload.vue` 的版本列表源 + 文件下载源（2 处 3 按钮组）改用 `SegmentedButtons`
  - `MemorySection.vue` 的分配模式（1 处 3 按钮组）改用 `SegmentedButtons`（@select 模式，保留 handleSaveMemoryMode 回调）
- 减少约 100 行模板样板代码

#### 代码重构阶段 2.2-2.3：新增 useTauriEvent + usePolling composable
- 现象：`listen`/`unlisten` + `setInterval`/`clearInterval` 样板代码在 5+ 处重复，且容易遗忘 onUnmounted 清理
- 修复：
  - 新建 `composables/useTauriEvent.ts`：封装 `listen` + 自动 `onUnmounted` 清理 unlisten 句柄
  - 新建 `composables/usePolling.ts`：封装 `setInterval` + 自动 `onUnmounted` 清理 timer + 防重复启动
- 应用：将 `MemorySection.vue` 和 `SettingsLaunch.vue` 的 1 秒内存轮询（逻辑完全相同）迁移到 `usePolling`，消除样板代码
- 后续可逐步迁移 `useCommunityDownload.ts`、`JavaDownloadBar.vue`、`stores/auth.ts`、`stores/version.ts` 的事件监听
- 说明：未强制全量迁移，避免一次性改动过大；新代码默认使用新 composable

#### 代码重构阶段 2.8：提取 Maven 坐标转路径到 utils/maven.rs
- 现象：Maven 坐标转路径的核心逻辑（`split(':')` → `replace('.', '/')` → 拼接路径）在 4 处重复实现：
  - `minecraft/launch/mod.rs` 私有 `maven_name_to_path` → 相对路径 String
  - `minecraft/version/libraries.rs` pub `maven_to_relative_path` → 相对路径 String（与上者逻辑完全相同）
  - `minecraft/version/libraries.rs` pub `maven_to_path` → 绝对路径 String
  - `minecraft/loaders/shared.rs` pub `maven_path_to_local` → 绝对路径 PathBuf
- 修复：新建 `minecraft/utils/maven.rs`，提供 `pub fn maven_to_relative_path(name) -> String` 和 `pub fn maven_to_local_path(name, game_dir) -> PathBuf`
- 原 4 个函数中：`launch/mod.rs` 删除私有实现改为直接调用；`libraries.rs` 和 `shared.rs` 的 pub 函数保留为薄包装委托（维持 API 兼容）

#### 代码重构阶段 2.10：统一 SHA1 计算到 file_checker::compute_sha1_hex
- 现象：字节级 SHA1 计算代码在 4 处重复（均为 `sha1::Sha1::new()` + `update` + `hex::encode(finalize)` 模式）：
  - `minecraft/java/download.rs` 私有 `compute_sha1_hex(bytes)`（Java 运行时下载校验）
  - `minecraft/community/preload.rs` `compute_modrinth_sha1(path)`（Modrinth 指纹计算）
  - `minecraft/launch/pipeline/natives.rs` 内联（JAR 文件 SHA1 校验，CWE-494 防护）
  - `minecraft/launch/pipeline/natives.rs` 内联（提取文件 SHA1 审计日志）
- 已有 `minecraft/utils/file_checker.rs::compute_file_hash` 支持文件级哈希，但缺少字节级工具
- 修复：在 `file_checker.rs` 新增 `pub fn compute_sha1_hex(bytes: &[u8]) -> String`；4 处调用点改为 import 或直接调用
- `java/download.rs` 删除本地 `compute_sha1_hex`（`verify_bytes_sha1` 保留，含日志逻辑）
- `preload.rs` 的 `compute_modrinth_sha1` 改为委托 `compute_sha1_hex`
- `natives.rs` 2 处内联代码替换为函数调用

#### 代码重构阶段 2.9：统一 Java 版本检测到 java::detect_java_version
- 现象：Java 版本检测逻辑在 3 处重复实现：
  - `minecraft/launch/mod.rs` 私有 `get_java_version(&Path)`（启动参数构建时用）
  - `minecraft/loaders/forge_installer.rs` 私有 `get_java_major_version(&str)`（Forge 安装器用）
  - `minecraft/java/mod.rs` 公开 `detect_java_version(&str)`（更完善：支持目录路径、黑名单检查、JRE/JDK 判定）
- 前两处实现逐字节相同（`Command::new(java).arg("-version")` + 正则 `version "(\d+)\."`），是 `detect_java_version` 的子集
- 修复：删除 `launch/mod.rs` 和 `forge_installer.rs` 的私有实现，改为调用 `crate::minecraft::java::detect_java_version`
- 额外修复：`get_java_version_weight`（PCL2 权重表）在 `java_selector.rs` 和 `java/mod.rs` 各有一份相同实现；将 `java_selector.rs` 的版本改为 `pub`，`java/mod.rs` 删除本地实现改为调用前者

#### 代码重构阶段 2.7：提取 fmt_elapsed 到 community/common.rs
- 现象：`fmt_elapsed` 函数在 4 个文件中各有一份**完全相同**的实现（格式化耗时为 ms/s），违反 DRY：
  - `minecraft/sources.rs`（私有 `fn fmt_elapsed`）
  - `minecraft/community/curseforge.rs`（私有 `fn fmt_elapsed`）
  - `minecraft/community/modrinth.rs`（私有 `fn fmt_elapsed`，参数用全限定 `std::time::Instant`）
  - `minecraft/community/preload.rs`（私有 `fn fmt_elapsed_from`，函数名带 `_from` 后缀但实现一致）
- 修复：新建 `minecraft/community/common.rs`，提供 `pub fn fmt_elapsed(start: Instant) -> String`；4 个文件删除本地实现，改为 import
- `preload.rs` 的 2 处 `fmt_elapsed_from(start)` 调用同步改名为 `fmt_elapsed(start)`

#### 代码重构阶段 2.1：整合 formatDownloads 到 utils/format.ts
- 现象：`formatDownloads` 函数在 `ResourceCard.vue` 和 `ResourceDetail.vue` 中各有一份**完全相同**的实现（中文万/亿单位格式化），违反 DRY 原则
- 修复：将 `formatDownloads` 提取到已有的 `src/utils/format.ts`，两个组件改为 `import { formatDownloads } from '@/utils/format'`
- 说明：`ResourceDetail.vue` 的本地 `formatSpeed` 保留（精度与 utils 版本不同：MB/s 用 1 位小数、KB/s 用 0 位小数，属于显示偏好差异，非重复代码）

#### 代码重构阶段 1.11：setup.rs 改用原子写入
- 现象：`src-tauri/src/minecraft/version/setup.rs` 的 `save_with_options` 和 `ensure_complete` 两个写入点都直接使用 `std::fs::write(&path, content)`，若写入过程中进程崩溃/断电/磁盘满，setup.ini 会处于半写状态，导致版本元数据损坏
- 修复：改为原子写入模式（与 `storage::Storage::write_config` 一致）：
  - 先写入 `setup.ini.tmp` 临时文件
  - 再 `std::fs::rename` 到 `setup.ini`（同分区 rename 在 POSIX 和 Windows 上均保证原子性）
- 影响函数：`VersionSetup::save_with_options`、`VersionSetup::ensure_complete`

#### 代码重构阶段 1.10：删除 sdk/helpers.rs 死代码
- 现象：`src-tauri/src/sdk/helpers.rs` 的 `get_system_memory_static()` 函数存在两个问题：
  - 每次调用都通过 `libloading::Library::new` 重新加载 DLL，性能开销大（应复用 `SdkInstance` 的共享句柄）
  - 该函数在整个代码库中**无任何调用方**，属于死代码
- 实际内存查询路径：`commands::system::get_system_memory` → `minecraft::system::get_system_memory()`（使用 `sysinfo` crate，不经过 SDK）
- SDK 侧已有 `SdkInstance::get_system_memory()` 方法（使用共享库句柄，实现正确），但同样未被调用
- 修复：直接删除 `sdk/helpers.rs` 文件，并从 `sdk/mod.rs` 移除 `mod helpers;` 和 `pub use helpers::*;`
- 决策依据：与阶段 1.6 一致，遵循"删除无引用 dead code"原则；保留 `SdkInstance::get_system_memory()` 方法作为 SDK 公共 API 表面（未来可能启用）

#### 代码重构阶段 1.8：skin.rs 添加 [Skin] 日志前缀规范
- 现象：`src-tauri/src/minecraft/skin.rs` 的所有 `log_info!`/`log_warn!` 调用使用无前缀的英文消息（如 `"Downloading skin from: {}"`），不符合项目日志规范（其他模块均使用 `[Sources]`/`[Community] CF`/`[Shell]`/`[Chunk]`/`[Natives]` 前缀）
- 修复：为全部 18 处日志调用添加 `[Skin]` 前缀，并将消息文本统一为中文，与项目其他模块风格一致
- 说明：本模块直接使用 `http::get_client()` 而非 `sources::fetch_with_fallback`，因为：
  - 皮肤/披风 PNG 为二进制下载，`fetch_with_fallback` 仅返回 String 文本
  - 目标域名 `textures.minecraft.net` 和 `api.minecraftservices.com` 没有 BMCLAPI 镜像，无回退需求
  - 已在源码顶部添加注释说明此决策

#### 代码重构阶段 1.6：清理 Dead Code（6 处未引用的 interface/function）
- 现象：6 处类型/函数定义从未被任何调用方引用，纯死代码：
  - `src/types/settings.ts` 的 `AppSettings` interface（实际配置走 `ConfigSnapshot`，已被取代）
  - `src/types/version.ts` 的 `InstalledVersion` interface（`VersionSelect.vue` 自定义了同名本地 interface，未引用此处的）
  - `src/types/community.ts` 的 `DetailRequest` interface（`getProjectDetail` 直接用内联对象参数）
  - `src/utils/api/community.ts` 的 `CurseForgeConfig` 和 `CommunityConfig` interfaces（仅作文档说明，无类型引用；对应字段已在 `ConfigSnapshot`/`ConfigPatch` 中）
  - `src/utils/api/sdk.ts` 的 `isSdkInitialized()` 函数（无任何前端调用方，但后端命令 `is_sdk_initialized` 保留）
- 修复：删除上述 6 处 dead code；`community.ts` 中两段配置说明改为注释引用 `ConfigSnapshot`/`ConfigPatch` 字段
- `Theme` 和 `Language` 类型保留（被 `stores/settings.ts` 引用）

#### 代码重构阶段 1.5：统一 DownloadStage 类型定义到 types/download.ts
- 现象：`DownloadStage` 类型在 3 处重复定义：
  - `stores/version.ts` 定义 `DownloadStage` + `DownloadProgress`
  - `composables/useDownloadPolling.ts` 重复定义 `RawDownloadStage`（与 DownloadStage 字段对应）
  - `utils/api/system.ts` 内联定义 `getDownloadProgress` 返回类型（又写一遍字段）
- 修复：新建 `src/types/download.ts` 统一定义 4 个类型：`StageStatus`、`DownloadStage`、`DownloadProgress`、`RawDownloadStage`、`RawDownloadProgress`
- 重构：
  - `stores/version.ts` 删除原 `DownloadStage`/`DownloadProgress` 定义，改为 `import type` + `export type` re-export（保持向后兼容）
  - `composables/useDownloadPolling.ts` 删除 `RawDownloadStage` 定义，改为 `import type` 自 `@/types/download`
  - `utils/api/system.ts` 的 `getDownloadProgress` 返回类型改用 `RawDownloadProgress`，消除内联定义

#### 代码重构阶段 1.3：消除 resolveVersionIcon 同名不同签名冲突
- 现象：`useVersionMeta.ts` 导出 `resolveVersionIcon(type: string)`（按 type 查表），`useVersionSettings.ts` 也导出 `resolveVersionIcon(logo, versionId, explicitType?)`（含 logo 优先策略），同名不同签名，调用方极易混淆
- 修复：将 `useVersionSettings.ts` 的 `resolveVersionIcon` 重命名为 `resolveVersionIconWithLogo`，并补充 JSDoc 说明两者区别
- 同步更新调用方：`VersionSelect.vue` 解构 + 模板调用；`Versions.vue` 解构（保留旧名 alias，避免影响 `getVersionIcon` 内部调用）
- `useVersionMeta.ts` 的 `resolveVersionIcon(type)` 保留原名不变（语义清晰，且已被 `Versions.vue` 以 `resolveIconByType` alias 引用）

#### 代码重构阶段 1.2：SettingsOther.vue 改用统一配置入口
- 现象：`SettingsOther.vue` 的 `logLevel` 使用调试用 `getConfigValue('Log', 'level')` / `setConfigValue('Log', 'level', String(level))` 读写，违反"统一走 `applyConfig`/`getConfigMap`"约定
- 修复：`loadLogLevel` 改用 `getConfigMap()` 取 `cfg.logLevel`；`saveLogLevel` 改用 `applyConfig({ logLevel: level })`（后端 `apply_config` 会同步调用 `logger::set_level` 立即生效）
- 范围说明：`stores/java.ts` 的 `getConfigValue('Java', 'path')` 不在本次修复范围——后端 `commands/system/config.rs:94` 明确注释"Java path 不在 AppConfig 中，走 INI [Java] 独立存储"，是有意设计，不属于违规（frontend: src/views/settings/SettingsOther.vue）

#### 代码重构阶段 1.1：修复 useVersionGroups 遗漏 LiteLoader 检查
- 现象：含 LiteLoader 的资源版本不会被分到 "LiteLoader 1.12.2" 等独立分组
- 根因：`loaderNames(flags)` 函数只检查 Forge/NeoForge/Fabric/Quilt，遗漏 `ModLoaderFlags.LiteLoader`
- 修复 1：`loaderNames` 末尾增加 `if (flags & ModLoaderFlags.LiteLoader) list.push('LiteLoader')`，顺序与 `typeMetaMap.order` 对齐（Forge→NeoForge→Fabric→Quilt→LiteLoader）（frontend: src/composables/useVersionGroups.ts）
- 修复 2：版本号排序时去掉加载器前缀的正则同步补充 LiteLoader：`/^(Fabric|Forge|NeoForge|Quilt|LiteLoader)\s+/`

#### Mod 详情按钮等待 3 秒才弹窗（预加载已完成但不知道）
- 现象：用户点击某个 mod 详情按钮，没有任何网络请求，等了 3 秒才弹本地信息弹窗
- 根因：该 mod 的 jar 内没有 metadata（没有 fabric.mod.json/mods.toml/mcmod.info），所以 slug 一直为空。`handleShowInfo` 在 slug 为空时固定等待 3 秒（30次 × 100ms 轮询），但缓存命中时预加载在 14ms 内就全部完成了，slug 不会再变化，3 秒等待完全浪费
- 修复 1：后端预加载完成时 emit `mods-preload-done` 事件（缓存命中和正常完成两个路径都 emit）（backend: src-tauri/src/minecraft/community/preload.rs）
- 修复 2：前端 `useModsPreload` 新增 `isPreloadDone` 状态，监听 `mods-preload-done` 事件（frontend: src/composables/useModsPreload.ts）
- 修复 3：`handleShowInfo` 在 slug 为空时先判断 `isPreloadDone`：若已完成则立即走本地信息弹窗，不等待；若未完成才进入等待循环，且循环中每次也检查 `isPreloadDone`，一旦完成立即跳出（frontend: src/views/version-settings/ModTab.vue）

#### 启动时错误跳转到登录页
- 现象：已登录用户每次启动应用都进入登录页，不应该是首页
- 根因：`restoreSession()` 是异步的（在 App.vue onMounted 中调用），但路由守卫 `beforeEach` 在应用启动时就同步执行，此时 `currentUser` 还是 null，`isLoggedIn = false`，导致 `requiresAuth` 路由被重定向到 `/login`
- 修复 1：auth store 新增 `isRestoring` 标志（初始 true，restoreSession 完成后置 false）（frontend: src/stores/auth.ts）
- 修复 2：路由守卫在 `isRestoring = true` 期间放行所有路由，不拦截 requiresAuth 路由。App.vue 有 isRestoring 加载遮罩覆盖整个恢复期，用户看不到路由变化（frontend: src/router/index.ts）
- 修复 3：App.vue 在 `restoreSession` 完成后主动修正路由：已登录但停在 /login → 跳首页；未登录但停在 requiresAuth 页面 → 跳登录页（frontend: src/App.vue）
- 修复 4：已登录用户访问 /login 时路由守卫自动重定向到首页（避免登录后又手动回到登录页）

#### 版本筛选 tag 对新格式快照版适配
- 现象：Minecraft 新版本号格式（如 `26.2-snapshot-2` / `26.2-snapshot-3`）被识别为两个独立的筛选 tag，应该归为同一个 `26.2`
- 根因：`getFilterVersionName` 中 `name.split('.').slice(0, 2).join('.')` 对 `26.2-snapshot-2` 的 split 结果是 `["26", "2-snapshot-2"]`，slice(0,2).join('.') 后仍然是 `"26.2-snapshot-2"`，没有截断
- 修复：在截断到二级版本号之前，先用 `name.split('-')[0]` 去掉 `-snapshot-数字` 等后缀，这样 `26.2-snapshot-2` 和 `26.2-snapshot-3` 都会归到 `26.2` tag（frontend: src/composables/useVersionGroups.ts）

#### Mod 默认 logo 改为图片
- 无 jar logo 的 mod 项不再显示加载器首字母色块，改为显示 `assets/Mods/default-min.png` 默认图片（frontend: src/views/version-settings/ModTab.vue）

#### Mod 列表两阶段加载（完整对齐 PCL2，秒加载 + 排序修复）
- 现象：用户反馈每次进入 Mod 列表都要等好几秒（143 个 mod 要等 jar 元数据全部读完），PCL2 进入基本秒加载；且禁用的 mod 总是被排到列表末尾
- 根因分析（参考 PCL2 `LocalResourceLoaders.vb`）：
  - PCL2 同步阶段**只做文件枚举**（`DirectoryUtils.GetFiles`），完全不读 JAR 内容，所以瞬间返回
  - PCL2 排序规则只按 `File.Name`（含扩展名）字母序升序，**禁用状态不参与排序**（第 88 行 `ModList.OrderBy(Function(m) m.File.Name)`）
  - MoLaunch 原 `list_mods` 对每个 jar 同步调用 `read_mod_metadata`（打开 jar + 读 fabric.mod.json/mods.toml/mcmod.info + 提取 logo base64 + 查 mcmod 译名），143 个 mod = 143 次磁盘 IO，这是慢的根本原因
  - MoLaunch 原排序规则「启用的排前面 + 文件名升序」导致禁用的 mod 被挤到末尾
- 修复 1：`list_mods` 极致轻量化（backend: src-tauri/src/commands/version/mods.rs）
  - 去掉 `read_mod_metadata` 调用，元数据字段（translated_name/description/version/logo_data/slug）全部返回空
  - 只做文件枚举 + 获取文件大小 + 推断加载器类型（从文件名），保证瞬间返回
  - 排序改为只按 `file_name`（含扩展名）字母序升序，禁用状态不参与排序（与 PCL2 一致）
- 修复 2：把 JAR 元数据读取合并到 preload 阶段（backend: src-tauri/src/minecraft/community/preload.rs）
  - `read_mod_metadata` 从 private 改为 `pub(crate)`，返回类型从元组改为 `ModMetadata` 结构体
  - 新增 `finalize_metadata` 辅助函数统一处理 logo 提取 + 译名查询
  - `preload_mods_detail` 重构为两阶段：先用 `tokio::task::spawn_blocking` + `Semaphore(8)` 并发读所有 jar 元数据 + 算 hash，每读完一个就 emit 元数据事件（前端立即看到译名、logo、版本）；再批量查 CF/MR project，每查到一个 emit project 事件
  - `PreloadUpdate` 结构扩展：新增 slug/description/version/logo_data/translated_name 字段（原只有 project）
  - 缓存结构扩展：`CachedMod` 存储完整元数据 + project，第二次进入直接从缓存恢复所有信息（真正秒加载）
  - 缓存版本号从 1 升到 2（旧缓存自动失效）
- 修复 3：前端 `useModsPreload` 扩展（frontend: src/composables/useModsPreload.ts）
  - 事件 payload 新增 slug/description/version/logo_data/translated_name 字段
  - 监听器按字段是否 undefined 决定是否更新（支持分次 emit：元数据先到、project 后到）
- 修复 4：`handleShowInfo` 等待逻辑（frontend: src/views/version-settings/ModTab.vue）
  - 用户点详情按钮时如果 slug 还为空（预加载还没读到 jar 元数据），先等待最多 3 秒（每 100ms 检查一次），等待期间显示 spinner
  - 等待后仍无 slug 才走本地信息弹窗；等待期间 project 就绪则直接弹窗

#### Mod 启用/禁用原地更新（不刷新列表、不重排位置）
- 现象：用户反馈禁用 Mod 后整个列表重新加载（视觉闪烁），且禁用的 Mod 自动窜到列表最后（被后端排序规则「启用的排前面」挤到禁用区末尾），设计不合理
- 根因：`handleToggleMod` 调用 `await loadMods()` 重新加载整个列表，触发后端排序 + 丢失预加载的 `project` 字段（list_mods 返回时 project 为空，需要重新等预加载）
- 后端 `toggle_mod` 命令返回类型从 `Result<(), String>` 改为 `Result<String, String>`，返回重命名后的新文件名（backend: src-tauri/src/commands/version/mods.rs）
- 前端 `toggleMod` 封装同步改为 `Promise<string>`（frontend: src/utils/api/personalization.ts）
- 前端 `handleToggleMod` 改为原地更新：按 `file_name` 找到对应 mod，用整对象替换更新 `file_name` + `is_enabled` 字段（保留 `enabled_name`、`project` 等其他字段不动）。mod 在列表中的位置完全不动，预加载的 project 字段也保留。工具栏统计 `enabledCount` / `disabledCount` 是 computed，会自动同步更新（frontend: src/views/version-settings/ModTab.vue）

#### 版本设置 - Mod 管理工具栏固定（列表独立滚动）
- 原布局：Mod 管理页与 OverviewTab/SetupTab 共用 VersionSettings.vue 的 `flex-1 overflow-y-auto p-6` 滚动容器，工具栏用 `sticky top-0` 尝试固定。因父容器 padding 存在，列表滚动时会从工具栏上方和两侧 padding 间隙穿过，视觉上列表滑过了操作栏
- 修改 VersionSettings.vue 内容区容器：按 `activeCategory` 切换 class，mod tab 用 `flex-1 flex flex-col overflow-hidden`（不滚动），其他 tab 保持原 `overflow-y-auto p-6`（frontend: src/views/VersionSettings.vue）
- 修改 ModTab.vue 为独立 flex 布局：根容器 `flex flex-1 flex-col overflow-hidden`，工具栏 `flex-none border-b`（永远固定不滚动），列表区单独包进 `flex-1 overflow-y-auto p-6`（只有列表滚动）。移除原 sticky/负 margin 方案（frontend: src/views/version-settings/ModTab.vue）

#### Mod 列表加载状态统一 + 打开文件位置修复
- Mod 列表加载状态改为项目统一样式（与 VersionSelect.vue 一致）：大尺寸（h-8 w-8）svg spinner 居中垂直排列 + 文字，替代原内联小尺寸（h-4 w-4）水平排列（frontend: src/views/version-settings/ModTab.vue）
- 修复 Mod「打开文件位置」按钮打开的是「文档」库而非实际位置（backend: src-tauri/src/commands/system/game_dir.rs）：根因是 `reveal_in_explorer_impl` 用 `Command::new("explorer").arg(format!("/select,{}", path))`，Rust Command::arg 会给含空格的整个参数加引号，导致 explorer 收到 `"/select,C:\path with spaces\file.jar"` 后不识别 `/select` 开关，把整个字符串当路径找不到，回退到文档库。原记忆「explorer /select,<path> 形式带引号能正确解析」结论有误，实际与裸路径行为一致都会回退。修复：改用 `cmd /c` 构造完整命令 `explorer /select,"<path>"`，手动给路径加引号，explorer 正确解析 /select 开关和带引号的路径（与 open_path_impl 修复方式一致）

### 重构

#### Shell 命令统一封装到 minecraft::system::shell 模块
- 新增 `src-tauri/src/minecraft/system/shell.rs` 统一 shell 命令封装模块，整合所有系统级外部命令调用：
  - 文件管理器交互：`open_path`（打开文件夹）、`reveal_in_file_manager`（选中文件）—— 原 `game_dir.rs` 的 `open_path_impl`/`reveal_in_explorer_impl`
  - 进程管理：`kill_process_tree`（杀进程树）—— 原 `launch.rs` 的 taskkill/kill
  - 文件权限：`restrict_file_permissions`（限制权限为当前用户）—— 原 `script_export.rs` 的 icacls/chmod
- 所有 shell 命令统一：跨平台差异处理（Windows/macOS/Linux）、安全校验（拒绝路径遍历 `..` 和 UNC 路径 `\\`）、统一日志（`[Shell]` 前缀 + 调用前后都记录）、错误转换为 String
- 重构调用方：
  - `game_dir.rs`：删除 `open_path_impl`/`reveal_in_explorer_impl` 实现（约 90 行），Tauri 命令 `open_path`/`reveal_in_explorer`/`open_game_dir` 改为转发调用 shell 模块（backend: src-tauri/src/commands/system/game_dir.rs）
  - `launch.rs`：删除 taskkill/kill 平台分支代码，`stop_game` 改为调用 `shell::kill_process_tree(pid)`（backend: src-tauri/src/commands/version/launch.rs）
  - `script_export.rs`：删除 `restrict_script_permissions` 函数（约 30 行），改为直接调用 `shell::restrict_file_permissions`（backend: src-tauri/src/commands/version/script_export.rs）
  - `mods.rs`：`open_mods_dir`/`reveal_mod_file` 调用方从 `crate::commands::system::open_path_impl`/`reveal_in_explorer_impl` 改为 `crate::minecraft::system::shell::open_path`/`reveal_in_file_manager`（backend: src-tauri/src/commands/version/mods.rs）

#### reveal_in_file_manager 改用 ShellExecuteW（彻底修复打开位置错误）
- 现象：用户反馈 Mod「打开文件位置」按钮打开的是「此电脑」而非实际位置。日志显示 `cmd /c explorer /select,"<path>"` 命令确实执行了，但 explorer 回退到默认位置
- 根因：Rust `Command` 在 Windows 上对含空格和引号的 arg 会加引号并转义内部引号（`"` → `\"`），但 cmd.exe **不识别 `\"` 转义**。故障链：
  1. Rust 构造 arg `explorer /select,"C:\path with spaces\file.jar"`（含空格和引号）
  2. Rust 转义后传给 cmd.exe：`cmd /c "explorer /select,\"C:\path with spaces\file.jar\""`
  3. cmd.exe 去掉首尾引号（规则：首尾是 `"` 且内部有引号时去掉），不处理 `\"` → `explorer /select,\"C:\path with spaces\file.jar\"`（字面反斜杠引号）
  4. explorer 收到 `/select,\"...\"` 参数，解析失败，回退到默认位置「此电脑」
- 修复：Windows 平台 `reveal_in_file_manager` 改用 Win32 API `ShellExecuteW` 直接调用 explorer.exe（backend: src-tauri/src/minecraft/system/shell.rs）。ShellExecuteW 接收 UTF-16 字符串作为参数，绕过 Rust `Command` 的参数转义和 cmd.exe 的引号解析，explorer.exe 直接收到原始参数 `/select,"<path>"`，用 `CommandLineToArgvW` 正确解析引号并合并路径
- 前两次修复方案均无效的原因：
  - 第一次 `Command::new("explorer").arg(format!("/select,{}", path))`：Rust 给含空格的整个 arg 加引号，explorer 不识别 `/select,"..."`
  - 第二次 `cmd /c explorer /select,"<path>"`：cmd.exe 不识别 Rust 的 `\"` 转义，explorer 收到字面反斜杠引号
- ShellExecuteW 是 Windows 95+ API，完全兼容所有 Windows 版本，是 Windows 上启动程序最可靠的方式

#### 整合包安装应用 community_filename_format 命名规范
- 现象：用户反馈整合包安装下载的 mod 没有按设置里的 `community_filename_format` 命名（如设置了 `[译名] 原名`，整合包安装后 jar 文件名仍是原始名）
- 根因：`install_cf_mods` 和 `install_mr_files` 直接用原始 `file_name` / `path` 作为目标文件名，没有调用 `apply_filename_format`，也没有查询 mcmod 译名
- 修复 CF 整合包（backend: src-tauri/src/commands/community/install.rs `install_cf_mods`）：
  - 用 manifest 中的 `project_id` 列表，调 CF 批量接口 `GET /v1/mods?modIds=...` 查询 mod info 拿 slug
  - 通过 manifest 关联 `file_id ↔ project_id`，构造 `file_id → 译名` 映射（slug → `mcmod::lookup_cf` 查译名）
  - 对每个下载文件调用 `apply_filename_format(orig_name, translated, config.community_filename_format)` 重命名
- 修复 MR 整合包（backend: src-tauri/src/commands/community/install.rs `install_mr_files`）：
  - 新增 `extract_mr_project_id` 从 `downloads` URL 提取 project_id（URL 格式 `https://cdn.modrinth.com/data/<id>/versions/...`）
  - 用提取的 project_id 列表，调 MR 批量接口 `GET /projects?ids=[...]` 查询 project info 拿 slug
  - 构造 `file_index → 译名` 映射（slug → `mcmod::lookup_mr` 查译名）
  - 仅对 `mods/` 路径下的文件应用 `apply_filename_format`（resourcepacks/shaderpacks 保留原名，mcmod 数据库只覆盖 mod）
- 新增 CF 批量查询 mod info 接口（backend: src-tauri/src/minecraft/community/curseforge.rs `batch_get_mod_slugs`）：调 `GET /v1/mods?modIds=...` 返回 `modId → slug` 映射，失败时返回空 map 不阻断下载
- 新增 MR 批量查询 projects 接口（backend: src-tauri/src/minecraft/community/modrinth.rs `batch_get_project_slugs`）：调 `GET /projects?ids=[...]` 返回 `project_id → slug` 映射，失败时返回空 map 不阻断下载
- 失败容错：所有译名查询失败时返回空 map，下载流程继续，仅文件名不应用格式（与 PCL2 行为一致，不让网络问题阻断整合包安装）

#### Mod 管理「详情」按钮关联社区资源 + 新增「前往百科」按钮
- 现象：用户参考 PCL2，希望 Mod 管理列表的「详情」按钮能直接打开社区资源详情弹窗（即搜索 mod 时弹出的 ResourceDetail），而不是只显示本地信息；无法关联的 mod 才回退到本地信息弹窗；另外希望加个「前往百科」按钮直接打开 mcmod.cn
- 后端 `ModInfo` 新增 `slug: String` 字段，`read_mod_metadata` 返回元组扩展为 `(translated, description, version, logo, slug)`，把从 jar 内 metadata 读到的 slug（fabric.mod.json 的 id / mods.toml 的 modId / mcmod.info 的 modid）带回前端用于关联 CF/MR 平台工程（backend: src-tauri/src/commands/version/mods.rs）
- 前端 `ModInfo` interface 同步新增 `slug: string` 字段（frontend: src/utils/api/personal.ts）
- ModTab.vue「详情」按钮逻辑改造（参考 PCL2 MyLocalModItem.Info_Click）：
  - 有 slug：先调 `getProjectDetail('CurseForge', slug, 'Mod')`（CF API 支持用 slug 查询 mod），失败再调 `getProjectDetail('Modrinth', slug, 'Mod')`（MR API 同样支持 slug 查询）。成功则弹出复用的 `ResourceDetail` 组件展示完整 mod 详情（版本、下载、描述等），与社区资源搜索的详情弹窗完全一致
  - 失败或无 slug：回退到 `showLocalModInfo` 显示本地信息弹窗（描述、文件、版本、译名、加载器），与原行为一致
  - 「详情」按钮加载期间 disabled 防止重复点击（frontend: src/views/version-settings/ModTab.vue）
- 新增「前往百科」按钮（在「详情」按钮右侧，hover 列表项时显示，参考 PCL2 PageDownloadCompDetail.BtnIntroWiki_Click）：
  - 有 slug：先调 `getMcmodUrl('CurseForge', slug)` 查 mcmod.cn 直链，无则调 `getMcmodUrl('Modrinth', slug)`（CF 收录更全，优先 CF）。查到直链则用 `@tauri-apps/plugin-shell` 的 `open` 打开 `https://www.mcmod.cn/class/<id>.html`
  - 查不到直链或无 slug：打开 mcmod.cn 搜索页 `https://www.mcmod.cn/search?key=<keyword>`，关键字优先用 `translated_name`，其次用文件名去 `.jar` / `.disabled` / `.old` 后缀
- 操作按钮从 4 个变为 5 个（详情 / 前往百科 / 打开文件位置 / 启用禁用 / 删除），按钮 hover 配色：详情=蓝、前往百科=翠绿、打开文件位置=灰、启用=绿/禁用=橙、删除=红

#### Mod 详情按钮体验优化（并发请求 + 防呆 spinner + 预取上下文）
- 现象：用户反馈点击「详情」按钮后才发请求有明显卡顿感；串行 CF → MR 查询若 CF 失败还要等 MR；用户不知道是否点到了按钮会重复点击
- 并发请求：`handleShowInfo` 改用 `Promise.any` 并发请求 CF + MR，谁先成功用谁（响应时间从 `CF + MR` 缩短为 `max(CF, MR)`），全部失败才回退本地信息弹窗（frontend: src/views/version-settings/ModTab.vue）
- 防呆 spinner：把 `detailLoading` 单一布尔改为 `detailLoadingFor: string | null`（记录当前加载中的 mod file_name），按钮显示旋转 spinner + Tooltip 文字变为「正在加载详情...」+ 禁用同 mod 重复点击。加载中时按钮容器强制 `opacity-100`，避免鼠标离开列表项后 spinner 隐藏让用户以为没点到（frontend: src/views/version-settings/ModTab.vue）
- 预取上下文：onMounted 时调用 `prefetchVersionContext` 异步获取整合包的 MC 版本号和 mods 目录路径，避免用户点击详情按钮后再请求造成卡顿（frontend: src/views/version-settings/ModTab.vue）

#### Mod 详情弹窗自动选中整合包版本 + 下载默认到 mods 文件夹
- 现象：用户希望从 ModTab 打开资源详情弹窗后，自动选中顶部筛选 tag 为整合包对应的版本；点击下载按钮时，saveFile 对话框默认定位到整合包的 mods 文件夹
- 后端新增 `get_version_game_version` Tauri 命令（backend: src-tauri/src/commands/version/list.rs）：从版本 JSON 提取 MC 版本号（如 "1.20.1"），复用 `version::scan::extract_original_version` 的解析策略（inheritsFrom → --fml.mcVersion → downloads URL 正则 → jar → id 正则）。把 `extract_original_version` 改为 `pub(crate)` 供 commands 层复用（backend: src-tauri/src/minecraft/version/scan.rs）
- 后端新增 `get_version_mods_dir` Tauri 命令（backend: src-tauri/src/commands/version/mods.rs）：返回版本 mods 目录绝对路径（自动创建），用于前端下载 mod 时指定默认保存位置
- 前端封装（frontend: src/utils/api/personal.ts）：新增 `getVersionGameVersion` / `getVersionModsDir` 调用上述两个命令
- ResourceDetail 新增 `gameVersion` 和 `modsDir` 两个可选 prop（frontend: src/components/community/ResourceDetail.vue）：
  - `gameVersion`：版本列表加载完成后，调 `getFilterVersionName` 把 "1.20.1" 截断成 "1.20"，匹配 filterOptions 后 `setFilter` 自动选中顶部筛选 tag
  - `modsDir`：`handleDownload` 调 `saveFile` 时作为 `defaultDirectory` 传入，对话框默认打开到 mods 文件夹
- 导出 `getFilterVersionName` 函数（frontend: src/composables/useVersionGroups.ts）供 ResourceDetail 调用
- ModTab 把预取的 `versionGameVersion` / `versionModsDir` 通过 prop 传给 ResourceDetail（frontend: src/views/version-settings/ModTab.vue）

#### 百科搜索 URL 修正 + 关键字去版本号
- 现象：用户反馈百科搜索 URL 应该是 `https://search.mcmod.cn/s?key=` 而不是 `https://www.mcmod.cn/search?key=`；搜索关键字不应包含版本号（如 "AI-Improvements-1.20-0.5.2" 应截断为 "AI-Improvements"），否则百科搜索匹配不到结果
- URL 修正（frontend: src/views/version-settings/ModTab.vue `handleOpenWiki`）：`https://www.mcmod.cn/search?key=` → `https://search.mcmod.cn/s?key=`
- 关键字截断：新增 `stripModVersion` 函数，用正则 `^([^-\s+_]+(?:[-\s+_][^-\s+_]+)*?)[-+_]\d+\.\d+` 在第一个版本号位置截断。例：`AI-Improvements-1.20-0.5.2.jar` → `AI-Improvements`，`FabricAPI-0.92.2+1.20.4` → `FabricAPI`，`create-1.20.1-6.0.4.jar` → `create`

#### save_file 命令支持默认目录 + MR 404 优雅处理
- 后端 `save_file` Tauri 命令新增 `default_directory: Option<String>` 参数（backend: src-tauri/src/commands/system/game_dir.rs）：调用 `dialog.set_directory(path)` 设置对话框默认打开目录，路径不存在时静默忽略。用于从 ModTab 打开资源详情下载 mod 时默认定位到 mods 文件夹
- 前端 `saveFile` 封装同步新增 `defaultDirectory` 可选参数（frontend: src/utils/api/system.ts）
- MR API 404 优雅处理（backend: src-tauri/src/minecraft/community/modrinth.rs `mr_get`）：
  - 现象：用户日志显示 `[WARN] MR 响应解析失败: https://api.modrinth.com/v2/project/oworld2create (error decoding response body: EOF while parsing a value at line 1 column 0)`。根因是 MR 对不存在的 slug 返回 404 + 空 body，`resp.json()` 解析空 body 报 "EOF while parsing"，警告日志让用户以为出错了
  - 修复：抽出 `parse_resp` 辅助函数，先检查 HTTP 状态码，404 时返回 "Modrinth 资源不存在" 错误并记 INFO（不报警告），非 2xx 记警告，2xx 才调 `resp.json()`。镜像回退路径也复用 `parse_resp`
  - 404 时跳过镜像回退：source=1 时官方失败原本会回退镜像，但 404 表示资源真不存在（镜像也是 404），重试无意义，通过 `is_not_found` 判断跳过

#### CF slug 查询修复（missing field `data` 警告消除）
- 现象：用户日志显示 `[WARN] CF 响应解析失败: https://mod.mcimirror.top/curseforge/v1/mods/oworld2create (error decoding response body: missing field 'data' at line 1 column 110)`。根因是 CF API `/v1/mods/<id>` 只接受数字 modId，对非数字 slug 镜像返回不含 `data` 字段的不同响应结构
- 修复（backend: src-tauri/src/minecraft/community/curseforge.rs `get_project`）：判断 `project_id` 是否全数字，数字走原 `/mods/<id>` 路径，非数字走 `/mods/search?gameId=432&slug=<slug>` 搜索接口取首个结果。两种响应结构分别用不同 struct 反序列化，避免 "missing field" 错误






### 新增

#### Mod 详情预加载架构（完整对齐 PCL2 LocalResourceOnlineLoader）
- 设计目标：参考 PCL2 `MyLocalModItem.Info_Click`（PageInstanceMod.xaml.vb 第 751-792 行）的核心设计——**详情按钮本身不发任何网络请求**，只判断 `Entry.Project` 是否已被预加载填充，实现零延迟跳转。预加载由 `LocalResourceOnlineLoader` 在 `list_mods` 返回后立即后台执行（哈希批量查询 + 工程详情拉取）
- 后端新增预加载核心模块（backend: src-tauri/src/minecraft/community/preload.rs）：
  - MurmurHash2 算法实现（CF 指纹算法）：读取文件字节后**跳过空白字节**（0x09/0x0A/0x0D/0x20），再用 seed=1、m=0x5bd1e995、r=24 计算（与 PCL2 `LocalResourceFile.vb` 第 417-490 行实现一致）
  - SHA1 hash 计算（MR 文件识别算法，标准 SHA1）
  - `preload_mods_detail` 主入口流程：1) 读持久化缓存 → 2) 计算每个 mod 的 CF MurmurHash2 + MR SHA1 → 3) `tokio::join!` 并发批量查询 CF/MR → 4) 合并结果（CF 优先，MR 兜底）→ 5) 每查到一个 project 就 `app.emit("mods-preload-update", ...)` → 6) 写入持久化缓存
  - 持久化文件缓存：`.Molaunch/cache/preload_mods/{version_id}.json`，6 小时 TTL + 版本号 gating（版本号变化强制刷新，参考 PCL2 `Cache/LocalMod.json` 的 key=`ModrinthHash + VanillaVersion + ModLoaders`）
  - Tauri 事件推送：`PreloadUpdate { version_id, file_name, project }`，每个 mod 查到后单独 emit，前端逐个响应式更新
- 后端新增 CF 批量指纹查询（backend: src-tauri/src/minecraft/community/curseforge.rs）：
  - `build_cf_post_request` + `cf_post<T>` 辅助函数（POST 请求携带 API Key + JSON body，支持 source 策略 + 镜像回退）
  - `fingerprint_search` 4 步批量查询：1) POST `/v1/fingerprints/432` 带 fingerprints 数组 → exactMatches → 2) 收集 modIds 构造 fingerprint→modId 映射 → 3) POST `/v1/mods` 带 modIds 数组批量查工程详情 → 4) 映射 fingerprint→ResourceProject 并缓存
- 后端新增 MR 批量 SHA1 查询（backend: src-tauri/src/minecraft/community/modrinth.rs）：
  - `mr_post<T>` 辅助函数（POST 请求 + source 策略 + 404 跳过镜像回退 + 镜像回退，与 `mr_get` 行为一致）
  - `version_files_search` 4 步批量查询：1) POST `/v2/version_files` 带 SHA1 hashes → 返回 version+project_id 映射 → 2) **校验 file.hashes.sha1 与查询 sha1 匹配**（防 MR 返回错位）→ 3) GET `/v2/projects?ids=[...]` 批量查工程详情 → 4) 映射 sha1→ResourceProject 并缓存
- 后端新增 `preload_mods_detail_cmd` Tauri 命令（backend: src-tauri/src/commands/version/preload.rs）：扫描版本 mods 目录构造 `PreloadModInput` 列表（file_name + 文件字节），`tokio::spawn` 异步任务不阻塞命令返回；注册到 `lib.rs` invoke_handler
- 后端 `get_mods_dir` 从 private 改为 `pub(crate)`（backend: src-tauri/src/commands/version/mods.rs）供 preload 命令复用
- 前端新增 `useModsPreload` composable（frontend: src/composables/useModsPreload.ts）：`listen<PreloadUpdatePayload>('mods-preload-update', cb)` 监听事件，按 `file_name` 匹配 mods 数组对应项，用 `mods.value[i] = { ...mods.value[i], project }` 确保 Vue 响应式触发（直接赋值属性不会触发）
- 前端 `ModInfo` 新增 `project?: ResourceProject` 字段（frontend: src/utils/api/personal.ts）并封装 `preloadModsDetail(versionId)` 调用上述命令
- 前端 ModTab.vue `handleShowInfo` 改造为三级 fallback（参考 PCL2 `MyLocalModItem.Info_Click`）：
  1. **零延迟路径**：`mod.project` 已被 `preload_mods_detail_cmd` 后台预加载填充 → 直接弹 ResourceDetail（与 PCL2 `Entry.Project IsNot Nothing` 分支一致）
  2. **并发 fallback**：预加载未就绪（用户点太快）或预加载失败 → `Promise.any` 并发请求 CF + MR，谁先成功用谁
  3. **本地信息**：无 slug 或两个平台都查不到 → 弹本地信息弹窗 + 百科搜索按钮（与 PCL2 `Else` 分支一致）
- onMounted 启动预加载事件监听（必须在 `loadMods` 之前，避免错过早期事件）→ `loadMods` → `prefetchVersionContext` → `preloadModsDetail`（后台异步，不阻塞 UI）；onUnmounted 停止监听

#### 整合包安装：完整流程（后端 + 前端）
- 新增 `install_modpack` Tauri 命令（backend: src-tauri/src/commands/community/install.rs），参考 PCL2 ModModpack.vb ModpackInstall 实现
- 格式自动识别：下载原始整合包到 `versions/{instance_name}/`，用 zip 根目录关键文件判定格式：
  - `manifest.json` → CurseForge 整合包
  - `modrinth.index.json` → Modrinth 整合包
- CurseForge 整合包前置 API Key 检查（用户需求）：调用 `secure_storage::get_config_async()` 异步获取配置，若未启用或未配置 API Key，在最开始就返回错误，错误信息明确指向「设置 → 社区资源」
- CurseForge 完整安装：
  - 解析 `manifest.json` → 提取 `minecraft.version` + `modLoaders[].id`（解析为 loader 名+版本）+ `files[]`
  - POST `/v1/mods/files` 批量查询所有 file 的下载信息（一次请求拿到全部 URL，避免逐个查询）
  - `download_files_concurrent` 基于 `tokio::sync::Semaphore` 并发下载所有 mods 到 `versions/{instance}/mods/`，原子计数器汇总进度
- Modrinth 完整安装：
  - 解析 `modrinth.index.json` → 从 `dependencies.minecraft` 提取游戏版本，从 `fabric-loader`/`quilt-loader`/`forge`/`neoforge` 提取加载器信息
  - 遍历 `files[]` 直接下载（含 `downloads` URL）到 `instance_dir/{path}`
- overrides 解压：解压 `overrides/` + `client-overrides/` 前缀的文件到 instance 目录（同时支持 CF 与 MR 格式）
- 进度共享 download_state：安装全程走 `state.download_state`（与版本下载共用），4 个加权阶段（下载整合包 10 / 解析 1 / 下载依赖 40 / 复制配置 5），由 `DownloadPanel` + 下载管理页面统一展示（参考 PCL2 LoaderTaskbar + PageSpeedLeft 机制，不单独做弹窗/页面）
- 返回 `InstallModpackResult`（format/gameVersion/loader/loaderVersion/archivePath/instanceDir），前端据此调用 `install_merged` 安装游戏本体 + 加载器
- 前端新增 `handleInstallModpack`（frontend: src/components/community/ResourceDetail.vue），两段式调用：`installModpack`（整合包专属部分）→ `installMerged`（游戏本体+加载器），共享同一 `download_state`，DownloadPanel 连续展示
- 前端详情页版本按钮按 `resource_type` 分流：ModPack 类型显示「安装」按钮（RocketLaunchIcon）调用 `handleInstallModpack`，其他类型显示「下载」按钮（ArrowDownTrayIcon）调用 `handleDownload`（参考 PCL2 PageDownloadCompDetail SwapType=9 安装 / 8 另存为）

#### 整合包安装：并发下载进度与失败诊断修复
- 修复「下载速度/已下载字节」始终为 0：原 `download_single_file` 用 `resp.bytes().await` 一次性加载，从不更新 `bytes_downloaded`/`bytes_total`/`global_speed`（backend: src-tauri/src/commands/community/install.rs）
- 改为流式下载：`download_single_file_multi` 边接收边写文件，通过 `AtomicU64` 实时累积 `bytes_done`/`bytes_total`，前端能看到下载途中速度、累计字节持续增长（而非每个文件完成才跳一次）
- 新增 300ms 独立定时器任务：流式下载过程中定时调用 `update_modpack_progress` 刷新 `state.download_state` 的 stage 与 global 字段，参考 PCL2 `PageSpeedLeft` 300ms `DispatcherTimer` 轮询机制
- 修复 Modrinth 整合包部分文件下载失败（如 123/129 卡住）：原代码仅取 `downloads[0]`，遇到失效 URL 直接失败。改为传入 `downloads` 全部 URL 数组，按顺序尝试直到成功，参考 PCL2 ModpackInstall 多源回退
- 修复日志不完整：原失败时只 push 到 errors 列表，无任何 log_info。改为每个失败立即打印 `target_path`、尝试过的 URL 列表、错误信息；函数返回前汇总打印完整失败列表（编号 + URL + 错误），便于排查
- 失败错误信息从「仅第一个」改为「失败总数 + 首个错误」，方便快速判断是网络问题还是部分文件问题

#### install_merged 阶段错位修复 + 释放资源 hash 检查修复
- 修复「加载器安装」阶段堆很久但 MC 本体/库/assets 没分阶段显示（backend: src-tauri/src/commands/version/install.rs）：原 install_merged 只重置已有 stages 的 progress 但不替换列表内容，整合包安装的 4 个阶段（下载整合包/解析/依赖/overrides）残留，install_merged push 一个「加载器安装」正好凑 5 个。download_version_full 调 stage_callback(0..4) 与 stages 错位，导致前 4 个已 Finished 阶段被反复改 Loading，最后所有进度都堆到 stage[4]「加载器安装」
- 修复方式：install_merged 启动时清空 stages 重新设置为标准 5 阶段（版本清单/版本信息/客户端/库文件/资源文件，与 download_version_full 的 stage_callback 索引对应），按需追加「加载器安装」。修复后用户能看到 MC 本体、库文件、资源文件各自独立的进度条
- 修复「释放嵌入资源 hash 不匹配但实际文件不存在」警告（backend: src-tauri/src/resources.rs）：原代码只判断 `target_path.exists()`，当目标文件不存在但 `.sha256` 校验文件残留时，会读到旧 hash 触发「不匹配」警告。改为同时检查 target 和 hash 文件存在：两者都存在且匹配才跳过；只有一方存在时打印「缓存状态不一致」；两者都不存在时静默首次释放

#### 下载进度阶段分组展示（参考 PCL2 PageSpeedLeft 任务列表）
- 后端 `DownloadStage` 新增 `group: Option<String>` 字段（backend: src-tauri/src/state/mod.rs），用于按任务分组。整合包安装的 4 个阶段统一 `group="整合包安装"`，install_merged 追加的 5 个标准阶段 + 加载器阶段统一 `group="MC本体安装"`
- install_merged 改为追加模式（不再清空已有 stages）：保留整合包安装历史阶段显示，通过 `stage_offset` 修正 stage_callback/progress_callback 的索引偏移。这样整合包安装完成后，用户能在同一界面看到「整合包安装」+「MC本体安装」两个分组的完整进度历史
- 前端 Downloads.vue 改造为分组折叠展示（frontend: src/views/Downloads.vue）：按 `group` 字段聚合 stages，每个分组渲染为可点击折叠的卡片，展开看子阶段（版本清单/版本信息/客户端/库文件/资源文件/加载器安装）。默认全展开，用户点击可折叠
- 同步更新前端类型定义：`stores/version.ts` 的 `DownloadStage` 加 `group: string | null`；`composables/useDownloadPolling.ts` 的 `RawDownloadStage` 加 `group?` 并在映射时透传

#### 整合包 mod 下载复用 DownloadManager（自动分片下载）
- 整合包依赖文件下载改用 `DownloadManager`（backend: src-tauri/src/commands/community/install.rs），与 MC 本体/库/assets 走同一套下载基础设施，自动按文件大小触发分片下载（>1MB/chunk 走 `chunk::download_chunked`，小文件直连）
- 旧实现 `download_single_file_multi`（直接 reqwest 流式）已删除，避免维护两套下载逻辑
- CF/MR 文件列表新增 `file_size` 字段传入 `DownloadTask.expected_size`，DownloadManager 据此判断是否分片
- DownloadManager 的 progress_callback 接入 download_state，实时更新 stage bytes/files/speed
- 修复 stage_callback 在整合包+MC本体混合场景下的索引错位：原 `actual_index > 0` 条件在 stage_callback(0) 时会把 MC 本体第一个阶段（stage_offset）误标记为 Finished。改为 `actual_index > prev` 严格比较，避免误标

#### 整合包原始包下载也走分片下载 + 启动参数占位符替换修复
- 整合包原始包下载（Stage 0）从 `download_with_progress`（直接 reqwest 流式）改为 `DownloadManager::download_batch` 单任务（backend: src-tauri/src/commands/community/install.rs），自动按文件大小触发分片下载，与依赖文件下载同一套基础设施
- 旧实现 `download_with_progress` 函数已删除
- 修复整合包下载失败「所有下载源均失败」（backend: src-tauri/src/commands/community/install.rs）：根因是 stage 0 传入 `expected_size: 0`，DownloadManager 据此判断 `can_chunk=false`，走单流下载，而 Smart 模式下 cdn.modrinth.com 非镜像 URL 超时只有 5s，30MB 整合包 5s 超时不够，3 次重试都失败。修复：下载前先 GET + Range:bytes=0-0 探测 Content-Range/Content-Length，传入 `expected_size`，触发分片下载（每片 60s 超时，远高于单流的 5s）。不用 HEAD 是因为 Modrinth CDN 会 307 重定向到 cdn-alt，HEAD 请求在重定向后拿不到 Content-Length
- 修复启动 Forge 1.20.1 整合包报错 `InaccessibleObjectException: Unable to make field static final ... MethodHandles$Lookup.IMPL_LOOKUP accessible`（backend: src-tauri/src/minecraft/launch/mod.rs）：根因是 `build_jvm_args` 未替换 Forge JSON 中的 `${library_directory}`、`${classpath_separator}`、`${version_name}` 占位符，JVM 把字面量 `${library_directory}` 当作路径，导致 `-p`（module-path）指向不存在的路径，securejarhandler jar 没被加到 module path，进而 `--add-opens java.base/java.lang.invoke=cpw.mods.securejarhandler` 失效（JVM 警告 `Unknown module: cpw.mods.securejarhandler`），最终反射打开失败。修复：在 push 前替换所有 Mojang 占位符为实际值（`${library_directory}` → libraries 绝对路径，`${classpath_separator}` → `;`，`${version_name}` → version_id）

#### 下载日志与进度回滚修复
- 下载相关日志统一转换为人类可读大小（`format_bytes`）：之前所有下载日志（整合包大小、CF/MR 下载列表总大小、分片下载开始/完成、chunk 完成、探测文件大小）都打印原始 bytes 数字，如 `29975184 bytes`，难以阅读。现在统一格式化为 `28.6 MB` / `4726296 bytes` → `4.5 MB` 等（backend: src-tauri/src/minecraft/download/chunk.rs、src-tauri/src/minecraft/download/downloader.rs、src-tauri/src/commands/community/install.rs）
- DownloadManager 自动探测文件大小：原 install.rs 在 stage 0 手动用 GET + Range:bytes=0-0 探测整合包大小再传给 `expected_size`，重复了下载基础设施的判断逻辑。改为 `expected_size: 0` 直接交给 DownloadManager，由 `downloader.rs` 内部新增的 `probe_file_size` 函数自动探测（GET + Range:bytes=0-0 优先，HEAD 回退），安装代码无需感知细节（backend: src-tauri/src/minecraft/download/downloader.rs、src-tauri/src/commands/community/install.rs）。`probe_file_size` 解决了 Modrinth CDN 307 重定向后 HEAD 不返回 Content-Length 的兼容性问题
- 修复分片下载失败时已下载大小归零：原 `chunk::download_chunked` 在分片失败时只清理临时文件，但本次分片下载已增量加到 `file_progress.downloaded_bytes` 的字节数没回滚，导致下次重试时进度累加偏高甚至超过 total。修复：失败时调用 `saturating_sub(total_downloaded)` 回滚本次增量（backend: src-tauri/src/minecraft/download/chunk.rs）
- 修复单流下载进度不更新：原 `download_from_url` 的 `progress` 参数完全没用（变量名 `_progress`），单流下载过程中前端看不到任何进度推进，只能等文件完成后跳一次。改为流式接收时每 chunk 增量更新 `progress.downloaded_bytes`，与分片下载保持一致（backend: src-tauri/src/minecraft/download/downloader.rs）
- 单流下载失败时同样回滚 progress：新增闭包 `rollback_progress(downloaded, &progress)`，在流读取错误/写文件错误/超过 `byte_limit` 时统一回滚已增量加的字节数（backend: src-tauri/src/minecraft/download/downloader.rs）
- 修复下载速度一直显示「计算中...」：原 `manager.rs` 的 `speed_window` 只在任务完成时 push，下载过程中窗口为空无法计算速度。改为在 300ms 定时器中持续 push `downloaded_bytes` 快照，下载过程中实时计算速度（backend: src-tauri/src/minecraft/download/manager.rs）
- 修复 `MutexGuard` 跨 `await` 导致的 `Send` 错误：timer 的 async block 中先 `lock().unwrap()` 拿到 `StdMutex::MutexGuard`，再 `await` 锁 `sw_for_timer`，`MutexGuard` 跨 `await` 持有不满足 `Send`。改为先 snapshot `downloaded_bytes` 的值（释放 `MutexGuard`），再 `await` 锁 `speed_window`（backend: src-tauri/src/minecraft/download/manager.rs）
- 修复任务完成时 `downloaded_bytes` 重复累加：原 `manager.rs` 在任务完成时 `p.downloaded_bytes += result.downloaded`，但分片下载和单流下载过程中已增量更新过，导致进度偏高甚至超过 total。移除完成时的重复累加，仅更新 `completed_files` 计数（backend: src-tauri/src/minecraft/download/manager.rs）
- 修复分片下载成功后被误判为校验失败导致重试 3 次：原 `downloader.rs` 分片下载完成后用 `task.expected_size`（=0）做校验，而 `FileChecker` 在 `actual_size==0` 且无 hash 时直接判定"无法校验"→ 删文件 → 重试。根因是 `probe_file_size` 探测出的真实大小只更新了局部变量 `file_size`，没传给 `FileChecker`。修复：分片下载和单流下载校验统一用 `file_size`（探测后的真实大小），不再用 `task.expected_size`；同时简化掉单流下载原来的 `if task.expected_size == 0 && downloaded > 0` fallback 分支（backend: src-tauri/src/minecraft/download/downloader.rs）
- 修复前端整合包总大小显示「计算中...」：`download_batch` 初始化 `total_bytes` 按 `task.expected_size`（=0）求和，`probe_file_size` 探测出的真实大小没回写到 `progress.total_bytes`。修复：探测成功后 `p.total_bytes = p.total_bytes.saturating_add(file_size)`（backend: src-tauri/src/minecraft/download/downloader.rs）
- 重构文件大小探测逻辑，真正"直接丢给分片"：删除 `downloader.rs` 的 `probe_file_size` 函数和 `download_single` 里的预探测代码，把探测逻辑内聚到 `chunk::download_chunked` 内部（file_size=0 时先 GET + Range:bytes=0-0 探测再分片，并回写 `total_bytes`）。`download_single` 的 `can_chunk` 判断改为：file_size 已知时按大小判断，file_size=0 时直接 `chunk_count > 1` 尝试分片，让分片模块自己探测。FileChecker 用 `chunk_result.total`（分片）或 `downloaded`（单流回退）。这样整合包原始包（expected_size=0）直接丢给分片，无需调用方探测（backend: src-tauri/src/minecraft/download/chunk.rs、src-tauri/src/minecraft/download/downloader.rs）

### 重构

#### 下载引擎统一：整合包安装 + MC 本体安装合并为同一套系统
- 背景：原 `install_modpack`（社区整合包安装）和 `install_merged`（MC 本体安装）各自维护独立的进度同步逻辑——整合包侧有 5 个私有辅助函数 + 300ms timer + 原子计数器 + 速度计算；MC 本体侧有内联 `speed_window`（VecDeque）+ 全局字节累加。两套逻辑重复且不一致
- 新增 `DownloadState` 统一方法（backend: src-tauri/src/state/mod.rs），8 个方法覆盖全部进度操作：
  - `reset_stages`：清空重置（独立安装流程用）
  - `append_stages`：追加保留（连续安装流程：整合包 → MC 本体，返回 offset 作索引偏移）
  - `set_current_stage`：切换当前阶段（自动把前一阶段标记 Finished）
  - `set_stage_status` / `set_stage_bytes`：本地操作（解析 zip、复制 overrides 等）
  - `sync_stage_from_progress`：核心统一方法——同步 `DownloadManager` 的 `GlobalProgress` 到指定阶段 + 累加所有 Finished/Loading 阶段的 bytes 到 global + 信任 DownloadManager 的 `current_speed`（它已有 300ms 滑动窗口，不再在前端重复维护 speed_window）
  - `mark_complete` / `mark_failed`：终态标记
- `install_modpack` 改造（backend: src-tauri/src/commands/community/install.rs）：删除 `reset_modpack_state` / `set_stage` / `update_stage_bytes` / `mark_failed` / `update_modpack_progress` 5 个私有辅助函数；`download_files_concurrent` 删除 300ms timer / `AtomicU64` 计数器 / `last_speed_check` / `is_active` 标志，改用 `sync_stage_from_progress`
- `install_merged` 改造（backend: src-tauri/src/commands/version/install.rs）：删除 `use std::collections::VecDeque` 和 `use std::time::Instant`（不再使用内联 speed_window）；progress_callback 和 stage_callback 改用统一方法 `sync_stage_from_progress` / `set_current_stage`；最终完成标记改用 `mark_complete()`
- `download_version_full` 改造（backend: src-tauri/src/minecraft/download/mod.rs）：复用单个 `DownloadManager` 实例（此前每阶段 new 一个独立 manager + 独立 timer）；`download_client_jar` / `download_libraries` / `download_assets` 签名简化为接收 `&DownloadManager`；`fix_version_files` 也改为复用单实例；`DownloadManager` 新增 `source_mode()` getter
- 修复整合包 → MC 本体连续安装时轮询提前停止：原 `install_modpack` 末尾调用 `mark_complete()` 设置 `is_complete=true`，前端轮询检测到后停止轮询并 `finishDownload()`，导致后续 `install_merged` 的进度完全无法显示。改为不调用 `mark_complete()`（仅标记最后一个 stage 为 Finished），由 `install_merged` 在全部完成后统一调用
- 修复前端 Downloads.vue 分组进度展示（加权平均 + 字节汇总 + 子阶段字节/文件进度），修正「剩余文件」标签
- 修复分片下载重试时 `progress.total_bytes` 重复累加导致前端显示总大小翻倍（28.6 MB → 57.2 MB）：根因是 `download_single` 的重试循环中 `file_size` 始终为 `task.expected_size`（=0），每次重试都重新调用 `download_chunked`，每次都探测文件大小并 `saturating_add` 到 `total_bytes`。修复：`file_size` 改为 `mut`，第一次探测到真实大小后记住（`file_size = chunk_result.total`），后续重试传入已知大小，不再触发探测和重复累加（backend: src-tauri/src/minecraft/download/downloader.rs）
- 修复分片下载失败 chunk 的增量进度未回滚导致 `downloaded_bytes` 偏高：原 `download_chunk` 在 stream 错误/写文件错误/超过 byte_limit 时直接返回 Err，但已增量加到 `file_progress.downloaded_bytes` 的字节数没回滚。`download_chunked` 只回滚成功 chunk 的 bytes（`total_downloaded`），失败 chunk 的增量残留。修复：`download_chunk` 新增 `rollback` 闭包，在所有 Err 返回路径前回滚本次增量加的字节数（与 `download_from_url` 的回滚逻辑一致）（backend: src-tauri/src/minecraft/download/chunk.rs）
- 优化 FileChecker 日志：整合包原始包等 `expected_size=0 + 无 hash` 的文件预检查时，原 `log_warn!("File check failed due to missing metadata")` 会让用户误以为是真正的校验失败。实际这是预期行为（无元数据无法校验 → 强制重下）。改为 `log_debug!`，仅诊断时可见（backend: src-tauri/src/minecraft/utils/file_checker.rs）

#### 配置更新接口统一
- 新增 `apply_config` 单一 IPC 命令取代此前分散在 6 个文件的 19 个 `set_*` setter 命令（backend: src-tauri/src/commands/system/apply_config.rs）
- 新增 `ConfigPatch` 结构体（所有字段 `Option<T>`，仅传需要改的字段），前端通过 `applyConfig(patch)` 一次性更新多字段，避免多次 IPC 往返和多次落盘
- 三段式分流处理：1) 校验阶段（mirror_url SSRF 防护、download_source/meta_source 枚举校验）2) 加密字段分流（CurseForge API Key 走 secure_storage SDK DES 加密）3) 普通字段统一更新（单次 update_config 闭包内赋值 + 联动 + 副作用）
- 删除 19 个旧 setter：`set_proxy_mode/type/url`、`set_mirror_url`、`set_download_source`、`set_meta_source`、`set_max_download_speed`、`set_max_download_threads`、`set_chunk_count`、`set_min_memory`、`set_max_memory`、`set_memory_mode`、`set_isolation_mode`、`set_log_level`、`set_curseforge_config`、`set_community_config`
- 保留 `set_game_dir` 和 `set_selected_version`（在版本切换流程中被内部调用）
- 保留 `set_config_value`（通用 INI 直写，用于调试/迁移）
- 顺手修复：删除 4 处 `_skip_reinit` 死参数；统一 `set_log_level` 行为（持久化 + 立即调用 `logger::set_level` 生效）
- 前端 `ConfigPatch` 类型与 `applyConfig` 函数（frontend: src/utils/api/system.ts）
- 4 个 Vue 组件改造为 `applyConfig(patch)` 调用：SettingsAdvanced.vue、SettingsDownload.vue、SettingsLaunch.vue、CommunityConfigCard.vue

#### apply_config 请求体与 get_config 对称
- 后端 `apply_config` 命令参数从 `patch: ConfigPatch` 改为 `entries: Vec<ConfigEntry>`，与 `get_config` 返回格式完全一致：`[{ "key": "proxyMode", "value": "none" }, ...]`（backend: src-tauri/src/commands/system/apply_config.rs）
- 后端内部把数组转为 `serde_json::Map` 再反序列化为 `ConfigPatch`，仅包含传入的字段会被更新
- `ConfigEntry` 结构体加 `Deserialize`（此前只有 `Serialize`，因为只有 get 用）
- `mirror_url` / `selected_version` 回到双层 Option 语义：`null` 表示清除（Some(None)），非空字符串表示设置，不传表示不更新
- 前端 `ConfigPatch` 类型 `mirrorUrl` / `selectedVersion` 为 `string | null`（null 表示清空，与 `getConfig` 返回格式一致）
- 前端 `applyConfig(patch)` 内部把对象转为 `ConfigEntry[]` 数组再 IPC：`invoke('apply_config', { entries })`，调用方式仍为 `applyConfig({ communitySource: 0 })`，IPC 格式与 `getConfig` 对称

#### 修复切换侧栏误触发 apply 的 bug
- `useDebouncedSave` 的 `onScopeDispose` 此前无条件调用 `flushSave`，导致组件卸载（切换侧栏）时即使无改动也会触发 `applyConfig` 请求（frontend: src/composables/useDebouncedSave.ts）
- 修复：新增 `dirty` 标志，`scheduleSave` 时置 `dirty=true`，`flushSave` 仅在 `dirty=true` 时才真正执行 `flushFn`；组件卸载时 `dirty=false` 直接跳过，避免误触发

#### useDebouncedSave 新增字段追踪模式（只传改变的字段）
- 此前组件 `flushSave` 无差别地把当前所有值塞进 `applyConfig`，用户只改一项也会把整栏所有字段一起发往后端覆盖写入（frontend: src/composables/useDebouncedSave.ts）
- 新增 patch 模式：`useDebouncedSave('patch', flushFn, delay)` 返回 `markDirty(key, value)`，watch 回调标记改过的字段，防抖触发后只把改过的字段传给后端
- 累积语义：用户在防抖窗口内连续改动多个字段（或跨组件累积），flush 时传所有改过的字段，未改的字段不传，后端保持原值不变
- 4 个 Vue 组件改用 patch 模式 + `markDirty`：
  - CommunityConfigCard.vue：4 个 watch → `markDirty('communitySource', v)` 等
  - SettingsAdvanced.vue：5 个 watch（proxyMode/proxyType/proxyUrl/curseforgeEnabled/curseforgeApiKey），删除手动 `pendingChanges` Set
  - SettingsDownload.vue：5 个 watch（metaSource/downloadSource/maxDownloadSpeed/maxDownloadThreads/chunkCount），删除手动 `pendingChanges` Set
  - SettingsLaunch.vue：3 个 watch（minMemory/maxMemory/isolationMode/memoryMode），不再直接 `await applyConfig` 而走 markDirty 防抖
- MemorySection.vue 保留简单模式：它走 `updateVersionPersonalization`（版本独立 setup.ini），不是全局 `applyConfig`
- IPC 请求体最小化示例：用户只改 `communitySource`，请求体仅 `[{ key: "communitySource", value: 1 }]`

#### 修复资源详情弹窗展开/收起卡顿
- 卡顿根因：`useVersionGroups` 的 `groups` 是 `computed`，内部读 `expandedSet.value.has(title)`，每次 `toggleGroup` 改 Set 都会触发整个 `groups` 重新计算，重新创建所有 VersionGroup 对象和内部 versions 数组，导致 v-for 全部重渲染，展开/收起动画中断（frontend: src/composables/useVersionGroups.ts, src/components/community/ResourceDetail.vue）
- 修复：把 `expanded` 状态从 `groups` 中剥离，存到独立的 `reactive<Record<string, boolean>>`；`groups` 改为 `shallowRef` + `watch`，只在 `versions` 或 `versionFilter` 变化时重新计算，不再依赖 expanded 状态
- 模板中 `g.expanded` 改为 `expandedOf(g.title)`，`toggleGroup` 只改 `expandedMap`，不触发 `groups` 重算
- 性能提升：点击展开/收起时只更新当前卡片的 expanded 状态，不再重渲染整个版本列表，动画流畅

#### 修复大量分组时展开仍卡顿（懒挂载内容）
- 上一轮把 expanded 状态从 groups 剥离后，少量分组场景已流畅；但当版本覆盖范围大（如 1.8–1.24.2，几十个分级）时展开仍卡顿，而筛选后展开流畅（frontend: src/composables/useVersionGroups.ts, src/components/community/ResourceDetail.vue）
- 根因：折叠卡片虽 `grid-rows-[0fr]` + `overflow-hidden` 隐藏内容，但内部 `v-for="v in g.versions"` 仍渲染全部版本条目 DOM，几十个分组 × 每组十几条 = 上千节点挂载在文档里；`grid-rows 0fr→1fr` 过渡期间浏览器每帧 reflow 会级联计算所有兄弟 grid 容器，节点越多越卡；筛选后 groups 重算只剩少数分组，兄弟 DOM 大减故流畅
- 修复：内容区加 `v-if="mountedOf(g.title)"` 懒挂载——折叠卡片不渲染版本条目 DOM；`toggleGroup` 首次展开时先设 `mountedMap[title]=true`（保持 0fr 折叠态挂载），`await nextTick()` 后再设 `expandedMap[title]=true` 触发 0fr→1fr 动画；挂载后保留 DOM，后续收起/展开动画正常
- `setFilter` 不再手动清状态，改由 `watch([versions, versionFilter])` 统一清空 `expandedMap`/`mountedMap`；单组自动展开也走"先挂载后展开"两步，保证首屏单组也有展开动画

#### 资源打包方式改造（参考 PCL2 ExtractResources）
- 重写 `resources` 模块：所有外部资源在编译时通过 `include_str!`/`include_bytes!` 嵌入二进制，运行时零文件 IO 读取，彻底废弃此前基于 `env!("CARGO_MANIFEST_DIR")` 拼路径的实现（backend: src-tauri/src/resources.rs）
- 修复发布版 bug：原实现 `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")` 在打包后指向开发机路径，用户机器上不存在，导致首次启动释放默认配置和 Forge 安装器全部失败
- 嵌入的资源清单：
  - 文本资源（`include_str!`）：`defaults/config.ini`、`defaults/instance.ini`、`defaults/setup.ini`、`moddata.txt`
  - 二进制资源（`include_bytes!`）：`forge-installer.jar`、`java-wrapper.jar`
- 二进制资源释放带 sha256 校验：参考 PCL2 的 `ExtractResources`，只在目标文件不存在或 hash 不匹配时写盘，同目录写 `{name}.sha256` 校验文件用于下次启动比对，避免每次启动重复写大文件拖慢启动、触发杀软误报
- `mcmod.rs` 第 22 行从直接 `include_str!("../../../resources/moddata.txt")` 改为走统一接口 `crate::resources::read_resource("moddata.txt")`，所有资源访问统一收口到 `resources` 模块
- 删除 `get_resources_dir`/`get_resource_path`/`exists`/`list_dir`/`default_config_path`/`default_instance_path`/`forge_installer_path`/`java_wrapper_path` 等基于运行时路径的函数（嵌入二进制后不再需要）
- `extract_resource` 签名从 `&PathBuf` 改为 `&Path`（更通用，调用处 `&PathBuf` 自动 deref 兼容）

#### 修复详情页下载未应用文件名格式设置
- 详情页"下载到任意路径"流程此前直接用 `v.file_name` 原始名（如 `jei-26.2-neoforge-30.11.0.67.jar`）作为保存对话框默认名，未应用设置页的 `community_filename_format` 配置，与"安装到游戏目录"流程行为不一致
- 后端 `apply_filename_format` 从 `fn` 提升为 `pub fn`，新增 `format_download_filename` Tauri 命令，内部读 `config.community_filename_format` 调用 `apply_filename_format`（backend: src-tauri/src/commands/community/install.rs）
- 前端新增 `formatDownloadFilename(fileName, translatedName?)` 封装（frontend: src/utils/api/community.ts）
- `ResourceDetail.vue` 的 `handleDownload` 改为：弹保存对话框前先调 `formatDownloadFilename(v.file_name, project.translated_name)` 取格式化后的文件名，再传给 `saveFile` 和 `downloadResourceToPath`，成功提示也用格式化后的名字
- 译名取自 `ResourceProject.translated_name`（整个工程一个译名，非 version 级）

#### 版本分组简化（顶部筛选截断到二级版本号）
- 详情弹窗顶部筛选滑块此前保留完整版本号（1.12、1.12.1、1.12.2 各成一项），项目覆盖版本范围大时滑块项过多（frontend: src/composables/useVersionGroups.ts）
- 拆分为两个函数：
  - `getFilterVersionName`（顶部筛选用）：截断到二级版本号，1.12.2 → "1.12"，26.1.3 → "26.1"
  - `getGroupedVersionName`（下面版本分组卡片用）：保留完整三级版本号，1.12.2 → "1.12.2"
- 低于 1.12 的版本（1.11、1.10、1.9 等）统一归到"远古版"标签，两边都适用
- `computeGroups` 筛选匹配改用 `getFilterVersionName`：选中 "1.12" 时匹配 1.12、1.12.1、1.12.2 所有版本，下面分组卡片仍按完整版本号各自独立成组

#### 远古版按主版本号分组（下面分组卡片）
- 此前低于 1.12 的版本统一归到单个"远古版"，1.8、1.9、1.10、1.11 全部合并一组（frontend: src/composables/useVersionGroups.ts）
- 下面分组卡片改为按次版本号分组：1.8.x → "1.8"，1.11.x → "1.11"，各自独立成组，仅非标准格式版本才归"远古版"
- 顶部筛选滑块保持合并：低于 1.12 的版本统一归"远古版"标签，筛选项不显示 1.8/1.9/1.10/1.11 各项，避免过多

#### 下面版本分组保留完整三级版本号
- 此前远古版按二级版本号分组（1.10.1/1.10.2 → "1.10"），用户反馈希望远古版也按完整版本号独立成组（frontend: src/composables/useVersionGroups.ts）
- `getGroupedVersionName` 改为：所有标准版本（1.x 和 26.x）都保留完整版本号，1.10.1 → "1.10.1"，1.12.2 → "1.12.2"
- 只有无法识别的非标准格式版本才归"远古版"
- 顶部筛选滑块仍用 `getFilterVersionName` 截断到二级，1.10.1 → "远古版"，1.12.2 → "1.12"

#### 详情页"转到 MC百科"改为直链跳转（参考 PCL2）
- 此前用搜索 URL `https://search.mcmod.cn/s?key=<name>`，PCL2 是直链 `https://www.mcmod.cn/class/<id>.html`（backend: src-tauri/src/minecraft/community/mcmod.rs, src-tauri/src/commands/community/detail.rs）
- 研究发现 PCL2 不调 API，完全靠 moddata.txt 的**行号**作为 class id：第 N 行 → class id = N，URL 即 `https://www.mcmod.cn/class/<N>.html`
- 关键设计：moddata.txt 空行也占用行号（PCL2 WikiEntry.vb 的 `i += 1` 在 `Continue For` 之前），此前 MoLaunch 解析时 `continue` 跳过空行且不计数，会导致行号错位；已修复
- `Database` 的 value 从 `String`（仅中文名）改为 `Entry { chinese_name, class_id }` 结构
- 新增 `lookup_class_id(platform, slug) -> Option<u32>` 查询函数
- 新增 `get_mcmod_url` Tauri 命令：接受 platform + slug，返回直链 URL 或 null
- 前端 `openMcmod()`：先查 class id 直链，查不到回退搜索 URL，点击后 toast 提示"正在打开 MC 百科详情页"或"未找到直链，已跳转到搜索页"

#### HorizontalFilter 滚动条过渡动画
- 滚动条从"直接蹦出"改为淡入淡出（frontend: src/components/common/HorizontalFilter.vue）
- webkit 滚动条 thumb 默认 `background-color: transparent`，hover 容器时通过 `transition: background-color 0.2s ease` 平滑过渡到灰色半透明
- 避免突兀的滚动条出现效果

#### 详情页"复制名称"按钮 toast 提示
- 复制成功后 toast 提示"已复制: <名称>"

#### 修复详情页外链按钮点击无反应
- Tauri WebView2 内 `window.open` 和 `<a target="_blank">` 会被拦截，点击后无任何反应（frontend: src/components/community/ResourceDetail.vue）
- 改用 `@tauri-apps/plugin-shell` 的 `open()` 调用系统默认浏览器打开外链
- 影响按钮：转到 CurseForge/Modrinth、转到 MC百科

#### 移除 ResourceCard 图片 lazy loading 消除 WebView2 Intervention 警告
- ResourceCard.vue 的 `<img>` 此前用 `loading="lazy"`，WebView2 会 Intervention 把 lazy 图片替换为 placeholder 并延迟 load 事件，控制台输出警告（frontend: src/components/community/ResourceCard.vue）
- 搜索结果列表中的 logo 本身就在视口内，lazy loading 无意义，直接移除

#### 修复启动时不创建 .minecraft 文件夹导致"打开游戏目录"报错
- 此前启动初始化链只创建 `.Molaunch` 系列目录，从不创建 `.minecraft`，首次启动点击"打开游戏目录"会报"路径不存在"（backend: src-tauri/src/lib.rs, src-tauri/src/commands/system/game_dir.rs）
- 参考 PCL2 `McFolderListLoadSub:124-128`：PCL2 启动时主动 `DirectoryUtils.Create(PathExeFolder & ".minecraft\versions\")`
- `lib.rs` 的 `run()` 在 `AppState::new()` 后增加：`resolve_game_dir(&config.game_dir).join("versions")` 不存在时 `create_dir_all`
- `open_game_dir` 命令增加防御性创建：路径不存在时先 `create_dir_all` 再打开，避免启动时创建失败导致命令仍报错

#### "转到 MC百科"按钮只对 Mod 类型显示
- MC 百科数据库（moddata.txt）只包含 Mod 条目，PCL2 也仅对 Mod/数据包类型显示该按钮（frontend: src/components/community/ResourceDetail.vue）
- 添加 `v-if="project.resource_type === 'Mod'"` 条件，整合包/资源包/光影/数据包不显示该按钮

#### PCL2 整合包安装逻辑研究结论（未实现，待后续开发）
- PCL2 下载页"整合包"分类的资源安装流程：
  - 下载原始包到 `versions\{InstanceName}\原始整合包.{zip|mrpack}`
  - 调用 `ModpackInstall` 解压 → 解析 manifest.json/modrinth.index.json → 复制 overrides → 批量下载依赖 mods → 安装游戏本体
- 不同资源类型的处理差异：
  - Mod/资源包/光影/数据包：只下载到对应子文件夹（mods/resourcepacks/shaderpacks），不解压不解析
  - 整合包"安装"：完整走 ModpackInstall 流程
  - 整合包"另存为"：仅下载原始压缩包，不做后续处理
- MoLaunch 当前整合包下载流程与 PCL2"另存为"一致，缺少完整的安装流程，待后续实现
- 此前全局 `document.addEventListener('scroll', ..., { capture: true })` 会捕获所有元素的 scroll 事件，下拉框、弹窗等组件滚动时也会显示返回顶部按钮（frontend: src/components/common/BackToTop.vue）
- 新增 `isNonMainScroller` 过滤：向上查找祖先，遇到 `position: fixed/absolute` 的元素说明在弹层内，跳过不处理
- 只响应页面级滚动容器的 scroll 事件
- 此前方案 webkit 滚动条 height 0→6px 瞬变导致突兀（frontend: src/components/common/HorizontalFilter.vue）
- 改为：滚动条高度始终 6px（不变），thumb 默认 `background-color: transparent`，hover 时只改变 thumb 颜色（`transition: background-color 0.25s ease` 平滑过渡）
- 避免 height 属性变化导致的瞬变效果

#### 详情页新增"转到 MC百科"和"复制名称"按钮
- 操作按钮行新增两个按钮（frontend: src/components/community/ResourceDetail.vue）
- "转到 MC百科"：用译名优先（无译名用原名）打开 `https://search.mcmod.cn/s?key=<name>` 搜索
- "复制名称"：将译名或原名复制到剪贴板，使用浏览器 `navigator.clipboard` API

#### HorizontalFilter 箭头状态修复 + hover 显示滚动条
- 右侧 tag 显示不出右滑按钮的 bug 修复：用 `ResizeObserver` 监听容器和内容尺寸变化，确保 options 变化后箭头状态及时更新（frontend: src/components/common/HorizontalFilter.vue）
- 滚动条改为 hover 显示：默认隐藏，鼠标移入容器时显示细滚动条（6px 高，灰色半透明），移出隐藏
- 组件卸载时正确断开 ResizeObserver 连接，避免内存泄漏

#### 配置读取接口统一
- 新增 `get_config` 单一 IPC 命令取代此前分散在 5 个文件的 14 个 `get_*` getter 命令（backend: src-tauri/src/commands/system/apply_config.rs）
- 新增 `ConfigEntry` 结构体 `{ key: String, value: Value }`，返回扁平化数组 `Vec<ConfigEntry>`，格式为 `[{ "key": "proxyMode", "value": "none" }, ...]`，便于前端遍历和按需过滤
- 新增 `ConfigSnapshot` 结构体（后端内部用于类型安全地构建全量快照），序列化为 JSON 后转为 `ConfigEntry` 数组返回
- 新增 `keys: Option<Vec<String>>` 请求参数：不传或空数组返回全部字段；传数组时仅返回指定字段（camelCase 名称），减少不必要的全量传输
- 修复 CurseForge API Key 不返回的 bug：`get_config` 和 `apply_config` 均改用 `secure_storage::get_config_async()` 异步触发首次 DES 解密，取代此前同步 `get_cached()` 导致懒加载未触发、首次返回空字符串的问题（同时修复 `apply_config` 中仅改 `enabled` 不改 `api_key` 时误清空已有 key 的隐患）
- 删除 14 个旧 getter：`get_proxy_mode/type/url`、`get_mirror_url`、`get_download_source`、`get_meta_source`、`get_max_download_speed`、`get_max_download_threads`、`get_chunk_count`、`get_min_memory`、`get_max_memory`、`get_memory_mode`、`get_isolation_mode`、`get_log_level`、`get_game_dir`、`get_selected_version`、`get_curseforge_config`、`get_community_config`
- download.rs / proxy.rs / game.rs 清空为占位（保留模块声明供后续扩展），移除对应 `pub use` 避免未使用导入警告
- community 模块 `get_curseforge_config` / `get_community_config` 两条 re-export 同步移除（backend: src-tauri/src/commands/community/mod.rs, secure_config.rs, community_config.rs）
- 前端 `ConfigEntry` / `ConfigSnapshot` 类型与 `getConfig()` / `getConfigMap()` 函数（frontend: src/utils/api/system.ts），删除 17 个旧 setter 与 14 个旧 getter
- 前端新增全局配置缓存：模块级 `configCache` + 并发请求合并（`configPromise`），切换侧栏时各组件 `getConfigMap()` 直接读缓存不再重复 IPC；`applyConfig(patch)` 保存成功后 `Object.assign` 同步更新缓存；新增 `refreshConfig()` 手动清空缓存用于强制刷新场景
- `getConfig(keys?, force?)` 返回 `ConfigEntry[]` 数组格式；`getConfigMap(force?)` 返回 `ConfigSnapshot` 对象格式（从缓存转换，方便组件按字段名访问）；均支持 `force=true` 强制刷新
- 5 个 Vue 组件改造为 `getConfigMap()` 单次调用：SettingsAdvanced.vue、SettingsDownload.vue、SettingsLaunch.vue、CommunityConfigCard.vue、MemorySection.vue
- 保留 `get_config_value`（通用 INI 直读，用于调试/迁移）和 `get_config_path`

#### 社区资源配置项落地实现
- `community_filename_format`（文件名格式）落地：install.rs 新增 `apply_filename_format` 函数，根据 5 种格式拼接译名与原名（`【译名】原名` / `[译名] 原名` / `译名-原名` / `原名-译名` / `仅原名`），支持 `.jar.disabled` / `.jar.old` 多段后缀保留（backend: src-tauri/src/commands/community/install.rs）
- `DownloadRequest` 新增 `translated_name` 字段（可选），前端安装时传入 mcmod 译名，后端按 `community_filename_format` 重命名后落盘
- `community_mod_local_name_style`（Mod 管理样式）落地：ModInfo 新增 `translated_name` 字段，list_mods 解析 jar 内 `fabric.mod.json` / `META-INF/mods.toml` / `mcmod.info` 获取 mod slug，查询 mcmod 数据库获取中文译名（backend: src-tauri/src/commands/version/mods.rs）
- 前端 ModTab.vue 新增 `modTitle` / `modSubtitle` 辅助函数，根据 `community_mod_local_name_style` 切换标题与详情显示：0=标题译名+详情文件名，1=标题文件名+详情译名（frontend: src/views/version-settings/ModTab.vue）
- `community_source`（下载源策略）和 `community_ignore_quilt`（忽略 Quilt）此前已在 curseforge.rs / modrinth.rs 实现，本次确认无遗漏

#### 社区资源配置 UI 改为下拉框
- 社区资源卡片的「来源」和「文件名格式」从按钮组改为自定义 `Select` 组件（复用项目组件，非原生 select），符合项目"使用自定义组件而非浏览器原生组件"的约束（frontend: src/components/community/CommunityConfigCard.vue）
- 「来源」Select 使用 `#option` 插槽展示双行（label + desc 描述），触发器只显示简短 label，下拉面板显示完整描述
- 「文件名格式」Select 使用默认选项渲染，展示示例文件名（如 `[机械动力] create-1.21.1`）
- 「Mod 管理样式」和「忽略 Quilt」保留原按钮组形式（选项数量少，按钮组更直观）

### 新增

#### 3D 皮肤模型预览
- 新增 SkinModel3D 组件，基于 three.js 构建完整的 Minecraft 3D 人物模型（头/身/双臂/双腿），支持皮肤纹理 UV 映射（frontend: src/components/common/SkinModel3D.vue）
- 支持 classic（Steve）和 slim（Alex）两种模型
- 支持披风 3D 渲染（平面挂在身体后方，略向后倾斜）
- 支持鼠标拖动旋转查看模型，闲置 3 秒后自动缓慢自转
- 替换 SkinManager 中的 2D canvas 预览为 3D 模型

#### 皮肤与披风管理
- 披风 PNG 下载：新增 `download_cape_png` Tauri 命令，从 profile_json 的 capes[].url 获取已装备披风并下载（backend: src-tauri/src/minecraft/skin.rs, src-tauri/src/commands/skin.rs）
- CapeInfo 增加 `url` 字段，解析披风下载地址（backend: src-tauri/src/minecraft/skin.rs）
- `downloadCapePng` 前端封装（frontend: src/utils/tauri.ts）

#### 离线账号默认皮肤
- 新增 default-skin.ts：内置 Steve/Alex 默认皮肤纹理（canvas 生成），参考 PCL2 的 McSkinSex 函数根据 UUID 计算皮肤类型（frontend: src/utils/default-skin.ts）
- SkinAvatar 支持离线账号：传 login_type='Offline' 时使用默认皮肤（frontend: src/components/common/SkinAvatar.vue）

#### 账号切换
- 账号切换改为单卡片左右滑动切换（一次只显示一个账号），支持拖动/滚轮切换，带平滑动画，末尾预留新增账号卡片（frontend: src/components/home/AccountSelector.vue）

### 修复
- 修复皮肤头像裁剪：overlay 层（头发层）现在检查透明像素，避免空白覆盖脸部（frontend: src/components/common/SkinAvatar.vue）
- 修复 willReadFrequently 警告：所有频繁读取 getImageData 的 canvas 添加 { willReadFrequently: true } 选项
- 修复 SkinManager loadInfo 中 getSkinCapeInfo 失败导致后续步骤不执行的问题（每步独立 try-catch）

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

#### 皮肤与披风管理
- 玩家皮肤头像加载（参考 PCL2：从 profile_json 的 skins[].url 获取皮肤 PNG 地址，下载后用 canvas 裁剪 (8,8,8,8) 脸层 + (40,8,8,8) 头发层）
- 皮肤 PNG 全图显示（直接从 textures.minecraft.net 下载，不依赖第三方渲染服务）
- 当前形象信息展示（用户名、皮肤模型 Steve/Alex、当前披风）
- 皮肤上传功能（multipart/form-data，支持 classic/slim 两种模型，后端直接读取本地文件避免 base64 转换）
- 披风列表展示与装备/取消（28 种披风中文名映射，参考 PCL2）
- 修改密码快捷入口（跳转 `https://account.live.com/password/Change`）
- 修改用户名快捷入口（跳转 `https://www.minecraft.net/zh-hans/msaprofile/mygames/editprofile`）
- `SkinAvatar` 组件：通用皮肤头像组件，canvas 裁剪支持高清皮肤（128x64 等），加载失败时回退到首字母渐变占位符
- `SkinManager` 弹窗：完整的皮肤/披风管理界面
- `AccountSelector` 接入真实皮肤头像显示与皮肤管理入口

### 变更
- 微软登录采用 Device Code Flow（设备码流程，与 PCL2 一致），使用 v2.0 consumers 端点 + MoLaunch 独立 Azure 应用 Client ID
- 认证存储从单一 `auth.json` 文件改为 Windows 注册表分字段存储（参考 PCL2，路径 `HKCU\Software\MoLaunch`）
- 敏感字段（Token、用户名、UUID 等）单独 SDK DES 加密，非敏感字段（登录类型）明文存储
- Token 刷新使用 login.live.com 旧版端点（与 PCL2 一致）
- reqwest 启用 `multipart` feature 以支持皮肤上传

### 修复
- 修复微软登录申请设备码时返回 `AADSTS700016` 错误（旧版 Minecraft 启动器 Client ID 与 `login.microsoftonline.com` v2.0 端点不兼容，改用 `login.live.com` 旧版端点）
- 修复 `DeviceCodeModal.vue` 中 `openUrl` 导入错误（Tauri 2 shell 插件 API 变更为 `open`）

### 重构（代码质量 V2 - 阶段 1：重复代码整合）

> 完整审查报告见 `docs/CODE_QUALITY_REPORT_V2.md`。本阶段聚焦消除前后端重复代码与样板，不改变业务逻辑。

#### 前端重复代码整合
- **F3 formatBytes 重复实现消除**：`ExternalDownload.vue`、`QuickTools.vue`、`SystemMonitorPanel.vue`、`CacheMonitorPanel.vue`、`custom-layout/datasource.ts` 共 5 处本地 `formatBytes` 全部删除，统一改用 `@/utils/format` 的 `formatBytes`
- **F4 toast 兼容别名删除**：移除 `src/utils/toast.ts` 底部的 `showInfo`/`showSuccess`/`showError`/`showWarning` 4 个兼容别名（与 `modal.ts` 同名导出冲突），强制 22 个调用文件迁移到 `toastXxx` 前缀命名（`AdvanceFieldsPanel.vue`、`InstalledList.vue`、`DevModeToggle.vue`、`LogViewer.vue`、`LaunchPanel.vue`、`CrashDialog.vue`、`AccountSelector.vue`、`ResourceDetail.vue`、`ResourceDetailHeader.vue`、`Community.vue`、`SetupTab.vue`、`FolderSidebar.vue`、`SkinUploadPanel.vue`、`JavaCustomMode.vue`、`JavaModeSelector.vue`、`MemorySection.vue`、`useLaunchState.ts`、`JavaDownloadBar.vue`、`SettingsDeveloper.vue`、`SettingsOther.vue`、`SettingsCache.vue`、`JavaPathSelector.vue`）
- **F1 SettingsPersonal 配置读写**：`SettingsPersonal.vue` 的 `gameLanguage` 改用 `useConfigPage` composable，移除手动的 `loadGameLanguage`/`saveGameLanguage` 函数及 `getConfigMap`/`applyConfig` 直接调用
- **F6 invoke() 绕过封装层消除**：新增 3 个 API 封装函数消除 Vue 组件直接 `invoke()` 调用：
  - `src/utils/api/system.ts` 新增 `writeTextFile(path, content)`
  - `src/utils/api/plugins.ts` 新增 `exportPluginSample(destPath, asZip)` 和 `readLayoutSample(format)`
  - `SettingsPlugins.vue` 和 `SettingsPersonal.vue` 已迁移到封装函数
- **F7 原生 confirm() 替换**：`SettingsPlugins.vue` 的原生 `confirm()` 改用 `@/utils/modal` 的 `showConfirm`
- **F5 safeCall 迁移**：暂缓（safeCall 改变返回语义需逐个审查，留待后续逐步迁移）

#### 后端重复样板整合
- **B1 resolve_mirror_and_source 补完**：`commands/version/list.rs`、`commands/version/loaders.rs` 改用 `state::resolve_mirror_and_source` helper 消除 lock/clone/drop 样板；`java.rs`/`download.rs`/`manage.rs` 因使用 `download_source` 字段（文件下载源）而非 `meta_source`（版本列表源），与 helper 语义不同，保留手动实现
- **B2 resolve_game_dir_from_state 补完**：`commands/version/mods/mod.rs`、`commands/community/install/mod.rs`、`commands/version/list.rs`、`commands/version/mods/helpers.rs`、`commands/version/script_export.rs`、`commands/version/download.rs`、`commands/version/manage.rs` 改用 `state::resolve_game_dir_from_state` helper；`folder.rs` 因需 `mut config` 调用 `save_config`（非只读样板）保留手动实现
- **B4 utils::fs 模块创建**：新增 `src-tauri/src/utils/fs.rs`，提供 `ensure_dir(path)` 和 `read_to_string(path)` 两个 helper，替换 `commands/plugins/mod.rs`、`commands/system/game_dir.rs`、`commands/community/install/concurrent.rs` 共 5 处 `create_dir_all` 样板
- **B3 log_err 迁移**：暂缓（167 处机械替换工作量大，仅改变错误日志格式不影响功能，helper 保留供新代码使用）

### 重构（代码质量 V2 - 阶段 2：前端超长文件拆分）

> 完整审查报告见 `docs/CODE_QUALITY_REPORT_V2.md`。本阶段聚焦拆分超过 300 行的 Vue 组件和超过 400 行的 TypeScript 文件，不改变业务逻辑。

#### Vue 组件拆分（11 项）
- **2.1 SettingsPlugins.vue（583 → 192）**：抽 `plugins/PluginFlowSteps.vue`（70 行）+ `plugins/PluginListSection.vue`（208 行）+ `plugins/PermissionTableSection.vue`（105 行）子组件，主文件仅保留外部插件安装入口
- **2.2 ExternalDownload.vue（474 → 193）**：抽 `composables/useExternalDownload.ts`（265 行，URL+文件名+目录+下载状态+文件列表逻辑）+ `external-download/DownloadedFileList.vue`（70 行）子组件
- **2.3 ColorPicker.vue（475 → 261）**：style 215 行提取到外部 `ColorPicker.css`，使用 Vue SFC `<style scoped src>` 引入
- **2.4 SettingsPersonal.vue（472 → 73）**：抽 `personal/AppearanceSection.vue`（52 行）+ `personal/HomePanelModeSection.vue`（115 行）+ `personal/CustomLayoutSection.vue`（253 行）子组件
- **2.5 SettingsMore.vue（467 → 76）**：抽 `more/AboutTab.vue`（158 行）+ `more/CreditsTab.vue`（245 行）+ `more/TutorialTab.vue`（25 行）+ `utils/aboutLogos.ts`（24 行，共享 logoMap+resolveLogo+openLink）
- **2.6 Select.vue（431 → 211）**：style 221 行提取到外部 `Select.css`，使用 `<style scoped src>` 引入
- **2.7 ModUpdateDialog.vue（424 → 194）**：抽 `composables/useModUpdate.ts`（208 行，版本查询+过滤+下载）+ `mod-tab/VersionTable.vue`（78 行）子组件
- **2.8 QuickTools.vue（408 → 69）**：抽 `quick-tools/CleanupTool.vue`（244 行）+ `quick-tools/MemoryOptimizer.vue`（131 行）子组件，主文件保留敬请期待占位
- **2.9 LoaderSelect.vue（317 → 270）**：抽 `composables/useLoaderCompatibility.ts`（95 行，MC 版本类型判断 + 加载器兼容性检查）
- **2.10 HomeClockCard.vue（305 → 125）**：抽 `composables/useHomeClockCards.ts`（208 行，时钟 + 4 种轮播卡片加载 + 自动翻页）
- **2.11 AccountSelector.vue（303 → 175）**：抽 `composables/useAccountCards.ts`（182 行，账号列表构建 + 切换/删除/登出）

#### TypeScript 文件拆分（2 项）
- **2.12 useModOperations.ts（582 → 125 编排层）**：拆为三个子 composable —— `composables/useModList.ts`（358 行，列表加载/过滤/单 Mod 操作/预加载/详情查询/版本上下文/文件监听）+ `composables/useModBatchOps.ts`（172 行，多选状态 + 批量启用/禁用/删除）+ `composables/useModUpdateDialog.ts`（78 行，更新对话框状态 + 打开/批量更新/安装完成回调），主文件仅做编排，对外 API 完全不变
- **2.13 stores/plugins.ts（428 → 382）**：抽出纯函数和数据结构到 `@/utils/pluginInstaller`（130 行，包含 `PersonalizationData` 接口 + `DEFAULT_CUSTOM_LAYOUT` 默认值 + `externalManifestToPluginManifest` 清单转换 + `loadPersonalizationData`/`savePersonalizationData`/`fetchCustomLayoutContent` 后端封装 + `isValidHomePanelMode` 字符串校验），store 文件聚焦状态管理与生命周期编排

### 重构（代码质量 V2 - 阶段 3：后端超长文件拆分）

> 完整审查报告见 `docs/CODE_QUALITY_REPORT_V2.md`。本阶段聚焦拆分超过 400 行的 Rust 文件和超过 200 行的单函数，不改变业务逻辑。

#### 3.1 commands/plugins/mod.rs 拆分（1230 → 178 编排层 + 7 子模块）
- 拆为 `plugins/{mod,sandbox,install,spawn,window,layout,export,personalization}.rs`
- `mod.rs`（178 行）保留共享类型（`ProcessPermissions` / `WindowPermissions` / `ExternalPluginManifest` / `ExternalPluginEntry`）+ 共享 helper（`plugins_root` / `is_valid_plugin_id` / `read_plugin_manifest`）+ 子模块声明
- `sandbox.rs`（119 行）：`list_external_plugins` / `read_external_plugin_file` / `uninstall_external_plugin`
- `install.rs`（269 行）：`install_external_plugin_from_dir` / `install_external_plugin_from_zip` + `copy_dir_recursive` / `determine_zip_prefix` / `extract_zip_safely` helper
- `spawn.rs`（268 行）：`plugin_spawn_process` + `ProcessResult` / `is_command_allowed` / `which_canonical` / `paths_equal` / `truncate_output` helper（Windows 忽略大小写与 `.exe` 后缀，UTF-8 字符边界切割）
- `window.rs`（174 行）：`plugin_create_window` + `extract_domain` / `is_domain_allowed` helper（支持 `*.` 通配符前缀）
- `layout.rs`（89 行）：`load_custom_layout` + `hash_url` helper（sha256 缓存文件名，24h TTL）
- `export.rs`（100 行）：`read_layout_sample` / `export_plugin_sample`（ZIP 现场打包用 zip crate）
- `personalization.rs`（86 行）：`read_personalization` / `write_personalization` + `personalization_path` helper（Windows: `%APPDATA%/.MolaLaunch/personalization.json`）
- `lib.rs` invoke_handler 改用完整子模块路径注册命令（`commands::plugins::sandbox::list_external_plugins` 等），因 `tauri::command` 宏生成的 `__cmd__` 符号无法通过 `pub use` 重导出（参考 `commands/community/install/mod.rs` 注释）

#### 3.2 commands/version/launch.rs 拆分（555 → 287 mod.rs + 255 build_config + 111 failure）
- 拆为 `launch/{mod,build_config,failure}.rs`，`launch_game` 函数从 ~410 行缩减为 65 行编排层
- `mod.rs`（287 行）：共享类型（`GameExitEvent`）+ 共享 helper（`parse_server_enter` / `resolve_game_language`）+ `launch_game` 编排 + `spawn_exit_watcher` 退出监视 + 其他短命令（`get_launch_progress` / `cancel_launch` / `stop_game` / `get_running_game` / `get_launch_history`）
- `build_config.rs`（255 行）：`build_launch_config` — 从全局配置 + 版本独立设置 + 前端入参构建 `LaunchConfig`（Java 路径解析 / 服务器地址 / 额外参数 / 内存 / 认证信息 / 离线皮肤 UUID 调整 + 资源包替换 / 隔离模式）
- `failure.rs`（111 行）：`handle_launch_failure` — LaunchProcess 阶段失败时等待 watcher 崩溃分析（最多 15 秒）、构造 fallback CrashInfo、清理状态、发送 `game-exited` 事件
- 所有 `#[tauri::command]` 函数留在 `mod.rs`（tauri::command 宏限制），非命令 helper 通过 `pub(super)` 暴露给父模块

#### 3.3 minecraft/version/libraries.rs 拆分（503 → 52 mod.rs + 313 parse.rs + 101 filter.rs + 63 download.rs）
- 拆为 `libraries/{mod,parse,filter,download}.rs`，公共 API 通过 `pub use` re-export 保持完全向后兼容
- `mod.rs`（52 行）：`LibEntry` 结构 + `name()` 方法 + `maven_to_path` helper + 子模块 re-export
- `parse.rs`（313 行）：`parse_libraries`（JSON 解析，3 分支：natives 字段 / Forge 26.2+ 新格式 / 普通库）+ `check_rules`（平台规则匹配）+ `is_native_matching_arch`（架构过滤）+ `deduplicate_libs` / `get_version_from_name` / `compare_versions_ge` 去重逻辑
- `filter.rs`（101 行）：`find_missing_libs`（`std::thread::scope` 并行校验）+ `quick_check_lib`（快速模式：仅文件存在+大小；完整模式：SHA1 校验）
- `download.rs`（63 行）：`build_download_urls`（BMCLAPI/maven/libraries 镜像替换 + mirror_url fallback）

#### 3.4 minecraft/download/downloader.rs 拆分（384 → 15 mod.rs + 244 single.rs + 148 stream.rs）
- 拆为 `downloader/{mod,single,stream}.rs`，公共 API 通过 `pub use` re-export 保持完全向后兼容（唯一调用方 `manager.rs` 无需修改）
- `mod.rs`（15 行）：`MAX_UNVERIFIED_BYTES` 常量（`pub(crate)` 供 stream.rs 引用）+ 子模块 re-export
- `single.rs`（244 行）：`download_single`（文件存在检查 + 目录创建 + 分片阈值计算 + URL 顺序循环 + 重试 + 分片/单流选择 + 校验）
- `stream.rs`（148 行）：`download_from_url`（HTTP 请求 + 流式处理 + 限速 + 暂停/取消信号 + 全局进度增量更新 + 字节数上限校验）
- 偏离 V2 报告建议：报告建议拆为 4 文件 `{mod,single,retry,merge}.rs`，实际拆为 3 文件 `{mod,single,stream}.rs`。原因：重试逻辑仅 `while attempt < max_retries` 3 行循环且与 URL 循环、分片/单流选择紧耦合，强行抽出 `retry.rs` 需传递 12 个参数，反而降低可读性；`merge.rs` 改名 `stream.rs` 更准确反映职责（单 URL 流式下载）

#### 3.5 minecraft/version/scan.rs 拆分（334 → 168 mod.rs + 121 loaders.rs + 59 version_extract.rs）
- 拆为 `scan/{mod,loaders,version_extract}.rs`，公共 API 通过 `pub(crate) use` re-export 保持向后兼容
- `mod.rs`（168 行）：`VersionInfo` 结构 + `scan_installed_versions`（扫描 versions 目录）+ `parse_version_info`（解析单个版本 JSON）+ `get_version_chain`（继承链）+ `uninstall_version`（卸载）
- `loaders.rs`（121 行）：`detect_loaders`（检测 OptiFine / Fabric / NeoForge / Forge / LiteLoader + 快照判断，调用 `version_extract::extract_original_version`）
- `version_extract.rs`（59 行）：`extract_original_version`（5 策略提取原版版本号：inheritsFrom → --fml.mcVersion → downloads URL → jar → id 正则）
- 偏离 V2 报告建议：报告建议拆为 `scan/{mod,loaders,assets}.rs`，实际将第三文件命名为 `version_extract.rs`。原因：该文件内容为原版版本号提取（5 策略正则/字段匹配），与"assets"（资源文件）无关，`version_extract.rs` 更准确反映职责

#### 3.6 commands/version/script_export.rs 拆分（354 → 212 mod.rs + 106 content.rs + 80 resolve_java.rs）
- 拆为 `script_export/{mod,content,resolve_java}.rs`，`export_launch_script` 是 `#[tauri::command]` 保留在 mod.rs
- `mod.rs`（212 行）：`export_launch_script` 编排（版本设置 + 内存配置 + 隔离模式 + Java 路径 + 服务器解析 + 认证信息 + 离线皮肤 + 构建启动参数 + 调用 content 子模块）
- `content.rs`（106 行）：`ScriptLaunchInfo` 结构（借用引用避免克隆）+ `build_script_content`（.bat 脚本生成：CRLF + GBK + 版权头 + 启动提示 + Java 命令 + 敏感参数脱敏）+ `write_script_file`（GBK 编码写入 + 文件权限限制）
- `resolve_java.rs`（80 行）：`resolve_java_path`（用户指定路径校验兼容性 → 系统搜索 → select_best_java_with_loader）

#### 3.7 commands/community/install/mod.rs 拆分（484 → 37 mod.rs + 293 resource.rs + 173 modpack.rs）
- 拆为 `install/{mod,resource,modpack}.rs`，6 个 `#[tauri::command]` 按职责分散到子模块
- `mod.rs`（37 行）：`pub mod` 声明（concurrent / curseforge / helpers / modpack / modpack_stages / modrinth / resource / types）+ 类型 re-export
- `resource.rs`（293 行）：`download_resource`（下载到游戏目录）+ `download_resource_to_path`（下载到自定义路径）+ `install_resource`（语义化别名）+ `format_download_filename`（文件名格式化）+ `get_resource_install_path`（获取安装路径）
- `modpack.rs`（173 行）：`install_modpack`（CF API Key 检查 → 下载原始包 → 解析格式 → 下载依赖 → 复制 overrides）
- lib.rs 注册路径从 `install::*` 改为 `install::resource::*` / `install::modpack::*`，`commands/community/mod.rs` re-export 同步更新
- 偏离 V2 报告建议：报告建议拆为 `{mod,modpack,overrides,loader}.rs`，实际拆为 `{mod,resource,modpack}.rs`。原因：`overrides` 提取逻辑已在 `concurrent.rs`、`loader` 解析逻辑已在 `modpack_stages.rs`，无需重复拆分；新增 `resource.rs` 容纳 5 个资源下载命令，比原报告的 `overrides`/`loader` 更贴合实际职责

#### 3.8 commands/version/mods/mod.rs 拆分（406 → 31 mod.rs + 162 list.rs + 103 manage.rs + 126 install.rs）
- 拆为 `mods/{mod,list,manage,install}.rs`，8 个 `#[tauri::command]` 按职责分散到子模块
- `mod.rs`（31 行）：`pub mod` 声明（helpers / install / list / metadata / manage / types / watcher）+ 类型 re-export（`get_mods_dir` / `read_mod_metadata` / `ModInfo` / `ModMetadata`），原有 `watcher.rs`/`helpers.rs`/`metadata.rs`/`types.rs` 子模块保留不动
- `list.rs`（162 行）：`is_version_modable`（检查版本是否可装 mod）+ `list_mods`（遍历 mods 目录读取元数据并返回列表）+ 私有 `infer_loader_type`（按文件名特征推断加载器类型）
- `manage.rs`（103 行）：`toggle_mod`（启用/禁用 mod 通过 `.disabled` 后缀）+ `delete_mod`（删除 mod 文件）
- `install.rs`（126 行）：`install_mod`（从 URL 下载并安装 mod）+ `open_mods_dir`（在资源管理器打开 mods 目录）+ `get_version_mods_dir`（返回 mods 目录路径）+ `reveal_mod_file`（在资源管理器定位到指定 mod 文件）
- lib.rs 注册路径从 `mods::*` 改为 `mods::list::*` / `mods::manage::*` / `mods::install::*`（watcher 子模块原本已用完整路径）
- 修复 `watcher.rs` 第 29 行 `use super::sanitize_version_id` 失效：该函数位于 `commands::version::sanitize_version_id`，应改为 `use super::super::sanitize_version_id`
- 偏离 V2 报告建议：报告建议拆为 `{list,toggle,delete,install,watcher}.rs` 5 个子模块，实际拆为 `{mod,list,manage,install}.rs` 4 个（watcher 已存在）。原因：`toggle_mod` 与 `delete_mod` 函数过短（30~50 行）合并为 `manage.rs` 更紧凑，避免文件碎片化

#### 3.9 minecraft/loaders/forge.rs 拆分（385 → 59 mod.rs + 152 install.rs + 174 legacy.rs）
- 拆为 `forge/{mod,install,legacy}.rs`，原 `forge.rs` 已删除，由 `forge/` 目录替代
- `mod.rs`（59 行）：模块入口 + `mod install; mod legacy;` + `pub use install::install;` + `list_versions`（BMCLAPI JSON 格式 / 官方 HTML 格式双解析）
- `install.rs`（152 行）：`install` 调度器（下载 installer JAR → 根据 `forge_installer::needs_injector` 判断走 modern 或 legacy）+ `install_modern`（1.13+ injector 方式：launcher_profiles 初始化 → Mojang 映射下载 → 嵌入资源提取 → Java 查找 → 安装器执行 → 版本 JSON + MC JAR 复制）
- `legacy.rs`（174 行）：`install_legacy`（1.7.10 ~ 1.12.2 旧版安装）：解析 `install_profile.json`，区分方式 2（含 `install` 字段，1.7.10 及更早，按 `filePath` 提取 JAR）和方式 1（含 `json` 字段，1.8~1.12.2，解压 `maven/` 到 libraries + 复制 MC JAR）
- 子模块路径调整：`super::shared` / `super::forge_installer` 等同级模块引用改为 `super::super::shared` / `super::super::forge_installer`（多一层 `super`）
- 偏离 V2 报告建议：报告建议拆为 `forge/{mod,install,profile}.rs`，实际拆为 `forge/{mod,install,legacy}.rs`。原因：函数名为 `install_legacy` 且职责为完整安装流程（含 profile 解析 + JAR 提取 + 版本 JSON 写入 + MC JAR 复制），不只是 profile 解析，`legacy.rs` 更准确反映文件职责

#### 3.10 commands/auth/account.rs 拆分 + 应用 log_err（373 → 41 mod.rs + 117 ms.rs + 173 offline.rs + 95 session.rs）
- 拆为 `account/{mod,ms,offline,session}.rs`，原 `account.rs` 已删除，由 `account/` 目录替代
- `mod.rs`（41 行）：模块入口 + `pub mod ms; pub mod offline; pub mod session;` + `MsAccountInfo` + `OfflineAccountInfo` 数据类型
- `ms.rs`（117 行）：微软账号管理 3 个命令（`get_ms_accounts` 列表 / `remove_ms_account` 删除 / `switch_ms_account` 切换含 token 过期自动刷新）
- `offline.rs`（173 行）：离线账号管理 5 个命令（`get_offline_accounts` 列表 / `set_offline_skin` 皮肤设置 / `save_custom_skin` 自定义 PNG 上传含文件头 + 大小校验 / `remove_offline_account` 删除 / `switch_offline_account` 切换）
- `session.rs`（95 行）：会话命令（`get_login_status` 内存→磁盘恢复优先级 + 微软 token 静默刷新 / `logout` 清空内存+磁盘当前用户）
- 应用 `log_err`：14 处 `.map_err(|e| e.to_string())` 改为 `.map_err(log_err("描述性标签"))`，统一记录 `log_error!` 日志（如 `Failed to load auth storage` / `Failed to remove MS account` / `Failed to refresh MS token` 等）
- 保留 3 处 `save_custom_skin` 中的 `format!("读取皮肤文件失败: {}", e)` 等中文错误文案，因前端依赖中文错误展示
- lib.rs 注册路径从 `account::*` 改为 `account::ms::*` / `account::offline::*` / `account::session::*`，`commands/auth/mod.rs` re-export 同步更新

#### 3.11 + 4.3 window_title.rs 拆分 + 9 处命令迁入 shell 模块（403 → 73 mod.rs + 155 windows.rs + 73 macos.rs + 99 linux.rs）
- 拆为 `window_title/{mod,windows,macos,linux}.rs`，原 `window_title.rs` 已删除，由 `window_title/` 目录替代
- `mod.rs`（73 行）：跨平台公共 API（`apply_window_title` 60s 等待 + 5 分钟持续改写轮询循环 + `render_title` 模板渲染支持 `{date}`/`{time}`）+ 平台分发 `use`（`#[cfg(windows)]` / `#[cfg(target_os="macos")]` / `#[cfg(target_os="linux")]`）+ 子模块声明
- `windows.rs`（155 行）：Win32 API 实现（`EnumWindows` 枚举 + 三层过滤：类名 GLFW30/LWJGL/SunAwtFrame + 排除辅助窗口 + 进程启动时间 ≥ Java 进程；`SetWindowTextW` 改写标题）
- `macos.rs`（73 行）：osascript 实现，改用 `crate::minecraft::system::shell::run_osascript`（原直接 `Command::new("osascript")` 迁移到 shell 模块）
- `linux.rs`（99 行）：xdotool/wmctrl 实现，改用 `shell::xdotool_search_pid` / `shell::xdotool_set_window_name` / `shell::wmctrl_list` / `shell::wmctrl_rename` / `shell::ps_pid_exists`（原 7 处直接 `Command::new` 迁移到 shell 模块）
- `minecraft/system/shell.rs` 新增 6 个封装函数（`#[cfg(target_os="...")]`）：
  - macOS：`run_osascript(script) -> Result<Output, String>`
  - Linux：`xdotool_search_pid(pid, only_visible)` / `xdotool_set_window_name(window_id, title)` / `wmctrl_list()` / `wmctrl_rename(old, new)` / `ps_pid_exists(pid) -> bool`
  - 所有函数均带 `[Shell]` 前缀日志 + `shell_err` 统一错误格式
- 完成 V2 报告 4.3：9 处跨平台 `std::process::Command::new` 全部迁入 shell 模块（macOS 2 处 + Linux 7 处）

### 重构（代码质量 V2 - 阶段 4：约定违规修复）

#### 4.1 + 4.2 shell 模块迁移
- `minecraft/system/shell.rs` 新增 `run_executable_output(program, args, cwd) -> Result<Output, String>` 通用封装（统一 `[Shell]` 前缀日志 + Windows CREATE_NO_WINDOW + `shell_err` 错误格式）
- `minecraft/java/detect.rs:27` 的 `java -version` 执行改用 `shell::run_executable_output`
- `minecraft/launch/pipeline/pre_launch.rs:33` 的 `cmd /C` / `sh -c` 改用 `shell::run_executable_output`

#### 4.4 插件子进程评估
- 评估为不适用：`plugins/spawn.rs` 使用 `tokio::process::Command`（异步）+ 权限校验 + 命令白名单 + 超时控制 + 输出截断，是沙箱子进程执行器，非系统 shell 调用，不迁移

#### 4.5 配置命令评估
- 保留 `get_config_value` / `set_config_value` 命令：Java path 有意不走 AppConfig（存独立 `[Java]` INI section），这两个命令作为非 AppConfig 配置的通用读写出口

#### 4.6 原生 button 替换（6 个文件 9 处）
- `SkinManager.vue:79` 关闭按钮 → `<Button type="ghost" size="mini">`
- `SettingsAdvanced.vue:202` 密码显示切换 → `<Button type="ghost" size="mini">`
- `TaskGroupCard.vue:67,85` 暂停/恢复 + 取消按钮 → `<Button type="text" size="mini">`（保留语义色 via `!` class 覆盖）
- `TaskGroupCard.vue:101` 分组折叠行 → `<div role="button">`（结构性元素）
- `InstalledList.vue:170,181,189,200` play/stop/launch/delete 按钮 → `<Button type="text">` + 自定义 class 覆盖（`.play-btn` / `.delete-btn` scoped 样式保持）
- `Settings.vue:92` 侧边栏分类项 → `<div role="button">`（结构性导航元素）
- `OverviewTab.vue:57` 收藏按钮 → `<Button type="text" size="small">` + `!` class 覆盖（黄色/灰色语义色）
- `OverviewTab.vue:103` Select 自定义触发器 → `<div role="button">`（Select 组件 trigger slot 结构性元素）

#### 4.7 `:title` 属性替换为 `<Tooltip>`（3 处实际违规）
- `ExternalDownload.vue:59` 下载目录 `<span :title>` → `<Tooltip>` 包裹
- `HomeClockCard.vue:103` 轮播指示点 `:title` → `<Tooltip>` 包裹 + 原生 button 转 `<div role="button">`
- `ColorPicker.vue:225` 预设色板 `:title` → `<Tooltip>` 包裹 + 原生 button 转 `<div role="button">`
- `QuickTools.vue` 已无 `:title`（此前已修复）；`ResourceDetail.vue:241` 的 `:title` 是 `<VersionGroupCard>` 组件 prop 非原生 title 属性（误报）

#### 4.8 空状态补 icon + 居中布局
- `LogViewer.vue:131-133` 暂无日志 → `DocumentTextIcon` + flex 垂直水平居中
- `ResourceDetail.vue:253-256` 暂无版本数据 → `ArchiveBoxXMarkIcon` + flex 垂直水平居中

#### 工具页扩展：版本隔离参数 + NavSidebar 公共组件
- 截图管理 / 资源包转换 / 存档管理 三工具新增 `version_id` 参数，
  适配版本隔离模式（按版本隔离配置解析 saves / screenshots / resourcepacks 目录）。
  后端 [src-tauri/src/commands/tools/types.rs](src-tauri/src/commands/tools/types.rs) 新增 `*ListParams` 结构，
  [mod.rs](src-tauri/src/commands/tools/mod.rs) 改为解析 params，
  前端 [src/utils/api/tools.ts](src/utils/api/tools.ts) 对应 API 加 `versionId` 可选参数。
- 抽取 [NavSidebar.vue](src/components/common/NavSidebar.vue) 公共组件，
  [Settings.vue](src/views/Settings.vue) / [VersionSettings.vue](src/views/VersionSettings.vue) 移除内联侧边栏代码改用公共组件，
  支持 tab 同步到 URL query。
- 新增工具分类页骨架：[diagnostic](src/views/tools/diagnostic/) / [game-resource](src/views/tools/game-resource/) / [java](src/views/tools/java/)。
- DataExporter 从数据工具迁入 [QuickTools.vue](src/views/QuickTools.vue)；[DataPage.vue](src/views/tools/data/DataPage.vue) 删除（拆分后无用）。
- [toast.ts](src/utils/toast.ts) 的 error / warning / info 同步打印 console 日志便于追踪。

#### cubiomes 转为 git submodule
- 背景：seedmap 架构迁移后 `src-tauri/cubiomes/` 是 fork 仓库（https://github.com/MoTeam-cn/cubiomes）
  的本地 clone，含嵌套 `.git` 目录，git 无法将其内部源码（含 `cubiomes_wrapper.c`）作为普通文件
  纳入主仓库，导致 clone 的人无法复现 WASM 编译。
- 变更：
  - cubiomes fork 仓库新增 commit `0617539`：`cubiomes_wrapper.c` 扩展 WASM 封装层，
    支持群系查询与 7 个结构查找（ravine 系列 / nether_fossil / fossil / fossil_diamond）。
  - 主仓库移除嵌套目录，改为 git submodule 引用：
    - 新增 [.gitmodules](.gitmodules) 注册 `src-tauri/cubiomes` → `https://github.com/MoTeam-cn/cubiomes`。
    - submodule 锚定 commit `0617539`（heads/master）。
- 使用：clone 主仓库后需执行 `git submodule update --init --recursive` 拉取 cubiomes 源码，
  之后 build.rs 会自动调用 emcc 编译 WASM。

#### 清理云端误追踪文件与 Cargo.toml 注释
- 背景：`src-tauri/Cargo.lock` 与 `logo_data/` 早已在 `.gitignore` 排除，但早期误提交至云端
  仍被追踪；`Cargo.toml` 两处依赖注释含 "参考 PCL2" 字样需移除。
- 变更：
  - [.gitignore](.gitignore)：`# Rust` 段新增 `src-tauri/Cargo.lock` 显式排除（与全局 `Cargo.lock` 并列）。
  - `git rm --cached src-tauri/Cargo.lock`：从索引移除，本地文件保留，下次 push 后云端不再追踪。
  - `git rm --cached -r logo_data/`：同上清理 3 个 logo 数据文件。
  - [src-tauri/Cargo.toml](src-tauri/Cargo.toml)：`notify` 与 `windows` 依赖注释移除 "参考 PCL2 ..." 字样。

#### 服务器状态检测 MOTD 彩色渲染（§ 格式化代码解析）
- 背景：服务器状态检测工具的 MOTD 显示为纯文本，丢失了 Minecraft 多人联机 § 颜色/格式代码
  的彩色效果（如 §a 绿色、§l 粗体、§c 红色），视觉体验差。
- 根因：后端 [network.rs](src-tauri/src/commands/tools/network.rs) `extract_motd` 主动 `strip_section_codes`
  剥离了 § 代码，前端 [ServerPinger.vue](src/views/tools/network/ServerPinger.vue) 直接 `{{ result.motd }}` 纯文本渲染。
- 方案：后端新增 `motd_raw` 字段保留 § 代码（`motd` 纯文本字段保留向后兼容），前端新增 `parseMcMotd`
  工具函数解析为带样式的 HTML span，组件 `v-html` 渲染。
- 变更：
  - [src-tauri/src/commands/tools/types.rs](src-tauri/src/commands/tools/types.rs)：
    `ServerPingResult` 新增 `motd_raw: String` 字段（保留 § 格式化代码）。
  - [src-tauri/src/commands/tools/network.rs](src-tauri/src/commands/tools/network.rs)：
    `extract_motd` 重命名为 `extract_motd_raw`（保留 § 代码），调用处改为先取 raw 再 `strip_section_codes`
    生成纯文本 `motd`，两字段一并返回。
  - [src/utils/motd.ts](src/utils/motd.ts)：新建（120 行）。导出 `parseMcMotd(raw)` 函数，
    解析 § + 颜色代码（0-9/a-g，对应 MC Java 调色板）和格式代码（k 混淆/l 粗体/m 删除线/n 下划线/o 斜体/r 重置）
    为 HTML span + inline style。颜色代码隐式重置格式（对齐 MC 行为）。输出做 HTML 转义防 XSS。
  - [src/utils/api/tools.ts](src/utils/api/tools.ts)：`ServerPingResult` 接口新增 `motd_raw: string` 字段。
  - [src/views/tools/network/ServerPinger.vue](src/views/tools/network/ServerPinger.vue)：
    MOTD 显示区改为 `v-html="parseMcMotd(result.motd_raw)"`（优先用 raw 解析彩色），
    无 raw 时回退纯文本 `motd` 字段。
- 白底配色适配：MC 原版调色板为暗背景设计，亮色（a/b/c/d/e/g）在白底上刺眼难读。
  [motd.ts](src/utils/motd.ts) `COLOR_MAP` 整体下调明度（如 #55FF55 → #1E8B1E、#FFFF55 → #B09000），
  保留色相区分，确保白底可读性。
- 纯文本/彩色切换：MOTD 区右侧新增切换按钮（复用 Button.vue + Tooltip.vue），
  图标用 DocumentTextIcon/PaintBrushIcon（不用 Emoji）。仅当 `motd_raw` 与 `motd` 不一致时显示。

#### 修复 23 个 tsc 类型错误（清零）
- 背景：长期累积的 TypeScript 类型错误，虽不影响运行但污染类型检查输出。
- 修复清单（共 12 文件）：
  - [src/composables/useAccountCards.ts](src/composables/useAccountCards.ts) +
    新建 [src/components/home/account-selector/types.ts](src/components/home/account-selector/types.ts)：
    `AccountCardData` 接口抽离到独立 .ts（`*.vue` shim 不支持命名导出），`logoutUser` → `logout`。
  - [src/stores/version.ts](src/stores/version.ts)：`loaderVersionsCache.forge` 类型从 `string[]`
    改为 `{ version; is_recommended; release_time }[]`，与 `listForgeVersions` 返回值一致。
  - [tsconfig.json](tsconfig.json)：`lib` 从 `ES2020` 改为 `ES2021`，支持 `Promise.any`。
  - [src/composables/useModList.ts](src/composables/useModList.ts)：8 处 `toastSuccess(text1, text2)` 合并为单参数模板字符串。
  - [src/composables/useModUpdate.ts](src/composables/useModUpdate.ts)：`props.mod` 在异步回调外捕获到局部变量 `mod` 避免非空收窄丢失。
  - [src/composables/useSearchProgress.ts](src/composables/useSearchProgress.ts)：删除未使用 `startTime` 变量。
  - [src/composables/useSkinOperations.ts](src/composables/useSkinOperations.ts)：删除未使用 `defaultSkins` import。
  - [src/plugins/custom-layout/datasource.ts](src/plugins/custom-layout/datasource.ts)：删除空 import 语句。
  - [src/router/index.ts](src/router/index.ts)：未使用 `from` 参数改为 `_from`。
  - [src/stores/plugins.ts](src/stores/plugins.ts)：`ExternalPluginEntry` 改从 `@/utils/api/plugins` import；`onGameLaunch/onGameExit` 用 `Promise.resolve()` 包装返回值。
  - [src/utils/api/config.ts](src/utils/api/config.ts)：`configCache as ConfigSnapshot` 改为 `as unknown as ConfigSnapshot`。
  - [src/utils/api/skin.ts](src/utils/api/skin.ts) + [src/utils/tauri.ts](src/utils/tauri.ts)：`CachedImage` 接口移至 `image-cache.ts` 统一持有，删除 `skin.ts` 重复定义。

#### 修复种子地图缩放时 ravine/fossil 结构丢失（范围过大整片跳过）
- 背景：用户反馈缩放地图时控制台疯狂输出 `[cubiomes] ravine 范围过大，跳过` /
  `mega_ravine` / `underwater_ravine` / `fossil` / `fossil_diamond` 等警告，
  导致这些结构类型在大缩放级别下完全无法加载（地图空白）。
- 根因：[generatorWorker.ts](src/utils/seedmap/generatorWorker.ts) `callChunkFinder` 在
  `numX > sizeLimit || numZ > sizeLimit`（非 mega 64 chunks / mega 32 chunks）时直接 `return null`
  整片跳过。但缩放时可视范围常超过 1024 方块（64×16），导致所有 ravine/fossil 类结构被跳过。
- 修复：将 `callChunkFinder` 改为**分块查找**模式：
  - 范围在 `sizeLimit` 内：单次 WASM 调用（原逻辑）
  - 范围超过 `sizeLimit`：将大范围切分为 `sizeLimit × sizeLimit` 的子块，逐个调用 WASM 查找，合并所有结果
  - 抽取 `callFinderOnce` 函数承载单次调用（buffer 分配 → WASM 调用 → 结果读取 → 释放）
  - 移除 3 处 `console.warn('范围过大，跳过')`（分块后不再返回 null，除非内存分配失败）
- 效果：缩放时 ravine/fossil/nether_fossil 类结构正常加载，控制台无警告刷屏。

#### 计算工具页面重构（自研组件替换原生 HTML + 交互优化）
- 背景：计算工具页（坐标计算 + 调色板）多处使用原生 `<button>` / `<input>` 违反项目硬约束
  （必须用自研组件），且交换按钮 icon-only 视觉不清晰。
- 变更：
  - [src/views/tools/calc/CoordCalculator.vue](src/views/tools/calc/CoordCalculator.vue)：
    交换按钮改用 [Button.vue](src/components/common/Button.vue) type="outline" + 文字标签
    （原 icon-only ghost 按钮视觉不清晰）；地狱门换算模式按钮改用 Button（primary/outline）。
  - [src/views/tools/calc/ColorPalette.vue](src/views/tools/calc/ColorPalette.vue)：
    HEX 输入改用 [Input.vue](src/components/common/Input.vue)；复制按钮、格式化代码按钮
    改用 [Button.vue](src/components/common/Button.vue)；RGB 数字输入改用 Input。
    RGB range 滑块保留原生（项目无 Slider 组件，range 是浏览器唯一实现，属合理例外）。
    染料色块保留原生 button（纯色块无文字，Button 组件 padding/hover 不适用）。
    抽取 `formatCodes` 常量到 script（原内联 v-for 数组），提升可读性。

### 待实现
- Mod 管理功能
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

*本文档最后更新于 2026-07-21*
