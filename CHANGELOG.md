# 更新日志

本项目的所有重要更改都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)。

## [未发布]

### 新增

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

*本文档最后更新于 2026-06-26*
