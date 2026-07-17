# 更新日志

本项目的所有重要更改都将记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
并且本项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)。

## [未发布]

### 修复

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
