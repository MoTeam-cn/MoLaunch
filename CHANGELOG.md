# Changelog

本项目所有重要变更均会记录在此文件中。格式参考 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)。

## [Unreleased]

### Added

- **工具页新增「指令生成」分类与三类纯前端指令生成器**（[CommandPage.vue](src/views/tools/command/CommandPage.vue) / [ItemEditor.vue](src/views/tools/command/ItemEditor.vue) / [SignShop.vue](src/views/tools/command/SignShop.vue) / [SummonEntity.vue](src/views/tools/command/SummonEntity.vue) / [data.ts](src/views/tools/command/data.ts) / [generator.ts](src/views/tools/command/generator.ts) / [Tools.vue](src/views/Tools.vue)）：新增「指令生成」分类，纯前端离线生成 Minecraft Java 指令——物品编辑（/give）复用合成配方物品数据（5000+ 条）支持搜索选择、数量与目标玩家（@p/@a/@s/@r）、自定义名称/16 色、Lore 多行、34 种常用附魔动态增删；告示牌商店（/setblock）支持 12 种告示牌类型、4 朝向、四行文字与颜色；召唤实体（/summon）支持 30 种常见实体、`~` 相对坐标、数量与自定义名称。SNBT/文本组件转义与指令拼接逻辑集中在独立纯函数模块 `generator.ts`（JSON 双引号转义 + 单引号 SNBT 包裹），分类页复用 SubTabBar 顶部菜单栏并支持 `?subtab=` 深链直达子工具。

- **联机 P2P 组网失败诊断：友好提示 + 双方 NAT 类型 + TURN 无资源时给出 FRP 替代方案**（[protocol.ts](src/utils/online/protocol.ts) / [protocol.rs](src-tauri/src/minecraft/online/protocol.rs) / [onlineSession.ts](src/composables/online/onlineSession.ts) / [useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts) / [useRoomHost.ts](src/composables/useRoomHost.ts) / [P2pFailureCard.vue](src/components/online/P2pFailureCard.vue) / [RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue) / [RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) / [ParticipantList.vue](src/components/online/ParticipantList.vue) / [format.ts](src/utils/online/format.ts)）：协议层新增控制消息 `NatType(0x07)`（前后端同步），DataChannel 建立后双方自动交换 NAT 类型；加入方面板连接失败时以全新 `P2pFailureCard` 替代僵硬错误提示——用通俗文案解释「双方网络都处于较严格 NAT、打洞未穿透」，并绘制「我的网络 / 对方网络」的 NAT 徽章（悬停可看联机兼容性说明）；当服务端 TURN 中继无可用资源（未拉取到或返回空列表）时补充给出第三方内网穿透 / FRP（SakuraFrp、花生壳）、虚拟组网（ZeroTier、Radmin VPN、Tailscale）、路由器 UPnP/DMZ 等替代联机方案；房主侧对 `failed` 参与者同样展示诊断卡片，参与者列表新增 NAT 徽章列；NAT 元数据解析提取为共享 `resolveNatMeta`。

- **工具页新增「创作工具」分类与渐变文字生成器**（[GradientTextPage.vue](src/views/tools/creation/GradientTextPage.vue) / [Tools.vue](src/views/Tools.vue) / [utils/gradient-text](src/utils/gradient-text)）：离线生成 Minecraft 兼容渐变文字，支持多行文本 + 行级格式（粗体/斜体/下划线/删除线/混淆）、颜色停靠点增删排序、MC 原版阴影预览、19 种输出格式（Vanilla / Vanilla compatible / Standard HEX / CMI / MiniMessage / MineDown / SNBT / TrChat / TabooLib / Chat Colors / MOTD / BBCode / JSON / HTML / CSV / Terraria 等）、复制/下载导出与颜色预设保存导入，状态经 localStorage 持久化；核心逻辑（插值/空白豁免/阴影计算/格式适配）为独立纯逻辑模块 `src/utils/gradient-text/`，含 9 项单测。

- **创作工具分类新增「合成配方生成器」**（[CreationPage.vue](src/views/tools/creation/CreationPage.vue) / [RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue) / [utils/recipe-generator](src/utils/recipe-generator) / [scripts/generate-recipe-assets/generate.mjs](scripts/generate-recipe-assets/generate.mjs) / [recipe_generator.rs](src-tauri/src/commands/tools/recipe_generator.rs)）：离线可视化生成 Minecraft 合成配方数据包。覆盖 1.12～26.2 共 19 个版本，9 种配方类型（合成/无序合成/熔炉/高炉/烟熏/营火/切石/锻造合成/纹饰锻造），自动按版本切换四种 JSON 格式策略（legacy `{item,data}` / object / id-result / string 直写）、数据包目录（recipes→recipe、items→item）与 pack_format 单值/区间写法；支持合成网格任意摆放与空白裁剪、自定义物品/标签、cooking 经验与时长、show_notification、纹饰图案等；物品图标使用内置 16×16 简化纹理图集（minecraft-assets 生成，1180 纹理 2048×528），双语文案（中英）；导出为数据包 zip（1.13+，走 `recipe_generator_export` 后端打包）或单配方 JSON（1.12）；核心逻辑含 32 项单测。

### Removed

- **移除工具模块「启动器数据导出」功能**（[data_export.rs](src-tauri/src/commands/tools/data_export.rs) / [types/data_export.rs](src-tauri/src/commands/tools/types/data_export.rs) / [dispatcher.rs](src-tauri/src/commands/tools/dispatcher.rs) / [mod.rs](src-tauri/src/commands/tools/mod.rs) / [types/mod.rs](src-tauri/src/commands/tools/types/mod.rs) / [DataExporter.vue](src/views/tools/data/DataExporter.vue) / [QuickTools.vue](src/views/QuickTools.vue) / [data.ts](src/utils/api/tools/data.ts) / [core.ts](src/utils/api/tools/core.ts)）：便捷工具页「启动器数据导出」无实际意义，前后端整体移除——后端删除 `data_export` 模块与 `export_launcher_data` action 注册，前端删除 `DataExporter.vue` 组件、`exportLauncherData` 封装及 `EXPORT_LAUNCHER_DATA` action 常量；`tools/data.ts` 中其余数据类工具（崩溃分析 / 截图 / 资源包 / 版本 JSON / NBT）封装保留。

### Changed

- **指令生成工具下拉改用公共 Select 组件**（[ItemEditor.vue](src/views/tools/command/ItemEditor.vue) / [SignShop.vue](src/views/tools/command/SignShop.vue) / [SummonEntity.vue](src/views/tools/command/SummonEntity.vue) / [generator.ts](src/views/tools/command/generator.ts)）：物品编辑（目标玩家 / 名称颜色 / 附魔）、告示牌商店（类型 / 朝向 / 文字颜色）、召唤实体（实体 / 名称颜色）共 8 处浏览器原生 `<select>` 全部替换为项目公共 `Select` 组件，下拉交互与设置页保持一致；16 色选项映射提取为公共 `COLOR_OPTIONS` 供三个组件复用，避免重复实现。

- **工具页左侧菜单归并为 6 个一级分类**（[Tools.vue](src/views/Tools.vue) / [CommonPage.vue](src/views/tools/CommonPage.vue) / [StoragePage.vue](src/views/tools/StoragePage.vue) / [ModNetworkPage.vue](src/views/tools/ModNetworkPage.vue) / [JavaDiagPage.vue](src/views/tools/JavaDiagPage.vue) / [CreateCmdPage.vue](src/views/tools/CreateCmdPage.vue)）：左侧菜单由 13 项精简为 6 项——下载管理（外部下载）、常用工具（今日人品 / 便捷工具 / 坐标计算 / 调色板）、存档资源（存档管理 / 截图管理 / 资源包转换 / 种子地图）、Mod 网络（依赖检测 / Mod 去重 / 服务器检测 / 延迟测试）、Java 诊断（Java 下载器 / 环境检测 / 运行时列表 / 版本 JSON / NBT 查看）、创作指令（渐变文字 / 合成配方 / 物品编辑 / 告示牌商店 / 召唤实体）；原 13 个分类页统一归并为 5 个组合分类页（内部顶部 SubTabBar 切换，支持 `?tab=xxx&subtab=yyy` 深链直达），删除 10 个被合并的分类页文件。

- **所有工具分类页重构为顶部子菜单切换，并移除工具页右侧 TOC 悬浮导航**（[Tools.vue](src/views/Tools.vue) / [ArchivePage.vue](src/views/tools/archive/ArchivePage.vue) / [ModToolsPage.vue](src/views/tools/mod-tools/ModToolsPage.vue) / [NetworkPage.vue](src/views/tools/network/NetworkPage.vue) / [CalcPage.vue](src/views/tools/calc/CalcPage.vue) / [JavaPage.vue](src/views/tools/java/JavaPage.vue) / [DiagnosticPage.vue](src/views/tools/diagnostic/DiagnosticPage.vue) / [GameResourcePage.vue](src/views/tools/game-resource/GameResourcePage.vue) / [SeedMapPage.vue](src/views/tools/seedmap/SeedMapPage.vue) / [CreationPage.vue](src/views/tools/creation/CreationPage.vue) / [GradientTextPage.vue](src/views/tools/creation/GradientTextPage.vue)）：存档管理 / Mod 工具 / 网络工具 / 计算工具 / Java 管理 / 诊断工具 / 游戏资源 / 种子地图八个分类页与创作、指令生成分类保持一致，由多工具垂直叠加改为顶部 `SubTabBar` 菜单栏（sticky 吸顶、带图标），每次只渲染当前工具，并统一支持 `?subtab=` 深链直达子工具；子工具锚点 `id` 保留供外部跳转。同时放弃工具页 TOC：`Tools.vue` 移除 `ToolToc` 组件引用与 `tocRefreshKey` 重扫逻辑（外容器 padding 改为仅对直接在 Tools.vue 渲染的 3 个分类生效），`CreationPage`/`GradientTextPage` 清理 `data-toc-*` 标注；`ToolToc` 组件本身保留，现仅由实验性 AI 聊天页的消息快捷跳转使用。

- **微软登录链路抓取并持久化 Xbox 用户 ID（xuid）**（[exchange.rs](src-tauri/src/minecraft/auth/microsoft/exchange.rs) / [types.rs](src-tauri/src/minecraft/auth/storage/types.rs) / [save.rs](src-tauri/src/minecraft/auth/storage/save.rs) / [load/file.rs](src-tauri/src/minecraft/auth/storage/load/file.rs) / [load/registry.rs](src-tauri/src/minecraft/auth/storage/load/registry.rs) / [registry.rs](src-tauri/src/minecraft/auth/storage/registry.rs) / [state/auth.rs](src-tauri/src/state/auth.rs) / [types.rs](src-tauri/src/minecraft/launch/types.rs)）：登录/刷新时从 XSTS 响应 `DisplayClaims.xui[0].xui` 提取 Xbox 用户 ID，随微软账号与当前用户持久化（`StoredMsAccount`/`CurrentUser`/`LocalAuthResult`/`AuthInfo` 新增 `xuid` 字段，`#[serde(default)]` 兼容旧数据，Windows 注册表新增 `MsCurrentXuid` 加密键、非 Windows 加密文件新增 xuid 字段，账号切换/刷新/静默刷新路径同步更新），供后续启动参数 `--xuid` 使用；离线/authlib 账号该字段为空。

- **配方抽屉新增方向键快速切换目标格**（[RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue)）：抽屉提示「您正在为「第 X 格」选择物品」右侧新增上/下/左/右四个方向键按钮，无需关闭抽屉即可在格子间切换（合成网格按行列移动，2×2/3×3 自动适配；熔炼/切石/锻造等线性槽位按上一个/下一个移动），到达边界时对应按钮自动禁用。

- **合成配方输入槽支持滚轮调整数量**（[RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue) / [formatter.ts](src/utils/recipe-generator/formatter.ts)）：此前仅结果槽可滚轮调产出数量。现所有已放置物品槽位（合成格/原料/模板/底材/材料）均可滚轮调整 1-64；JSON 输出中 1.21.2+（string 策略）的输入槽位携带 `count` 字段（`{item, count}` / `{tag, count}`，旧版本策略不输出以保证数据包合法），如「2 个锻造模板 + 树苗」的合成。

- **合成配方生成器背景图统一迁移至 `src/assets/Syn/`**（[recipe-layouts.ts](src/views/tools/creation/recipe-generator/recipe-layouts.ts)）：背景图此前存放在 `recipe-generator/assets/bg/` 局部目录，现统一迁至全局资源目录 `src/assets/Syn/`（5 张临时占位图不变，后续自行替换）。

- **合成配方生成器槽位改为「工作台界面背景图 + 热点」布局**（[recipe-layouts.ts](src/views/tools/creation/recipe-generator/recipe-layouts.ts)（新增） / [RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue) / [RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue) / [vite.config.ts](vite.config.ts)）：此前除合成外的配方类型均复用抽象网格/行布局，观感与 Minecraft 原版工作台界面不一致。现引入布局数据模块 `recipe-layouts.ts`——每种配方类型绑定对应工作台 GUI 背景图（合成/熔炼/篝火/切石机/锻造共 5 张，696×292 坐标系），槽位以像素盒精确定位；编辑器按布局渲染背景图与绝对定位热点，点击空热点弹抽屉选物、点击已放置槽位清除、结果槽滚轮调数量、标签悬浮成员浮层与 2×2 禁用槽 barrier 遮罩均保留。背景图暂为临时占位（代码中已标注，后续自行替换）。vite.config.ts 增加 vitest `include: ['src/**/*.test.ts']`，避免扫描到工作区内第三方源码目录的测试文件。

- **合成配方生成器床物品图标改为 3D 立体渲染**（[bed-render.mjs](scripts/generate-recipe-assets/bed-render.mjs)（新增） / [generate.mjs](scripts/generate-recipe-assets/generate.mjs)）：图集中 16 色床物品此前误用 64×64 实体「折叠展开图」（`entity/bed/<color>` 被映射为经典实体贴图），在槽位中显示为平面展开且模糊。现依据 26.2 官方床模型几何（`template_bed_head/foot`）与按面拆分的 9 张分面贴图（`bed_head_north` / `bed_down` / 各色 `_bed_head_{east,up,west}` / `_bed_foot_{east,south,up,west}`），在生成器中内置软件 3D 光栅化（painter's 深度排序 + z-buffer + 重心插值纹理采样 + 4× 超采样 + bbox 居中适配），渲染参数与游戏内 `display.gui`（rotation [30,340,0] / translation [2,3,0] / scale [0.5325]）一致，为 16 色床各渲染一张 16×16 立体图标（含床垫、床头高板、床尾矮板与四条腿），替换图集内 `entity/bed/<color>` 条目；图集纹理数 1180→1209，图集尺寸同步更新。

- **合成配方调色板改为虚拟滚动，滑动不再卡顿**（[ItemPalette.vue](src/views/tools/creation/recipe-generator/ItemPalette.vue) / [TagPalette.vue](src/views/tools/creation/recipe-generator/TagPalette.vue)）：物品调色板最多 1180 个条目、标签调色板同样量大，此前一次性渲染全部 DOM，滚动与输入搜索明显卡顿。现复用项目已内置的 `vue-virtual-scroller`（`RecycleScroller`）只渲染可视区条目（物品按 4 列一行虚拟化，标签按行虚拟化），列表高度行为保持不变。

- **创作工具顶部子菜单紧贴页面左上角**（[Tools.vue](src/views/Tools.vue) / [CreationPage.vue](src/views/tools/creation/CreationPage.vue)）：此前子菜单距离顶部与左侧各有 24px 内边距（来自工具页滚动容器 `p-6`），且负外边距方案受 margin collapse 影响顶部始终无法贴边。现采用与设置页 about 页签完全相同的方案——工具页外容器对 creation 分类不设 padding，由 CreationPage 自带 `p-6`，顶部子菜单渲染在 padding 之外紧贴左上角，滚动时仍 sticky 吸顶。

- **创作工具分类页改为顶部子菜单切换**（[CreationPage.vue](src/views/tools/creation/CreationPage.vue)）：渐变文字生成器与合成配方生成器此前垂直叠加一页放不下，现复用设置页同款 `SubTabBar` 顶部菜单栏（sticky 吸顶、带图标），每次只渲染当前工具；支持 `?subtab=` 深链直达指定工具，`#tool-gradient-text` / `#tool-recipe-generator` 目录锚点保留。

- **合成配方生成器改为左右双栏布局**（[RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue)）：原三栏（设置｜编辑+预览｜调色板）布局在 ≤1280px 窗口折叠为单列。现重写为「左：展示区（槽位编辑 + 校验提示 + JSON 预览）+ 右：功能区（配方设置 + 物品/标签调色板）」两栏，内容多高就多高、由页面滚动条自然滚动，不锁定容器高度；JSON 预览保持最大高度内部滚动，校验提示改为复用项目 `Alert` 组件逐条展示。

- **合成配方槽位编辑器支持点击空格子编辑**（[RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue) / [formatter.ts](src/utils/recipe-generator/formatter.ts)）：点击空格子（结果槽除外）发出 `edit-slot` 请求供页面弹抽屉选择，当前编辑中的格子以蓝色高亮；槽位标题映射 `slotCaption` 提取到 `formatter.ts` 供编辑器与页面复用，避免重复实现。

- **物品/标签调色板改为抽屉选择**（[RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue)）：移除右侧常驻调色板，改为点击配方空格子时从右侧展开抽屉（`Drawer`，标题随目标槽位显示，如「选择底材」/「选择物品（第 3 格）」），抽屉内保留物品/标签页签与搜索；选中后填入该格并自动关闭，点遮罩或按 ESC 取消，正在编辑的格子持续高亮。

- **配方抽屉内容限高自适应，并新增目标槽位说明**（[ItemPalette.vue](src/views/tools/creation/recipe-generator/ItemPalette.vue) / [TagPalette.vue](src/views/tools/creation/recipe-generator/TagPalette.vue) / [RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue)）：抽屉内物品/标签调色板不再固定 28rem 高度，改为随抽屉内容区自适应填满（`flex:1; min-height:0`，虚拟滚动列表在抽屉内部滚动），展开抽屉不再把整页撑长、不再出现多余滚动条；抽屉顶部新增说明「您正在为「XX」选择物品」（如「第 3 格」「底材」），明确当前正在编辑的目标槽位。

- **标签调色板重做：多标签成行 + 中文显示**（[tag-zh.ts](src/utils/recipe-generator/tag-zh.ts) / [TagPalette.vue](src/views/tools/creation/recipe-generator/TagPalette.vue) / [RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue)）：标签不再一行一个英文 ID，改为每行 2 个标签成行展示（保留按行虚拟滚动）并新增「共 N 个标签」统计；新增内置标签中文名映射 `tag-zh.ts`（覆盖全部 235 个内置标签，未知标签回退为可读英文），调色板条目与已放入槽位的标签均显示中文名，搜索框同时支持中文名 / 英文 ID 匹配。

- **抽屉交互与主题色优化**（[RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue) / [ItemPalette.vue](src/views/tools/creation/recipe-generator/ItemPalette.vue) / [TagPalette.vue](src/views/tools/creation/recipe-generator/TagPalette.vue) / [RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue)）：抽屉内主色元素（页签激活态、物品/标签悬停高亮、正在编辑的格子边框与光晕）由硬编码蓝色改为跟随主题色变量；选择物品/标签后抽屉不再立即关闭，而是自动定位到下一个空格子（crafting 按 2x2/3x3 网格顺序、其他类型按输入槽顺序）连续放置，全部填满才关闭，避免每次点击都跳回合成区。

- **标签槽位显示成员贴图并新增悬停浏览浮层**（[tag-resolve.ts](src/utils/recipe-generator/tag-resolve.ts) / [RecipeTagPopup.vue](src/views/tools/creation/recipe-generator/RecipeTagPopup.vue) / [RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue)）：放入合成格的标签不再显示问号占位，改为取该标签下首个有贴图成员物品的贴图作为图标，格子左上角显示主题色「#」角标区分标签材料；悬停标签格弹出浮层，横排展示该标签全部有贴图成员物品的图标（标题显示标签名与成员数），内容超宽时左右缓慢自动滑动浏览，浮层跟随槽位定位并随页面滚动更新位置。

- **标签格点击清除时自动关闭悬停浮层**（[RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue)）：点击已放置标签的合成格清除材料时，正在循环滑动展示成员物品的悬停浮层立即消失，无需再等鼠标移开。

### Fixed

- **修复 CI lint 审查到 `scripts/` 下 Node 工具脚本导致流水线失败**（[.eslintrc.cjs](.eslintrc.cjs)）：`npm run lint`（`eslint . --ignore-path .gitignore`）此前会把 `scripts/` 目录一并纳入前端 ESLint 审查，`scripts/generate-recipe-assets/generate.mjs`（资产生成脚本，运行于 Node 环境）因使用 `Buffer`/`process` 等 Node 全局与未使用变量直接报错退出。现 `ignorePatterns` 追加 `scripts/**`——脚本为本地资产生成 / CI 上传等工具代码，不参与前端 lint；git 跟踪不受影响，脚本继续入库。

- **修复启动过程中「停止启动」不立即生效、文件校验继续运行**（[runner.rs](src-tauri/src/minecraft/launch/pipeline/runner.rs) / [execute.rs](src-tauri/src/minecraft/launch/pipeline/execute.rs) / [validate.rs](src-tauri/src/minecraft/launch/pipeline/validate.rs) / [fix.rs](src-tauri/src/minecraft/download/fix.rs) / [manager/core.rs](src-tauri/src/minecraft/download/manager/core.rs) / [useLaunchState.ts](src/composables/useLaunchState.ts) / [LaunchPanel.vue](src/components/home/LaunchPanel.vue)）：此前 `cancel_flag` 为异步 `Mutex<bool>`，文件校验阶段（`validate_and_fix_files`）构造的 `DownloadManager` 未接入取消信号，且校验过程无取消检查——点击「停止启动」后下载/扫描仍继续跑到结束，最后才输出 `启动失败：[文件检查] 启动已取消`。现：① `cancel_flag` 改为 `AtomicBool` 并新增 `is_cancelled()`，可直接共享给下载管理器；② 校验阶段下载管理器接入取消标志，下载任务感知取消后立即中止；③ `validate_and_fix_files` 在读取版本信息、构造管理器、文件补全前/后逐段检查取消并立即以「启动已取消」返回（`fix_version_files` 三个阶段间同样检查），无需等待全部检查完成；④ 前端在用户主动取消时不再弹出「启动失败/启动已取消」错误提示（启动面板已在点击取消时提示「已取消启动」），避免重复且误导的失败弹窗。

- **修复高版本启动参数占位符未替换、feature 规则未评估**（[rules.rs](src-tauri/src/minecraft/version/libraries/parse/rules.rs) / [parse/mod.rs](src-tauri/src/minecraft/version/libraries/parse/mod.rs) / [libraries/mod.rs](src-tauri/src/minecraft/version/libraries/mod.rs) / [game_args.rs](src-tauri/src/minecraft/launch/game_args.rs) / [arguments.rs](src-tauri/src/minecraft/launch/arguments.rs) / [storage/registry.rs](src-tauri/src/storage/registry.rs)）：高版本（如 26.2 / 1.21.4）version JSON 的 `arguments.game` 中 `--width/--height`、`--quickPlay*`、`--clientId`、`--xuid` 等参数通过 feature 规则控制注入且携带 `${...}` 占位符，此前规则评估仅识别 `is_demo_user`，其余 feature 条目（`has_custom_resolution`、`is_quick_play_*`）一律放行且占位符缺失替换，导致启动参数出现 `--width ${resolution_width}`、`--clientId ${clientid}`、`--xuid ${auth_xuid}` 等未替换变量。现：① `check_rules` 重构为薄包装并新增 `check_rules_with_features`，按 feature 名实际取值评估规则（缺失视为 `false`，向后兼容）；② 游戏参数构建按 `has_custom_resolution`（宽高是否已设置）评估规则，快速联机/演示条目正常过滤；③ 补齐 `${clientid}`、`${auth_xuid}`、`${resolution_width/height}` 占位符替换，`${quickPlayPath/Singleplayer/Multiplayer/Realms}` 空串兜底；④ 分辨率与 `--quickPlayMultiplayer` 手动追加增加去重保护，避免与 JSON 注入重复；⑤ 启动器 clientId 以 UUIDv4 生成一次并持久化到系统注册表 KV（Windows 注册表 / 非 Windows system.json），跨启动复用。

- **修复多阶段下载时下载面板每次阶段切换闪烁/误跳回主页**（[core.rs](src-tauri/src/minecraft/download/manager/core.rs) / [mod.rs](src-tauri/src/minecraft/download/manager/mod.rs) / [full_download.rs](src-tauri/src/minecraft/download/full_download.rs) / [fix.rs](src-tauri/src/minecraft/download/fix.rs) / [session.rs](src-tauri/src/minecraft/download/session.rs)）：安装/补全版本时「资源文件→库文件」等阶段切换之间共享批次计数 `panel_active_count` 短暂归零，`download_batch` 立即 emit `download-panel-state {visible:false}`，前端下载面板监听将 `downloading` 置 false，Downloads 页 `watch` 据此执行 `router.back()` 把用户踢回主页。现下载管理器新增 `hold_panel()`/`release_panel()`（在共享计数上叠加一次持有，不触发瞬态归零），并配套 RAII 守卫 `PanelLease`（Drop 自动释放，含错误提前返回）；完整下载（`download_version_full`）、文件补全（`fix_version_files`）与下载会话（`DownloadSession`，含 `Drop` 释放）在顶层持有一份面板会话，阶段切换时面板持续显示，结束才隐藏。

- **修复校验失败重下时下载进度虚高累计**（[retry.rs](src-tauri/src/minecraft/download/downloader/retry.rs) / [api.rs](src-tauri/src/minecraft/download/chunk/api.rs)）：资源文件（chunk 分片校验失败后回退单流、单流校验失败后换源重下、分片合并失败）等重下路径只删文件不回滚已计入的 `downloaded_bytes`，同一文件反复重下导致进度从实际约 400MB 虚高堆到 523MB。现三类失败分支统一回滚本次已计数的下载字节（`saturating_sub`），进度回到真实已落盘字节，不再虚高。

- **修复配方生成器结果槽点击无响应、非合成类型仍显示九字格**（[RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue) / [RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue) / [formatter.ts](src/utils/recipe-generator/formatter.ts)）：① 结果槽此前被 `slot !== resultSlot` 判断排除在点击编辑之外，空结果槽点击无反应（页面却提示「缺少配方产物」），现允许点击结果槽打开物品抽屉选择产物（已填点击仍为清除、滚轮调数量不变）；② `gridSlots` 此前对任何配方类型都返回 3×3 网格槽，导致熔炼/切石/锻造等类型仍显示九字格，现仅合成配方返回网格，其余类型走「一行输入槽 + 结果槽」布局（行模式此前漏渲染结果槽，现补齐并带「产物」标题）；③ 抽屉标题/槽位标签为结果槽显示「产物」，修复点击结果槽时显示「第 产物 格」的问题；④ 结果槽选完物品直接关闭抽屉，不再跳转到输入槽。

- **修复配方物品图标缺失与床图标错用羊毛**（[generate.mjs](scripts/generate-recipe-assets/generate.mjs)）：三类成因逐一修复——① 上游 minecraft-assets 的 items_textures 把各色床错误映射为对应羊毛贴图，现静态覆写为官方床实体贴图 `entity/bed/<color>`（含 1.12 旧版床）；② 1.21.9+ 起 items_textures 删除指南针条目但贴图文件仍在，覆写为 `items/compass_16`；③ 26.2「Chaos Cubed」新增的朱砂/硫黄方块族（26 个）与金蒲公英，minecraft-assets 尚未收录，现从 Mojang 官方 26.2 客户端 jar 提取 9 张基础方块纹理（台阶/楼梯/墙复用基础纹理，与既有 slab/stairs 映射惯例一致），铜傀儡像 8 变种一并覆写为官方实体贴图（waxed 复用对应氧化态）；图集重建后仅 `air` 无贴图。

- **修复配方物品/标签资源加载为空，抽屉只剩一个「default」标签**（[resources.ts](src/utils/recipe-generator/resources.ts)）：`import.meta.glob` 加载 JSON 资产时模块形状为 `{ default: <json> }`，此前直接 `await mod()` 导致物品列表始终为空、标签只读到一个 `default` 键——这正是抽屉里标签只有「default」、无法筛选的原因。现按项目约定取 `.default`（与 `aboutLogos` 一致），物品/标签正常加载；新增资源加载回归测试。

- **合成配方页切换不再报 atlas 为空的 2 个 Vue prop 校验警告**（[RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue)）：首帧渲染时 `loading=false` 而 `atlas` 仍为 `null`，`v-else` 主体分支会把 `atlas!`（实际为 null）传给 RecipeSlotsEditor / ItemPalette 触发警告。现将 `loading` 初始值改为 `true`，资源加载完成前不渲染主体。

- **合成配方生成器界面全中文化**（[versions.ts](src/utils/recipe-generator/versions.ts) / [RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue) / [ItemPalette.vue](src/views/tools/creation/recipe-generator/ItemPalette.vue) / [RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue)）：配方类型下拉由英文 ID 改为中文（合成/熔炼/高炉烧炼/烟熏/营火烹饪/切石/锻造/纹饰锻造/锻造转换），分类下拉改为中文（装备/建筑/杂物/红石/食物/方块），页头副标题、分组/分类标签、文件名占位符改为中文；物品调色板条目主文案改显中文名（英文 ID 保留在悬停提示），非合成类槽位下方英文标题（ingredient/template/base/addition）改为原料/模板/底材/材料。

- **合成配方生成器：修复物品图标图集切割错位**（[RecipeItemIcon.vue](src/views/tools/creation/recipe-generator/RecipeItemIcon.vue)）：此前背景图直接按图集原始尺寸（2048×528）平铺，而槽位/调色板图标元素为 30～38px，16×16 紧密排列的贴图会一个格子露出相邻 2 个甚至 2×2 个。现按「元素尺寸 / 贴图区域宽度」等比放大背景尺寸与偏移，每个图标只显示其对应的单个贴图。

- **合成配方生成器：修正 typecheck/lint 报错**（[formatter.ts](src/utils/recipe-generator/formatter.ts) / [versions.ts](src/utils/recipe-generator/versions.ts) / [validation.ts](src/utils/recipe-generator/validation.ts) / [resources.ts](src/utils/recipe-generator/resources.ts) / [RecipeGeneratorPage.vue](src/views/tools/creation/recipe-generator/RecipeGeneratorPage.vue) / [RecipeSlotsEditor.vue](src/views/tools/creation/recipe-generator/RecipeSlotsEditor.vue) / [RecipeItemIcon.vue](src/views/tools/creation/recipe-generator/RecipeItemIcon.vue)）：补齐 `RecipeSlot` 类型导入与网格槽位索引类型收窄、版本列表改为 `as const` 字面量数组（`metadata` 索引类型收敛）、图集 JSON 经 `unknown` 中转断言、清理未使用导入；Toast 改为项目统一 `@/utils/toast`（成功/错误/普通分级），cooking 时长输入改为 `:model-value` + 更新回调以支持 `number | null`（空值回落默认时长），图标组件 `label` prop 补默认值消除 `vue/require-default-prop` 警告。

- **版本同步补齐多语言 README 版本徽章**（[sync-version.cjs](scripts/sync-version.cjs) / [version-sync.yml](.github/workflows/version-sync.yml)）：原脚本只更新主 README.md 的 shields.io 版本徽章，README_EN / README_JA / README_ZH-HANT 的徽章停留在旧版本（如 0.3.5-rc4），打 tag 后多语言文档版本漂移。现改为遍历根目录全部 4 个语言 README 同步徽章，并同步扩展 git-auto-commit 的 file_pattern 以纳入 bot 自动提交范围。

## [0.3.6-rc3] - 2026-08-13

> 使用审查模型修复部分漏洞，可选项安全更新

### Fixed

- **先启动 MC（已开放局域网）再开房间时端口不再丢失，进房自动回查补上**（[useRoomHost.ts](src/composables/useRoomHost.ts) / [lan_probe.rs](src-tauri/src/commands/online/manager/lan_probe.rs) / [ports.rs](src-tauri/src/minecraft/launch/watcher/ports.rs) / [scheduler.rs](src-tauri/src/minecraft/launch/watcher/scheduler.rs) / [tun.ts](src/utils/api/online-manager/tun.ts)）：watcher 检测到 MC 局域网端口后经 `online://mc-port-detected` 事件推送，但事件只在端口首次变化时发出（`last_port` 按端口去重），若在开房间（监听注册）之前已开放局域网，事件被丢弃且不会重发，房间详情端口停留在建房表单默认值。现抽取 `listening_tcp_ports` 到 `watcher/ports.rs` 供联机模块复用，新增 `get_running_mc_port` action 按当前游戏进程 PID 扫描监听端口，房主进房时主动回查一次；事件回查共用 `applyDetectedPort`（含手动端口守卫与重复端口去重）。

- **`setRemoteAnswer` 幂等处理 ICE restart 并发竞态，不再误关自愈中的连接**（[mesh-peer.ts](src/composables/useWebRTCMesh/mesh-peer.ts)）：`setRemoteAnswer` 与 `closeParticipant` / ICE restart 并发时，`signalingState` 可能在幂等守卫（`!== have-local-offer`）之后才变化，`setRemoteDescription` 会抛 `InvalidStateError`。现捕获该异常视为「跳过」（返回 `false`）而非「协商失败」，调用方 `autoAcceptConfirmedAnswer` 不再对该瞬态竞态执行 `closeParticipant` 误杀一个正在恢复的连接；仅真正异常才抛出让调用方关闭残留 PC。

- **加入方 Offer 监控链在协商/重连进行中不再断裂**（[onlineSession.ts](src/composables/online/onlineSession.ts)）：`restartMonitorTick` 此前在 `reconnecting || negotiating` 时早退且**不重新排程**，若恰好在该窗口触发则监控链永久停止，房主 ICE restart 后的新 Offer 不被感知直到连接最终失败。现早退分支也调用 `scheduleRestartMonitor()`，配合 `finally` 兜底，任何路径都持续排程，不再依赖在途调用兜底。

- **加入方连接被关闭后的恢复路径补全**（[onlineSession.ts](src/composables/online/onlineSession.ts)）：`connectionState` watch 此前仅处理 `connected/failed/disconnected`，不处理 `closed`——全量重建失败后 `pc=null`、`connectionState='closed'`，`pullTurnThenRecover` 不再触发，加入方永久停留在「假在线、无连接」的半死状态。现补 `closed` 分支：角色仍为 guest 时直接 `attemptGuestReconnect`；`reconnectAttempts` 已耗尽时给出明确 toast。

- **重连时 `leaveRoom` 失败导致 `AlreadyJoined` 冲突增加重试兜底**（[useRoomReconnect.ts](src/composables/useRoomReconnect.ts)）：`reconnectAsGuest` 先 `leaveRoom` 清理旧 participant 记录，失败被 `.catch()` 静默吞掉时旧记录（status=joined）残留，`joinRoom` 返回 `AlreadyJoined` 使重连停摆。现记录清理是否成功（`leftClean`），`joinRoom` 失败且清理失败时再清一次并重试一次，仍失败才抛出。

- **ICE restart 后切回 Answer 快档，缩短断线恢复感知延迟**（[useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts)）：全参与者建连后 `answerTimer` 停在 30s 慢速档，ICE restart 上传新 Offer 后不会切回快档，房主最长 30s 后才拉到加入方重答。现 `restartIceForParticipant` 上传新 Offer 后调用 `scheduleAnswersNext()`，且 `scheduleAnswersNext` 把 `restartInFlight` 中尚未建连的参与者纳入快档判定，尽快捕获重答。

- **抑制刷新滞后窗口内 Offer 重复生成关闭进行中的 PC**（[useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts)）：服务端上传 Offer 后置 `hostOfferReady` 存在 <2s 轮询周期延迟，若此时 `offerGenerating` 已移除会触发重复生成，而 `createOfferFor` 会先关闭旧 PC 破坏进行中的协商。现新增本地 `offerReadyLocal` 集合标记「已生成待服务端确认」，抑制同 participantId 在滞后窗口内重复生成，服务端确认 `hostOfferReady=true` 或参与者离开后清除。

- **大厅 keep-alive 下加入成功后清理 `joinTarget`，抽屉可再次打开**（[LobbyBrowser.vue](src/components/online/LobbyBrowser.vue)）：加入成功后 `isInRoom` watch 立即切走分类，抽屉滑出动画被 keep-alive 冻结、`@close` 不触发，`joinTarget` 残留导致回到大厅后密码/整合包房间抽屉无法再打开。现在 `onDeactivated` 主动清理 `joinTarget`。

- **房间已满/已关闭时加入按钮禁用并提示原因**（[LobbyRoomCard.vue](src/components/online/LobbyRoomCard.vue)）：已满/已关闭房间的加入按钮此前禁用但无任何原因提示。现新增 `disabledReason` computed，三态互斥渲染「在房间中 / 房间已满或已关闭 / 可加入」。

- **接受/拒绝加入申请与创建房间按钮增加防重复提交守卫**（[PendingAnswerList.vue](src/components/online/PendingAnswerList.vue) / [RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) / [useRoomHostActions.ts](src/composables/useRoomHost/useRoomHostActions.ts) / [useRoomHost.ts](src/composables/useRoomHost.ts) / [onlineSession.ts](src/composables/online/onlineSession.ts) / [CreateRoomForm.vue](src/components/online/CreateRoomForm.vue) / [useCreateRoomForm.ts](src/composables/useCreateRoomForm.ts)）：接受/拒绝按钮连点会并行发出多个 `confirmParticipant`；创建房间「stun」阶段（`fetchStunServers` 不置 `roomLoading`）按钮仍可点导致并发建房。现 `handleConfirm` 用响应式 `confirming` Set 防重入并驱动按钮禁用，`handleCreateRoom` 用 `creating` 标志覆盖 stun 阶段防连点。

- **清理已离开参与者的连接状态残留键，避免无界累积**（[mesh-peer.ts](src/composables/useWebRTCMesh/mesh-peer.ts) / [useWebRTCMesh.ts](src/composables/useWebRTCMesh.ts) / [useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts)）：长会话中已关闭/已离开参与者的 `connectionStates`/`channelOpen`/`negotiating` 键此前永不删除，随加入/离开次数无界累积。现 `closeParticipant` 删除 `negotiating` 与 `channelOpen` 键，新增 `removeConnState` 供轮询在离开参与者清理时同步删除 `connectionStates`、`offerReadyLocal` 及各 restart 标记键。

- **轮询结果陈旧防护与冗余提示清理**（[useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts) / [useRoomHostActions.ts](src/composables/useRoomHost/useRoomHostActions.ts)）：`pollAnswers` 请求在途期间离开/关闭房间后，结果返回仍会执行自动放行逻辑产生对空 roomCode 的无效请求，现快照 `reqRoomCode` 并在处理后校验 role/roomCode 匹配，不匹配直接丢弃；`refreshBans` 移除每次刷新都弹的「封禁列表已刷新」冗余 toast。

- **修复上一轮联机修复引入的 5 处回归**（[LobbyRoomCard.vue](src/components/online/LobbyRoomCard.vue) / [onlineSession.ts](src/composables/online/onlineSession.ts) / [useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts) / [useRoomHostActions.ts](src/composables/useRoomHost/useRoomHostActions.ts) / [mesh-peer.ts](src/composables/useWebRTCMesh/mesh-peer.ts)）：
  - `LobbyRoomCard` 的 `inRoom` 与 `disabledReason` 两个独立 `v-if` 在房间中时同屏渲染双按钮，改为互斥链 `v-if/v-else-if/v-else`。
  - `guestLeaveAndCleanup` 主动退出时 `guestWebrtc.close()` 触发的 `closed` watch 会在 `await guestLeaveRoom` 期间误触发 `attemptGuestReconnect` 重新加入刚退出的房间，现置 `reconnecting=true` 阻断。
  - `offerReadyLocal` 清理只遍历当前参与者列表，参与者在 Offer 生成后、`hostOfferReady` 置位前离开会残留键无界累积，现在离开清理循环一并 `delete`。
  - `confirming` 为非响应式 Set 时按钮禁用反馈失效，改为 `ref<Set<string>>` 使 `:disabled` 生效。
  - `closeParticipant` 先删 `channelOpen` 键再被 `setConnState` 内部 `channelOpen.set(false)` 撤销，调整为先置 `closed` 再删键。

### Changed

- **移除前端 composable 与组件中的 `any` 类型，改用 `unknown` 配合类型守卫**（[useDebouncedSave.ts](src/composables/useDebouncedSave.ts) / [useFabricApi.ts](src/composables/useFabricApi.ts) / [usePackUpdate.ts](src/composables/usePackUpdate.ts) / [useModUpdate.ts](src/composables/useModUpdate.ts) / [useResourceDownload.ts](src/composables/useResourceDownload.ts) / [useDependencyConfirm.ts](src/composables/useResourceDownload/useDependencyConfirm.ts) / [useResourceModpackInstall.ts](src/composables/useResourceModpackInstall.ts) / [ResourceDetail.vue](src/components/community/ResourceDetail.vue) / [Community.vue](src/views/Community.vue)）：`catch (e: any)` 改为 `catch (e: unknown)` 并用 `e instanceof Error ? e.message : ...` 类型守卫提取错误信息；`useDebouncedSave` 实现返回类型 `: any` 改为 `SimpleReturn | PatchReturn` 联合类型（保留 overload 精确签名），符合「禁止使用 any、无法确定用 unknown 并配合类型守卫」的规范。

- **Frp 子项目 CI 对齐主仓库：同步后以 `workflow_call` 直调构建工作流，上传脚本重写为主仓库同格式**（[sync-upstream.yml](Frp/.github/workflows/sync-upstream.yml) / [goreleaser.yml](Frp/.github/workflows/goreleaser.yml) / [ci-upload.cjs](Frp/hack/ci-upload.cjs)）：
  - `sync-upstream.yml` 删除「Trigger goreleaser build workflow」步骤（原 `gh workflow run` 主动触发 + `sleep 10` 等 tag 服务端可见），sync job 输出 `tag` / `should_sync`，新增 `release` job 在 job 级 `uses: ./.github/workflows/goreleaser.yml` 以 `workflow_call` 直调构建（与主仓库 version-sync.yml → release.yml 模式一致），移除 `actions: write` 权限与 `GH_TOKEN`。
  - `goreleaser.yml` 新增 `workflow_call` 触发（input `tag`，required），checkout ref 与版本解析由 `github.event.inputs.tag` 改为 `inputs.tag`（兼容 workflow_dispatch）；上传步骤 env 由 `API_BASE_URL` 改为 `secrets.MOLAUNCH_ACTION_PUSH_SERVER`。
  - `hack/ci-upload.cjs` 重写为 [scripts/ci-upload.cjs](scripts/ci-upload.cjs) 同格式：apiServer 地址改读必填 `MOLAUNCH_ACTION_PUSH_SERVER`（原 `API_BASE_URL` + 硬编码默认地址）；预签名请求带 `sizes`，超过分片阈值走分片上传（与 updater 共用 `/v3/ci/complete-upload`）；新增 Cloudflare 回源错误码（520~527/530）与网络错误的指数退避重试（`s3PutWithRetry` / `apiPostWithRetry`）；保留 frp 特有 `component` 参数与 `/v3/ci/frp/presign-upload`、`/v3/ci/frp/releases` 端点。

## [0.3.6-rc2] - 2026-08-13

> 小更新，但是用联机服务的必更新。

### Fixed
- **修复已连接参与者被重复自动放行导致的 `setRemoteAnswer` 状态错误与连接被打断**（[mesh-peer.ts](src/composables/useWebRTCMesh/mesh-peer.ts) / [useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts)）：服务端 `find_pending_answers` 会永久返回已确认参与者已提交的 Answer（无消费语义），房主此前每 2s 轮询都对同一参与者重复 `setRemoteAnswer`——WebRTC 中一次协商完成后 `signalingState` 回到 `stable`，再次设置 Answer 必然抛 `Called in wrong state: stable`，且失败处理里 `closeParticipant` 还会把已建立的 P2P 连接关掉再靠 ICE restart 自愈，形成「每 2 秒闪断一次」的破坏循环。现 `setRemoteAnswer` 幂等化：PC 不存在或不在 `have-local-offer` 状态时返回 `false` 直接跳过，**不再抛错、不再误关已建连接**（ICE restart 后 PC 处于 `have-local-offer`，放行不受影响）；两处 autoAccept 按返回值区分「成功/跳过/失败」，仅真正协商失败才关闭残留 PC。

### Changed
- **信令轮询按连接状态自适应降频，大幅降低云端请求压力**（[useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts) / [useWebRTC.ts](src/composables/useWebRTC.ts)）：
  - 房主 Answer 轮询（`pollAnswers`）：原条件只要存在 `confirmed` 参与者就永远 2s 高频；现仅「有待确认申请（`answered`/`joined`）或已确认但尚未建连（DataChannel 未 open）的参与者」保持 2s，**全部连接建立后退避到 30s 慢速档**，仅低频感知 ICE restart 重答（断线时 channel 关闭自动回到 2s）；参与者轮询同样把新加入申请（`joined`）纳入活跃条件，新申请出现即回到 2s 及时展示。
  - 加入方等待房主 Offer 的轮询（`fetchOfferAndAnswer`）：间隔由 1000ms（与注释不符的回归）恢复为 2000ms，并在等待期间指数退避（2s→4s→8s→10s 封顶，总超时仍为 180s），避免授权前置下长等待时对云端每分钟 60 次请求。

## [0.3.6-rc1] - 2026-08-12

- 重做大厅加入房间交互并修复「提交 Answer 缺失导致永远连不上」（[LobbyBrowser.vue](src/components/online/LobbyBrowser.vue) / [LobbyJoinDialog.vue](src/components/online/LobbyJoinDialog.vue)，替换删除 [LobbyJoinConfirmDialog.vue](src/components/online/LobbyJoinConfirmDialog.vue)）：此前 `doJoin` 走 `showPrompt` 密码弹窗——点确定立即收起、`guestJoinRoom` 成功瞬间 Online.vue watch 直接把大厅切到房间详情（页面在抽屉关闭动画中「抽动」），且 `fetchOfferAndAnswer` 拿到房主 Offer 生成 Answer 后**从未 `submitAnswer`**（RoomManager / reconnectAsGuest 均提交，唯大厅入口漏掉），房主永远收不到 Answer、无法 confirm 建连，「加入中」卡死到最后超时，表现为「大厅加入房间很奇怪」。现重做为统一加入抽屉：有密码/整合包的房间先弹抽屉（密码输入 + 整合包校验内嵌）→ 点「加入房间」后抽屉保持打开显示加入中 → **失败时抽屉不收起、错误内联展示可直接改密码重试**（取消按钮此时禁用），成功才收起抽屉、`@close` 后组件卸载并由 role 变化切到房间详情；无密码无整合包房间仍直接加入。加入成功拆两段：`joinViaLobby` 只完成 `guestJoinRoom`（拿到 participantId 后抽屉收起），`continueJoin` 后台继续 TURN 拉取 → 等待房主 Offer → 生成 Answer 并 `submitAnswer`（[online-manager/room.ts](src/utils/api/online-manager/room.ts)），失败清理参与者与 RoomManager 对齐。

- 联机大厅离开后再回来自动刷新列表（[LobbyBrowser.vue](src/components/online/LobbyBrowser.vue)）：侧边栏分类切换走 keep-alive，从大厅切到设备/房间管理等分类再切回时组件不重新挂载、`onMounted` 不会再次执行，列表停留在旧数据。现补充 `onActivated` / `onDeactivated`：离开大厅超过 15s 后切回自动重新拉取列表（15s 内切回不刷新，避免快速切换闪烁），初始挂载仍由 `onMounted` 拉取、不受影响。

- 修复大厅入口加入有密码房间后自动重连「密码错误」（[LobbyBrowser.vue](src/components/online/LobbyBrowser.vue)）：`doJoin` 加入成功后未调用 `rememberJoinPassword`（与 RoomManager 加入路径不一致），`pendingJoinPassword` 保持空串，此后 P2P 断线自动重连 `peekJoinPassword()` 拿到空密码重新 join 密码房间直接失败（弹「重连错误：密码错误」），提权重启快照里的 `password` 同样为空。现加入成功即记录密码；重进其他房间时密码被新值覆盖，无残留旧密码问题。

- 修复联机加入方「一进房间就连接房主」（授权前置，[useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts) / [useRoomHostActions.ts](src/composables/useRoomHost/useRoomHostActions.ts) / [PendingAnswerList.vue](src/components/online/PendingAnswerList.vue) / [RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) / [useWebRTC.ts](src/composables/useWebRTC.ts)）：此前房主对 `status=joined`（尚未接受）的参与者**自动**生成并上传 SDP Offer，加入方轮询到 `ready` 立即创建 PeerConnection 进入「连接中」，而房主授权（confirm）发生在加入方提交 Answer 之后——房主侧尚未 setRemoteAnswer，ICE 永远无法配对，加入方持续 connecting 最终失败，触发 `pullTurnThenRecover → attemptGuestReconnect → room_leave` 自动退房死循环，房主端表现为「看不到任何申请、申请一直没人」。现将授权提前到协商之前：房主在「加入申请」抽屉（参与者驱动）点击接受 → `confirmParticipant(true)` → 状态 `confirmed` → 轮询才为 confirmed 参与者生成 Offer；加入方授权前 `ready=false` 只显示「等待房主接受」，不建连不自动退房，拿到 Offer 后提交 Answer，房主 `pollAnswers` 对已确认参与者自动放行 `setRemoteAnswer` 建连；`fetchOfferAndAnswer` 等待房主接受超时从 30s 放宽至 180s（[useWebRTC.ts](src/composables/useWebRTC.ts)）。配套 api-server：`submit_answer` 对已 confirmed 参与者保持 confirmed（授权在 Offer 前完成，Answer 到达不覆盖状态）、`list_answers` 返回已确认参与者已提交的 Answer 供自动放行（[offers.rs](api-server/src/services/signaling/offers.rs) / [signaling.rs](api-server/src/repositories/signaling.rs)）。

- 修复加入方自动退房后房间密码丢失（[useRoomReconnect.ts](src/composables/useRoomReconnect.ts)）：提权重启恢复的房间只走 `consumeReconnectPassword` 一次性消费（消费后 `reconnectPassword` 清空），从未调用 `rememberJoinPassword` 落盘 `pendingJoinPassword`，此后 P2P 断线自动重连 `peekJoinPassword()` 拿到空串，密码房间重新 join 空密码直接失败。现 `reconnectAsGuest` joinRoom 成功后 `rememberJoinPassword(password)`，自动重连链路始终持有密码。

- 修复大厅加入失败不清理参与者（[LobbyBrowser.vue](src/components/online/LobbyBrowser.vue)）：`doJoin` 失败（含等待房主接受超时）此前只 toast 不清理，服务端参与者残留导致大厅人数虚高、重进房间状态异常；现与 RoomManager 加入路径一致——失败时 `guestLeaveRoom` + `resetRoomState` + 关闭 PC。

- 修复云端更新日志「作者的话」note 解析（[updateLog.ts](src/utils/updateLog.ts)）：服务端 release_notes 的 note 行为 `- note: 内容` markdown 列表项格式，`extractNoteLines` / `stripNoteLines` 原正则仅匹配行首 `note:` 导致提取失败，检查更新抽屉回退展示构建时旧 note；现兼容 `[-*+]?` 列表前缀，并清理 note 文本行尾的 commit 链接（支持 `(https://...)` 括号包裹形式）。

- 修复静默下载期间手动「检查更新」无响应（[check.ts](src/utils/updater/check.ts) / [state.ts](src/utils/updater/state.ts)）：Windows 后台预下载与手动检查共享 `updaterFlags.checking` 防并发标志，静默下载流程（`silentCheckAndDownload` 两步：检查 + 下载新 exe 到 `%APPDATA%/.Molaunch/last.exe`）期间 `checking` 被长时占用，手动 `checkForUpdate` 命中 `if (checking) return` 被静默忽略，表现为「检查更新」按钮点击无反应，下载完成才恢复；现拆分为独立标志 `checking`（手动检查）与 `silentChecking`（静默检查 + 后台下载），两者互不阻塞；另在更新弹窗底部，后台预下载期间「立即更新」按钮禁用置灰并 hover 提示「后台正在预下载新版本，无需手动更新，退出应用后自动安装」（[state.ts](src/utils/updater/state.ts) 新增响应式 `silentDownloading` 状态 + [UpdateDialog.vue](src/components/about/UpdateDialog.vue) 复用 Tooltip 组件），避免与后台预下载并发下载同一版本。

## [0.3.5] - 2026-08-12

- 修复进入主页时账号 IPC 重复调用（[useAccountCards.ts](src/composables/useAccountCards.ts)）：`get_ms_accounts` / `get_offline_accounts` / `get_authlib_accounts` 原先各被调用两次——账号卡片组件挂载时（AccountSelector onMounted）拉一遍、App.vue 启动流程的 `restoreSession` 内 `Promise.all` 又拉一遍，产生成堆 `[Startup][IPC]` 日志。现账号列表统一由 `restoreSession` 加载，卡片组件不再自行拉取（数据为 store 响应式，加载完成后自动渲染）；登录成功 / 删除 / 切换账号后的刷新仍由 auth store 内部显式触发。

- 下载面板与返回顶部按钮视觉完全统一（[DownloadPanel.vue](src/components/common/DownloadPanel.vue)）：下载按钮缩至与 BackToTop 一致的 44px 圆钮、图标改用 solid 风格白图标、采用相同阴影与 hover/active 动效（hover 上移 2px + 阴影增强、按下缩小）；进度环贴边外圈（r=20 / dasharray 125.66）；BackToTop 消失时下载按钮从避让位平滑滑回贴底（`bottom` 过渡），两个浮标同时出现不再突兀。Tailwind 浮动类收敛为 scoped CSS。

- 清理前端死代码（[Trigger.vue](src/components/common/Trigger.vue) / [InstalledList.vue](src/components/version/InstalledList.vue) / [DownloadProgressOverlay.vue](src/components/community/resource-detail/DownloadProgressOverlay.vue) / [useCommunityDownload.ts](src/composables/useCommunityDownload.ts) / [deeplink.ts](src/utils/deeplink.ts) / [click-outside.ts](src/utils/click-outside.ts)）：全仓无引用的孤儿组件（预留未接线的 popover 触发器、被 VersionSection 取代的旧版已安装列表、规划未接入的下载进度浮层）、连带孤儿 composable（useCommunityDownload）、前端无消费方的深链接工具均删除；`click-outside.ts` 移除仅被孤儿组件调用的 `onClickOutside`（保留 Drawer 使用的 `onEscape`）；`useExternalDownload` 子目录下已被根目录版本取代的旧实现切片一并清理。

- 下载面板显隐改为后端 emit 驱动（[core.rs](src-tauri/src/minecraft/download/manager/core.rs) / [config.rs](src-tauri/src/minecraft/download/config.rs) / [session.rs](src-tauri/src/minecraft/download/session.rs) / [app.rs](src-tauri/src/state/app.rs) / [useDownloadStream.ts](src/composables/useDownloadStream.ts) / [version.ts](src/stores/version.ts) / [DownloadPanel.vue](src/components/common/DownloadPanel.vue)）：右下角下载面板不再由前端调用方主动 `startDownload` 控制，`DownloadManager.download_batch` 开始/结束时经 `download-panel-state` 事件（`{visible}`）通知前端显示/隐藏，多个并发批次通过 `AppState.panel_active_count` 共享计数器协调（首个开始显示、最后结束隐藏，避免并发时面板提前消失）；`DownloadManagerConfig` / `DownloadManager` 增加 `silent` 字段、`DownloadSession` 的 `start_grouped` / `start_grouped_with_manager` / `attach` 增加 `silent` 参数，供后端调整——下载 Java（[java.rs](src-tauri/src/commands/java.rs) / [java_check.rs](src-tauri/src/minecraft/launch/pipeline/java_check.rs)）、更新程序（[install_windows.rs](src-tauri/src/commands/system/updater/install_windows.rs)）、启动时文件补全（[validate.rs](src-tauri/src/minecraft/launch/pipeline/validate.rs)）、frpc 补全（[system_default.rs](src-tauri/src/commands/frp/binary/system_default.rs)）均静默下载，不弹出面板；`DownloadManagerConfig` 增加 `panel_counter` 字段并在 `from_state` / `from_state_for_meta` 中自动填充，加载器安装（Fabric/Forge/NeoForge/LiteLoader/Fabric API）与外部下载等 `from_config` 路径也能接入共享计数器，避免并发下载时面板提前消失、整合包安装加载器阶段面板闪烁；frpc 下载成功路径补 `mark_complete`，修复 `download_state.is_active` 残留导致重启后 `isDownloading` 恢复误判弹出已结束的下载面板。

- 简约化返回顶部按钮（[BackToTop.vue](src/components/common/BackToTop.vue)）：移除渐变背景、涟漪动画、外圈光晕与弹性动画，改为与右下角下载面板同款纯色圆钮（primary-600 纯色 + 白色箭头 + 简洁阴影 + hover 加深），两个浮标同时出现时视觉统一。
- Windows 更新下载复用通用 DownloadManager（[install_windows.rs](src-tauri/src/commands/system/updater/install_windows.rs) / [api.rs](src-tauri/src/commands/system/updater/api.rs) / [mod.rs](src-tauri/src/commands/system/updater/mod.rs) / [updater.rs](src-tauri/src/commands/system/manager/updater.rs)）：此前 Windows 更新（弹窗下载 + 后台预下载到 appdata）为两处自写单线程 HTTP 下载（`bytes_stream` / `bytes()`），现统一改为复用 Minecraft 下载体系的多线程 `DownloadManager`——文件分片 + 失败重试 + 用户下载限速配置（`DownloadManagerConfig::from_state`），进度经 `GlobalProgress` 回调转发为 `update-download-progress` 事件（DownloadManager 每 300ms 回调一次，等价替代原 256KB 节流），单流路径自动从响应头回填 total、收尾事件兜底前端切换「安装中」；`download_and_install_update` / `download_update_to_appdata` 两个 action 增加 AppState 参数以读取下载配置，`PROGRESS_THROTTLE_BYTES` 常量仅保留给官方 plugin 路径（macOS/Linux）。其余自写 `.bytes()` 点均为合理保留：FRP 厂商 API JSON 响应（≤1MB 非文件下载）、皮肤/披风 PNG 内存小图（DownloadManager 无内存下载 API）、后台图片缓存小图。

- 修复 Windows 更新弹窗进度停滞与「完整日志」无反馈（[install_windows.rs](src-tauri/src/commands/system/updater/install_windows.rs) / [install_unix.rs](src-tauri/src/commands/system/updater/install_unix.rs) / [mod.rs](src-tauri/src/commands/system/updater/mod.rs) / [UpdateDialog.vue](src/components/about/UpdateDialog.vue) / [UpdateLogDialog.vue](src/components/about/UpdateLogDialog.vue)）：Windows 更新下载原为一次性 `response.bytes()` 全程不推进度、下完立即 `app.exit(0)`，前端进度条只能停在 0% 且无感退出。现改为流式下载，按节流阈值（每累计 256KB）经 `update-download-progress` 事件实时推送 `downloaded/total`（进度常量 `PROGRESS_EVENT` / `PROGRESS_THROTTLE_BYTES` 提升为 updater 模块公共常量，Windows / macOS / Linux 共用），收满 total 时前端切换「安装中」，启动 updater.exe 前停留 1 秒让用户感知；启动后更新日志抽屉的「完整更新日志」按钮改为先 toast 提示、1s 后再打开 GitHub Releases，避免无任何点击反馈。

- 修复更新包签名校验失败（[verify.rs](src-tauri/updater/src/verify.rs) / [verify_test.rs](src-tauri/updater/src/verify_test.rs)）：tauri `signer sign` 生成的 `.sig` 文件内容为「4 行标准 minisign 文本的 base64 编码」（与 tauri-plugin-updater 约定一致），updater 此前直接按 4 行文本解析导致 `Invalid encoding in minisign data`。现签名解析先检测文本格式，非文本则先 base64 解码再解析，两种格式均兼容，并补单元测试。

## [0.3.5-rc9] - 2026-08-12

- apiServer 地址移入 CI Secret（[ci-upload.cjs](scripts/ci-upload.cjs) / [release.yml](.github/workflows/release.yml)）：上传脚本不再硬编码 apiServer 地址（原默认 `https://api.molaunch.moiu.cn`），改为从 `MOLAUNCH_ACTION_PUSH_SERVER` 环境变量读取并校验必填；release 工作流两处上传步骤注入 `secrets.MOLAUNCH_ACTION_PUSH_SERVER`。

- CI 上传脚本 MoSign API 请求增加 Cloudflare 回源错误重试（[ci-upload.cjs](scripts/ci-upload.cjs)）：预签名上传 URL、完成分片、注册版本三个接口此前直接裸请求，遇 HTTP 520~530（如 522）即失败退出；现抽取公共 `apiPostWithRetry`（复用与 S3 上传同一套 RETRYABLE_STATUS / MAX_RETRIES 退避策略，每次重试重新签名），三个接口统一接入，消除重复的签名+请求样板代码。

- 修复厂商 frpc 命令模式 token 拼接错误（[spawn.rs](src-tauri/src/commands/frp/process/spawn.rs)）：模板 `-t 17062:{token}` 原被拆成 `-t 17062:` + 独立参数 token，frpc 将 token 当作未知子命令导致启动失败；现按词内联替换 `{token}`，token 与端口拼为同一参数（与 args 模式行为一致），并补单元测试。

- 检查更新抽屉同步「作者的话」note 高亮（[UpdateDialog.vue](src/components/about/UpdateDialog.vue) / [updateLog.ts](src/utils/updateLog.ts)）：检查更新弹窗的更新日志此前直接按时间线渲染服务端 release_notes，未提取 `note:` commit；现与启动时更新版本弹窗一致——优先从服务端 notes 解析 `note:` 前缀行（新增 `extractNoteLines` / `stripNoteLines` 工具），提取不到则回退构建时 git note（`getChangelogNotes`），以「作者的话」高亮块展示在时间线上方，并从时间线中剔除原 note 行避免重复。

## [0.3.5-rc8] - 2026-08-12

- `/v1` 业务请求自动应对 PoW challenge（[request.rs](src-tauri/src/minecraft/online/client/request.rs) / [auth.rs](src-tauri/src/minecraft/online/client/auth.rs)）：`call_v1` 与 auth 接口共用新增的 `send_with_pow_retry`，收到 `401 + code=1007`（pow_challenge_required）且 challenge.path 匹配时自动求解并携带 `{header_name}: {challenge_id}:{nonce}` 头重试一次，登录、注册、刷新及 TURN 拉取等接口无需各自处理 1007 挑战（此前 `GET /v1/signaling/rooms/{code}/turn` 会残留 401 WARN 日志且拉取失败降级兜底）。

- 系统 TURN 改为进入房间即拉取、建房瞬间不拉（[useRoomHost.ts](src/composables/useRoomHost.ts) / [useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts) / [onlineSession.ts](src/composables/online/onlineSession.ts) / [RoomManager.vue](src/components/online/RoomManager.vue) / [LobbyBrowser.vue](src/components/online/LobbyBrowser.vue) / [roomRefreshActions.ts](src/stores/online/roomRefreshActions.ts) / [roomState.ts](src/stores/online/roomState.ts) / [mesh-peer.ts](src/composables/useWebRTCMesh/mesh-peer.ts) / [useWebRTC.ts](src/composables/useWebRTC.ts) / [webrtc-helpers.ts](src/utils/online/webrtc-helpers.ts)）：加入方 join 后、房主首个参与者生成 Offer 时即拉取系统 TURN 并合并进 ICE 服务器，使首轮协商就带 relay candidate——P2P 直连不受影响（ICE 优先 host/srflx），打洞失败时中继立即可用，无需等 failed 后 ICE restart 的漫长恢复；建房瞬间（尚无参与者）不拉取，避免白费 `/turn` 请求与 PoW 计算。同一房间仅拉取一次（`systemTurnServers` / `systemTurnLoaded` 缓存，房间切换清空），首轮拉取失败降级 STUN+自定义 TURN，P2P 失败恢复路径幂等重试。另新增 `toRtcIceServers` 公共转换（createPeerConnection / setConfiguration 两处共用）。

- Java 运行时下载对接通用下载器（[pipeline.rs](src-tauri/src/minecraft/java/download/pipeline.rs) / [files.rs](src-tauri/src/minecraft/java/download/files.rs) / [verify.rs](src-tauri/src/minecraft/java/download/verify.rs) / [java_check.rs](src-tauri/src/minecraft/launch/pipeline/java_check.rs) / [java.rs](src-tauri/src/commands/java.rs)）：自动下载 Java（无可用 Java 时「获取 Java」阶段触发）原为逐文件串行下载，java runtime 大文件（jvm.dll 等共上百 MB）单连接单流，速度慢。现阶段 4 改为复用通用 `DownloadManager`（`download_batch`）——文件级并发（`max_threads`）+ 单文件分片（`chunk_count`，自动探测 Range 不支持则回退单流）+ FileChecker 尺寸/SHA1 校验 + 限速。进度事件 `java-download-progress` 格式不变（`progress::emit` 桥接 `GlobalProgress` 字节级刷新，前端 Java 下载进度条无需改动）；保留原行为：路径穿越校验、已存在文件跳过（断点续传）、任意文件失败清理整个 runtime 目录、Unix 可执行权限设置、`verify_downloaded_java` 运行验证。并发/分片/限速参数来自用户下载设置（`LaunchConfig` / `AppConfig.download`）。

- 修复 Java 下载进度百分比滞后（[JavaDownloadBar.vue](src/views/version-settings/JavaDownloadBar.vue) / [LaunchLog.vue](src/components/home/LaunchLog.vue)）：下载百分比原优先按文件数比例（`current/total`）计算，Java runtime 大文件（jvm.dll / server 等）先被并发下载时文件数增长慢，出现「已下载过半但显示 17%」的滞后。现改为字节优先（`bytes_downloaded / bytes_total`，Rust 侧为实时字节），字节未知时回退文件数比例；启动流程右侧 Java 下载进度条的文字百分比与条宽统一使用同一字节比例（原条宽按字节、文字按文件数，两者不一致）。

- 修复联机大厅加入含整合包房间确认抽屉直接消失后突然弹出密码抽屉（[LobbyJoinConfirmDialog.vue](src/components/online/LobbyJoinConfirmDialog.vue) / [LobbyBrowser.vue](src/components/online/LobbyBrowser.vue)）：原实现点击「加入房间」瞬间卸载确认抽屉（关闭动画被截断），随后才弹出「输入密码」抽屉，视觉上「内容消失后右边突然蹦出抽屉」。现改为先播完确认抽屉的关闭动画，`Drawer` 关闭动画结束后（`@close`）再卸载组件，并记录待执行加入动作（延续原「有整合包先确认、有密码再输入」流程），两个抽屉动画平滑衔接。

- 种子地图「从存档加载」弹窗改为项目通用抽屉形式（[LoadSaveDrawer.vue](src/views/tools/data/LoadSaveDrawer.vue) / [SeedMap.vue](src/views/tools/data/SeedMap.vue)）：原 [LoadSaveModal.vue](src/views/tools/data/LoadSaveModal.vue) 为自定义居中 Modal，现重写为复用 `components/common/Drawer.vue` 的右侧抽屉（`placement=right` + `render-in-place` + `popup-container=#app-content`），交互与房间工具抽屉等保持一致，删除旧 Modal 组件。

- 修复「本次更新日志」弹窗首次运行死锁（[updateLog.ts](src/utils/updateLog.ts)）：原逻辑在 localStorage 无记录（全新安装/从未弹过窗）时直接跳过且不写入，导致 key 永不落盘、后续升级永远检测不到版本变化。现改为首次运行仅记录当前版本（不弹窗），升级时才能正常对比弹窗。

- 修复种子地图版本 ≤1.12 时结构筛选块全部消失（[config.ts](src/views/tools/data/useSeedMap/config.ts) / [useSeedMap.ts](src/views/tools/data/useSeedMap.ts) / [LoadSaveDrawer.vue](src/views/tools/data/LoadSaveDrawer.vue)）：`SEEDMAP_MC_VERSIONS` 的 value 原为紧凑编号（1.7→4、1.12→9、1.13→10、26.2→28），与 cubiomes 真实 `MCVersion` 枚举（`MC_1_7=10`、`MC_1_12=15`、`MC_1_13=16`、`MC_26_2=34`）错位，`getStructuresForVersion` 用紧凑编号与 `javaSinceValue`（真实枚举）比较导致 1.12 及以下所有结构（Village=10 等）被过滤。现 value 全部改为真实 cubiomes 枚举，前端筛选与 WASM 地图渲染/结构查找统一基于正确版本枚举，默认版本与「从存档加载」兜底值同步修正为 34。

- FRP 官方公共服务器创建隧道移除分配接口调用（[public-server.ts](src/utils/api/frp-manager/public-server.ts) / [usePublicServers.ts](src/composables/usePublicServers.ts) / [frp.rs](src-tauri/src/minecraft/online/frp.rs) / [public_server_actions.rs](src-tauri/src/commands/frp/manager/public_server_actions.rs)）：api-server 已移除 `POST /v1/frp/allocate|release|keepalive` 下发链路，`GET /v1/frp/servers` 列表直接返回完整连接信息（公共 token / 地址 / 端口 / TLS）。前端选择公共服务器后从列表项直接回填表单（远程端口客户端随机生成），不再请求云端分配 token 与端口；Rust 侧同步删除 `allocate_public_server` / `release_public_server` / `keepalive_public_server` 三个 IPC action、`PublicFrpServer` 精简为列表接口实际返回的 7 个字段（移除 `serverType` / `onlineUsers` / `maxUsers` / `loadPercent` / `allocatable`），避免反序列化因缺失字段失败。

## [0.3.5-rc7] - 2026-08-11

- 修复检查更新对预发布版本比较失效（[check.rs](src-tauri/src/commands/system/updater/check.rs) / [check_test.rs](src-tauri/src/commands/system/updater/check_test.rs)）：原 `parse_semver` 用 `split(['.', '-'])` 拆版本并丢弃 pre-release 段，导致 `0.3.5-rc7` 与 `0.3.5-rc6` 被解析为相同版本，rc/beta/alpha 迭代时「检查更新」始终返回无更新。现按语义化版本规则比较——主版本号相同则比较 pre-release 段（纯数字按数值、`rc`/`beta` 前缀按前缀+数字尾比较），正式版高于预发布版，rc9→rc10 两位数字后缀也按数值正确识别。

- TURN 中转闭环（[api-server](api-server) [signaling.rs](api-server/src/models/signaling.rs) / [rooms.rs](api-server/src/services/signaling/rooms.rs) / [offers.rs](api-server/src/services/signaling/offers.rs) + [roomRefreshActions.ts](src/stores/online/roomRefreshActions.ts) / [onlineSession.ts](src/composables/online/onlineSession.ts) / [useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts) / [mesh-peer.ts](src/composables/useWebRTCMesh/mesh-peer.ts)）：①修复多人房间 TURN 中转不可用——凭证 HMAC 绑定调用者 IP+设备，房主广播的凭证对参与者无效，加入方改为加入时用自己的凭证自拉 `/turn`（`guestPullTurnServers`），与房主广播（无 regionCode）合并去重后建链；②`IceServerEntry` 服务端下发节点 `name`/`region`/`regionCode`，新增 [transport-info.ts](src/utils/online/transport-info.ts) `detectTransportInfo` 通过 `getStats` 选中 candidate-pair 的 localCandidate 类型判定实际走直连还是中继，联机面板常驻状态行（[ConnectionTransportStatus.vue](src/components/online/ConnectionTransportStatus.vue)）显示「P2P 直连」或「TURN 中转 + 节点国旗/名称」，国旗用 country-flag-icons 渲染真实 SVG（regionCode 经 hasFlag 校验）；③P2P 断线自动 ICE restart——房主侧监听各参与者 connectionState，failed/长时间 disconnected 自动 `restartIce()` 重发新 Offer（冷却+限次，重启前已确认的参与者重答自动放行不再弹确认框），加入方侧启动新 Offer 监控（断线快速轮询/正常慢速轮询）发现 ice-ufrag 变化即轻量重答，超时未恢复回退原有全量重建；④服务端 `upload_participant_offer`/`submit_answer` 状态校验放宽到 joined/answered/confirmed 以支持重协商；⑤TURN 未启用时仍回退云端 `stun_servers`（`resolveIceServers` 兜底不变）。

- 云端 `ice_servers` 列改为仅存 TURN（[roomCrudActions.ts](src/stores/online/roomCrudActions.ts) + [rooms.rs](api-server/src/services/signaling/rooms.rs)）：创建房间时 `stun_servers` 列存纯 STUN URL、`ice_servers` 列只存用户自定义 TURN（未配置则存空数组），不再冗余存储 STUN 转换条目；服务端读取侧 `parse_ice_servers` 对空数组回退 `stun_servers` 转换，旧房间与旧客户端兼容不变。

- 房间详情新增「工具」抽屉（[RoomToolsDrawer.vue](src/components/online/RoomToolsDrawer.vue) / [RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) / [RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue)）：房主侧与「加入申请/参与者/封禁」同行、加入方侧与「退出房间」同行，点击弹出抽屉提供三个工具——①检查 MC 服务：复用工具中心 `serverPing`（SLP 1.7+），房主测本机 127.0.0.1、加入方测房主虚拟 IP（走 TUN 桥接），展示 MOTD/在线人数/版本/延迟；②检查网络连通性（仅加入方可见）：复用 `tcpCheck` 对房主虚拟 IP:MC 端口做 TCP 握手，验证 P2P + 虚拟网卡链路；③端口自动检测（[lan_probe.rs](src-tauri/src/commands/online/manager/lan_probe.rs) 新增 `lan_port_probe` action + [lan_probe_test.rs](src-tauri/src/commands/online/manager/lan_probe_test.rs)）：绑定 UDP 4445 并加入多播组 224.0.2.60，监听 MC 局域网发现广播解析 `[AD]port[/AD]`（与多人游戏发现房间同源）——房主可得服务器实际开放端口，加入方可得本地伪装代理端口（即多人游戏界面显示的端口）。

- MC 多人游戏界面直接发现房主房间（局域网伪装，[lan_fake.rs](src-tauri/src/commands/online/manager/lan_fake.rs) / [onlineSession.ts](src/composables/online/onlineSession.ts) / [RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue)）：加入方进入房间且 TUN 就绪后，本地起 TCP 转发代理（端口自动分配）并按 MC 1.12+ 局域网发现协议周期广播 `[MOTD]...[/MOTD][AD]port[/AD]`，本机 MC 客户端在「多人游戏」界面直接看到房主房间，点击进入经代理转发到房主虚拟 IP:MC 端口（走 TUN 桥接），无需手动输入地址；退出房间/停 TUN 自动停止伪装。
- 房主 TUN 数据包按目标虚拟 IP 定向转发（[tunRouting.ts](src/utils/online/tunRouting.ts) / [onlineSession.ts](src/composables/online/onlineSession.ts)）：房主将 TUN 读出的 IP 包解析目标地址并映射到对应参与者的 DataChannel 单播，替代原无差别广播，房主上行带宽由「每包 ×N」降为「每包 ×1」，消除人数上升时的广播冗余；无法识别目标（非 IPv4/未知地址）时自动回退广播，兼容性不变。
- 端口捕获测试迁移至独立测试文件（[scheduler_test.rs](src-tauri/src/minecraft/launch/watcher/scheduler_test.rs)）：`scheduler.rs` 内嵌 `#[cfg(test)] mod tests` 移除，按项目规范迁移为同目录 `scheduler_test.rs`（`#[path]` 引入），`parse_lan_port` 正则可测性不变。
- 修复 TUN 提权重启时误报「虚拟网卡启动失败」（[useVirtualLan.ts](src/composables/useVirtualLan.ts)）：用户确认以管理员权限重启后，不再向调用方抛原始 `TUN_PERMISSION_DENIED` 权限错误（重启前 500ms 内会误弹失败 toast），改为提示「正在以管理员权限重启，重启后自动恢复虚拟网卡」；UAC 被拒或重启失败时仍保留原错误提示。
- MC 局域网端口自动捕获重构（[scheduler.rs](src-tauri/src/minecraft/launch/watcher/scheduler.rs) / [log_reader.rs](src-tauri/src/minecraft/launch/watcher/log_reader.rs)）：启动器已知游戏 Java 进程 PID，新增按 PID 轮询该进程监听的非回环 TCP 端口（netstat2，连续两次确认后上报），MC 开放局域网即自动识别端口，不再依赖日志格式与 stdout 可用性；日志正则修正覆盖各版本实测格式（`Started on 4053` / `Local game hosted on port 49152` / `Published server on ip:port`），`logs/latest.log` 兜底保留；双信号共用去重上报入口，避免重复 emit。
- 房主 MC 端口支持手动指定（[HostMcPortEditor.vue](src/components/online/HostMcPortEditor.vue) / [HostRoomInfoCard.vue](src/components/online/HostRoomInfoCard.vue) / [useRoomHost.ts](src/composables/useRoomHost.ts) / [onlineSession.ts](src/composables/online/onlineSession.ts)）：自动捕获不可靠时房主可手动编辑端口，手动值为最高可信度——设置后自动捕获结果不再覆盖（`hostMcPortManual` 标记），立即经 `HOST_MC_PORT` 控制消息广播给所有参与者；可一键「恢复自动」。
- 联机申请/成员操作按钮由图标改为文字（[PendingAnswerList.vue](src/components/online/PendingAnswerList.vue) / [ParticipantList.vue](src/components/online/ParticipantList.vue) / [RoomHostPanel.vue](src/components/online/RoomHostPanel.vue)）：加入申请「接受/拒绝」、参与者「踢出」、房主「封禁」按钮不再使用 heroicons 图标，直接显示文字标签，语义更明确。

- P2P 联机轮询改为自适应退避（[useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts)）：房主参与者/Answer 轮询在稳态下（无待生成 Offer 的参与者、无待确认申请）由 2s 退避到 10s，活跃时自动恢复 2s，空闲时云端压力降低约 5 倍；`setInterval` 改为 `setTimeout` 链式调度天然防重入，`stopTimers` 后不会因进行中的请求重新拉起定时器；轮询参与者发现新加入者时联动刷新一次 Answer 申请，申请呈现不受退避影响。
- 联机面板提示统一复用 `AlertV2` 组件（[RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) / [WhitelistEditor.vue](src/components/online/WhitelistEditor.vue) / [RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue)）：接近人数上限预警、P2P 已联通指引、白名单为空警告、连接失败提示替换原先手写 CSS 提示块，样式与既有提示保持一致。
- 启动按钮图标统一为 heroicons（[LaunchPanel.vue](src/components/home/LaunchPanel.vue)）：播放/停止/加载中三个手写内联 SVG 替换为 `PlayIcon` / `StopIcon` / `ArrowPathIcon`，与全站图标体系一致。

- 修复 P2P 联机加入方 WebRTC 状态异常显示 closed（[useWebRTC.ts](src/composables/useWebRTC.ts)）：`close()` 原先无条件将连接状态置为 closed，即使从未建立过 PeerConnection；应用启动时全局会话以空角色调用 `guestWebrtc.close()` 会让状态从启动起就显示 closed。现仅在确实存在连接时才置 closed，且新建 PeerConnection 时重置状态为 `new`，避免复用实例残留上一次会话的 closed。
- P2P 联机握手提速：房主参与者/Answer 轮询间隔由 5s 收紧到 2s（[useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts)），加入方 SDP Offer 轮询间隔由 2s 收紧到 1s（[useWebRTC.ts](src/composables/useWebRTC.ts)），从加入房间到房主看到「加入申请」的整体等待明显缩短。
- 修复加入方连接建立后房主虚拟 IP 一直显示「等待房主广播」（[protocol.ts](src/utils/online/protocol.ts) / [mesh-peer.ts](src/composables/useWebRTCMesh/mesh-peer.ts) / [useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts) / [onlineSession.ts](src/composables/online/onlineSession.ts) / [protocol.rs](src-tauri/src/minecraft/online/protocol.rs)）：新增 `HOST_VIRTUAL_IP` 控制消息（subtype 0x06），房主在参与者的 DataChannel 建立后向该参与者广播自己的虚拟 IP，加入方收到后回填并显示，无需再干等。
- 加入方面板 WebRTC 状态徽章改为中文标签（[RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue)）：`new` 显示「等待房主接受」、`connecting` 显示「连接中…」等，不再展示英文原始状态。

- 修复 GitHub Issue 模板不生效（`.github/ISSUE_TEMPLATE/bug_report.md` / `feature_request.md` / `question.md`）：简单 Markdown 模板 front matter 的描述字段应为 `about`（`description` 仅适用于 Issue Forms 结构化表单），将三个模板的 `description:` 修正为 `about:`，模板现可正常显示于新建 Issue 的选择页。

- 提交规范更新：commit message 默认不再携带 `!c` 标记（`!c` 仅作为可选的 CI 跳过标记，需要跳过本次推送触发的构建时才附加）；同步更新 `AI_AGENT_GUIDELINES.md` / `DEVELOPMENT_GUIDELINES.md` / `CONTRIBUTING.md` / `DEVELOPMENT_BLUEPRINT.md` / `.github/PULL_REQUEST_TEMPLATE.md` 中的格式示例与说明。

- 修复版本设置资源包/光影页不可用状态仍显示顶部工具栏（[PackTab.vue](src/views/version-settings/PackTab.vue)）：对齐 mods 列表处理，未安装所需程序（如光影无 OptiFine/Iris 加载器）时 `not-modable` 空状态整体覆盖页面，顶部「从文件安装/打开文件夹/刷新」及筛选、搜索等操作功能一并隐藏，不再可点击。

## [0.3.5-rc6] - 2026-08-11

> 此版本仅修复小部分功能，可选更新。

- 更新检查接口新增 `channel` 参数（[api_paths.rs](src-tauri/src/api_paths.rs) / [check.rs](src-tauri/src/commands/system/updater/check.rs)）：按当前版本后缀自动推导分支（[client_type.rs](src-tauri/src/utils/client_type.rs) 新增 `channel_name`，rc/beta 归 beta、alpha/dev 归 alpha），预发布版本可正确查询对应分支更新。
- 全部 HTTP 请求统一接入中间人检测（[http.rs](src-tauri/src/http.rs) 新增 `is_tls_cert_error`/`request_error_msg`）：更新检查、FRP、皮肤、联机、AI、Microsoft 登录、authlib 皮肤站、CurseForge/Modrinth、游戏下载、URL 文件名探测等请求在 TLS 证书校验失败时统一返回「检测到中间人攻击，已自动断开链接」，不再暴露原始证书错误。
- 开发者模式「实验性功能」新增「更新检测分支」切换（[ExperimentalTab.vue](src/views/settings/developer/ExperimentalTab.vue) / [developer.rs](src-tauri/src/commands/system/developer.rs) 新增 `get_update_branch`/`set_update_branch`）：可手动选择更新检查使用的发布分支（跟随版本/stable/beta/alpha），覆盖 [check.rs](src-tauri/src/commands/system/updater/check.rs) 请求更新清单时的 `channel` 参数，用于跨分支更新检测调试；撤销开发者模式时自动恢复「跟随版本」。

## [0.3.5-rc5] - 2026-08-10

- Rust 后端全库执行 `cargo fmt` 统一格式，并修复 clippy `unnecessary_sort_by` 告警（[pack_common.rs](src-tauri/src/commands/version/pack_common.rs) 按小写文件名排序改用 `sort_by_key`）。
- 构建配置（[vite.config.ts](vite.config.ts)）：新增 `vendor-markdown` chunk 分割（marked + dompurify，AI 聊天与更新日志共用）；精简过时注释并删除已不存在的 arco-design 说明，chunk 命名清理改为链式调用。
- 联机页面创建房间表单、加入房间表单、房主/加入方面板（房间详情）顶部新增两条 `AlertV2` 提示：「P2P联机对房主的网络质量要求较高，如遇连接不上可尝试更换房主」与「如遇到违法违规房间，请及时向我们举报」（[CreateRoomForm.vue](src/components/online/CreateRoomForm.vue) / [RoomManager.vue](src/components/online/RoomManager.vue) / [RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) / [RoomGuestPanel.vue](src/components/online/RoomGuestPanel.vue)）。
- 实验性 AI 聊天每条 AI 回复底部左侧新增「由AI生成的内容，注意甄别」声明（[ChatMessageItem.vue](src/components/experimental/ChatMessageItem.vue)），与右侧 token 统计 / 模型信息同行展示，正文生成期间即显示。
- 禁止启动画面（[splash.js](public/splash.js)）响应右键菜单与快捷键：拦截 F1~F12 及所有 Ctrl/Cmd/Alt 组合键（刷新、DevTools、关窗、复制粘贴等），并阻止拖拽，与主窗口 `useDevToolsGuard` 同一防护思路；splash 无输入框，故未保留编辑键。
- 新增版本设置「检测并重装加载器」（[repair_loader.rs](src-tauri/src/commands/version/list/repair_loader.rs) + [RepairLoaderDrawer.vue](src/views/version-settings/repair-loader/RepairLoaderDrawer.vue) + [useVersionOverviewActions.ts](src/composables/useVersionOverviewActions.ts) + [OverviewTab.vue](src/views/version-settings/OverviewTab.vue)）：检测 Forge/Fabric/LiteLoader 是否损坏（版本 JSON 中存在加载器库但对应库文件缺失或为空），损坏时复用 `install_single_loader` 安装链自动重装，并将新生成的加载器 JSON 合并回当前版本 JSON（minecraftArguments/arguments token 去重、libraries 同名替换、其余字段加载器覆盖、去掉 inheritsFrom），随后调用 `fix_version_files` 幂等补全并清理临时加载器版本目录；Quilt/OptiFine 检测到损坏但暂无重装链时提示不支持自动重装。交互为抽屉式（`render-in-place` 挂载到 `#app-content`，与项目其他弹层一致）：打开后经新增 `detect_loader_damage` IPC 独立扫描，无损坏显示「当前文件无损坏」+ 图标；损坏时先询问用户是否重新安装（展示加载器类型/版本），确认后才执行重装；重装阶段监听后端 `repair-loader-progress` 事件按 `installing → merging → done/error` 推送进度（installing 阶段轮询转发现有安装伪进度 ticker），修复完成整体模糊并居中提示「检查到文件有损坏，已完成修复」。新增同目录测试 [repair_loader_test.rs](src-tauri/src/commands/version/list/repair_loader_test.rs)（11 例）。
- 修复整合包详情弹窗中部分版本识别不出 MC 版本（[convert.rs](src-tauri/src/minecraft/community/curseforge/convert.rs) / [convert.rs](src-tauri/src/minecraft/community/modrinth/convert.rs)）：CurseForge 老整合包文件 `game_versions` 常为空或仅有无点值（如 `Minecraft 1.12`）被过滤掉，现对 ModPack 在版本列表为空时从文件名/显示名兜底提取 MC 版本（如 `RLCraft 1.12.2 - Beta v2.8.1.zip` → `1.12.2`，新增 [version_extract.rs](src-tauri/src/minecraft/community/version_extract.rs) 的 `extract_mc_version_from_name`）；版本列表链路透传 `resource_type` 以区分 Mod/整合包。
- 修复 Legacy Forge（1.12.2 及以下）安装时 maven 库 0 个文件解压（[legacy.rs](src-tauri/src/minecraft/loaders/forge/legacy.rs)）：Zip Slip 防护改用 `utils::path::ensure_safe_relative_path` 段级校验（复用 assets 下载同一工具），不再依赖 `canonicalize`——Windows 上已存在的 `libraries` 基目录 `canonicalize` 返回 `\\?\` 前缀，而尚未解压的目标降级为普通路径，`starts_with` 必然失败导致全部 maven 条目被误判跳过，Forge 库缺失无法启动。
- 修复 CI clippy 失败（[online_query.rs](src-tauri/src/minecraft/community/preload/online_query.rs)）：`resource_type` 为 `Copy` 类型，去掉 `tokio::join!` 中多余的 `.clone()`（clippy `clone_on_copy`）。
- 修复版本选择页空状态「下载游戏」跳错页面（[VersionSelect.vue](src/views/VersionSelect.vue)）：此前跳转到下载管理页（`/apps/downloads`），改为进入下载页（`/apps/versions`，原版/社区资源安装）。
- Native 库日志降级为 debug（[natives.rs](src-tauri/src/minecraft/launch/pipeline/natives.rs)）：`[Natives] Processing/Extracting/Extracted/JAR SHA1 verified` 逐文件日志由 INFO 改为 debug 级别（SHA1 不匹配等警告仍为 warn），避免启动时刷屏。
- 启动脚本导出支持 macOS / Linux（[script_export](src-tauri/src/commands/version/script_export/) + [useVersionOverviewActions.ts](src/composables/useVersionOverviewActions.ts)）：此前仅生成 Windows .bat，现按当前系统自动切换格式——Windows 生成 .bat（GBK + CRLF + icacls 权限限制），macOS/Linux 生成 .sh（UTF-8 + shebang + chmod 700 可执行权限）；文件对话框默认名与过滤扩展名同步跟随系统；保存路径缺扩展名时后端自动补齐；.sh 脚本同样写入真实 access_token / uuid 可直接启动，含敏感信息警告。
- 修复导出的启动脚本 access_token / uuid 被脱敏导致无法启动（[content.rs](src-tauri/src/commands/version/script_export/content.rs) + [export.rs](src-tauri/src/commands/version/script_export/export.rs)）：脚本直接写入真实的 `--accessToken` / `--uuid`（文件权限限制为当前用户），移除「已脱敏为 *** 请手动替换」逻辑，文件头部保留并加强警告提示（含启动必需 token，勿分享，失效后重新导出）。
- 修复概览页「光影文件夹」快捷方式误显示（[useVersionSettings.ts](src/composables/useVersionSettings.ts) + [OverviewTab.vue](src/views/version-settings/OverviewTab.vue)）：此前沿用 `isModable`（有 Mod 加载器即显示），改为复用后端 `is_packs_available` 对 Shader 检查 OptiFine/Iris（与光影管理页 PackTab 同一逻辑），无光影加载器时隐藏入口，切换版本自动重新检查。
- 图片缓存日志降级为 debug（[download.rs](src-tauri/src/minecraft/image_cache/download.rs)）：`[ImageCache] 已缓存` 由 INFO 改为 debug 级别，避免每个图片缓存都刷屏 INFO 日志。
- 资源包/光影列表 UI 对齐 Mod（[PackTab.vue](src/views/version-settings/PackTab.vue) + [PackListItem.vue](src/views/version-settings/pack-tab/PackListItem.vue)）：列表外层新增 `p-6` 内边距与圆角白卡片容器（含边框阴影），不再铺满难看；列表项内边距加宽（`px-4 py-3`），图标改为平台 logo → 包内图标 → 保底图三级优先，标题行显示平台工程名。
- 资源包/光影详情与更新联动（[usePackList.ts](src/composables/usePackList.ts) + [usePackOperations.ts](src/composables/usePackOperations.ts) + [PackUpdateDialog.vue](src/views/version-settings/pack-tab/PackUpdateDialog.vue)）：新增详情按钮，点击后按「project 已就绪 → 等待预加载 → 本地信息」三级 fallback 弹窗，复用社区 [ResourceDetail.vue](src/components/community/ResourceDetail.vue)（CurseForge / Modrinth 版本列表联动）；匹配到平台工程的包额外显示「更新/更改版本」按钮，复用 mod-tab 的 VersionTable 弹出版本列表安装新版本。
- 预加载参数化复用 Mod 链路（[preload](src-tauri/src/minecraft/community/preload/) + [packs/preload.rs](src-tauri/src/commands/version/packs/preload.rs)）：`PreloadScope`（事件前缀 / 资源类型 / 缓存目录 / 是否读 JAR）把 mods 专用预加载泛化为 packs 共用，zip 包仅按 hash 匹配 CF/MR 工程（不读 JAR 元数据）；缓存按 `preload_mods` / `preload_resourcepack` / `preload_shader` 分目录，前端 `usePacksPreload` 合并 `packs-preload-update` 事件，`AbortHandle` 管理预加载 task，切换/卸载时取消。
- 修复资源包/光影目录重复监听日志（[watcher.rs](src-tauri/src/commands/version/packs/watcher.rs) + [pack_common.rs](src-tauri/src/commands/version/pack_common.rs)）：移除 `watcher.rs` 自带的 `[PackWatcher] 开始监听` 日志，仅保留 `pack_common::watch_dir` 统一打印一行（含事件名），避免同一监听输出两条日志（实际仍是全局单例 watcher + 500ms 防抖，无重复监听）。
- 新增版本「资源包 / 光影」管理（[packs](src-tauri/src/commands/version/packs/) + [PackTab.vue](src/views/version-settings/PackTab.vue)）：版本设置新增资源包 / 光影两个子页，支持安装、启停、删除、刷新、打开目录、定位文件，启停与删除会同步写 options.txt（资源包 `resourcePacks` 数组 / 光影 `shaderPack` 键），游戏内与启动器侧状态一致；图标优先取包内 pack.png / icon.png / preview.png（限量读取防 zip 炸弹），无则用保底图。
- 提取 pack_common 公共抽象层（[pack_common.rs](src-tauri/src/commands/version/pack_common.rs)）：目录解析、列表枚举、启停重命名、删除、安装、原子更新（DownloadSession）、notify 目录监听均上收为公共函数，mods 链路（list/manage/install/update/watcher/helpers）与 packs 模块共用，不再复制粘贴；options.txt 读写同步提取为 `minecraft/resourcepack_options.rs`，离线皮肤模块同步改为复用。
- 保底图迁移与公共函数（[assets.ts](src/utils/assets.ts) + 三处组件）：`src/assets/Mods/default.png`、`default-min.png` 迁移至 `src/assets/Common/`，新增 `defaultAsset(min?)` 公共函数统一返回保底图 URL；PackListItem（大图）、ModListItem、ModUpdateDialog（小图）三处引用全部收敛到该函数。
- 修复资源包/光影页空状态滚动条（[PackEmptyState.vue](src/views/version-settings/pack-tab/PackEmptyState.vue) + [ModEmptyState.vue](src/views/version-settings/mod-tab/ModEmptyState.vue)）：空列表状态去掉 `min-h-[400px]` 撑高，不再因内容区高度不足出现多余滚动条。
- 光影可用性检查（[packs/list.rs](src-tauri/src/commands/version/packs/list.rs) + [PackTab.vue](src/views/version-settings/PackTab.vue)）：`is_packs_available` 对光影检查 OptiFine（版本 JSON/ID）或 Iris（mods 目录含 iris*.jar）加载器，无加载器时显示「该版本不支持光影」提示（仿 Mod 的 not-modable，含跳转下载/版本选择按钮）；资源包原版即可用，不受影响。

## [0.3.5-rc4] - 2026-08-10

- release 工作流 S3 上传自动重试（[ci-upload.cjs](scripts/ci-upload.cjs)）：上传安装包 / 签名文件到 S3 时，遇 Cloudflare 回源源站错误（HTTP 520~527、530，如 522）或网络错误自动重试，指数退避（1s/2s/4s 封顶 8s）最多 3 次；403 等鉴权错误不重试，分片上传各分片同样生效；`httpRequest` 同步精简注释。

- stars/forks 徽章配色调整（[README.md](README.md) + [README_EN.md](README_EN.md) + [README_ZH-HANT.md](README_ZH-HANT.md) + [README_JA.md](README_JA.md)）：移除 `labelColor=165dff`，标签区恢复默认深色，仅数值区为蓝色 `165dff`，与右侧 issues 等徽章外观一致；保留自定义金色星星 / 白色分支图标。

- 修复文本框滚动误触发返回顶部按钮（[Input.vue](src/components/common/Input.vue)）：textarea 模式内部滚动此前会被全局 scroll 监听捕获（BackToTop 按钮误弹出），现复用 `data-inner-scroll` 白名单标记（BackToTop 已在滚动监听与容器检测中统一过滤），文本域内部滚动不再触发全局返回顶部按钮。

- stars/forks 徽章样式统一（[README.md](README.md) + [README_EN.md](README_EN.md) + [README_ZH-HANT.md](README_ZH-HANT.md) + [README_JA.md](README_JA.md)）：由 `for-the-badge` 深色大徽章改回 `style=flat` + 蓝色 `165dff`，与 issues/last-commit/contributors 右侧徽章协调；保留自定义金色星星 / 白色分支图标。

- 修复 CI 的 clippy/fmt 失败（[lib.rs](src-tauri/src/lib.rs)）：`is_internal_navigation` 的 `map_or(false, ...)` 改为 `is_some_and`（clippy `unnecessary_map_or`），并按 rustfmt 规范重排；本地 `cargo fmt --check` 与 `cargo clippy --all-targets -- -D warnings` 均已通过。

- README 徽章与功能特性改版（[README.md](README.md) + [README_EN.md](README_EN.md) + [README_ZH-HANT.md](README_ZH-HANT.md) + [README_JA.md](README_JA.md)）：stars 徽章改用金色星星、forks 沿用白色分支，两者切换为 `for-the-badge` 样式（深色标签 + 金色高亮，`logoSize=auto`）；「功能特性」由数十行多子章节精简为 6 条要点 + 结尾一句云端 PoW 说明；贡献者章节追加 Repobeats 仓库活跃度统计图。

- 新增全局外部链接导航守卫（[App.vue](src/App.vue) + [useExternalLinkGuard.ts](src/composables/useExternalLinkGuard.ts) + [lib.rs](src-tauri/src/lib.rs)）：禁止 webview 内直接跳转外部网站——此前 AI 日志分析等页面输出 GitHub 链接，点击会直接跳走、页面被困在应用内无法关闭。前端 App.vue 挂载全局点击拦截（复用 `handleMarkdownLinkClick`，二次确认后经 shell 插件在系统浏览器打开）；后端新增 `on_navigation` 导航守卫插件兜底拦截 JS 程序化导航，仅放行内部 URL（内置协议 + localhost/*.localhost）。

- 修复 README GitHub 徽章自定义图标不生效（[README.md](README.md) + [README_EN.md](README_EN.md) + [README_ZH-HANT.md](README_ZH-HANT.md) + [README_JA.md](README_JA.md)）：stars/forks 徽章追加的自定义白色星形/分支 SVG 图标此前不可见——`github/*` 徽章未指定 `style` 时默认 social 样式，近白背景忽略 `color` 使白色图标隐形；现为全部 5 个徽章（stars/forks/issues/last-commit/contributors）补充 `style=flat`，stars/forks 另设 `labelColor=165dff`，图标于蓝色标签上以白色渲染，徽章整体蓝底统一。

- README 精简并新增贡献者（[README.md](README.md) + [README_EN.md](README_EN.md) + [README_ZH-HANT.md](README_ZH-HANT.md) + [README_JA.md](README_JA.md)）：四种语言同步移除「技术架构 / 项目结构 / 环境要求 / 开发与构建（含质量检查命令）」章节，文档聚焦产品特性；在「鸣谢」与「相关链接」之间新增「贡献者」章节，接入 contrib.rocks 本项目（MoTeam-cn/MoLaunch）贡献者头像墙。

- 创建房间交互优化（[CreateRoomForm.vue](src/components/online/CreateRoomForm.vue)）：「高级设置」入口按钮仅在实际启用（关联整合包/开启白名单）时显示状态徽章，未启用时不再显示「未启用」灰标，按钮更清爽；创建房间进度（STUN → 创建两步指示器）由按钮下方的内联区块改为右侧抽屉，点击「创建房间」自动弹出、创建完成/失败自动收起（无遮罩、无手动关闭按钮，由进度状态驱动开合）。

- 联机房间详情抽屉化（[RoomHostPanel.vue](src/components/online/RoomHostPanel.vue) + [CreateRoomForm.vue](src/components/online/CreateRoomForm.vue) + [PendingAnswerList.vue](src/components/online/PendingAnswerList.vue) + [ParticipantList.vue](src/components/online/ParticipantList.vue) + [BannedList.vue](src/components/online/BannedList.vue)）：房主面板「待确认加入申请 / 参与者 / 封禁列表」由页面内直接展示改为右侧抽屉，详情页仅保留「房间管理」按钮卡（按钮带红色 Tag 待办数与参与者/封禁数量徽标），封禁抽屉标题内置刷新按钮；创建房间「高级设置」（整合包关联 + 白名单管理）由可折叠卡片改为抽屉，表单页仅留「高级设置」入口按钮并保留启用状态徽章；三个列表组件去 Card 壳适配抽屉容器，待确认/封禁列表新增空状态（icon + text 垂直水平居中）。

- 联机会话全局化（新增 [onlineSession.ts](src/composables/online/onlineSession.ts)，[App.vue](src/App.vue) 启动时挂载）：WebRTC 多 PC/单 PC、TUN 虚拟网卡、房主三路信令轮询从页面/面板组件生命周期提升为应用级全局单例，离开联机页（路由切走）不再触发 onUnmounted 清理，P2P 连接与虚拟网卡保持不断，返回联机页直接恢复；`useWebRTC`/`useWebRTCMesh`/`useVirtualLan`/`useRoomHost`/`useGlobalTauriEvent` 新增 `autoClose`/`autoStop`/`autoLifecycle`/`autoRemove` 选项由会话显式管理生命周期（默认仍自动清理，不破坏既有调用方）。

- 加入方房间状态监控（[onlineSession.ts](src/composables/online/onlineSession.ts)）：加入方每 30s 请求云端房间信息，房主关闭/房间过期/被服务端清理（code=1002）时自动感知并清理会话退出，不再出现房主关房后加入方无感知；DataChannel 控制消息（房主 MC 端口 / TURN 服务器下发）与数据包转发 TUN 也改为全局绑定，切页不丢失。

- 修复下载页展开「选择加载器」后全局返回顶部按钮残留（[BackToTop.vue](src/components/common/BackToTop.vue)、[useFloatingButtonState.ts](src/composables/useFloatingButtonState.ts)、[Versions.vue](src/views/Versions.vue)）：LoaderSelect 是页内视图切换（非路由切换），BackToTop 的可见状态不会被路由钩子重置，残留按钮会遮挡右下角「开始安装」；新增 `backToTopEnabled` 白名单开关（全局可复用），LoaderSelect 展开时禁用 BackToTop（隐藏并停止响应滚动），收起/切换分类/离开页面时自动恢复并重新检测滚动位置。

- 加入方 P2P 断线自动重连（[onlineSession.ts](src/composables/online/onlineSession.ts)）：WebRTC 网络抖动先由 ICE 自行恢复；`disconnected` 超过 5s 未恢复或直接 `failed` 时自动走服务端信令重建（leaveRoom → joinRoom 新 participant_id，房主轮询自动为新参与者重新生成 Offer，完成重新协商），失败按 3s/6s/12s 退避重试至多 3 次，连接恢复自动清零计数；[reconnectAsGuest](src/composables/useRoomReconnect.ts) 同步增强：重连前 close 旧 PC（避免 failed 态复用）、重新注入加密密钥、服务端按房间递增分配虚拟 IP 导致重新 join 必然换 IP 时重启 TUN 接口（顺带修复管理员提权重启后虚拟网卡 IP 与服务端不同步的问题）；房主轮询（[useRoomHostPolling.ts](src/composables/useRoomHost/useRoomHostPolling.ts)）自动清理已离开/被拒绝参与者的残留 PeerConnection，防止重连产生的旧 participant 连接泄漏。

- 修复下载总大小虚高（[stream.rs](src-tauri/src/minecraft/download/downloader/stream.rs)）：单流下载回填 `total_bytes` 由「无条件按 content_length 累加」改为「仅 `expected_size=0`（大小未知）时回填」，已知大小文件已在 `download_batch` 初始化时按 `expected_size` 求和计入，此前会被二次累加导致「已下载/总大小（累计）」随下载过程持续增长、完成时显示总量远超实际；失败回滚条件同步对齐，避免 3 次重试间 total 翻倍。

## [0.3.5-rc3] - 2026-08-10

- 修复 Release 内容生成（[scripts/generate-release-content.cjs](scripts/generate-release-content.cjs)）：条目格式改为短哈希反引号前置 + 提取 `feat(scope):` 括号内 scope 加粗渲染（`**scope**: 描述`），commit by 署名改为 `[@login](url)` 纯文字链接；协作者头像改经 [images.weserv.nl](https://images.weserv.nl/) `mask=circle` 烘焙圆形 PNG 渲染——GitHub 正文 HTML sanitizer 会剥离 `<img>` 的 style/class（`border-radius` 圆角失效导致方块头像），圆角烘焙进图片本体可绕过清洗；移除 `href="#"` 空链接与 `@gravatar` 裸文字回退，仅当邮箱与登录名均缺失时才回退名字文字。

- SDK 移除 updater FFI（[instance.rs](src-tauri/src/sdk/instance.rs) + [ffi_types.rs](src-tauri/src/sdk/ffi_types.rs) + [types.rs](src-tauri/src/sdk/types.rs) + [sdk.rs](src-tauri/src/commands/sdk.rs) + [manager.rs](src-tauri/src/commands/sdk/manager.rs) + [sdk-manager.ts](src/utils/api/sdk-manager.ts)）：新 SDK 已删除 `mc_update_check_lite` / `mc_update_free_info_lite` 导出，旧绑定将其视为必需符号导致整个 SDK lite 加载失败（`Failed to get mc_update_check_lite: GetProcAddress failed`）；移除 `FFIUpdateInfoLite`/`UpdateInfoLite` 结构、`update_check_lite()` 方法、`check_update_lite` 命令与前端 `CHECK_UPDATE_LITE` action 后 SDK 恢复加载；更新检测改由主包 tauri-plugin-updater 自研链路承担，不再依赖 SDK。

- 消除 release 工作流 Node 20 弃用警告（[release.yml](.github/workflows/release.yml)）：`upload-artifact` v5→v6、`download-artifact` v5→v8、`action-gh-release` v2→v3，全部运行在 node24 运行时；`tauri-action` v0 与 `rust-cache` v2 本身已是 node24，无需变更。

- cargo audit 剩余告警处理（[.cargo/audit.toml](.cargo/audit.toml)）：`rustls-pemfile`（RUSTSEC-2025-0134）已随依赖升级消除；其余 18 项告警逐一验证为上游锁定传递依赖（gtk-rs GTK3 系列 10 项 + glib 0.18.5 unsound 依赖 libappindicator 0.9.0→gtk 0.18；paste 1.0.15 经 tun-rs→netlink-packet-utils；proc-macro-error 1.0.4 经 glib-macros；unic-* 5 项经 tauri-utils→urlpattern 0.3.0），均无升级路径，已在 audit.toml 中按 advisory ID ignore 并逐条注明理由与移除条件。

- 升级 Rust HTTP/TLS 依赖修复 cargo audit 漏洞（[Cargo.toml](src-tauri/Cargo.toml) + [manage.rs](src-tauri/src/certs/manage.rs) + 新增 [.cargo/audit.toml](.cargo/audit.toml)）：`reqwest` 0.11→0.12、`rustls-native-certs` 0.6→0.8，统一到 rustls 0.23，`rustls-webpki` 0.101.7（3 个 RUSTSEC-2026-0098/0099/0104）与 `rustls-pemfile` 1.0.4 移出依赖树；`load_system_root_certificates` 适配 `CertificateResult`（`CertificateDer`→`reqwest::Certificate`）；`rsa` 0.9.10 上游无补丁（RUSTSEC-2023-0071，Marvin Attack 仅影响私钥操作，项目只用公钥 OAEP 加密不可达）在 audit.toml 中 ignore 并注明理由；GTK3/glib 等 19 项警告均为 Tauri Linux 传递依赖，不阻塞 CI。

## [0.3.5-rc2] - 2026-08-10

- CI 升级 GitHub Actions 运行时到 Node 24（[ci.yml](.github/workflows/ci.yml) + [release.yml](.github/workflows/release.yml) + [version-sync.yml](.github/workflows/version-sync.yml) + [license-sync.yml](.github/workflows/license-sync.yml)）：`checkout` v4→v5、`setup-node` v4→v5、`upload-artifact`/`download-artifact` v4→v5、`action-gh-release` v1→v2、`git-auto-commit-action` v6→v7，消除 "Node.js 20 is deprecated ... forced to run on Node.js 24" 警告；`rust-cache` v2.9.2 与 `tauri-action` v0 本身已是 node24，无需变更。

- CI 修复（[ci.yml](.github/workflows/ci.yml) + [release.yml](.github/workflows/release.yml)）：Node 18 → 22（`@vitejs/plugin-vue@6` 的 `getHash` 使用 Node 21.7+ 的 `crypto.hash`，Node 18 下前端构建报 `crypto.hash is not a function`）；cargo 安全审计 action 由已失效的 `rustsec/rustsec-action` 更换为官方维护的 [actions-rust-lang/audit](https://github.com/actions-rust-lang/audit)@v1（`file` 指向 `src-tauri/Cargo.lock`，不自动创建 issue）。

- 普通重启记住上次页面（[relaunchSnapshot.ts](src/utils/relaunchSnapshot.ts) 新增 `saveLastPage`/`readLastPage`，[App.vue](src/App.vue) 路由 `afterEach` 记录 + 启动恢复）：**默认关闭**，可在「设置 → 个性化 → 启动」开启，开启后打开设置页等任意业务页再重启，自动回到上次打开的页面；UAC 提权重启仍走加密快照恢复页面 + 房间会话。

- 重启快照统一改为 SDK 加密存储（[relaunchSnapshot.ts](src/utils/relaunchSnapshot.ts) 新增，替代 [relaunchRestore.ts](src/utils/relaunchRestore.ts) + [roomSnapshot.ts](src/utils/roomSnapshot.ts)；后端新增 [relaunch.rs](src-tauri/src/commands/relaunch.rs) 命令，复用 `sdk_crypto` 加解密封装）：重启前将"当前页面 + 在线房间会话（含房间密码 / roomKey）"经 SDK AES-256-CBC 加密后写入 localStorage，新实例启动后解密恢复页面跳转与房间自动重连；修复 CodeQL 明文存储敏感信息告警，升级前遗留的旧版明文快照键启动时自动清理。

- 加密方案彻底依赖 SDK AES-256-CBC（[sdk_crypto.rs](src-tauri/src/utils/sdk_crypto.rs) + [crypto_v3.rs](src-tauri/src/migrations/crypto_v3.rs) + [instance.rs](src-tauri/src/sdk/instance.rs) + [ffi_types.rs](src-tauri/src/sdk/ffi_types.rs)）：删除自实现文件级加密（master.key / DPAPI / AES-256-GCM / `v2:` 前缀）全部路径，加解密完全由 SDK `mc_encrypt_token` / `mc_decrypt_token` 承担（AES-256-CBC 写入，解密自动兼容旧 DES，协议见 [token-encryption.md](docs/token-encryption.md)）；启动迁移改用 SDK 0.6.0 新增 `mc_decrypt_token_ex` 检测算法版本（1=DES 旧密文，2=AES 当前），仅将存量 DES(v1) 数据（认证注册表/auth.json、联机 device.json、FRP token、CurseForge/AI api_key）重加密为 AES(v2)，v2 数据直接跳过，完成写 `crypto_v3.done` 标记，失败不阻塞启动且保持原样。

- 前端 lint 告警清零（[.eslintrc.cjs](.eslintrc.cjs) + 8 个组件文件）：`@typescript-eslint/no-unused-vars` 配置下划线忽略参数与 rest 解构兄弟字段；`Input.vue` 的 `maxlength`（默认 -1 不限制）、`ToggleRow.vue` 的 `description`/`tooltipText` 补默认值；6 处 `v-html` 点位加 eslint-disable 注释并注明安全兜底（renderMarkdown 的 DOMPurify 消毒 / MOTD 转义白名单 / 静态图标资源），`npm run lint` 现已零告警通过。

- 代码风格清理（cargo fmt 与 clippy）：对安全修复引入的代码执行 rustfmt 格式化（certs/manage.rs、plugins/layout.rs、plugins/spawn.rs、resources.rs、utils/path.rs），修复 `needless_borrow`（extract.rs）与 `manual_is_multiple_of`（bridge.rs）两处 clippy 告警；`cargo clippy --all-features -D warnings` 与 `cargo fmt --check` 现已零告警通过（对齐 CI 门禁）。

- 自定义根证书校验加固（[manage.rs](src-tauri/src/certs/manage.rs) + [perms.rs](src-tauri/src/minecraft/system/shell/perms.rs) + [Cargo.toml](src-tauri/Cargo.toml)）：`add_custom_cert` 写盘前用 `x509-parser` 校验 BasicConstraints `CA:TRUE` 与有效期（区分未生效/已过期，错误携带主题 CN），叶子证书/任意 PEM 不再能被加入信任链；`cert_dir` 每次返回前收紧目录权限（Windows icacls 当前用户 / Unix 0700，新增公共 `restrict_dir_permissions`，可自愈旧版本宽权限目录）。

- TUN 入站帧校验（[bridge.rs](src-tauri/src/minecraft/online/bridge.rs)）：DataChannel → TUN 写入前校验 IPv4 帧源/目标地址均属于虚拟子网（复用 `VirtualNetInfo` 的 IP 与前缀，零新增状态），越界帧丢弃并计数式告警；非 IPv4/无法解析帧默认放行，不破坏现有联机流程；目标组播帧如后续需要可在校验中加白名单。

- 加固 ECIES/PoW/SSE 健壮性（[ecies.rs](src-tauri/src/minecraft/online/ecies.rs) + [pow.rs](src-tauri/src/minecraft/online/pow.rs) + [sse.rs](src-tauri/src/ai_core/client/sse.rs)）：ECIES 解密拒绝全零临时公钥与全零共享密钥（覆盖 X25519 低阶点）；PoW difficulty 钳制上限 32 防服务端放大 DoS；SSE 缓冲新增单行 1MB / 累计 4MB 上限，超限丢弃该行并计数。

- 依赖升级修复已知 CVE（[Cargo.toml](src-tauri/Cargo.toml) + [package.json](package.json) + [package-lock.json](package-lock.json)）：
  - Rust：`zip` 2→4（4.6.1）、`rusqlite` 0.31→0.40（bundled，libsqlite3-sys 至 0.38），读取/写入/查询 API 无破坏性变化、调用点零改动（zip-slip 防护走既有 `utils::path::ensure_safe_relative_path` 不受影响）；`cargo check` 与 `cargo test --lib`（225 passed）通过。
  - 前端：`vite` 5→6.4.3（修复 GHSA-v6wh-96g9-6wx3 / GHSA-fx2h-pf6j-xcff）、`@vitejs/plugin-vue` 5→6.0.8、`vitest` 1→3.2.7（vite-node 3.2.4）、`@vue/eslint-config-typescript` 12→13（typescript-eslint 依赖升级，minimatch 至 9.0.9 消除 ReDoS 链）；`npm audit --audit-level=high` 归零，typecheck / build / lint 通过。

- CI 新增依赖漏洞扫描（[.github/workflows/ci.yml](.github/workflows/ci.yml)）：frontend-check 末尾追加 `npm audit --audit-level=high`，rust-clippy 末尾追加 `rustsec/rustsec-action`（lockfile 指向 src-tauri/Cargo.lock），权限保持 `contents: read` 不变。

- 修复插件自定义布局 SSRF/内网探测（[net.rs](src-tauri/src/utils/net.rs) 新增 + [layout.rs](src-tauri/src/commands/plugins/layout.rs) + [validate.rs](src-tauri/src/commands/frp/sandbox/validate.rs)）：内网地址判定抽为公共函数 `utils::net::is_private_address`（v4 私网/回环、v6 回环、localhost，含 host:port），frp 校验复用同一实现；`load_custom_layout` 请求前解析 URL host 并拦截内网地址，插件布局不再能探测 `127.0.0.1:*` / `192.168.*` 等内网端点。

- 本地凭证存储加密升级（[sdk_crypto.rs](src-tauri/src/utils/sdk_crypto.rs) + [Cargo.toml](src-tauri/Cargo.toml) + [online storage.rs](src-tauri/src/minecraft/online/storage.rs) + [auth manager.rs](src-tauri/src/minecraft/auth/storage/manager.rs) + [frp storage.rs](src-tauri/src/commands/frp/auth/storage.rs) + [secure_storage.rs](src-tauri/src/minecraft/community/secure_storage.rs) + [ai storage.rs](src-tauri/src/ai_core/storage.rs)）：新增文件级强加密原语 `encrypt_file_securely`/`decrypt_file_securely`（AES-256-GCM + 随机 12B nonce，输出 `v2:base64(...)`），32 字节随机主密钥存 `AppData/master.key`（Windows 用 DPAPI 保护，非 Windows 0600 权限）；联机 device.json / MC 账号 / FRP token / CurseForge / AI api_key 统一改用新封装，SDK DES 仅回退解密旧数据；删除联机存储 SDK 不可用时的明文降级分支，加密/解密失败直接返回错误。

- 修复 picker 子窗口 XSS 风险（[scheme.rs](src-tauri/src/commands/tools/picker_window/scheme.rs) + [markdown.html](src-tauri/resources/templates/markdown.html) + [picker-templates.ts](src/config/picker-templates.ts) + [resources.rs](src-tauri/src/resources.rs)）：`__PICKER_DATA__` 注入前将 `</script` / `<!--` 转为 JSON 合法转义（`<\/script` / `<\!--`）防脚本标签逃逸；markdown 渲染改为 `DOMPurify.sanitize(marked.parse(...))` 消毒（新增 dompurify.min.js 内联注入，所有库共用注入路径一并受益），try/catch 兜底分支同样消毒；CSP 因依赖库无 nonce 内联注入必须保留 'unsafe-inline'，已注明风险由数据转义 + DOMPurify 收敛。

- 修复插件子进程执行安全缺陷（[spawn.rs](src-tauri/src/commands/plugins/spawn.rs)）：构建 Command 时 `env_clear()` 并仅注入白名单变量（PATH / 代理 / SystemRoot / TEMP / ComSpec / USERPROFILE 等）防敏感环境变量泄漏；stdout/stderr 改用 `take(MAX_OUTPUT_BYTES+1)` 有界读取防无界内存消耗；实现每插件并发计数（`max_concurrent`，默认 1 上限 5，正常退出与超时 kill 均释放计数）。

- 收缩 IgnoreTls 作用域（[tls.rs](src-tauri/src/http/tls.rs) + [client.rs](src-tauri/src/http/client.rs) + [http.rs](src-tauri/src/http.rs) + [developer.rs](src-tauri/src/commands/system/developer.rs)）：新增 `ignore_tls_allowed(host)`，仅当 IgnoreTls 开启且目标为 localhost/127.0.0.1/::1 时才允许跳过证书校验；现有通用 HTTP 客户端不绑定 base_url、无法低成本获知目标 host，保守起见一律不再调用 `danger_accept_invalid_certs`，联机认证/下载等链路不再受 IgnoreTls 全局绕过；开关开启时启动日志输出一次性 WARNING 提示；默认配置（IgnoreTls 关闭）行为不变。

- 修复整合包 overrides 解压路径穿越（Zip Slip）（[extract.rs](src-tauri/src/commands/community/install/concurrent/extract.rs) + [path.rs](src-tauri/src/utils/path.rs)）：新增公共校验 `ensure_safe_relative_path`（段级 `ParentDir` 检查 + 拒绝空串/空字节/绝对路径/盘符前缀），`extract_overrides_once` 解压前逐条目校验，恶意 zip 内 `overrides/../../..` 条目不再能逃出 instance 目录写任意文件；附内联单测。

- 修复可执行资源释放的缓存信任缺陷（DLL 劫持）（[resources.rs](src-tauri/src/resources.rs)）：`extract_resource` 命中缓存改为重算目标文件本体 sha256 与期望值比对（不再信任可伪造的同目录 `.sha256` 文本），不一致即重新覆盖写入；写入后追加 `restrict_file_permissions` 收紧权限（Windows icacls / Unix 0600）。SDK DLL、updater.exe、wintun.dll 等全部资源释放点随之受益。

- 修复 frpc 命令直连模式 token 参数注入（[spawn.rs](src-tauri/src/commands/frp/process/spawn.rs) + [validate.rs](src-tauri/src/commands/frp/sandbox/validate.rs)）：`{token}` 不再经 `split_whitespace` 拆词，改为整体单一参数传入；token 校验追加拒绝前导 `-`、空白字符与 `,`/`=`，spawn 前另做同规则防御校验（纵深，覆盖升级前遗留的旧数据），并顺带避免 token 落入启动日志。

- 日志脱敏覆盖面扩展（[sanitize.rs](src-tauri/src/logger/sanitize.rs) + [sanitize_tests.rs](src-tauri/src/logger/sanitize_tests.rs)）：JSON 敏感字段集新增 `password`/`passwd`/`secret`/`api_key`/`apikey`/`client_secret`/`authorization`，新增 `Authorization: Bearer` 头与 URL query（token/key/api_key/apikey/signature/sig）脱敏，JWT 每段长度阈值由 10 降至 8。

- 修复 CI 三项检查失败（[.github/workflows/ci.yml](.github/workflows/ci.yml)）：
  - Rust 格式（`cargo fmt --all`）：[build.rs](src-tauri/build.rs)、[tun.rs](src-tauri/src/commands/online/manager/tun.rs)、[auth.rs](src-tauri/src/minecraft/online/client/auth.rs)、[pow.rs](src-tauri/src/minecraft/online/pow.rs)、[registry.rs](src-tauri/src/storage/registry.rs) 按 rustfmt 重排（长链换行、use 排序、闭包折叠）。
  - Clippy：`apply_config/secure.rs` 的 `apply_hint`（launch_count）与 `apply_user_agreed`（版本号）原 `let key = reg_key()?` 在非 Windows 下绑定 unit 值触发 `let_unit_value`，改为内联 `reg_set(&reg_key()?, ...)`；`apply_config/types/entry.rs` 的 `build_snapshot` 12 参数触发 `too_many_arguments`，按仓库既有惯例加 `#[allow(clippy::too_many_arguments)]`。
  - ESLint：`public/splash.js` 在 `window.__TAURI?.core` 上直接解构触发 `no-unsafe-optional-chaining`（短路返回 undefined 时抛 TypeError），改为 `window.__TAURI?.core?.invoke` 安全取值；`scripts/capture-splash.mjs` 使用 `process` 触发 `no-undef`，首行声明 `/* eslint-env node */`。

- Release 内容为每条提交追加 requarks 风格署名（[scripts/generate-release-content.cjs](scripts/generate-release-content.cjs)）：`classify` 的 `git log` 输出新增 author email/name，作者数据改为 `fetchAuthors()` 统一从 git + compare API 构建并复用（`classify` 与协作者区块共用）；署名命中 GitHub 登录名时渲染 `*(commit by [@login](url))*`（GitHub 原生渲染头像），邮箱查不到 GitHub 账号时用 Gravatar 小头像（20px `<img>` + md5 邮箱 identicon）兜底，比 requarks 的纯文本 `@login` 覆盖更全。

- Release 分类结构参考 requarks/changelog-action 细化（[scripts/generate-release-content.cjs](scripts/generate-release-content.cjs) + [.github/workflows/release.yml](.github/workflows/release.yml)）：
  - 「其他」不再一锅端，按常规提交类型细分为独立小节：性能优化 / 重构 / 测试 / 构建系统 / 文档 / 代码风格 / 杂项 / 其他（各小节 `###` 标题，无提交则不渲染）；
  - 新增「破坏性变更」小节置顶：conventional `type!: / type(scope)!:` 写法或 message 含 `BREAKING CHANGE` 的提交归入（`!c` 为 CI 跳过标记，不会误判），release body 在 NOTES 之后、FEATURES 之前渲染；
  - 保留零依赖解析与协作者头像 `<img>` 渲染不变。

- 修复 Release 协作者区块仍显示纯文字而非头像（[scripts/generate-release-content.cjs](scripts/generate-release-content.cjs)）：协作者渲染由 `@login` 提及（release 正文不渲染 @ 头像）改为 `<img>` 标签 + `width/height` 属性（避用会被 HTML sanitizer 剥离的 style），头像来源优先级为 compare API 的 `avatar_url` → GitHub 账号头像 URL（`avatars.githubusercontent.com/u/...`）→ Gravatar identicon（按 author email 的 md5 兜底，未关联 GitHub 账号的提交者如 `MoLaunch Bot <bot@moteam.top>` 也能显示确定性头像）；仅当邮箱与登录名均缺失时才回退名字文字。

- README 新增繁体中文（README_ZH-HANT.md）、英文（README_EN.md）、日文（README_JA.md）三种语言版本，并在 README 加入语言切换栏。

- 移除 `.github/dependabot.yml` 依赖自动更新配置（npm / cargo / GitHub Actions 每周检查），不再自动提交依赖更新 PR。

- 新增 `CONTRIBUTING.md` 贡献指南，并补充 `.github` 配置：Issue 模板选择器（config.yml）。

- 开发文档 `DEVELOPMENT_GUIDELINES.md`（开发规范）与 `DEVELOPMENT_BLUEPRINT.md`（架构蓝图）纳入版本控制（此前为 git 排除的内部文档）。

- 新增 GitHub Issue / PR 预设模板（`.github/ISSUE_TEMPLATE/`）：Bug 报告、功能建议、使用提问三个 Issue 模板（参考 reqable-app 结构），以及 `PULL_REQUEST_TEMPLATE.md`（含仓库提交规范 `type(scope): 描述 + !c`、CHANGELOG 同步与本地验证 checklist）。

- 下载管理进度推送由自建 WebSocket 改回 Tauri plugin event（emit）方案：删除 `src-tauri/src/ws/` 模块（server/auth/mod）与 `tokio-tungstenite` 依赖，`AppState` 移除 `progress_tx`/`ws_port`/`ws_token`（新增 `app_handle` 于 setup 注入），`get_ws_port` IPC 与前端 `getWsPort` 工具移除；后端所有进度/阶段/完成/暂停/恢复/取消推送统一走 `app.emit("download-progress")`（含整合包安装、资源下载等 `broadcast_current` 全部路径），前端 `useDownloadStream.ts` 从「getWsPort 建连 + auth 鉴权 + 3 秒重连」改为订阅 `download-progress` 事件（模块级单例监听，无需按下载状态建连断开），初始状态恢复链路（`isDownloading` + `getDownloadProgress` IPC）保留。

- 修复管理员提权重启不生效（UAC 确认后程序不重启）：`commands/online/manager/tun.rs` 提权启动改为携带 `--restart-as-admin` 参数，`src-tauri/src/lib.rs` 启动时检测该参数则跳过 `single-instance` 插件注册——此前新进程会被单实例插件识别为"第二实例"强制退出（旧进程 500ms 后才退出），导致 UAC 弹出但最终没有任何实例存活。
- 提权重启后恢复原页面：`useVirtualLan` 确认提权前将当前路由写入 `utils/relaunchRestore.ts`（localStorage），`App.vue` 会话恢复完成后校验路径合法性并 `router.replace` 跳回原页面（含 `?tab=` 页签），不再落回主页；未登录或路径非法时静默忽略。
- 提权重启后自动恢复房间会话（联机）：确认提权前将 roomState 快照（房间码/虚拟 IP/ICE/DataChannel 密钥/加入密码）写入 `utils/roomSnapshot.ts`，`App.vue` 启动时恢复——房主侧 RoomHostPanel 挂载后自动重建 TUN 并轮询为新参与者重新生成 Offer；加入方侧 `composables/useRoomReconnect.ts` 自动重新加入同一房间（新 participant_id 触发房主重新生成 Offer）并重建 WebRTC，UAC 被拒绝时清除恢复标记避免误恢复。

- 修复 Release 工作流提交区间与协作者头像生成（.github/workflows/release.yml + [scripts/generate-release-content.cjs](scripts/generate-release-content.cjs)）：
  - 提交区间：`release` job 的 checkout 由默认分支改为检出发布 tag（`ref: v${{ inputs.version }}`），`build-and-upload` 的 release_notes 区间同样改用 `v$VERSION` tag 锚定（`git rev-parse "${VERSION_TAG}^{commit}"`）——此前 `git describe HEAD^` 会把本次发布 tag 自身当作上一个 tag，导致 `git log v0.3.5-rc1..HEAD` 只取到 tag 之后的零星提交（本应 35 条却只剩 3 条）。
  - 协作者头像：baseSha 改由 `git rev-list -n 1 <tag>` 解析——原 `git rev-parse <tag>` 对 annotated tag 返回 tag 对象 sha（非 commit sha），compare API 直接 404，头像静默回退为 `git shortlog` 文本；作者集合改从 `git log` 的 author email 构建，经 compare API 匹配 GitHub 登录名，输出 `@login` 提及（GitHub 原生渲染圆形头像，不再使用会被正文 HTML sanitizer 剥离 style 导致方块头像的 `<img>`，也不再依赖 gravatar 兜底）；脚本新增可选参数 `head_sha` 由 workflow 显式传入当前检出提交，避免脚本内解析 HEAD 的歧义。

## [0.3.5-rc1] - 2026-08-09

- `src-tauri/updater/README.md` 相关链接移除指向 `docs/updater/design.md` 的入口（`docs/` 为本地内部文档目录，已通过 `.gitignore` 排除、不提交云端，README 不再暴露内部文档路径）。

- 修复 `version-sync.yml` 调用发布工作流的方式：原在 step 级 `uses: ./.github/workflows/release.yml` 触发报错（GitHub 只在 job 级支持调用可复用工作流，step 级 `uses` 仅接受含 `action.yml` 的本地 action 目录）；改为独立的 `release` job（`uses` + `needs: sync`，版本号经 `sync.outputs.version` 传递，`secrets: inherit` 继承签名私钥等密钥）。

- 打包元信息统一为全称：`package.json` / `package-lock.json` 的 `name` 由 `mo-launch` 改为 `molaunch`（与 tauri.conf.json 的 `identifier`（`com.moteam.molaunch`）与 deep-link scheme 保持一致），`package.json` 补充 `copyright` 字段（`Copyright © 2026 MoTeam. All rights reserved.`，与 LICENSE / `bundle.copyright` 一致）。

- [src-tauri/updater/README.md](src-tauri/updater/README.md) 按主仓库 README 风格重写为完整组件文档：新增居中标题 + 徽章栏（Rust / Windows / minisign / License）、`[!IMPORTANT]` 定位说明（子 crate 非完整启动器）、mermaid 工作流程图（等待退出 → 验签 → 替换 → 重启全流程及各退出码分支）、功能特性章节（进程等待 / 签名校验 / 文件替换 / 重启新版本）；并修正原文与实现不符的描述——验签依赖实为 `minisign-verify`（BLAKE2b-512 prehash + Ed25519，与 tauri-plugin-updater 同款）而非 ed25519-dalek + SHA-512，退出码顺序按 `main.rs` 实际执行顺序（参数 1 / 超时 2 / 替换 3 / 启动 4 / 验签 5）排列表述，集成方式对齐真实链路（`resources.rs::extract_updater` 释放 + `install_windows.rs` 经 `apply_pending_update` 启动子进程 + `last.exe`/`last.sig` 缓存）。

- 版本同步工作流（`.github/workflows/version-sync.yml`）的版本文件更新逻辑抽离到 [scripts/sync-version.cjs](scripts/sync-version.cjs)（Node.js 脚本，不再在 workflow 中堆叠 `node -p` / `npm version` / `grep` / `sed` bash）：`Update version files` 步骤改为一行 `node scripts/sync-version.cjs "$VERSION"`；脚本以 JSON 解析 + 2 空格缩进写回 `package.json` / `package-lock.json`（含 `packages[""]` 根条目，等价于原 `npm version` 行为），`src-tauri/tauri.conf.json` 采用定点字符串替换避免重排原格式，`src-tauri/Cargo.toml` 与 README.md shields.io 版本徽章（版本内 `-` 双写 `--` 转义）正则替换；文件无差异时不写盘，`git-auto-commit` 自动跳过行为不变。

- 版本同步工作流（`.github/workflows/version-sync.yml`）新增 README.md 版本徽章同步：打 `v*` 标签 / 手动指定版本时，按 shields.io 规则（路径中 `-` 为分隔符，版本内 `-` 需双写 `--` 转义）自动更新顶部 Version 徽章，并纳入自动提交范围。

- 修复 GitHub CodeQL 代码扫描告警：`.github/workflows/ci.yml` 全部 5 个 job 显式声明 `permissions: contents: read`（最小权限，消除 "Workflow does not contain permissions"）；`crypto_tests.rs` / `pow_test.rs` 中测试用固定盐/输入更名 `fixed_input` 并注明为确定性测试向量（消除 "Hard-coded cryptographic value" 误报，测试数据非真实密钥）。

- 发布构建提速 + Windows 便携版命名调整：`src-tauri/Cargo.toml` 发布 profile 由 `lto = true` + `codegen-units = 1`（fat LTO，CI 编译/链接最慢组合）改为 `lto = "thin"`，显著缩短全量构建时间，产物大小与性能影响极小（`opt-level = "s"` + `strip` 仍保证体积）；Windows 便携版产物更名 `MoLaunch_<version>_x64.exe`（去掉 `_portable` 后缀，与 `-setup` 安装版天然区分），release.yml 中便携版定位 glob（`*.exe` 排除 `*-setup.exe`）与 Release 附件 glob（`*_x64.exe`）同步调整，客户端 / 云端无硬编码文件名、不受影响。

- 发布工作流（`.github/workflows/release.yml`）Release 内容重构：body 删除「Downloads」区块（Assets 区已展示产物，移除冗余指引），提交记录不再一栏到底——按 commit 前缀自动分类为「新增内容（`feat*`）/ 修复（`fix*`）/ 其他」三个独立小节，每栏保留 `- subject ([hash](commit链接))` 格式并剥离尾部 `!c` 标记；`note:` 前缀的「作者的话」提取置顶展示（`######` 小字号标题，与更新弹窗语义一致）；最后一栏新增「协作者」小节（`git shortlog -sn` 统计本阶段内全部作者，按提交次数降序、顶部 20 人）。Windows setup 安装版维持现状：`--bundles nsis` 构建后仅作为 workflow artifact 附加到 GitHub Release，不上传 S3、不注册 apiServer（与便携版分流，便携版才推云端）。分类与协作者头像生成逻辑整体抽离到 [scripts/generate-release-content.cjs](scripts/generate-release-content.cjs)（Node.js 脚本，workflow 的 `Generate changelog from commits` 步骤只保留一行 `node` 调用，不再在 YAML 中堆叠 bash/Python）；协作者头像经 GitHub compare API 按提交邮箱关联账号拉取 `avatar_url`，未关联账号回退 Gravatar identicon 占位，API 不可用时回退 `git shortlog` 文本列表，工作流不因接口故障失败。

- 重写三份开发文档以匹配当前系统架构：`AI_AGENT_GUIDELINES.md`（新增「当前架构要点」章节：仓库结构、配置读写 / shell / resources / 下载源 / 进度回滚 / 组件复用 / 日志颜色 / 行数 / 测试位置等硬约束，以及「作者的话」多 note 约定）、`DEVELOPMENT_GUIDELINES.md`（技术栈与 scope 更新、补充更新日志多 note 约定、api-server 联动约定、后端测试文件拆分规范、命令速查增加 api-server 检查）、`DEVELOPMENT_BLUEPRINT.md`（目录结构、前端三大管理器域、后端模块总览、安全策略、新增第八章 MoLaunch 云端 api-server 架构）。并在两份规范中补充「作者的话」落地用法：`note:` 提交用 `git commit --allow-empty` 创建（零文件变更），且必须在打版本 tag 之前提交（插件按 tag 区间提取，tag 之后会落到下个版本）。

- 更新日志弹窗支持「作者的话」（多条）：约定 commit message 以 `note:` 开头即作者寄语（如 `note: 感谢大家的支持`），vite 构建插件 `updateLogPlugin` 将版本区间内**全部** `note:` commit 按顺序提取为数组，经虚拟模块 `virtual:update-log` 独立下发 `notes` 字段（不再混入 commit 列表，`ReleaseTimeline` 逻辑不变）；`UpdateLogDialog` 顶部按顺序渲染多条引用气泡样式的作者寄语区块（`ChatBubbleOvalLeftIcon` + Markdown 渲染，链接复用 `handleMarkdownLinkClick` 打开，无 note 时整块不渲染，完全向后兼容）。

- 打包程序补充版权元数据：`tauri.conf.json` 的 `bundle` 新增 `publisher: "MoTeam"` 与 `copyright: "Copyright © 2026 MoTeam. All rights reserved."`（与 LICENSE 版权声明对齐），Windows 打包产物右键属性「详细信息」将显示发布者与版权（tauri-build 写入 `CompanyName` / `LegalCopyright` 版本资源）。

- README.md 调整与 GitHub alert 应用：新增「与 MoLaunch 云端」小节（面向普通读者简述 PoW 轻量验证——注册/登录等云端接口先算一道哈希题再放行，正常使用无感、只有刷接口的人会感受到成本）；顶部引用块升级为官方 alert——第三方免责声明用 `> [!CAUTION]`，前排新增紫色 `> [!IMPORTANT]`（说明本项目为个人独立开发、多处使用 AI 辅助 Vibe Coding、如有不足请包涵），鸣谢 PCL2 独立声明用 `> [!NOTE]`。

- 联机鉴权接口接入 PoW Challenge 工作量证明（服务端 `api-server` 同步实现）：注册 / 登录 / 刷新 token 请求首次返回 `401 + code:1007` 时，自动解析服务端下发的 challenge（`challenge_id`/`salt`/`difficulty`/`path`/`header_name`），在后台线程并行求解 `SHA256(salt‖nonce)` 前导零哈希（`std::thread` 分片 + `mpsc::recv_timeout` 3 秒超时，不依赖 rayon、不阻塞 UI），求解成功后带 `header_name: {id}:{nonce}` 头重试一次——请求头字段名由服务端 DTO 动态下发（`PowChallengeResponse`），客户端不再硬编码 `x-molaunch-pow`，旧服务端未下发时回退默认值；重试前置校验 challenge 的 `path` 与请求路径一致（与服务端路径强绑定双保险），求解超时或失败则按原始 401 返回。登出不参与 PoW，保持原逻辑。求解模块 [src-tauri/src/minecraft/online/pow.rs](src-tauri/src/minecraft/online/pow.rs)（含解析 / 求解 / 前导零单测，测试按规范移出到同目录 `pow/pow_test.rs`，文件头注释 ≤5 行），[client/auth.rs](src-tauri/src/minecraft/online/client/auth.rs) 三个方法复用统一的重试辅助函数，两端零新增依赖（复用已有 `sha2`/`hex`/`rand`）。

- 开发者选项页面新增「使用协议与免责声明」抽屉（对齐联机 / 实验性功能 / 工具页）：`disclaimer.ts` 的 `DisclaimerKind` 增加 `developer` 类型，`DisclaimerDialog.vue` 新增开发者选项说明分支（日志查看、证书设置、DevTools、深链接注册等面向开发与排障场景，含 TLS 校验变更的风险提示），`Settings.vue` 在切换到「开发者」分类且当日未同意时弹出抽屉，同意后当日不再提醒（复用按自然日 localStorage 记录机制）。
- 修复更新日志时间线只渲染前两条的问题（`releaseTimeline.ts`）：`parseItems` 原先将带 CI 跳过标记 `!c` 的提交行整行 `continue` 跳过，而云端 release_notes 中 `v0.3.3-rc1..v0.3.4` 区间共 26 条提交、其中仅「accept linux runtime platform key」「retry empty AI tool follow-up response」两条不带 `!c`，导致前端展示的日志只剩两条；现改为剥离 `!c` 标记后保留条目（与 vite.config.ts `updateLogPlugin` 生成本地日志时剥离 `!c` 的语义对齐），服务器返回的完整更新日志可全部渲染。
- 新增「今日人品」便捷工具（[src/views/quick-tools/LuckyTool.vue](src/views/quick-tools/LuckyTool.vue) + 纯前端算法 [src/utils/lucky.ts](src/utils/lucky.ts)，移植自 `docs/fix-bug/runk.js`）：基于本机设备 ID（经 `sdk_manager` 获取）与日期哈希生成 0-100 每日幸运值，同一设备当天固定、跨天自动重置；展示幸运值大数字、等级标签、评语与进度条，设备 ID 打码显示。
- 修复开屏窗口无法鼠标拖拽移动：`public/splash.html` 的 `<body>` 挂裸值 `data-tauri-drag-region` 并给 `.scene` 及全部子元素 `pointer-events: none` 作整窗拖拽区，`public/splash.js` 在捕获阶段拦截 `mousedown` 直接调用 Tauri 原生的 `plugin:window|start_dragging`（失败原因写入开屏状态栏便于定位）；经状态栏诊断确认根因是权限——实测报错 `window.start_dragging not allowed on window "splashscreen"`，splashscreen 窗口不在任何 capabilities 权限集内（`migrated.json` 只授予 `main`），新建 [src-tauri/capabilities/splashscreen.json](src-tauri/capabilities/splashscreen.json) 单独授予 `core:window:allow-start-dragging` 后，无边框 + `transparent` 开屏窗口可任意位置按住拖动。
- 工具页新增「趣味工具」分类：`Tools.vue` 侧边栏在「外部下载」与「便捷工具」之间插入 `fun-tools` 新一级菜单（`FaceSmileIcon` 图标），并设为进入工具页的默认分类；「今日人品」`LuckyTool.vue` 由 `QuickTools.vue` 便捷工具列表迁移至该分类独立渲染（`v-else-if="activeCategory === 'fun-tools'"`），`ToolToc` 侧边目录同步生效。
- 全局返回顶部按钮白名单（`data-inner-scroll`）：为 17 处 main 内嵌次级滚动容器打标，滚动它们不再误触发全局「返回顶部」按钮——公共组件 `NavSidebar`（设置/工具/联机等页面共用左侧分类）、`LaunchPanel`（首页账号区）、`ChatConversationList`（AI 会话栏）、`LoaderCard`、`WhitelistEditor`、`HttpLogViewer`、`LogViewer`、`FrpLogs`，页面级 `DownloadSidebar` / `FolderSidebar` 侧栏、`CleanupTool` 文件树、`JavaManager` / `ResourcePackConverter` / `ScreenshotManager` / `NbtViewer` / `ArchiveManager` 限高列表、`AiModelSettings` 模型列表；另在 `Drawer.vue` 根节点单点标记，一次性覆盖全部挂载到 `#app-content` 的右侧抽屉（更新/崩溃/消息/提示/协议等）内嵌滚动，避免右下角按钮遮挡抽屉内操作。
- 启动时展示「本次更新日志」弹窗（对齐 PCL2 做法）：`App.vue` 初始化完成后比较 localStorage 记录的上次运行版本与当前版本，仅当版本升高时弹出一次 `UpdateLogDialog` 抽屉（右侧 560px，复用 `ReleaseTimeline` 渲染更新日志，底部含「完整更新日志」外链到 GitHub Releases）；弹窗前先写入当前版本保证只弹一次，「全新安装（无记录）/ 同版本 / 版本回退」均不弹。日志内容由 vite.config.ts 的 `updateLogPlugin` 在构建时基于 git 生成——仓库用 tag 管理版本（tag 名即版本号），取「上一 tag → 最新 tag」之间的全部 commit message 生成 Markdown（剥离 CI 跳过标记 `!c`），不依赖 CHANGELOG 也不把完整历史打进前端包；dev-api 新增 `molaunch.showUpdateLog()`（直接弹出弹窗）与 `molaunch.resetUpdateLog()`（清空已读记录，下次启动重新弹出）调试命令。

- 新增许可证同步工作流（`.github/workflows/license-sync.yml`）：根目录 `LICENSE` 作为唯一权威副本，向 main 推送更新或手动触发时，自动同步至 `src-tauri/LICENSE`、`src-tauri/updater/LICENSE`、`src-tauri/resources/LICENSE.txt` 并提交（提交信息带 `!c` 跳过 CI）；无差异时不提交，副本文件变化不会再次触发。
- 新增版本号同步工作流（`.github/workflows/version-sync.yml`）：推送 `v*` 标签或手动指定版本号时，逐文件检查 `package.json` / `package-lock.json` / `src-tauri/Cargo.toml` / `src-tauri/tauri.conf.json`，仅更新未同步到目标版本的字段并提交（提交信息带 `!c`）；全部一致时不产生提交。同步完成后调用 `release.yml` 激活发布流程，发布版本号由本工作流传入。
- 发布工作流（`.github/workflows/release.yml`）改由 `workflow_call` + 手动触发：不再直接监听 tag 推送，版本号改为从调用方输入读取，release 创建时显式指定 `tag_name`，避免 `github.ref` 不再是 tag 导致的问题；同时移除构建期「更新版本号」步骤（版本号统一由版本同步工作流保证）。
- 自动同步工作流的自动提交改用 `stefanzweifel/git-auto-commit-action`：默认使用官方 `github-actions[bot]` 提交者，无差异时自动跳过提交，不再手动执行 `git config` / commit / push。

- 弹窗统一抽屉化：删除居中的 `Modal.vue` 全局弹窗，新增 `MessageDrawer.vue`（复用公共 `Drawer.vue` 右侧抽屉，`render-in-place` 挂载到 `#app-content`，宽度 520）承载全部错误 / 警告 / 信息 / 成功提示 + 确认 + 输入框模式——`defineExpose` 对外接口与旧 Modal 完全一致，`utils/modal.ts` 及全站 95 处调用方零改动，启动「云端连接失败」等提示全部改为右滑抽屉；内置**消息队列**：同时触发多条（如并行任务连续失败）时排队依次展示，当前一条关闭（关闭按钮 / 遮罩 / ESC / 确认 / 取消）动画结束后自动滑出下一条，保证任何一条提示都不丢失；`KickConfirmDialog.vue`（踢出确认，选封禁时长）/ `LobbyJoinConfirmDialog.vue`（加入房间确认，内嵌整合包校验卡片）同步由居中 teleport 弹窗重构为右侧抽屉。保留居中模态的有：`UserAgreementDialog`（强制协议门禁）、`ExitConfirmDialog`（退出确认）、`DependencyConfirmDialog`（嵌套于 ResourceDetail 全屏详情之上，层级需高于 Drawer 固定 z-1000）、`CopyMessageDialog` / `VersionPickerDialog`（工具浮层）、Toast（轻提示非弹窗）。验证：`npx vue-tsc --noEmit`、ESLint（改动文件）通过。
- dev-api 新增抽屉消息测试命令（仅 dev 模式）：`molaunch.showError(title?, message?, details?)` / `showWarning` / `showInfo` / `showSuccess`（省略参数时使用样例数据，直接触发对应类型的右滑消息抽屉）、`molaunch.showConfirm(title?, message?)`（返回 `Promise<boolean>`）、`molaunch.showPrompt(title?, message?, defaultValue?)`（返回 `Promise<string | null>`，取消返回 null）、`molaunch.demoMessages()`（一次触发错误 / 警告 / 信息 / 成功 4 条，验证消息队列依次展示不丢失）；`help()` 文案与示例同步更新。
- 消息抽屉交互优化：详情区「查看详情」按钮常驻（文案固定不切换）且为可展开/收起开关——点击切换详情显隐，ChevronDown 图标随状态旋转 180°（`transition-transform duration-300`），展开使用 `grid-rows-[0fr/1fr]` 高度过渡动画（复用项目 MoLaunchIntro / VersionSelect 同款折叠模式）；多条消息排队时在**抽屉底部左侧**显示灰色小字「还有 x 个待看」（随队列实时递减，仅剩 1 条时消失），不再使用全局窗口角标。验证：`npx vue-tsc --noEmit`、ESLint 通过。
- 开屏动画 GIF 与录制脚本：`scripts/capture-splash.mjs` 借助 Puppeteer 逐帧录制 splashscreen 动画（门闩机制确保从动画起点开始捕获，录制时长 5000ms 完整覆盖进度条补满，共 94 帧），Pillow 合成 640×180 两份：`images/splash-loop.gif`（无限循环播放，总时长约 4.7s）与 `images/splash.gif`（播放一遍后在「就绪」完成画面停留 10s，不再跳回开头，供 README 展示）；README 顶部品牌展示由静态 logo 更换为 `images/splash.gif`，居中展示宽度 800，开头即可预览启动器开屏动画。
- `CachedImage.vue` 加载中状态改为动态 spinner：组件引入相对定位包裹层承载 attrs（class），img 未加载完成时 `opacity-0` + 内置 `animate-spin` 旋转图标（主色，SVG 圆环写法），加载完成过渡显示真实图片；失败或无 `src` 时仍渲染 fallback 插槽。默认占位图标由静态 CubeIcon 变为加载中动画，覆盖 4 处使用方（ResourceCard / ResourceDetailHeader / DependencyItem / DependencyInlineList）。验证：`npx vue-tsc --noEmit`、ESLint 均通过。
- 修复缓存图片首屏显示占位图标问题（`CachedImage.vue`）：远程图加载失败触发 fallback 后，即使后端异步缓存完成并 emit `image-cached` 事件，`failed` 标记也未重置，导致占位图标不消失、需切页重新挂载才恢复；现于 `onImageCached` 匹配到当前 pending URL 并切换本地缓存 URL 时同步重置 `failed = false`，缓存就绪后占位图立即恢复为真实图片。
- 新增正版购买提示：启动成功次数计数 + 永久忽略标记统一存入系统存储（改造 `storage/registry.rs` 为跨平台 KV：Windows 走注册表 `HKCU\Software\MoLaunch`，macOS/Linux 走全局共用文件 `~/.config/Molaunch/system.json` 且目录缺失自动创建即"有保底"）——非 Windows 上所有"注册表字段"（开发者模式、IgnoreTls、正版提示计数/忽略标记）收敛到同一个文件，避免逐功能建文件紊乱；计数与忽略标记经 `get_config`/`apply_config` 以 `launchCount`/`hintBuy` 暴露（分流到 `secure.rs`，不进 AppConfig）。前端启动成功后自增计数，命中阈值（对齐 PCL2 `ModLaunch.vb` 正版提示：3/8/15/30/50/70/90/110/130/180/220/280/330/380/450/550/660/750/880/950/1100/1300/1500/1700/1900）且非微软账号、中文系统时弹出「正版购买建议」（新增 `BuyHintDialog.vue`，与崩溃弹窗同构的右侧 Drawer 抽屉：计数横幅 + 购买理由 + 权益列表，底部「前往购买」打开官网并永久忽略、「暂不考虑」仅关闭）；dev-api 新增 `molaunch.setLaunchCount(n)` / `molaunch.showBuyHint()` 测试命令。
- 新增「去 GitHub 点 Star」提示（参照 PCL2 `ModLaunch.vb` 赞助弹窗，目标改为项目仓库而非爱发电）：与购买提示共用同一 `launchCount` 计数，独立阈值（对齐 PCL2 赞助：10/20/40/60/80/100/120/150/200/250/300/350/400/500/600/700/800/900/1000/1200/1400/1600/1800/2000）与独立忽略标记 `hintStar`（新增系统存储键 `HintStar`，`read_hint` 返回三元组，`ConfigPatch`/`ConfigSnapshot` 同步扩展）；启动成功后由统一入口 `maybeTriggerLaunchHints` 自增一次并分别检查两个提示，避免计数重复自增。新增 `StarHintDialog.vue`（右侧 Drawer 抽屉：「你已通过 MoLaunch 启动游戏 n 次」+ 恳请支持，「去点 Star」打开仓库并永久忽略、「暂不考虑」仅关闭）；`starHint.ts` 预留 apiServer 配置下发：`StarHintRemoteConfig` 结构（开关/阈值/仓库地址/标题/正文/按钮文案）+ `fetchRemoteStarHintConfig` 当前返回 null 用本地默认，后续 apiServer 就绪直接实现该函数即可覆盖，无需改动触发链路。dev-api 新增 `molaunch.showStarHint()` 测试命令。
- 抽屉组件优化：`Drawer.vue` 新增可选关闭反悔期（`undoMs` 默认 0 即关闭，由调用方按需传入毫秒数开启，如崩溃弹窗传 3000）——面板滑出后节点保留、对应边缘露出可点击的恢复小 tab（跟随主题色 `--color-primary-*`，带滑入/收出位移动画与剩余秒数倒计时气泡「还有 x 秒后关闭」，先播完 tab 消失动画再真正卸载），点击立即重新展开、超时后真正卸载；配合 `unmountOnClose` 则关闭即卸载（`AskUserDialog.vue` 取消提问语义保持 `unmount-on-close`，避免恢复空抽屉）。
- 崩溃弹窗优化：解决方案多行展示增强——AI 生成的「建议：1. xxx；2. xxx」编号列表按「。建议：」/「建议：N.」编号/「；」多级拆分，每条方案独立成行，不再与「建议：」挤在同一行。
- 提示抽屉整合：`BuyHintDialog.vue` / `StarHintDialog.vue` 合并为统一的 `HintDialog.vue` 单抽屉——两个提示（正版购买 / 点 Star）共用同一个右侧 Drawer，同时触发（如 dev-api 依次调用 `molaunch.showBuyHint()` + `molaunch.showStarHint()`）时不再叠加渲染两个抽屉，而是标题栏出现分段切换器（复用项目 `SegmentedButtons.vue`），内容区横向滑动切换两页（`translateX` 0.3s 动画）；各页均保留「你已通过 MoLaunch 启动游戏 n 次」计数横幅；`buyHint.ts` / `starHint.ts` 的实例接口相应改为 `showBuy` / `showStar`，`App.vue` 改为挂载单个 `HintDialog` 并注册给两处 ref。
- 分段按钮组件复用化：`SegmentedButtons.vue` 改为基于项目 `Button.vue` 渲染（不再手写原生 `<button>`），选中态通过 Tailwind utilities 层类名覆盖（优先于 components 层 `.btn-*`），API 与外观保持不变。
- 使用协议与免责声明：新增 `DisclaimerDialog.vue`（右侧 Drawer 抽屉，按 `kind` 区分联机 / 实验性功能 / 工具三份协议）——通用声明（本启动器及作者不承担使用后果）+ 联机说明（房间管理仅经 MoLaunch 服务器，不涉及流量中转与内容传播；经国内外 TURN 服务器获取网络类型并使用 P2P 创建虚拟 TUN 网络；FRP 隧道由第三方提供、每家厂商各有其用户协议需用户遵守）+ 实验性说明（AI 对话发送至自行配置的模型端点、数据外发风险自担）+ 工具说明（本地为主、外部下载/网络工具请求第三方）+ 合规提醒；`Online.vue` / `Experimental.vue` / `Tools.vue` 进入时展示，改为「每日一弹」——点击「我已知悉并同意」后写入 localStorage，当天不再弹出（次日重新提醒），记录逻辑收敛在 `utils/disclaimer.ts` 的 `hasAgreedToday` / `markAgreedToday`。
- drawer 强制关闭：协议抽屉设置 `closable=false` / `mask-closable=false` / `esc-to-close=false`，必须点击按钮才能关闭，规避用户未确认而绕过协议。
- 联机页面可进入 + 云端离线封禁：`TopNavLayout` 导航「联机」不再因云端不可达而置灰禁用，始终可点击进入；云端离线时由 `useOnlineNav` 将「房间管理」「联机大厅」置为封禁态（`NavSidebar` 新增 `sealed`，灰色 + 锁图标，点击仍触发 `@click`），`Online.vue` 拦截点击弹窗告知原因（复用 `showWarning`，展示 cloudError）；设备页注册 / 登录 / 设备信息卡片叠加新组件 `SealedOverlay` 封条遮罩（点击弹窗告知原因），NAT 检测（第三方 TURN/STUN）仍可用；已加入房间时（P2P 仍工作）保留「房间详情」，云端断开自动切回「设备」；状态徽章新增「云端离线」。删除废弃的 `CloudDisconnectedMask.vue`。
- 协议抽屉交互补全：`DisclaimerDialog.vue` 点击「我已知悉并同意」后追加 success toast「已确认使用协议，今日不再提醒」；跳往其他页面时若抽屉仍开着（未确认）追加 warning toast「已放弃确认使用协议，下次进入将再次提醒」（`onBeforeRouteLeave` 守卫，联机 / 实验性 / 工具三处共用同一组件即同时生效）。
- 许可协议展示优化（`LicenseTab.vue` 重写）：标题栏不吸顶常驻、正文随外层设置容器统一滚动（撤回 sticky + 内层独立滚动方案），GitHub 原文外链置于标题栏右侧；正文不引入 markdown（CommonMark 加粗受 flanking 规则限制，`**“许可方”**` 等引号加粗会原样渲染），改为极简渲染器——先 HTML 转义防注入，再对短引号内容（产品名 / 法律术语，≤20 字符）直接转 `<strong>` 加粗 + `letter-spacing` 字距（scoped `:deep` 限定作用域），段落按空行拆分为 `<p>`、段内换行保留 `<br />`，长引号保留原文；底部说明补充「本许可证只携带此版本构建时的许可证，不排除后续版本更迭许可证更新的情况，具体以仓库许可证版本为主」。
- 新增《用户协议》全局门禁（首次启动须同意后才能使用，参照 PCL2 首启协议）：前端 `utils/userAgreement.ts` 自设计简短协议内容（引言 + 账号/软件/隐私/变更四要点）并预留 apiServer 远端下发（`fetchRemoteUserAgreementConfig` 当前返回 null，后续就绪即覆盖本地默认、触发链路无需改动）；完整条款外链服务条款 `https://molaunch.moiu.cn/terms-of-service.html` 与隐私政策 `https://molaunch.moiu.cn/privacy-policy.html`。同意状态持久化到系统存储（新增 `UserAgreed` / `UserAgreedVersion` 键，Windows 注册表 / 其他系统全局共用文件，与 `launchCount`/`hintBuy`/`hintStar` 同分流），经 `get_config`/`apply_config` 以 `userAgreed`/`userAgreedVersion` 暴露；协议内容有实质更新时自增 `USER_AGREEMENT_VERSION`（当前 1），已同意版本低于当前版本即重新要求同意。新增 `UserAgreementDialog.vue`（强制弹窗：无关闭按钮 / 无遮罩点击 / 无 Esc，Teleport 到 body 以 `z-[10050]` 覆盖启动加载遮罩与普通弹窗，同意按钮「同意并继续」，正文含引言 + 要点列表 + 条款外链，失败态可重试）；`App.vue` 启动 `initApp` 开头触发 `maybeRequireUserAgreement()` 门禁检查（fire-and-forget + 失败静默忽略，不阻塞其余初始化）；dev-api 新增 `molaunch.showUserAgreement()` / `molaunch.resetUserAgreement()` 测试命令。
- 联机大厅刷新按钮靠右：`LobbyBrowser.vue` 搜索栏中刷新按钮（Tooltip 包裹的图标按钮）加 `ml-auto`，与搜索框 / 加载器筛选拉开到行尾对齐。
- 设置 - 更多新增「许可协议」子页签：`SettingsMore.vue` 顶部子菜单在「鸣谢」与「教程」间插入 `license` 项（复用 `ScaleIcon`），新增 `more/LicenseTab.vue` 展示项目许可协议全文（加载 / 失败 / 正文三段式，正文 `whitespace-pre-wrap` 排版 + GitHub 原文外链）。
- 许可协议「副本引用」设计：项目根目录 `LICENSE` 为唯一权威副本 → `build.rs` 新增 `sync_license()`，每次构建自动将其同步到 `src-tauri/resources/LICENSE.txt`（内容无变化不写盘避免无意义重编译，根 LICENSE 变更经 `rerun-if-changed` 触发重新构建）→ `resources.rs` 在 `embedded_text` 注册 `"LICENSE.txt"`（include_str! 编译期嵌入二进制，确保每次打包都包含最新协议）→ 新增 `get_project_license` IPC 命令（`commands/system/about.rs` + `system/manager/dispatcher.rs`），前端经 `about.ts` 的 `getProjectLicense` 读取展示。同时提交 `resources/LICENSE.txt` 现网快照，保证资源目录自包含。
- 《用户协议》弹窗交互补全（`UserAgreementDialog.vue`）：弹窗改横向长方形比例（`max-w-3xl` + 内容区 `max-h-[min(56vh,26rem)]`，贴合启动器长条窗口）；底部新增已读确认 `Checkbox`「我已阅读并同意本《用户协议》」（未勾选时禁用「同意并继续」按钮，每次打开重置未勾选状态）；《服务条款》/《隐私政策》外链按钮回归正文原位置，与灰色「需同意后方可继续使用」提示文案置于同一行、按钮在文案左侧（整体靠右 `justify-end`、底部对齐 `items-end` 不与文案垂直居中、文案 `whitespace-nowrap` 不换行），标题栏还原为「标题 + 版本」不被挤动，底部保持单行「已读勾选 + 取消/同意」布局；新增「取消」挽留功能——点取消先进入二次确认态（「确定要退出 MoLaunch 吗？」+「返回上一步」/红字「确定退出」），确认后才 `invoke('request_exit')` 退出程序（后端清理 frpc/TUN 后退出进程，不经 closeBehavior 关闭询问 / 托盘分流）；通过在弹窗遮罩上挂 `data-tauri-drag-region`（与 TopNavLayout 顶部拖拽区同机制）修复弹窗弹出后无法拖动窗口的问题。
- 新增开屏启动动画（双窗口 splashscreen 方案，解决启动加载库时的空白卡顿感）：`tauri.conf.json` 配置两个窗口——`splashscreen`（640×200 无边框透明置顶居中、跳过任务栏，先加载 `public/splash.html`）与 `main`（主窗口 `visible:false` 后台加载，避免启动库加载期间白屏）。开屏页从 `docs/Run-html/run.html` 设计稿优化而来（去掉 `✅` 等 Emoji、品牌名统一 `MoLaunch`、品牌色 `#165dff` 贯穿图标描边/进度条/光标闪烁、`scene` 容器 640×180 内容左置垂直居中、副标语改为 `.show` 类过渡 3200ms 淡入）；全局 CSP `script-src 'self'` 不允许内联脚本，开屏逻辑抽为外部 `public/splash.js`（打字机标题 + 进度条 2400ms + 状态文案，4600ms 兜底 `window.__TAURI?.core.invoke('frontend_ready')`，浏览器预览环境静默）。后端新增 `splash.rs` 的 `frontend_ready` 命令（幂等：关闭 splashscreen 窗口 + 显示/还原/聚焦 main 窗口）；`Cargo.toml` 启用 `macos-private-api` feature、`app` 块启用 `withGlobalTauri` + `macOSPrivateApi`（透明窗口 macOS 必需）。前端 `utils/splash.ts` 的 `notifyFrontendReady` 在 `App.vue` 挂载时调用，保证开屏至少展示 4.6s（`SPLASH_MIN_MS` 对齐动画总时长，避免播一半被切走），与 splash.js 兜底双保险。开屏图标改用前端库品牌 logo：`src/assets/logo.svg`（透明无边框纯 logo，内置浮动 / 扫光 / 脉冲 / 闪烁动画）同步一份到 `public/logo.svg` 供静态开屏页 `<img>` 引用（splash 页不经 Vite 打包，无法直接引用 `src/assets` 资源；CSP `img-src 'self'` 放行同源加载），`<img>` 内 SVG 自带 `<style>` 动画不受外部 CSP 影响仍生效，并去掉原简笔画图标的圆角容器背景 / 边框直接展示 logo；`run.html` 设计稿则相对引用源文件 `../../src/assets/logo.svg` 保持预览始终最新。开屏可读性修复：窗口背景由全透明改为品牌浅色渐变（白→淡蓝 + 左上品牌蓝光晕，`radial-gradient` + `linear-gradient` 双层叠加），深色文字 / 深蓝 logo 不再受桌面背景干扰，同时 `splashscreen` 窗口配置补 `backgroundColor: "#eef3ff"`（与渐变底色一致，覆盖 Windows 透明无边框窗口顶部 1px 白线的 WebView2 边缘底色泄漏），辅助文字（`info` / `sub`）颜色加深一档保证对比度。
- 验证：`npx vue-tsc --noEmit`、`npx eslint`（改动文件）、`cargo check --manifest-path src-tauri/Cargo.toml`、`npm run build`（确认 dist 生成 `splash.html`/`splash.js`）均通过。
- 更新弹窗抽屉化（`UpdateDialog.vue`）：由自绘 teleport 模态框重构为项目公共 `Drawer.vue` 右侧抽屉（`render-in-place` + `popup-container="#app-content"`，与崩溃 / 提示抽屉同模式），宽度 560 贴合设置内容区；关闭约束映射到 Drawer 能力——`closable` / `mask-closable` / `esc-to-close` 均绑定 `canClose`（强制更新 / 下载中 / 安装中禁关，`@update:visible` 桥接 `closeDialog()`），不再手写 ESC / 遮罩 / 关闭按钮逻辑；更新日志时间线（`ReleaseTimeline` + `ReleaseTimelineItem`）原样保留，"最新版本行固定 + 日志区独立滚动"布局不变；`checking / downloading / installing / error / done` 各状态渲染、`update-download-progress` 进度事件监听、`available / error` 底部按钮（稍后 / 立即更新、关闭 / 重试）均保留。`modal-shell` 公共样式未删（`ModUpdateDialog` / `SkinManager` / `ProfileSelectModal` / `AiLogAnalyzer` 仍在使用）。
- 开屏图标回退为 SVG：PNG 显示效果不佳，重新从 `src/assets/logo.svg` 同步一份到 `public/logo.svg` 供静态开屏页 `<img>` 引用（splash 页不经 Vite 打包无法直接引用 `src/assets`），`public/splash.html` 改回 `logo.svg`、`docs/Run-html/run.html` 设计稿改回 `../../src/assets/logo.svg`；`index.html` favicon 维持 `href="/src/assets/logo.svg"` 引用 assets 下的 svg 不变。
- 开屏进度条留余量：动画阶段最多走到 92%（`MAX = 92`）即停住（状态文字停在「检查更新...」，不再显示「就绪」），避免 Tauri 初始化慢时"进度虚满仍在等待"的误导；真正切换窗口前（`frontend_ready` 兜底 4600ms 路径）才 `finish()` 补满至 100% 并显示「就绪」，真实前端就绪路径由窗口切换直接接管，无需补满。`run.html` 设计稿内联逻辑同步。
- README 重写为客观工程化风格（面向开发者 / 用户，去除产品营销化表述）：标题上方新增居中展示的 `images/logo.svg` 品牌 logo（`<p align="center">` 包裹，宽 200）；标语与简介改为中性描述，功能特性按模块客观罗列（版本管理 / 下载 / 账户 / 皮肤 / 联机 / 工具 / AI 助手 / 插件 / 其他），删除「快速启动，拒绝卡顿」等宣传句式与「正版购买建议与赞助提醒」等运营内容；Tauri 2 双进程技术架构（mermaid 分层架构图 + 前端 / 后端按域说明 + 项目结构树），开发构建部分精简为命令速览；顶部新增 shields.io GitHub 动态徽章（stars / forks / issues / last commit / contributors，`logo=github` 官方图标，品牌色 165dff）；新增「界面预览」板块，按 `images/` 目录预留 8 个截图位并以 `001.png` ~ `008.png` 序号命名（启动器主页 / 版本下载页 / 社区整合包 / 联机大厅 / 种子地图 / AI 聊天 / 皮肤披风弹窗 / 设置，2 列 HTML 表格布局，每格标注对应文件名与截图说明）；技术架构-前端注明 Button / Input / Select / Drawer / Slider 等核心组件参考借鉴 Arco Design Vue（复刻改写为 Vue SFC + Tailwind，文件顶部带 MIT 版权声明注释）与 Element Plus Icons SVG path 复用；新增「鸣谢」板块（特别感谢 Arco Design Vue / Element Plus Icons / PCL2 并附仓库链接——其中 PCL2 文案注明其为一款被广泛使用的 Minecraft 启动器、MoLaunch 前期从零开发、启动 Minecraft 相关逻辑参考其实现；核心依赖列 Vue / Tauri / Tailwind / Heroicons / skinview3d / Cubiomes / OpenLayers / Tokio / Reqwest，指向 licenses.txt 完整清单，保留 PCL2 独立第三方声明）；版本徽章更新至 0.3.5-rc1，保留第三方免责声明与 MoLaunch 分发有限许可证核心限制。

## [0.3.4] - 2026-08-07

- 修复 dev 重建死循环：新增 `src-tauri/.taurignore`，排除嵌套 Cargo 项目 `updater/target/`，避免其构建产物被 tauri dev 监听后与 `build.rs` 的 updater 自动构建互相触发无限"Rebuilding application"。
- 验证：已对照 tauri-cli v2.11.4 源码确认 `.taurignore` 的加载与过滤链路（`build_ignore_matcher` 仅在发现 `.taurignore` 时构建过滤规则）；需重启 `npm run tauri dev` 生效。

- 崩溃分析弹窗重构：`CrashDialog.vue` 由 PCL2 模拟样式重写为右侧 `Drawer` 抽屉布局（复用项目抽屉组件，与 AI 提问抽屉同构）。内容分区设计：按崩溃类别着色的原因横幅（左侧色条 + Tag）、主色信息框的解决方案、崩溃报告文件行（等宽路径 + 打开）、可折叠日志详情（深色代码块与日志查看器一致）；替换插件 shell 直连为统一 `openPath`；`CrashCategory`/`CrashInfo` 类型收敛至 `src/types/version.ts`（补齐后端 10 种分类，消除 3 处重复定义）。
- 验证：`npx vue-tsc --noEmit`、`npx vite build`、`npx eslint`（改动文件）均通过。

- 崩溃弹窗体验优化：解决方案支持多行展示——规则引擎建议以换行分行、AI 长文本按「。建议：」「；」拆分，保证每条方案独立成行；日志详情展开改动画折叠，复用 `Collapse.vue`（grid-rows 0fr→1fr 平滑高度动画），替代原布尔显隐切换。
- 验证：`npx vue-tsc --noEmit`、`npx eslint`（`CrashDialog.vue`）均通过。

- 依赖与版权清单审计：根据 `package.json`、`Cargo.toml` 及本地包元数据补齐前端运行时/开发工具链和 Rust build/条件依赖；修正 `marked`、`netstat2` 版本及 `notify`、`tun-rs`、`md5`、`rustls-native-certs` 等许可证记录，补充缺失的直接依赖与版权来源。

### 重构：将崩溃分析模块的 2 个内联测试块迁移至 `analyze_tests.rs` / `detector_tests.rs`，并统一 `layout.rs`、`modpack_meta.rs` 复用 `utils::hash::sha256_hex`；前端新增 `formatDateTime`，统一 5 处日期时间格式化调用，保持原有本地时区与无效值兜底行为。
- 后端头部注释治理：精简 20 个 Rust 文件的冗余模块头注释至 5 行以内，移除子模块罗列、历史背景、协议示例和实现细节；未修改业务逻辑。
- 验证：各批次均执行 `cargo check --manifest-path src-tauri/Cargo.toml` 与 `git diff --check`，通过。

- 前端 TypeScript 头部注释治理：精简 `click-outside.ts`、图标、FRP 链接、Markdown 及 API 文件共 7 个文件，均控制在 8 行以内；未修改业务逻辑。
- 验证：各批次均执行 `npx vite build` 与 `git diff --check`，通过。

- 超长文件拆分：`useAiChat.ts` 从 632 行拆为状态组装、消息、流式处理和类型模块；`commands/frp/api_spec/executor.rs` 从 624 行拆为请求、DTO 映射和执行编排模块，保持既有导出与调用链。
- 验证：`npx vue-tsc --noEmit`、`npx vite build`、`cargo check --manifest-path src-tauri/Cargo.toml`、`cargo test --manifest-path src-tauri/Cargo.toml`（212 passed）及 `git diff --check` 均通过。

- 文件拆分：`ReleaseTimeline.vue` 拆出版本说明解析与单版本条目组件；`TunnelCreateForm.vue` 拆出 `useTunnelCreateForm` 表单逻辑；FRP `install.rs` 拆出文件安装、ZIP 安全解压与卸载模块，保持调用链和行为不变。
- 验证：相关批次执行 `npx vite build`、`npx vue-tsc --noEmit`、`cargo check --manifest-path src-tauri/Cargo.toml`、`cargo test --manifest-path src-tauri/Cargo.toml`（212 passed）及 `git diff --check`，均通过。

- 文件拆分：FRP `tunnel.rs` 拆出参数、CRUD/持久化、配置生成与导入；`process/start.rs` 拆出准备、启动、监控模块；AI 设置页拆出端点、上下文和模型设置子组件，保持 API 与调用行为不变。
- 验证：相关批次执行 `cargo check`、`cargo test`（212 passed + 1 文档测试）、`npx vite build`、`npx vue-tsc --noEmit` 与 `git diff --check`，均通过。

- 文件拆分：后端 `http.rs` 拆出客户端构建与 TLS；实验聊天 `chat.rs` 拆出发送/重试/编辑/提问流程；崩溃分析 `rules.rs` 拆出规则类型与静态规则表，保持公开 API 和行为不变。
- 验证：相关批次执行 `cargo check`、`cargo test`（212 passed + 1 文档测试）及 `git diff --check`，均通过。

- 文件拆分：联机房间动作拆分为 CRUD/刷新/参与者模块；FRP 前端 API 拆为 core/provider/tunnel/auth/public-server 域；拖拽处理器拆出文件分发逻辑，保持现有导出和行为。
- 验证：相关批次执行 `npx vite build`、`npx vue-tsc --noEmit` 与 `git diff --check`，均通过。

- 文件拆分：导航配置、AI 日志分析、SQLite 连接/迁移/表访问、FRP API HTTP 传输/重定向、应用配置模型/默认值/路径/helper 分别模块化；保留公开 API、配置链路与安全行为。
- 验证：相关批次执行前端构建/类型检查、`cargo check`、`cargo test`（212 passed + 文档测试）、`cargo fmt`（配置模块）及 `git diff --check`，均通过。

- 文件拆分收尾：实验性 API 按 action 域拆分；Agent 工具拆为注册表与执行器，当前审计范围内前端/后端超 300 行文件已完成收敛，保持 action、tool schema 和调用 API 不变。
- 验证：相关批次执行 `npx vite build`、`npx vue-tsc --noEmit`、`cargo check`、`cargo test`（212 passed）及 `git diff --check`，均通过。

- 复杂度治理：拆分下载重试/校验、启动配置、启动监控、AI SSE 流式聚合、版本 JSON 查找/复制重试职责；保持下载进度回滚、启动参数、流式顺序和安装重试行为。
- 验证：相关批次执行 `cargo fmt`、`cargo check`、`cargo test`（212 passed + 文档测试）及 `git diff --check`，均通过。

- 架构与复杂度治理：online 信令/TUN 业务分发从 `utils` 迁入 `commands/online/manager`，解除反向依赖；同时完成下载、启动、AI 流式和版本加载等高复杂度函数拆分。
- 当前复扫：前端/后端超 300 行文件均已清零；Rust 头部注释超 5 行、TypeScript 头部注释超 8 行均已清零（`element-icons.ts` MIT 许可证按约定豁免）。
- 验证：相关批次执行 `cargo fmt`、`cargo check`、`cargo test`、`npx vite build`、`npx vue-tsc --noEmit` 与 `git diff --check`，均通过。

- 最终验证：全量复扫确认前端/后端超过 300 行文件均为 0；`npx vite build`、`npx vue-tsc --noEmit`、`cargo check --manifest-path src-tauri/Cargo.toml`、`cargo test` 与 `git diff --check` 均通过。Rust 仍有既有 `prepare_turns` dead_code 警告，不影响构建；前端仅有既有 chunk 体积警告。

- 清理实验聊天模块未使用的 `prepare_turns` 函数及其无效导入，消除 Rust `dead_code` 编译警告；未改变实际聊天调用链。

- 修复 AI 工具调用后提前结束：当工具执行成功但模型下一轮返回空正文时，允许基于已回填的工具结果重试一次最终回答，并记录告警日志；避免工具调用成功后直接结束聊天。

- 许可证重新设计：新增根目录 `LICENSE`，采用 MoLaunch 分发有限许可证，禁止项目及二次开发版本商业使用，要求二次开发公开完整源代码、明确第三方来源并遵守名称限制；同步 README、关于页和第三方版权清单，明确自有代码与第三方依赖的许可证边界。

- 许可证路径与 README 标准化：后端主 crate 和 updater crate 各自使用本目录 `LICENSE`，移除 Cargo 元数据中的 `../` 路径；README 重写为标准化产品文档，并明确第三方项目版权与许可证均记录于 `src-tauri/resources/about/licenses.txt`。

### 新增

- 背景：将账号 DTO/serde 辅助函数与 FRP 全隧道停止逻辑从聚合入口归位到已有职责模块，避免 `mod.rs` 混入业务实现。
- 改动：`commands/auth/account/mod.rs` 改为复用 `info.rs` 导出类型；`commands/frp/process/mod.rs` 将 `stop_all_tunnels` 迁移至 `stop.rs`；Tauri 转发函数保留技术例外。
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过。

### 新增

- 完成 `src-tauri/src` 全部 117 个 `mod.rs` 入口职责审计：102 个纯入口、15 个 A 类聚合入口夹带逻辑、0 个 B 类单文件模块；审计报告写入 `docs/fix-debug/07-modrs-entry-only.md`，本次未修改 Rust 源码。

- 工具页「Java 管理」重构为三个独立区块（`JavaPage.vue` 拆分，均带 `data-toc-card` 目录锚点；区块顺序：Java 下载器 → 已安装版本 Java 环境检测 → Java 运行时列表）：
  - **Java 运行时列表**（`JavaManager.vue`）：移除版本选择交互（Java 切换统一收敛到「设置 → 启动设置」），改为纯展示列表 + 重新检测，并用 `AlertV2` 说明定位，避免与设置页职责重复
  - **已安装版本 Java 环境检测**（新增 `JavaEnvCheck.vue`）：逐版本调用 `getVersionGameVersion`/`getVersionLoaderInfo` 解析 MC 版本与加载器 → `getJavaRequirements` 查询需求（min/max/recommended）→ 复用 `isJavaCompatible` 判断系统已装 Java 是否满足；不满足时显示一键预下载按钮（复用 `JavaDownloadBar`，目标版本取 recommended || min），下载后自动刷新重新校验
  - **Java 下载器**（新增 `JavaDownloader.vue`）：预设固定为 Mojang 官方 Runtime 可下载的三档（21/17/8）+「自定义」档（自定义作为快速选择里的一个选项，选中后才显示输入框，输入框宽度自适应不限制；复用 `isJavaMajorValid` 校验 1~2 位纯数字、8~26 区间、无特殊符号，上限参考 Adoptium 最新可用版本 26）；下载源固定为 Mojang 官方 Java Runtime（piston-meta / piston-data，可随镜像设置走 BMCLAPI）并在界面注明；下载目录强制固定为 `%APPDATA%\.minecraft\runtime\`，用 `AlertV2` 说明原因（与官方启动器一致、跨游戏目录共享、不受版本隔离影响）；「下载 Java X」按钮置于「目标版本」同行右侧；非官方版本（11/16/18/20/22~26 等自定义版本）提示官方 Runtime 可能未提供；`api/java.ts` 新增 Java 版本元数据常量（`MIN_JAVA_MAJOR`/`MAX_JAVA_MAJOR`/`LTS_JAVA_MAJORS` 及 `isLtsJavaMajor`，参考 Adoptium 版本分布）
  - `JavaDownloadBar.vue` 重构为按 `targetMajor` 驱动（原 `javaReqs` prop 改为 `targetMajor`，下载目标由父组件指定），供环境检测与下载器两处复用；`api/java.ts` 新增可复用纯函数 `describeJavaRequirement`（需求描述）/ `isJavaMajorValid`（版本号校验）/ `hasOfficialRuntime`（官方档判断）

- 新增公共组件 `Trigger.vue`（弹出触发器）与 `Drawer.vue`（抽屉），参考 Arco Design Vue 的定位/箭头、滑入遮罩设计思路，**API 为项目自定义精简版**（沿用 `Button`/`Input` 移植约定：头部 MIT 来源注释 + 样式分离到同目录 `Trigger.css` / `Drawer.css`）：
  - `Trigger.vue`：4 种触发方式（hover 可延迟 / click / focus / contextMenu）、12 向弹出位置（top/tl/tr、bottom/bl/br、left/lt/lb、right/rt/rb）、可选箭头（`showArrow`，随触发元素中心对齐）、`hoverStay` 移入弹层保持显示、Teleport 到 `popupContainer`（默认 body）+ fixed 定位 + 视口边界钳制；受控 `v-model:visible` / 非受控 `defaultVisible` 双模式；点击外部关闭复用 `@/utils/click-outside`；`#content` 插槽外观由调用方 `contentClass` 控制
  - `Drawer.vue`：4 个滑出方向（left/right/top/bottom）、`width`/`height`、`closable`/`mask`/`maskClosable`/`escToClose`、`header`/`title`/`footer` 插槽、关闭动画结束后卸载节点；受控 `v-model:visible` / 非受控 `defaultVisible` 双模式
- AI 聊天浮层统一接入新 `Drawer` 组件（右侧抽屉，替代手写 `getBoundingClientRect` 定位 + teleport 悬浮卡片）：
  - `ChatHeader.vue` 思考设置：图标入口改为切换右侧抽屉（遮罩点击 / X / ESC 关闭），删除手写悬浮窗定位与 document 点击外部关闭监听，关闭按钮原生 `title` 换为 `Tooltip` 组件
  - `AskUserDialog.vue` AI 提问：右下角悬浮卡片改为右侧抽屉（`width=400`），问题/选项置于内容区、自定义输入 + 取消/提交放入 `#footer` 插槽，X / ESC / 点击遮罩关闭统一视为取消提问
- 抽屉与滑块细节修复：抽屉右上角 X 关闭按钮加大（28px / 图标 16px）更醒目；思考设置抽屉恢复遮罩（点击左侧遮罩关闭）；`Slider.vue` 档位标签（低/中/高）下移留出 8px 间距、标签层 `pointer-events: none` 不再拦截滑块拖动，首尾标签改用 0% / -100% 位移避免溢出滑块外；思考程度滑块在抽屉中不再叠加 marks 文字（档位由右侧文本标签展示）

- 抽屉定位修复（本轮）：`Drawer.vue` 新增 `render-in-place` 就地渲染模式（Teleport `disabled` + `absolute` 铺满最近定位祖先），思考设置抽屉与 AI 提问抽屉启用后只出现在 nav 下方内容区（与 `DragOverlay` 同策略），抽屉右上角 X 关闭按钮不再被最高层级 nav（`z-[10002]`）遮挡；同时移除思考程度滑块上叠加的 `:marks` 低/中/高文字（不再与滑条挤在一起变形，档位改由右侧文本标签展示）
  - 挂载方式系统性改造：`App.vue` 内容容器打上全局挂载点标记 `id="app-content"`（nav 下方，与 `DragOverlay` 同挂载位置），`TopNavLayout.vue` 顶部 nav 打上 `id="app-nav"` 标记；`Drawer.vue` 的就地渲染模式支持 `popup-container` 指定该内容容器，抽屉 teleport 进 `#app-content` 内 `absolute` 铺满，从布局上物理避开 nav（不再依赖页面链上的 relative）；未来所有全屏/全局挂载组件均可复用 `#app-content` 挂载点避免被最高层级 nav 遮挡
  - 修复抽屉打开无动画（关闭有动画）：根因是 `v-if` 位于 `<transition>` 外层，每次打开时 transition 为全新挂载、其子元素属于初始渲染，Vue 默认不触发进入动画；遮罩与面板两个 transition 均补充 `appear`，打开时遮罩淡入 + 面板滑入，与关闭动画对称

- 滑块拖动平滑化：`Slider.vue` 新增 `snap` 吸附档位能力（拖动中 step 内部强制 1、thumb 连续平滑跟随，松手时吸附到最近档位再提交），思考程度滑块由三档跳跃改为平滑拖动后吸附到 低/中/高；`DragOverlay.vue` 挂载注释统一引用 `#app-content` 挂载点规范

- 聊天框修正：`#md-icon`（markdown 图标映射）改为 `vertical-align: middle` + `align-items/justify-content: center` 垂直居中，修复图标偏文字下方不对齐；移除消息列表下方的独立工作状态提示条（「正在进行下一步…/等待你的回答」），等待提问由 AskUserDialog 抽屉承载，工具间隙等空内容状态由 AI 回复框内部兜底承担（ChatMessageItem 无内容时显示「正在进行下一步…」）
- 等待提示兜底（本批）：`ChatMessageItem` 空正文兜底按阶段切换文案——刚发送等待模型应答显示「正在思考如何回答…」，工具调用/提问过渡期（`ExperimentalChat` 在 `waitingAsk` 或 `toolCalls` 激活时传入 `:waiting`）显示「正在进行下一步…」；正文输出后不再显示任何等待提示（仅保留流式光标），消除工具调用或提问等待期间明显的静默卡顿感
- 工具过渡期兜底修复（根因）：`useAiChat.ts` 工具调用 `running` 事件原仅清空 `streamingMsg.content`、残留旧 `reasoningContent`，而空正文兜底条件依赖 `!reasoningContent`，导致工具执行完成、SSE 文本流出前的过渡期回复框既不显示正文也不显示提示；现每次工具切入时一并清空思考内容，使回复框回到空正文态、由「正在进行下一步…」兜底覆盖整个过渡期，待新思考流输出后思考区块自动接管、提示消失；同时移除正文后追加提示逻辑与失去消费者的 `waitingNext` 状态

#### 实验性功能（AI 聊天 / Agent 工具 / 日志分析，默认关闭）

- 背景：将 AI 相关能力与日志分析迁入独立的「实验性」入口，默认不暴露；用户需在「设置 → 进阶设置」手动开启后才显示入口并初始化本地 SQLite 聊天存储，避免为未启用用户带来资源开销
- 本批前端修复：进入 AI 聊天且没有历史会话时自动创建首个会话；直接输入并发送无需先点击「新建对话」；统一会话/消息 API 字段为 camelCase（`conversationId` / `createdAt` / `toolCallsLog`），修复 `missing field conversationId`；Enter 发送增加 IME 组合输入保护（`isComposing` / `keyCode=229`），避免中文输入法被误触发
- AI 并入实验性统一分发（本批修复）：
  - 原独立 `commands::ai::ai_manager` Tauri 命令已移除，5 个 AI action（analyze_crash / check_status / save_config / load_config / list_models）全部并入 `experimental_manager` 分发，且统一受 `experimental_enabled` 开关保护（未开启时返回错误）
  - `commands/ai/manager.rs` 改为纯实现库（`analyze_crash` / `check_status` 为 `pub(crate)` 复用函数），`commands/ai/mod.rs` 不再声明 Tauri 命令
  - `src/utils/api/ai.ts` 的 `aiManager` 改指向 `experimental_manager`；`src/utils/dev-api.ts` 的 `molaunch.ai` 同步指向
  - 修复 `missing field baseUrl` 参数解析错误：后端 `AiConfig` / `AiProbeParams` / `AiStatusResult` 为 camelCase，前端 `ai.ts` 接口与 `SettingsAi.vue` 表单原为 snake_case 导致反序列化失败；现已统一为 camelCase（`baseUrl` / `apiKey` / `timeoutSecs` / `defaultModel`）
  - `ExperimentalLog.vue`：顶部自定义 amber 提示改为项目 `AlertV2` 组件；移除 `max-w-3xl` 限宽，崩溃日志分析框占满内容区，消除右侧大面积空白
- 开关与配置链路：
  - `AppConfig` 新增 `experimental_enabled` 字段（默认 `false`），INI 键 `[Experimental] enabled`
  - 贯通 `load.rs` / `save.rs` / `ConfigSnapshot` / `ConfigPatch` / `build_snapshot` / `apply/fields.rs`（`apply_experimental`）/ `flow.rs`，前端 `ConfigSnapshot` / `ConfigPatch` 同步新增 `experimentalEnabled`
  - 首次开启时惰性调用 `commands::experimental::db::ensure_initialized()` 创建 `experimental/chat.db` 并建表（幂等）；关闭仅隐藏入口，不删除数据
- 顶部导航与页面：
  - `TopNavLayout.vue`：新增条件渲染的「实验性」入口（`BeakerIcon`），默认隐藏，开启后显示在「设置」之前
  - 新增共享组合式函数 `src/composables/useExperimental.ts`：统一读取配置与监听 `experimental-mode-changed` 事件，供导航、页面守卫与设置开关复用
  - 新增 `src/components/settings/ExperimentalToggle.vue`：进阶设置中的自包含开关卡片（仿 DevModeToggle 模式）
  - 新增路由 `/apps/experimental` 与 `src/views/Experimental.vue`（子导航：AI 聊天 / 日志分析 / AI 设置；未开启时展示守卫空状态）
- 后端命令模块 `src-tauri/src/commands/experimental/`（经 `experimental_manager` IPC 注册）：
  - `db.rs`：SQLite 惰性初始化与会话/消息 CRUD（`rusqlite` 0.31 bundled 静态编译，运行时无需动态库）
  - `types.rs`：会话/消息/聊天入参出参类型
  - `manager.rs`：action 分发（create/list/delete/rename conversation、list/clear messages、chat_send、collect_context），全部 action 先校验 `experimental_enabled`
  - `agent.rs`：Agent 工具集（只读诊断工具：启动器信息 / 游戏日志 tail / 最新崩溃报告 / Mod 列表 / 启动器日志），结果截断防超长；启动器日志经既有脱敏函数处理后再交给模型
  - `ai_core/client.rs`：新增多轮 `chat_completions`（含 `tools` 下发与 `tool_calls` 回传）、`ChatTurn` / `ToolDef` / `ToolCall` 类型；`ai_core/mod.rs` 补导出
  - `ai_core/prompt.rs`：新增 `PromptKind::Chat` 助手提示词（引导优先使用工具取真实数据）
  - 聊天流程：保存用户消息 → 携带最近 20 条历史 + 工具定义请求 → 模型发起工具调用时循环执行（上限 4 轮）→ 保存助手回复；首条消息自动生成会话标题
  - 手动附加上下文兜底：`collect_context` 收集启动器/游戏日志/崩溃报告/Mod/启动器日志，拼入输入框后随消息发送（模型不支持工具调用时可用）
- 迁移：
  - `SettingsAdvanced.vue`：移除内嵌的 `<SettingsAi />`，AI 服务配置迁入实验性页「AI 设置」分类（复用 `SettingsAi.vue`）
  - `DiagnosticPage.vue`：移除「崩溃日志分析」卡片（`CrashAnalyzer`），迁入实验性页「日志分析」分类（仅本页可用）
  - 自动崩溃规则弹窗（`CrashDialog` 触发链路）按用户确认保留，不迁移
- 依赖：`src-tauri/Cargo.toml` 新增 `rusqlite = { version = "0.31", features = ["bundled"] }`；前端新增 `@lobehub/icons`（仅提取品牌 Mono 图标 path 数据，不直接引入 React 组件）
- AI 聊天前端重构（本批）：
  - `ExperimentalChat.vue` 由 545 行拆分为精简外壳（≤300 行），逻辑全部提取至 `src/composables/useAiChat.ts`，UI 拆分为 `ChatConversationList` / `ChatHeader` / `ChatMessageItem` / `AskUserDialog` / `VersionPickerDialog`
  - 消息操作栏：hover 默认隐藏，支持删除（后端配对级联）、重新生成（AI 消息）、复制（点击后选择「渲染后文本」或「Markdown 原文」）、编辑（仅最后一条用户消息，保存后自动重新生成）
  - 模型选择下拉框带品牌图标：`@lobehub/icons` 57 个品牌 Mono path 数据（`utils/lobe-model-icons.ts`）、模型名→品牌正则识别（`utils/model-icon.ts`）、`ModelIcon.vue` 渲染（未识别兜底 CpuChip）
  - Markdown 行内图标占位符 `[:icon:名称]`：`utils/md-icons.ts`（heroicons 24/outline path 数据 + 别名）、`markdown.ts` 新增 marked inline 扩展（仅消息正文生效）；新增 `markdownToPlainText` 供复制渲染后文本
  - 上下文窗口进度条：`utils/tokens.ts` 与后端一致的 token 估算（CJK≈1/字符，其余≈字符/4），≥70% 黄、≥90% 红；显示预估用量与后端校准 usage
  - 流式渲染：`ai-chat-stream` 事件逐 token 追加 + 闪烁光标；`ai-ask-user` 事件弹窗提问；`conversation-title-updated` 更新会话标题
  - 版本隔离感知：手动附加上下文与版本选择弹窗走 `list_installed_versions` + `VersionPickerDialog`，无版本时直接收集
  - 修复旧库 schema 缺失：`CREATE TABLE IF NOT EXISTS` 不会为已存在的 `messages` 表补列，导致 `no such column: pair_id`（查询/删除消息失败）；新增 `migrate_schema`（`PRAGMA table_info` 检查 + `ALTER TABLE ADD COLUMN` 补 `pair_id`/`version_id`），旧库升级后自动补齐
  - SQLite 架构重构（SQL 收敛 + 连接生命周期）：
    - 新增公共工具 `src-tauri/src/utils/sqlite.rs`：声明式 schema 迁移（`TableDef` 建表 SQL + 可迁移列 + 保留列，挂载时自动 `ADD COLUMN` 补全 / `DROP COLUMN` 去除）+ 全局连接维护（`mount` 幂等挂载、`with_conn` 加锁复用）+ 通用表访问（`Table`/`Cond`：insert/update_by_id/delete_where/query/query_first/count，SQL 生成全部集中于此）
    - `db.rs` 仅保留表结构声明（`CHAT_TABLES`）与数据访问函数，全部经 `Table` 语义接口调用（表 + 列 + 条件），不再出现任何 SQL 语句；移除每次操作重新 `open()` 连接的逻辑（改为全局连接）；配对删除等操作合并到同一连接内执行，避免嵌套加锁死锁
    - 业务层 `manager.rs` 零 SQL（配对回填改用 `db::set_message_pair_id`）；全项目 SQL 操作仅存在于 `utils/sqlite.rs` 一处
    - 连接生命周期：启动时读取 `experimental_enabled`，已启用则在启动流程挂载聊天库（连接由系统维护）；运行中开启时由 `apply_config` 挂载；未启用不挂载、不再每次请求检查
    - 聊天库就绪日志由 `log_info` 降为 `log_debug`，避免每次进入页面刷屏
  - 修复前端 void 返回误判：`safeCall` 包装 `Promise<void>`（Rust `()` 序列化为 `null`）后，`if (res)` / `if (!ok)` 会把成功判为失败，导致「删除会话失败 / 清空失败 / 提交回答失败」误提示；统一改为 `res !== undefined` / `ok === undefined` 判断（`useAiChat.ts` 删除会话/清空/删除消息/提交回答 4 处）
  - 修复新建会话重复创建：`newConversation` 内两次调用 `ensureConversation()`，无会话时点击一次创建两个会话；改为点击恰好创建一个新会话并切换（有/无会话行为一致）
  - UI 调整：token 进度条移至输入区按钮行右侧；发送按钮改为图标并置于输入框右下角（textarea 底部留白）；消息操作栏改为绝对定位（不再占用布局高度）；模型选择框加宽至 `w-60` 展示完整模型名
  - AI 客户端模块化（`ai_core/client.rs` → `client/` 目录 5 模块：`types`/`transport`/`tokens`/`chat`/`stream`）：复用公共 `http.rs` 全局 reqwest 客户端（`apply_config` 重建后对代理/IP/TLS 生效），AI 配置每次调用重新读取实现热重载；4 处错误响应移除 `truncate_chars` 截断改为完整打印，修复「警告日志只打印一半」问题
  - 技能调用消息内联展示（openclaw 风格）：后端 toolCall 事件携带全局序号 `index`（`r{轮次}-{序号}`）/`arguments`/`output`；对话流内联渲染技能调用条目（新组件 `ToolCallEntry`：执行中旋转图标 / 完成绿色对勾），点击展开详情面板查看入参与执行输出并支持复制；移除底部工具执行状态条
  - 修复工具调用后二次请求非流式：前端新增分帧打字机（`deltaQueue` + 16ms 帧 / 12 字符步长，`pushDelta`/`flushStreaming`），`streamingMsg` 独立占位消息渲染于技能条目之后，`done` 后刷新数据库内容；`onUnmounted` 清理定时器
  - 会话列表高度优化（`ChatConversationList`）：删除图标改为绝对定位（hover 显示不再拉长行高），行高略增 `py-2.5`
  - 全局返回顶部按钮（`BackToTop`）只响应外部容器：新增 `data-inner-scroll` 标记过滤，AI 聊天消息列表等内部滚动容器不再触发右下角返回按钮（此前会遮挡输入区发送按钮）；路由切换后的滚动位置恢复检测同步跳过内部容器
  - 修复日志工具只打印一半：`agent.rs` 的 `read_tail` / `read_launcher_logs` 原实现「先按字符截断、再取末尾行」，因 `truncate_chars` 保留的是行首，最终拿到的是日志**中间一段**；改为「先取末尾 N 行、再截断字符」，AI 读取游戏/启动器日志现在能看到真正的最新内容
  - 修复工具调用前的回复文本丢失与非流式：后端将 `chat_send` / `regenerate_reply` / `edit_message` 三处重复的工具循环统一抽取为 `run_tool_loop`（`manager.rs`），回复文本跨轮累积（原实现每轮覆盖，工具轮之前的文本丢失）、工具调用状态统一推送（原 regenerate/edit 不推送）、`done` 事件仅在全部轮次结束后推送一次并携带 usage 合计（原实现每轮流结束都推送，前端提前清空流式状态导致第二轮一次性渲染）；前端流式占位消息、工具调用条目与持久化消息在工具轮次间不再中断
  - 复制菜单交互补齐：`ChatMessageItem` 复制方式弹窗接入公共 `click-outside.ts`（`onClickOutside` + `onEscape`），点击菜单外部或按 ESC 即可关闭，无需必须选择一项
  - AI 工具链改为消息列表内、AI 回复消息框上方展示：极简收起条（默认收起），展开后以虚线时间线串联各工具调用（样式参考更新日志 ReleaseTimeline），每个节点展示工具名/状态（执行中/完成），点击展开入参与执行输出、支持复制；`done` 后工具链保留展示，由发送/重新生成/编辑/切换会话时统一清空
  - AI 工具链持久化入库（SQLite `tool_calls` 表）：工具调用记录随 AI 回复消息落库（新增 `list_tool_calls` IPC），前端按消息 id 分组加载，每条 AI 回复消息上方独立展示其工具链——刷新页面/重启应用后工具链仍保留，不再只存在于当前内存；删除消息/编辑重生成/清空/删除会话时级联清理对应工具链
  - 会话切换动画：消息区域切换会话时旧内容淡出、新内容淡入（`Transition` + 会话 id key），会话列表项新建/删除/切换时平滑移动与淡入淡出（`TransitionGroup` 增删/补位过渡）
  - AI 工具链展开动画：时间线整体淡入下滑、工具节点逐个入场（右侧移入）、节点详情展开淡入下滑，展开/收起不再生硬
  - 复制弹窗改版：消息复制由下拉菜单改为独立弹窗 `CopyMessageDialog`，使用项目统一组件风格——Input 只读文本域预览渲染后纯文本、AlertV2 提示复制格式、Button（outline/primary）选择复制「Markdown 原文」或「渲染后文本」，点击遮罩外部或 ESC 关闭
  - 最终消息只显示最后输出：后端 `run_tool_loop` 的 `reply` 改为保留最后一轮文本（工具前的过渡语句如「我来读取…」不再混入最终消息），前端在首个工具开始调用时清空已流式显示的过渡文本；工具调用过程由消息上方的工具链完整呈现
  - 修复上下文按钮报「未知上下文类型」：崩溃日志/游戏日志按钮的 kind 与后端 `collect_context` 匹配项统一（`crash_report` / `game_logs`），选择游戏版本后不再报错
  - 输出摘要标签（不额外消耗一次请求）：`AiConfig` 新增 `summary_tags` 开关（「AI 设置」新增配置项）；开启后聊天系统提示词要求模型在最终回复末尾自带 ≤15 字内容标签（`【TAG:xxx】`），后端 `extract_summary_tag` 剥离后存入 `messages` 表新增 `summary_tag` 列；前端对话目录概览（TOC）悬浮于消息区右侧（复用 `ToolToc`，新增 `minItems` 配置），展示各条 AI 回复的摘要标签，点击快捷跳转对应消息
  - 思考模型（如 DeepSeek-R1）思维链支持：`stream.rs` 解析 `delta.reasoning_content` 并通过 `reasoning` 事件流式推送；`ChatTurn` / `ChatResult` 新增 `reasoning_content`；涉及工具调用的轮次完整回传 `reasoning_content`（官方要求，否则 400）；前端 AI 消息内新增可折叠「深度思考」区块（流式生成时自动展开，完成后可折叠），思考内容随消息持久化到 `messages` 表新增 `reasoning_content` 列
  - 工具链展示调用前模型输出：`run_tool_loop` 将每轮模型调用工具前的过渡文本保存为 `pre_content`（`tool_calls` 表新增列），工具 `running` 事件携带 `preContent` 实时展示，持久化后刷新/重启仍保留
  - 修复对话目录 TOC 残留：`ToolToc` 除 `refreshKey` 外新增 `MutationObserver` 监听容器内 `[data-toc-card]` 的新增/移除/标题变更自动重扫——此前删除/切换会话时重扫发生在 `out-in` 淡出动画期间（旧卡片尚未移除），过渡结束后无再次扫描导致 TOC 残留旧条目；现在删除/切换/清空/流式完成后均实时同步（发送消息本就有 `refreshKey` 触发，不受影响）
  - 模型图标改用官方 `@lobehub/icons-static-svg` 静态映射库：删除 `utils/lobe-model-icons.ts`（57 品牌 path 数据），新增 `utils/model-brand-icons.ts`（54 品牌静态 import 官方 SVG，统一使用单色 mono 变体——纯 `currentColor` 填充、无渐变/defs，避免 WebView 中 `<img>` 渲染渐变失效显示为实心黑块），`ModelIcon.vue` 由 svg path 渲染改为 `<img :src>`；品牌识别修正：GLM 系列改用智谱现行品牌 `Zhipu`（`zhipu.svg`，替代旧版 ChatGLM 图标）、Qwen 使用官方单色 `qwen.svg`；`vite.config.ts` 的 `assetFileNames` 将 `@lobehub/icons-static-svg` 资源统一输出到 `dist/assets/@lobehub/` 目录；依赖由 `@lobehub/icons`（path 数据）改为 `@lobehub/icons-static-svg`
  - 品牌识别规则扩展（`model-icon.ts`）：修正 `qwen3.6-*`（`\bqwen[\d.-]*`）、`hy3/hy3-preview`（腾讯混元 `\bhy[-\d]`）、`nana-banana`（Google nano-banana 代号 → Gemini）；新增 `ling-*`（蚂蚁百灵 → Bailian）、`mimo/xiaomi`（小米 → XiaomiMiMo）、`laguna`（Poolside）；`MiniCPM` 与 `north-*` 因 lobehub 无专属官方图标，按用户指定复用 `HuggingFace.Color` 多色图标（硬编码色、非渐变，渲染无兼容问题）
  - 对话目录（TOC）改用用户消息内容：`data-toc-card` 由 AI 回复消息改挂用户消息，标题取用户消息纯文本前 15 字，点击仍跳转对应消息——据此移除 `summary_tags` 全链路（后端 `AiConfig.summary_tags` / INI key / `chat_system_prompt` 摘要指令 / `extract_summary_tag` / `messages.summary_tag` 列由声明式迁移自动删除；前端「输出摘要标签」设置开关 / `summaryTags` 字段 / 消息内标签展示块），会话目录不再依赖模型生成摘要
  - 深度思考完成后自动折叠：`ChatMessageItem` 的「深度思考」区块流式生成时自动展开，`streaming` 结束（false）时自动折叠；历史消息默认折叠，可点击手动展开
  - 模型图标双模式（彩色 / 黑白）：`AiConfig` 新增 `icon_color_mode`（默认彩色，AI 设置页新增「模型图标」下拉），全局响应式 `utils/model-icon-mode.ts` 保存后即时生效；`model-brand-icons.ts` 重建为双变体映射库（mono 经 `?url` 打包到 `dist/assets/@lobehub/` 以 `<img>` 渲染；color 经 `?raw` 内联注入——部分品牌彩色图含 SVG 渐变，`<img>` 渲染会失效显示黑块，故彩色必须真实 DOM 渲染）；无官方彩色变体的品牌（anthropic/grok/xiaomimimo/midjourney/openai/groq/lmstudio/ollama/cursor）彩色模式自动退回单色；未识别到品牌的模型统一兜底 HuggingFace 图标（移除 MiniCPM/north 的单独映射，兜底行为一致）
  - 对话渲染容错（`markdown.ts` 预处理，仅非代码块区域）：① 修复模型把整张表格挤在一行导致 GFM 表格错乱——按分隔单元格重建为多行表格；② 修复 `** xxx **` 星号内侧带空格导致加粗不生效——收紧为 `**xxx**`；围栏代码块内容原样保留
  - 修复工具参数解析误报：`ask_user` 曾出现入参含 `question` 仍报「缺少 question 参数」——模型输出的 `arguments` 非严格 JSON（带换行/夹杂文本）时 `from_str` 直接失败；新增 `parse_tool_arguments` 逐级容错（直接解析 → 截取首个 `{` 至末个 `}` 再解析 → 字符串结果二次解析防双重编码），所有工具调用共用
  - 消息固定展示回复模型：`messages` 表新增 `model` 列（声明式迁移自动加列），AI 回复入库时记录实际调用模型；前端 `ChatMessageItem` 图标与模型名优先取 `message.model`（切换右上角全局模型后历史消息图标不再随之变化），流式消息创建时即记录当前模型；消息 hover 操作栏新增发出/回复时间显示（`formatTimestamp`，本地消息时间统一为 Unix 秒与后端一致）
  - 消息信息展示改版（tag 化）：模型名由图标下方小字改为消息气泡右下角灰色 tag（`model` 优先消息记录、兜底当前模型）；AI 消息 hover 时间移到操作栏按钮右侧（用户消息仍在按钮左侧）；未识别模型的兜底 HuggingFace 图标保持不变；模型 tag 仅在最终正文存在时显示（纯思考/纯工具调用消息不显示，避免空正文下标签突兀）
  - 重新生成「第 N 次重试」标识：`messages` 表新增 `retry_count` 列（声明式迁移自动加列，默认 1），重新生成回复时读取旧回复序号 +1 写入；AI 消息气泡右上角以 amber tag 显示「第 N 次重试」（序号 >1 时），流式占位消息在重新生成时即带正确序号
  - 修复 Kimi 图标彩色模式不可见：`kimi-color.svg` 主体为 `fill="#fff"` 白色（浅色气泡背景上看不清），从彩色映射中移除，彩色模式下回退单色渲染
  - 会话列表支持图标折叠：`ChatConversationList` 头部新增收起/展开图标按钮（双左箭头收起为窄条、双右箭头恢复），收起后仅保留窄条节省消息内容区空间，折叠状态本地保持
  - 聊天页原生 tooltip 全面替换为项目 `Tooltip.vue` 组件：`ChatMessageItem` 消息操作栏 4 个按钮（删除/重新生成/复制/编辑）、`ChatConversationList` 删除会话与折叠按钮、`ExperimentalChat` 上下文进度条与发送按钮；`ChatHeader`/`AskUserDialog`/`ToolCallEntry`/`CopyMessageDialog`/`VersionPickerDialog` 经扫描确认无原生 `title`（`ChatHeader` 标题、TOC `data-toc-title`、`VersionPickerDialog` 弹窗标题均为 props/数据属性，非 tooltip）
  - Markdown 行内图标兼容 `[::名称]` 双冒号格式：`markdown.ts` inline 扩展正则同时匹配 `[:icon:名称]` 与 `[::名称]`（如 `[::game]`），均映射 `utils/md-icons.ts` 同名图标渲染为行内 SVG；同步更新 `resources/prompts/chat.md` 图标占位符说明并新增「自我介绍」段（首次交互/询问身份时输出 `[::game] 你好，我是 MoLaunch 启动器的智能助手，专门帮助你处理与 Minecraft 相关的各种问题。`）
  - 流式回复可打断：`AppState` 新增 `chat_cancel_flag`（AtomicBool 取消信号），`chat_completions_stream` 新增 `cancelled` 参数——请求发出前与流式循环内均检查，检测到取消立即中断并返回已生成部分内容（丢弃不完整的工具调用）；`manager.rs` 新增 `cancel_chat` action 置位信号，`chat_send`/`regenerate_reply`/`edit_message` 发起前重置；前端发送按钮在模型回复期间切换为暂停图标（`PauseIcon`），点击调用 `experimentalCancelChat`，已生成部分保留入库，取消后空回复显示「（已停止生成）」
  - 全局拖拽遮蔽层极简重写：`DragOverlay.vue` 移除按拖拽类型区分的彩色卡片，改为全屏半透明蒙层 + 四周白色虚线边框 + 居中上传图标与提示文案（`pointer-events-none` 不拦截拖放事件）
  - 全局拖拽遮蔽层二次修复（本轮）：`dragState` 新增 `status` 字段（`accept`/`pending`/`reject`，由 `classifyDrag` 设置），虚线边框颜色作为检测指示：可拖入浅绿（`emerald-300`）/ 待分析（zip、空路径）浅黄（`yellow-300`）/ 确定不支持（rar、未知类型、含非 Mod 多文件）红（`red-400`）；背景由 `bg-black/25` 加强为 `bg-black/45 + backdrop-blur-md`，遮蔽罩下方内容不可透见
  - 全局拖拽遮蔽层定位修正（本轮）：不再使用 `fixed inset-0 + z-index 压层级` 的方式，改为将 `<DragOverlay />` 挂载到 App.vue 内容容器 `div.relative.h-full`（nav 下方）内，组件内部 `absolute inset-0 z-50` 铺满所在容器——从布局上物理限制在内容区，顶部虚线不再穿到 nav 区域；`App.vue` 顶层挂载点移除，DragOverlay 组件去掉 Teleport
  - 聊天图标占位符收敛：`resources/prompts/chat.md` 新增固定图标映射表（game/mod/server/check/warn/error/info/download/tip/search 共 10 个，均为 `md-icons.ts` 既有图标），明确「只能从表中选择、严禁自创名称、正文外禁止使用」；「自我介绍」段原文移除反引号包裹（模型会连同反引号一起输出导致渲染成代码块、`[::game]` 显示为字面文本），改为直接书写占位符
  - 图标渲染改「引入」而非硬编码：`md-icons.ts` 移除手工转录的 SVG path 数据，改为从 `@heroicons/vue/24/outline` import 各图标组件建立映射，新增 `mdIconComponent(name)` 返回组件、`mountMdIcons(root)` 把 v-html 中的 `.md-icon[data-md-icon]` 占位符替换为 heroicons Vue 组件；`markdown.ts` inline 扩展渲染器改为输出占位符，`ChatMessageItem` 在内容更新后调用 `mountMdIcons`（遵守「必须通过引入使用图标组件、不在 ts 硬编码」的组件规则）
  - 图标渲染修复（本轮）：`markdown.ts` inline 扩展全面兼容模型输出的各种占位符变体——`[::名称]` 双冒号、`[:icon:名称]`、`[:名称]` 单冒号、`[名称]` 无冒号，均替换为图标；**仅当名称命中已知图标表时才替换**（普通文本如 `[注]`、markdown 链接 `[text](url)` 经负向前瞻排除、行内代码/代码块不生效），未知名一律原文保留；占位符改由 class 携带名称（`md-icon-名称`）而非 data 属性，避免被 DOMPurify 剥离；`ChatMessageItem` 由「watch content + nextTick」改为 **MutationObserver 监听 markdown 容器**，初始渲染（历史消息）、流式追加、消息替换均会挂载图标，已挂载的跳过
  - 图标渲染兜底（本轮）：模型实际输出常漏写闭合括号（如 `[::game 你好...`），原正则要求 `]` 闭合导致整体未识别、占位符原样显示。tokenizer 改为**闭合括号可选**（`\]?`），前缀 `[:icon:`/`[::`/`[:` 均可选；负向前瞻 `(?!\]?\()` 放在 `\]?` 之前并用 `\]?\(` 断言，防止正则回溯破坏 `[game](url)` 形式链接（链接文本为图标名时仍保持链接）。实测 18 用例全通过：各种格式闭合/漏写均正确渲染图标，未知名、中文、行内代码、链接不受影响。同步强化 `chat.md` 自我介绍为「必须一字不差原样输出」
  - 提示词节省 token：`resources/prompts/chat.md` 新增规则——思考内容（reasoning_content）、工具调用说明、分析推理、中间草稿一律使用英文编写，仅最终面向用户的回复正文使用中文；图标格式强调必须使用 `[::名称]` 双冒号格式（不能省略冒号）
  - 生成统计显示：`useAiChat` 记录本次回复开始时间，done 事件按 `totalTokens / 耗时` 计算生成速度 `chatSpeed`（t/s）；`ExperimentalChat` 输入区上下文进度条旁显示 `xxxx t · xx t/s`（`formatTokens` 过千转 k，如 `1.2k t · 45 t/s`），仅在有 usage 数据时显示
  - 修复流式输出丢字符（本轮，抓包实证）：用户抓包确认后端 SSE 原始流完整（`[::game]`、`MoLaunch`、「问题」均在），但前端最终显示缺 `]`、丢「Mo」与「问题」。根因锁定 `ai_core/client/stream.rs` 的流式循环：`stream.chunk()` 返回任意字节块，SSE 的 `data:` 行可能被 chunk 边界**切断**——被切断的不完整 JSON `from_str` 解析失败被 `continue` 丢弃，而紧随其后的续行因无 `data:` 前缀被整体跳过，导致丢字符与丢词。重写为**主缓冲 + 按 `\n` 切完整行**处理（`handle_line` 闭包统一解析单行、尾部残留留待下 chunk），流自然结束处理残留行、取消时返回不完整调用；`[DONE]`/finish 均走统一返回路径
  - token/耗时口径重构（本轮）：`messages` 表新增 `prompt_tokens`/`completion_tokens`/`total_tokens`/`duration_ms` 四列（声明式迁移自动加列）；`MessageItem` 新增对应字段。token 全部采用**后端流式 usage 累计**（`run_tool_loop` 各轮 `on_done` 累积，含全部工具调用轮次的 prompt+completion，不再用前端估算）；`duration_ms` 为**总生成耗时**（从首个请求发出到全部工具轮次结束，含工具调用与 ask_user 等待，与中转站口径一致）；三入口（chat_send/regenerate_reply/edit_message）用户消息零值落库、AI 消息写入真实 usage 与耗时，`done` 事件携带 `durationMs`
  - 生成统计移至消息框（本轮）：`ChatMessageItem` AI 消息气泡右下角模型 id 左侧新增 token 统计 tag（显示**输出 token**，单位用全额 `token`，如 `133 token · 33 t/s`，过千转 k，tabular-nums；速度 = 输出 token ÷ 总耗时，与中转站口径一致）；优先读历史持久化字段（`completionTokens` + `durationMs`），流式占位消息经 `liveSpeed`/`liveCompletion` props 用 done 事件实时值；`ExperimentalChat` 输入区原统计显示移除
  - 链接二次确认（本轮）：`markdown.ts` 的 `handleMarkdownLinkClick` 拦截外部链接后先 `showConfirmAsync` 弹确认框（展示目标 URL），确认后才经 Tauri shell 打开系统浏览器，防止 AI 输出夹带外链被误点
  - AI 提问工具重做（本轮）：`ask_user` 选项由纯字符串扩展为 `{label, description?}`（后端 `agent.rs` 归一化：兼容纯字符串与对象，最多 6 项）；`AiAskUserEvent`/`AskUserOption` 类型同步；前端 `AskUserDialog.vue` 由全屏遮挡弹窗改为**右下角悬浮卡片**（非阻断式，不遮挡全局、可继续操作页面其他区域），选项以按钮列表展示、备注文字置于选项下方，保留自定义答案输入
  - 会话列表拖拽与 hover 展开（本轮）：`ChatConversationList` 支持**拖动右侧手柄自由调宽**（160~360px，宽度持久化 localStorage，拖拽时禁用文本选择）；收起为窄条后**鼠标移入自动展开、移出自动收起**（`mouseenter`/`mouseleave` 控制 hover 态，宽度过渡平滑）；标题正常省略（`truncate`），悬停经 `Tooltip` 展示完整标题（class 透传 `min-w-0 flex-1` 保持省略生效）；`ChatHeader` 顶部会话标题改为 `flex-1` 自适应最宽并省略（原固定 `max-w-40`）
  - 重试 tag 与「深度思考」同行（本轮）：`ChatMessageItem` 顶部元信息行合并为同一行 flex 布局——左侧「深度思考」切换按钮、右侧「第 N 次重试」amber tag（`ml-auto` 靠右）；无思考内容时 tag 单独显示，避免「第 N 次重试」独占一行
  - 深度思考收起间距修正（本轮）：虚线边框区块改为仅展开时渲染（`v-if="message.reasoningContent && thinkingOpen"`）——收起时不再残留空的 `border-b + pb` 区块把按钮行与正文撑远；顶部元信息行与虚线间距统一收紧为 `mb-1.5`，虚线只作展开时思考内容与正文的分隔
  - 上下文进度条改真实 usage 口径（本轮）：`tokenEstimate` 不再用纯字符估算（原实现只统计各消息 `content`，不含思考内容/系统提示词/工具定义/工具调用轮次，数值远低于真实输入）；改为**取最新一条 AI 消息的 `promptTokens`（usage，即最近一次请求实际发送给模型的完整输入 token，含全部上下文开销）**，其后未回复的用户消息补估算，无 usage 时（新会话/旧数据）退化为前端估算并计入 `reasoningContent`——进度条反映真实上下文窗口占用
  - 会话压缩同口径校准（本轮）：后端 `compress_context` 不再按字符估算触发（与前端同因，会压缩过晚导致请求超长被服务拒绝）；新增 `estimate_context_usage`（与前端 `tokenEstimate` 口径一致：最新 AI 消息的 `prompt_tokens` + 其后消息估算，无 usage 退化为全量估算并计入思考内容），三个入口（chat_send / regenerate_reply / edit_message）压缩判断均以该真实占用为基准
  - 上下文进度条展示优化（本轮）：`formatTokens` 整数千位显示 `184k`（原 `184.0k`）；进度条加宽（`w-24`→`w-28`）并显示 `已用 / 上限` 双值（如 `1.9k / 184k`，tabular-nums 等宽），tooltip 补充百分比（`上下文已用 X / Y token（Z%）`）
  - AI 设置页交互补全（本轮）：进入页面**自动从服务端拉取模型列表**（配置了服务地址时，无需手动点击「加载模型」）；修复保存无提示——`aiSaveConfig` 返回 `Promise<void>`，safeCall 成功后结果为 `undefined`，原 `if (ok)` 恒为 false 导致「保存配置」成功也不弹 toast，改用 `ok !== undefined` 判断后正常提示「AI 配置已保存」
  - 重试/编辑无实时流输出修复（本轮）：done 事件到达时若打字机队列（`deltaQueue`）尚未消费完，不再立即 `flushStreaming` 清空队列，而是置 `donePending` 标记、等 `typeNextFrame` 把队列逐字消费完后再统一收尾（清占位 + `refreshMessages`）——此前短回复 + 密集流下 delta 与 done 同批到达时队列被清空、正文只能靠刷新一次性渲染（表现为「点击重试等全部响应完才一口气输出」）；同时 `flushStreaming` 统一重置 `donePending` 防状态残留
  - 流式消息图标不挂载修复（本轮）：`ChatMessageItem` 的 MutationObserver 改为在 `content` 首次非空后再绑定（`watch content + flush: 'post'` 兜底），流式占位消息挂载时内容为空、markdown 容器尚未渲染，此前 observer 未绑定导致流式追加的 `[::名称]` 图标占位符不替换（重试后正文图标缺失）
- AI 聊天增强 v2（`docs/AI_CHAT_ENHANCEMENT_V2_DESIGN.md`，本批）：
  - 思考模式控制：后端 `ChatCompletionsRequest` 新增 `reasoning_effort`（low/medium/high）透传（`stream.rs`/`chat.rs` 请求构造同步补齐）；`run_tool_loop` 与 `chat_send`/`regenerate_reply`/`edit_message` 三处 IPC 透传 `reasoningEffort`；前端新增公共组件 `Slider.vue`（原生 range 封装 + primary 配色 + 档位 marks），`ChatHeader` 新增「思考开关」（`Checkbox`）+ 思考程度滑块（低/中/高，开关关闭时滑块禁用），`useAiChat` 新增 `enableReasoning`/`reasoningLevel`，三个 API 调用按开关状态传参
  - 工作状态提示条：`useAiChat` 新增 `waitingNext`（loading 且占位无正文/无思考/无工具 →「正在进行下一步…」）与 `waitingAsk`（ask_user 提问中 →「等待你的回答：…」），`ExperimentalChat` 消息区底部以 spinner + 文案胶囊展示——解决 ask_user 最长 120s 等待期间"卡住没动弹"的感知问题
  - 工具链实时展开：`ToolCallEntry` 新增 `autoExpand` prop，流式区（进行中）自动展开实时看各工具 running→done，完成（streamingMsg 清空）自然收起，与深度思考交互一致
  - 自动滚动优化：`useAiChat` 维护 `scrolledUp` 状态（滚动容器 `scroll` 差值 >64px 视为用户上滑），内容变化仅在未上滑时自动滚底；发送/重试/编辑/切换会话重置；不再打断用户上滑
  - 日志分析 AI 模式：后端新增 `ai_analyze_log` IPC 与独立事件 `ai-analyze-stream`（delta 逐行推送 / `【STEP:N/5】` 标记 → step 事件 / done 携带全文）；新增提示词模板 `resources/prompts/log_analyze_steps.md`（`PromptKind::LogAnalyzeSteps`，5 环节输出要求）；前端新建 `AiLogAnalyzer.vue`（模型下拉 + 日志粘贴 + 复用 `StepProgressBar` 5 环节进度 + 流式 Markdown 结论 + `mountMdIcons` 图标），`ExperimentalLog.vue` 顶部 `SegmentedButtons` 切换「本地检测引擎 / AI 分析」
  - AskUserDialog 重新设计：primary 渐变头部 + 「需要你的确认」标题 + 关闭 X；选项改为「点选 + 底部提交」卡片（选中态 primary 边框/背景 + CheckCircleIcon，label + description 灰字备注）；底部自定义答案 `Input` + 取消/提交 `Button`（无选择且无输入时禁用）；进入动画 opacity + translate-y
  - 进度条扫光动画：`main.css` 新增 `@keyframes progress-sweep` 与 `.progress-sweep`（2.5s 从左到右慢扫渐变光带）；token 进度条与 `StepProgressBar`（`sweep` prop，默认关闭不影响既有使用方）叠加扫光层
  - 思考设置改为悬浮窗（本轮调整）：`ChatHeader` 不再内联摆放思考开关 + 滑块，改为**设置图标（AdjustmentsHorizontalIcon，开启时 primary 高亮）+ 锚定悬浮窗**——点击图标在图标下方弹出「思考设置」小卡片（非全局 Modal，`teleport to body` + fixed 定位 + 点击外部/内部关闭），内为「思考模式」Checkbox 与「思考程度」Slider（低/中/高，关闭时禁用）；沿用现有 `enableReasoning` / `reasoningLevel` props/emits，接口不变
  - 进度条流星效果（本轮调整）：`.progress-sweep` 由白色慢扫光带改为 **Codex 风格紫色流星**——头部亮紫（`#c4b5fd`）渐隐到透明尾巴 + 紫色光晕（双层 box-shadow），2s 线性从左到右往复扫过（token 进度条与日志分析进度条同步生效）
  - 日志分析两级流水线（本轮）：`ExperimentalLog` 移除「本地 / AI」二选一切换，改为**两级流水线**——第一级 `CrashAnalyzer`（本地规则引擎初检，粘贴日志 → 识别问题范围条目），结果区新增「用 AI 深度分析」按钮（emit `ai-followup`）；第二级 `AiLogAnalyzer`（接收本地初检后传回的日志文本自动发起分析，`externalLogText` prop + `consumed` 事件，后端 `localAnalyze=true` 注入预检范围，避免超长全文直发模型）；`ai_analyze_log` 后端新增 `local_analyze` 参数，用 `analyze_log_text` 预检后把「本地初检结果摘要 + 截断原文」作为用户上下文注入
  - AI 工具本地预检与行范围（本轮）：`crash_analyzer.rs` 提取纯函数 `analyze_log_text`（无 state，供命令与 AI 工具复用）；agent 工具增强——`read_game_logs` / `read_crash_report` 新增 `startLine` / `endLine`（按行范围精确定位，从 1 起）与 `localAnalyze`（true 时本地引擎先初检返回问题范围摘要，省 token）；新增 `analyze_crash_log` 工具（读最新崩溃报告 → 本地预检 → 返回分类/级别/关键行/建议范围摘要）与 `read_log_lines` 工具（读 logs/latest.log 指定行段）——AI 拿到预检范围后若发现缺关键日志，可自行调用行范围工具补读上下文
  - 修复提示词模板未注入（本轮）：`resources.rs` 的 `embedded_text` 遗漏登记 `prompts/log_analyze_steps.md`，导致 `ai_analyze_log` 读取模板失败、静默回退内置兜底；已登记补上
  - 日志分析两级流水线单输入（本轮）：日志页只保留本地引擎**一个输入框**——本地引擎识别出具体问题（非 other/info 类）直接展示结果；**无法定位具体问题时自动转交 AI**（`crash_analyzer.rs` 新增 `locate_keyword_context` 定位首个命中行并截取**前后各 15 行**带行号上下文，只把这段范围发给 AI，不再直发全文）；AiLogAnalyzer 在流水线模式下隐藏自己的输入框（`hasExternal`），仅保留模型下拉与结论展示
  - 日志分析交互重设计（本轮）：`AiLogAnalyzer` 重构为纯 **AI 深度分析结果面板**——彻底移除日志输入框（`Input`/`logText`/`hasExternal` 逻辑删除），只保留模型下拉 + 环节进度条 + 流式 Markdown 结论；`ExperimentalLog` 成为流水线协调者——页面唯一输入框在 `CrashAnalyzer` 本地引擎，本地无结果自动转 AI（emit `ai-followup` → AI 面板自动分析），本地有结果时点「用 AI 深度分析」转交；AI 面板空态不再显示「开始分析」大按钮（等待自动分析），仅在有结论时提供「重新分析」/「清空」；重复分析用「先置空再下一帧赋值」保证相同日志也能触发 watch
  - 修复流式请求超时误杀思考型模型（本轮）：根因是 `http.rs` 全局客户端自带 30s 客户端级超时，对 SSE 流式请求同样生效——思考型模型（R1 等）首 token 数十秒~数分钟，请求未返回第一个字节即被 `operation timed out` 中断。修复：`http.rs` 新增 `build_stream_client`（同管线但 `timeout` 置为无整体超时），`transport.rs` 新增 `authorized_stream_builder`，`stream.rs` 的流式请求改用该客户端，并把首字节等待超时放宽到 `max(config.timeout_secs, 180)`——正文读取不再受客户端级超时约束
  - AI 日志分析思考流与失败兜底（本轮）：后端 `ai_analyze_log` 的 `on_reasoning_delta` 由空实现改为向前端推送 `{ reasoning: delta }`（思考过程流透传），流式异常时推送 `{ error: msg }` 事件；前端 `AiAnalyzeStreamEvent` 新增 `reasoning`/`error` 字段，`AiLogAnalyzer` 在分析中展示「思考过程」区块（限高 `max-h-40` + 内部滚动 + 自动滚底），收到 `error` 即停止分析并 toast 报错——不再出现"进度条卡 25% 无反馈"
  - 阅读工具关键词搜索（本轮）：`read_game_logs` / `read_crash_report` / `read_log_lines` 新增 `keyword` 参数——按关键词定位首个命中行，返回其前后各 15 行上下文（带行号）；AI 拿到初检范围后若需更多上下文，可自行按关键词检索或指定行范围补读
  - 本地预检定位复用 detail（本轮）：修复「本地引擎识别到通用错误（other/info 类）却仍把全文发给 AI」——本地识别到的 other/info 条目其 `detail` 就是问题消息原文（如 `java.lang.Error: ServerHangWatchdog detected...`），`pick_log_keyword` 优先从 `detail`/`title` 提取特征词元（大写开头的类名/组件名，跳过 Exception/Error/Throwable 宽泛词），用该关键词在原文定位问题行，只把**前后各 15 行上下文**交给 AI，绝不回退发全文；仅当本地完全没识别到任何条目时才回退全文截断。前端 `CrashAnalyzer` 同步改为：本地无条目**或全部为 other/info**（无具体分类）时自动转交 AI 深度分析
  - AI 深度分析弹窗化 + 思考日志折叠（本轮）：`AiLogAnalyzer` 由页内卡片改为**弹窗形式**——复用 main.css 高度限制方案（`.modal-shell` 顶部对齐 + `.modal-body` `calc(100vh-100px)` 上限 + `.modal-scroll` 内容滚动区），标题栏带关闭按钮、ESC/遮罩可关闭；本地引擎转交 AI 时自动打开弹窗并发起分析。**深度思考日志默认收起**（折叠头「思考过程」+ 旋转箭头），点击展开查看（限高 `max-h-60` 内部滚动，展开时自动滚底）；结论完成后仍可展开回看思考过程。`ExperimentalLog` 页面顶部提示由自绘图标块改为 **AlertV2**（灰底 info 风格）
  - AI 深度分析弹窗重设计（本轮）：弹窗完全参考**更新日志弹窗（UpdateDialog）**的设计语言——`.modal-shell` 内嵌 `absolute inset-0 bg-black/40` 遮罩、`.modal-body max-w-xl mt-2` 顶部对齐、标题栏 `px-6 pt-5 pb-3`（SparklesIcon + text-base 标题 + XMarkIcon 关闭）、内容区 `.modal-scroll px-6 pb-2` 限高滚动、**固定底部按钮栏 `bg-gray-50 rounded-b-lg`**（关闭/重新分析）
  - 阶段指示重设计（本轮）：新增 `AnalyzeStageBar` 组件（与 Input/Button 同设计语言——灰底无边框 + 圆角 + text-xs）——**思考阶段只显示「深度思考中…」灰底不可用状态（脉冲点），绝不显示阶段/伪进度**；只有正文输出收到【STEP:N/5】标记（`step` 事件）时才显示当前环节（primary 蓝标签 + 已完成绿勾）；删除 StepProgressBar 的百分比/伪进度爬升展示，不再有"思考过程中显示 25% 卡住"的误导
  - 弹窗手动触发 + 关闭取消 SSE（本轮）：本地引擎初检完成转交 AI 时**不再默认弹窗**——只暂存待分析文本，页面底部出现「用 AI 深度分析」按钮，**点击后才打开弹窗**并执行；**关闭弹窗时若分析仍在进行，自动调用后端 `cancel_log_analyze` 停止 SSE 流**（后端 `AppState` 新增 `analyze_cancel_flag`，`ai_analyze_log` 将其传给 `chat_completions_stream` 的 cancelled 参数，被取消时优先推送 `{ cancelled: true }` 事件而非报错）；前端 `AiAnalyzeStreamEvent` 新增 `cancelled` 字段，收到后静默停止流式状态
  - 移除重复 AI 入口按钮 + toast 文案修正（本轮）：移除 `AiLogAnalyzer` 页面底部的外层「用 AI 深度分析」触发按钮（唯一入口保留在 `CrashAnalyzer` 本地结果区内），点击本地结果区的入口按钮才打开弹窗并执行；`CrashAnalyzer` 的 toast 文案由「已转交 AI 深度分析」改为「可点击『用 AI 深度分析』进一步诊断」，不再暗示已自动转交
  - 弹窗二次确认 + 结论正文限高（本轮）：点击「用 AI 深度分析」打开弹窗后**不自动开始分析**——弹窗内显示模型选择 + 「开始分析」按钮，用户确认模型后手动点击才发起（空状态文案改为「确认模型后点击『开始分析』启动 AI 深度分析」）；**分析结论正文容器限高 `max-h-72` + `overflow-auto`**（左右上下双向滚动，不拉长弹窗整体高度），思考日志同步改为双向滚动，均复用深度思考的容器内滑动模式
  - 修复 Select 下拉被弹窗遮挡（本轮）：根因是层级（z-index）冲突——`.modal-shell` 弹窗为 `z-[10000]`，Select 下拉面板 inline `zIndex: 9999`，比弹窗低导致下拉被盖住（`teleport to body` 无法解决层级问题）。已将 `Select.vue` 下拉面板 z-index 从 9999 提升到 **10010**，弹窗内模型选择下拉可正常展开
  - 思考阶段动态省略号（本轮）：`AnalyzeStageBar` 思考阶段的文案由静态「深度思考中…」改为「**正在思考如何判断问题**」+ 动态省略号动画——`setInterval` 每 400ms 递增点数量（0→1→2→3 循环，约 1.2s 一轮），省略号从前到后有规律地增长；进入正文输出阶段自动停止动画，重新分析回到思考阶段自动恢复
  - 提示词面向玩家优化 + 通用错误发完整日志（本轮）：`log_analyze_steps.md` 重写——明确受众是**普通玩家**（不是技术人员），输出必须含「问题定位」「如何修复」分节并给出**具体可执行的修复步骤**；玩家无法自行修复时（服务端核心 bug 等）必须明确提示「建议向社区或他人反馈」并附上反馈所需信息；**思考过程强制使用英文**节省 token（输出给玩家的正文仍为中文）。同时修复：本地引擎识别到**不确定的通用错误**（如 ServerHangWatchdog）时，不再只发问题行 ±15 行上下文——改为**发送完整日志**给 AI 判断（片段可能漏关键信息导致误判），并附带本地引擎识别的疑似信息标题；删除不再使用的 `pick_log_keyword` / `extract_feature_token` 函数
  - 实验性后端模块化重构（本轮，规范约束：>350 行必须重构、≤300 行可接受、文件头注释 5 行内、函数注释 3 行内、优先复用公共函数）：
    - `db.rs`（597 行）→ `db/` 目录：`mod.rs`（schema 声明 `CHAT_TABLES` + 惰性初始化 + 迁移 + 公共 re-export）/ `conversations.rs` / `messages.rs` / `tool_calls.rs`，全部数据访问仍经 `utils/sqlite` 的 `Table` 语义接口，零 SQL
    - `agent.rs`（678 行）→ `agent/` 目录：`mod.rs`（`AgentContext` + `tool_definitions()` + `execute_tool()` + `collect_context()`）/ `logs.rs` / `crash.rs` / `info.rs` / `ask.rs`（`ASK_USER_QUEUE` 等待队列与 `reply_ask_user` 回填）
    - `manager.rs`（1232 行）→ `manager/` 目录：`mod.rs`（公共辅助 `ensure_enabled` / `build_context` / `build_config_summary`）/ `dispatcher.rs`（`DISPATCHER` 注册表 + `dispatch` 入口）/ `chat.rs`（chat_send / regenerate_reply / edit_message / reply_ask_user）/ `context.rs`（resolve_chat_model / build_turns / estimate_context_usage / compress_context）/ `tool_loop.rs`（run_tool_loop 多轮工具循环 + parse_tool_arguments）/ `analyze.rs`（ai_analyze_log 5 环节流式 + process_analyze_line）/ `emit.rs`（emit_chat_done / emit_chat_status / generate_title）；`regenerate_reply`/`edit_message` 的模型解析抽为 `resolve_model_override` 共用，消除三入口内联重复
    - 公共能力收敛到 `utils/`：新增 `utils/fs.rs`（`ensure_dir` / `read_to_string` / `tail_lines` / `read_tail` / `newest_file`）与 `utils/format.rs` 的 `read_line_range`（带行号），`agent` 三读文件工具、日志分析页、启动器日志读取统一复用，不再各自实现
    - **mod.rs 入口化（本轮）**：落实「mod.rs 只能作为入口文件，逻辑不得写入」规范，三个目录的 mod.rs 仅保留模块声明与 re-export——
      - `agent/mod.rs`：`AgentContext` + 工具定义/执行/上下文收集 + 版本 helper（`effective_dir`/`require_version`/`version_arg`/`installed_version_ids`）整体迁至 `agent/tools.rs`（可见性 `pub(super)` 保持模块内私有），mod.rs 只留 `pub use`
      - `db/mod.rs`：`CHAT_TABLES` schema/表句柄/`now`/路径声明迁至 `db/schema.rs`；`ensure_initialized`/`migrate_legacy_db`/行映射/`touch_conversation_with` 迁至 `db/init.rs`；`conversations.rs`/`messages.rs`/`tool_calls.rs` 导入路径同步改为 `super::schema::`/`super::init::`
      - `manager/mod.rs`：`ensure_enabled`/`build_context`/`build_config_summary`/`HISTORY_LIMIT` 迁至 `manager/common.rs`，dispatcher/chat 引用同步更新
      - 复核结果：全部 6 个 mod.rs（`ai_core` 2 个 + `experimental` 4 个）均为纯入口；全部 rs 文件 ≤330 行；`ai_core`/`experimental` 无内嵌测试（项目既有测试均为同目录 `xxx_test.rs` 规范，此处无需迁移）
    - 拆分后各文件 ≤350 行（最大 `manager/chat.rs` 330 行）；`cargo check` 0 error；既有 clippy 警告（`stream.rs` let-else / `add_message` 12 参数 / `sqlite.rs` doc 列表缩进）均位于拆分前代码，不在本次改动范围
  - 会话压缩管线重设计（本轮，对齐 `docs/Agent_Compression.md`，方案 A：L1+L3 核心管线）：
    - 废弃旧 `compress_context`（丢最旧消息）与三入口内联压缩逻辑，新增 `manager/compression/` 子模块——`mod.rs`（纯入口）/ `trigger.rs`（触发判定：Token 使用率 ≥80% / 消息条数 ≥50 / 单条工具输出 >20K / 30s 防抖）/ `l1.rs`（工具输出文本截断：头 2500 + 尾 2000 字符）/ `l3.rs`（复用 `summarize.md` AI 语义摘要，失败静默降级为 L1+丢最旧）/ `rebuild.rs`（重塑器：摘要 system → 边界标记 → 最近 15 条原始消息，含工具轮次注入）/ `pipeline.rs`（总控 `compact_if_needed`）
    - 摘要持久化：新增独立表 `conversation_summaries`（主键 `conversation_id`，`db/summaries.rs` 删后插 upsert），删除会话时同步清理持久化摘要与防抖记录（`clear_cooldown`），避免内存/数据残留
    - 工具轮次注入统一化：历史工具调用以文本块追加到对应 assistant 消息后（规避 OpenAI 兼容服务对「assistant(tool_calls) 后必须紧跟 tool 消息」的严格配对），压缩与未压缩两条路径共用 `rebuild::inject_tool_blocks`（按枚举索引注入，消除 O(n²) 定位）
    - 三入口（chat_send / regenerate_reply / edit_message）统一接入 `common::build_chat_turns`（拉取工具记录 → `compact_if_needed` → 压缩时 emit 状态提示），消除三处重复逻辑；触发判定仅统计历史窗口内消息绑定的工具记录（窗口外旧记录不误触发）
    - `summarize.md` 重写对齐文档保真度要求（保留工具调用/任务目标/约束、第三人称、≤400 字、中文、无 Emoji）
    - 新增 `l1_test.rs` 同目录单测（`truncate_text` / `compact_records` / `estimate_tool_calls_size` 共 5 例）；全部压缩子模块 ≤160 行、mod.rs 纯入口；`cargo check` 0 error，clippy 无新增警告（既有 3 处位于拆分前代码）
    - 压缩管线文档对齐增量优化（本轮）：
      - `trigger::evaluate` 移除内联的字符估算 fallback（与 `context::estimate_context_usage` 同口径重复），统一复用后者（真实 usage 优先、无 usage 退化估算），消除双份估算实现与参数传递
      - `ChatTurn` 新增公共工厂 `ChatTurn::plain(role, content)`（`ai_core/client/types.rs`），`manager/context.rs` 的 `build_turns` 与 `compression/rebuild.rs`（原私有 `plain_turn`）两处复用，消除重复手写构造
      - L1 微压缩按内容形态增强（对齐文档 §4.1）：JSON 结构 >1000 节点时仅保留顶层 Key（嵌套对象 → `{...}`、数组 → `[... N 项 ...]`、长字符串截断）；代码/日志 >2000 行时保留首尾各 20 行、中间替换省略标记；其余超长文本维持字符头尾保留（头 2500 + 尾 2000）
      - pipeline 新增 L1 后达标检查（对齐文档 §8）：L1 压缩后估算占用已低于触发阈值时跳过 L3，避免无谓的 LLM 摘要调用；未达标才走 L3，失败仍降级 L1+丢最旧
      - 新增 `trigger_test.rs` / `rebuild_test.rs` 同目录单测与 `test_support.rs` 公共夹具（消息/工具记录构造），`l1_test.rs` 补充行级截断与 JSON 精简用例，压缩模块单测全量通过；`cargo check` 0 error，clippy 0 警告
  - 验证：`cargo check`（0 error）、`vue-tsc --noEmit`（0 error）、`eslint`（0 error，warning 为既有 v-html 提示）、`vite build`（0 error，`dist/assets/@lobehub/` 54 个 mono 图标）

### 修复

- AI 提问工具注释缺失 + 前端抽屉自适应展示修复（本轮）：
  - 后端 `tools.rs` 的 `ask_user` 工具 `options` schema 原只声明 `items: {"type": "string"}`，模型输出对象格式会被拒绝，导致「即使提示带注释也没有注释」；现改为 `oneOf` 同时接受纯字符串与 `{"label", "description"}` 对象（`required: ["label"]` + `additionalProperties: false`），模型可正式输出带 `description` 的选项
  - 提示词 `resources/prompts/chat.md` 第 4 条补充：每个候选选项推荐附带 `description` 注释（说明含义/适用场景），仅在含义显而易见才可省略
  - 前端新增可复用组件 `OverflowText.vue`（自适应省略：单行 `truncate` / 多行 `line-clamp`，`ResizeObserver` 监听容器尺寸变化如抽屉展开后自动重测，仅真实溢出时才显示 Tooltip 完整内容，未溢出不打扰）；`AskUserDialog.vue` 问题（3 行）、选项标签（1 行）、选项注释（2 行）均改用该组件
  - `AskUserDialog.vue` 选项容器由 `<button>` 改为 `<div role="button" tabindex="0">` + 键盘事件（原 button 内嵌 Tooltip 的 div trigger 违反 HTML 嵌套规范会被隐式闭合导致布局错乱）；后端 `ask.rs` 本就兼容字符串/对象两种选项并透传前端，前端 `AskUserOption` 类型已含 `description`，无需改动

- 整合包实例加载器信息缺失修复（本轮）：整合包/普通版本安装的 `install_merged`（`flow.rs`）在写 `setup.ini` 时未把已持有的加载器版本传下去——`save_setup_and_create_isolation` 调 `VersionSetup::new` 时 6 个版本参数全部为 `None`（注释写"从目录或 JSON 提取"但未兑现），导致所有经此安装的版本 setup.ini 只有 `Type/OriginalVersion`、缺 `ForgeVersion` 等 `XxxVersion` 键，`get_version_loader_info` 的加载器版本恒为空（RLCraft / SkyFactory 4 / 最小的机械动力 / Zombie Invade 100 Days 均受影响）。修复分三块：① `setup_persist.rs`/`flow.rs` 安装时把 forge/neoforge/fabric/optifine/liteloader 版本写入 setup.ini；② `load.rs` `load_or_create` 新增 `backfill_loader_versions` 自愈逻辑——对已有但缺 `XxxVersion` 的 setup.ini 从版本 JSON 的 libraries 回填并持久化（不覆盖 OriginalVersion/Type/个性化），`get_version_loader_info` 改走 `load_or_create`；③ 保底仍回退 `modpack.meta.json`（在线整合包含权威版本），`get_version_game_version` 在 JSON 提取失败时先回退 setup.ini `OriginalVersion` 再回退 meta。四个实例均可返回 MC 版本与加载器版本

- 整合包在线安装后残留 zip 修复（本轮）：`install_modpack`（`online.rs`）安装成功后未删除下载的原始整合包 zip，`InstallModpackResult.archive_path` 虽返回给前端但全前端无消费点，导致 zip 永久残留在 `versions/<实例名>/` 目录（如 `versions/RLCraft/RLCraft.zip`）。现于安装成功后在 async 块外删除该 zip（此时文件句柄已释放，避免 Windows 占用删除失败），删除失败仅告警不阻断结果，并将 `archive_path` 置空避免返回已不存在的路径；本地拖拽安装 `install_local_modpack` 不动用户原始文件，维持原逻辑

- CI 发布流程通道解析修复（本轮）：`scripts/ci-upload.cjs` 的渠道推导收敛到服务端合法取值——服务端 `VALID_CHANNELS` 仅接受 `stable / beta / alpha`，原实现把 `-rc` 推导为 `rc`、`-canary/-nightly` 推导为 `canary`，导致 `-rc1` 类 tag 触发 release 时「版本注册失败 (code=1001)：通道取值非法（仅 stable / beta / alpha）」。现 `-rc→beta`（Release Candidate 归入 beta 灰度通道）、`-canary/-nightly/dev/未知→alpha`，与 `docs/updater/design.md` 三通道约束一致；同步更新脚本头部与函数注释

- Java 运行时下载匹配修复（本轮，对齐 `docs/java-runtime-download-bugs-and-fix.md`）：
  - 【致命】Java 8 下载必然失败：组件 key 由 `java-runtime-legacy`（官方不存在）修正为 `jre-legacy`，并移除 `version.name.starts_with("8.")` 的失效模糊匹配（legacy 版本名为 `8u51-...`，不是 `8.` 开头）
  - 【错配】21→`java-runtime-gamma`、17→`java-runtime-alpha` 修正为 21→`java-runtime-delta`、17→`java-runtime-gamma`（对齐官方 all.json 真值：`java-runtime-alpha` 实为 Java 16）
  - 【平台硬编码】`match.rs` 不再固定 `windows-x64` / `windows-arm64`，改为按编译期 `target_os + target_arch` 推导（windows/mac/linux × x64/arm64/x86/i686），无法匹配时返回清晰错误而非默认回退 Windows
  - 【死代码】移除数字 key 精确匹配（`"21"/"17"/"8"` 在官方 all.json 中不存在）；移除依赖 HashMap 遍历顺序的 version.name 模糊匹配，改为显式映射表（8→`jre-legacy`、16→`java-runtime-alpha`、17→`java-runtime-gamma`、21→`java-runtime-delta`、25→`java-runtime-epsilon`）
  - `match_component` 返回 `Result` 携带清晰错误（未知 target / 缺平台节点 / 缺组件 / 解析失败），`pipeline.rs` 不再拼接硬编码 `platform: windows-x64` 文案
  - 新增 `download/match_test.rs` 同目录单测（5 例）：映射表对齐官方、未知 target 返回 None、端到端匹配 8/16/17/21/25、错误路径（未知 target / 缺平台 / 缺组件）；`cargo test --lib` 211 例全绿
  - 前端下载入口补全：`OFFICIAL_JAVA_MAJORS` 与工具页 `JavaDownloader` 预设档由 21/17/8 三档补全为官方 all.json 实际提供的五档（25/21/17/16/8），下载源说明与头部注释同步更新；后端 `download_java` 链路（`java_manager` → `commands/java.rs` → `download_java_runtime`）本就无 target 白名单，修复 `match.rs` 后 Java 16/25 均可正常下载

### 新增

- `src/components/about/MoLaunchIntro.vue`：重写「MoLaunch 实现原理」文案（约 300 字），补充多版本隔离与 Java 自动下载能力说明，精简联机/账号细节

### 合规

#### PCL2 许可合规重构（重度使用 5 条义务落实）

- 背景：依据 `docs/PCL2_LICENSE_COMPLIANCE.md`（2026-08-05 全项目分域复刻审计），本项目开发初期参考并移植了 PCL2 部分代码/数据/文案，依《PCL 分发有限许可》与《PCL 存储库合理使用指南》判定为「重度使用」，需落实 5 条义务并消除逐字复刻证据
- 改动：
  - `src-tauri/src/minecraft/launcher_profiles.rs`：魔数常量改为 `rand` 运行时随机生成（`AUTH_ACCOUNT_ID` Lazy 静态，`random_hex_id`），删除 `LoginType::Nide` 变体（对应 §6.3 处置）
  - `src-tauri/src/minecraft/auth/types.rs`：同步删除 `Nide` 变体
  - 崩溃分析器架构重写（对应 §5/§6.2，全项目最高风险项）：
    - 原「crit1 → stack → crit3」三级顺序短路结构已废弃，重构为「Collect → Detect → Score」：多路独立检测器并行提取证据（`detector.rs` / `detector_stack.rs`），评分器按置信度聚合产出结论（`scorer.rs`）
    - 规则改为声明式数据表（`rules.rs` 的 `KEYWORD_RULES`），与检测逻辑解耦；新增规则只需追加条目，无需改动流程代码
    - 原 `crit1/`、`crit3.rs`、`stack.rs` 已删除；`CrashInfo` / `CrashCategory` 与 `scheduler.rs` 调用点、`analyze_crash` 签名保持不变
    - 识别率等价：规则关键字集合与原逻辑逐项对照覆盖（运行时日志 13 条 + 崩溃报告 4 条 + hs_err 5 条 + 宽松规则 7 条 + 堆栈提取），行为差异仅在多证据并列时改为"置信度高者胜"（如崩溃报告含 F3+C 手动崩溃时按证据可信度择优），中文文案沿用批 2 原创版
    - 注：§5.1 建议的 `rules.toml` 未采用——项目无 `toml` crate 依赖，为避免新增依赖，规则表以 Rust 声明式常量承载（同为"规则即数据"形态）
  - `src/components/common/CrashDialog.vue`：标题改为「游戏启动阶段出错」，三按钮改为「查看报告 / 导出报告 / 关闭」，移除 MyMsgText 注释引用（对应 §6.6）
  - `src-tauri/src/minecraft/community/curseforge/fingerprint.rs`：清除 VB 伪代码注释，改为自述步骤
  - `src-tauri/src/minecraft/community/searcher/sort.rs`：排序权重表与 `score` 逻辑改为平台量纲补偿自定策略（对应 §6.4）
  - `src-tauri/src/minecraft/launch/jvm_args/build.rs`、`loaders/forge_installer.rs`、`resources.rs`：JLW/LUA/JavaWrapper 相关代码补充来源注释，改为指向第三方开源项目官方仓库（Java Launch Wrapper：MIT，https://github.com/00ll00/java_launch_wrapper；lwjgl-unsafe-agent：Apache-2.0，https://github.com/HMCL-dev/lwjgl-unsafe-agent），属性名与二进制内部契约对齐
  - `src-tauri/resources/about/licenses.txt`：新增 Java Launch Wrapper（MIT）与 lwjgl-unsafe-agent（Apache-2.0）两条第三方开源许可声明
  - 删除 `docs/JLW_LUA_REFACTOR_PLAN.md`：两个 jar 均为有官方仓库的独立开源项目，作为第三方依赖引入即可，无需自研替换
  - `src-tauri/src/commands/version/mods/metadata/sources.rs`：移除注释中对 PCL `LocalResourceFile.LoadMetadataFromJar` 的方法名引用，改为「依据各加载器官方文件格式规范」
  - `src-tauri/src/minecraft/community/curseforge/convert.rs`：CF 依赖排除 ID（306612/634179）提取为具名常量 `CF_EXCLUDED_DEPENDENCY_IDS`
  - `src-tauri/src/minecraft/community/searcher/aggregate.rs`：`PAGE_SIZE=40` 补充取值依据注释（CF 单页上限 50 / Modrinth 100，取保守值）
  - `src-tauri/src/commands/community/install/modpack_stages/parsers/curseforge.rs`、`modrinth.rs`：Quilt 排除与 Forge recommended 特判补充「本项目功能性决策」理由注释
  - `src-tauri/resources/about/acknowledgements.txt`：特别鸣谢新增 Plain Craft Launcher 2 (PCL2) 条目（置于 MoCDN 之后），表达开发初期的启发致谢；logo 使用 `PCL2.ico`，作者头像使用 `LTCatt.jpg`（均位于 `src/assets/AboutIcon/`）
  - `src/views/settings/more/CreditsTab.vue`：「关于 PCL2」段落改为中性简述（保留独立第三方声明与许可链接），致谢移入鸣谢列表；logo 缺省时新增首字占位
- 复用点：`rand` crate（Cargo.toml 已有依赖）、`once_cell::sync::Lazy`（已有）
- 验证：`cargo check`（0 error）、`vue-tsc --noEmit`（0 error）、`eslint`（0 error）

### 新增

#### AI 分析模块（本地 OpenAI 兼容服务，全息化设计）

- 背景：引入 AI 模型分析逻辑，目前仅提供本地分析服务；AI 后续不只用于日志分析，因此设计为通用模块
- 改动：
  - 新增 `src-tauri/src/ai_core/`（服务层，不含 Tauri 依赖，分层解耦）：
    - `config.rs`：AI 服务配置（base_url / api_key / timeout_secs / models / default_model，含 `resolve_model`：显式指定 > 默认模型 > 已启用模型首个；**默认留空**，需在设置页配置后才能使用）
    - `prompt.rs`：按场景构造提示词（系统 prompt + 崩溃日志用户消息拼接，多源日志截断控制上下文）
    - `client.rs`：OpenAI 兼容 API 客户端（复用 `crate::http::get_client`，支持可选 `Authorization: Bearer` 认证头；`chat` 支持显式指定模型；`list_models` 拉取服务端模型列表）
    - `storage.rs`：配置持久化到 `config.ini` [AI] 段，api_key 经 SDK DES 加密存储（懒加载解密，参照 CurseForge 模式），模型列表以 JSON 数组存储
  - 新增 `src-tauri/src/commands/ai/`（IPC 层，仿照 commands/tools 模式）：
    - `mod.rs`：`ai_manager` Tauri 命令入口（`generate_handler!` 注册点，必须定义在本模块）
    - `manager.rs`：action 分发（复用 `utils::dispatcher::Dispatcher` + `handler!` 宏）；`save_config` 从 AppState 取 SDK 加密 api_key；`list_models` / `check_status` 支持传入前端当前表单值探测（避免未保存时探测旧配置）
    - `types.rs`：IPC 类型（AnalyzeCrashParams 含可选 model / AiAnalysisResult / AiStatusResult / AiProbeParams）
  - `src-tauri/src/lib.rs`：注册 `pub mod ai_core;` + invoke_handler 注册 `ai_manager` + 注入 AI 存储 SDK 引用（`ai_core::storage::set_sdk`）
  - `src-tauri/resources/defaults/config.ini`：新增 [AI] 段模板（base_url / api_key / timeout_secs / models / default_model）
  - `src-tauri/src/utils/format.rs`：新增通用 `truncate_chars`（字符级截断，兼容中文），供 ai_core 复用
  - 新增 `src/utils/api/ai.ts`（前端封装）：`aiManager` + `AI_ACTIONS` 常量 + 5 个 action 封装
  - action 清单：`analyze_crash`（崩溃日志 AI 分析，默认使用 default_model，可显式指定 model）/ `check_status`（探测服务可用）/ `save_config` / `load_config` / `list_models`（拉取服务端模型列表）
- 新增 `src/views/settings/SettingsAi.vue`：进阶设置页「本地 AI 服务」配置卡片（服务地址 / API Key 密码框 / 请求超时滑块 10-300s / 模型管理：从服务端加载模型列表 → 多选框勾选启用（非全量导入）→ 默认模型下拉选择 + 检测连接 + 保存配置，持久化到 config.ini [AI] 段）
- 新增 `src/utils/dev-api.ts` 调试命令：`molaunch.showCrashDialog()`（用样例数据直接触发错误日志弹窗，便于平时难以复现场景下检查弹窗展示）与 `molaunch.ai(action, params)`（控制台调用 ai_manager IPC）
- 复用点：`crate::http::get_client`、`utils::dispatcher::Dispatcher` + `handler!` 宏、`utils::format::truncate_chars`、`utils::sdk_crypto::encrypt_with_sdk` / `decrypt_with_sdk_optional`（参照 CurseForge secure_storage 模式）、`Storage::get_config` / `set_config`（config.ini [AI] 段）；崩溃日志场景输入与现有 `watcher/analyzer/collect.rs` 收集结果对齐（runtime_log/error_lines/crash_report/hs_err）
- 合规：所有文件 ≤300 行、文件头注释 ≤5 行、mod.rs 仅聚合不堆积、无重复造轮子
- 验证：`cargo check`（0 error）、`vue-tsc --noEmit`（0 error）、`eslint`（0 error）




### 修复

#### Tag 组件预设色 purge 丢失 + 全局缺 import 补齐

- 背景：① `Tag.vue` 的 `colorClass` 实为动态拼接 `` `tag-${props.color}` ``（注释声称用静态映射但代码未改），导致 `main.css` 中 `@layer components` 的 `.tag-red`/`.tag-arcoblue`/`.tag-orange` 等预设色类因只出现在动态拼接里被 Tailwind purge，全部 Tag 只剩无背景的基础样式 → 更新日志时间线等处的标签变白、丢失语义色；② 全局替换时 10 个文件模板已用 `<Tag>` 但漏加 import，未注册组件被当作原生 `<tag>` 元素渲染（无样式直接显示文字），造成「点击展开」等文字冲突
- 改动：
  - `src/components/common/Tag.vue`：`colorClass` 改为 switch 静态映射（同 Button.vue 的 typeClass 模式，返回 `tag-red`/`tag-green` 等完整字面量类名，Tailwind 可静态扫描）；新增 `primary` 预设色（类型 + PRESET_COLORS + colorClass case）
  - `src/assets/styles/main.css`：新增 `.tag-primary`（`color: var(--color-primary-500)` + `rgb(var(--color-primary-rgb-500)/0.1)` 浅底，跟随项目主题色换肤）
  - 补齐 10 个文件缺失的 `import Tag from '@/components/common/Tag.vue'`：`MoLaunchIntro`、`JavaPathSelector`、`ArchiveManager`、`DataExporter`、`ModDedupScanner`、`CreateRoomForm`、`UpdateDialog`、`JavaManager`、`FabricApiInfoCard`、`ModpackSelector`
  - `ReleaseTimeline.vue`：最新版本号与「最新」标签改用 `primary`（跟随主题色），「测试版」通道标签由 `orange` 改 `gold`（黄色）
- 验证：`vue-tsc --noEmit`（0 error）、`npx eslint src`（0 error；9 个既有 warning）

#### 清理 CHANGELOG 中第三方启动器相关引用
- 背景：CHANGELOG 大量变更记录以「参考 PCL2 / 与 PCL2 一致 / 复刻 PCL2 ...」表述，含对第三方启动器源码（`ModBase.vb`、`ResourceSearcher.vb`、`LocalResourceFile.vb`、`MyMsgText.xaml` 等）的引用，需在文档层面去品牌化
- 改动：新增 `scripts/strip-pcl2.cjs` 批量替换脚本（UTF-8 无 BOM 读写，替换前备份 `CHANGELOG.bak`，精确上下文规则 + 替换后残留计数），清除 CHANGELOG.md 中全部「PCL2 / Plain Craft Launcher / PCL-main / PCL 路径」相关引用：
  - 「参考/复刻/对齐 PCL2 xxx」统一改写为「参考/复刻主流启动器」或中性表述，移除对外部源码的具体行号/文件引用
  - 历史记录中「清理代码中 PCL2 相关注释」「PCL2 整合包安装逻辑研究结论」等标题改为第三方启动器/中性表述
  - 涉及 `PCL/` 目录、`PCL\Logo.png`、`PCL 路径` 的历史迁移描述改为中性表述（保留 MoLaunch 命名）
- 验证：替换后 `CHANGELOG.md` 中 `PCL2|PCL-main|Plain Craft Launcher|PCL` 相关引用为 0，正文语义通顺、中文无乱码，备份文件保留可回退


### 新增

#### 全项目自绘 badge/tag/chip 样式统一替换为 Tag 组件

- 背景：此前多组件各自内联 `rounded-full/rounded + px-* py-* + text-xs/[10px]/[11px] + bg-*-100 text-*-700` 重复实现标签样式，色彩不统一、脱离 Arco 风格（组件复用约定要求用自定义组件而非原生 HTML/badge 样式）
- 改动：将 30 个文件中**纯展示型**标签统一替换为 `@/components/common/Tag.vue`（`size="small"`，色板按语义映射 Arco 预设色），并移除对应内联标签 class
  - 类别/来源标签：`ModpackRequirementCard`、`ModpackSelector`、`CreateRoomForm`、`LobbyRoomCard`、`CleanupGroupList`、`OverviewTab`、`ExternalDownload`、`RemoteTunnelSync`
  - 版本/通道标签：`VersionSection`（正式版/已安装）、`VersionGroupCard`（releaseColor 改为返回预设色名）、`JavaManager`、`JavaPathSelector`、`ModDedupScanner`、`LoaderCard`（版本标签与 `ver.tags` 动态色映射）、`ReleaseTimeline`（时间线版本号 vX.Y.Z → arcoblue/gray 预设、"最新"→arcoblue、测试版通道→orange）
  - 状态信息标签：`ArchiveManager`（有效）、`DataExporter`（文件大小）、`MemoryOptimizer`（强力/轻量）、`FabricApiInfoCard`（将自动安装）、`UpdateDialog`（强制更新）
  - 计数徽章：`VersionSelect`、`DownloadedFileList`、`ExportOptions`（必选）
  - 其他：`AboutTab`（版本号）、`ModListItem`（mod 版本）、`ResourcePackConverter`（ZIP/文件夹）、`NbtTreeNode`（tagColor 改为返回预设色名）、`DependencyItem`（platformLabel）、`SettingsCache`（TTL/分类）、`MoLaunchIntro`/`SeedMapIntro`（点击展开）
  - 既有 `.badge-*`/`.tag-*` 通用类与含图标/Tooltip/交互/语义状态徽章（`OnlineTopBar`、`AccountCard`、`RoomGuestPanel`、`OnlineDevicePanel`、`DeepLinkTab`、`PluginListSection`、`ModToolbar`、`PermissionTableSection`、`CrashAnalyzer`、`LaunchPanel` 等）按规则保留
- 验证：`vue-tsc --noEmit`（0 error）、`npx eslint src`（0 error 0 warning，仅既有 9 warning）


### 新增

#### Tag 组件：复刻 Arco Design Vue Tag（替代自绘前缀 tag）

- 背景：更新日志时间线中提交前缀 tag（`fix:`/`feat:` 等）此前为自绘样式（粗大圆角块 + 自定色），视觉粗糙、与整体 Arco 风格不协调
- 改动：
  - 新增 `src/components/common/Tag.vue`：参考 Button.vue 的复刻方式，从 Arco Design Vue 提取 Tag 组件（顶部含 MoTeam 版权声明与 MIT 许可说明）——内置 13 种预设色（red/orangered/orange/gold/lime/green/cyan/blue/arcoblue/purple/pinkpurple/magenta/gray，浅色底 + 深色字，色板取自 Arco 官方 1 号/6 号）、3 种尺寸（small 20px / medium 24px / large 28px）、可关闭按钮（closable + close 事件）、icon 槽位；支持自定义 hex/rgb 颜色（背景即色值，白字，与 Arco 行为一致）；颜色类名用静态映射避免 Tailwind purge（同 Button.vue 注释说明）
  - `src/assets/styles/main.css`：`@layer components` 新增 `.tag` 基础样式 + `.tag-size-*` 尺寸 + 13 色预设（浅底深字）+ `.tag-icon`/`.tag-close-btn`/`.tag-close-icon`
  - `src/components/about/ReleaseTimeline.vue`：提交前缀改用 `<Tag size="small" :color="...">`（feat→green / fix→red / docs→blue / refactor→purple / perf→orange / chore→gray / style→cyan / test→magenta / build→arcoblue / ci→gold / 其他→gray），删除原 `.commit-tag` 与 `.tag-*` scoped 样式，保留 commit-item 布局对齐（:deep(.tag) flex:none）
- 验证：`vue-tsc --noEmit`、`eslint`（0 error 0 warning）通过


### 新增

#### 更新日志时间线增强：版本通道标签 + 提交前缀 tag + 弹窗拉长

- 背景：时间线展示的更新日志中，① 无法直观区分正式版/测试版；② 提交条目 `fix(xxx): xxx` 这类 conventional commits 前缀语义无标注；③ 弹窗宽度 `max-w-md`（28rem）偏窄、日志区 `max-h-44`（11rem）高度受限，多版本日志展示不完整
- 改动（`src/components/about/ReleaseTimeline.vue`）：
  - **版本通道识别**：复用 `utils/version.ts` 的 `parseVersion` 判断 stable/rc/beta/alpha/canary，非 stable 版本在版本徽章旁展示琥珀色"测试版"标签（正式版不展示）
  - **提交前缀识别**：按 `- prefix(scope): 内容`（conventional commits）正则切分条目，feat/fix/docs/refactor/perf/chore/style/test/build/ci 各渲染语义化彩色 tag（如"新功能/修复/文档/重构"），tag 徽章位于**内容前面**，正文自动剥离 `fix:` 等前缀；tag 前保留项目符号圆点，维持 markdown 列表视觉；未收录前缀用灰色"其他"；无前缀的普通列表项原样保留
  - **省略细粒度 scope 条目**：命中 `skin/watcher/modrinth/searcher/download/image_cache/java/parse/jvm_args/skin_resourcepack/signaling` 作用域的提交条目直接省略，避免拉长行宽；识别到 `!c` 标记（跳过 CI 等）的条目默认忽略
  - **左侧竖线贯穿到底**：移除"最后一项竖线不延伸"规则，改为最后一项增加底部留白，竖线完整延伸到最底部
  - **版本节点可折叠（带动画）**：新增通用组件 `src/components/common/Collapse.vue`（`grid-template-rows 0fr→1fr` 平滑高度过渡，无需测量高度，与 MoLaunchIntro/CollapsibleCard/CleanupGroupList 折叠方案一致）；时间线点击版本标题折叠/展开该版本日志——**默认全部展开，折叠仅作用于当前点击的版本**，不影响其他节点；chevron 箭头带 300ms 旋转动画；支持键盘（Enter/空格）操作与 focus 样式
  - **弹窗尺寸与滚动区域**（`src/components/about/UpdateDialog.vue`）：宽度 `max-w-md` → `max-w-xl`（28rem→36rem）；"发现新版本"分支改为独立 flex 容器——**最新版本行固定**，仅下方日志区域按 `min-h-0 flex-1 overflow-y-auto` 独立滚动（其余状态仍用 modal-scroll 整体滚动）
- 验证：`vue-tsc --noEmit`、`eslint`（0 error 0 warning）通过；解析逻辑用例单测通过（5 通道识别 + 9 前缀/scope/`!c` 识别 + 1 多版本合并切分）


### 新增

#### 更新日志时间线组件：`ReleaseTimeline.vue` 分版本分段渲染

- 背景：api-server 返回的 `notes` 为「当前版本 → 最新版本」多版本合并 Markdown（每段以 `## MoLaunch <version>` 开头，最新在前），此前在更新弹窗中整段 `v-html` 渲染，版本边界不清晰、阅读体验差
- 改动：
  - 新增 `src/components/about/ReleaseTimeline.vue`：按版本标题（`##`~`####` + 可选 `MoLaunch` 前缀 + 可选 `v` + 语义化版本号）切分为独立节点，左侧竖线 + 圆点（最新节点高亮 + "最新"标签）串联成时间线，每节点渲染对应版本的 Markdown 正文；无法识别版本标题时（历史单段数据）退化为整段 Markdown 渲染
  - `src/components/about/UpdateDialog.vue`：更新日志区域改用 `<ReleaseTimeline :notes="updateState.notes" />`，移除原 `notesHtml` computed、`renderMarkdown`/`handleMarkdownLinkClick` import 与 `.markdown-body` scoped 样式（已随组件迁移）
  - 链接点击仍走 `handleMarkdownLinkClick` 系统浏览器策略，XSS 防护（DOMPurify）不降级
- 验证：`vue-tsc --noEmit`、`eslint`（0 error 0 warning）通过；解析逻辑 5 个用例单测通过（CI 多版本格式 / v 前缀 + rc 后缀 / 三级标题变体 / 无标题退化 / 空串）


### 修复

#### updater.exe 签名校验与 Tauri 官方插件对齐（minisign 格式）

- 背景：`src-tauri/updater/src/verify.rs` 此前自研校验存在三处根因问题：① 硬编码公钥 `RWQ...696F` 是从 `tauri.conf.json` 复制损坏的（真正公钥行为 `RWQ...EvIO...696E`，原值 65 位非法 base64 无法解码）；② 自假设「公钥 34 字节取 `[2..]`、签名 66 字节取 `[2..]`」偏移错误，且缺少 key_id 匹配与全局签名校验；③ prehashed 用 SHA-512，而 minisign 标准是 BLAKE2b-512 —— 导致 Windows 自研更新校验从未真正工作
- 另：`scripts/ci-upload.cjs` 注册版本时把 `.sig` 内容去除换行/空白后存储，破坏 minisign 4 行格式，Tauri 官方插件（macOS/Linux）同样无法解析
- 改动：
  - `src-tauri/updater/Cargo.toml`：移除 `ed25519-dalek`、`sha2`，新增 `minisign-verify = "0.2"`（与 tauri-plugin-updater 同款，零依赖）
  - `src-tauri/updater/src/verify.rs`：重写为 `minisign_verify` 实现（`PublicKey::decode` + `Signature::decode` + `verify(bin, sig, allow_legacy=true)`），公钥常量 `PUBKEY_B64` 与 `tauri.conf.json` 的 `plugins.updater.pubkey` 逐字符一致（同一份 `dW` 开头 base64，更换密钥时两处同步即可）
  - `scripts/ci-upload.cjs`：`.sig` 内容原样存储（保留 4 行换行，`SIGNATURE_B64` 改名 `SIGNATURE`），不再去除空白
- 验证：updater `cargo check --offline` 通过（minisign-verify v0.2.5 锁定）；ci-upload.cjs `node --check` 通过；DB `signature TEXT` 无长度限制
- 注意：数据库中已有的历史 release 记录 signature 仍是旧拼接格式（无法解析），需重新发布或用管理后台更新 signature 后才可校验通过


### 新增

#### 收编 apiServer 接口路径：新增 `src-tauri/src/api_paths.rs` 统一集中定义

- 背景：启动器所有请求云端 apiServer 的 HTTP 路径此前散布在 9 个文件（认证/JWKS/时间、FRP、Signaling 房间/会话/白名单/大厅、更新检查）中硬编码，路径变更需逐处修改、易遗漏
- 改动：
  - 新增 `src-tauri/src/api_paths.rs`：集中定义全部 `/v1/*`、`/v3/*` 路径常量（静态路径直接引用；带参路径用 `{room_code}`/`{participant_id}` 等命名占位符，调用方 `.replace()` 填充；updater 的 `{{target}}` 风格模板保持一致）
  - `lib.rs` 注册 `pub mod api_paths`
  - 改造 9 个文件引用常量：`client/auth.rs`、`client/jwks.rs`、`client/time.rs`、`online/frp.rs`、`signaling/room_api.rs`、`signaling/session.rs`、`signaling/whitelist.rs`、`signaling/lobby.rs`、`commands/system/updater/check.rs`（删除原 `UPDATER_PATH` 局部常量）
  - 日志路径（`http_log::log_http_request` / log_info 的路径字段）同步改用常量，保证请求与日志单一来源
- 验证：`cargo check`、`cargo clippy`（无告警）、`cargo test`（181 passed）通过


### 新增

#### 更新下载：macOS/Linux 显示真实进度 + 升级日志全量合并 + raw 端点统一响应

- 背景：① macOS/Linux 点击「立即更新」时 `install_unix.rs` 以空闭包调用官方 plugin 的 `download_and_install`，前端看不到下载进度；② v1 raw manifest 端点仍以 Tauri plugin 裸 JSON 格式返回且参数错误返回裸状态码，与既有 UnifiedResponse 约定不一致；③ 更新日志仅返回最新一条 `release_notes`，跨版本升级时用户看不到中间版本改动
- 改动：
  - `src-tauri/src/commands/system/updater/install_unix.rs`：`download_and_install` 的 `on_chunk` 闭包累计已下载字节，节流（每 256KB 或下载完成）经 `app.emit("update-download-progress", { downloaded, total })` 推送进度
  - `src-tauri/src/commands/system/updater/check.rs`：解析改为先解包 UnifiedResponse `{ code, msg, data }`（`code != 1` 视为无更新），字段从 `data` 内读取
  - `src/components/about/UpdateDialog.vue`：`onGlobalEvent` 监听 `update-download-progress`，仅 downloading 状态写入 `updateState.downloaded/total`（total>0 显示真实百分比，total=0 保持 indeterminate）
  - `api-server`：`GET /v1/updates/manifest/raw` 去掉 Tauri 裸 JSON 兼容逻辑，改走与 `/manifest` 一致的 `UnifiedResponse` 包装；删除 `TauriManifest` 结构
  - `api-server/repositories/updates.rs`：新增 `find_releases_after_version`（按 `published_at DESC` 拉取平台/架构/通道全部上架版本）
  - `api-server/services/updates.rs`：`check_for_update` 查询当前版本之后至最新版本的全部版本，按语义化版本降序合并 `release_notes`（`## MoLaunch <version>` 分段拼接，空日志跳过），供前端一次渲染一路更新日志；版本过滤在 service 层用 semver，避免 SQL 字符串比较对 `0.10.0`/`0.9.0` 误判
- 验证：
  - api-server：`cargo check --all-features`、`cargo test` 通过；clippy 无新增告警
  - client（Windows 目标）：`cargo check`、`cargo clippy` 通过；macOS/Linux 的 `install_unix.rs` 因本机无交叉编译器由 CI 验证
  - 前端：`vue-tsc --noEmit`、`eslint` 通过


### 修复

#### Windows 退出时静默更新失效：last.sig 签名未缓存导致 updater 参数解析失败

- 背景：后台预下载链路（定时 `download_update_to_appdata` → 退出时 `apply_pending_update`）启动 updater.exe 时只传了 `--old-exe` / `--new-exe` / `--pid`，漏传 updater 参数解析要求的必填 `--signature`，导致 updater.exe 以退出码 1 失败，退出时自动替换实际不生效（立即更新路径不受影响）
- 改动（`src-tauri/src/commands/system/updater/install_windows.rs`）：
  - 新增 `last_signature_path()`：`%APPDATA%/.Molaunch/last.sig`（与 `last_exe_path()` 对称）
  - `download_update_to_appdata_impl`：下载 last.exe 后把 `UpdateInfo.signature` 同步写入 last.sig（写失败回滚 last.exe 保证配对一致性）；签名缺失时直接报错，避免下载无法替换的无效文件
  - `apply_pending_update_impl`：读出 last.sig 并作为 `--signature` 传给 updater.exe；last.exe 存在但 last.sig 缺失/为空时视为预下载不完整，清理两个文件后返回 false，等下次定时检查重新下载
- 验证：`cargo check` / `cargo clippy --all-features` 通过（零警告）

## [0.3.2] - 2026-08-05


### 新增

#### 外部下载工具：工具原理说明 + 高级设置

- 背景：外部下载工具此前只有 URL + 文件名输入，缺少品牌下载器的进阶能力（自定义 UA / 并发线程 / 分片 / 限速），也无法向用户说明其工作原理
- 改动：
  - `src-tauri/src/commands/tools/types/download.rs`：`DownloadFileParams` 新增可选字段 `user_agent` / `max_threads` / `chunk_count` / `max_speed`
  - `src-tauri/src/http.rs`：新增 `build_client_with_user_agent()`，支持自定义 UA 构建请求客户端（保留代理 / TLS 配置）
  - `src-tauri/src/minecraft/download/config.rs`：`DownloadManagerConfig` 新增 `user_agent` 字段与 `apply_overrides()`（按任务覆盖线程数 / 分片数 / 限速 / UA，`None` 保持原值）
  - `src-tauri/src/minecraft/download/manager/core.rs`：`from_config` 在配置含自定义 UA 时构建对应客户端
  - `src-tauri/src/minecraft/download/session.rs`：新增 `start_grouped_with_manager()`，支持注入自定义 manager
  - `src-tauri/src/commands/tools/download.rs`：`download_file` 读取高级设置 → `apply_overrides` → 注入 manager 下载
  - 前端：新增 `src/composables/external-settings.ts`（localStorage 持久化）；`download.ts` 的 `downloadFile` 支持 `DownloadFileSettings` 传参；`ExternalDownload.vue` 新增「工具原理」说明区与「高级设置」折叠面板（自定义 UA / 并发线程数 / 单文件分片数 / 限速 MB/s / 恢复默认）
- 验证：`cargo test --all-features` 181 个测试全部通过；`vue-tsc`、`eslint`、`vite build` 通过


### 新增

#### FRP 后端内联测试拆分为独立测试文件

- 背景：`detect.rs`、`frpc_config.rs`、`provider_system.rs`、`auth/pkce.rs`、`api_spec/http.rs` 等在源码文件末尾内联了 `#[cfg(test)] mod tests { ... }`，与项目"测试独立成文件"的约定不符
- 改动：
  - 上述 5 个文件改为 `#[cfg(test)] #[path = "xxx_tests.rs"] mod tests;` 引用方式
  - 新增独立测试文件：`detect_tests.rs`、`frpc_config_tests.rs`、`provider_system_tests.rs`、`auth/pkce_tests.rs`、`api_spec/http_tests.rs`
- 验证：`cargo test --all-features` 177 个测试全部通过


### 新增

#### 系统托盘：打开主页面 / 检查更新 / 退出

- 背景：此前关闭只能通过前端 X 按钮走 `handleClose`，缺少常驻托盘入口，无法在不退出进程的前提下收起主界面
- 改动：
  - `src-tauri/Cargo.toml`：tauri 启用 `tray-icon`（+`image-ico`）feature
  - `src-tauri/src/tray.rs`（新增）：托盘右键菜单三项——打开主页面（`unminimize`+`show`+`set_focus`）/ 检查更新（emit `tray-check-update`，前端复用 `checkForUpdate`）/ 退出（`cleanup_and_exit`）；左键单击托盘图标同样打开主界面；图标复用 `Images/icon.ico`
  - `src-tauri/src/lib.rs`：`setup` 中创建托盘；注册 `tray::request_exit` 命令
  - `src/components/layout/TopNavLayout.vue`：`useTauriEvent` 监听 `tray-check-update` 触发检查更新
- 验证：`cargo check/clippy` 通过


### 新增

#### 关闭主界面退出选择框，行为可持久化 + 设置页可改

- 背景：点击关闭时希望让用户选择"直接退出 / 保留托盘关闭主界面"，可勾选"下次不再提醒"记住本次选择，且可在设置页修改
- 改动：
  - 配置新增 `close_behavior`（`ask` 每次询问 / `tray` 保留托盘 / `exit` 直接退出，默认 `ask`）：`AppConfig` + `save/load` + `ConfigPatch/ConfigSnapshot/fields/validate` + 前端 `ConfigPatch`
  - `src/components/layout/ExitConfirmDialog.vue`：两项选择 + "下次不再提醒，记住本次选择"复选框；勾选后通过 `applyConfig({ closeBehavior })` 持久化
  - `src/components/layout/TopNavLayout.vue`：`handleClose` 按 `close_behavior` 分流——ask 弹框 / tray 隐藏窗口 / exit 直接退出
  - `src/views/settings/SettingsPersonal.vue`：新增「主界面 → 关闭主界面时」下拉（每次询问 / 保留托盘 / 直接退出）
  - `src-tauri/resources/defaults/config.ini`：`[General]` 补充 `close_behavior`（默认 `ask`）与注释，保证通过 `sync_config` 的 `merge_missing_from` 自动合并进老用户已存在的配置文件
- 验证：`vue-tsc --noEmit`、`eslint`、`vite build` 通过，Vue 文件均 ≤300 行


### 新增

#### 托盘退出弹确认框 + 更新日志 Markdown 渲染 + 退出框样式优化

- 背景：托盘右键"退出"此前直接触发退出，不走确认框；且主窗口处于隐藏/托盘状态时弹窗可能不可见；更新日志以纯文本 `whitespace-pre-line` 展示，无法呈现 Markdown 结构；退出确认框内容（左下复选框 + 右下按钮）在小宽度下会换行
- 改动：
  - `src-tauri/src/tray.rs`：托盘「退出」先 `open_main_window`（show + set_focus）保证主界面在最前，再 emit `tray-request-exit`
  - `src/components/layout/TopNavLayout.vue`：`tray-request-exit` 监听改走 `handleClose()`（与 X 按钮一致按 `close_behavior` 分流，`ask` 时弹出选择框）
  - `package.json` + `src/utils/markdown.ts`（新增）：引入 `marked`（渲染）+ `dompurify`（消毒），封装 `renderMarkdown`，解决云端更新日志的 XSS 风险
  - `src/components/about/UpdateDialog.vue`：更新日志 `notes` 由纯文本改为 `renderMarkdown` 渲染的 HTML（`v-html` + 作用域样式），支持标题/列表/代码块/链接
  - `src/components/layout/ExitConfirmDialog.vue`：标题与正文加 `whitespace-nowrap` 防换行、正文改 `text-gray-500`，底部复选框与按钮文案收窄并加 `flex-none`，避免挤压换行
- 验证：`cargo check/clippy`、`vue-tsc --noEmit`、`eslint`、`vite build` 通过，Vue 文件均 ≤300 行


### 新增

#### 更新日志 Markdown 链接禁止页面内跳转，改走系统浏览器

- 背景：更新日志渲染的 GitHub commit 等外链在 webview 内直接导航，导致跳出 SPA 页面且无法返回
- 改动：
  - `src/utils/markdown.ts`：自定义 marked renderer 为 http(s) 链接注入 `target="_blank" rel="noopener noreferrer"`；新增 `handleMarkdownLinkClick` 事件委托——拦截 `a[href]` 点击，`preventDefault` + 通过 `@tauri-apps/plugin-shell` 的 `open` 调用系统默认浏览器打开，从渲染层与交互层双重禁止页面内跳转
  - `src/components/about/UpdateDialog.vue`：更新日志容器绑定 `handleMarkdownLinkClick`
- 验证：`vue-tsc --noEmit`、`eslint`、`vite build` 通过，Vue 文件均 ≤300 行


### 新增

#### 更新日志链接点击走系统浏览器并加 toast 提示

- 背景：Markdown 外链点击虽已改走系统浏览器，但无任何反馈，用户不知道发生了什么
- 改动：
  - `src/utils/markdown.ts`：`handleMarkdownLinkClick` 调用 `open` 成功后 `toastInfo('已在系统浏览器中打开')`，失败时 `toastError('打开外部链接失败')`
- 验证：`vue-tsc --noEmit`、`eslint` 通过


### 新增

#### NAT 类型检测流程控制台日志

- 背景：联机设备侧边栏的端口/NAT 类型检测（STUN 探测）流程不透明，需要观察各阶段以便排查
- 改动：
  - `src/utils/online/detect.ts`：`detectNatTypeWithStun`/`detectNatType` 增加 `[NAT]` 前缀控制台日志——使用的 STUN 服务器列表、每个 ICE candidate（含地址:端口）、gathering 状态流转、超时兜底、推断结果与耗时
- 验证：`vue-tsc --noEmit`、`eslint` 通过


### 新增

#### frp 厂商 OAuth2 回调后自动将启动器窗口置于最前

- 背景：用户点击厂商授权页后浏览器跳回本地回调端口，但启动器窗口被浏览器盖住，用户看不到认证结果
- 改动：
  - `src-tauri/src/commands/frp/auth/oauth2/flow.rs`：`start_oauth2` 新增 `app: &tauri::AppHandle` 参数，收到回调后（token 交换前）`unminimize + show + set_focus` 将主窗口置顶聚焦
  - `src-tauri/src/commands/frp/manager/auth_actions.rs`：`start_oauth2` action 由 `_app` 改 `app` 传入 AppHandle
- 验证：`cargo check/clippy` 通过


### 修复

#### FRP 日志 ANSI 转义序列乱码清理

- 背景：frpc 输出带颜色控制符（如 `ESC[1;34m`、`ESC[0m`），前端日志区直接显示为 `[1;34m` 乱码
- 改动：
  - `src-tauri/src/commands/frp/log_redact.rs`：新增 `strip_ansi_sequences()`，用 CSI 正则（`\x1b\[[0-9;?]*[ -/]*[@-~]`）移除颜色/光标控制序列
  - `src-tauri/src/commands/frp/process/capture.rs`：`capture_stream` 在脱敏前先清洗 ANSI 序列
  - `src-tauri/src/commands/frp/log_redact_tests.rs`：补充 ANSI 清洗用例
- 验证：`cargo test --all-features` 181 个测试全部通过


### 修复

#### 隧道启停加载状态按隧道 ID 区分

- 背景：点击某条隧道「启动」，其他隧道的启动/停止按钮也进入加载态，缺少按隧道区分的判断
- 改动：
  - `src/stores/frp/tunnelSlice.ts`：新增 `tunnelActionTunnelId`，启停时记录对应隧道 ID，finally 清空
  - `src/components/frp/TunnelManager.vue`：向列表透传 `actionTunnelId`
  - `src/components/frp/TunnelList.vue`：按钮加载态改为仅当 `actionLoading && actionTunnelId === tunnel.id` 时展示
- 验证：`vue-tsc --noEmit`、`eslint`、`vite build` 通过


### 修复

#### SeedMap 组件 v-model 属性名还原为 kebab-case

- 背景：`SeedMap.vue` 中 `SeedMapControls` / `SeedMapSidebar` 的 `v-model:userX` 等 camelCase 写法与 Vue 组件 props 定义不符，还原为 kebab-case（`v-model:user-x` 等）
- 改动：`src/views/tools/data/SeedMap.vue` 的 `v-model` 属性统一还原为 kebab-case
- 验证：`vue-tsc --noEmit` 通过


### 修复

#### 托盘「退出」改为直接退出，不弹确认框

- 背景：托盘退出此前转交前端走 `handleClose()`，`ask` 模式下会弹出确认框，与"托盘退出即退出"的预期不符
- 改动：
  - `src-tauri/src/tray.rs`：「退出」菜单直接调用 `cleanup_and_exit(app)`（统一清理 frpc 隧道 / TUN 虚拟网卡 / 保存配置后退出），不再 emit `tray-request-exit`
  - `src/components/layout/TopNavLayout.vue`：移除已无触发方的 `tray-request-exit` 监听
- 验证：`cargo check/clippy`、`vue-tsc --noEmit`、`eslint` 通过


### 修复

#### 关闭 picker 子窗口误触发主进程退出确认

- 背景：关闭 picker:// 选择器子窗口（如端口选择弹窗）的按钮时，错误地触发了主进程的退出确认（`window-close-requested` 弹窗）
- 根因：`on_window_event` 的 `CloseRequested` 拦截未区分窗口，对所有窗口（含 picker 子窗口）都执行 `prevent_close` + 按 `close_behavior` 分流，子窗口关闭被误当作主窗口关闭
- 改动：
  - `src-tauri/src/lib.rs`：`on_window_event` 开头增加 `if window.label() != "main" { return; }`，关闭拦截与 DevTools 状态重置仅作用于主窗口，picker 等子窗口关闭直接放行
- 验证：`cargo check/clippy` 通过


### 修复

#### 选择"保留托盘"主界面不关闭：capabilities 缺 window hide 权限

- 背景：退出确认框选"保留托盘"后主界面窗口不隐藏（"直接退出"正常，因走 `request_exit` 命令不依赖窗口权限）
- 根因：Tauri 2 中 `core:window:default` 不包含 `hide`，而 `capabilities/migrated.json` 未声明 `core:window:allow-hide`，前端 `appWindow.hide()` 被后端拒绝
- 改动：
  - `src-tauri/capabilities/migrated.json`：补 `core:window:allow-hide`
  - `src/components/layout/TopNavLayout.vue`：`handleClose`/`onExitConfirm` 的 `hide()` 加 `.catch` 失败兜底 toast
- 验证：`vue-tsc --noEmit`、`eslint` 通过


### 修复

#### Windows 构建不再引入官方 updater plugin（条件编译）

- 背景：自动更新为双轨实现——Windows 便携版走自实现 updater（`install_windows.rs`，updater.exe 替换 + 退出延迟安装），官方 `tauri-plugin-updater` 仅 macOS/Linux 使用（`install_unix.rs` 转发），此前 Windows 也一并编译链接官方 plugin，白白增大体积
- 改动：
  - `src-tauri/Cargo.toml`：`tauri-plugin-updater` 从 `[dependencies]` 移到 `[target.'cfg(not(target_os = "windows"))'.dependencies]`
  - `src-tauri/src/lib.rs`：`.plugin(tauri_plugin_updater::Builder::new().build())` 包 `#[cfg(not(target_os = "windows"))]`
  - `src-tauri/capabilities/updater.json`（新增）：`updater:default` 权限独立成文件并 `platforms: ["linux","macOS"]`；`capabilities/migrated.json` 移除 `updater:default`（Windows 上该权限不存在会致 tauri-build 失败）
- 验证：`cargo check/clippy`（Windows 目标）通过


### 修复

#### 关闭路径清理缺口：Alt+F4 绕过 handleClose / frpc 残留 / TUN 未显式停止

- 背景：此前退出清理（保存配置 / 联机退房 / applyPendingUpdate）只在前端 X 按钮的 `handleClose` 内执行，Alt+F4、任务栏关闭等会绕过它；且在退出时 frpc 隧道进程、TUN 虚拟网卡也没有显式清理，可能残留 frpc.exe、TUN 网卡
- 改动：
  - `src-tauri/src/lib.rs`：`on_window_event` 的 `CloseRequested` 由"仅打日志"改为 `api.prevent_close()` 拦截，并按 `close_behavior` 分流——tray 隐藏窗口 / exit 执行退出清理 / ask 向后端 emit `window-close-requested` 转交前端弹框；该钩子覆盖 Alt+F4 等绕过前端的关闭路径
  - `src-tauri/src/tray.rs`：`cleanup_and_exit` 统一清理——遍历 frpc 进程表停所有隧道、`virtual_lan_bridge.take().stop()` 停止 TUN、`save_config` 保存配置，再 `app.exit(0)`
  - `src-tauri/src/commands/frp/process/mod.rs`：新增 `stop_all_tunnels`（迭代全局 `RUNNING` 表逐个 `stop_tunnel`）
  - 前端 `request_exit` 命令调用点位于 `doExit` 末尾，保证先保存配置/联机退房/待安装更新，再由后端兜底清理
- 验证：`cargo check/clippy` 通过


### 修复

#### 联机房间切页被销毁：保活提升为全局 store 定时器，脱离页面生命周期

- 背景：keepalive 定时器绑定在 RoomHostPanel 的 useRoomHost 上，从联机页切到其他页面（如设置页）会导致 Online.vue 卸载 → `stopTimers()` 停止 30s 保活，离开超过服务端 `keepalive_timeout`(120s) 后房间被判失联销毁；切回时因 store 中 role 仍为 host、activeCategory 是组件内 ref 默认回 create，出现"侧边栏高亮创建、内容区却显示房间详情"的错位
- 改动：
  - `src/stores/online.ts`：新增全局保活定时器（`GLOBAL_KEEPALIVE_INTERVAL=30s`），在 store 层运行，不依赖任何组件生命周期；role='host' 才上报，捕获 `RoomClosedError` 时 `resetRoomState` + toast
  - `src/composables/useRoomHost/useRoomHostPolling.ts`：`startTimers` 移除保活定时器（全局已承担），保留 `doKeepalive` 供断连恢复补发；`stopTimers` 同步清理
  - `src/composables/useRoomHost.ts`：`onRoomClosed` 回调瘦身为组件侧清理（stopTimers/lan.stop/hostMesh.close/setRoomKey(null)），`resetRoomState` + toast 交由全局保活统一处理，避免双弹窗
  - `src/composables/useOnlineNav.ts`：`isReady` watch 中若已进入房间则房间详情优先（覆盖重挂载时 role 保留但 activeCategory 默认 create 的错位）
- 验证：`vue-tsc --noEmit`、`eslint`、`vite build` 全部通过


### 修复

#### 联机房间失联被销毁：keepalive 失败可感知 + 房间关闭主动退出 + 断连自动补发

- 背景：房主 keepalive 业务失败被静默吞掉（`result.code !== 1` 直接 `return null`），用户看起来"一直在保活"，实际服务端超过 `keepalive_timeout`(120s) 未收到有效上报即判定失联关房（本次 V8MY2S 房间命中失联条件：仅创建后一次 keepalive 成功，之后无上报，23:36 被清理，23:39 的请求才报"房间已关闭"）
- 改动：
  - `src/stores/online/roomActions.ts`：`keepalive()` 不再静默返回 null——`code=1001` 抛出 `RoomClosedError`，其余业务失败抛出普通 Error
  - `src/composables/useRoomHost/useRoomHostPolling.ts`：`doKeepalive` 捕获 `RoomClosedError` 后停止轮询并触发 `onRoomClosed` 回调；新增 `RoomHostPollingOptions` 注入
  - `src/composables/useRoomHost.ts`：注入 `onRoomClosed`（停止轮询、`lan.stop`、`hostMesh.close` + `setRoomKey(null)`、`store.resetRoomState`、toast 提示）；`cloudConnected` 由断开恢复为连接时，除 `startTimers()` 外立即补发一次 doKeepalive/pollParticipants/pollAnswers，避免在 120s 窗口内漏报被判失联
- 验证：`vue-tsc --noEmit` 通过、`eslint` 通过


### 修复

#### 联机房间详情列表显隐优化 + BackToTop 去 tooltip

- 背景：上一轮将参与者列表改为恒渲染（空态提示），本次按反馈恢复「有参与者才显示」的 if 守卫，并补充入场动画；封禁列表、白名单列表同步做显隐治理；右下角全局返回顶部按钮去掉 tooltip
- 改动：
  - `src/components/online/RoomHostPanel.vue`：`ParticipantList` 恢复 `v-if="room.participants.length > 0"`，`BannedList` 增加 `v-if="bannedList.length > 0"`，两者外层包 `<Transition>`（淡入 + 上移，500ms ease-out），有人加入时平滑出现
  - `src/components/online/ParticipantList.vue`：条目改用 `<TransitionGroup>` + `participant` 过渡（enter 淡入下移 400ms cubic-bezier、leave 右移淡出、move 平滑重排），踢出/加入不再僵硬
  - `src/components/online/WhitelistEditor.vue`：未启用时仅显示「启用白名单」开关与说明；启用后才显示添加输入框；白名单列表仅在「启用且已有条目」时出现，空状态仅在启用时展示
  - `src/components/common/BackToTop.vue`：移除外层 `Tooltip`（返回顶部按钮上滑功能），保留按钮涟漪/光晕/滑入动画
- 验证：`vue-tsc --noEmit` 通过、`eslint` 通过、Vue 文件均 ≤300 行


### 修复

#### FRP 厂商隧道导入端口固定 7000 与 TCP 检测参数类型错误

- 背景：联机房间详情不显示参与者列表（participants 为空时列表区整块隐藏）；FRP 从厂商同步隧道时服务端端口被固定写成 7000；隧道创建表单 TCP 检测报 `参数解析失败: invalid type: string "17000", expected u16`
- 改动：
  - `src/components/online/RoomHostPanel.vue`：`ParticipantList` 移除 `v-if="participants.length > 0"` 守卫，空时也渲染列表区
  - `src/components/online/ParticipantList.vue`：空列表时显示「暂无参与者加入」空状态（icon + text 垂直水平居中，符合空状态规范）
  - `src/components/online/RoomHostPanel.vue` / `RoomGuestPanel.vue`：「剩余时间」增加 Tooltip 说明房间保留时间机制（超时无人加入自动清退；正常游玩自动续期保留）
  - `src-tauri/src/commands/frp/api_spec/executor.rs`：`resolve_field` 真正实现 `split` 拆分（此前仅占位直接返回原始值），`map_tunnels` 中 `serverHost` 取拆分后第 0 段、`serverPort` 取第 1 段；`get_field` 调用同步补 `None`
  - `src/components/frp/RemoteTunnelSync.vue`：导入端口改用 `Number()` 解析并去掉 `|| 7000` 兜底，服务端端口无效时中止导入并 toast 提示
  - `src/components/frp/TunnelCreateForm.vue`：TCP 检测端口 `Number(form.serverPort)` 强转（Input 组件会把数值字符串化，此前字符串 `"17000"` 无法反序列化为 `u16`）
- 根因：`resolve_field` 的 split 分支从未拆分 `host:port`，前端 `parseInt("节点域名:17000")` 为 `NaN` 被 `|| 7000` 兜底掩盖；Input 组件 `String(val)` 化导致端口以字符串传给后端 `u16` 参数
- 验证：`cargo check` 通过、`cargo test` 152 passed、`vue-tsc --noEmit` 通过


### 修复

#### 复用全局 formatBytes：消除 UpdateDialog 局部实现遮蔽

- 背景：审计（docs/fix-debug/05-utils-reuse.md P0）发现 `UpdateDialog.vue` 组件内定义的局部 `formatBytes` 遮蔽了 `utils/format.ts` 全局实现，且展示口径不一致（局部用 1 位小数，全局默认 2 位小数）
- 改动：
  - `src/components/about/UpdateDialog.vue`：删除局部 `formatBytes`，新增 `import { formatBytes } from '@/utils/format'`，下载大小文案统一走全局实现
- 设计决策：复用既有全局工具而非保留局部副本，符合「可复用函数必须提取到单独 TypeScript 文件」项目约定
- 验证：`npx vite build` 通过（exit 0）


### 修复

#### 后端复用重构（审计 A2-A5，docs/fix-debug/03-backend-architecture.md）

- 背景：审计发现 frp/paths.rs 本地 ensure_dir、version/mod.rs sanitize_version_id、offline.rs PNG 校验、detect.rs setup.ini 手写解析均与 utils/已有工具重复
- 改动：
  - `commands/frp/paths.rs` + `commands/frp/mod.rs`（A2）：删除本地 `ensure_dir`，re-export 改为 `utils::fs::ensure_dir`
  - `commands/version/mod.rs`（A3）：`sanitize_version_id` 复用 `utils::path::sanitize_file_name`，保留 `:` 与 components 增量校验
  - `commands/auth/account/offline.rs` + `commands/auth/authlib/helpers.rs`（A4）：`save_custom_skin` 改为调用 `read_png_file`（可见性 `pub(super)`→`pub`，helpers 模块 `pub(crate)`）
  - `commands/version/list/detect.rs`（A5）：setup.ini `Type=` 手写解析改为复用 `VersionSetup::load`，`Release`/`Unknown` 继续降级检测
  - `minecraft/version/state.rs`：`VersionType::from_str` 补充 `old_alpha`/`old_beta` 别名映射（原 detect.rs 依赖此映射，复用后保持行为等价）；`state_tests.rs` 新增别名测试
- 验证：`cargo check`、`cargo test --lib` 通过（151+1 passed / 0 failed）、`cargo clippy` 零警告


### 修复

#### 前端复用重构（审计 P1，docs/fix-debug/05-utils-reuse.md）

- 背景：审计发现前端 4 处局部 `toLocaleString` 时间戳格式化、4 处手写 listen/unlisten 样板、9+ 处散落 `navigator.clipboard` 均与既有工具重复
- 改动：
  - `utils/format.ts`：`formatTimestamp` 新增可选 `options.withSeconds` 参数（默认行为不变），展示格式差异通过参数解决，禁止调用方手拼
  - `views/Versions.vue`、`views/tools/archive/ArchiveManager.vue`、`views/tools/data/ScreenshotManager.vue`、`components/online/BannedList.vue`：删除局部 `formatDate`/`toLocaleString`，改用 `formatTimestamp`
  - `composables/useCommunityDownload.ts`、`composables/useExportTab.ts`、`views/version-settings/JavaDownloadBar.vue`：手写 listen/unlisten 样板改用 `useTauriEvent`
  - `composables/useVirtualLan.ts`：后台 TUN 数据流监听改用 `onGlobalEvent`（全局单例，永不 unlisten，消除竞态）
  - 新增 `utils/clipboard.ts`（统一剪贴板，支持可选 toast）；`utils/seedmap/format.ts` 的 `copyToClipboard` 提升为 re-export；替换 9 处散落 `navigator.clipboard` 调用（HttpLogViewer/VirtualIpCard/RoomHostPanel/RoomGuestPanel/DeviceCodeModal/Modal/ResourceDetailHeader 等）
- 设计决策：优先复用既有工具；`useVirtualLan` 的 TUN 数据流属后台持续推送，选 `onGlobalEvent` 而非 `useTauriEvent`；`DeviceCodeModal` 保留返回值驱动复制失败状态的语义
- 验证：`npx vue-tsc --noEmit`（exit 0）、`npx vite build`（exit 0）


### 修复

#### 前端头部注释精简（审计 P1，docs/fix-debug/06-frontend-header-comments.md）

- 背景：项目规范要求前端 ts 文件头部注释最多 8 行（许可证例外）；审计扫描发现 131 个文件超限，其中 20 个头部 ≥24 行为 P1（另 111 个 9~23 行为 P2 待后续处理）
- 改动（20 文件，删约 510 行注释）：
  - 头部精简至 ≤8 行一句话职责；删除重构背景/变更历史/ASCII 数据流图/JSON-XML 样例/协议格式文档
  - 7 个文件的重要设计信息迁移到函数/类型级 `/** */` 注释（信息无丢失）：crypto.ts（帧性能）、parser.ts（JSON/XML 样例）、generatorWorker.ts（WASM API 清单）、terrainShading.ts（渲染算法）、protocol.ts（帧布局与子类型）、structures.ts（queryMode 语义）、developer.ts（解锁触发链）
  - 关键约束保留：useVirtualLan（onGlobalEvent 永不 unlisten）、useGlobalTauriEvent（unlisten 竞态消除）、useWebRTC/useWebRTCMesh（无 trickle ICE、AES-GCM）
- 验证：`npx vue-tsc --noEmit`（exit 0）、`npx vite build`（exit 0）


### 架构

#### 大文件拆分（审计阶段二批6，docs/fix-debug/04-file-line-limits.md P1+P1 提级）

- 背景：项目规范要求单文件 ≤300 行（Vue 组件为硬性约束）；审计定位 9 个 P1 文件（>=400 行）与 5 个超限 Vue 组件
- 前端拆分（14 文件）：
  - `utils/seedmap/generatorWorker.ts`（752 行）拆为 `wasm-bindings.ts`（WASM 绑定/内存安全/init）+ `tile-render.ts`（biome 上色/地形阴影）+ `structure-search.ts`（结构查找）+ 主文件（消息队列与分发）；WASM 单例经 `wasm` 引用对象共享
  - `views/tools/data/useSeedMap.ts`（546 行）拆为 `useSeedMap/map-events.ts`（事件处理）+ `useSeedMap/map-init.ts`（地图初始化）+ 主文件（299 行）
  - `composables/useRoomHost.ts`（432→149）拆 `useRoomHost/`（轮询/动作切片）；`useResourceDownload.ts`（413→299）拆 `useResourceDownload/`（进度/依赖确认切片）；`stores/frp.ts`（403→35）拆 `providerSlice/tunnelSlice/logsSlice`；`useSkinOperations.ts`（363→146）拆 `useSkinState/useSkinActions`
  - 5 个 Vue 组件提级拆分：CreateRoomForm（→useCreateRoomForm）、CustomLayoutSection（→useCustomLayout）、SeedMap（→SeedMapControls/SeedMapSidebar）、AuthCenter（→useFrpAuthCenter）、TunnelManager（→TunnelList）
- 后端拆分（13 文件）：
  - `commands/frp/types.rs`（722→模块聚合）拆为 `types/tunnel.rs`/`types/provider.rs`/`types/auth.rs`/`types/api_spec.rs` + `types/mod.rs` re-export（对外 `types::xxx` 路径不变）
  - `commands/frp/provider.rs`（426）拆 `provider_system.rs`/`provider_external.rs`；`auth/mod.rs`（420）拆 `auth/handlers.rs`；`sandbox.rs`（408）拆 `sandbox/validate.rs`/`sandbox/adapter.rs`（`#[path]` 测试声明同步调整）
- 设计决策：全部保持对外 API/消息契约/序列化结构不变；前端优先 composable 提取（状态与模板强耦合场景）、子组件承接模板片段（双向绑定用 defineModel）；后端保持 `pub use` re-export 路径兼容
- 验证：`npx vue-tsc --noEmit`（exit 0）、`npx vite build`（exit 0）、`cargo check`（exit 0）、`cargo test --lib`（152 passed / 0 failed）、`cargo clippy --lib`（零警告）


### 架构

#### SDK DES 加解密统一（审计 A1，docs/fix-debug/03-backend-architecture.md）

- 背景：审计发现 SDK DES 加解密「锁取 → encrypt/decrypt_token → 错误映射」样板被复制 4 份（frp/auth/storage.rs、minecraft/community/secure_storage.rs、minecraft/auth/storage/mod.rs、minecraft/online/storage.rs），且错误语义不一致（前两者失败视为无数据返回 None，后两者返回 Result）
- 改动：
  - 新增 `utils/sdk_crypto.rs`：`encrypt_with_sdk` / `decrypt_with_sdk`（Result 语义，供 auth/online）/ `decrypt_with_sdk_optional`（失败 log_warn + None，供 frp/community）；统一错误消息带调用方上下文
  - 4 个调用方改用公共 helper，删除各自复制实现（约 72 行）
- 设计决策：保留两种语义变体而非强行统一为一种，避免改变各调用方现有容错行为（最小修改）；helper 放 utils 层（纯 SDK 操作，无业务依赖）
- 验证：`cargo check`（exit 0）、`cargo test --lib`（152 passed / 0 failed）、`cargo clippy --lib`（零警告）


### 架构

#### 分发注册表迁移至 commands 域（审计 B1/B2，docs/fix-debug/03-backend-architecture.md）

- 背景：审计发现 `utils/*_manager.rs`（17 个分发注册表）反向依赖 `commands` 业务层，与 `commands::* → utils::dispatcher` 形成双向耦合；tools 域分发器位置与其余域不一致
- 改动（17 域迁移，注册表内容一字未改）：
  - A 组（system/config/meta/skin/image_cache/java/sdk/plugins 8 域）：manager 物理移动至对应 commands 域（如 `utils/system_manager.rs` → `commands/system/manager.rs`），commands mod.rs 加 `pub(crate) mod manager;` 转发改 `manager::dispatch`
  - B 组（community/frp/online/version_list/install/export/progress/launch 8 域）：同步迁移；`online_manager` 拆为 `commands/online/manager/`，`load_creds_with_auto_refresh` 引用方（4 处）改 `commands::online::manager::...`；`version/progress.rs` 单文件转子目录 `progress/`（mod.rs + manager.rs）
  - 补迁 `utils/version_mods_manager.rs` → `commands/version/mods/manager.rs`（11 个 action 注册表），引用注释同步更新
  - `utils/mod.rs` 删除全部 17 个 `pub mod *_manager;` 声明，仅保留纯工具模块（cache/dispatcher/format/fs/path/sdk_crypto/signaling/tun/version 等）
  - tools 域确认无需迁移（`tools_manager` 本体本就在 `commands/tools/mod.rs`）
  - 为 `DeviceStatus.device_pk` 补 `#[allow(dead_code)]`（`#[serde(skip)]` 设计安全字段，仅供后端内部逻辑使用）
- 设计决策：采用「物理移动 + 相对导入 + mod.rs 转发」而非拆分重构，注册表逻辑零改动，行为完全等价；utils 层自此不再引用任何 commands 业务符号
- 验证：`cargo check`（exit 0）、`cargo test --lib`（152 passed / 0 failed）、`cargo clippy --lib`（零警告）


### 架构

#### 阶段三批9：行数治理 51 个文件拆分至 ≤300（前端 13 / 后端 32，docs/04-file-line-limits.md）

- 背景：批 6 已拆分 P1 档（>=400 行），本批收敛全部 300-399 档超限文件，落实「单文件 ≤300 行」规范
- 前端拆分（13 文件）：`useModList`（383→261）、`stores/plugins`（381→98）、`stores/online/roomSlice`（358→切片）、`useWebRTCMesh`（317→65）、`useDragDrop/handlers`（341→246）、`useExternalDownload`（303→112/source+task）、`sdk.ts`（302→域拆分 config/version/system/process/window/events）、`custom-layout/parser`（→json/xml/schema）、`seedmap/structure-search`（→chunk-finder/find-structures/find-specials）、`online/nat-type`（→detect/format）、`updater`（→check/install/state）、`personalization`（→域分组）、`types/frp`（→tunnel/provider/auth）；主文件 re-export 保持对外路径不变
- 后端拆分（28 文件 + version/install 收尾）：
  - minecraft 域：`skin.rs`→`skin/{avatar,cape,upload}`、`auth/storage/operations.rs`→`operations/{authlib,ms}`、（`load.rs`→分平台分支）、`launch/watcher`（mod.rs 完成 + 拆 process/scheduler）、`watcher/analyzer/crit1`（→collect/rules）、`community/modrinth`（→http/search/version_files）、`community/searcher`（→aggregate/sort）、`download/manager`（→mod+state）、`image_cache`（→store/download/cleanup）、`java/search`（→mod/platform/version）、`version/libraries/parse`（→mod/path/rules）、`launch/jvm_args`（→mod/build/rules）、`launch/skin_resourcepack`（→mod/generate/install）、`online/signaling/types`（→ice/room/session）
  - commands 域：`system/manager`（→mod+config/game_dir/developer/updater）、`auth/meta_manager`（→mod+offline/microsoft/authlib）、`commands/skin`（→mod+list/upload/cape）、`tools/picker_window`（→mod+scheme）、`apply_config/apply`（→mod+fields）、`tools/cleanup`（→mod+fs）、`modpack_stages/parsers`（→curseforge/modrinth/hmcl）、`version/launch`（→mod+build/spawn）、`concurrent/detect`（→collect/rules）、`tools/resourcepack`（→list/convert）、`frp/api_spec`（→registry/executor）、`frp/auth/oauth2`（→mod/exchange）、`deeplink/protocol`（→windows/linux）、`config.rs`（→load/save）、`storage/mod.rs`（→paths/fs）；`version/install/mod.rs` 拆分半成品修复并压至 238 行
  - 全部保持对外 API/消息契约/序列化结构不变，`pub use` re-export 路径兼容；`#[path]`/`#[cfg]` 测试声明与平台门控同步调整
- 设计决策：纯类型/纯 RESP 文件（signaling/types、types/frp.ts）按域拆且给出收益说明；平台分支（windows/linux）独立文件消除死代码告警；主文件转目录手工聚合不低于风险
- 验证：`npx vue-tsc --noEmit`（exit 0）、`npx vite build`（exit 0）、`cargo check`（exit 0 无新告警）、`cargo test --lib`（152 passed / 0 failed）、`cargo clippy --lib`（零警告）


### 架构

#### 前端头部注释 P2 档精简（审计批 8，docs/06-frontend-header-comments.md）

- 背景：项目规范要求前端 ts 文件头部注释 ≤8 行；前一阶段已清 P1 档（>=24 行），本批收敛 9~23 行档的 P2 剩余文件
- 改动：composables 系 24 个、stores/types/plugins/router/tutorials/config 系 22 个、utils/api/online/seedmap/useSeedMap 系 30 个共约 76 个文件头部注释精简至 ≤8 行（净删约 460 行）；`useTauriEvent` 等竞态防护约束、`useWatermarkData` 屏印缓存等关键设计信息保留在头部或迁移至函数/类型级 `/** */` 注释，`element-icons.ts` MIT 许可证头部豁免
- 验证：`npx vue-tsc --noEmit`（exit 0）


### 架构

#### 复用与架构收尾（审计批10，docs/05-utils-reuse.md 与 docs/03-backend-architecture.md）

- 背景：05 报告 P2 项——sha256 摘要实现复制 3 份、contains("..") 路径防御内联 7+ 处、utils::fs 公共工具不足
- 改动：
  - 新增 `utils/hash.rs::sha256_hex(&[u8])->String`，统一 `resources.rs` / `authlib/client/meta.rs` / `frp/binary/external.rs` 三处等价实现（入参与输出语义完全一致；hkdf_sha256 与数据字段未动）
  - `utils/path.rs` 新增 `is_safe_relative_path`：统一为 Path::components 段级 ParentDir 校验，替换 10 处内联 `contains("..")`（viewer/plugins_sandbox/version_json/version_mods_install/archive 三件/args/assets/shell_open/java_files）；`sanitize_file_name` 文件名净化语义与路径安全语义差异加注释标注（语义优化：`foo..bar` 类字面量片段放行，无穿越风险的真实 `..` 段仍全拒）
  - `utils::fs` 收敛 6 处低风险 create_dir_all/read_to_string（tools/download、tools/data_export、tools/version_json、community/modpack 安装系），消除冗余 map_err 包装
  - `certs.rs::validate_filename` 白名单语义与 `sanitize_file_name` 黑名单差异函数级注释文档化；plugin:fs 前端仅 1 处调用不抽象（调用方数量 <2）
  - 收尾两个 300-399 档遗漏文件：`certs.rs`（304 行，转为 `certs/` 目录并从 mod.rs 拆出 `pem.rs` PEM 解析）、`commands/frp/types/api_spec.rs`（303 行，转为 `api_spec/` 目录拆 `models`/`field_mapping`）；演进后前后端全量扫描零超限
- 验证：`cargo check`（exit 0 无新告警）、`cargo test --lib`（152 passed）


### 架构

#### Mod 入口净化：mod.rs 只保留「模块声明 + re-export」（docs/07-modrs-entry-only.md）

- 背景：审计批07 要求「凡是 mod.rs 入口文件有逻辑代码的必须脱离出来，只能作为入口文件」。全库 110 个 mod.rs 中 38 个纯入口合规、72 个含逻辑（A 类 = 目录已有子模块但入口夹带实现，B 类 = 单文件模块豁免）
- 改动（分 5 批收敛 59 项）：
  - 07-1 commands 域：tools/system/online/frp/auth/community/version/skin/plugins 各 manager 的 DISPATCHER 注册 + IPC 函数移入新建 dispatcher.rs，mod.rs 收敛为纯 re-export；删除 7 个纯转发 dispatcher.rs
  - 07-2 minecraft 域：launch/system/auth/community/download/image_cache/java/loaders/version 的 fn/impl/struct 移入对应子模块（新建 types.rs/manager.rs/entry.rs/pipeline.rs 等 36 个文件），mod.rs 仅入口；测试模块随逻辑迁移（`#[path]` 调整）
  - 07-3 frp 子域 + 顶层：ws/state/logger/storage/sdk/migrations/certs/signaling_manager/deeplink 逻辑移入 server.rs/core.rs/manager.rs/library.rs/router.rs 等
  - 07-4 version 域补充：export/install/script_export/zip/updater/apply_config 逻辑移入 api.rs/flow.rs/export.rs
  - 07-5 剩余：cleanup/picker_window/modpack_stages/archive/resourcepack/online auth/client/pipeline 逻辑移入子文件 + 2 处 clippy `module_inception` 修复（pipeline/runner、client/core）
- 关键约束（实证）：`#[tauri::command]` 函数无法移入子模块再 pub use 重导出（generate_handler 的 `__cmd__*` 宏仅模块内文本可见），命令转发函数保留在 mod.rs 并注释原因；E0364 可见性越界（重导出项可见性不得超过待导出项）；`mod linux/macos/windows;` 声明需 `#[cfg]` 门控（修复 window_title 缺门控导致的 Windows 编译失败）；`#[macro_export]` 宏移子模块需 `$crate::` 绝对路径
- 验证：`cargo check`（exit 0 无告警）、`cargo test --lib`（152 passed / 0 failed）、`cargo clippy --lib`（零警告）


### 重构

#### 后端测试代码拆分：8 个文件内联 mod tests 迁移至 xxx_tests.rs

- 背景：项目规范要求测试代码必须放到同目录 xxx_test.rs；审计（docs/fix-debug/01）发现 deeplink/security.rs、utils/client_type.rs、commands/frp/{types,sandbox,auth/flows,api_spec/{jsonpath,envelope,config_gen}}.rs 共 8 个文件以内联 `#[cfg(test)] mod tests { ... }` 携带 25 个测试函数（约 350 行），与其余 27 个文件的外部测试文件模式不一致
- 改动（16 文件）：8 个主文件删除内联测试块，改为 `#[cfg(test)] #[path = "xxx_tests.rs"] mod tests;`；同目录新增 8 个 `xxx_tests.rs` 测试文件（纯移动，业务逻辑零改动）
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过（exit 0）；`cargo test --manifest-path src-tauri/Cargo.toml --lib` 全部通过（151 passed / 0 failed，25 个用例无丢失）


### 重构

#### 后端：头部注释精简（27 个文件，>5 行 → ≤5 行）

- 背景：项目注释规范要求文件头部注释 ≤5 行（版权声明除外），审计（docs/fix-debug/02）扫描发现 27 个 .rs 文件超限（最长 20 行）
- 改动：精简 27 个文件头部 `//!` 模块注释至 ≤5 行；删除子模块罗列/历史背景/路径枚举/迁移策略清单等冗余；协议级内容保留在函数级 `///` 文档注释中
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过（exit 0，零警告）

## [0.3.0] - 2026-08-02

### 新增

#### CI：release_notes 上报 + release_url 指向 GitHub Release 页面 + S3 分片上传（>50MB）

- 背景：CI 上报 api-server 时此前不携带本次更新内容（`release_notes`）；`release_url` 的设计初衷即指向 GitHub Release 页面（tag 地址），不做任何替换/直链化，后续可人工跳转查看该版本发布页；大文件（>50MB）单次 PUT 直传存在超时/连接中断风险
- 改动：
  - **`.github/workflows/release.yml`**：`build-and-upload` job 的 checkout 改为 `fetch-depth: 0`（获取全量历史与 tag），新增「Generate release notes from commits」步骤（与 release job 同款逻辑：上一 tag → HEAD 的提交记录，无上一 tag 时取最近 50 条），通过 `GITHUB_ENV` 写入 `RELEASE_NOTES`；两个 ci-upload.cjs 调用追加第 8 个参数 `"$RELEASE_NOTES"`，随版本注册上报本次更新日志（启动器「检查更新」对话框展示）
  - **`api-server`**（详见 api-server/CHANGELOG.md）：`/v3/ci/presign-upload` 与 `/v3/ci/frp/presign-upload` 支持 `sizes` 字段并按 50MB 阈值返回分片上传凭证；新增 `/v3/ci/complete-upload` 完成分片合并（updater 与 frp 共用）
  - **`scripts/ci-upload.cjs`**：预签名请求携带 `sizes`；按服务端返回的 `multipart` 字段自动分流——分片上传按序 PUT 各分片（收集 ETag）后回传 `upload_id` + 分片列表完成合并，小文件与 .sig 保持单次 PUT；`release_url` 原样上报（GitHub Release tag 页面 URL，不做下载直链转换）
- 验证：api-server `cargo check` 通过（exit 0，无警告）；`node --check scripts/ci-upload.cjs` 通过（exit 0）

#### CI：Ubuntu 22.04 依赖升级为 Tauri v2 官方组合（修复 rust-clippy job 失败）

- 背景：GitHub Actions `rust-clippy` job（runs-on: ubuntu-22.04）执行 `cargo clippy --all-features -- -D warnings` 失败，根因是 Tauri v2 的 webkit2gtk 传递依赖 `soup3-sys v0.5.0` 构建需要系统库 `libsoup-3.0`，而 ci.yml 仍是 Tauri v1 时代依赖列表（缺少 libsoup-3.0-dev）
- 改动（2 个 workflow 文件）：`.github/workflows/ci.yml` 的 `rust-clippy` / `build` 两处 Linux 依赖安装步骤、`.github/workflows/release.yml` 的 `build-and-upload` Linux 依赖安装步骤，统一更新为 Tauri v2 官方 Ubuntu 22.04 组合（保守保留 `libwebkit2gtk-4.0-dev`，新增 `libsoup-3.0-dev` 等）：
  `sudo apt-get install -y libwebkit2gtk-4.0-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev`
- 验证：仅 workflow 依赖命令改动，无代码逻辑变更；`libsoup-3.0-dev` / `libayatana-appindicator3-dev` / `libxdo-dev` 均在 ubuntu-22.04 官方源可用

#### 开发者存储栏：补充 AppData 全局共享目录展示

- 背景：开发者页「存储」子页签此前只展示便携式目录（.Molaunch）与系统缓存，未展示 AppData 全局共享目录——而 certs 证书、providers frpc 二进制、frp_auth 认证 token、online 联机数据、auth.json 账号认证都存放在 `%APPDATA%/.Molaunch/` 下
- 改动（3 文件）：
  - **`src-tauri/src/commands/system/developer.rs`**：`StorageDirs` 新增 6 个字段——`appdataRoot`（全局共享根目录）/ `appdataCerts`（TLS 证书）/ `appdataProviders`（frpc 厂商二进制）/ `appdataFrpAuth`（FRP 认证 token）/ `appdataOnline`（联机数据）/ `appdataAuthFile`（账号认证文件）；`get_storage_dirs` 经 `storage::appdata` 模块填充，新增 `appdata_subdir_str` / `appdata_root_str` / `appdata_file_str` 三个辅助函数（APPDATA/HOME 环境变量缺失时返回空串，不 panic）
  - **`src/utils/api/developer.ts`**：`StorageDirs` 接口同步新增 6 个字段（camelCase）
  - **`src/views/settings/developer/StorageTab.vue`**：新增「AppData 全局共享」卡片（6 个条目，账号认证文件用「定位」、其余「打开」，空路径条目过滤不显示）；原「AppData 缓存」改名为「Minecraft 运行缓存」（与 AppData 全局共享区分，避免歧义）
- 设计决策：
  - **展示不创建目录**：仅 `appdata_subdir`（不 `ensure`），避免打开存储页就产生副作用；空路径过滤保证环境变量缺失时页面正常
  - **文件 vs 目录按钮区分**：auth.json 是文件用「定位」（revealInExplorer），其余目录用「打开」（openPath），与既有 config 条目一致
- 验证：`cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`npx vue-tsc --noEmit` 通过（exit 0）、改动文件 `npx eslint` 通过（exit 0）、StorageTab.vue 165 行（<300 约束）

#### Frp 认证存储迁移：keyring → SDK DES 加密文件（AppData 全局共享）

- 背景：FRP 厂商 token 原用 `keyring`（OS 密钥存储）保存，存在 Windows Credential Manager / macOS Keychain / Linux Secret Service。改为复用项目自有的 SDK 内置 DES 动态加密（与 CurseForge api_key / 联机凭证同一套加密），移除第三方 keyring 依赖
- 存储位置：token 存 **AppData 全局共享目录** `%APPDATA%/.Molaunch/frp_auth/{provider_id}.json`（macOS/Linux 为 `~/.config/Molaunch/frp_auth/`），与 frpc 厂商二进制（providers/）同级 —— 认证 token 属设备级共享数据（跨启动器实例共享，便携版换目录/更新不丢认证），非便携式实例数据
- 后端改动（9 文件）：
  - **`src-tauri/src/commands/frp/auth/storage.rs`**（重写）：keyring → AppData `frp_auth/{provider_id}.json` 加密文件
    - `TokenRecord` 结构（access_token / refresh_token / expires_at / scopes，serde 整体序列化）
    - 全局 `SDK_REF: OnceLock<Arc<TokioMutex<Option<SdkInstance>>>>` + `set_sdk()` 启动注入（与 CurseForge secure_storage 同一模式，异步锁取 SDK 后调 `encrypt_token` / `decrypt_token`）
    - `store_token_info`（加密写，`expires_in` 相对秒数换算绝对过期时间；Unix 0o600 权限）/ `load_token_record`（解密读，SDK 不可用或解密失败视为未认证 + log_warn）/ `delete_provider_auth`（删文件，撤销认证）
    - 保留 `now_secs` / `generate_state` 同步辅助；删除 `KEY_*` 常量与 `store_secret` / `load_secret` / `delete_secret` / `load_expires_at` / `load_scopes` 等 keyring 单字段 API
  - **`src-tauri/src/commands/frp/paths.rs`**：新增 `auth_file_path(provider_id)`（`ensure_appdata_subdir("frp_auth")`，APPDATA 环境变量缺失时降级回便携式目录）
  - **`src-tauri/src/commands/frp/auth/mod.rs`**：新增 `pub fn set_sdk(...)` 注入入口；`get_auth_status` / `refresh_token` / `load_token` 改 `load_token_record().await`；`revoke_auth` 改 `delete_provider_auth().await` 单次删除
  - **`src-tauri/src/commands/frp/auth/oauth2.rs` / `device_code.rs`**：`store_token_info` 调用加 `.await`
  - **`src-tauri/src/commands/frp/auth/api_key.rs`**：`store_secret` → `store_token_info(...).await`（API Key 作为 access_token 存文件）
  - **`src-tauri/src/lib.rs`**：启动时 `commands::frp::auth::set_sdk(app_state.sdk.clone())` 注入 SDK 引用
  - **`src-tauri/src/migrations/portable_to_appdata.rs`**：抽取通用 `migrate_dir()`，新增 `migrate_frp_auth()` —— 旧路径 `{base_dir}/frp/auth/` 启动时自动迁移到 AppData `frp_auth/`（复用既有 certs/providers 迁移策略：AppData 已有数据则删除旧目录、复制失败保留旧目录下次重试）
  - **`src-tauri/src/migrations/mod.rs`**：迁移项注释补充 frp_auth
  - **`src-tauri/Cargo.toml`**：移除 `keyring = { version = "3", features = [...] }` 依赖（及其平台后端 features 注释）
- 设计决策：
  - **SDK 全局引用注入而非逐函数传 state**：`get_auth_status` / `load_token` 等无 state 参数的公开函数签名不变，仿 secure_storage 用 `OnceLock` 注入，改动面最小且模式一致
  - **整体 JSON 加密而非逐字段**：单文件一次加密写、一次解密读，天然无并发竞态；`skip_serializing_if` 缺省字段不落盘
  - **降级策略**：写入时 SDK 不可用 → 返回错误（用户可见提示）；读取时 SDK 不可用 / 解密失败 → 视为未认证并 log_warn（用户重新认证即可，避免启动崩溃）
  - **AppData 全局共享而非便携式实例目录**：认证 token 与厂商账号绑定（设备级），frpc 厂商二进制也在 AppData（providers/），二者语义一致 —— 便携版换目录/更新版本不丢认证；启动迁移兜底旧路径数据
  - **不迁移旧 keyring 数据**：token 过期时间短、刷新即可重建，开发阶段无需迁移脚本
- 验证：`cargo check --lib` 通过（exit 0）、`cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`cargo test --lib` 151/151 通过（exit 0）

#### deeplink 注册/卸载工具 + Windows 便携版（安装版/便携版分离）

- 背景：后续打包策略下 **Windows 同时分发安装版（NSIS）与便携版（绿色版）**，macOS/Linux 维持单一产物。便携版未经过安装程序，无法自动注册 `molaunch://` 协议，需在代码内提供注册/卸载工具函数，并在设置页提供手动入口供用户抉择；CI 同步区分安装版 + 便携版两种产物
- 后端改动（4 文件）：
  - **新增 `src-tauri/src/deeplink/protocol.rs`**：跨平台协议注册/卸载/状态工具
    - `DeeplinkStatus` 结构（serde 序列化，camelCase）：registered / registeredExe / currentExe / platformSupported / message，供前端展示
    - `status()`：查询协议当前注册状态（含"已注册但指向其他路径"的移动场景提示）
    - `register()`（幂等）：Windows 写 `HKCU\Software\Classes\molaunch`（URL Protocol + DefaultIcon + shell\open\command，**免管理员权限**）；Linux 写 `~/.local/share/applications/*-handler.desktop` + xdg-mime 关联；macOS 不支持（协议由打包 Info.plist 的 CFBundleURLTypes 声明，`platform_supported` 返回 false）
    - `unregister()`（幂等）：Windows 删除整个 HKCU 键（键不存在按成功处理）；Linux 删除 desktop 文件
    - `auto_register()`：启动自动注册——已注册且指向当前 exe → 跳过（安装版场景，不重复写注册表）；已注册但指向旧路径 → 自动重注册（便携版被移动）；未注册 → 注册（便携版首次启动）
  - **`src-tauri/src/deeplink/mod.rs`**：声明 `mod protocol` + re-export `auto_register / register_protocol / protocol_status / unregister_protocol / DeeplinkStatus`
  - **`src-tauri/src/deeplink/router.rs`**：`init()` 中原 `#[cfg(debug_assertions)]` 的 dev 动态注册改为 `#[cfg(not(target_os = "macos"))]` 调 `protocol::auto_register()`（生产便携版/开发环境均幂等注册）
  - **`src-tauri/src/utils/system_manager.rs`**：新增 3 个 action——`get_deeplink_status` / `register_deeplink` / `unregister_deeplink`（注册/卸载后返回最新状态）
- 前端改动（4 文件）：
  - **`src/utils/api/system-manager.ts`**：`SYSTEM_ACTIONS` 新增 `GET_DEEPLINK_STATUS` / `REGISTER_DEEPLINK` / `UNREGISTER_DEEPLINK`
  - **`src/utils/api/developer.ts`**：新增 `DeeplinkStatus` 接口 + `getDeeplinkStatus()` / `registerDeeplink()` / `unregisterDeeplink()` 三个封装
  - **新增 `src/views/settings/developer/DeepLinkTab.vue`**（128 行）：状态卡片（已注册/未注册徽标 + 说明 + 当前程序路径/注册表登记路径/平台支持）+ 注册协议 / 卸载协议 / 刷新状态 三个按钮
  - **`src/views/settings/SettingsDeveloper.vue`**：新增 `deeplink` 子页签（LinkIcon，位于「系统信息」后），渲染 DeepLinkTab
- CI 改动（1 文件）：
  - **`.github/workflows/release.yml`**：Windows 矩阵 `args: --bundles nsis,portable`（NSIS 安装版 + portable 便携版同时产出）；locate 步骤定位 `*_portable.exe`；新增「Upload portable build (Windows only)」步骤用 `ci-upload.cjs` 以 `bundle_type=portable` 单独上传（与 NSIS 安装版并存，缺失时跳过不失败）；macOS/Linux 维持单一产物
- 设计决策：
  - **代码工具函数而非外部脚本**：注册/卸载逻辑内置于 `protocol.rs`，与 deeplink 模块共存，可在应用启动自动调用、也可由设置页触发，用户无需下载额外脚本
  - **幂等 + 移动场景自愈**：`auto_register` 检测到注册表指向旧路径（便携版被移动/更新路径变化）时自动重注册到当前 exe，避免"点了没反应"；指向当前 exe 则零操作，安装版场景不干扰安装器注册
  - **HKCU 免管理员**：写用户级注册表（`HKCU\Software\Classes`），无需 UAC 提权，便携版随处可注册
  - **macOS 明确不支持运行时注册**：协议由 tauri.conf.json 打包写入 Info.plist，前端界面显示"平台不支持"并禁用按钮，避免误导
  - **Windows 双产物 + portable 独立上传**：安装版由 NSIS 注册协议（updater 主格式），便携版靠应用内工具；portable 产物以独立 `bundle_type` 上传，下载页可区分两种形态
- 验证：`cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`npx vue-tsc --noEmit` 通过（exit 0）、改动文件 `npx eslint` 通过（exit 0）

#### 统一请求 User-Agent（Molaunch/{主版本}.{clientType}）

- 背景：需要统一的请求 UA 标识客户端平台与渠道，便于后端识别与灰度分流。设计文档见 `docs/client.md`（两位编码：十位平台/架构、个位渠道类型）
- 改动（3 文件）：
  - **新增 `src-tauri/src/utils/client_type.rs`**：UA 工具函数
    - `platform_code()`：平台/架构 → 十位码（1 Windows x86_64 / 2 x86 / 3 ARM64 / 4 macOS x86_64 / 5 ARM64 / 6 Linux x86_64 / 7 ARM64 / 8 Android / 9 iOS），用 `cfg!` 宏编译期推导
    - `channel_code()`：版本号预发布后缀 → 个位渠道码（无后缀→0 正式 / `-rc`→1 灰度 / `-beta`→2 内测 / `-alpha`/`-dev`→3 开发 / `-nightly`→4 每日 / 未知→3 兜底）
    - `user_agent()`：`Molaunch/{主版本}.{clientType}`（如 Windows x86_64 正式版 → `Molaunch/1.0.0.10`），版本取 `CARGO_PKG_VERSION` 主版本部分
    - 3 个单元测试（渠道映射 / 版本清洗 / UA 格式）
  - **`src-tauri/src/utils/mod.rs`**：声明 `client_type` 模块
  - **`src-tauri/src/http.rs`**：`user_agent()` 改用 `utils::client_type::user_agent`，UA 从 `MoLaunch/{os} {version}` 改为统一格式
- 版本映射说明（前后端对齐）：
  - CI 打 tag（如 `v1.0.0-rc1`）时 Update version 步骤会**同时改写** package.json / Cargo.toml / tauri.conf.json，前端水印（`__APP_VERSION__`）与 Cargo 版本天然一致，无需前端改动
  - 本地开发 `0.1.0-beta.1` → UA `Molaunch/0.1.0.12`（beta→内测2）；CI `1.0.0-rc1` → `Molaunch/1.0.0.11`（rc→灰度1）；正式 `1.0.0` → `Molaunch/1.0.0.10`（正式0）
- 验证：`cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`cargo test client_type` 3 个测试通过

#### 深度链接（molaunch:// 协议）+ 可扩展后缀路由

- 背景：为启动器引入 deep link 能力——注册 `molaunch://` 协议，并设计成可扩展组件：业务模块可注册后缀路由（如 `molaunch://run`），后续按需接入
- 后端改动（5 文件）：
  - **`src-tauri/Cargo.toml`**：添加 `tauri-plugin-deep-link = "2"`（协议注册/事件）、`tauri-plugin-single-instance = { version = "2", features = ["deep-link"] }`（单实例 + 新实例 URL 转发）、`url = "2"`（URL 解析）
  - **`src-tauri/tauri.conf.json`**：`plugins.deep-link.desktop.schemes = ["molaunch"]`（Windows/Linux 打包时 NSIS/安装器自动写注册表；macOS 打包时自动生成 Info.plist 的 `CFBundleURLTypes`）
  - **新增 `src-tauri/src/deeplink/`**（模块，`mod.rs` 仅作入口，业务逻辑拆分到子文件）
    - `mod.rs`：入口——模块声明 + 公共 API re-export（`register`/`register_sync`/`dispatch`/`init`/`DeeplinkRequest`）
    - `request.rs`：`DeeplinkRequest` 结构 + `parse()` URL 解析（scheme / host / path / query，URL 解码）
    - `router.rs`：路由注册表（仿 Dispatcher 注册式模式）+ `dispatch()` 分发 + `init()` 初始化
    - `handlers.rs`：内置路由 `run`（启动游戏，骨架）/ `install`（安装整合包，**强制白名单校验**）/ `open`（前端页面跳转，骨架）
    - `security.rs`：下载域名白名单安全校验（仅 https + 白名单域名 + 拒绝 userinfo 注入），含单元测试
  - **`src-tauri/src/lib.rs`**：注册 single-instance（含聚焦窗口）+ deep-link 插件；setup 钩子调用 `deeplink::init(app.handle())`
- 前端改动（2 文件）：
  - **`package.json`**：添加 `@tauri-apps/plugin-deep-link@2`
  - **新增 `src/utils/deeplink.ts`**：`onDeeplink()`（监听后端事件做 UI 跳转）、`getStartupDeeplink()`（启动唤醒场景）、`parseMolaunchUrl()`（前端侧解析/预览）
- 设计决策：
  - **Windows 单实例 + deep-link feature**：OS 点击 `molaunch://` 会以 URL 作为 CLI 参数启动新进程；single-instance 插件保证只有主实例运行，并把新实例的 URL 转发为 `deep-link://new-url` 事件——插件顺序必须 single-instance 在前
  - **macOS 走系统事件而非 CLI 参数**：macOS 不支持运行时注册协议，scheme 由 `tauri.conf.json` 声明、打包时写入 Info.plist `CFBundleURLTypes`；链接到达通过 `RunEvent::Opened` 直接派发给运行中实例（插件 `.on_event` 已 emit `deep-link://new-url`，冷启动 URL 写入 `get_current`），无需 single-instance 转发，`on_open_url` 订阅统一覆盖
  - **dev 模式动态注册仅 Windows/Linux**：未安装时 Windows 写 `HKCU\Software\Classes`、Linux 写 desktop 文件，`molaunch://` 可在开发环境点击测试；macOS dev 模式不支持运行时注册（插件返回 UnsupportedPlatform），用 `#[cfg(all(debug_assertions, not(target_os = "macos")))]` 排除
  - **install 路由强制域名白名单**（安全红线）：`validate_download_url` 校验仅 https + 白名单域名（media.forgecdn.net / edge.forgecdn.net / mediafilez.forgecdn.net / cdn.modrinth.com / modrinth.com / moiu.cn / mocdn.net，子域名通配）+ 拒绝 userinfo 注入，防恶意网站通过 `molaunch://install?url=病毒` 诱导下载；白名单收录需人工审核
  - **后端 handler 是逻辑入口，前端事件只做 UI**：handler 可访问 AppHandle 调起业务（启动游戏等），前端 `deeplink://new` 事件只负责页面跳转提示
  - **扩展方式**：新路由只需业务模块内 `deeplink::register("xxx", ...)`，无需改核心文件
  - **mod.rs 仅作入口**：业务逻辑按职责拆分（request / router / handlers / security），降低单文件复杂度、便于测试与扩展
- 验证：`cargo check` + `cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`cargo test deeplink` 4 个安全测试通过、`npx vue-tsc --noEmit` 通过（exit 0）、`npx eslint src/utils/deeplink.ts` 通过（exit 0）

#### FRP 厂商接口规范改造（阶段 6：前端隧道同步 + 授权前置检查）

- 背景：阶段 3+5+8 后端已完成 `fetch_tunnels` action（按 endpoints.json 配置从厂商 API 拉取隧道列表 + 账号信息），但前端缺少对应调用入口。本次完成前端全链路：类型定义 → IPC 封装 → 同步组件 → TunnelManager 入口集成，且拉取前强制检查授权状态
- 改动（4 文件）：
  - **`src/types/frp.ts`**：新增 3 个类型
    - `RemoteTunnelInfo`：厂商 API 返回的远程隧道信息（id/name/tunnelType/status/serverHost/serverPort/token/localHost/localPort/remotePort/customDomain），字段为字符串型（部分厂商返回带前导 0 的端口）
    - `RemoteAccountInfo`：厂商 API 返回的账号信息（id/username/email/token）
    - `FetchTunnelsResult`：`{ tunnels: RemoteTunnelInfo[], account: RemoteAccountInfo }`
  - **`src/utils/api/frp-manager.ts`**：`FRP_ACTIONS` 新增 `FETCH_TUNNELS = 'fetch_tunnels'`；新增 `fetchTunnels(providerId)` 便捷封装函数，返回 `FetchTunnelsResult`
  - **`src/components/frp/RemoteTunnelSync.vue`**（新组件，219 行）：从厂商 API 同步隧道的完整交互面板
    - 厂商选择（Select 组件，只列出 `authType !== 'none' && enabled` 的厂商）
    - 选中厂商后自动调用 `getAuthStatus` 检查授权状态
    - 未授权时显示 ShieldExclamationIcon + "厂商未授权" + "请先到认证页面完成授权" 居中提示（icon + text 垂直水平居中）
    - 已授权时显示绿色"已认证"标记 + "拉取隧道"按钮
    - 拉取后展示远程隧道列表（名称/类型/状态/服务器地址/本地端口/远程端口/自定义域名）
    - 每条隧道有"导入"按钮，点击后将 `RemoteTunnelInfo` 映射为 `CreateTunnelParams` emit 给父组件（端口字符串 parseInt 转数字，tunnelType 非 tcp/udp 时回退 tcp）
    - 导入后标记"已导入"（绿色 CheckCircleIcon），避免重复导入
  - **`src/components/frp/TunnelManager.vue`**：顶部操作栏新增"从厂商同步"按钮（CloudArrowDownIcon + Tooltip），点击展开 `RemoteTunnelSync` 面板（Transition 动画与其他面板一致）。`handleRemoteImport` 调用 `store.createTunnel` 创建本地隧道，成功后 toast 提示。合并 heroicons import 为单行 + 精简文件头注释控制行数在 300 行内（289 行）
- 设计决策：
  - **授权前置检查**：用户选择厂商后自动检查授权状态，未授权时不展示拉取按钮，从源头避免未授权调用 `fetch_tunnels` 导致后端报错
  - **导入而非自动创建**：远程隧道拉取后只展示，用户手动点击"导入"才创建本地隧道，避免自动创建大量未确认的隧道配置
  - **字段类型映射**：后端 `TunnelInfo` 字段为 `String`（兼容厂商返回的带前导 0 端口），前端导入时 `parseInt` 转为 `number`（匹配 `CreateTunnelParams` 的 `localPort/serverPort/remotePort: number`）
  - **RemoteTunnelSync 独立组件**：不内联到 TunnelManager（已 289 行），遵循 300 行约束
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### FRP 厂商接口规范改造（阶段 7：教程文档更新）

- 背景：阶段 1-6 完成了后端类型定义、API 引擎、auth 重写、前端隧道同步，但 tutorial-frp.html 教程仍为旧设计（单 manifest.json 架构），缺少 endpoints.json 规范说明。本次按新设计全面重写教程
- 改动（1 文件）：
  - **`src-tauri/resources/templates/tutorial-frp.html`**：从 264 行重写为 578 行，新增内容：
    - **三文件架构**：manifest.json（元信息+指针）+ auth.json（认证交互层）+ api/endpoints.json（API 规范）三层分离说明
    - **auth.json 章节**：认证交互层配置（type=oauth2/device_code/api_key/none），字段说明表（authorizeUrl/clientId/clientSecret/scopes/redirectPort 等）
    - **endpoints.json 章节**（核心新增）：完整 API 规范说明
      - baseUrl 与 auth（token 注入：headerName/headerPrefix/headerKeyName）
      - authFlows（认证流程定义）：oauth2（token+refresh）/ device_code（request+poll）/ remote_login / api_key，含占位符列表和 FieldExtractor（from=body/header）说明
      - envelope（响应包裹解析）：successField/successValue/errorField/dataField
      - config（配置生成模式）：url/fields/args 三种模式
      - endpoints（API 端点定义）：account/tunnels.list/tunnels.config，含 itemsField 嵌套展平和 FieldMapping 三种形式（字符串/对象 split/模板 {account.token}）
    - **完整示例：标准 OAuth2 厂商**：三文件完整配置（manifest.json + auth.json + endpoints.json）
    - **完整示例：OpenFRP 非标准厂商**：关键差异说明（token 在响应头/frpc 启动参数模式/隧道列表嵌套/合并字段拆分/账号 token 引用）
    - **从 URL 安装**：新增安装方式说明
  - manifest.json 字段表新增 `authFile` 和 `api.endpointsFile` 两个字段说明
- 设计决策：教程按"概念说明 → 字段表 → 代码示例"三段式组织，每个概念配表格和代码块；OpenFRP 作为非标准厂商示例单独列出关键差异，帮助开发者理解如何适配非标准接口

#### FRP 厂商配置三项修复（多平台 frpc 映射 + logo 加载 + authType 回退）

- 背景：用户测试 LoliaFrp 厂商配置时发现三个问题：(1) 不同 OS/arch 需要不同 frpc 二进制但 manifest 只支持单路径；(2) manifest 声明了 icon 但前端不加载；(3) auth.json 声明了 type=oauth2 但认证中心显示"无需认证"
- 改动（7 文件）：
  - **`src-tauri/src/commands/frp/types.rs`**：
    - `BinaryConfig` 新增 `paths: Option<HashMap<String, String>>` 字段，key 格式 `{os}_{arch}`（如 `windows_amd64`），优先于 `path` 字段
    - `ProviderInfo` 新增 `icon: Option<String>` 字段（后端填充绝对路径，前端用 convertFileSrc 渲染）
  - **`src-tauri/src/commands/frp/provider.rs`**：
    - 新增 `current_platform_key()` 函数：返回当前平台 key（`{os}_{arch}`，arch 映射 x86_64→amd64 / aarch64→arm64 / x86→386）
    - 新增 `resolve_bundled_path()` 函数：优先从 paths 按当前平台查找，回退到 path
    - 新增 `resolve_auth_type()` 函数：manifest.auth.auth_type 为 "none" 时回退从 auth.json 的 type 读取，避免厂商在 manifest 和 auth.json 中重复声明
    - `get_frpc_path_for_provider` 和 `is_external_frpc_ready` 改用 `resolve_bundled_path` 支持多平台
    - `list_providers` 改用 `resolve_auth_type` + 填充 `icon` 绝对路径
  - **`src-tauri/src/commands/frp/install.rs`**：`build_provider_info` 改用 `resolve_auth_type` + 填充 `icon`
  - **`src-tauri/src/commands/frp/auth/mod.rs`**：`get_auth_status` 改用 `resolve_auth_type`
  - **`src-tauri/tauri.conf.json`**：CSP 的 `img-src` 和 `connect-src` 新增 `https://asset.localhost`；新增 `assetProtocol` 配置（scope 限制为 `$APPDATA/.Molaunch/providers/**`）
  - **`src/types/frp.ts`**：`ProviderInfo` 新增 `icon?: string`
  - **`src/components/frp/ProviderList.vue`**：导入 `convertFileSrc`，厂商图标位置条件渲染（有 icon 时 `<img>` 否则 `ServerStackIcon`）
  - **`src-tauri/resources/templates/tutorial-frp.html`**：bundled 分发方式说明新增 `paths` 多平台映射字段说明和示例
- 设计决策：
  - **paths 优先于 path**：`resolve_bundled_path` 先查 paths 当前平台，找不到回退到 path，兼容旧配置
  - **authType 回退读取**：manifest.auth.auth_type 缺省为 "none"，厂商只需在 auth.json 中声明 type 即可，无需在 manifest 中重复声明
  - **icon 绝对路径 + convertFileSrc**：后端返回 icon 绝对路径，前端用 Tauri 的 convertFileSrc 转为 webview 可访问 URL，通过 assetProtocol 配置限定 scope 到 providers 目录
- 验证：`cargo check` 通过（exit 0）、`npx vue-tsc --noEmit` 通过（exit 0）

### 修复

#### CI：Windows 不再强依赖 setup 签名（便携版为云端唯一 Windows 产物）

- 背景：Windows 更新走自研流程（`check_update` → 下载云端 portable.exe + `.sig` → updater.exe 替换），`tauri-plugin-updater` 在 Windows 不参与；产品方案为 **Windows 只使用便携版**，安装版（setup）仅手动下载。但 release.yml 的 `Locate installer and signature` 步骤对所有平台硬性要求 setup.exe 及其 `.sig`，Windows job 因 `MoLaunch_0.3.0_x64-setup.exe.sig` 缺失报错退出
- 改动（2 文件）：
  - **`.github/workflows/release.yml`**：
    - `Locate installer and signature` 改为平台差异化：macOS/Linux 仍要求安装包 + `.sig`（推云端）；Windows 改为要求便携版 + `.sig`（云端唯一产物），setup 不要求 `.sig`（仅附加 GitHub Release）
    - `Upload setup as workflow artifact` 仅上传 `*-setup.exe`（去掉 `.sig`）
    - `release` job 的 `files` 仅附加 `**/*-setup.exe`（去掉 `.sig`）
  - **`CHANGELOG.md`**：记录本次调整
- 设计决策：Windows setup 的 `.sig` 仅供 tauri 官方 updater 协议使用，便携版方案不需要，故不再作为 CI 硬性要求
- 验证：`js-yaml` 解析 release.yml 通过；locate 平台分支与 setup artifact / release files 内容核对无误

#### CI：macOS/Linux 打包缺 updater 签名（.sig 未生成）+ macOS deeplink dead_code

- 背景：v0.3.0 release CI 中 macOS/Linux job 在 `Locate installer and signature` 步骤报 `签名文件不存在`（macOS: `MoLaunch.app.tar.gz.sig`，Linux: `MoLaunch_0.3.0_amd64.AppImage.sig`）；macOS 另报 `constant PROTOCOL is never used` 警告
- 根因（2 个独立问题）：
  1. **tauri.conf.json 缺 `bundle.createUpdaterArtifacts`**。Tauri v2 官方文档要求该字段显式设为 `true`，打包器才会创建 updater 产物并生成 `.sig` 签名文件。当前缺失（默认不生成），导致三平台均无 `.sig`（Windows setup 同样受影响，仅便携版因 CI 手动 `tauri signer sign` 有签名）
  2. **`src/deeplink/protocol.rs` 的 `PROTOCOL` 常量无 cfg**：仅在 Windows（注册表）/Linux（desktop 文件）的 cfg 函数中使用，macOS 上编译时未使用 → dead_code 警告
- 改动（3 文件）：
  - **`src-tauri/tauri.conf.json`**：`bundle` 增加 `"createUpdaterArtifacts": true`（CLI 构建时对 NSIS/deb/rpm/AppImage/macOS .app 生成 `.sig` 签名）
  - **`src-tauri/src/deeplink/protocol.rs`**：`const PROTOCOL` 加 `#[cfg(any(windows, target_os = "linux"))]`（macOS 协议由 Info.plist 声明，无需运行时注册常量）
  - **`CHANGELOG.md`**：记录本次修复
- 验证：`node -e JSON.parse` 解析 tauri.conf.json 通过（createUpdaterArtifacts=true）；`cargo fmt --check` 与 `cargo check` 通过（exit 0）；PROTOCOL 全部引用点均在 cfg 函数内核对无误

#### CI：Windows 产物分发调整（便携版推云端，setup 仅附加 GitHub Release）

- 背景：上一轮将便携版与 setup 都上传云端，实际需求是 **Windows 便携版才推送云端（S3 + 注册），setup 安装版不推送存储，仅附加到 GitHub Release**（供用户手动下载安装）
- 改动（2 文件）：
  - **`.github/workflows/release.yml`**：
    - `Upload to S3 and register release` 步骤加 `if: matrix.platform != 'windows'`——Windows setup（nsis）不再上传 S3、不再注册版本；macOS/Linux 保持原逻辑推云端（其产物是唯一更新格式）
    - Windows 便携版上传步骤保留（`bundle_type=portable`，云端 Windows 唯一产物）
    - 新增 `Upload setup as workflow artifact (Windows only)`：将 `bundle/nsis/*-setup.exe` 及 `.sig` 上传为 workflow artifact
    - `release` job 新增 `Download Windows setup artifact` 步骤，`softprops/action-gh-release@v1` 增加 `files: **/*-setup.exe` + `.sig`，将 Windows setup 附加到 GitHub Release Assets
    - Release 页面 Downloads 文案更新（区分便携版/安装版/macOS/Linux）
  - **`CHANGELOG.md`**：记录本次调整
- 设计决策：
  - **云端只保留便携版**：客户端「检查更新」协议与便携版/安装版共用，便携版作为 Windows 自动更新主格式；setup 安装版仅面向手动安装用户
  - **artifact 传递**：release job 与 build job 分离，通过 `actions/upload-artifact@v4` / `download-artifact@v4` 跨 job 传递 setup 文件
- 验证：本地 `js-yaml` 解析 release.yml 通过（jobs: build-and-upload, release；release.files 含 `**/*-setup.exe` 与 `.sig`）

#### CI：修复 Windows 构建失败（tauri --bundles 不支持 portable 目标）

- 背景：release.yml Windows job 在 `Build Tauri` 步骤报 `error: invalid value 'portable' for '--bundles [<BUNDLES>...]' [possible values: msi, nsis]`，v0.3.0 发布构建失败。根因：**Tauri v2 的 `--bundles` 参数在 Windows 平台仅支持 `msi / nsis`**，官方不存在 `portable` 打包目标（tauri-bundler `PackageType` 枚举、CLI `--help` 均无此值），`--bundles nsis,portable` 为无效写法
- 改动（2 文件）：
  - **`.github/workflows/release.yml`**：Windows job 的 `args` 由 `--bundles nsis,portable` 改为 `--bundles nsis`（修复报错，产物与 `bundle_type: nsis`、updater 协议一致）；新增 `Create portable build (Windows only)` 步骤——将 `src-tauri/target/release/MoLaunch.exe` 单文件复制为 `bundle/nsis/MoLaunch_{version}_x64_portable.exe`，并用 `npx tauri signer sign` 生成配套 `.sig`，命名沿用 locate 步骤 `*_portable.exe` 约定，预签名上传/版本注册逻辑零改动复用
  - **`CHANGELOG.md`**：记录本次修复
- 设计决策：
  - **便携版 = release 单文件 exe**：Tauri v2 无原生 portable 目标，便携版直接复制构建产物单文件（WebView2 依赖系统运行时，Win10/11 预装），满足"复制即用"诉求
  - **签名复用 updater 密钥**：便携版 `.sig` 与 NSIS 安装包共用 `TAURI_SIGNING_PRIVATE_KEY`，客户端下载校验一致；`ci-upload.cjs` 无需改动
- 验证：本地 `npx tauri build --help` 确认 Windows 平台 `--bundles` 仅 `msi, nsis`；`npx tauri signer sign --help` 确认 `TAURI_SIGNING_PRIVATE_KEY` 环境变量用法与 `<FILE>.sig` 输出

#### CI：补充 reg_delete 非 Windows stub 标记 dead_code（clippy 遗漏项）

- 背景：上一轮修复 27 个 clippy 错误后，`rust-clippy` job 仍报 `reg_delete` never used（`src/storage/registry.rs:76`）。根因：`#[allow(dead_code)]` 只加在 Windows 版本上（该注释原属 Windows 版），`#[cfg(not(windows))]` stub 版本遗漏
- 改动（1 文件）：`src/storage/registry.rs` 的非 Windows `reg_delete` stub 补 `#[allow(dead_code)]`，与其他 3 个 `reg_*` stub 一致
- 验证：`cargo clippy --all-targets -- -D warnings` 通过（exit 0）

#### CI：修复 clippy 27 个错误（registry 死代码 + sort_by 优化）

- 背景：CI `rust-clippy` job 报 27 个错误（`docs/Error/workflow/clippy.txt`），两类：
  1. **dead-code（23 个）**：`minecraft/auth/storage/registry.rs` 的 20 个 `KEY_*` 注册表键名常量、`storage/registry.rs` 的 `REG_SUBKEY` 与 3 个 `reg_*` 非 Windows stub 函数在 Linux 编译时无调用方。根因：这些常量/函数只在 Windows 注册表路径使用，但未加 `#[cfg(windows)]` 或未对 stub 标记 `#[allow(dead_code)]`（Windows 本地编译有调用方，Linux 编译暴露）
  2. **unnecessary-sort-by（4 个）**：`frp/process/log.rs`、`tools/download.rs`、`tools/screenshot.rs` 的降序排序和 `version/mods/list.rs` 的升序排序，clippy 建议 `sort_by_key` + `Reverse`
- 改动（6 文件）：
  - **`src/minecraft/auth/storage/registry.rs`**：20 个 `KEY_*` 常量全部加 `#[cfg(windows)]`（`ALL_KEYS` 原本已有）
  - **`src/minecraft/auth/storage/mod.rs`**：`mod registry;` 加 `#[cfg(windows)]`（与常量 cfg 一致，Linux 不编译空模块）
  - **`src/storage/registry.rs`**：`REG_SUBKEY` 加 `#[cfg(windows)]`；`reg_key`/`reg_get`/`reg_set` 的 `#[cfg(not(windows))]` stub 加 `#[allow(dead_code)]`（与既有 `reg_delete` stub 一致）
  - **`src/commands/frp/process/log.rs`** / **`src/commands/tools/download.rs`** / **`src/commands/tools/screenshot.rs`**：`sort_by(|a,b| b.x.cmp(&a.x))` → `sort_by_key(|b| Reverse(b.x))`
  - **`src/commands/version/mods/list.rs`**：`sort_by(|a,b| a.fn.cmp(&b.fn))` → `sort_by_key(|a| a.fn)`
- 验证：`cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`cargo test --all-features` 通过（exit 0，1 passed / 2 ignored）
- 说明：dead-code 为 Linux 平台专属（Windows 有调用方），已按 cfg 语义修复；本地无法交叉编译 Linux（缺 x86_64-linux-gnu-gcc），通过逐项核对 clippy.txt 报错点 + Windows 编译验证

#### CI：修复最终两个编译错误（dist 目录缺失 + auth/storage log_warn 导入）

- 背景：上轮修复后 CI `rust-clippy` / `rust-test` 仍失败（`docs/Error/workflow/clippy.txt`、`test.txt`），剩 2 个错误：
  1. **`frontendDist ../dist 不存在`**（lib.rs:205 `tauri::generate_context!`）：`cargo clippy/test --all-features` 启用 `tauri/custom-protocol` feature 后，tauri-build 校验 `frontendDist` 指向的 `../dist` 目录必须存在。本地开发有 `dist/` 所以不报，CI 全新 checkout 没有
  2. **`auth/storage/mod.rs:125` 找不到 `log_warn` 宏**：`restrict_file_permissions`（`#[cfg(unix)]`）用 `log_warn!` 但该模块未导入（上一轮遗漏）
- 改动（2 文件）：
  - **`src-tauri/src/minecraft/auth/storage/mod.rs`**：补 `#[cfg(unix)] use crate::log_warn;`（Unix 编译需要导入；`#[cfg(unix)]` 保证 Windows 编译不产生 unused import）
  - **`.github/workflows/ci.yml`**：`rust-clippy` / `rust-test` 两个 job 在 cargo 命令前加 `- name: Ensure frontend dist directory` 步骤 `mkdir -p dist`（空目录即满足 tauri-build 存在性校验）
- 验证：`cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`cargo test --all-features` 通过（exit 0）
- 说明：本地已确认 `dist/` 存在是本地编译通过、CI 失败的环境差异根因；`mkdir -p dist` 空目录方案已覆盖 CI 场景

#### CI：修复 Linux 平台编译/检测错误（补 SDK 资源 + 6 个代码修复）

- 背景：CI 的 `rust-clippy` / `rust-test` job 在 Ubuntu 上构建失败，`docs/Error/workflow/clippy.txt` 报告 14 个错误，分两类：**资源缺失**（`resources.rs` 用 `include_bytes!` 编译期嵌入 SDK 动态库，但仓库缺 Linux/macOS 版本）和 **代码错误**（Linux 专属路径 + cfg 条件导致的 lint/编译问题，Windows 本地编译看不出来）
- 资源修复（4 文件，`src-tauri/resources/sdk/`）：
  - **新增** `run_sdk_lib-linux-x86_64.so`、`run_sdk_lib-darwin-aarch64.dylib`（macOS 版按 [resources.rs](file:///c:/Users/XiaoMo/Desktop/MoLaunch/src-tauri/src/resources.rs#L93-L96) 引用名 `darwin-aarch64` 重命名，Intel Mac `macos-x86_64.dylib` 未被引用删除避免误导）
  - **更新** `run_sdk_lib-windows-x86_64.dll`；至此 3 个被 `include_bytes!` 引用的平台文件全部就位
- 代码修复（6 文件）：
  - **`src/commands/system/updater/mod.rs`**：补 `use crate::log_info;` 并加 `#[cfg(not(target_os = "windows"))]`（`log_info!` 只在非 Windows 分支使用，Linux 需要导入、Windows 无条件导入会 unused import——平台差异双解决）
  - **`src/commands/system/updater/install_unix.rs`**：`download_and_install(|_| {})` → `download_and_install(|_, _| {}, || {})`，适配 tauri-plugin-updater v2.10.1 的 2 参数签名（进度闭包 2 参数 + 完成闭包）
  - **`src/minecraft/system/shell/exec.rs`**：`kill_process_tree` 重构为两个自包含 cfg 块（Windows 块内含 taskkill 错误检查；Unix 块内完整处理 ps+kill 错误），消除不可达代码（E0593 前 return 后通用检查）与类型推断失败（E0282，原跨 cfg 的 `let output` 类型无法统一）
  - **`src/minecraft/online/tun.rs`**：`use crate::{log_debug,...}` 拆分（`log_debug!` 只在 Windows 块用，下沉进 `#[cfg(windows)]` 块）；`let mut builder` 改 shadow 绑定 + if-else 表达式消除 unused mut（Windows 需重绑定、Linux 不需要）
  - **`src/minecraft/system/shell/admin.rs`**：`use crate::{log_error, log_info}` 拆分（`log_error!` 只在 Windows `relaunch_as_admin` 用，下沉进 Windows 块）
  - **`src/commands/frp/process/start.rs`**：删除顶层 `use crate::log_warn;`（移入 Windows Job Object 块内）；删除 `#[cfg(unix)]` 块内 `use std::os::unix::process::CommandExt;`（tokio `pre_exec` 固有方法，无需 trait import）
- 验证：`cargo check --lib` 通过（exit 0）、`cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`cargo test --all-features` 通过（exit 0，1 passed / 2 ignored）
- 说明：资源修复已本地验证 include_bytes 全部匹配；代码修复按 cfg 语义逐平台推演，Windows 功能不受影响

#### 升级 netstat2 0.9.1 → 0.11.2：修复 Linux 新版 libc 编译失败（CI rust-clippy / rust-test）

- 背景：CI 的 `rust-clippy` / `rust-test` job 构建失败，`netstat2 v0.9.1` 编译时报 `tcp_info` 结构体字段错误。根因：netstat2 0.9.1 发布于 2020 年，Linux 实现直接访问 libc `tcp_info.state` 字段；新版 libc（0.3.x）将字段重命名为 `tcpi_state` 且类型改为 `__be16`，旧 crate 未固定 libc 版本上限导致解析到新版 libc 后编译失败（本项目因 Frp 端口占用检测 [ports.rs](file:///c:/Users/XiaoMo/Desktop/MoLaunch/src-tauri/src/commands/tools/network/ports.rs) 直接依赖 netstat2）
- 改动（2 文件）：
  - **`src-tauri/Cargo.toml`**：`netstat2 = "0.9"` → `"0.11"`（0.11.2 为 2025-08 最新版，已修复 libc 兼容）
  - **`src-tauri/Cargo.lock`**：netstat2 0.9.1 → 0.11.2（新增 bindgen/netlink-packet-* 等 Linux 构建依赖）
- API 兼容性：`get_sockets_info` / `AddressFamilyFlags` / `ProtocolFlags` / `SocketInfo` / `TcpState` 等稳定 API 在 0.11 未破坏，ports.rs 代码零改动
- 验证：`cargo check --lib` 通过（exit 0）、`cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`cargo test --all-features` 通过（exit 0）

#### CI：新增 workflow_dispatch 手动触发入口

- 背景：CI 的 `paths` 过滤不包含 `.github/workflows/**`，仅修改 workflow 文件推送不会自动触发 CI（无法验证上一条 webkit2gtk 4.1 依赖修复）
- 改动（1 文件 `.github/workflows/ci.yml`）：`on:` 增加 `workflow_dispatch:`——可在 GitHub Actions 页面「Run workflow」按钮手动触发，用于 workflow 自身改动后的验证
- 说明：手动触发时 `github.event.head_commit.message` 为空，`check-skip` 的 `!c` 匹配失败走 `skip=false`，CI 正常执行全部 job

#### CI：WebKitGTK 依赖版本升级（4.0 → 4.1）修复 javascriptcoregtk 缺失

- 背景：CI 的 `rust-clippy` / `rust-test` job 构建失败，错误为 `javascriptcore-rs-sys` 找不到 `javascriptcoregtk-4.1.pc`。项目是 **Tauri v2**，webview 依赖 webkit2gtk **4.1**（`javascriptcoregtk-4.1`），但 workflow 沿用了 Tauri v1 时代的 `libwebkit2gtk-4.0-dev`（4.0 版），导致 pkg-config 找不到对应库
- 改动（2 文件 3 处）：
  - **`.github/workflows/ci.yml`**（2 处）：`rust-clippy` job、`rust-test` job 的 Linux 依赖安装命令 `libwebkit2gtk-4.0-dev` → `libwebkit2gtk-4.1-dev`（保留 `libsoup-3.0-dev` / `libayatana-appindicator3-dev` / `librsvg2-dev` 等 Tauri v2 依赖组合）
  - **`.github/workflows/release.yml`**（1 处）：`build-and-upload` job 的 Linux 依赖同步升级为 4.1（打 tag 打包时同样需要 4.1）
- 验证：grep 确认 3 处已全部替换，无 4.0 残留；Ubuntu 22.04 (jammy) 官方源提供 `libwebkit2gtk-4.1-dev`，与 Tauri v2 官方 GitHub workflow 示例一致

#### CI：改为纯检查流水线（移除 Tauri 打包 job），补充前端 typecheck 与 Rust 测试

- 背景：用户要求 CI 只做检查、不要 build 程序。原 `build` job 用 tauri-action 做三平台（Windows/macOS/Linux）完整打包，耗时最长且每次提交都跑；且前端类型检查（vue-tsc）此前只在 build job 中隐含执行，Rust 单测（`*_tests.rs`）从未在 CI 中运行过
- 改动（1 文件 `.github/workflows/ci.yml`）：
  - **删除 `build` job**：移除 tauri-action 三平台打包矩阵（不再产构建产物，CI 只做静态/单元检查）
  - **`frontend-check` 增加 `Type check` 步骤**：`npm run typecheck`（vue-tsc --noEmit），弥补删除 build 后缺失的前端类型检查
  - **新增 `rust-test` job**：`cargo test --all-features`（ubuntu-22.04，复用与 clippy 相同的 Tauri v2 Linux 依赖安装 + rust-cache），让后端 `dependency_resolver_tests` / `log_redact_tests` / `markdown_table_tests` / `state_tests` / `ini_tests` 等单测进入 CI 防线
  - 保留：check-skip（!c 跳过）/ frontend-check（lint+typecheck）/ rust-fmt / rust-clippy
- 效果：CI 从「3 平台打包 + 3 项检查」收敛为「5 项纯检查」，无打包耗时；类型检查与单测纳入 CI 兜底
- 验证：`npm run typecheck` 通过（exit 0）、`cargo test --all-features` 通过（exit 0，1 passed / 2 ignored）；`--all-features` 仅启用 `custom-protocol`（tauri 配置项），Linux 环境无平台风险
- 说明：Tauri 实际打包仍由 `release.yml` 承担（打 tag 触发），CI 与发布职责分离

#### CI：Rust 格式检查失败（cargo fmt）——32 个 .rs 文件格式对齐

- 背景：GitHub Actions `rust-fmt` job 执行 `cargo fmt --all -- --check` 失败，输出在 `docs/Error/workflow/ci-fmt.txt`——大量长行未拆分、import 排序、multiline 压缩等与 rustfmt 1.8.0-stable 不一致（多为 frp/deeplink/migrations 模块新代码）
- 改动：本地 `cargo fmt --all` 格式化 **32 个 .rs 文件**（src-tauri/src/commands/frp/** 17 个、deeplink/** 4 个、migrations/** 2 个、minecraft/** 5 个、其他 4 个），**纯格式改动、零逻辑变更**
- 验证：`cargo fmt --all -- --check` 通过（exit 0）；`cargo check` / `cargo clippy --all-targets -- -D warnings` 通过（exit 0）

#### CI：前端 lint 失败（eslint）——忽略第三方压缩库 + 修复 5 个真实错误

- 背景：GitHub Actions `frontend-check` job 执行 `npm run lint` 失败，输出在 `docs/Error/workflow/web.txt`：204 problems（195 errors, 9 warnings），其中 157 个来自第三方压缩库被误 lint、29 个来自 Node 脚本 `ci-upload.cjs` 未声明 node 环境、5 个真实代码错误
- 改动（7 文件）：
  - **`.eslintrc.cjs`**：新增 `ignorePatterns`——`src-tauri/resources/view/*.min.js`（marked.min.js / qrcode.min.js，第三方生成文件不应 lint）、`src-tauri/resources/wasm/*.js`（cubiomes.js）
  - **`scripts/ci-upload.cjs`**：文件头加 `/* eslint-env node */`，`require`/`process`/`Buffer` 识别为 Node 全局（保留 lint 覆盖）
  - **`src/components/online/KickConfirmDialog.vue` / `LobbyJoinConfirmDialog.vue`**：修复 `vue/require-toggle-inside-transition`——transition 根元素补 `v-if="visible"`（新增 visible ref + onMounted 置 true，行为仅新增进入淡入动画，关闭逻辑不变）
  - **`src/composables/useDebouncedSave.ts`**：修复 `no-inner-declarations`——分支块内 5 个 `function` 声明改为 `const` 箭头函数（作用域与调用顺序不变）
  - **`src/utils/version.ts`**：修复 `no-useless-escape`——正则 `[.\-]?` → `[.-]?`
- 验证：`npx eslint . --ext .vue,.js,.jsx,.cjs,.mjs,.ts,.tsx,.cts,.mts --ignore-path .gitignore` 通过（exit 0，0 errors / 9 warnings，warnings 为既有 Input/ToggleRow 缺默认 prop 等，不影响 CI）；`npx vue-tsc --noEmit` 通过（exit 0）

#### http.rs 精简：fetch 函数收敛为"2 原语 + 2 薄包装"，删除废弃 fetch_bytes

- 背景：用户反馈 `http.rs` 的请求函数过多过杂（5 个 fetch 函数），很多重复，"一个就行了"
- 改动（1 文件）：
  - **`src-tauri/src/http.rs`**：
    - **核心原语保留 2 个**：`get_text_with_status`（GET + 状态码，鉴权需区分 204/403）、`post_json_with_status`（POST JSON + 状态码）
    - **薄包装 2 个**：`fetch_url` 改为 `get_text_with_status` 的薄包装（非 2xx 报错）；`fetch_url_to_file` 改为 `fetch_url` 的薄包装（多一步写盘）
    - **删除废弃的 `fetch_bytes`**（全项目 0 处调用，只剩注释提及）
  - 效果：请求函数从 5 个减为 4 个，重复请求逻辑消除（`fetch_url` 从 10 行缩到 5 行），每个函数职责单一清晰
- 设计决策：
  - **2 个原语是真正独立的**：GET/POST + 保留状态码是鉴权流程（yggdrasil 204/403）的硬需求，不可合并
  - **薄包装消除重复**：多数调用方只需"成功拿文本 / 失败报错"，薄包装让原语聚焦状态码语义，包装聚焦易用性
- 验证：`cargo check` + `cargo clippy --all-targets -- -D warnings` 通过（exit 0）；调用方（download/util、authlib、loaders 等 7 文件 14 处）无需改动

#### IP 信任系统：auto 模式按目标域名实时判断 v4/v6，移除固定测 Cloudflare

- 背景：用户反馈系统"IP 信任"的自动检测固定测 Cloudflare 的 IP（1.1.1.1 / 2606:4700:4700::1111），但**Cloudflare 连通 ≠ 目标域名连通**——目标域名可能是单栈（只有 A 或只有 AAAA）、或 Cloudflare 可达但该域名实际不可达，固定测一次就全局定死地址族会出错。需求：按**要请求的域名**实时判断 v4/v6 状态并自动选择
- 根因：[`src-tauri/src/http.rs`] 的 `resolve_local_address("auto")` 调用 `auto_detect_ip_version()`，启动时对 Cloudflare 两个地址族做 TCP 测速，结果绑定到整个客户端生命周期
- 改动（1 文件）：
  - **`src-tauri/src/http.rs`**：
    - `resolve_local_address("auto")` 改为返回 `None`（不设置 local_address），由 reqwest/hyper 底层 **Happy Eyeballs** 机制对**目标域名**实时解析 A/AAAA 记录、并发尝试连接、自动选择先连通的一方（单栈域名自然落到对应地址族，双栈并发择优）
    - 删除 `auto_detect_ip_version` / `detect_faster_stack` / `test_tcp_connect`（固定测 Cloudflare 的旧逻辑）
    - 清理不再使用的 imports（`ToSocketAddrs` / `Instant` / `Ipv6Addr`）
- 设计决策：
  - **Happy Eyeballs 就是"按目标域名实时判断"**：连接时对实际要请求的域名做 DNS 解析，v4/v6 并发尝试，先成功者胜出——比"启动时测一次 Cloudflare"更准确、更实时，且天然处理单栈域名
  - **保留 v4 模式**：用户显式选"IPv4 优先"仍走 `local_address(0.0.0.0)`；"any" 模式本就跟随 DNS 不绑定
  - **reqwest 限制**：`local_address` 仅支持客户端级（`ClientBuilder`），无请求级 API，故 auto 采用"不绑定 + Happy Eyeballs"而非请求级切换
- 验证：`cargo check` + `cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`cargo test deeplink` 4 个测试通过

#### 种子地图：修复 require 报错 + 输入框标签被挤换行

- 背景：用户反馈工具页种子地图 ①选择版本/维度时点击报 `ReferenceError: require is not defined` + Vue warn（Unhandled error during component update）；②"种子"标签与输入框被挤到换行，布局空间不够
- 根因：
  1. **`useSeedMap.ts` 的 watch 里用了 `require('@/utils/seedmap/structures')`**：项目是 Vite（浏览器 ESM）环境，没有 Node 的 `require`，运行到版本/维度变化时直接抛错，导致组件更新异常（Vue warn 只是表象）
  2. **`Input.vue` 的 `.input-root` 是 `display:inline-block; width:100%`**：在 flex 布局的控制栏里，根元素 100% 宽会独占整行，把"种子"标签挤到换行；`width="200px"` 只作用于内层 `.input-wrapper`，外层根元素仍撑满
- 改动（3 文件）：
  - **`src/views/tools/data/useSeedMap.ts`**：第 474 行 `require(...)` 改为顶部静态 `import { getStructuresForVersion }`，并去掉 filter/map 回调里的显式对象类型标注（`StructureTypeConfig.queryMode` 是 `string | undefined`，原标注 `{ queryMode: string }` 不匹配）
  - **`src/components/common/Input.vue`**：`width` prop 同时应用到根元素（`:style="{ width }"`），传入固定宽时根元素不再 100% 撑满；不传 width 时行为不变（仍 100%）
  - **`src/views/tools/data/SeedMap.vue`**：无需改动（控制栏本身布局正确，问题在 Input 组件）
- 设计决策：
  - **ESM 静态导入替代运行时 require**：`getStructuresForVersion` 是静态导出函数，直接 import 即可，消除运行时 ReferenceError
  - **width 双定位**：`.input-root` 收固定宽 + `.input-wrapper` 保持 100%（相对根元素），保证固定宽输入框在 flex 布局中不撑破容器，其他传 width 的场景（ColorPalette 等）同步受益
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）、改动文件 `npx eslint` 通过（0 errors）

#### 联机页刷新后侧边栏分类恢复失效（tunnels 等 FRP tab 丢失）

- 背景：用户反馈联机页选择侧边栏「tunnels」等分类后刷新页面，侧边栏回到「创建房间」，URL `?tab=` 参数无法控制恢复
- 根因（两层时序/逻辑问题）：
  1. **`useOnlineNav` 的 `isReady` watch 恢复逻辑不完整**：只恢复了 `create`/`join`/`room_details` 三个 tab，`tunnels`/`providers`/`auth`/`logs`/`lobby`/`device` 等落入 else 分支，而 `activeCategory` 初始为 `device` → 默认跳到「创建房间」
  2. **NavSidebar 的 onMounted 恢复时机过早**：`useTabPersistence` 在挂载时读 `route.query.tab` 并校验 `isValid`，但此时 `refreshStatus` 异步未完成，`categories` 只含 `[device]`（isReady=false），`isValid('tunnels')` 失败 → 不恢复
- 改动（1 文件）：
  - **`src/composables/useOnlineNav.ts`**：
    - 新增 `VALID_TABS` 集合 + `isValidCategory()`：涵盖全部可恢复分类（device / lobby / create / join / room_details / providers / tunnels / auth / logs）
    - `isReady` watch 改为权威恢复点：isReady 变 true 时（categories 已就绪）从 URL 完整恢复任意合法 tab；`room_details` 未在房间时回退 `create`
    - watch 加 `{ immediate: true }`：isReady 初始即 true（本地有缓存状态）时也能立即恢复，不依赖后续变化
- 设计决策：
  - **权威恢复点收敛到 useOnlineNav**：NavSidebar 的 onMounted 恢复受 categories 就绪时序制约不可靠，统一由 `useOnlineNav` 在 isReady=true 且 categories 完整时恢复
  - **合法性白名单**：`tutorial` 是动作项（跳设置页）不写入 URL，故排除；`room_details` 特殊处理（未在房间回退）
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）、`npx eslint` 通过（exit 0）

#### 设备码登录：复制失败则放弃自动打开网页 + 失败文案联动

- 背景：用户反馈 ①若自动复制设备码失败，不应再自动打开授权网页（用户还没拿到码就跳浏览器没意义）；②复制失败时弹窗文案应与 toast 提示一致（目前文案仍写"设备码已复制到剪贴板"）
- 改动（1 文件）：
  - **`src/components/common/DeviceCodeModal.vue`**：
    - `watch(deviceCodeInfo)` 改为 async：先 `await copyToClipboard(user_code)`，**复制失败（返回 false）→ 置 `copyFailed=true` + toast 提示"复制设备码失败，请手动复制"，并 `return` 跳过 2s 自动打开网页**
    - 新增响应式 `copyFailed` 状态：驱动弹窗文案
      - 复制成功："授权网页已打开（未打开可点下方按钮），输入以下代码：" + "设备码已复制到剪贴板，可直接粘贴"
      - 复制失败："请点击下方按钮打开 Microsoft 登录页，并手动复制输入以下代码：" + "复制失败，请手动长按/右键复制设备码"
    - `copyCode`（手动"重新复制"按钮）改为 async 并反馈：成功 toast "设备码已复制到剪贴板" + 恢复 `copyFailed=false` 文案；失败 toast "复制失败，请手动复制"
    - 弹窗打开时重置 `copyFailed=false`
- 设计决策：
  - **自动打开依赖复制成功**：复制成功 → 2s 后自动打开网页（用户粘贴即可）；复制失败 → 不自动打开，引导用户手动复制/手动打开，避免跳了个空网页
  - **文案单一数据源**：`copyFailed` 同时驱动 toast 和弹窗内文案，杜绝"toast 说失败、文案说已复制"的矛盾
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）、改动文件 `npx eslint` 通过（exit 0）

#### 设备码登录：去除步骤名展示 + 设备码自动复制文案修正

- 背景：用户反馈 ①既有进度条就不该再显示"获取 XBL Token"等阶段名，两处信息冗余；②设备码弹窗说明文案仍写"请手动点击按钮复制代码并打开网页"，但实际已自动复制到剪贴板，文案过时
- 改动（1 文件）：
  - **`src/components/common/DeviceCodeModal.vue`**：
    - `exchanging`（Token 交换中）阶段：移除 `msLoginStepLabel` 阶段名展示和步骤列表，改为简短文案"正在登录，请稍候..." + 加载图标，下方仅保留进度条（`<StepProgressBar :show-steps="false" />`）
    - 设备码区域文案：由"点击下方按钮打开 Microsoft 登录页，并输入以下代码：请手动点击按钮复制代码并打开网页"改为"授权网页已打开（未打开可点下方按钮），输入以下代码：设备码已复制到剪贴板，可直接粘贴"
    - 按钮文案："点击打开 Microsoft 登录页" → "打开 Microsoft 登录页"，"复制设备码" → "重新复制"（自动复制失败时的兜底）
- 设计决策：
  - **进度条替代阶段名**：Token 交换过程用户只关心"还有多久"，进度条 + 简短文案足够；`stepIndex` 仍驱动进度条推进，`msLoginStepLabel` 保留在 store（不显示但可复用）
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）、改动文件 `npx eslint` 通过（exit 0）

#### 设备码登录自动复制设备码 + 自动打开授权网页

- 背景：用户反馈设备码登录流程需要手动点击"复制设备码"再手动点"打开登录页"，体验繁琐。希望弹窗出现后自动复制设备码，并自动打开授权网页（弹窗出来后延迟 2s 触发）
- 改动（1 文件）：
  - **`src/components/common/DeviceCodeModal.vue`**：
    - `watch(deviceCodeInfo)` 监听到设备码后：自动 `copyToClipboard(user_code)`（toast 提示"已复制到剪贴板"），并启动 2s 定时器自动调用 `openLoginUrl()`（带 verification_uri 白名单校验，防钓鱼跳转）
    - 新增 `autoOpened` 标志 + `autoOpenTimer`：防止自动打开与用户手动点击"打开登录页"重复；关闭弹窗/组件卸载时清理未触发的定时器
    - `openLoginUrl` / `copyCode` / 自动复制调用处补充 `void` 前缀，避免 floating promise
- 设计决策：
  - **延迟 2s 自动打开**：给用户先看到设备码和弹窗内容的时间，也避免弹窗刚渲染就弹浏览器
  - **白名单校验保留**：自动打开同样走 `ALLOWED_URIS` 白名单，不因自动化放宽钓鱼防护
  - **仅复制官方 user_code**：避免剪贴板嗅探，不复制 verification_uri 等其他内容
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）、改动文件 `npx eslint` 通过（exit 0）

#### 微软登录弹窗：进度条仅限 Token 交换阶段 + 弹窗改回居中样式

- 背景：用户反馈上一版把伪进度条加到了"等待授权中"状态（用户在浏览器输入设备码期间进度条就在走，不合理）；且该弹窗误用了 main.css 的 `modal-shell/modal-body` 顶部对齐 + `max-height` 限高方案（那是给长列表/多步骤弹窗用的，如 UpdateDialog/ModUpdateDialog），微软登录弹窗内容少，应改回居中自适应
- 改动（2 文件）：
  - **`src/components/common/DeviceCodeModal.vue`**：
    - 进度条（`StepProgressBar`）**只保留在 `exchanging`（Token 交换中）阶段**——正是获取 XBL/XSTS/MC Token 等需要等待的步骤，用伪进度让用户心里好受点
    - 移除 `requesting`（准备登录）、`waiting`（等待授权 Web/DeviceCode）阶段的进度条，恢复为原来的加载图标 + 提示文字
    - 弹窗从 `modal-shell/modal-body/modal-scroll`（顶部对齐 + `calc(100vh-100px)` 限高）改回自包含居中样式：`fixed inset-0 flex items-center justify-center` + `relative w-full max-w-md bg-white rounded-2xl shadow-xl`，不再依赖 main.css 的全局弹窗方案
  - **`src/components/common/StepProgressBar.vue`**：新增 `isDone` 判定（`currentIndex >= steps.length - 1` 视为完成），完成态真实进度直接置 100%，修复"5 步流程最多走到 80%，伪进度封顶 95% 永远到不了 100%"的 bug
- 设计决策：
  - **伪进度只服务"等待中的处理"**：等待用户授权是用户操作环节，不走进度；只有后端在跑 Token 转换链（等待网络/服务器响应）时才用伪进度缓解焦虑
  - **弹窗样式自包含**：微软登录弹窗内容简短，不套用全局限高方案，改回居中 + 自适应高度（与 Modal.vue 一致）
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）、改动文件 `npx eslint` 通过（exit 0）

#### 修复内存分配轮询切换页面未停止（usePolling 卸载后启动泄漏）

- 背景：用户反馈设置页面内存分配有时从该页切到其他页面后，内存 IPC 轮询仍持续（devtools 不美观）。根因：`MemoryAllocation` / `MemorySection` 在异步 `onLoad`（`await getSystemMemory`）之后才 `startMemoryPolling()`；若在异步等待期间切走页面，组件已卸载（`usePolling` 的 `onUnmounted` 已执行过），之后 `start()` 才执行，注册的 interval 永远不会被清理 → 泄漏
- 改动（1 文件）：
  - **`src/composables/usePolling.ts`**：新增 `unmounted` 标志，`onUnmounted` 时置 true 并 stop；`start()` 若已卸载直接 return，杜绝"卸载后启动的 interval 永不清理"
- 设计决策：
  - **不依赖调用方顺序**：修复放在 `usePolling` 内部，对 MemoryAllocation / MemorySection 等所有使用方统一生效，即使异步 onLoad 晚于卸载也能安全忽略
  - **保留正常卸载清理**：已挂载期间切换页面，onUnmounted 仍正常 stop
- 验证：改动文件 `npx eslint` 通过（exit 0）、`npx vue-tsc --noEmit` 通过（exit 0）

#### 自定义游戏窗口标题留空时不再改写为空白

- 背景：用户反馈「版本设置 → 自定义游戏窗口标题」留空时，启动游戏后窗口标题被改成了空白。根因：前端留空时保存的是空字符串，启动配置 `window_title = Some("")` 传入 `GameWatcher`，`start_monitoring` 中 `if let Some(ref title)` 匹配空字符串成功，`apply_window_title(pid, "")` 把窗口标题清空
- 改动（1 文件）：
  - **`src-tauri/src/minecraft/launch/watcher/mod.rs`**：`start_monitoring` 中启动窗口标题改写前加 `!title.trim().is_empty()` 判断，空值/纯空白不改写（跟随全局设置）；`GameWatcher::new` doc 注释同步补充"空值不改写"
- 验证：`cargo check` 通过（exit 0）

#### 主页启动进度从轮询改为 Tauri event 推送

- 背景：主页开始游戏后右侧内容区进度条之前靠前端 `setInterval` 每 200ms 调 `get_launch_progress` 轮询，浪费 IPC 且实时性受限。改为后端在启动流水线每步更新进度时直接 emit 事件
- 改动（2 文件）：
  - **`src-tauri/src/minecraft/launch/pipeline/execute.rs`**：`update_progress` 末尾通过 `self.config.app_handle` emit `"launch-progress"` 事件，payload 为 `LaunchProgress` 快照（stage / stage_progress / overall_progress / message）；`use tauri::Emitter` 导入
  - **`src/composables/useLaunchState.ts`**：删除 200ms 轮询（`launchProgressTimer` + `setInterval` + `getLaunchProgress` 调用），改为 `listen('launch-progress')` 事件监听；新增 `LAUNCH_PROGRESS_EVENT` 常量与后端事件名对应；`startProgressListener` / `stopProgressListener` 用 unlisten 函数管理，启动前 await 注册确保不丢早期进度事件
- 设计决策：
  - **复用 LaunchConfig.app_handle**：该字段原本用于 Java 自动下载推送进度，现在流水线进度也用它，无需新增状态
  - **保留 get_launch_progress 命令**：后端命令与前端 API 保留，供其他场景（如恢复状态）使用
- 验证：`cargo check` + `cargo clippy --all-targets -- -D warnings` 通过（exit 0）、`npx vue-tsc --noEmit` 通过（exit 0）

#### 联机页取消前端本地 JWT 过期拦截（交给后端自动续期兜底）

- 背景：用户反馈 JWT 过期时联机页面被"拦截"——侧边栏只剩「设备」分类（房间管理/联机大厅隐藏）并强制切回设备页，设备面板显示"登录已过期"登录卡片。但这是**前端基于本地 `token_expired` 的判断**，实际后端有完整的静默续期机制：所有业务 action（信令/房间/FRP/更新器）统一走 `load_creds_with_auto_refresh` 在调用前自动 refresh 续期，前端 `onlineManager` 还有 1003 → refresh → login → register 降级链兜底。前端本地判断过期就拦截页面属于误伤——过期瞬间自动续期即可恢复
- 改动（3 文件）：
  - **`src/composables/useOnlineNav.ts`**：`isReady` 由「已注册 + 已登录 + 未过期」放宽为「已注册 + 已登录」，不再把 `token_expired` 当作拦截条件（房间管理/大厅/FRP 分类保持可用）；watch 注释同步更新
  - **`src/components/online/OnlineDevicePanel.vue`**：`needLogin` 由「已注册 &&（未登录 || 过期）」收紧为「已注册 && 未登录」，JWT 过期不再弹出登录卡片；设备信息卡片中"JWT 过期时间"红色高亮提示保留（仅提示不拦截）
  - **`src/views/Online.vue`**：顶部状态联动注释同步更新
- 设计决策：
  - **信任后端续期链**：后端 `load_creds_with_auto_refresh`（access 过期 → refresh 续期，refresh 也过期才报错）+ 前端 1003 降级链（refresh → login → register）已覆盖所有业务 action，前端无需重复判断 token 过期
  - **兜底路径**：若静默续期全部失败（业务请求报 1003 且重试链也失败），由各调用方 toast 报错提示用户重新登录，而非整页拦截
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### Windows 打开 URL/路径改用 ShellExecuteW（修复 OAuth2 授权链接参数丢失）

- 背景：测试 LoliaFrp OAuth2 授权跳转时，浏览器收到的授权链接只有 `client_id`，日志中 URL 完整但多出 `'redirect_uri' 'response_type' 'scope' 'state'`。根因：`open_url` 用 `cmd /c start "" <url>` 打开链接，cmd.exe 把 URL 查询参数中的 `&` 当作命令分隔符，导致后续参数被拆分丢弃并作为命令执行
- 改动（1 文件）：
  - **`src-tauri/src/minecraft/system/shell/open.rs`**：新增 `win_shell` 模块（`#[cfg(target_os = "windows")]` 封装 `ShellExecuteW` + `to_wide_null` 辅助），`open_url`/`open_path` 的 Windows 分支从 `cmd /c start` 改为 `ShellExecuteW` 直接交给系统默认程序，URL/路径原样以 UTF-16 传递，彻底绕过 cmd.exe 命令解析；`reveal_in_file_manager` 复用 `win_shell` 模块删除局部重复声明
- 设计决策：
  - **ShellExecuteW 绕过 cmd.exe**：ShellExecuteW 把目标字符串原样交给 shell，不经过 cmd.exe 的 `&`/引号解析，URL 中的查询参数不会再被截断
  - **open_path 一并替换**：文件路径同样可能含 `&` 等特殊字符，统一改用 ShellExecuteW 消除同类隐患，与 open_url 行为一致
  - **消除重复声明**：`reveal_in_file_manager` 原有局部 `ShellExecuteW` FFI 声明和 `to_wide_null` 与 `win_shell` 模块重复，重构为复用（符合"可复用函数提取到公共模块"约定）
  - **`log_error`/`shell_err` import 加 `#[cfg]`**：Windows 分支不再使用 `shell_err`（macos/linux 分支仍用），按平台条件导入避免 unused import
- 验证：`cargo check --lib` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib`（141 通过 0 失败）均通过（exit 0）

#### 认证流程引擎支持 {baseUrl} 占位符（修复 token 交换 relative URL 报错）

- 背景：用户测试 LoliaFrp OAuth2 授权，授权链接与本地回调均正常，但 token 交换报「认证流程请求失败: builder error: relative URL without a base」。根因：endpoints.json 的 `authFlows.oauth2.token.url` 使用 `{baseUrl}` 占位符（如 `{baseUrl}/oauth2/token`），但 flows.rs 的 `fill_template` 占位符列表不含 `baseUrl`，URL 保持相对路径传给 reqwest 导致解析失败
- 改动（4 文件）：
  - **`src-tauri/src/commands/frp/auth/flows.rs`**：`FlowContext` 新增 `base_url: Option<String>` 字段；`get()` 支持 `baseUrl`/`base_url` 占位符；`fill_template` 占位符列表加入 `baseUrl`；模块文档同步；测试新增 `{baseUrl}/oauth2/token` 替换断言
  - **`src-tauri/src/commands/frp/auth/oauth2.rs`**：`start_oauth2` 构造 `FlowContext` 传入 `base_url: Some(spec.base_url.clone())`
  - **`src-tauri/src/commands/frp/auth/device_code.rs`**：`DeviceCodeSession` 新增 `base_url: String` 字段；`start_device_code` 请求与 `poll_device_code` 轮询构造 `FlowContext` 均传入 base_url（poll 阶段已不重新加载 spec，baseUrl 存会话复用）
  - **`src-tauri/src/commands/frp/auth/mod.rs`**：`refresh_token` 构造 `FlowContext` 传入 base_url
- 设计决策：
  - **支持 `{baseUrl}` 占位符而非改 endpoints.json**：两个厂商（loliaFrp / Frp Test）的 endpoints.json 均以 `{baseUrl}/...` 声明认证端点，是既定模板规范；在 flows 引擎补上占位符支持可让所有厂商受益，避免在配置文件里重复写完整 URL
  - **baseUrl 存入 DeviceCodeSession**：`poll_device_code` 轮询阶段不再加载 endpoints.json，baseUrl 在 `start_device_code` 时存入会话，轮询时取出使用
- 验证：`cargo check --lib` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib`（141 通过 0 失败）均通过（exit 0）

#### 认证流程请求/响应调试日志（定位 token 解析失败）

- 背景：用户测试 LoliaFrp OAuth2 token 交换报「OAuth2 响应缺少 access_token」，但现有日志只打印请求方法/URL/contentType，不打印请求参数和响应体，无法判断是请求参数错误还是响应结构不匹配
- 改动（2 文件）：
  - **`src-tauri/src/commands/frp/auth/flows.rs`**：`send_flow_request` 日志增强
    - 请求日志追加 body（form-urlencoded 转为 `k=v&k=v` 串，JSON 直接打印）
    - 新增响应日志：`HTTP {status} - {body}`（读 body 后打印）
    - 请求/响应日志均经 `log_redact::redact_log` 脱敏，避免 token/secret 明文泄漏
  - **`src-tauri/src/commands/frp/auth/oauth2.rs`**：`access_token` 提取失败时错误消息附带脱敏后的响应体（`OAuth2 响应缺少 access_token（HTTP {status}，响应: {...}）`），同时 log_error 输出
- 设计决策：
  - **日志集中放在 flows 引擎**：所有认证流程（oauth2/device_code/refresh）都经 `send_flow_request`，一处加日志全流程受益，避免各调用方重复埋点
  - **复用 `log_redact::redact_log`**：响应体可能含 access_token/refresh_token，打印前脱敏，符合日志安全约定
- 验证：`cargo check --lib` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib`（141 通过 0 失败）均通过（exit 0）

#### 修复 loliaFrp endpoints.json OAuth2 token 响应提取路径（对齐标准扁平结构）

- 背景：用户测试 LoliaFrp OAuth2 认证，token 交换返回 HTTP 200 且响应为标准 OAuth2 扁平结构 `{"access_token": ..., "refresh_token": ..., "expires_in": ...}`，但程序仍报「OAuth2 响应缺少 access_token」。根因：loliaFrp 的 endpoints.json 中 token/refresh 响应提取路径误用业务接口的包裹格式 `$.data.access_token`/`$.msg`，而 OAuth2 token 接口响应是扁平结构（无 data 包裹，错误字段为 `error`/`error_description`）
- 改动（1 文件）：
  - **`docs/loliaFrp/api/endpoints.json`**：`authFlows.oauth2.token` 与 `refresh` 的 response 路径修正：
    - `$.data.access_token` → `$.access_token`，`$.data.refresh_token` → `$.refresh_token`，`$.data.expires_in` → `$.expires_in`
    - `errorField` `$.msg` → `$.error`，`errorDescription` `$.msg` → `$.error_description`
- 排查结论：
  - **`docs/Frp Test/frp/api/endpoints.json`**（模板）已是标准路径 `$.access_token`/`$.error`/`$.error_description`，无同类问题
  - **教程文档 `tutorial-frp.html`** 全部示例均为 `$.access_token` 扁平路径，无同类问题
  - **jsonpath 引擎** `extract(&v, "$.access_token")` 已有 `test_extract_simple` 测试覆盖，代码侧无需改动
- 验证：配置文件 JSON 语法无误；`cargo check --lib` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib`（141 通过 0 失败）均通过（exit 0）。注意：本配置位于 git 忽略的 docs/ 目录，需重新安装 lolia-frp 厂商（或手动更新已安装配置）后生效

#### 启用 keyring 平台后端（修复认证成功后状态仍显示未认证）

- 背景：用户测试 LoliaFrp OAuth2 认证，日志显示「OAuth2 认证成功: provider=lolia-frp, expires_at=...」，但 `get_auth_status` 返回 `{"authType":"oauth2","authenticated":false}`，且无 expiresAt/scopes。根因：`Cargo.toml` 中 `keyring = "3"` 未显式启用任何平台后端。keyring 3 的 default features 为空，Windows/macOS 未启用 `windows-native`/`apple-native` 时回退到 **mock 后端**（纯内存、无持久化、每次 `Entry::new` 独立创建空凭证）。因此 token 写入后立即用新 Entry 读取必得 `NoEntry`，且程序重启即丢
- 改动（1 文件）：
  - **`src-tauri/Cargo.toml`**：`keyring = { version = "3", features = ["windows-native", "apple-native", "sync-secret-service"] }`，按平台启用 Windows Credential Manager / macOS Keychain / Linux Secret Service 后端（与代码注释声明的设计意图一致）；并补充注释说明为什么必须显式启用
- 排查依据（已核验 keyring 3.6.3 源码）：
  - `src/lib.rs` L296-297：`#[cfg(all(target_os = "windows", not(feature = "windows-native")))] pub use mock as default;`
  - `src/mock.rs`：mock 凭证 "no persistence other than in the entry itself"，每次 Entry::new 独立空凭证 → store 后新 Entry load 必 NoEntry
  - `src/windows.rs`：windows-native 用 `CredWriteW`/`CredReadW` 写 Windows Credential Manager Generic 凭据（持久化），target name 为 `{user}.{service}`（`access_token.frp:lolia-frp`），service 含冒号合法
- 验证：`cargo check --lib`（keyring 3.6.3 按 windows-native 重新编译）+ `cargo clippy --all-targets -- -D warnings` + `cargo test --lib`（141 通过 0 失败）均通过（exit 0）
- 注意：启用后端后需**重新完成一次 OAuth2 认证**，token 才会持久化到 Windows 凭据管理器（旧的 mock token 未持久化，已丢失）

#### 修复刷新 token 401 invalid_client（refresh 请求未携带 client_secret）

- 背景：用户测试 LoliaFrp，认证成功并持久化后，刷新 token 报 `HTTP 401 Unauthorized - invalid_client`。根因：`refresh_token`（`src-tauri/src/commands/frp/auth/mod.rs`）构造 `FlowContext` 时只传了 `client_id`，`client_secret` 为 `None`；而 endpoints.json 的 refresh body 含 `"client_secret": "{clientSecret}"`，占位符无值，请求体留下字面 `{clientSecret}`，Lolia 服务端校验客户端凭据失败返回 invalid_client
- 改动（1 文件）：
  - **`src-tauri/src/commands/frp/auth/mod.rs`**：`refresh_token` 按 authType 同时解析 `client_id` 与 `client_secret`（`resolve_oauth2_config`/`resolve_device_code_config` 均返回 `AuthFile*::client_secret: Option<String>`），一并传入 `FlowContext`
- 排查结论（RFC 6749 §6 刷新 token 要求客户端认证）：
  - **loliaFrp refresh body** 含 `{clientSecret}` → 必须传 client_secret（本次修复）
  - **device_code poll body**（Frp Test 模板）只有 `grant_type/device_code/client_id`，不含 `{clientSecret}` → poll 不传 client_secret 无问题（RFC 8628 公开客户端可省略），已核对无需改动
  - **device_code start 流程** 此前已传 client_secret，无此问题
- 验证：`cargo check --lib` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib`（141 通过 0 失败）均通过（exit 0）。注意：需重新编译后端生效；调试日志（flows.rs 请求 body 打印）可直接核对 refresh 请求中 client_secret 是否真实传入（会脱敏显示）

#### 厂商列表显示外部厂商 frpc 就绪状态

- 背景：用户安装的 lolia-frp 厂商在厂商列表不显示 frpc 是否就绪（未在 bin 目录放置客户端），而内置「系统默认」厂商有就绪/未就绪状态显示。根因：`ProviderList.vue` 的模板中 frpc 就绪状态只在 `v-if="provider.builtin"` 分支渲染，外部厂商走 `v-else` 分支只显示启禁 Select + 卸载按钮
- 改动（2 文件）：
  - **`src/components/frp/ProviderList.vue`**：外部厂商操作区增加 frpc 就绪状态显示（绿色 CheckCircleIcon「frpc 就绪」/ 黄色 ExclamationCircleIcon「frpc 未就绪」，与内置厂商样式一致）；更新组件注释说明外部厂商 frpc 来源（bundled 手动放入 bin 目录 / url 安装包自带）
  - **`src/components/frp/TunnelCreateForm.vue`**：未就绪提示文案「请先在厂商列表页下载 frpc」改为「确认客户端已就绪」（外部厂商无下载按钮，原文案误导）
- 排查依据：后端 `list_providers`/`build_provider_info` 对内外部厂商均返回 `frpc_ready`（`is_external_frpc_ready` 检查 `providers/<id>/bin/...` 是否存在），数据层无问题，纯前端渲染缺失
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### 厂商服务器白名单支持 `*.domain` 通配符

- 背景：用户测试 LoliaFrp，启动/导入隧道报「服务器地址 jp-4.qwq.fan 不在厂商 lolia-frp 的允许列表内」。根因：loliaFrp 是平台型厂商，frps 节点按地区动态分配（`jp-4.qwq.fan` 仅日本 4 号节点，还有 us/hk/sg 等），manifest 配置 `allowCustomServer:false` + 空 `allowedServers` 导致所有服务器被拒；白名单机制本为「托管型」厂商（固定 frps）设计，无法穷举平台动态节点
- 改动（5 文件）：
  - **`src-tauri/src/commands/frp/binary/external.rs`**：`host_matches`（原私有，支持 `*.example.com` 一级通配符）提升为 `pub(crate)` 供沙箱复用
  - **`src-tauri/src/commands/frp/binary/mod.rs`**：`pub(crate) use external::host_matches;` 导出
  - **`src-tauri/src/commands/frp/sandbox.rs`**：`validate_network_permissions` 白名单匹配新增通配符分支（`s_host == addr_host || host_matches(addr_host, s_host)`），原有完整 `host:port` 匹配、host 匹配行为不变；新增 3 个单元测试（通配符/精确/白名单三种形式）
  - **`docs/loliaFrp/manifest.json`**：`allowedServers: []` → `["*.qwq.fan"]`（匹配任意子节点域名）
  - **`src-tauri/resources/templates/tutorial-frp.html`**：`allowedServers` 字段说明补充通配符支持
- 设计决策：
  - **复用既有 `host_matches`**：项目已有通配符匹配实现（binary.rs 下载域名白名单用），不重复实现，仅提升可见性后导出
  - **保持 `allowCustomServer:false`**：仍禁止任意自定义服务器，仅放行官方节点域名 `*.qwq.fan`；内网 SSRF 检查（`is_private_address`）不受影响
- 验证：`cargo check --lib` + `cargo clippy --all-targets -- -D warnings` + `cargo test --lib`（144 通过 0 失败）均通过（exit 0）
- 注意：manifest 变更位于 git 忽略的 docs/ 目录，需**重新安装 lolia-frp 厂商**（或手动更新已安装的 `providers/lolia-frp/manifest.json`）后生效

#### 前端体验优化：同步面板自动关闭 + 厂商下拉放开未就绪过滤 + Toast 点击展开

- 背景：用户反馈三点体验问题——① 从厂商同步导入全部隧道后仍需手动点按钮收起面板；② 新建/编辑隧道的厂商下拉看不到已导入的 lolia-frp（其 frpc 未就绪被过滤）；③ 长 toast 消息被单行省略截断，无法查看完整内容
- 改动（4 文件）：
  - **`src/components/frp/RemoteTunnelSync.vue`**：新增 `close` 事件；`handleImport` 导入后若所有远程隧道均已导入（`every` 检查 `importedIds`），自动 emit `close`
  - **`src/components/frp/TunnelManager.vue`**：`<RemoteTunnelSync>` 监听 `@close="showSync = false"` 收起同步面板
  - **`src/components/frp/TunnelCreateForm.vue`**：`providerOptions` 由「enabled 且 frpcReady 或 builtin」放宽为「enabled」（未就绪厂商可选，表单下方保留「frpc 未就绪」警告提示）；编辑模式下确保当前隧道厂商即使在选项中（被禁用时也并入）
  - **`src/components/common/Toast.vue`**：`ToastItem` 新增 `expanded` 状态；点击 toast 切换展开/收起；展开时文字 `white-space: normal` + `word-break: break-all` 换行显示完整内容，`max-height` 限制约 4 行（104px）超出可滚动，`.toast-item` 加 `cursor: pointer` 提示可点击
- 设计决策：
  - **同步面板自动关闭**：以「是否全部导入」为判断依据（`remoteTunnels.every`），非固定延迟，与用户操作闭环一致；`close` 由子组件通知父组件收起，保持单向数据流
  - **厂商下拉放开过滤**：原始 `(p.frpcReady || p.builtin)` 过滤导致从厂商同步导入的隧道（厂商未就绪）无法在编辑/新建时选择对应厂商；未就绪安全性由表单既有的警告提示兜底，编辑场景额外保证当前隧道厂商必然可选
  - **Toast 点击展开**：保持默认单行省略（不挤占空间），交互式展开而非默认全部换行，符合「最小视觉干扰」；高度上限 104px（约 4 行）+ 内滚动防止超大消息撑满屏幕
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### 同步面板导入逻辑修正（识别已导入隧道、失败不关闭）

- 背景：用户测试同步导入时发现——点击导入**已存在**的隧道，面板"没任何提示就自己关闭了"。根因：`handleImport` 同步 `emit('import')` 后立即标记 `importedIds` 并判断 `every` 关闭面板，而父组件 `store.createTunnel` 是异步的——同名隧道创建失败（store 的 toastError）时面板已提前关闭；且面板完全不识别本地已存在的隧道（每次打开 `importedIds` 重置）
- 改动（2 文件）：
  - **`src/components/frp/RemoteTunnelSync.vue`**：导入逻辑改为直接调用 `useFrpStore().createTunnel`（不再 emit `import` 给父组件中转）
    - `isImported(tunnel)` 判断改为「本会话已导入 或 本地已存在同名隧道」（`localTunnelNames` 由 `store.tunnels` 派生），已存在隧道显示「已导入」不可重复导入
    - `handleImport` 改为 async：导入中按钮显示 loading；**成功才**计入自动关闭判断（`every` 全导入后 emit `close`）；失败不标记不关闭，错误由 store 的 `toastError('创建隧道失败：...')` 提示
    - 移除 `import` 事件定义
  - **`src/components/frp/TunnelManager.vue`**：移除 `handleRemoteImport` 及 `@import` 绑定（导入已内聚到子组件）
- 设计决策：
  - **导入内聚到子组件**：RemoteTunnelSync 需要「导入是否成功」的结果来决定标记/关闭，而 Vue emit 无返回值，中转父组件无法回传状态；直接调用 store 保持单向数据流且职责清晰
  - **按名称识别已导入**：远程隧道 id 与本地隧道 id 由导入时重新生成（createTunnel 新建），无稳定对应关系；名称是唯一可靠匹配键（后端已校验隧道名唯一）
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### Dev API 新增 reload 刷新命令（支持强制无缓存刷新）

- 背景：用户调试时无法强制无缓存刷新前端页面，请求在 `window.molaunch` 调试 API 增加刷新命令
- 改动（1 文件）：
  - **`src/utils/dev-api.ts`**：新增 `molaunch.reload(force?)` 命令
    - 无参 / `false`：普通刷新（`location.reload()`）
    - `true`：强制无缓存刷新——URL 追加时间戳参数 `_molaunch_reload=<Date.now()>` 后重新导航，URL 变化使浏览器绕过本地缓存重新拉取资源（WebView2 对全新 URL 不做缓存命中）
  - `MolaunchDevAPI` 接口、`HELP_TEXT`、示例区同步更新
- 设计决策：
  - **不使用 Tauri v2 Webview 的 reload/navigate**：`@tauri-apps/api/webview` 的 `Webview` 类无 `reload`/`navigate` 方法（为 v1 API），改用浏览器原生 `location` API，无额外依赖
  - **时间戳参数而非清缓存**：未使用 `clearAllBrowsingData()`（会清 localStorage/sessionStorage，导致登录态/主题色丢失）；时间戳 query 只绕过资源缓存，保留应用状态
  - **history 路由兼容**：Vue Router 使用 `createWebHistory`，时间戳加在 search 上不影响 pathname 路由匹配，刷新后停留在当前页面
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### Toast 修复点击展开 + 悬停不自动关闭

- 背景：用户反馈两问题——① 点击 toast 不展开省略的长消息；② 鼠标悬停在 toast 上时仍会自动关闭
- 根因 ①：`.toast-container` 设了 `pointer-events: none`（为避免左下角空白区挡点击），但 `.toast-item` 未恢复 `pointer-events: auto`，导致 click/mouseenter 事件全部穿透无法触发——`toggleExpand` 从未执行
- 改动（1 文件，`src/components/common/Toast.vue`）：
  - **点击展开修复**：`.toast-item` 加 `pointer-events: auto`（容器保持 none 不影响空白区穿透）；`.toast-accent` 加 `align-self: stretch` 修复展开态父容器高度 auto 时竖条 `height:100%` 塌陷
  - **悬停不自动关闭**：`ToastItem` 新增 `timer?` 字段统一管理自动关闭定时器；`@mouseenter` 调 `pauseAutoDismiss`（clearTimeout），`@mouseleave` 调 `resumeAutoDismiss`（重新计时）
  - **展开暂停关闭**：`toggleExpand` 展开时同时暂停自动关闭（配合悬停可从容阅读完整内容），收起时恢复
  - `show`/`dismiss`/`shake` 统一改用 `timer` 字段管理，避免重复 `setTimeout` 泄漏
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### Toast 第二轮修复：竖条全高 + 滑出动画 + 移出 2s 关闭

- 背景：用户复测后反馈三点——① 展开后左侧颜色竖条消失；② 消失动画要能"从右往左滑回去"；③ 鼠标移出后希望 2s 自动关闭（而非按文字长度计时）
- 改动（1 文件，`src/components/common/Toast.vue`）：
  - **竖条全高**：`.toast-accent` 从 flex 子项改 `position: absolute; left:0; top:0; bottom:0`（父 `.toast-item` 加 `position: relative`），任何高度下都撑满，不再依赖 `height:100%`/`align-self` 的 auto 塌陷问题；`.toast-icon` 左边距 10px→13px 补偿竖条脱流后的对齐
  - **滑出动画**：`toast-out` 从 `translateX(-60px)` 改为 `translateX(-100%)`（完整向左滑出，与进入方向相反，视觉"从右往左滑回去"），时长 0.25s→0.3s，高度收拢延迟 0.2s→0.25s 错开
  - **移出 2s 关闭**：`resumeAutoDismiss` 固定 `2000ms` 重新计时（原按 `calcDuration` 文字长度计时）
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### Toast 修复滑出动画不可见（基类样式为隐藏态导致瞬间消失）

- 背景：用户复测发现 toast 消失时**直接消失**，没有任何右滑过渡动画
- 根因（CSS animation-fill-mode 经典坑）：`.toast-item` 基类样式是隐藏态 `opacity:0; transform:translateX(-80px)`。Toast 正常显示靠 `.toast-enter` 的 `forwards` 填充冻结在可见态；当 `hiding=true` 切换为 `.toast-hiding` 时，`toast-enter` 被移除 → forwards 填充失效 → 元素**瞬间回落基类隐藏态**（不可见），随后 `toast-out` 动画在全程不可见的状态下滑动，肉眼无动画
- 改动（1 文件，`src/components/common/Toast.vue`）：`.toast-item` 基类改为可见结束态 `opacity:1`（移除 `opacity:0; translateX(-80px)`），初始隐藏态由 `toast-in` 动画的 `0%` 关键帧提供；退出动画 `toast-out` 现在从可见态（translateX(0)/opacity:1）滑到 `translateX(-100%)/opacity:0`，动画可见
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### FRP 字段映射反序列化 + 认证中心图标修复

- 背景：用户测试 LoliaFrp 厂商认证报「解析 endpoints.json 失败: invalid type: string "id", expected struct FieldMapping」；同时认证中心不加载厂商 logo（都是默认图标）
- 改动（2 文件）：
  - **`src-tauri/src/commands/frp/types.rs`**：`FieldMapping` 改为手动实现 `Deserialize`，通过 untagged 枚举支持三种形式：
    - 字符串（`"id"`）→ `field: Some("id")`（直接取 item 字段）
    - 模板字符串（`"{account.token}"`）→ `value: Some(...)`（引用账号信息）
    - 对象（`{"field": "connectAddress", "split": ":"}`）→ 按字段解析
  - **`src/components/frp/AuthCenter.vue`**：认证中心厂商卡片图标从硬编码 `ShieldCheckIcon` 改为 `provider.icon`（有 icon 显示 `<img>`，无 icon 回退默认图标），与 ProviderList 保持一致
- 设计决策：
  - **字符串形式映射到 field**：`FieldMapping` 结构体手动反序列化，字符串默认为厂商字段名；以 `{` 开头视为模板引用 `value`，与 `resolve_field` 的优先级（value → field）匹配
  - **不新增测试文件**：FieldMapping 反序列化逻辑通过 untagged 枚举天然支持三种形式，与现有 `HashMap<String, FieldMapping>` 字段兼容
- 验证：`cargo check` + `cargo clippy --all-targets -D warnings` + `cargo test --lib`（140 通过）+ `npx vue-tsc --noEmit` 均通过（exit 0）

#### FRP 厂商认证配置读取修复（auth.json 回退 + 测试补齐）

- 背景：测试 LoliaFrp 厂商调用 `start_oauth2` 报「厂商 lolia-frp 的 manifest 缺少 auth.oauth2 配置」。原因：新设计将 OAuth2 交互配置（authorizeUrl/clientId/scopes/redirectPort）存放在 auth.json（`AuthFileOAuth2`），但 `oauth2.rs` 仍从 `manifest.auth.oauth2`（旧 `OAuth2Config`）读取。LoliaFrp 的 manifest 未内嵌 auth 块，因此报错
- 改动（6 文件）：
  - **`src-tauri/src/commands/frp/provider.rs`**：
    - 新增 `read_auth_file()`：按 manifest.authFile 相对路径读取并解析 auth.json，文件缺失/解析失败返回 None
    - 重构 `resolve_auth_type()`：复用 `read_auth_file`（行为不变）
    - 新增 `resolve_oauth2_config()` / `resolve_device_code_config()`：从 auth.json 读取 `AuthFileOAuth2` / `AuthFileDeviceCode`，错误消息明确指向 auth.json
  - **`src-tauri/src/commands/frp/auth/oauth2.rs`**：`require_oauth2_config(&manifest.auth)` 改为 `resolve_oauth2_config(provider_id, &manifest)`，从 auth.json 读取
  - **`src-tauri/src/commands/frp/auth/device_code.rs`**：两处 `require_device_code_config(&manifest.auth)` 改为 `resolve_device_code_config(provider_id, &manifest)`
  - **`src-tauri/src/commands/frp/auth/mod.rs`**：`refresh_token` 取 clientId 从 `manifest.auth.oauth2/device_code` 改为按 `resolve_auth_type` 匹配 `resolve_oauth2_config` / `resolve_device_code_config`
  - **`src-tauri/src/commands/frp/auth/api_key.rs`**：`manifest.auth.auth_type` 判断改为 `resolve_auth_type`（支持 auth.json 声明 type 的厂商）
  - **`src-tauri/src/commands/frp/auth/storage.rs`**：删除已无调用方的 `require_oauth2_config` / `require_device_code_config` / `require_api_key_config` 及无用 import
- 设计决策：
  - **统一从 auth.json 读取交互配置**：manifest.json 只保留 `authFile` 指针和 `auth.type`（缺省 none），OAuth2/Device Code 的交互参数统一收敛到 auth.json，与 endpoints.json（请求/响应规范）职责分离
  - **resolve_* 系列函数放 provider.rs**：与 `resolve_auth_type` 同处，避免 auth 子模块反向依赖造成循环引用
  - **不新增 `resolve_api_key_config`**：api_key 的请求头注入规范在 endpoints.json `authFlows.api_key` 中定义，auth.json 的 `api_key` 块当前无读取方，按最小修改原则不新增死代码
- 顺带修复（clippy/test 全量验证暴露的预存问题）：
  - **`src-tauri/src/minecraft/community/mcmod/mod.rs`**：测试导入 `extract_words` 从错误的 `parsers` 改为 `search`（重构遗留，修复 `cargo clippy --all-targets` E0432）
  - **`src-tauri/src/minecraft/download/rate_limiter_tests.rs`**：`granted >= 40 && granted <= 60` 改为 `(40..=60).contains(&granted)`（clippy manual-range-contains）
  - **`src-tauri/src/minecraft/online/auth_tests.rs` / `storage_tests.rs`**：`Default::default()` + 逐字段赋值改为结构体初始化（clippy field-reassign-with-default）
  - **`src-tauri/src/commands/frp/log_redact.rs`**：正则 `["']?` 吞掉 JSON key 右引号导致 `{"token:"***"}` 输出残缺，改为捕获组保留引号，修复 `redacts_json_token` 测试
  - **`src-tauri/src/commands/frp/api_spec/envelope.rs`**：测试中 envelope 字段路径 `"flag"` 改为 `"$.flag"`（jsonpath 要求 `$` 前缀），修复 4 个 envelope 测试
- 验证：`cargo check` + `cargo clippy --all-targets -D warnings` + `cargo test --lib`（140 通过 0 失败）均通过（exit 0）

#### 简单表单弹窗样式还原（移除误加的高度限制）

- 背景：之前统一弹窗高度限制时，将 `modal-shell`/`modal-body`/`modal-scroll` 三个 CSS 工具类应用到所有弹窗，但输入框、复选框、单选按钮等简单表单弹窗不应使用高度限制（`max-height: calc(100vh - 100px)`）和滚动区，且应垂直居中而非顶部对齐。本次还原这5个弹窗为简单居中布局
- 改动（5 文件）：
  - **`src/components/common/Modal.vue`**：通用弹窗（error/warning/info/success/confirm/prompt），含输入框模式，还原为居中显示
  - **`src/components/online/KickConfirmDialog.vue`**：踢出确认弹窗，含封禁时长单选卡片，还原为居中显示
  - **`src/components/online/LobbyJoinConfirmDialog.vue`**：加入房间确认弹窗，含整合包校验卡片，还原为居中显示
  - **`src/views/tools/archive/ArchiveBackupDialog.vue`**：存档备份弹窗，含输入框 + 复选框，还原为居中显示
  - **`src/views/tools/data/LoadSaveModal.vue`**：从存档加载种子弹窗，含两个 Select 下拉框，还原为居中显示
- 改动内容：每个弹窗3处类名替换
  - `modal-shell` → `fixed inset-0 z-[10000] flex items-center justify-center p-4`（`items-start` → `items-center` 居中，`pt-14 pb-4` → `p-4` 四边等距）
  - `modal-body max-w-xxx mt-2` → `relative w-full max-w-xxx bg-white rounded-lg shadow-xl`（移除 `max-height` 和 `flex flex-col`，去掉 `mt-2`）
  - `modal-scroll px-x py-x` → `px-x py-x`（移除 `flex-1 overflow-y-auto` 滚动特性，保留 padding）
- 设计决策：长内容弹窗（如 ResourceDetail、教程、日志查看等）保留 `modal-shell`/`modal-body`/`modal-scroll` 高度限制方案不变；简单表单弹窗直接 inline tailwind 类，不新增 CSS 工具类，避免类名膨胀
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

### 重构

#### FRP 厂商接口规范改造（阶段 3+5+8：auth 重写 + 调用方切换 + 旧模块清理）

- 背景：阶段 1+2 已完成类型定义和 api_spec 引擎，但 auth 模块（oauth2.rs / device_code.rs / refresh_token）仍用硬编码 form 请求 + TokenResponse 反序列化，无法适配 OpenFRP 等非标准厂商。本次完成全量切换：所有认证流程由 flows.rs 引擎按 endpoints.json authFlows 配置驱动，删除旧 api_schema 模块
- 改动（7 文件）：
  - **`src-tauri/src/commands/frp/auth/oauth2.rs`**：重写 start_oauth2，删除 `exchange_code_for_token` 硬编码 form 请求 + `TokenResponse` 反序列化。改为加载 endpoints.json `authFlows.oauth2.token` 配置，通过 `flows::send_flow_request` 引擎构造请求，按 `response.accessToken/refreshToken/expiresIn/errorField/errorDescription` FieldExtractor 解析响应。保留 `build_authorize_url`（用户交互层标准 OAuth2 流程）和 `wait_for_callback`（本地 HTTP 服务接收回调）
  - **`src-tauri/src/commands/frp/auth/device_code.rs`**：重写 start_device_code + poll_device_code，删除 `DeviceCodeResponse` 反序列化和硬编码 form 请求。改为读 `authFlows.device_code.request/poll` 配置，通过 flows 引擎发送请求，按 `deviceCode/userCode/verificationUri/pollInterval/expiresIn/accessToken/refreshToken/errorField` FieldExtractor 解析。DeviceCodeSession 内存会话改存 `poll_flow: FlowRequest` 和 `pending_error: Option<String>` 替代原 `token_url + client_id`，运行时按配置驱动轮询
  - **`src-tauri/src/commands/frp/auth/mod.rs`**：重写 `refresh_token`，删除硬编码 form 请求 + `TokenResponse` 反序列化。改为读 `authFlows.oauth2.refresh` 配置（缺失时回退到 `oauth2.token`），通过 flows 引擎驱动。删除 `TokenResponse` 内部类型，新增 `get_extractor` 和 `extract_flow_error` 内部辅助函数（与 oauth2.rs/device_code.rs 中同名函数对齐）
  - **`src-tauri/src/commands/frp/types.rs`**：`OAuth2Config` 和 `DeviceCodeConfig` 新增 `client_secret: Option<String>` 字段（部分厂商需要）
  - **`src-tauri/src/commands/frp/api_spec/mod.rs`**：`TunnelInfo` 和 `AccountInfo` 加 `Serialize` 派生（供 provider_actions 返回前端）；修复 `if let Some(ref x)` 引用重复警告；移除未使用的 HashMap 导入
  - **`src-tauri/src/utils/frp_manager/provider_actions.rs`**：`fetch_vendor_config` action 重命名为 `fetch_tunnels`，调用切换到 `frp::api_spec::fetch_tunnels`，返回结构改为 `{tunnels, account}` 对象（前端按 camelCase 取值）
  - **`src-tauri/src/commands/frp/mod.rs`**：移除 `pub mod api_schema` 声明
  - 删除：`src-tauri/src/commands/frp/api_schema/` 目录（mod.rs / helpers.rs / http.rs / mapping.rs / tests.rs），共 5 个文件。旧 `api_schema` 模块基于硬编码 api-schema.json，已被 `api_spec` 模块（基于可配置 endpoints.json）完全替代
- 设计决策：
  - 占位符统一通过 `flows::FlowContext` 传递（`{clientId} {clientSecret} {redirectUri} {code} {scope} {deviceCode} {refreshToken} {apiKey} {publicKey} {requestUuid}` 等），flows.rs 引擎递归填充 body 模板，form-urlencoded 自动转换为 key=value 对
  - 响应字段提取支持 `from=body`（JSONPath）和 `from=header`（响应头字段名），覆盖 OpenFRP 的 token 在响应 Header 的非标准场景
  - `OAuth2Flow.refresh` 缺失时回退到 `OAuth2Flow.token`，适配部分厂商用同一端点刷新的场景
  - Device Code 会话内存存储 `poll_flow: FlowRequest` 而非 `token_url + client_id`，确保运行时按配置驱动而非硬编码
- 验证：`cargo check` + `cargo clippy -D warnings` + `npx vue-tsc --noEmit` 均通过（exit 0）

#### FRP 厂商接口规范改造（阶段 1+2：类型定义 + API 引擎）

- 背景：各厂商接口响应结构各不相同（成功判断、字段命名、数据位置、配置获取方式均有差异）。原 `api_schema` 模块使用固定结构解析，无法适配非标准厂商（如 OpenFRP）。新设计将接口响应解析全部做成可配置项，厂商只需在 `endpoints.json` 中声明即可
- 改动（6 文件）：
  - **`src-tauri/src/commands/frp/types.rs`**：新增 18 个类型定义，覆盖 endpoints.json 全部可配置项
    - `ApiSpec`（顶层结构）/ `AuthHeader`（token 注入）/ `AuthFlows`（认证流程集合）
    - `OAuth2Flow` / `DeviceCodeFlow` / `ApiKeyFlow` / `RemoteLoginFlow`（四种认证流程）
    - `FlowRequest`（流程请求定义 + 响应字段提取规则）/ `FieldExtractor`（body/header 取值）
    - `Envelope`（响应包裹解析：successField/successValue/errorField/dataField）
    - `ConfigMode`（三种配置模式：url/fields/args）/ `EndpointsDef` / `EndpointDef` / `TunnelsDef`
    - `ResponseDef`（itemsField/itemsField/tunnelIdField/fields/encoding）/ `FieldMapping`（field/split/value）
    - `AuthFile` / `AuthFileOAuth2` / `AuthFileDeviceCode` / `AuthFileApiKey`（认证交互层）
    - `ProviderManifest` 新增 `auth_file` 和 `api.endpoints_file` 字段
  - **`src-tauri/src/commands/frp/api_spec/mod.rs`**（新模块）：API 引擎主模块，`load_api_spec` 加载 endpoints.json + `fetch_tunnels` 拉取隧道列表 + 账号信息映射 + 隧道列表映射（含 `{account.token}` 占位符解析）
  - **`src-tauri/src/commands/frp/api_spec/jsonpath.rs`**（新模块）：JSONPath 解析，支持 `$.a.b` 嵌套字段 + `$.data[*].proxies[*]` 多级数组展平，含 5 个单元测试
  - **`src-tauri/src/commands/frp/api_spec/envelope.rs`**（新模块）：响应包裹解析，envelope 成功判断（支持类型宽松匹配）+ 错误消息提取 + 数据字段提取，含 4 个单元测试
  - **`src-tauri/src/commands/frp/api_spec/http.rs`**（新模块）：HTTP 请求发送，复用 api_schema 重定向防护逻辑，适配新 EndpointDef 类型，集成 envelope 成功校验
  - **`src-tauri/src/commands/frp/api_spec/config_gen.rs`**（新模块）：配置生成，三种模式实现：url 直写（预留 base64/xor/aes 解码扩展点）/ fields 拼 ini / args 启动参数，含 3 个单元测试
  - **`src-tauri/src/commands/frp/mod.rs`**：注册 `api_spec` 模块，re-export 新类型
- 设计决策：新模块与旧 `api_schema` 并存，避免一次性全量替换导致编译中断。后续阶段将重写 auth 模块、更新调用方、删除旧模块
- 验证：`cargo check` 通过（exit 0），含 12 个新增单元测试

#### FRP 厂商支持从 URL 下载安装

- 背景：厂商安装仅支持「从文件夹」和「从 ZIP」两种本地方式，厂商提供者无法通过一个链接让用户直接下载安装，需手动下载 ZIP 再选择文件
- 改动（3 后端 + 3 前端）：
  - **`src-tauri/src/commands/frp/install.rs`**：新增 `install_provider_from_url(url)` 函数。校验 HTTPS → reqwest 下载到临时文件（最多 5 次重定向）→ 复用 `install_provider_from_zip` 安装 → 无论成功失败都清理临时文件
  - **`src-tauri/src/utils/frp_manager/mod.rs`**：新增 `InstallProviderFromUrlParams { url }` 参数结构体
  - **`src-tauri/src/utils/frp_manager/provider_actions.rs`**：注册 `install_provider_from_url` action，反序列化参数调用后端函数
  - **`src/utils/api/frp-manager.ts`**：新增 `INSTALL_PROVIDER_FROM_URL` action 常量 + `installProviderFromUrl(url)` IPC 封装函数
  - **`src/stores/frp.ts`**：新增 `installProviderFromUrl(url)` store action（loading + toast + 刷新列表），与 `installProviderFromZip` 风格一致
  - **`src/components/frp/ProviderList.vue`**：顶部操作栏新增「从 URL」按钮（LinkIcon），点击调用 `showPrompt` 弹出输入框让用户粘贴 HTTPS URL，确认后调用 store action
- 设计决策：复用 `install_provider_from_zip` 全部安装逻辑（含 Zip Slip 防护、manifest 校验、重复检测），URL 下载仅负责获取临时 ZIP 文件；仅限 HTTPS（用户主动提供 URL，无需域名白名单）；临时文件始终清理
- 验证：`cargo check` + `npx vue-tsc --noEmit` 均通过（exit 0）

#### 教程系统（Markdown 渲染 + picker 子窗口）

- 背景：FRP 厂商开发文档（manifest.json 格式、OAuth2/Device Code/API Key 认证配置）无处查阅，开发者无从下手编写厂商包。需一个内置教程系统承载开发指南与启动器使用基础
- 改动（5 新增 + 4 修改）：
  - **新增 `src/tutorials/index.ts`**：教程元数据索引。`TutorialMeta` 接口含 id/title/description/category/content，`TUTORIALS` 数组集中注册所有教程。Markdown 内容通过 Vite `?raw` 后缀导入，新增教程只需在此文件追加一项
  - **新增 `src/tutorials/launcher-basics.md`**：启动器基础使用教程，覆盖版本安装、账号管理、联机功能入门
  - **新增 `src/tutorials/frp-provider-guide.md`**：FRP 厂商开发指南，详述 manifest.json 清单格式、认证配置（OAuth2/Device Code/API Key）、网络与进程权限
  - **新增 `src-tauri/resources/templates/tutorial.html`**：亮色教程渲染模板（与前端 Vue 应用白底灰字 + 主色蓝样式一致）。通过 `res://localhost/web-common/view/marked.min.js` 加载 marked.js 渲染 Markdown，支持 GFM 与换行。复制粘贴右键禁用（标题栏区），内容区可选中文本
  - **`src/config/picker-templates.ts`**：新增 `tutorial` 模板配置（width=760, height=600），CSP 与 markdown 模板一致（允许 `res:` 加载 marked.min.js）
  - **`src/utils/picker-window.ts`**：新增 `openTutorialWindow(params)` 便捷函数，使用 `tutorial` 模板打开子窗口
  - **`src/vite-env.d.ts`**：新增 `declare module '*.md?raw'` 类型声明，支持 Markdown 文件原始内容导入
  - **`src/views/settings/more/TutorialTab.vue`**：从空占位改为分类展示教程列表（基础 / FRP 开发两组），点击「阅读」调用 `openTutorialWindow` 在 picker 子窗口渲染
  - **`src/views/settings/SettingsMore.vue`**：教程子页签说明更新；新增 `useRoute` 读取 URL `?subtab=tutorial` query 参数，支持外部页面深链到教程子页签
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### FRP 子菜单「教程」按钮深链跳转

- 背景：教程系统已落地，但 FRP 管理页（厂商列表/穿透管理/认证中心/运行日志）的开发者无法直接发现教程入口，需手动导航到设置-更多-教程
- 改动（3 修改）：
  - **`src/views/online/OnlineTopBar.vue`**：新增 `showHelp` prop 与 `goTutorial` emit。当 `showHelp=true` 时在状态徽章与设置按钮之间显示「教程」按钮（BookOpenIcon），点击 emit `goTutorial`
  - **`src/views/Online.vue`**：新增 `FRP_SUB_IDS` 常量（providers/tunnels/auth/logs）与 `showFrpHelp` computed（activeCategory 在 FRP_SUB_IDS 中时为 true）。新增 `goTutorial()` 跳转 `/apps/settings?tab=about&subtab=tutorial`，并绑定到 OnlineTopBar
  - **`src/views/settings/SettingsMore.vue`**：`onMounted` 读取 `route.query.subtab`，校验值在 subTabs 列表中后切换 activeSubTab，实现深链直达教程子页签
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

### 维护

#### 清理信令/FRP 模块装饰性分隔线注释

- 背景：`minecraft/online` 下 5 个 Rust 文件存在 AI 生成的 `// ====== xxx ======` 装饰性分隔线注释，用户要求清理该风格
- 改动（5 文件，共 14 处）：
  - **`src-tauri/src/minecraft/online/frp.rs`**（5 处）：frpc manifest / 公共 frps 服务器 / 分配端口 / 释放续期 / OnlineClient 扩展方法
  - **`src-tauri/src/minecraft/online/signaling/types.rs`**（3 处）：ICE / STUN / TURN、整合包元数据、房间核心类型
  - **`src-tauri/src/minecraft/online/signaling/lobby.rs`**（2 处）：大厅类型、OnlineClient 扩展方法
  - **`src-tauri/src/minecraft/online/signaling/session.rs`**（2 处）：封禁 / Offer 类型、OnlineClient 扩展方法
  - **`src-tauri/src/minecraft/online/signaling/whitelist.rs`**（2 处）：白名单类型、OnlineClient 扩展方法
- 设计决策：仅将 `// ===== 标题 =====` 形式的整行分隔线注释改为普通注释 `// 标题`，保留标题文字；不删除任何代码、不改动任何逻辑（`git diff` 验证均为纯注释行变更）
- 验证：`git diff` 确认 5 文件改动全部为注释行替换，代码零变更；grep 复查目标文件无残留分隔线

#### 清理联机/加密模块装饰性分隔线注释（第二批）

- 背景：续上一批清理，`utils/frp_manager`、`utils/online_manager`、`utils/signaling_manager`、`minecraft/online/crypto.rs` 仍存在 AI 生成的 `// ====== xxx ======` 装饰性分隔线注释，用户要求继续清理该风格
- 改动（4 文件，共 14 处）：
  - **`src-tauri/src/utils/frp_manager/mod.rs`**（2 处）：参数结构体、DISPATCHER 注册（两处均为"纯分隔线夹标题"三行结构）
  - **`src-tauri/src/utils/online_manager/mod.rs`**（3 处）：返回类型、辅助函数（子模块共用）、DISPATCHER 入口
  - **`src-tauri/src/utils/signaling_manager/mod.rs`**（3 处）：参数结构体、辅助函数（子模块共用）、注册入口
  - **`src-tauri/src/minecraft/online/crypto.rs`**（6 处）：Ed25519、X25519、HKDF、AES-256-GCM、RSA-OAEP、错误类型
- 设计决策：仅将 `// ===== 标题 =====` 形式的整行分隔线注释改为普通注释 `// 标题`，保留标题文字；两条纯分隔线夹标题的三行结构压缩为一行标题注释；不删除任何代码、不改动任何逻辑
- 验证：`cargo check` 通过（exit 0）；grep 复查 4 文件均无 `// ====` 残留

#### 清理装饰性分隔线注释（第三批）

- 背景：续上一批清理，`minecraft/image_cache.rs`、`minecraft/skin.rs`、`commands/system/developer.rs`、`commands/system/updater/install_windows.rs`、`minecraft/community/mcmod/search.rs`、`minecraft/auth/storage/mod.rs`、`minecraft/auth/authlib/types.rs`、`minecraft/auth/authlib/client/profile.rs` 仍存在 AI 生成的 `// ====== xxx ======` 装饰性分隔线注释，用户要求继续清理该风格
- 改动（8 文件，共 15 处）：
  - **`src-tauri/src/minecraft/skin.rs`**（4 处）：不使用 sources::fetch_with_fallback 说明、数据结构、披风别名中文映射、核心逻辑
  - **`src-tauri/src/minecraft/auth/storage/mod.rs`**（3 处）：认证存储管理器、加解密工具、缓存控制
  - **`src-tauri/src/minecraft/auth/authlib/types.rs`**（3 处）：请求结构、响应结构、角色属性与材质
  - **`src-tauri/src/minecraft/image_cache.rs`**（1 处）：Tauri URI scheme 注册
  - **`src-tauri/src/commands/system/developer.rs`**（1 处）：DevTools 控制
  - **`src-tauri/src/commands/system/updater/install_windows.rs`**（1 处）：后台静默下载 + 退出时替换
  - **`src-tauri/src/minecraft/community/mcmod/search.rs`**（1 处）：中文搜索本地映射
  - **`src-tauri/src/minecraft/auth/authlib/client/profile.rs`**（1 处）：yggdrasil 皮肤管理端点
- 设计决策：仅将 `// ===== 标题 =====` 形式的整行分隔线注释改为普通注释 `// 标题`，保留标题文字；两条纯分隔线夹标题的三行结构压缩为一行标题注释；不删除任何代码、不改动任何逻辑（`git diff` 验证均为纯注释行变更）
- 验证：`cargo check` 通过（exit 0）；grep 复查 8 文件均无分隔线残留

#### 修复 picker 子窗口无法加载 marked.min.js / qrcode.min.js

- 症状：教程/Markdown/二维码 picker 子窗口无法加载 `marked.min.js` / `qrcode.min.js`，页面显示「渲染失败：无法加载 marked.min.js」
- 根因：picker 子窗口 origin 为 `https://picker.localhost/`（Windows），res:// 资源在 Windows 上转为 `https://res.localhost/`，跨源 script 加载受 CSP 与跨源策略双重限制。原方案通过动态 `<script src="res://...">` 加载依赖库，在 picker 子窗口中不可靠
- 改动（4 文件）：
  - **`src-tauri/src/commands/tools/picker_window.rs`**：URI scheme handler 新增依赖库内联注入逻辑。markdown/tutorial 模板注入 `view/marked.min.js`，qrcode 模板注入 `view/qrcode.min.js`，作为 `<script>` 标签内联到 HTML（在 `__PICKER_DATA__` 注入之前），彻底消除 res:// 跨源加载依赖
  - **`src-tauri/resources/templates/tutorial.html`**：移除动态 `<script>` 加载逻辑，直接使用后端内联注入的 `marked` 全局变量渲染 Markdown
  - **`src-tauri/resources/templates/markdown.html`**：同 tutorial.html，移除动态加载，直接使用内联 `marked`
  - **`src-tauri/resources/templates/qrcode.html`**：同上，移除动态加载，直接使用内联 `QRCode`
  - **`src/config/picker-templates.ts`**：markdown/tutorial/qrcode 三个模板 CSP 的 `script-src` / `connect-src` 由 `res:` 改为 `res: https://res.localhost`（与主应用 CSP 口径一致，作为防御性配置保留）
- 设计决策：选择后端内联注入而非修 CSP，因为内联注入彻底消除跨源依赖，不依赖平台特定的 URL 转换行为，且 marked.min.js（~40KB）内联开销可忽略
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` + `npx vue-tsc --noEmit` 均通过（exit 0）

#### 教程系统改用硬编码 HTML 模板 + 教程入口移至 FRP 侧边栏

- 背景：后端内联注入 marked.min.js 方案在 picker 子窗口中仍不稳定（Windows res:// 跨源行为平台差异），用户反馈"还是不行"。同时教程入口放在 OnlineTopBar 按钮上不够直观，用户难以发现
- 改动（6 修改 + 3 删除 + 2 新增）：
  - **新增 `src-tauri/resources/templates/tutorial-basics.html`**：硬编码 HTML 启动器基础教程（无需 marked.min.js 渲染），样式与前端 Vue 应用一致
  - **新增 `src-tauri/resources/templates/tutorial-frp.html`**：硬编码 HTML FRP 厂商开发指南（同上）
  - **删除 `src-tauri/resources/templates/tutorial.html`**：旧的 Markdown 渲染模板，已被硬编码 HTML 模板替代
  - **删除 `src/tutorials/launcher-basics.md`**：Markdown 源文件，内容已迁移到 HTML 模板
  - **删除 `src/tutorials/frp-provider-guide.md`**：同上
  - **`src-tauri/src/resources.rs`**：注册 `tutorial-basics.html` / `tutorial-frp.html`，移除旧 `tutorial.html`
  - **`src-tauri/src/commands/tools/picker_window.rs`**：依赖库注入逻辑仅保留 markdown/qrcode 模板，tutorial-basics/tutorial-frp 硬编码 HTML 无需注入
  - **`src/config/picker-templates.ts`**：移除旧 `tutorial` 模板配置，新增 `tutorial-basics` / `tutorial-frp` 模板配置（CSP 使用 BASE_CSP，无需 res:）
  - **`src/tutorials/index.ts`**：`TutorialMeta` 接口 `content` 字段改为 `template` 字段（对应 picker 模板名），TUTORIALS 数组使用 `template: 'tutorial-basics'` / `'tutorial-frp'`
  - **`src/views/settings/more/TutorialTab.vue`**：点击「阅读」改为调用 `openDisplayWindow({ template, title })` 加载硬编码 HTML 模板
  - **`src/utils/picker-window.ts`**：移除 `openTutorialWindow` 便捷函数（已不再使用）
  - **`src/composables/useFrpSidebar.ts`**：FRP 侧边栏子菜单新增「教程帮助」子项（BookOpenIcon），点击跳转设置-教程页
  - **`src/views/Online.vue`**：移除 `showFrpHelp` computed 和 `FRP_SUB_IDS`，改用 `handleCategoryChange` 拦截 `tutorial` 动作项调用 `goTutorial()` 跳转；NavSidebar 从 `v-model` 改为 `:model-value` + `@update:model-value` 显式绑定
  - **`src/views/online/OnlineTopBar.vue`**：移除 `showHelp` prop、`goTutorial` emit、教程按钮（BookOpenIcon）和 BookOpenIcon 导入，恢复为纯标题栏 + 状态徽章 + 设置按钮
- 设计决策：放弃 Markdown 渲染方案改为硬编码 HTML，消除所有跨源加载依赖；教程入口从顶栏按钮移到 FRP 侧边栏子菜单，与 FRP 功能内聚，用户在 FRP 管理页面可直接发现教程
- 验证：`cargo check` + `npx vue-tsc --noEmit` 均通过（exit 0）

#### 教程模板 base-help.html 基础模板 + 右侧 TOC 导航

- 背景：tutorial-basics.html 和 tutorial-frp.html 各自重复约 70 行相同样式代码，新增教程需复制粘贴样式；同时缺少目录导航，长文档（如 FRP 开发指南 8 个章节）需手动滚动定位
- 改动（1 新增 + 2 重写 + 2 修改）：
  - **新增 `src-tauri/resources/templates/base-help.html`**：帮助文档基础模板。提供统一样式（标题栏 + 内容区 + 代码/表格/引用块等）+ 右侧 TOC 导航（复刻 ToolToc.vue 行为：≥3 项自动显示、收起灰色短横线、hover 展开标题、滚动高亮当前项、点击 smooth 跳转预留 20px 偏移）。从 `__PICKER_DATA__.title` 读取标题栏文字，从 `__PICKER_DATA__.content` 读取内容 HTML 并注入，扫描 h2 标题自动生成 TOC
  - **重写 `src-tauri/resources/templates/tutorial-basics.html`**：从完整 HTML 改为纯内容文件（仅 h1/h2/p/ul/ol 等内容标签，无 `<html>`/`<head>`/`<style>`/`<script>`），样式由 base-help.html 提供
  - **重写 `src-tauri/resources/templates/tutorial-frp.html`**：同上，纯内容文件
  - **`src-tauri/src/commands/tools/picker_window.rs`**：`open_picker_window` 存储数据时注入 `title` 字段（base-help 从 data.title 读取标题栏文字）；URI scheme handler 对 `tutorial-*` 模板读取 base-help.html 作为实际模板，将原始内容文件注入 `data.content`
  - **`src-tauri/src/resources.rs`**：注册 `templates/base-help.html`
- 设计决策：base-help.html 复刻 ToolToc.vue 的右侧 TOC 行为（收起/展开/滚动高亮/点击跳转），不引入新模式；tutorial-*.html 改为纯内容文件，新增教程只需写内容无需复制样式
- 验证：`cargo check` + `npx vue-tsc --noEmit` 均通过（exit 0）

#### 修复 base-help.html 教程窗口内容未加载（改用后端占位符替换）

- 症状：教程 picker 子窗口显示"无内容"，标题栏加载正常但内容区为空
- 根因：base-help.html 原始 `<script>` 在解析时立即执行读取 `window.__PICKER_DATA__`，但后端注入脚本 `<script>window.__PICKER_DATA__ = {...};</script>` 追加在原始 `<script>` 之后，执行顺序导致读取时尚为 undefined
- 改动（2 文件）：
  - **`src-tauri/resources/templates/base-help.html`**：改用后端占位符替换方案。模板中使用 `{{__TITLE__}}` 和 `{{__CONTENT__}}` 占位符，由后端在响应前完成字符串替换形成完整 HTML。移除所有 `__PICKER_DATA__` 读取逻辑与 DOMContentLoaded 包装，脚本仅保留 TOC 自动生成与交互（内容已在 DOM 中，脚本位于 body 底部可直接执行）
  - **`src-tauri/src/commands/tools/picker_window.rs`**：tutorial-* 模板改为后端占位符替换路径，读取 base-help.html 后将 `{{__TITLE__}}` 替换为标题、`{{__CONTENT__}}` 替换为教程内容，直接返回完整 HTML，跳过 `__PICKER_DATA__` 注入路径
- 设计决策：后端占位符替换形成完整 HTML，彻底消除 JS 运行时时序依赖，比 DOMContentLoaded 方案更简洁可靠

#### 修复联机设备登录"接口成功但前端报失败"

- 症状：点击联机页面"登录"按钮后，api-server 返回 HTTP 200 + 有效数据，但前端 toast 提示"设备登录失败，请稍后重试"；同时 `auth_status` 查询显示 `logged_in: true`（旧 token 仍有效）
- 根因：`LoginResponse` / `RegisterResponse` 的 `time` / `req_id` 字段缺少 `#[serde(default)]`（`RefreshResponse` 已有），api-server 返回时若缺失或类型不匹配这两个元数据字段，`serde_json::from_str` 整体失败 → 后端返回 `Err("登录请求失败: JSON 解析失败")` → 前端 toast 报错。但 api-server 已验证登录，旧 token 仍有效，`auth_status` 读本地旧凭证显示已登录
- 改动（3 文件）：
  - **`src-tauri/src/minecraft/online/auth/types.rs`**：`LoginResponse` / `RegisterResponse` 的 `time` / `req_id` 加 `#[serde(default)]`，与 `RefreshResponse` 保持一致，容忍服务端省略或类型不匹配这两个非业务字段
  - **`src/stores/online/authSlice.ts`**：`login()` / `register()` 的 `safeCall` 增加 `onError` 回调，toast 显示后端返回的真实错误信息（如 "登录请求失败: JSON 解析失败: ..."），不再吞掉错误细节
  - **`src/components/online/OnlineDevicePanel.vue`**：移除 `handleLogin` / `handleRegister` 的通用 "请稍后重试" toast（store 层已显示具体错误，避免重复 toast）
- 验证：`cargo check` + `npx vue-tsc --noEmit` 均通过（exit 0）

#### 统一前端弹窗高度限制（公共 CSS 工具类）

- 背景：项目中各弹窗高度限制实现不一，部分弹窗内容超长时会撑满甚至溢出视口。参考下载页 Mod 详情弹窗的高度限制方案，提取公共 CSS 工具类统一所有弹窗的布局与滚动行为
- 改动（1 修改 + 9 弹窗改造）：
  - **`src/assets/styles/main.css`**：新增三个公共工具类——`.modal-shell`（外层定位容器，顶部对齐避免大高度弹窗上下溢出）、`.modal-body`（弹窗主体，`max-height: calc(100vh - 100px)` 整体上限 + flex 列布局）、`.modal-scroll`（内容滚动区，`flex-1 overflow-y-auto`，header/footer 不随内容滚动）
  - **`src/components/common/Modal.vue`**：外层容器改用 `.modal-shell`（修复此前 body 已用 `.modal-body`/`.modal-scroll` 但外层 shell 未替换的遗留）
  - **`src/components/common/ProfileSelectModal.vue`**、**`src/components/common/DeviceCodeModal.vue`**、**`src/components/common/SkinManager.vue`**、**`src/components/about/UpdateDialog.vue`**、**`src/components/online/LobbyJoinConfirmDialog.vue`**：外层/主体/内容区统一替换为 `.modal-shell`/`.modal-body`/`.modal-scroll`，移除各自硬编码的 `max-h-*` 与 `fixed inset-0` 样式
  - **`src/components/online/KickConfirmDialog.vue`**、**`src/views/tools/archive/ArchiveBackupDialog.vue`**、**`src/views/tools/data/LoadSaveModal.vue`**：同上改造，按钮栏独立为 footer（`bg-gray-50 rounded-b-lg`）不随内容滚动
  - **`src/views/version-settings/mod-tab/ModUpdateDialog.vue`**：外层改用 `.modal-shell`，主体改用 `.modal-body max-w-2xl`，内容区改用 `.modal-scroll`，移除原 `max-h-[85vh]` + `flex-1 overflow-y-auto` 重复实现
- 不改造项：`CrashDialog.vue`（自定义深色主题 `bg-dialog-bg`/`text-brand-*`，与 `.modal-body` 的 `bg-white` 冲突，且已有 `max-h-[85vh]` 高度限制）；`App.vue` 还原遮罩与 `DragOverlay.vue` 拖拽遮罩（非弹窗，全屏覆盖层）
- 验证：`npx vue-tsc --noEmit` 通过（exit 0）

#### 统一启动时存储迁移到 `migrations/` 模块

- 背景：项目有多个启动时自动执行的存储迁移逻辑散落在不同模块（`storage/appdata.rs` 的命名迁移与便携式迁移、`minecraft/online/storage.rs::OnlineStorage::load` 的 device.json 旧路径迁移、`storage/mod.rs::migrate_global_dirs` 的 online 残留目录清理）。把所有启动时存储迁移逻辑归到统一的 `migrations/` 目录，像数据库自动迁移那样启动时由 `Storage::init` 调用 `run_all()` 一次性执行
- 改动（4 新增 + 4 修改）：
  - **新增 `src-tauri/src/migrations/mod.rs`**：模块入口，提供 `run_all()` 按依赖顺序执行全部迁移（appdata_naming → portable_to_appdata → online_legacy）。`copy_dir_recursive`/`dir_is_non_empty` 迁移专用辅助函数提取为本模块 `pub(super)` 供子模块复用
  - **新增 `src-tauri/src/migrations/appdata_naming.rs`**：从 `storage/appdata.rs` 迁入 `migrate_legacy_appdata_root()` → `pub fn migrate()`，保留 Windows canonicalize 大小写不敏感比较逻辑
  - **新增 `src-tauri/src/migrations/portable_to_appdata.rs`**：从 `storage/appdata.rs` 迁入 `migrate_from_portable()` → `pub fn migrate()`，内部对 certs/providers 两个子目录执行便携式→AppData 迁移
  - **新增 `src-tauri/src/migrations/online_legacy.rs`**：从 `OnlineStorage::load` 提取 device.json 旧路径迁移逻辑 → `pub fn migrate()`，原样转写文件（不涉及 SDK 解密/加密），迁移后清理整个旧 `online/` 目录（合并原 `migrate_global_dirs` 的 online 残留清理）。`legacy_device_path()` 一并迁入为 `pub(crate) fn`，供 `OnlineStorage::save/clear` 复用
  - **`src-tauri/src/lib.rs`**：新增 `pub mod migrations;`（按字母序插入 logger 与 minecraft 之间）
  - **`src-tauri/src/storage/mod.rs`**：`migrate_global_dirs()` 简化为仅调用 `crate::migrations::run_all()`，删除内联迁移逻辑与不再使用的 `log_warn` 导入
  - **`src-tauri/src/storage/appdata.rs`**：删除 `migrate_legacy_appdata_root`/`migrate_from_portable`/`copy_dir_recursive`/`dir_is_non_empty`（已迁入 migrations），仅保留 `appdata_root`/`appdata_subdir`/`ensure_appdata_subdir` 路径解析函数与 `log_info` 导入
  - **`src-tauri/src/minecraft/online/storage.rs`**：`OnlineStorage::load` 删除 legacy_path 迁移分支（启动时已由 migrations 执行）；`save`/`clear` 的 `Self::legacy_device_path()` 改为调用 `crate::migrations::online_legacy::legacy_device_path()`；删除 `legacy_device_path()` 方法与 `DEVICE_FILE` 常量（已迁入 migrations）
- 设计决策：
  - `legacy_device_path()` 迁移到 `migrations/online_legacy.rs` 而非保留在 `online/storage.rs`（"更内聚"方案）：依赖方向为业务→基础设施（migrations），避免 migrations 反向依赖业务模块 `OnlineStorage`，单一真源保证 save/clear 与启动迁移路径口径一致
  - 辅助函数 `copy_dir_recursive`/`dir_is_non_empty` 放 `migrations/mod.rs` 作为 `pub(super)`，与 `commands/frp/install.rs`、`commands/plugins/install.rs` 各自的本地副本解耦（不跨模块共享）
  - `online_legacy::migrate` 合并了原 `OnlineStorage::load` 的文件迁移与 `migrate_global_dirs` 的 online 目录清理：device.json 存在则迁移后清理目录，不存在则清理残留目录，迁移失败保留旧目录（数据保护）
- 已知遗留：`certs.rs` 模块文档注释仍引用 `storage::appdata::migrate_from_portable`（已迁移到 `migrations::portable_to_appdata`），按任务约束未改动业务文件，注释为已知陈旧
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过（exit 0，7.16s）

#### 认证存储回归双轨制：Windows 注册表 + 非 Windows 结构化逐字段加密文件

- 背景：上一版（cbaaff1）把认证存储从 Windows 注册表统一改为跨平台整体加密文件存储（整个 `PersistedAuthState` 序列化为 JSON 后用 SDK DES 加密成一个密文字符串写入单文件）。用户不满意：Windows 老用户希望回到注册表逐字段存储，非 Windows 希望文件存储但结构化逐字段加密（而非整体加密一个 JSON 字符串）
- 改动（4 个文件 + 1 新增 + 1 删除）：
  - **新增 `src-tauri/src/minecraft/auth/storage/registry.rs`**：从 cbaaff1^ 恢复认证专用注册表键名常量（`KEY_LOGIN_TYPE`/`KEY_MS_CURRENT_*`/`KEY_AUTHLIB_CURRENT_*`/`ALL_KEYS` 等），低层 reg_* 操作复用 `crate::storage::registry`
  - **`src-tauri/src/minecraft/auth/storage/mod.rs`**：
    - 新增 `mod registry;`、删除 `mod migrate;`
    - 删除 `storage_path()` 方法与 `AUTH_FILE` 常量（Windows 用注册表、非 Windows 文件路径在 load/save 内部解析）
    - 恢复 `reg_get_decrypted`/`reg_set_encrypted` 辅助方法（`#[cfg(windows)]`，内部 inline `use` 避免非 Windows 未使用导入告警）
    - 保留 `encrypt`/`decrypt`/`invalidate`/`restrict_file_permissions` 不变
    - 更新模块顶部 doc 注释说明双轨制存储
  - **`src-tauri/src/minecraft/auth/storage/save.rs`**：双轨制实现
    - Windows：恢复 cbaaff1^ 注册表实现（`save_to_registry`），先清旧值再逐字段写入，敏感字段 SDK 加密
    - 非 Windows：新增 `save_to_file`，写入 `%APPDATA%/.Molaunch/auth.json`（macOS/Linux `~/.config/Molaunch/auth.json`）。明文 JSON 结构中 `name`/`uuid`/`access_token`/`client_token` 等敏感字段单独 SDK 加密为字符串值，`login_type` 明文；可空字段（profile_json/refresh_token/expires_at/server_url/server_name）Some 加密 / None 存 null；多账号列表先序列化为 JSON 字符串再 SDK 加密；Unix 设置 0o600 权限；写完刷新缓存
  - **`src-tauri/src/minecraft/auth/storage/load.rs`**：双轨制实现
    - Windows：恢复 cbaaff1^ 注册表实现（`load_from_registry`），按 LoginType 分支逐字段 SDK 解密读取
    - 非 Windows：新增 `load_from_file`，读取 auth.json → `serde_json::Value` → 逐字段 SDK 解密。current_user 统一读取全部字段（与 save 的 uniform 结构对应）；多账号列表 SDK 解密后反序列化；文件不存在返回 default；解析失败返回 Err。新增 `decrypt_field`/`decrypt_opt_field`/`decrypt_account_list` 三个非 Windows 辅助方法消除重复
    - 保留内存缓存优先逻辑
  - **删除 `src-tauri/src/minecraft/auth/storage/migrate.rs`**：Windows 用注册表、非 Windows 用文件，无需跨平台迁移
- 设计决策：
  - 非 Windows current_user 存储全部字段（uniform 结构），不同于注册表按 login_type 分字段（Legacy/Microsoft/AuthlibInjector 各存不同键）。文件结构稳定，新增字段只需扩 JSON 对象，不破坏旧数据
  - 非 Windows 加密失败用 `?` 传播错误（与历史注册表行为一致），不再降级明文存储（避免 token 落盘为明文的安全风险）
  - `operations.rs`/`types.rs` 未修改（高层操作仅依赖 `load`/`save`，与存储细节解耦）
- 验证：`cargo check` + `cargo clippy -- -D warnings` 均通过（0 警告）
- 影响范围：Windows 老用户回到注册表存储体验；非 Windows 用户得到结构化加密文件（每个敏感字段独立密文，便于审计与未来字段级迁移）

#### 统一 AppData 目录命名 + wintun.dll 路径复用 appdata 模块

- 背景：排查认证存储问题时发现 AppData 目录命名不一致（`personalization.rs`/`online/storage.rs` 误用 `.MolaLaunch`，便携式目录与 updater last.exe 用 `.Molaunch`），macOS/Linux 区分大小写会变成两个独立目录；同时 `resources.rs::extract_wintun` 独立拼路径而非复用 `appdata::appdata_root`，与 certs/providers/auth 等保持一致目录约定的口径不符
- 改动（7 个文件）：
  - **`src-tauri/src/storage/appdata.rs`**：`appdata_root()` 返回值统一为 `.Molaunch`（Windows `%APPDATA%/.Molaunch/`，macOS/Linux `~/.config/Molaunch/`），与便携式目录、updater last.exe 一致。旧路径 `.MolaLaunch` 由 `crate::migrations::appdata_naming` 启动时一次性迁移
  - **`src-tauri/src/resources.rs`**：`extract_wintun` 复用 `appdata::appdata_root()` 而非独立拼 `APPDATA` 环境变量与 `.MolaLaunch` 字面量，wintun.dll 释放路径从 `.MolaLaunch` 统一为 `.Molaunch`
  - **`src-tauri/src/commands/plugins/personalization.rs`**：`personalization_path()` 复用 `crate::storage::appdata::appdata_root` 而非内联平台分支拼路径，与 online/auth/certs/providers 等全局共享资源保持一致的目录约定（同时消除 `.MolaLaunch` 误用）
  - **`src-tauri/src/certs.rs` + `commands/frp/paths.rs` + `minecraft/online/bridge.rs`**：文档注释中的 `.MolaLaunch` 统一更新为 `.Molaunch`
  - **`src/stores/plugins.ts` + `src/utils/pluginInstaller.ts`**：前端文档注释中的 `.MolaLaunch` 统一更新为 `.Molaunch`（5 处）
- 命名历史：早期 `personalization.rs` 与 `online/storage.rs` 误用 `.MolaLaunch`（多了一个 La），后续 `auth/storage` 跟随。updater 的 last.exe 一直用 `.Molaunch`。现全部统一为 `.Molaunch`
- 验证：`cargo check` + `cargo clippy -D warnings` 均通过
- 影响范围：
  1. **目录命名统一**：AppData 下不再出现 `.Molaunch` 和 `.MolaLaunch` 两个目录（macOS/Linux 区分大小写会变两个），旧 `.MolaLaunch` 启动时由 `migrations::appdata_naming` 自动迁移到 `.Molaunch`
  2. **wintun.dll 路径统一**：复用公共 appdata 模块，与 certs/providers/auth 等保持一致
  3. **personalization 路径复用**：消除内联平台分支，与全局共享资源口径一致

#### 目录存储逻辑调整：certs/providers 迁移到 AppData 全局共享

- 背景：用户发现 `.Molaunch/` 便携式目录下存在 `certs/`、`online/`、`providers/` 三个本应全局共享的目录。这些是设备级资源（一份 TLS 证书、一份 frpc 二进制即可被所有启动器实例复用），但原来每个启动器实例各存一份，浪费磁盘空间且管理混乱
- 改动（5 个文件）：
  - **新增 `src-tauri/src/storage/appdata.rs`**：公共 AppData 路径辅助模块，集中管理 `%APPDATA%/.Molaunch/`（Windows）/ `~/.config/Molaunch/`（macOS/Linux）路径。提供 `appdata_root` / `appdata_subdir` / `ensure_appdata_subdir` / `migrate_from_portable` 四个函数。原本 `OnlineStorage::appdata_device_path` 与 `AuthStorage::storage_path` 各自重复实现同一套平台路径逻辑，现统一抽取到此模块
  - **`src-tauri/src/storage/mod.rs`**：`Storage::init` 新增 `migrate_global_dirs` 步骤，启动时自动：
    1. `certs` 从便携式迁移到 AppData（用户全局信任一次，多启动器共享）
    2. `providers` 从便携式迁移到 AppData（frpc 二进制全局共享，避免每实例重复下载几十 MB）
    3. 清理 `online` 残留目录（device.json 已在 v2 迁至 AppData，旧目录遗留需清理）
    - 迁移策略：AppData 已有数据则跳过并删除便携式旧目录；便携式目录递归复制到 AppData 后删除原目录；失败不阻塞启动，下次启动再次尝试
  - **`src-tauri/src/certs.rs`**：`cert_dir()` 改为返回 `%APPDATA%/.Molaunch/certs/`，复用 `appdata::ensure_appdata_subdir`；APPDATA 环境变量缺失时降级回便携式目录（极少发生）
  - **`src-tauri/src/commands/frp/paths.rs`**：`providers_root()` 改为返回 `%APPDATA%/.Molaunch/providers/`，同样复用 `appdata::ensure_appdata_subdir`；模块顶部 doc 注释区分便携式路径（frp/tunnels.json 等）与全局共享路径（providers/）
  - **`src-tauri/src/minecraft/online/storage.rs` + `src-tauri/src/minecraft/auth/storage/mod.rs`**：`appdata_device_path` 与 `storage_path` 内部平台分支逻辑替换为复用 `crate::storage::appdata::appdata_root`/`appdata_subdir`，消除重复实现，与 certs/providers 保持一致目录约定
- 保留便携式：`config.ini`、`instance.ini`、`logs/`、`cache/`、`temp/`、`Download/`、`frp/`（tunnels.json/providers.json/logs/config）仍是当前启动器实例绑定的运行时数据，保持便携式存储
- 验证：`cargo check` 通过
- 影响范围：多启动器实例不再重复存储证书和 frpc 二进制；旧用户首次启动新版本自动无缝迁移，数据不丢失；迁移后旧目录被清理，`.Molaunch` 目录更清爽

#### 修复 frpc 下载后 ZIP 文件未清理（2 处）

- 背景：用户发现 `.Molaunch/providers/` 下残留下载的 zip 文件，提取 frpc 后未删除
- 改动（2 个文件）：
  - **`src-tauri/src/commands/frp/binary/system_default.rs`**：系统默认厂商 frpc 下载流程，注释承诺"提取 frpc 后删除"但代码遗漏。在 `extract_frpc_from_zip` 成功后加 `std::fs::remove_file(&zip_path)` 清理临时 ZIP
  - **`src-tauri/src/commands/frp/binary/external.rs`**：外部厂商 frpc 下载流程，`dl.archive == true` 时解压后未删除原始 archive。在 `extract_archive` 成功后加 `std::fs::remove_file(&target_path)` 清理；返回信息区分 archive/非 archive 模式
- 验证：`cargo clippy -- -D warnings` 0 警告
- 影响范围：providers 目录不再残留冗余 zip 文件，节省磁盘空间

#### 跨平台兼容性 M5/M6/M7 评估结论（保持现状 + 补注释）

- 背景：P0+P1+P2+P3 全部清零后，评估剩余 3 项中低问题（M5/M6/M7），结论为保持现状并补充注释说明设计意图
- 改动（2 个文件，仅注释）：
  - **M5 `src-tauri/src/commands/tools/memory.rs`**：Linux/macOS 分支补注释说明 `mode` 参数被有意忽略（light/strong 在 Windows 区分 StandbyList，Unix 无等价概念，强行映射引入伪区分）
  - **M6 `src-tauri/src/minecraft/image_cache.rs`**：`cache_image_url` 补 doc 注释说明 `cfg(not(windows))` 实际仅覆盖 macOS/Linux（Android 非支持目标），未来支持 Android 时需改显式 `cfg(any(macos, linux))`
- **M7 `reveal_in_file_manager` Linux 降级**：保持现状不修复。Linux 文件管理器 `--select` 语法碎片化，保守回退打开父目录是合理妥协；改进需引入 dbus `org.freedesktop.FileManager1.ShowItems` 依赖，兼容性不确定
- 验证：`cargo clippy -- -D warnings` 0 警告
- 影响范围：无功能改动，仅注释清晰化

#### 修复跨平台兼容性 P3 文档/注释漂移（L1+L2+L4，L3 验证为误报）

- 背景：P0/P1/P2 全部清零后，处理 P3 低优先级文档/注释漂移问题。完整扫描报告见 `docs/CROSS_PLATFORM_COMPATIBILITY.md`
- 改动（3 处修复，2 个文件）：
  - **L1 `src-tauri/src/minecraft/launch/pipeline/pre_launch.rs:8`**：`run_pre_launch` doc 注释"语法同 Windows cmd"与实际实现不符（非 Windows 用 `sh -c`）
    - 改为：`Windows: \`cmd /C\`，Unix: \`sh -c\`，不等待退出，失败仅记录日志`
  - **L2 `src-tauri/src/minecraft/launch/pipeline/pre_launch.rs::validate_pre_launch_cmd`**：检测关键词偏 Windows（powershell/iex/invoke-），缺 sh 注入向量
    - 补充命令分隔符 `;`（sh 标准分隔符，cmd 也支持）
    - 补充 sh 注入关键词：`eval` / `exec` / `source`（sh 内置命令，常用于注入链）
    - 补充 doc 注释说明检测向量覆盖 Windows cmd 与 Unix sh 两种后端
  - **L4 `src-tauri/src/minecraft/launch/pipeline/process_spawn.rs:35`**：`cmd.env("appdata", &args.game_dir)` 在所有平台设置，macOS/Linux 上 appdata 是无意义环境变量
    - 改为：`#[cfg(target_os = "windows")]` 包裹，仅 Windows 设置（appdata 是 Windows 特有约定，Unix Mod 遵循 XDG）
- **L3 `process_spawn.rs:40-43` 验证为误报**：报告称未显式 `use std::os::windows::process::CommandExt`，实际 `tokio::process::Command` 在 Windows 上有 inherent `creation_flags` 方法（不通过 std CommandExt trait），`cargo check` 0 错误已验证无需修改
- 验证：`cargo clippy -- -D warnings` 0 警告
- 影响范围：PreLaunch 安全检测覆盖 sh 注入向量；非 Windows 不再注入冗余 appdata 环境变量；文档注释与实现一致

#### 修复跨平台兼容性 P2 中等问题（4 项连续修复）

- 背景：P0 阻塞项与 P1 体验项清零后，继续处理 P2 中等问题，覆盖 updater stub 语义、frp 进程组管理、SDK 平台覆盖、注册表 bool 语义模糊。完整扫描报告见 `docs/CROSS_PLATFORM_COMPATIBILITY.md`
- 改动（4 处修复，7 个文件）：
  - **M1 `src-tauri/src/commands/system/updater/mod.rs`**：非 Windows stub 明确化
    - `download_update_to_appdata` 与 `apply_pending_update` 在非 Windows 分支静默 `Ok(false)`，无任何日志提示，调用方无法区分"无更新"与"平台不支持"
    - 改为：两个 stub 分支均加 `log_info!` 提示"由 tauri-plugin-updater 接管"，并补充 doc 注释说明平台差异与前端应使用的命令
  - **M2 `src-tauri/src/commands/frp/process/start.rs` + `stop.rs`**：非 Windows 进程组管理
    - 问题：Windows 通过 Job Object（`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`）实现启动器退出时自动清理 frpc，非 Windows 无对应机制，stop_tunnel 仅靠 `kill_process_tree` 的 ps 递归查询，可能漏掉短命子进程
    - 改造：start.rs 在 `cmd.spawn()` 前用 `pre_exec(setpgid(0, 0))` 让 frpc 成为新进程组 leader（PGID = frpc PID）；stop.rs 非 Windows 改用 `libc::killpg(pgid, SIGTERM)` 一次性杀整个进程组（含 frpc 派生子进程），ESRCH 不算错误
    - 新增依赖：`libc = "0.2"` 加入 `[target.'cfg(unix)'.dependencies]`（仅 macOS/Linux 引入，Windows 不受影响）
    - 语义：SIGTERM 而非 SIGKILL，给 frpc 优雅退出（关闭连接、刷新日志）的机会
  - **M3 `src-tauri/src/sdk/mod.rs`**：SDK 平台覆盖加 fallback
    - 问题：`get_sdk_filename()` 用 `#[cfg(target_os = "...")]` 仅覆盖三个组合，未匹配的 Intel Mac / Linux aarch64 / FreeBSD 等平台会编译失败（`#[cfg]` 分支不完整导致函数无返回值）
    - 改造：精确匹配 `#[cfg(all(target_os, target_arch))]`，并加 `#[cfg(not(any(...)))]` fallback 返回 `"unsupported-platform"`，`check_sdk_library()` 在 `extract_sdk()` 时因嵌入资源不存在返回明确错误，避免编译失败但运行时无法加载
    - 新增平台支持时需同步：1) 编译 SDK 产物；2) 加入 resources/sdk/；3) 添加对应 `#[cfg]` 分支
  - **M4 `src-tauri/src/storage/registry.rs` + `developer.rs` + `apply_config/secure.rs`**：`reg_get_bool` 语义模糊
    - 问题：`reg_get_bool` 在非 Windows 上固定返回 `false`，调用方无法区分"值实际为 false"与"平台不支持读取"
    - 改造：返回类型 `bool → Option<bool>`，Windows 端键不存在返回 `None`、值为 false 返回 `Some(false)`、值为 true 返回 `Some(true)`；非 Windows stub 返回 `None`（平台不支持）
    - 调用方（6 处）：developer.rs 3 处 + secure.rs 3 处，全部加 `.unwrap_or(false)` 保持原"键不存在视为 false"语义，但日志/调试时可 `match` 区分"不支持"与"false"
- 验证：`cargo check` 0 错误，`cargo clippy` 0 警告（Windows 平台；Unix 分支为标准 POSIX 调用，待 macOS/Linux CI 验证）
- 影响范围：macOS/Linux 上 frpc 子进程清理更可靠（killpg 替代 ps 递归查询）；未支持平台 SDK 加载失败时给出明确错误而非编译失败；注册表 bool 语义清晰，便于后续跨平台扩展

#### 修复跨平台兼容性 P1 体验项：auth/storage 凭据持久化改文件存储（S4+S5）

- 背景：`auth/storage` 系列文件围绕 Windows 注册表设计，非 Windows 平台 `save` 静默 `Ok(())` 不持久化、`load` 返回 `default()` 空状态，导致 macOS/Linux 上登录后重启启动器会丢失所有登录信息。完整扫描报告见 `docs/CROSS_PLATFORM_COMPATIBILITY.md`
- 改造方案（参考 `minecraft/online/storage.rs` 的跨平台文件存储实现）：保留 SDK DES 加密层，存储介质从注册表改为单文件
  - 存储路径：Windows `%APPDATA%/.MolaLaunch/auth.json`，macOS/Linux `~/.config/MolaLaunch/auth.json`（与 `online/storage.rs` 目录约定一致）
  - 序列化策略：整个 `PersistedAuthState` 通过 `to_storage_json()` 手动构建 JSON（避免派生 `Serialize` 误暴露 token 到 IPC）→ SDK DES 加密 → 写入单文件
  - 权限保护：Unix 显式设置 0o600（仅当前用户可读写），Windows 依赖 NTFS 默认 ACL
  - 容错：SDK 不可用时降级明文存储（带 WARN）；环境变量缺失/文件不存在返回空状态而非报错
- 改动（5 文件）：
  - **`src-tauri/src/minecraft/auth/storage/types.rs`**：新增 `CurrentUser::to_storage_json()` 与 `PersistedAuthState::to_storage_json()` 方法（手动构建 JSON，含全部敏感字段）
  - **`src-tauri/src/minecraft/auth/storage/mod.rs`**：删除 `reg_set_encrypted`/`reg_get_decrypted`（Windows 专有方法）；新增 `storage_path()` 函数（跨平台路径解析）；新增 `restrict_file_permissions()` Unix 权限辅助函数；删除 `mod registry` 声明
  - **`src-tauri/src/minecraft/auth/storage/save.rs`**：重写为文件存储（序列化 → 加密 → 写文件 → Unix 设 0o600 → 刷新缓存）
  - **`src-tauri/src/minecraft/auth/storage/load.rs`**：重写为文件存储（读文件 → 解密 → 反序列化；文件不存在/环境变量缺失返回空状态）
  - **`src-tauri/src/minecraft/auth/storage/registry.rs`**：删除（键名常量 + `ALL_KEYS` 不再需要）
  - **`src-tauri/src/storage/registry.rs`**：`reg_delete` 加 `#[allow(dead_code)]`（auth/storage 改文件存储后无调用方，保留以备 crate 级复用）
- 未做迁移：Windows 老用户升级后需要重新登录（beta 阶段允许，未做注册表→文件一次性迁移）
- 验证：`cargo check` 0 错误，`cargo clippy` 0 警告
- 影响范围：所有平台统一行为（macOS/Linux 登录态可持久化；Windows 从注册表迁移到文件，老用户需重新登录一次）

#### 修复跨平台兼容性 P0 阻塞项（macOS/Linux release 准备）

- 背景：扫描后端 Windows 专有逻辑时发现 4 处 P0 阻塞问题，会导致 macOS/Linux 上游戏无法启动或进程树无法正确清理。完整扫描报告见 `docs/CROSS_PLATFORM_COMPATIBILITY.md`
- 改动（4 处修复，3 个文件）：
  - **S1 + S2 `src-tauri/src/minecraft/launch/jvm_args.rs`**：JVM 参数 classpath 分隔符与库目录路径分隔符硬编码 Windows 风格，导致 macOS/Linux 上 JVM 无法解析 classpath 与库路径
    - 行 184：删除 `.replace('/', "\\")`，保留 `PathBuf` 原生分隔符（Windows `\` / Unix `/`，JVM 全平台均接受原生分隔符）
    - 行 223：`${classpath_separator}` 替换值由硬编码 `";"` 改为 `if cfg!(target_os = "windows") { ";" } else { ":" }`（Windows `;` / Unix `:`）
  - **S3 `src-tauri/src/minecraft/java/download/files.rs`**：`find_java_exe` 硬编码 `"java.exe"` 与 `"windows-x64"` 子目录，macOS/Linux 上永远找不到 Java
    - 可执行文件名：`if cfg!(target_os = "windows") { "java.exe" } else { "java" }`
    - 候选子目录：Windows `windows-x64` / macOS `mac-os`（Mojang 官方 manifest 命名）/ Linux `linux`
    - 递归查找兜底 `find_recursive` 改为接收 `exe_name` 参数（原为闭包内硬编码，无法跨平台）
  - **S6 `src-tauri/src/minecraft/system/shell/exec.rs`**：`kill_process_tree` Unix 分支仅 `kill -9 <pid>`，不杀子进程，与函数名"树"语义不符
    - 改为 `ps -A -o pid= -o ppid=` 一次性获取所有进程的父子关系（POSIX 标准，Linux/macOS 通用）
    - 递归收集所有后代 PID（含 pid 自身）后批量 `kill -9`，先杀子进程避免 reparent 到 init
    - 增加调试日志：`log_debug!` 输出收集到的进程数量
- 验证：`cargo check` 0 错误，`cargo clippy` 0 警告
- 影响范围：仅 macOS/Linux 行为变化（修复启动失败与孤儿进程），Windows 行为不变

#### 修正 V3 拆分记录中 4 处文件名错误

- 背景：核对 V3 报告 P1 黄区+边界拆分记录时发现 4 处文件名与实际目录不符（拆分落地时记录有误，代码本身正确）
- 改动（纯文档纠错，未改任何代码）：
  - `minecraft/system/shell/`：`reveal.rs` → `perms.rs`、`open_url.rs` → `open.rs`
  - `commands/tools/archive/`：`common.rs` → `helpers.rs`、补 `seed.rs`
  - `commands/tools/network/`：`port.rs` → `ports.rs`、`server_ping.rs` → `ping.rs`、`motd.rs` → `tcp.rs`
  - `commands/version/list/`：`scan.rs`/`filter.rs`/`sort.rs`/`meta.rs` → `detect.rs`/`installed.rs`/`remote.rs`/`modpack.rs`/`info.rs`
- 红区 6 个拆分记录核对全部正确，未改动

#### 升级 vue-tsc 1.8.27 → 2.2.12 并修复 29 处 Vue 模板类型错误

- 背景：`vue-tsc@1.8.27` 与 Node.js v24 不兼容（`Search string not found: /supportedTSExtensions/`），`npm run typecheck` 无法运行。升级到 2.2.12 后 2.x 严格模式暴露 29 处此前未检出的 Vue SFC 类型错误
- 升级：`package.json` devDependencies `vue-tsc` ^1.8.27 → ^2.2.12（typescript ^5.3.3 满足 2.x 要求）；`src-tauri/resources/about/frontend-dev-deps.txt` 同步更新版本号
- 验证：`npm run typecheck` 0 错误
- 修复（均为类型层/未使用变量清理，未改运行时行为，按错误类型分组）：
  - **TS6133 未使用变量（8 处，5 文件）**：
    - `src/components/common/SubTabBar.vue` / `src/views/version-settings/mod-tab/VersionTable.vue`：`defineProps` 返回值未使用，去掉 `const props =` 接收
    - `src/views/settings/settings-launch/JavaPathSelector.vue`：删除未使用 `import * as tauri`；删除 `#option` slot 解构中未使用的 `selected`
    - `src/views/Versions.vue`：删除 `useVersionInstallActions()` 解构中未使用的 `loadInstalledVersions`
    - `src/views/version-settings/ModTab.vue`：删除解构中未使用的 `selectedIds`、`hasSelection`
    - `src/components/home/AccountSelector.vue`：删除 `useSwipeNavigation` 解构中未使用的 `isAnimating`
  - **Java 路径选择相关（7 处，4 文件）**：
    - `src/views/version-settings/setup-tab/JavaCustomMode.vue`：`java_path` → `javaPath`（对齐 `VersionPersonalization` 驼峰字段名，2 处 TS2551）；`handleImportJava` 加 `if (!selectedId.value) return` null 守卫（1 处 TS2345）；`@update:model-value` handler 参数 `string` → `string | number`（对齐 `Select.vue` emit 声明，1 处 TS2322）
    - `src/views/version-settings/SetupTab.vue` / `src/views/version-settings/setup-tab/JavaModeSelector.vue`：同上，handler 参数 `string` → `string | number`（2 处 TS2322）
    - `src/views/version-settings/MemorySection.vue`：加 `if (!selectedId.value) return` null 守卫（1 处 TS2345）
  - **导出/重载/转换/参数数量（8 处，8 文件）**：
    - `src/components/home/AccountSelector.vue` / `src/components/home/account-selector/AccountIndicator.vue`：`AccountCardData` 改从 `./types` 导入（`AccountCard.vue` 未导出该类型，仅转导）
    - `src/components/common/skin-manager/SkinPreviewPanel.vue`：`skinUrl` → `skinUrl ?? undefined`（`SkinAvatar` prop 期望 `string | undefined`）
    - `src/components/common/SkinModel3D.vue`：`loadCape` 拆分 `if (newUrl) loadCape(newUrl) else loadCape(null)` 命中重载
    - `src/components/community/SearchBar.vue`：动态事件名 `emit(... as any)` 改为 `switch` 分发 + 字面量事件名窄化
    - `src/plugins/custom-layout/HtmlLayoutPanel.vue`：`window as Record<...>` → `window as unknown as Record<...>`
    - `src/components/community/DependencyConfirmDialog.vue`：`Button` type `"default"` → `"text"`（项目 Button 不支持 default）
    - `src/components/common/CrashDialog.vue`：`toastSuccess`/`toastError` 双参合并为单字符串
  - **App/事件/状态/LayoutSection（6 处，4 文件 + 2 个 .ts 根因）**：
    - `src/utils/toast.ts`：`setToastRef` 入参 `ToastRef` → `ToastRef | null`（与 `setModalRef`/`setCrashDialogRef` 一致，修复 `App.vue:51`）
    - `src/composables/useModUpdate.ts`：`emit` 参数改为与 `defineEmits` 一致的重载交集类型（修复 `ModUpdateDialog.vue:53`）
    - `src/plugins/custom-layout/LayoutSectionRenderer.vue`：html section `:ref` 回调补 `props.section.type === 'html'` 守卫窄化闭包内类型
    - `src/views/quick-tools/CleanupTool.vue`：移除 `v-if` 窄化域内恒为 false 的 `scanState === 'cleaning'`/`'scanning'` 死比较（状态联合已含全部成员，问题为模板控制流窄化）

#### 验证预存 5 个 tsc 错误已在历次 pass 中修复

- 背景：V3 报告记录 5 个预存 tsc 错误（`htmlShadowRenderer.ts`/`renderHelpers.ts`(3，CustomLayoutPanel 拆分引入) + `crypto.ts`(1，TS 5.7+ ArrayBuffer 严格模式)），需确认当前状态
- 验证方式：`npx tsc --noEmit`（项目本地 typescript + strict tsconfig，include 覆盖 `src/**/*.ts`）
- 结果：0 错误，5 个预存错误已在历次 pass 中修复
- 修复痕迹（已在之前的拆分/重构 pass 中落地，本次未改动代码）：
  - `src/utils/online/crypto.ts`：`raw as BufferSource` 显式转换（TS 5.7+ ArrayBuffer 严格模式）
  - `src/plugins/custom-layout/renderHelpers.ts`：`typeof raw !== 'string' && typeof raw !== 'number'` 守卫窄化 `unknown`
  - `src/plugins/custom-layout/htmlShadowRenderer.ts`：`const script = section.script` 提取局部变量解决闭包内 TS 无法窄化问题
- 环境备注：此前 `vue-tsc@1.8.27` 与 Node.js v24 不兼容导致无法运行 typecheck，现已升级到 2.2.12（见上条），Vue 模板类型检查已通过 `npm run typecheck` 补验

#### 修复 clippy `map_identity` 冗余 identity 闭包（2 处）

- 背景：clippy 报告 2 处 `.map(|x| x)` 形式的恒等映射，属于无意义闭包调用
- 改动（删除冗余 `.map(...)`，未改业务逻辑）：
  - `src-tauri/src/commands/community/install/concurrent/extract.rs`：`find_map(|p| name.strip_prefix(p.as_str()).map(|r| r))` → `find_map(|p| name.strip_prefix(p.as_str()))`
  - `src-tauri/src/commands/version/export/options/mod.rs`：`sort_by_key` 中 `.ok().map(|t| t)` → `.ok()`
- 验证：`cargo clippy --lib` 0 警告

#### 修复 clippy `type_complexity` 引入类型别名（3 处）

- 背景：clippy 报告 3 处函数签名类型嵌套过深（`Arc<dyn Fn... + Send + Sync>` / `Arc<Mutex<Option<Arc<tokio::sync::Mutex<Option<Child>>>>>>` / 7 元组返回类型），可读性差
- 改动（在文件顶部引入 `type` 别名，函数签名替换为别名，未改运行时行为）：
  - `src-tauri/src/minecraft/download/full_download.rs`：`type StageCallback = Arc<dyn Fn(usize, &str) + Send + Sync>;`
  - `src-tauri/src/minecraft/launch/pipeline/mod.rs`：`type ChildProcessHandle = Arc<Mutex<Option<Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>>>>;`
  - `src-tauri/src/minecraft/version/scan/loaders.rs`：`type LoaderDetectResult = (VersionType, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>, Option<String>);`
- 验证：`cargo clippy --lib` 0 警告

#### 修复 clippy `inherent_to_string` 改为 `Display` trait 实现（1 处）

- 背景：clippy 报告 `IniFile` 自定义 `to_string` 方法会与 `std::string::ToString` trait（由 `Display` 自动派生）冲突，应直接实现 `Display`
- 改动（删除 inherent `to_string` 方法，改为 `impl std::fmt::Display for IniFile`，输出格式完全一致）：
  - `src-tauri/src/storage/ini.rs`：`pub fn to_string(&self) -> String` → `impl Display for IniFile { fn fmt(&self, f: &mut Formatter<'_>) -> Result { ... } }`
- 调用方仍可用 `ini.to_string()`（通过 `Display` 自动派生 `ToString`），无破坏性变更
- 验证：`cargo clippy --lib` 0 警告

#### 修复 clippy `upper_case_acronyms` 为 Windows API 类型添加 `#[allow]`（2 处）

- 背景：clippy 报告 `HWND`/`HINSTANCE` 类型名含全大写缩写违反命名规范，但这两个是 Windows 系统 API 的标准类型名（来自 Win32），改名会破坏可读性
- 改动（在 `type` 别名前添加 `#[allow(clippy::upper_case_acronyms)]` 属性，未改类型本身）：
  - `src-tauri/src/minecraft/system/shell/admin.rs`：`HWND`、`HINSTANCE`
  - `src-tauri/src/minecraft/system/shell/open.rs`：`HWND`、`HINSTANCE`
- 验证：`cargo clippy --lib` 0 警告

#### 修复 clippy `should_implement_trait` 为 `from_str` 添加 `#[allow]`（4 处）

- 背景：clippy 报告自定义 `from_str` 方法会与 `std::str::FromStr` trait 方法名冲突。这些 `from_str` 是项目早期为简化枚举解析而写的便利方法（返回枚举而非 `Result`），改为实现 `FromStr` trait 需重构所有调用点（返回 `Result` + `unwrap`），短期不重构
- 改动（在 `from_str` 方法前添加 `#[allow(clippy::should_implement_trait)]` 属性，未改方法签名/逻辑）：
  - `src-tauri/src/logger/mod.rs`：`LogLevel::from_str`
  - `src-tauri/src/minecraft/community/types.rs`：`ModLoaders::from_str`
  - `src-tauri/src/minecraft/sources/mode.rs`：`DownloadSourceMode::from_str`
  - `src-tauri/src/minecraft/version/state.rs`：`VersionType::from_str`
- 验证：`cargo clippy --lib` 0 警告

#### 为 clippy `too_many_arguments` 告警函数添加 `#[allow]` 属性标注

- 背景：clippy 报告 10 个函数参数过多（8-17 个），这些函数多为启动编排/下载/参数构建的核心入口，参数数量由业务需求决定，短期不重构签名
- 改动（纯属性标注，未修改任何函数体/签名/业务逻辑）：
  - `src-tauri/src/commands/version/install/loader_helpers.rs`：`install_single_loader`（9 参）
  - `src-tauri/src/commands/version/launch/build_config.rs`：`build_launch_config`（12 参）
  - `src-tauri/src/commands/version/launch/mod.rs`：`launch_game`（12 参）
  - `src-tauri/src/minecraft/download/chunk/mod.rs`：`download_chunked`（9 参）
  - `src-tauri/src/minecraft/download/downloader/single.rs`：`download_single`（10 参）
  - `src-tauri/src/minecraft/download/downloader/stream.rs`：`download_from_url`（9 参）
  - `src-tauri/src/minecraft/launch/arguments.rs`：`build_launch_arguments`（17 参）
  - `src-tauri/src/minecraft/launch/game_args.rs`：`build_game_args`（12 参）
  - `src-tauri/src/minecraft/launch/jvm_args.rs`：`build_jvm_args`（11 参）
  - `src-tauri/src/minecraft/version/setup/types.rs`：`VersionSetup::new`（8 参，impl 块内）
- 约定：`#[allow]` 统一放在 `///` doc comment 前一行
- 验证：`cargo check --lib` 通过（14.79s，无错误，仅 2 处预存 `PathBuf` 未使用 import 告警）

### 重构

#### 拆分超长 Rust 文件（V3 代码质量报告 P1 红区 6 个）

- 背景：V3 报告 P1 红区（>500 行硬约束违规）6 个文件未拆——`minecraft/auth/authlib/client.rs`(639)、`commands/version/export/zip.rs`(612)、`commands/frp/process.rs`(593)、`commands/auth/authlib.rs`(548)、`commands/tools/types.rs`(540)、`minecraft/online/client.rs`(530)。沿用 V3 已有拆分约定（`modpack/`、`signaling/`、`options/` 三种风格：`mod xxx;` + `pub use` 具名/glob re-export + 共享状态留 `mod.rs` 用 `pub(super)` 暴露）
- 改动（纯重构，仅搬运代码 + 调整 `use` 导入路径，未改任何业务逻辑/公开 API 签名/错误处理/日志内容）：
  - [src-tauri/src/minecraft/auth/authlib/client/](src-tauri/src/minecraft/auth/authlib/client/)：639 → 7 文件（`mod.rs`/`types.rs`/`meta.rs`/`auth.rs`/`profile.rs`/`skin.rs`/`cape.rs`），按 HTTP 端点域分组；mod.rs 具名 re-export 9 函数 + `AuthlibInjectorMeta`，并额外保留 `authenticate`/`validate`/`refresh`/`YggdrasilError`（`login.rs` 依赖）；`join_url`/`parse_error`/`delete_texture` 共享辅助提升到 mod.rs
  - [src-tauri/src/commands/version/export/zip/](src-tauri/src/commands/version/export/zip/)：612 → 8 文件（`mod.rs`/`helpers.rs`/`modrinth.rs`/`curseforge.rs`/`hmcl.rs`/`mmc.rs`/`mcbbs.rs`/`compress.rs`），按导出格式分组；`build_modpack_zip` 编排函数留 mod.rs 按格式分发，签名未变
  - [src-tauri/src/commands/frp/process/](src-tauri/src/commands/frp/process/)：593 → 6 文件（`mod.rs`/`start.rs`/`capture.rs`/`stop.rs`/`status.rs`/`log.rs`），按 frpc 生命周期分组；`FrpcHandle`/`RUNNING` 全局状态提升到 mod.rs 用 `pub(super)` 暴露；6 个公开函数 re-export
  - [src-tauri/src/commands/auth/authlib/](src-tauri/src/commands/auth/authlib/)：548 → 6 文件（`mod.rs`/`types.rs`/`helpers.rs`/`login.rs`/`account.rs`/`skin.rs`），按登录/账号/皮肤职责分组；mod.rs 具名 re-export 6 函数 + 4 类型（父 `commands/auth/mod.rs` 要求）+ 5 个皮肤命令（`meta_manager` 依赖）
  - [src-tauri/src/commands/tools/types/](src-tauri/src/commands/tools/types/)：540 → 15 文件，按 tools 子模块一一对应分组（download/archive/network/screenshot/mod_tools/resourcepack/cleanup/version_json/crash_analyzer/nbt/picker_window/data_export/memory/filename + mod.rs）；mod.rs `pub use xxx::*;` glob re-export 63 个 pub struct，保证父 `tools/mod.rs` 的 `use types::*;` 透明兼容
  - [src-tauri/src/minecraft/online/client/](src-tauri/src/minecraft/online/client/)：530 → 5 文件（`mod.rs`/`time.rs`/`jwks.rs`/`auth.rs`/`request.rs`），按联机 API 域分组；保留 `pub use super::client_types::{BusinessResult, ClientError};` re-export；测试文件 `client_tests.rs` 原地未动，`mod.rs` 末尾 `#[cfg(test)] #[path = "../client_tests.rs"] mod tests;` 修正相对路径指向上一级
- 所有新文件均 ≤350 行（最大 231 行），头部 `//!` 注释 ≤5 行
- 验证：`cargo check` 通过（19.22s 无错误无警告）；`cargo fmt` 通过；clippy 经核查本次拆分**未引入任何新告警**（仅 2 处预存告警随纯重构迁移：`zip/mod.rs` 的 `ptr_arg` 来自原 `build_modpack_zip` 签名、`client/meta.rs` 的 `needless_borrow` 来自原 `base64_encode(&json.as_bytes())` 调用，均非本次新增）

#### 拆分超长 Rust 文件（V3 代码质量报告 P1 黄区 + 边界 13 个）

- 背景：V3 报告 P1 剩余 13 个 350-500 行 Rust 文件未拆——黄区 5 个（`system/shell.rs` 494 / `tools/archive.rs` 493 / `frp/binary.rs` 477 / `tools/network.rs` 470 / `community/mcmod.rs` 443）+ 边界 8 个（`online/auth.rs` 429 / `sources.rs` 390 / `version/list.rs` 383 / `updater.rs` 368 / `frp/mod.rs` 363 / `auth/storage/mod.rs` 363 / `apply_config/types.rs` 353 / `curseforge/mod.rs` 351）。沿用红区拆分约定（`mod xxx;` + `pub use` 具名 re-export + 共享状态留 `mod.rs` 用 `pub(super)` 暴露）
- 改动（纯重构，仅搬运代码 + 调整 `use` 导入路径，未改任何业务逻辑/公开 API 签名/错误处理/日志内容）：
  - **批 1（黄区 5 + 边界 1，共 6 个）**：
    - [src-tauri/src/minecraft/system/shell/](src-tauri/src/minecraft/system/shell/)：494 → 子目录（`mod.rs`/`exec.rs`/`window.rs`/`admin.rs`/`open.rs`/`perms.rs`），按 shell 命令职责分组；`shell_err` 共享辅助提升到 mod.rs 用 `pub(super)` 暴露
    - [src-tauri/src/commands/tools/archive/](src-tauri/src/commands/tools/archive/)：493 → 子目录（`mod.rs`/`list.rs`/`backup.rs`/`restore.rs`/`helpers.rs`/`seed.rs`），按存档操作分组
    - [src-tauri/src/commands/frp/binary/](src-tauri/src/commands/frp/binary/)：477 → 子目录（`mod.rs`/`system_default.rs`/`external.rs`/`archive.rs`），按厂商类型分组
    - [src-tauri/src/commands/tools/network/](src-tauri/src/commands/tools/network/)：470 → 子目录（`mod.rs`/`latency.rs`/`ports.rs`/`ping.rs`/`tcp.rs`），按网络工具类型分组
    - [src-tauri/src/minecraft/community/mcmod/](src-tauri/src/minecraft/community/mcmod/)：443 → 子目录（`mod.rs`/`database.rs`/`lookup.rs`/`parsers.rs`/`search.rs`），按 mcmod 功能分组；测试文件 `mcmod_tests.rs` 路径修正为 `#[path = "../mcmod_tests.rs"]`
    - [src-tauri/src/commands/version/list/](src-tauri/src/commands/version/list/)：383 → 子目录（`mod.rs`/`detect.rs`/`installed.rs`/`remote.rs`/`modpack.rs`/`info.rs`），按版本列表处理阶段分组
  - **批 2（边界 5 个）**：
    - [src-tauri/src/minecraft/online/auth/](src-tauri/src/minecraft/online/auth/)：429 → 子目录（`mod.rs`/`helpers.rs`/`keypair.rs`/`login.rs`/`refresh.rs`/`register.rs`/`types.rs`），按认证协议流程分组；测试文件 `auth_tests.rs` 路径修正为 `#[path = "../auth_tests.rs"]`
    - [src-tauri/src/minecraft/sources/](src-tauri/src/minecraft/sources/)：390 → 子目录（`mod.rs`/`cdn.rs`/`constants.rs`/`http.rs`/`mode.rs`/`paths.rs`），按镜像源管理职责分组；测试文件 `sources_tests.rs` 路径修正为 `#[path = "../sources_tests.rs"]`
    - [src-tauri/src/commands/system/updater/](src-tauri/src/commands/system/updater/)：368 → 子目录（`mod.rs`/`check.rs`/`install_windows.rs`/`install_unix.rs`），按平台分流分组；`UpdateInfo` 类型保留 mod.rs，平台分支用 `#[cfg(target_os = "windows")]` 编译期分流
    - [src-tauri/src/commands/system/apply_config/types/](src-tauri/src/commands/system/apply_config/types/)：353 → 子目录（`mod.rs`/`patch.rs`/`snapshot.rs`），按 ConfigPatch / ConfigSnapshot / build_snapshot 职责分组；mod.rs 保留 `ConfigEntry` + `build_snapshot` 函数
  - **批 3（边界 3 个，本次完成）**：
    - [src-tauri/src/commands/frp/](src-tauri/src/commands/frp/)：399 → mod.rs 47 + [types.rs](src-tauri/src/commands/frp/types.rs) 293 + [paths.rs](src-tauri/src/commands/frp/paths.rs) 59；types 集中所有共享数据类型（隧道/厂商清单/认证配置/日志文件），paths 集中路径辅助函数与 ID 校验，mod.rs 保留 `frp_manager` 命令入口 + 子模块声明 + re-export
    - [src-tauri/src/minecraft/auth/storage/](src-tauri/src/minecraft/auth/storage/)：411 → mod.rs 87 + [load.rs](src-tauri/src/minecraft/auth/storage/load.rs) 176 + [save.rs](src-tauri/src/minecraft/auth/storage/save.rs) 140；`AuthStorage::load` 与 `AuthStorage::save` 两个大方法各拆为独立 impl 块文件，mod.rs 保留 struct 定义 + new + 加解密工具 + invalidate
    - [src-tauri/src/minecraft/community/curseforge/](src-tauri/src/minecraft/community/curseforge/)：402 → mod.rs 20 + [fingerprint.rs](src-tauri/src/minecraft/community/curseforge/fingerprint.rs) 203 + [search.rs](src-tauri/src/minecraft/community/curseforge/search.rs) 73 + [project.rs](src-tauri/src/minecraft/community/curseforge/project.rs) 93；按 6 个公共 API 函数 + 1 个 helper 的功能域分组（指纹查询 / 搜索 / 工程与版本）
- 所有新文件均 ≤350 行（最大 293 行），头部 `//!` 注释 ≤5 行
- 附带修复（apply_config/types 拆分引入的编译错误）：
  - [src-tauri/src/commands/system/apply_config/types/mod.rs](src-tauri/src/commands/system/apply_config/types/mod.rs)：`build_snapshot` 函数体引用 7 个 Snapshot 子结构体（`ProxySnapshot`/`DownloadSnapshot`/`MemorySnapshot`/`CommunitySnapshot`/`LaunchAdvancedSnapshot`/`OnlineSnapshot`/`TlsSnapshot`）但未导入，补 `use snapshot::{...};`
- 附带修复（shell 拆分遗留的 unix-only import 未加 cfg 守卫）：
  - [src-tauri/src/minecraft/system/shell/admin.rs](src-tauri/src/minecraft/system/shell/admin.rs)：`use super::shell_err;` 加 `#[cfg(unix)]` 守卫（仅在 macOS/Linux `relaunch_as_admin` 分支用）
  - [src-tauri/src/minecraft/system/shell/window.rs](src-tauri/src/minecraft/system/shell/window.rs)：`use crate::log_info;` 与 `use super::shell_err;` 加 `#[cfg(unix)]` 守卫（所有窗口管理函数都是 unix-only）
- 验证：`cargo check` 通过（无错误无警告）；`cargo fmt` 通过；`cargo clippy --lib --no-deps` 0 错 0 警告（拆分本身未引入任何新告警）

#### 注释规范整改（V3 代码质量报告 P2）

- 背景：V3 报告 P2 违规——后端 .rs 文件头部 `//!` 注释超过 5 行（版权声明除外）、多处 `///` 文档注释超过 10 行、6 处无用注释（主要为代码复述/陈旧型），违反"头部注释 ≤5 行、方法注释不得浮夸、无用注释必须移除"硬约束
- 改动（仅删减/精简注释文字，未改动任何代码逻辑）：
  - **头部 `//!` 注释**：85 个后端 .rs 文件头部注释精简至 ≤5 行，保留模块用途、关键设计与安全要点，去除冗余实现细节
  - **`///` 文档注释**：全部超 10 行的文档注释精简至 ≤10 行（grep 复核无残留），涉及 http.rs / version/libraries/filter.rs / image_cache.rs / commands/system/mod.rs / auth/authlib/types.rs / community/modrinth/mod.rs / loaders/forge_html.rs / online/protocol.rs / state/config.rs / community/curseforge/mod.rs / java_selector/rules.rs / community/fuzzy.rs / community/mcmod.rs / download/assets.rs / version/preload.rs / version/mods/update.rs / system/shell.rs / commands/java.rs / online/crypto.rs / frp/provider.rs / system/developer.rs / community/install/helpers.rs / tools/crash_analyzer.rs / community/install/mmc.rs / frp/log_redact.rs 等
  - **无用注释清理**：移除 6 处无用/陈旧注释——minecraft/online/bridge.rs 3 处（描述 write_tx 关闭但实际用 `handle.abort()` 的陈旧注释 + 复述 abort 调用）、commands/frp/process.rs 1 处（描述未实现分支的陈旧注释）、logger/mod.rs 1 处（复述 `file.write_all` 的 "// 写入文件"）、lib.rs 1 处（误导性 "需要获取 AppState" 注释，实际仅打印日志）
- 验证：`cargo check` 通过无错误无警告（26s）；仅删减注释文字，未改动任何代码逻辑

#### 拆分超长 TS 文件（V3 代码质量报告 P2）

- 背景：V3 报告 P2 违规——7 个 TS 文件超过 400 行关注阈值（types/online 596 / utils/api/tools 494 / utils/api/online-manager 540 / composables/useDragDrop 455 / stores/frp 527 / stores/online 856 / views/tools/data/useSeedMap 828），违反"TypeScript 文件 >400 行需关注"硬约束。generatorWorker.ts 682 行经评估后保留（见下文说明）
- 改动（全部采用主文件 re-export 保持调用方路径完全兼容，零调用方改动）：
  - **types/online.ts**（596 → 25 行）：按域拆分为 `types/online/` 下 8 个子文件（`auth.ts`/`signaling.ts`/`modpack.ts`/`room.ts`/`tun.ts`/`nat.ts`/`whitelist.ts`/`lobby.ts`），主文件 `export * from './online/xxx'` 聚合
  - **utils/api/tools.ts**（494 → 27 行）：按工具类别拆分为 `utils/api/tools/` 下 7 个子文件（`core.ts`/`download.ts`/`cleanup.ts`/`mod.ts`/`data.ts`/`archive.ts`/`network.ts`），主文件 re-export
  - **utils/api/online-manager.ts**（540 → 28 行）：按 action 类别拆分为 `utils/api/online-manager/` 下 8 个子文件（`core.ts`/`auth.ts`/`room.ts`/`turn.ts`/`mesh.ts`/`tun.ts`/`whitelist.ts`/`lobby.ts`），主文件 re-export
  - **composables/useDragDrop.ts**（455 → 92 行）：拆分为 `composables/useDragDrop/` 下 `state.ts`（拖拽状态+扩展名常量+classifyDrag+hideOverlay）与 `handlers.ts`（文件类型分发与安装处理），主文件保留 useDragDrop() 生命周期函数并 re-export 子模块
  - **stores/frp.ts**（527 → 385 行）：抽取 `stores/frp/authSlice.ts`（useFrpAuthSlice：认证 state + actions），主文件解构合并 auth 切片
  - **stores/online.ts**（856 → 67 行）：拆分为 `stores/online/` 下 `types.ts`（RoomRole/RoomState/emptyRoom）+ 4 个 Pinia 切片（`authSlice.ts`/`roomSlice.ts`/`whitelistSlice.ts`/`natSlice.ts`），主文件组合切片并 re-export 类型
  - **views/tools/data/useSeedMap.ts**（828 → 545 行）：抽取 `useSeedMap/config.ts`（Zoom/extent 常量 + SEEDMAP_MC_VERSIONS + mapMcVersionToCubiomes）、`useSeedMap/tileLoader.ts`（createTileLoader 工厂）、`useSeedMap/structureManager.ts`（createStructureManager 工厂），主文件保留 initMap + 事件处理 + 生命周期
- generatorWorker.ts（682 行）保留未拆：WASM Worker 所有 handler 共享可变 `Module` 状态 + 紧耦合辅助函数（ensureHeap/writeSeedString/callChunkFinder/callFinderOnce），拆分需引入复杂 context 传递且不改善可读性；按约束"WASM Worker 如逻辑密集难拆可不拆"保留
- 附带修复（stores/online.ts 拆分引入的 tsc 错误）：
  - [src/stores/online/roomSlice.ts](src/stores/online/roomSlice.ts)：移除未使用的 `toastError` 导入；`guestJoinRoom` 的 roomState 对象补全缺失的 `hostModpack: undefined` 字段（RoomState 类型要求该字段必须存在）
- 验证：`tsc --noEmit` 零新增错误（仅预存于未修改文件的 4 个错误：htmlShadowRenderer.ts/renderHelpers.ts 3 个来自 CustomLayoutPanel 拆分、crypto.ts 1 个 TypeScript 5.7+ ArrayBuffer 严格模式兼容问题）

#### config 整合：移除 get_config_value/set_config_value 单字段命令

- 背景：V3 代码质量报告 P1 违规——配置读写未走统一的 `apply_config`/`get_config`，仍存在 `get_config_value`/`set_config_value` 单字段命令，违反"配置读写统一走 `apply_config`/`get_config`，不应新增 `set_*`/`get_*` 单字段命令"硬约束
- 改动（方案 A 根治：扩展 ConfigPatch/ConfigSnapshot 加入 java_path 字段，彻底删除单字段命令）：
  - [src-tauri/src/commands/system/apply_config/types.rs](src-tauri/src/commands/system/apply_config/types.rs)：`ConfigPatch` 新增 `java_path: Option<String>`（`#[serde(default, skip_serializing_if = "Option::is_none")]`）；`ConfigSnapshot` 新增 `java_path: Option<String>`（`#[serde(default)]`）；`build_snapshot` 新增 `java_path` 参数并注入 snapshot
  - [src-tauri/src/commands/system/apply_config/mod.rs](src-tauri/src/commands/system/apply_config/mod.rs)：`get_config` 通过 `Storage::instance().get_config("Java", "path")` 读取 INI [Java] path 并传入 `build_snapshot`
  - [src-tauri/src/commands/system/apply_config/apply.rs](src-tauri/src/commands/system/apply_config/apply.rs)：新增 `apply_java` 子函数写 INI [Java] path（不进 AppConfig，保留独立存储设计）；在 `apply_config_inner` 的 secure 分流阶段调用（与 `apply_curseforge`/`apply_developer_mode`/`apply_ignore_tls` 同级）
  - [src-tauri/src/commands/system/config.rs](src-tauri/src/commands/system/config.rs)：删除 `get_config_value`/`set_config_value`/`is_valid_config_key` 三个函数；移除不再使用的 `log_err` 导入；更新 `config_manager` 文档注释（4 → 2 个 action）
  - [src-tauri/src/utils/config_manager.rs](src-tauri/src/utils/config_manager.rs)：删除 `get_config_value`/`set_config_value` 导入、`GetConfigValueParams`/`SetConfigValueParams` 结构体、两个 `d.register` 块；更新文件头注释（4 → 2 个 action）
  - [src/utils/api/config.ts](src/utils/api/config.ts)：删除 `getConfigValue`/`setConfigValue` 封装函数；`ConfigSnapshot` 接口新增 `javaPath: string | null`；`ConfigPatch` 接口新增 `javaPath?: string`；更新文件头注释（4 → 2 个命令）
  - [src/utils/api/config-manager.ts](src/utils/api/config-manager.ts)：删除 `GET_CONFIG_VALUE`/`SET_CONFIG_VALUE` 常量；更新文件头注释（4 → 2 个 action）
  - [src/stores/java.ts](src/stores/java.ts)：`loadSavedJavaPath` 改用 `tauri.getConfigMap()` 读 `config.javaPath`；`saveJavaPath` 改用 `tauri.applyConfig({ javaPath: path })`
- 设计保留：Java path 仍走 INI [Java] path 独立存储，不进 AppConfig 内存态（历史有意设计，`apply_java` 不在 `update_config` 闭包内，与 `secure::apply_*` 同属非 AppConfig 分流）
- 验证：`cargo check` 通过无错误无警告；`tsc --noEmit` 零新增错误（仅预存于未修改文件的 5 个错误）；所有 Rust 文件均未超 350 行（apply.rs 350 / types.rs 383 为预存超标，本次仅 +8 行）

#### views/ 与 plugins/ 目录原生 button 整改为 Button.vue 组件

- 背景：V3 代码质量报告 P1 违规——前端硬约束要求"必须用项目自定义组件而非原生 HTML（`Button.vue` 不用 `<button>`）"，但 views/ 和 plugins/ 多个文件仍使用原生 `<button>` 实现刷新等常规按钮，与 `VersionSelect.vue` 已有的 `<Button type="ghost" size="small">` 刷新按钮模式不一致
- 改动（仅替换常规 icon+text 按钮为 `<Button>` 组件，保留纯图标/列表项/折叠头/链接卡片等特殊场景原生 button 并补充注释，未改变功能与交互）：
  - [src/views/settings/SettingsCache.vue](src/views/settings/SettingsCache.vue)：刷新按钮 → `<Button type="ghost" size="mini">`
  - [src/views/settings/plugins/PluginListSection.vue](src/views/settings/plugins/PluginListSection.vue)：刷新按钮 → `<Button type="ghost" size="mini">`（卸载按钮因自定义红色样式保留原生并注释）
  - [src/plugins/cache-monitor/CacheMonitorPanel.vue](src/plugins/cache-monitor/CacheMonitorPanel.vue)：刷新按钮 → `<Button type="ghost" size="mini">`
  - [src/plugins/custom-layout/CustomLayoutPanel.vue](src/plugins/custom-layout/CustomLayoutPanel.vue)：刷新按钮 → `<Button type="ghost" size="mini">`
  - [src/plugins/system-monitor/SystemMonitorPanel.vue](src/plugins/system-monitor/SystemMonitorPanel.vue)：主刷新按钮 → `<Button type="ghost" size="mini">`（缓存刷新为纯图标 + text-[10px] 紧凑尺寸，保留原生并修正注释）
  - [src/plugins/launch-history/LaunchHistoryPanel.vue](src/plugins/launch-history/LaunchHistoryPanel.vue)：刷新按钮 → `<Button type="ghost" size="mini">`
  - [src/plugins/version-stats/VersionStatsPanel.vue](src/plugins/version-stats/VersionStatsPanel.vue)：刷新按钮 → `<Button type="ghost" size="mini">`
  - [src/views/VersionSelect.vue](src/views/VersionSelect.vue)：版本列表项 button 补充"保留原生"注释（与 FolderSidebar/DownloadSidebar 列表项一致）
  - [src/views/version-settings/mod-tab/ModListItem.vue](src/views/version-settings/mod-tab/ModListItem.vue)：6 个纯图标工具栏按钮的共享保留注释修正（原误写"padding 0 15px"，实际 mini 为 0 11px；补充自定义 hover 配色这一核心原因）
- 保留原生 button 的场景（均有注释说明）：纯图标按钮（ModListItem 6 个工具栏按钮、SystemMonitor 缓存刷新）、列表项（FolderSidebar 文件夹项、DownloadSidebar 导航项、VersionSelect 版本项、PluginListSection 卸载按钮）、折叠头（SeedMapIntro、VersionSelect 分组头、CreditsTab 作者展开）、链接卡片（AboutTab 3 组依赖列表）
- 验证：`tsc --noEmit` 通过（仅预存于未修改文件的 5 个错误）；ESLint 对 9 个修改文件零报错；所有文件均未超 300 行（最大 CreditsTab.vue 270 行）

#### 日志级别降级：下载模块与 Frp Sandbox 内部细节从 log_info! 改为 log_debug!

- 背景：V3 代码质量报告 P2 违规——下载模块和 Frp Sandbox 的内部实现细节日志误用 `log_info!`，导致 INFO 级别刷屏，违反"内部实现细节日志必须使用 DEBUG 级别"硬约束
- 改动（仅将内部实现细节日志从 `log_info!` 降级为 `log_debug!`，宏调用格式不变；同步清理因此变为未使用的 `log_info` 导入，未触碰 `log_warn!`/`log_error!`）：
  - [src-tauri/src/minecraft/download/full_download.rs](src-tauri/src/minecraft/download/full_download.rs)：L126 下载完成统计（Libs/Assets 计数汇总）
  - [src-tauri/src/minecraft/download/stages.rs](src-tauri/src/minecraft/download/stages.rs)：L54 客户端 JAR 已存在跳过、L65 客户端 JAR 下载步骤、L114 Libraries 总数/缺失统计、L199 Assets 总数/缺失统计
  - [src-tauri/src/minecraft/download/manager.rs](src-tauri/src/minecraft/download/manager.rs)：L235 暂停期间检测到取消信号
  - [src-tauri/src/minecraft/download/fix.rs](src-tauri/src/minecraft/download/fix.rs)：L50 客户端 JAR 下载失败的预期性提示
  - [src-tauri/src/minecraft/download/downloader/single.rs](src-tauri/src/minecraft/download/downloader/single.rs)：L124 分片下载策略选择、L180 分片返回 404 回退单流
  - [src-tauri/src/minecraft/download/chunk/mod.rs](src-tauri/src/minecraft/download/chunk/mod.rs)：L119 分片下载开始（含文件大小探测）、L222 分片下载完成统计
  - [src-tauri/src/commands/frp/sandbox.rs](src-tauri/src/commands/frp/sandbox.rs)：L331 认证适配器执行完成细节
- 验证：`cargo check` 通过，无错误无警告

#### 拆分超 300 行 Vue 组件（满足项目硬约束）

- 背景：项目记忆明确约束 Vue 组件文件不得超过 300 行，4 个组件超标（CustomLayoutPanel 466 / Input 396 / ArchiveManager 335 / Online 356），需拆分以满足约束并提升可维护性
- 改动（保持现有功能、样式、API、交互完全不变，仅做职责切分）：
  - **CustomLayoutPanel**（[src/plugins/custom-layout/CustomLayoutPanel.vue](src/plugins/custom-layout/CustomLayoutPanel.vue)）：466 → 164 行。提取 `LayoutSectionRenderer.vue`（按 section type 渲染 stat-grid/list/progress/text/divider/html，141 行）、`renderHelpers.ts`（纯函数 + 颜色映射常量，78 行）、`htmlShadowRenderer.ts`（shadow DOM 渲染 + setupMolaunchApi，156 行）。复用 `@/composables/usePolling` 替代手写 setInterval
  - **Input**（[src/components/common/Input.vue](src/components/common/Input.vue)）：396 → 203 行。采用与 `Select.vue` / `ColorPicker.vue` 一致的模式，将样式提取到 [src/components/common/Input.css](src/components/common/Input.css)（192 行），主文件用 `<style scoped src="./Input.css">` 引入。Props/Emits/Slots 接口未变，31 个引用方零改动
  - **ArchiveManager**（[src/views/tools/archive/ArchiveManager.vue](src/views/tools/archive/ArchiveManager.vue)）：335 → 191 行。提取 `ArchiveBackupDialog.vue`（备份弹窗表单与逻辑，143 行）、`ArchiveRestorePanel.vue`（恢复面板表单与逻辑，114 行）。通过 props/emit 接口协调，模态弹窗语义保持等价
  - **Online**（[src/views/Online.vue](src/views/Online.vue)）：356 → 165 行。提取 `useOnlineNav.ts`（导航分类配置 + isReady/isInRoom 状态计算 + 自动切换分类 watch，217 行）、`CloudDisconnectedMask.vue`（云端连接失败遮罩，41 行）、`OnlineTopBar.vue`（顶部标题栏 + 状态徽章，56 行）。provide 链路（hostMesh/guestWebrtc/goToLogs）零改动

### 修复

#### toast 函数误用 BUG 修复（双参数被静默丢弃）

- 背景：`toast.ts` 中 `toastSuccess` / `toastError` / `toastWarning` / `toastInfo` 仅接受单参数，部分调用方误传 2 个参数，第二个参数被静默丢弃，导致用户看到的提示文案不完整
- 改动（统一改为单参数字符串拼接）：
  - [src/components/version/InstalledList.vue](src/components/version/InstalledList.vue)：`toastSuccess('已停止', '游戏进程已终止')` → `toastSuccess('已停止，游戏进程已终止')`；启动/停止失败的 `showError` 改为 `toastError`，防呆检查的 `showWarning` 改为 `toastWarning`（移除 `@/utils/modal` 依赖）
  - [src/components/home/LaunchPanel.vue](src/components/home/LaunchPanel.vue)：`toastError('启动失败', String(e))` → `toastError('启动失败：' + String(e))`；补 `toastSuccess('游戏已启动')` / `toastInfo('已取消启动')` / `toastInfo('已停止游戏')`
  - [src/components/common/CrashDialog.vue](src/components/common/CrashDialog.vue)：`openCrashReport` / `exportReport` 的 3 处 toast 双参数调用改为单参数拼接

#### Home / Login 模块补全缺失的 toast 提示

- 背景：登录、账号切换、外链打开、设备码复制等操作成功/失败均无反馈，或仅 `console.error` / `.catch(() => {})` 静默吞错
- 改动（仅在各操作的成功/失败分支追加 toast 调用，不改其他逻辑）：
  - [src/views/Login.vue](src/views/Login.vue)：离线/微软/外置登录成功补 `toastSuccess('登录成功')`；`openBuyPage` / `openOfficialSite` 失败补 `toastError`
  - [src/components/home/AccountSelector.vue](src/components/home/AccountSelector.vue)：`addAccount` 跳转登录页失败补 `toastError`
  - [src/components/common/ExternalLoginPanel.vue](src/components/common/ExternalLoginPanel.vue)：外置登录成功补 `toastSuccess('外置登录成功')`；`openRegister` 失败补 `toastError`
  - [src/components/common/DeviceCodeModal.vue](src/components/common/DeviceCodeModal.vue)：`copyToClipboard` 改为返回 `Promise<boolean>`，成功/失败均 toast；`openBrowser` 失败补 `toastError`；微软登录成功补 `toastSuccess`

#### Version 模块 composables 补全缺失的 toast 提示

- 背景：账号登出/删除/切换、Mod 列表刷新、版本列表刷新、文件选择取消、内存配置自动保存等场景缺失用户反馈
- 改动（仅在各操作的成功/失败分支追加 toast 调用，不改其他逻辑）：
  - [src/composables/useAccountCards.ts](src/composables/useAccountCards.ts)：`logout` / `removeAccount` / `switchAccount` 成功补 `toastSuccess`，失败由 `toastWarning` 提级为 `toastError`
  - [src/composables/useModList.ts](src/composables/useModList.ts)：`loadMods` 非 silent 调用成功补 `toastSuccess('Mod 列表已刷新')`；`handleInstallMod` 取消选择补 `toastInfo('已取消安装')`
  - [src/composables/useModUpdate.ts](src/composables/useModUpdate.ts)：`loadVersions` 失败补 `toastError('查询版本列表失败')`
  - [src/composables/useExportTab.ts](src/composables/useExportTab.ts)：3 处文件选择取消补 `toastInfo('已取消保存/读取/导出')`
  - [src/composables/useVersionOverviewActions.ts](src/composables/useVersionOverviewActions.ts)：`handleExportScript` 取消选择补 `toastInfo('已取消导出')`
  - [src/views/version-settings/MemorySection.vue](src/views/version-settings/MemorySection.vue)：`flushSaveMemory` 的 `safeCall` 补 `onError` 回调 `toastError('内存配置保存失败')`
  - [src/views/version-settings/setup-tab/JavaCustomMode.vue](src/views/version-settings/setup-tab/JavaCustomMode.vue)：`handleImportJava` 取消选择补 `toastInfo('已取消导入')`
  - [src/views/version-select/FolderSidebar.vue](src/views/version-select/FolderSidebar.vue)：加载文件夹列表失败补 `toastError`；`addFolder` 取消选择补 `toastInfo('已取消选择')`
  - [src/views/VersionSelect.vue](src/views/VersionSelect.vue)：`loadInstalled` 由 `safeCall` 改为 try/catch，进入时 `toastInfo('正在刷新版本列表...')`，成功 `toastSuccess('版本列表已刷新')`，失败 `toastError`

#### utils 通用层 / 联机 / Modal 补全缺失的 toast 提示

- 背景：通用工具函数 `openLink`、百科搜索、地图 tile 加载、Modal 复制、房间离开等场景静默吞错
- 改动（仅在各操作的成功/失败分支追加 toast 调用，不改其他逻辑）：
  - [src/utils/aboutLogos.ts](src/utils/aboutLogos.ts)：`openLink` 失败补 `toastError('打开链接失败')`（被 AboutTab / CreditsTab 等多处复用，统一兜底）
  - [src/composables/useModDetailQuery.ts](src/composables/useModDetailQuery.ts)：mcmod 搜索页打开失败补 `toastError('打开百科失败')`
  - [src/views/tools/data/useSeedMap.ts](src/views/tools/data/useSeedMap.ts)：tile 加载首次失败补 `toastError('地图加载失败，请重试')`（带 `tileErrorToastShown` 防抖标志，成功后重置，避免刷屏）；specials 请求失败补 `toastError('加载 specials 失败')`
  - [src/components/common/Modal.vue](src/components/common/Modal.vue)：`copyDetails` 改为 async + try/catch，成功 `toastSuccess('已复制错误详情')`，失败 `toastError('复制失败')`
  - [src/components/layout/TopNavLayout.vue](src/components/layout/TopNavLayout.vue) / [src/components/online/RoomManager.vue](src/components/online/RoomManager.vue)：`hostCloseRoom` / `guestLeaveRoom` 失败补 `toastError('离开房间失败')`

#### Settings 开发者配置加载补全缺失的 toast 提示

- 背景：开发者页多个子 Tab 的 onMounted 加载失败仅 `console.error`，用户无感知
- 改动（仅给 `safeCall` 补第三参数 `onError`，不改其他逻辑）：
  - [src/views/settings/developer/CertsTab.vue](src/views/settings/developer/CertsTab.vue)：加载自定义证书列表失败补 `toastError('加载自定义证书失败')`
  - [src/views/settings/developer/DevToolsTab.vue](src/views/settings/developer/DevToolsTab.vue)：查询 DevTools 状态失败补 `toastError('查询 DevTools 状态失败')`
  - [src/views/settings/developer/ExperimentalTab.vue](src/views/settings/developer/ExperimentalTab.vue)：加载开发者配置失败补 `toastError('加载开发者配置失败')`
  - [src/views/settings/settings-launch/MemoryAllocation.vue](src/views/settings/settings-launch/MemoryAllocation.vue)：获取系统内存失败补 `toastError('获取系统内存信息失败')`
  - [src/views/Settings.vue](src/views/Settings.vue)：读取开发者模式失败补 `toastError('读取开发者模式失败')`
  - [src/views/version-settings/SetupTab.vue](src/views/version-settings/SetupTab.vue)：`loadSetup` 失败补 `toastError('加载版本设置失败')`
  - [src/views/version-settings/setup-tab/JavaModeSelector.vue](src/views/version-settings/setup-tab/JavaModeSelector.vue)：加载 Java 需求失败补 `toastError('加载 Java 需求失败')`

#### stores 层补全缺失的 toast 提示

- 背景：`online` store 的 `logout` / `clear`、`plugins` store 的 `persistToBackend` 内部 `safeCall` 未传 `onError`，失败仅 console；外层调用方无法感知
- 改动（在 store 层统一加 toast，避免每个调用方重复处理）：
  - [src/stores/online.ts](src/stores/online.ts)：`logout` 失败补 `toastError('登出失败')`；`clear` 失败补 `toastError('清除凭证失败')`
  - [src/stores/plugins.ts](src/stores/plugins.ts)：`persistToBackend` 内部 `safeCall` 补 `onError` 回调 `toastError('保存设置失败')`，统一覆盖 `setHomePanelMode` / `setCustomLayoutConfig` 等所有持久化调用点（移除子 agent 在调用方加的冗余 try/catch）

#### Settings 模块补全缺失的 toast 提示

- 背景：Settings 各子页的部分操作（保存、加载、刷新、开关切换、连接测试）缺少用户可见反馈，仅打印 console，用户无法感知操作结果
- 改动（仅在各操作的成功/失败分支追加 toast 调用，不改其他逻辑）：
  - **公共 composable**（[src/composables/useConfigPage.ts](src/composables/useConfigPage.ts)）：防抖保存失败补 `toastError('配置保存失败')`；reload 加载失败补 `toastError('配置加载失败')`（保留原 `onLoadError` 回调，成功分支不动避免打扰）
  - **缓存管理**（[src/views/settings/SettingsCache.vue](src/views/settings/SettingsCache.vue)）：`loadCacheStats` 刷新成功后 `toastSuccess('缓存统计已刷新')`
  - **HTTP 日志**（[src/components/settings/HttpLogViewer.vue](src/components/settings/HttpLogViewer.vue)）：`loadEntries` 改为返回 `Promise<boolean>`，`onRefresh` 仅在成功时 `toastSuccess('HTTP 日志已刷新')`（避免失败时同时弹错误与成功提示）
  - **开发者模式**（[src/components/settings/DevModeToggle.vue](src/components/settings/DevModeToggle.vue)）：`toggleDevMode` 成功后 `toastInfo` 提示开关状态
  - **实验性功能**（[src/views/settings/developer/ExperimentalTab.vue](src/views/settings/developer/ExperimentalTab.vue)）：`toggleModrinthCdnRaw` 成功后 `toastInfo` 提示开关状态
  - **证书与安全**（[src/views/settings/developer/CertsTab.vue](src/views/settings/developer/CertsTab.vue)）：`changeTrustMode` 成功后 `toastInfo('信任源模式已切换')`；`toggleIgnoreTls` 成功后 `toastWarning` 提示安全风险
  - **api-server**（[src/components/settings/ApiServerCard.vue](src/components/settings/ApiServerCard.vue)）：`handleTestConnection` 成功 `toastSuccess('连接成功')`、失败 `toastError('连接失败')`
  - **Java 路径**（[src/views/settings/settings-launch/JavaPathSelector.vue](src/views/settings/settings-launch/JavaPathSelector.vue)）：`handleManualImportJava` 设置成功后 `toastSuccess('Java 路径已设置')`

#### 联机模块补全缺失的 toast 提示

- 背景：联机各交互（设备注册/登录、NAT 检测、封禁列表刷新、版本加载、大厅刷新）部分分支仅打印 console 或静默兜底，用户无可见反馈
- 改动（仅在各操作的成功/失败分支追加 toast 调用，不改其他逻辑）：
  - **设备面板**（[src/components/online/OnlineDevicePanel.vue](src/components/online/OnlineDevicePanel.vue)）：`handleRegister` 失败 `toastError('设备注册失败，请稍后重试')`；`handleLogin` 失败 `toastError('设备登录失败，请稍后重试')`；`handleDetectNat` 成功 `toastSuccess('NAT 检测完成')`，异常或 `natResult.error` 有值时 `toastError('NAT 检测失败，请检查网络后重试')`
  - **房主运营 composable**（[src/composables/useRoomHost.ts](src/composables/useRoomHost.ts)）：`refreshBans` 成功 `toastSuccess('封禁列表已刷新')`，业务失败（code!==1）`toastError(result.msg || '刷新封禁列表失败')`，异常 `toastError('刷新封禁列表失败')`
  - **创建房间表单**（[src/components/online/CreateRoomForm.vue](src/components/online/CreateRoomForm.vue)）：`onMounted` 加载版本列表失败补 `toastError('加载已安装版本列表失败，请重试')`；`onVersionSelect` 解析失败兜底时补 `toastWarning('版本信息解析失败，已使用版本 ID 作为兜底，请核对加载器类型')`
  - **大厅浏览**（[src/components/online/LobbyBrowser.vue](src/components/online/LobbyBrowser.vue)）：`fetchRooms` 改为返回 `Promise<boolean>`，新增 `handleManualRefresh` 仅在点击刷新按钮且成功时 `toastInfo('已刷新房间列表')`（onMounted/搜索/翻页等自动加载不提示）

#### 工具页补全缺失的 toast 提示

- 背景：`src/views/tools/` 下多个工具组件的操作（复制、检测、扫描、刷新、分析）缺失用户可见反馈，且 ColorPalette.vue 内联了 copyToClipboard 违反项目复用规则
- 改动（仅在各操作的成功/失败分支追加 toast 调用，不改其他逻辑）：
  - **调色板**（[src/views/tools/calc/ColorPalette.vue](src/views/tools/calc/ColorPalette.vue)）：删除内联 copyToClipboard（直接 navigator.clipboard?.writeText），改用 `@/utils/seedmap/format` 的共享 copyToClipboard（返回 Promise<boolean>）；新增 copyHex / copyCode 异步处理函数，复制成功 `toastSuccess`、失败 `toastError('复制失败')`
  - **服务器检测**（[src/views/tools/network/ServerPinger.vue](src/views/tools/network/ServerPinger.vue)）：`doPing` 成功（res.error 为空）后 `toastSuccess('检测完成，延迟 ' + res.latency_ms + ' ms')`
  - **Mod 去重**（[src/views/tools/mod-tools/ModDedupScanner.vue](src/views/tools/mod-tools/ModDedupScanner.vue)）：`runScan` 成功后按 duplicates.length 给 `toastWarning`（有重复）/ `toastSuccess`（无重复）
  - **Mod 依赖检测**（[src/views/tools/mod-tools/ModDependencyChecker.vue](src/views/tools/mod-tools/ModDependencyChecker.vue)）：`runCheck` 成功后按 missing.length 给 `toastWarning`（有缺失）/ `toastSuccess`（无缺失）
  - **存档管理 / 资源包转换 / 截图管理**（ArchiveManager.vue / ResourcePackConverter.vue / ScreenshotManager.vue）：新增 `refresh()` 包装函数（await loadList 后 `toastSuccess('已刷新')`），刷新按钮改调 refresh()，避免 onMounted/watch 触发时也弹 toast；loadVersions catch 块的 console.warn 替换为 `toastError('加载版本列表失败')`
  - **崩溃分析**（[src/views/tools/data/CrashAnalyzer.vue](src/views/tools/data/CrashAnalyzer.vue)）：`runAnalyze` 有结果时 `toastSuccess` 提示识别原因数量（与原有"未识别到已知崩溃模式"的 toastInfo 互补）
- 复用说明：复用 `@/utils/toast` 的 toastSuccess/toastError/toastWarning/toastInfo；复用 `@/utils/seedmap/format` 的 copyToClipboard 共享函数，删除 ColorPalette 内联实现

#### Community 与 Downloads 模块补全缺失的 toast 提示

- 背景：Community 资源搜索/详情与 Downloads 下载管理的部分操作（分类加载失败、打开官网、搜索无结果、重置筛选、暂停/恢复/取消下载、重试失败踢回、打开下载目录）缺失用户可见反馈，仅 console 或静默兜底，用户无感知
- 改动（仅在各操作的成功/失败分支追加 toast 调用，不改其他逻辑）：
  - **搜索筛选栏**（[src/components/community/SearchBar.vue](src/components/community/SearchBar.vue)）：`watch(resourceType)` 的 catch 块原连参数都没接收，改为 `catch (e)` 并补 `toastError('分类标签加载失败，请检查网络')`
  - **资源详情头部**（[src/components/community/resource-detail/ResourceDetailHeader.vue](src/components/community/resource-detail/ResourceDetailHeader.vue)）：新增 `openWebsite` 函数包装 `openUrl(project.website)`，与 openMcmod 对齐补 `toastInfo('正在打开官网')`；MC 百科直链查询 catch 块补 `toastWarning('MC 百科信息查询失败')`（保留原 console.debug）
  - **社区资源内容区**（[src/views/Community.vue](src/views/Community.vue)）：`doSearch` 成功但 `projects.length === 0` 时 `toastInfo('未找到匹配的资源')`；`onReset` 重置筛选后 `toastInfo('已重置筛选条件')`
  - **下载管理页**（[src/views/Downloads.vue](src/views/Downloads.vue)）：`handleTogglePause` 利用 `safeCall` 的 onError 回调失败 `toastError('操作失败')`，成功分支按暂停/恢复分别 `toastInfo('下载已暂停' / '下载已恢复')`；`handleCancel` 改用 `safeCall` 返回值判断成功失败，失败 `toastError('取消失败')` 并提前 return，成功 `toastInfo('下载已取消')` 后再 `finishDownload`（确保 toast 在 router.back 触发前调用）；`onMounted` 重试 6 次仍无任务时 `toastWarning('未检测到下载任务，已返回')`
  - **外部下载 composable**（[src/composables/useExternalDownload.ts](src/composables/useExternalDownload.ts)）：`openDownloadDir` 原 无 try/catch，补 try/catch + `toastError('打开目录失败')`，错误信息拼接 `e.message`
- 复用说明：复用 `@/utils/toast` 的 toastSuccess/toastError/toastWarning/toastInfo；Downloads.vue 复用 `safeCall` 既有的 onError 回调签名（第三参数）判断失败，不引入新工具函数；handleCancel 失败时提前 return 避免在 finishDownload 触发 router.back 后 toast 丢失

#### FRP 模块补全缺失的 toast 提示

- 背景：FRP 各组件的刷新、清空、认证流程、自检等操作缺少用户可见反馈，用户无法感知操作结果
- 改动（仅在各操作的成功/失败分支追加 toast 调用，不改其他逻辑）：
  - **认证中心**（[src/components/frp/AuthCenter.vue](src/components/frp/AuthCenter.vue)）：`openUrl` 失败 `toastError('打开链接失败')`（移除 `/* ignore */`）；`handleStartOAuth2` 成功 `toastInfo('认证窗口已在浏览器打开，请完成后返回')`；`handleStartDeviceCode` 成功 `toastInfo('Device Code 流程已启动，请访问验证链接输入用户码')`；`handleCancelDeviceCode` 加 `toastInfo('已取消 Device Code 流程')`；刷新按钮原内联 `store.loadAuthStatuses()` 抽出为 `handleRefreshAuthStatuses`，成功后 `toastInfo('认证状态已刷新')`
  - **FRP 日志**（[src/components/frp/FrpLogs.vue](src/components/frp/FrpLogs.vue)）：`handleRefresh` 成功 `toastInfo('日志已刷新')`；`handleClear` 加 `toastInfo('日志已清空')`
  - **厂商列表**（[src/components/frp/ProviderList.vue](src/components/frp/ProviderList.vue)）：`handleRefresh` 成功 `toastInfo('厂商列表已刷新')`
  - **隧道管理**（[src/components/frp/TunnelManager.vue](src/components/frp/TunnelManager.vue)）：`handleRefresh` 成功 `toastInfo('隧道列表已刷新')`
  - **隧道自检**（[src/components/frp/TunnelSelfCheck.vue](src/components/frp/TunnelSelfCheck.vue)）：`runCheck` 补 `catch (e) { toastError('自检失败：' + ...) }`，成功完成 `toastInfo('自检完成')`

### 新增

#### Frp 阶段四：安全加固

- 背景：完善 Frp 后端安全防护，覆盖网络白名单强制校验、frpc 进程隔离、日志脱敏、API 重定向防护四项，对应设计文档 §7 安全沙箱（§7.2 配置校验、§7.3 进程隔离、§7.7 frpc 二进制下载安全）
- 改动：
  - **网络白名单强制校验**（[src-tauri/src/commands/frp/sandbox.rs](src-tauri/src/commands/frp/sandbox.rs)）：
    - `validate_tunnel` 新增 `validate_network_permissions` 校验：读取厂商 manifest 的 `networkPermissions`，当 `allow_custom_server=false` 时 `server_addr` 必须在 `allowedServers` 白名单内（host 级匹配，允许白名单只写 host 或 host:port）
    - 新增 `is_private_address` 函数：非系统默认厂商禁止连接内网地址（10.0.0.0/8、172.16.0.0/12、192.168.0.0/16、127.0.0.0/8），防止 SSRF；系统默认厂商豁免（用户自建 frps 可能位于内网）
    - 新增本地端口特权端口检查：`local_port < 1024` 拒绝（防止 frpc 获取不必要的系统权限）
  - **frpc 进程环境变量清空 + Job Object**（[src-tauri/src/commands/frp/process.rs](src-tauri/src/commands/frp/process.rs)）：
    - frpc 子进程启动时 `env_clear()` 清空环境变量，仅保留 `PATH`，防止敏感环境变量泄露
    - Windows 下新增 `assign_process_to_job_object` 函数：创建带 `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` 的 Job Object 并关联子进程，确保启动器退出时 frpc 自动终止（防止僵尸进程）；依赖 `windows` crate 的 `Win32_System_JobObjects` feature（需在 Cargo.toml 启用）
    - `capture_stream` 新增 1MB 截断：单流（stdout/stderr）捕获超过 1MB 后写入截断提示并停止捕获，防止内存膨胀
    - `capture_stream` 集成 `log_redact::redact_log`：每行日志脱敏后再写入文件/推送前端
  - **日志脱敏模块**（新增 [src-tauri/src/commands/frp/log_redact.rs](src-tauri/src/commands/frp/log_redact.rs)）：
    - 实现 `pub fn redact_log(line: &str) -> String`：用正则匹配 `token`/`password`/`secret`/`api_key`/`auth_token`/`access_token`/`refresh_token` 字段（不区分大小写），将值替换为 `***`，保留字段名、分隔符和引号风格
    - 支持 TOML（`token = "xxx"` / `token=xxx`）与 JSON（`"token":"xxx"`）两种格式
    - `\b` 词边界确保只匹配完整字段名，不误匹配 `my_token` 等带前缀字段
  - **API 重定向防护**（[src-tauri/src/commands/frp/binary.rs](src-tauri/src/commands/frp/binary.rs)）：
    - `ensure_external_frpc` 的 HTTP 下载改用 `reqwest::redirect::Policy::none()` 禁止自动重定向
    - 手动检查 3xx 响应的 `Location` 头，解析重定向 URL 并校验域名是否在 `allowed_domains` 白名单内，不在白名单返回错误
    - 最多跟随 5 次重定向，防止无限循环；现有 SHA256 校验和文件写入逻辑保持不变
  - **共享类型扩展**（[src-tauri/src/commands/frp/mod.rs](src-tauri/src/commands/frp/mod.rs)）：
    - 新增 `pub mod log_redact;` 模块声明
    - 新增 `NetworkPermissions` 结构体（`allowed_servers` + `allow_custom_server`）
    - 新增 `ProcessPermissions` 结构体（`allowed_commands` + `timeout_ms`，默认 30000ms，上限 300000ms）
    - `ProviderManifest` 新增 `network_permissions: Option<NetworkPermissions>` 与 `process_permissions: Option<ProcessPermissions>` 字段（`#[serde(skip_serializing_if = "Option::is_none")]`）
    - 修复并发编辑导致的 `AuthConfig` 结构体闭合缺陷（补全 `}` 与 Default impl 的 oauth2/device_code/api_key 字段）
  - **认证适配器脚本沙箱**（[src-tauri/src/commands/frp/sandbox.rs](src-tauri/src/commands/frp/sandbox.rs)，对应 §7.5）：
    - 新增 `pub async fn run_auth_adapter(provider_id, command, args) -> Result<ProcessResult, String>`：沙箱化执行厂商自定义认证脚本
    - 流程：读取厂商 manifest 的 `process_permissions` → `is_command_allowed` 校验命令白名单 → 工作目录强制设为厂商目录 → `env_clear()` 清空环境变量仅保留 `PATH` → `tokio::time::timeout` 超时控制（默认 30s，上限 5min）→ 非 shell 执行 `Command::new` 防注入 → `truncate_output` 截断 stdout/stderr 到 1MB
    - 系统默认厂商直接拒绝（不提供自定义脚本）；`allowed_commands` 为空时拒绝
    - 新增 `run_auth_adapter` IPC action（参数 `RunAuthAdapterParams { providerId, command, args }`），返回 `ProcessResult { exitCode, stdout, stderr, timedOut, durationMs }`
  - **spawn.rs 工具函数提升可见性**（[src-tauri/src/commands/plugins/spawn.rs](src-tauri/src/commands/plugins/spawn.rs)）：
    - 将 `which_canonical` / `is_command_allowed` / `paths_equal` / `truncate_output` 四个函数从私有 `fn` 改为 `pub(crate) fn`
    - 将 `MAX_OUTPUT_BYTES` / `MAX_TIMEOUT_MS` 常量改为 `pub(crate)`
    - 设计文档 §7.5 明确要求"复用 spawn.rs:164-221"，避免在 frp 模块重复实现 which/白名单/截断逻辑
- 复用说明：
  - `is_private_address` 复用 `std::net::Ipv4Addr::is_private`/`is_loopback` 标准库方法，不重复实现 CIDR 判断
  - 重定向域名白名单匹配复用 `binary.rs` 既有 `host_matches` 函数（支持 `*.example.com` 一级通配符）
  - 日志脱敏的正则方案与项目已用的 `regex` crate 一致，`once_cell::sync::Lazy` 复用 process.rs 既有模式
  - 认证适配器沙箱复用 `spawn.rs` 的 `is_command_allowed` / `truncate_output` / `MAX_TIMEOUT_MS` / `ProcessResult`，不重复实现命令白名单与输出截断
- 待办（用户后续处理）：
  - 需在 `src-tauri/Cargo.toml` 的 `windows` features 中添加 `"Win32_System_JobObjects"` 以启用 Job Object 编译
- 安全收益：防止 frpc 子进程泄露敏感环境变量、防止僵尸进程、防止 SSRF、防止日志泄露 token/密码、防止下载重定向劫持

#### Frp 阶段三：认证体系（auth）

- 背景：Frp 阶段三需支持 OAuth2 / Device Code / API Key 三种认证流程，token 使用 OS 密钥存储（Windows Credential Manager / macOS Keychain / Linux Secret Service），认证后拉取厂商配置（参见设计文档 §6）
- 改动：
  - **新增认证模块**（新增 [src-tauri/src/commands/frp/auth.rs](src-tauri/src/commands/frp/auth.rs)）：
    - OS 密钥存储：使用 `keyring` crate，service=`frp:<provider_id>`，username=`access_token` / `refresh_token` / `expires_at` / `scopes`；keyring 不可用时返回明确错误
    - `get_auth_status(provider_id)`：查询认证状态（authenticated + authType + expiresAt + scopes），系统默认厂商始终 authenticated
    - `start_oauth2(state, provider_id)`：本地 `tokio::net::TcpListener` 监听 127.0.0.1:redirectPort 接收回调，浏览器跳转走 `shell::open_url`（项目约束），5 分钟超时，code 换取 token 后存储
    - `start_device_code(state, provider_id)`：POST deviceCodeUrl 获取设备码，device_code 存入内存会话（`Lazy<Mutex<HashMap>>`），返回 userCode + verificationUri + expiresIn + interval
    - `poll_device_code(state, provider_id)`：按 interval 轮询 tokenUrl，处理 pending / success / expired / declined / slow_down 五种状态
    - `refresh_token(state, provider_id)`：用 refresh_token 刷新 access_token
    - `revoke_auth(provider_id)`：删除所有 keyring 密钥 + Device Code 会话
    - `save_api_key(provider_id, api_key)`：API Key 直接存储为 access_token（无过期/无刷新）
    - `load_token(provider_id)`：读取 access_token（供 api_schema 模块调用，补全 api_schema 的依赖）
  - **mod.rs 扩展 AuthConfig**（[src-tauri/src/commands/frp/mod.rs](src-tauri/src/commands/frp/mod.rs)）：新增 `OAuth2Config` / `DeviceCodeConfig` / `ApiKeyConfig` 三个子配置结构体，AuthConfig 添加可选 oauth2 / device_code / api_key 字段；声明 `pub mod auth;`
  - **shell.rs 新增 open_url**（[src-tauri/src/minecraft/system/shell.rs](src-tauri/src/minecraft/system/shell.rs)）：跨平台打开 http/https URL（Windows cmd start / macOS open / Linux xdg-open），仅校验协议白名单不校验路径存在性，供 OAuth2 浏览器跳转使用
  - **前端类型**（[src/types/frp.ts](src/types/frp.ts)）：新增 AuthStatus / OAuth2Result / DeviceCodeResult / DeviceCodePollResult / DeviceCodePollStatus / SaveApiKeyParams
  - **前端 IPC 封装**（[src/utils/api/frp-manager.ts](src/utils/api/frp-manager.ts)）：新增 getAuthStatus / startOAuth2 / startDeviceCode / pollDeviceCode / refreshToken / revokeAuth / saveApiKey 七个封装函数 + 对应 FRP_ACTIONS 常量
  - **Pinia store**（[src/stores/frp.ts](src/stores/frp.ts)）：新增认证状态管理（authStatuses / authLoading / authActionLoading / deviceCodeInfos / deviceCodePolling / apiKeyInputs）+ 八个认证 actions
  - **认证中心 UI**（[src/components/frp/AuthCenter.vue](src/components/frp/AuthCenter.vue)）：替换占位 UI，实现单列卡片布局（259 行），覆盖四种认证类型 + 状态徽章 + Device Code 倒计时轮询 + API Key 输入 + 刷新/撤销操作，空状态 icon+text 居中
- 复用说明：
  - 复用 `super::provider::read_provider_manifest` 读取厂商清单，未重复实现 manifest 解析
  - 复用 `crate::http::get_client()` 全局 HTTP 客户端，未新建 reqwest::Client
  - 复用 `once_cell::sync::Lazy` 静态模式（与 frp_manager.rs 一致）存储 Device Code 会话
  - 复用 `crate::log_info!` / `crate::log_debug!` / `crate::log_error!` 宏
  - 前端复用 Button / Tooltip / Input 自定义组件 + showConfirm 确认弹窗 + ProviderList.vue 的徽章配色方案
- 依赖说明：
  - `keyring` crate 尚未添加到 Cargo.toml（代码中直接 use，由后续统一添加依赖）
  - OAuth2 state 生成基于系统时间纳秒 + PID，非密码学安全但足以防止本地回调伪造
- 待后续处理（不在本次修改范围）：
  - `src-tauri/src/utils/frp_manager.rs`：注册 get_auth_status / start_oauth2 / start_device_code / poll_device_code / refresh_token / revoke_auth / save_api_key 七个 IPC action
  - `src-tauri/Cargo.toml`：添加 `keyring` 依赖
  - `src-tauri/src/lib.rs`：如有需要调整模块声明

#### Frp 阶段三：厂商 API 引擎模块（api_schema）

- 背景：Frp 阶段三需支持认证后调用厂商 API 拉取 frpc 配置（frps 地址/端口/token/分配端口等），不同厂商的 API 路径/请求方式/响应格式各不相同，需按厂商打包的 api-schema.json 动态执行，避免为每个厂商硬编码适配逻辑
- 改动：
  - **新增 API 引擎模块**（新增 [src-tauri/src/commands/frp/api_schema.rs](src-tauri/src/commands/frp/api_schema.rs)）：
    - 类型定义：`ApiSchema` / `AuthInjection` / `Endpoints` / `ApiEndpoint` / `ApiParam` / `ConfigPayload`，全部带 `serde(rename_all = "camelCase")` 与设计文档 §7.6.2 schema 结构对齐
    - `load_api_schema(provider_id)`：读取并解析 `<providers>/<id>/api-schema.json`，校验 version=1 + baseUrl 为 HTTPS
    - `fetch_vendor_config(state, provider_id)`：认证后拉取配置主流程（加载 schema → 加载 token → 获取 device_id → 构造 HTTP 请求 → 响应映射 → 返回 ConfigPayload）
    - `render_config_template(provider_id, payload)`：读取 `config-template.toml`，替换 `{server_addr}` / `{server_port}` / `{token}` / `{assigned_remote_port}` / `{assigned_subdomain}` / `{自定义变量}` 占位符，字符串值做 TOML 转义
    - `get_json_path(value, path)`：按 dot 路径从 JSON Value 取值（pub，供外部复用）
    - HTTP 请求引擎：token 按 auth_injection.location 注入 header/query/body，params 模板填充 `{device_id}`，GET 参数走 query string、POST 参数走 JSON body
    - 安全约束（设计文档 §7.6.6）：超时默认 10s 最大 30s、响应体限制 1MB、重定向防护使用 `redirect::Policy::none()` + 手动校验 Location 同域白名单（最多 5 跳）、baseUrl 强制 HTTPS
    - 响应映射：按 response_mapping 的 dot 路径取值，标准字段名（兼容 camelCase/snake_case）写入 ConfigPayload，非标准字段名视为自定义变量
  - **mod.rs 注册模块**（[src-tauri/src/commands/frp/mod.rs](src-tauri/src/commands/frp/mod.rs)）：新增 `pub mod api_schema;` 声明
- 复用说明：
  - 复用 `super::{providers_root, validate_provider_id}` 路径与校验辅助函数，未重复实现厂商目录定位逻辑
  - 复用 `crate::commands::sdk::get_device_id` 获取 device_id，未重复 SDK 调用逻辑
  - TOML 字符串转义逻辑与 `tunnel.rs::escape_toml_string` 一致，因目标函数为私有且 tunnel.rs 不在本次修改范围，独立保留同名实现
- 依赖说明：
  - `crate::commands::frp::auth::load_token(provider_id) -> Result<String, String>`（async）由 auth 模块提供，用于从 OS 密钥存储读取 access_token；已在本次 auth 模块中实现
- 测试：覆盖 get_json_path / build_url / compute_timeout / extract_host / resolve_url / map_response（标准字段 + 自定义变量 + 必填缺失）/ render_config_template 占位符替换 + TOML 转义

### 调整

#### Checkbox 组件注释精简

- 精简 [src/components/common/Checkbox.vue](src/components/common/Checkbox.vue) 的 script 顶部块注释（从 17 行缩减到 4 行），移除冗余的视觉规格描述与"复刻 ArcoDesign"字样，仅保留组件名与用法示例；style 内两处"参考 ArcoDesign"注释同步移除

### 新增

#### Dev 调试 API（window.molaunch）

- 背景：联机/Frp 等页面在开发调试时缺少便捷入口，每次测试子窗口、IPC、路由跳转都需要走完整业务流程，效率低；同时 picker 子窗口各模板（port-picker / confirm / info / image-viewer / markdown / qrcode / redirect）缺少集中测试入口
- 改动：
  - **新增调试 API 模块**（新增 [src/utils/dev-api.ts](src/utils/dev-api.ts)）：
    - 导出 `setupDevApi(router)` 函数，仅在 `import.meta.env.DEV` 时挂载 `window.molaunch`，生产构建中 early-return 不影响 bundle 体积与安全性
    - 通过 `Object.defineProperty` 设置 `writable:false / configurable:false` 防止运行时被覆盖
    - 子命令：
      - `help()`：打印所有命令用法与示例
      - `templates()`：列出所有 picker 模板名
      - `picker(template, data?)`：打开 picker 子窗口（选择型返回值，展示型返回 undefined），自动复用 `utils/picker-window.ts` 既有 `openPickerWindow` / `openDisplayWindow`，避免重复实现
      - `pickPort()`：打开端口选择器（port-picker 模板快捷方式），返回 `number | null`
      - `navigate(path)`：通过注入的 router 实例跳转路由
      - `tools(action, params?)`：调用 `tools_manager` IPC（透传 action/params）
      - `frp(action, params?)`：调用 `frp_manager` IPC（透传 action/params）
      - `stores()`：动态 import 全部 8 个 Pinia store（auth/frp/java/online/plugins/sdk/settings/version），返回各自 `$state`，错误隔离单个 store 故障不影响其他
    - 全局类型：通过 `declare global { interface Window { molaunch?: MolaunchDevAPI } }` 让 TypeScript 识别 `window.molaunch`
  - **挂载入口**（[src/main.ts](src/main.ts)）：app.mount 后调用 `setupDevApi(router)`，dev 模式下控制台输出就绪日志
- 复用说明：
  - picker 子命令直接调用 `utils/picker-window.ts` 既有便捷函数，未重复封装 invoke/事件监听逻辑
  - tools/frp 子命令复用 `@tauri-apps/api/core` 的 invoke，与 `utils/api/tools.ts` / `utils/api/frp-manager.ts` 的请求结构一致（`{ req: { action, params } }`）
  - stores 子命令用动态 import 加载 store 模块，避免影响首屏 bundle
- 安全收益：生产构建中 `window.molaunch` 不存在，DevTools 无法通过此入口触发 IPC

### 新增

#### Checkbox 公共组件 + 全局替换原生复选框

- 背景：项目前端共有 13 处使用原生 `<input type="checkbox">`，视觉风格不统一（`accent-primary-500` / `h-4 w-4 rounded border-gray-300` 等多种 class），且缺乏 hover 背景效果、半选状态、scale 动画等 ArcoDesign 标准交互；项目规则要求复用自定义组件而非原生 HTML
- 改动：
  - **新增 Checkbox 组件**（新增 [src/components/common/Checkbox.vue](src/components/common/Checkbox.vue)）：
    - 复刻 Arco Design Vue 的 Checkbox 组件，视觉与交互完全对齐：14x14 方框、2px 边框、2px 圆角、icon-hover 28x28 圆形浅色背景、选中时主色底 + 白色勾选图标（scale 0→1 overshoot 动画）、半选时白色横条、禁用浅灰底
    - API 同时支持 `v-model`（双向绑定）和 `:checked` + `@change`（受控模式）两种用法，覆盖项目中所有复选框场景
    - 支持 `disabled`、`indeterminate`（半选）、`defaultChecked`（非受控默认值）props
    - 隐藏原生 input（opacity:0 + 宽高0）保留焦点可达性与键盘操作，用自定义 icon 模拟视觉
    - 顶部版权注释与 Button.vue / Input.vue 一致（Arco Design Vue 衍生 + MIT License）
  - **全局替换 13 处原生复选框**（8 个文件）：
    - [src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue)：启用 TLS 加密（v-model）
    - [src/components/online/ModpackSelector.vue](src/components/online/ModpackSelector.vue)：关联整合包开关（:checked + @change），onToggle 签名从 Event 改为 boolean
    - [src/components/online/WhitelistEditor.vue](src/components/online/WhitelistEditor.vue)：启用白名单开关（:checked + @change），简化 onToggleEnabled 调用
    - [src/views/version-settings/ExportTab.vue](src/views/version-settings/ExportTab.vue)：联网检查、仅 Modrinth（v-model）
    - [src/views/version-settings/export-tab/ExportOptions.vue](src/views/version-settings/export-tab/ExportOptions.vue)：导出选项顶层/子选项勾选（:checked + @click.stop.prevent）
    - [src/views/tools/archive/ArchiveManager.vue](src/views/tools/archive/ArchiveManager.vue)：排除玩家数据（v-model）
    - [src/views/tools/data/ScreenshotManager.vue](src/views/tools/data/ScreenshotManager.vue)：全选 + 列表项选择（:checked + @change / @click.stop）
    - [src/views/tools/data/DataExporter.vue](src/views/tools/data/DataExporter.vue)：导出项勾选（v-model）
  - **外层 `<label>` → `<div>`**：Checkbox 根元素是 `<label>`，原代码中多处将 input 包裹在 `<label>` 内实现点击触发，替换后 label 嵌套 label 非法，故将外层 `<label>` 改为 `<div>`，保留 cursor-pointer 等 class
- 复用说明：
  - Checkbox 组件复刻 ArcoDesign 的 checkbox.tsx + style/index.less + icon-check.tsx，未引入第三方依赖
  - 颜色变量复用项目 CSS 变量（var(--color-primary-500) / var(--color-primary-300) 等），跟随主题色变化
  - 勾选图标 SVG 路径直接复用 ArcoDesign 的 IconCheck 组件
- 体验收益：所有复选框视觉统一为 ArcoDesign 风格，hover 有浅色背景反馈，选中有 scale 动画，禁用状态清晰可辨

### 修复

#### port-picker 子窗口刷新体验优化

- 背景：[src-tauri/resources/templates/port-picker.html](src-tauri/resources/templates/port-picker.html) 的刷新按钮尺寸 34x34px 偏大、与 30px 高度的搜索框不协调；打开页面后若后端注入数据为空需等待 3 秒定时器才触发首次 fetchData；点击刷新按钮只有按钮自身 spinning 动画，缺少遮蔽罩+"刷新中..."文字反馈
- 改动：
  - **刷新按钮调小**（[src-tauri/resources/templates/port-picker.html](src-tauri/resources/templates/port-picker.html)）：34x34 → 30x30，与搜索框 30px 高度对齐；按钮内 SVG 显式设为 14x14，避免默认撑满父容器；搜索框 padding 8px 12px → 6px 10px、显式 height 30px；toolbar gap 8px → 6px
  - **首次立即拉取**：渲染后端注入的 `DATA.ports` 后立即调用 `fetchData()`，不再等 3 秒定时器；若 `DATA.ports` 为空则显示遮蔽罩 + "刷新中..."，解决"打开后要等才有数据"问题；DATA 已有数据时静默刷新，确保数据新鲜
  - **刷新遮蔽罩**：新增 `.list-wrap` 容器包裹列表 + 遮蔽罩；遮蔽罩 absolute 覆盖列表区域，半透明 Catppuccin Mocha 背景 + 1px blur + 22x22 蓝色环形 spinner + "刷新中..." 文字；用户点击刷新按钮时显示，fetchData 完成后隐藏；定时器触发的静默刷新不显示遮蔽罩，避免 3 秒一次闪屏
  - **fetchData 参数化**：`fetchData(withOverlay: boolean)`，点击刷新按钮传 `true`，首次空数据传 `true`，定时器传 `false`
- 复用说明：模板仍使用原生 fetch + 后端 `/data` 接口，未引入第三方库；样式沿用 Catppuccin Mocha 暗色主题，与既有 redirect.html / port-picker.html 风格一致
- 体验收益：刷新按钮不再"显得突兀"；打开子窗口立即拉取最新端口列表；点击刷新有明确视觉反馈

#### port-picker 端口列表分组排序折叠 + IP 显示

- 背景：[src-tauri/resources/templates/port-picker.html](src-tauri/resources/templates/port-picker.html) 此前端口列表为纯平铺结构，无法区分 Java 进程与其他进程；用户选择 Minecraft 服务器端口时，Java 进程监听的端口是最常见目标，但被淹没在大量无关端口中；同时列表未显示监听 IP，用户无法区分 127.0.0.1 与 0.0.0.0 等绑定地址
- 改动：
  - **端口列表分组**（[src-tauri/resources/templates/port-picker.html](src-tauri/resources/templates/port-picker.html)）：新增 `groupPorts` 函数将端口分为三组——Java 进程（含 javaw / java.exe 等变体，去掉扩展名后判断）→ 其他有程序名 → 无程序名；每组内按端口号升序排序
  - **默认折叠无程序名组**：Java 进程组与其他进程组默认展开，无程序名组默认折叠，减少视觉噪音；新增 `groupCollapseState` 记录用户手动折叠/展开状态，避免 3 秒定时刷新丢失交互状态
  - **分组标题栏**：新增 `.group-header` 样式，含折叠箭头（SVG，旋转动画 0.15s）、分组标题、端口数量徽标；点击标题栏调用 `toggleGroup` 切换显示
  - **显示监听 IP**：每行新增 `.ip` 元素显示从 `local_addr` 提取的 IP 部分（`extractIp` 用 `lastIndexOf(':')` 兼容 IPv6 `[::]:7000` 形式），等宽字体 11px 灰色，与端口号、协议并列
  - **搜索框 placeholder 更新**：`搜索端口或进程名...` → `搜索端口、IP 或进程名...`，提示用户可搜索 IP
- 复用说明：复用既有 `local_addr` 字段（`OpenPortInfo` 结构体已含此字段），未新增后端接口；分组与折叠逻辑在前端实现，不影响 `/data` 接口契约
- 体验收益：Java 进程（如 Minecraft 服务器）排在最前便于快速定位；无程序名的系统端口默认折叠减少干扰；IP 显示帮助区分绑定地址

#### 隧道创建表单端口选择按钮宽度收窄

- 背景：[src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue) 的本地端口选择按钮此前用 `InputGroup :ratio="[4, 1]"` 让按钮占 1/5 宽度，并配合 `class="w-full"` + `Tooltip block` 让按钮撑满列宽，视觉上按钮过宽与图标按钮的预期不符
- 改动：
  - **改用 flex 布局**（[src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue)）：移除 `InputGroup`，改为 `div.flex.items-center.gap-2`，Input `class="flex-1"` 撑满剩余空间，Button 不传 `w-full` 自适应图标宽度
  - **Tooltip 移除 block**：取消 `block` prop，让 Tooltip 收缩到按钮宽度，避免 tooltip 触发区域过宽
  - **清理未使用 import**：移除 `InputGroup` 的 import，避免 lint 报未使用警告
- 复用说明：与同文件中其他 form 控件（如"本地 IP" 单 Input）的布局风格保持一致，未引入新模式
- 视觉收益：端口选择按钮变为标准 32px 图标按钮，与搜索框对齐协调

#### InputGroup 内 Input 上下堆叠修复

- 背景：[src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue) 的服务器地址与端口使用 `<InputGroup :ratio="[3, 1]">` 布局，但 InputGroup 组件此前在"端口选择按钮宽度收窄"改动中被误删 import，导致 Vue 将 `<InputGroup>` 当作未知元素渲染，内部的两个 Input 失去 grid 容器约束，按默认流布局上下堆叠
- 改动：
  - **恢复 InputGroup import**（[src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue)）：重新添加 `import InputGroup from '@/components/common/InputGroup.vue'`，让组件正常渲染 grid 容器
  - **强化 grid item blockify**（[src/components/common/InputGroup.vue](src/components/common/InputGroup.vue)）：新增 `.input-group :deep(.input-root) { display: block; width: 100%; }`，确保 Input 组件的 `<span class="input-root">` 根元素在 grid 中表现为块级，让 `grid-template-columns` 列宽正确分配
- 复用说明：仅恢复被误删的 import + 补充 :deep 样式，未修改 Input 组件本身
- 体验收益：服务器地址与端口现在能正确按 3:1 比例左右排列

#### qrcode 模板改用本地库 + 依赖版权名单更新

- 背景：qrcode.html 此前通过在线 API（api.qrserver.com）生成二维码，需联网且无法离线使用；同时 about 目录下的依赖版权名单（frontend-deps.txt / backend-deps.txt）缺少近期新增的依赖（Tauri Plugin dialog/process/updater、netstat2、tokio-tungstenite、fastnbt、加密套件等）和嵌入资源（marked.min.js、qrcode.min.js）的版权声明
- 改动：
  - **qrcode 改用本地库**（[src-tauri/resources/templates/qrcode.html](src-tauri/resources/templates/qrcode.html) + [src-tauri/src/resources.rs](src-tauri/src/resources.rs) + [src/config/picker-templates.ts](src/config/picker-templates.ts)）：
    - `resources.rs` 的 `embedded_bytes` 新增 `view/qrcode.min.js` 分支（davidshimjs/qrcodejs 库，DOM 渲染）
    - `qrcode.html` 重写：通过 `res://` 协议动态加载 `view/qrcode.min.js`，调用 `new QRCode(element, {text, width, height, colorDark, colorLight, correctLevel})` 生成二维码，完全离线可用；移除在线 API 调用
    - `picker-templates.ts` 中 qrcode 模板 CSP 更新：`script-src` 新增 `res:` 允许加载本地库；`img-src` 移除 `https:` 不再允许外部图片
  - **前端依赖名单补齐**（[src-tauri/resources/about/frontend-deps.txt](src-tauri/resources/about/frontend-deps.txt)）：新增 `Tauri Plugin Dialog ^2.7.2`、`Tauri Plugin Process ^2.0.0`、`Tauri Plugin Updater ^2.0.0`；新增嵌入资源版权声明 `marked`（Markdown 解析库）和 `qrcodejs`（二维码生成库）
  - **后端依赖名单补齐**（[src-tauri/resources/about/backend-deps.txt](src-tauri/resources/about/backend-deps.txt)）：新增 `Tauri Plugins 2`（官方插件集）、`tokio-tungstenite 0.21`（WebSocket）、`rustls-native-certs 0.6`（根证书）、`serde_json 1.0`、`anyhow 1`、`thiserror 1.0`、`log 0.4`、`env_logger 0.10`、`once_cell 1`、`flate2 1`（gzip）、`fastnbt 2`（NBT 解析）、`netstat2 0.9`（端口枚举）、`hex 0.4`、`base64 0.22`、`encoding_rs 0.8`、`urlencoding 2.1`、`futures-util 0.3`、`Ed25519-Dalek 2`、`X25519-Dalek 2`、`hkdf 0.12`、`hmac 0.12`、`aes-gcm 0.10`、`rsa 0.9`、`rand 0.8`、`pem 3`、`tun-rs 2`（虚拟网卡）、`winreg 0.52`（注册表）
- 复用说明：
  - qrcode.html 的 `res://` 加载逻辑与 markdown.html 完全一致（动态判断 protocol 构造 URL）
  - qrcodejs 库的 DOM 渲染 API 直接使用，无需封装
- 离线收益：qrcode 模板不再依赖网络，可在无网环境下生成二维码

### 新增

#### Picker 子窗口多模板 + CSP 策略 + 便捷调用函数

- 背景：picker 子窗口此前仅 `port-picker` 和 `redirect` 两个模板，用户需要更多示例模板（确认框、信息展示、图片查看、Markdown 渲染、二维码）；同时白名单是前端 JS 校验，无法限制子窗口内可加载的资源范围；白名单域名错误（`moiteam.cn` 应为 `moteam.top`）
- 改动：
  - **白名单域名修正**（[src/config/picker-templates.ts](src/config/picker-templates.ts)）：`*.moiteam.cn` → `moteam.top` + `*.moteam.top` + `*.molaunch.moiu.cn`，覆盖项目实际使用的服务域名
  - **5 个新模板**（新增 [src-tauri/resources/templates/](src-tauri/resources/templates/)）：
    - `confirm.html`：确认对话框，标题 + 消息 + 确认/取消按钮，支持 danger 红色样式，点击返回 `true`/`false`，支持 Enter/Esc 快捷键
    - `info.html`：信息展示，标题 + 正文（支持 b/i/code/br/p 简单 HTML），自动转义其他标签
    - `image-viewer.html`：图片查看器，工具栏（缩小/重置/放大）+ 滚轮缩放 + 拖拽平移，居中显示
    - `markdown.html`：Markdown 渲染，通过 `res://` 协议加载后端嵌入的 `marked.min.js`，GFM + breaks 启用，暗色主题样式
    - `qrcode.html`：二维码展示，通过在线 API 生成 240x240 二维码，显示标签和原文预览
  - **CSP 策略化**（[src/config/picker-templates.ts](src/config/picker-templates.ts) + [src-tauri/src/commands/tools/picker_window.rs](src-tauri/src/commands/tools/picker_window.rs) + [src-tauri/src/commands/tools/types.rs](src-tauri/src/commands/tools/types.rs)）：
    - 前端：每个模板配置 `csp` 字段（Content-Security-Policy），定义 `BASE_CSP` 通用策略 + 各模板特化策略（image-viewer 扩展 img-src https/http；markdown 扩展 script-src res:；qrcode 扩展 img-src https:）
    - 后端：`OpenPickerWindowParams` 新增 `csp: Option<String>` 字段；`picker_window.rs` 新增 `PICKER_CSP_STORE` 存储 CSP；`build_response` 函数将 CSP 注入 HTTP 响应头 `Content-Security-Policy`
    - 前端：`PickerWindowParams` 接口新增 `csp?: string`；`openPickerWindow`/`openDisplayWindow` 自动从模板配置读取 CSP，调用方可覆盖
  - **便捷调用函数**（[src/utils/picker-window.ts](src/utils/picker-window.ts)）：
    - 新增 `openDisplayWindow(params)`：展示型窗口基类，用户关窗即 resolve（适用于无返回值的模板）
    - 新增 `openConfirmWindow({title?, message, confirmText?, cancelText?, danger?})`：返回 `Promise<boolean>`
    - 新增 `openInfoWindow({title, content})`：信息展示
    - 新增 `openImageViewerWindow({url, alt?})`：图片查看
    - 新增 `openMarkdownWindow({title, content})`：Markdown 渲染
    - 新增 `openQrcodeWindow({text, label?})`：二维码展示
    - `openRedirectWindow` 重构为调用 `openDisplayWindow`
  - **资源注册**（[src-tauri/src/resources.rs](src-tauri/src/resources.rs)）：`embedded_text` 新增 5 个模板分支（confirm/info/image-viewer/markdown/qrcode）；`embedded_bytes` 新增 `view/marked.min.js` 分支（供 markdown 模板通过 `res://` 协议加载）
  - **后端通用化模板读取**（[src-tauri/src/commands/tools/picker_window.rs](src-tauri/src/commands/tools/picker_window.rs)）：URI scheme handler 不再写死 `port-picker`/`redirect` 分支，统一从 `templates/<name>.html` 读取模板；port-picker 的 `/data` 请求保留特殊处理返回实时端口列表；新增 `cleanup_picker_stores` 函数统一清理模板/数据/CSP 存储
- 复用说明：
  - 所有模板复用 Catppuccin Mocha 暗色主题风格，与 redirect.html/port-picker.html 一致
  - 所有模板复用 `window.__PICKER_DATA__` 注入约定和右键/DevTools 快捷键禁用逻辑
  - 便捷函数复用 `openPickerWindow`/`openDisplayWindow` 基类，避免重复事件监听/invoke 逻辑
  - CSP 配置集中在 `picker-templates.ts`，修改策略只需改配置文件
- 安全收益：CSP 通过 HTTP 响应头注入，浏览器级别限制子窗口可加载的资源范围，防止模板被注入恶意资源；`script-src res:` 仅允许 markdown 模板从 `res://` 协议加载 marked.min.js

#### Picker 模板配置文件 + 重定向子窗口便捷函数

- 背景：picker 子窗口的模板默认参数（标题、尺寸）与重定向白名单此前无集中管理位置，新增重定向场景需改逻辑代码且白名单校验散落。本次将模板配置下沉到独立配置文件，并提供 `openRedirectWindow` 便捷函数封装白名单校验 + 窗口创建
- 改动：
  - **模板配置文件**（新增 [src/config/picker-templates.ts](src/config/picker-templates.ts)）：定义 `PickerTemplateConfig` 接口与 `PICKER_TEMPLATES` 配置表（含 `port-picker`、`redirect` 两个模板的默认标题/尺寸/白名单）；提供 `isUrlAllowed(url, allowedDomains)` 校验函数（支持精确匹配与 `*.example.com` 通配符前缀）与 `getTemplateConfig(template)` 读取函数
  - **重定向便捷函数**（[src/utils/picker-window.ts](src/utils/picker-window.ts)）：新增 `openRedirectWindow(url)`，先从配置表读取 `redirect` 模板配置，调用 `isUrlAllowed` 校验 URL 域名白名单，校验通过后复用同文件 `openPickerWindow` 创建子窗口；返回 `Promise<void>`（重定向窗口无需返回值）
- 复用说明：
  - `openRedirectWindow` 直接复用同文件既有 `openPickerWindow`，未重复事件监听/invoke 逻辑
  - 白名单与默认尺寸集中在配置文件，修改白名单只需改 `PICKER_TEMPLATES`，不需动 `picker-window.ts`
- 约束遵循：配置文件独立放在 `src/config/` 目录，不与逻辑文件混用；无 emoji；遵循最小修改原则（仅新增 1 个配置文件 + 1 个函数 + 1 行 import）

#### Picker 子窗口重定向模板（后端）+ 安全措施

- 背景：picker 子窗口此前仅支持 port-picker 模板，缺少通用的重定向页面；同时子窗口未禁用右键菜单与 DevTools 快捷键，存在调试与查看源码入口。本次新增 redirect 模板并补充安全措施
- 改动：
  - **重定向模板**（新增 [src-tauri/resources/templates/redirect.html](src-tauri/resources/templates/redirect.html)）：Catppuccin Mocha 暗色主题，居中显示 spinner + "正在跳转..." + 目标 URL；JS 从 `window.__PICKER_DATA__` 读取 `{ url }`，1 秒后自动 `location.href = url` 跳转，无效 URL 显示错误提示
  - **安全措施**（[src-tauri/resources/templates/port-picker.html](src-tauri/resources/templates/port-picker.html) + redirect.html）：`<body oncontextmenu="return false">` 禁用右键；JS 拦截 F12、Ctrl+Shift+I/J/C、Ctrl+U 快捷键禁用 DevTools 入口
  - **资源注册**（[src-tauri/src/resources.rs](src-tauri/src/resources.rs)）：`embedded_text` 新增 `templates/redirect.html` 分支
  - **数据存储**（[src-tauri/src/commands/tools/picker_window.rs](src-tauri/src/commands/tools/picker_window.rs)）：新增 `PICKER_DATA_STORE`（picker_id → data JSON），`open_picker_window` 存储前端传入的 `data`，`on_navigation` 与 `on_window_event` 清理时同步移除；URI scheme handler 新增 `redirect` 模板分支，读取存储的 data 注入模板
  - **禁用 DevTools**（[src-tauri/src/commands/tools/picker_window.rs](src-tauri/src/commands/tools/picker_window.rs)）：`WebviewWindowBuilder` 链式调用添加 `.devtools(false)`，子窗口级别关闭 DevTools
- 复用说明：
  - redirect 模板复用 port-picker 的配色与 `window.__PICKER_DATA__` 注入约定
  - 模板读取复用 `crate::resources::read_resource`，与所有文本资源读取一致
  - 数据存储复用 `PICKER_TEMPLATES` 的 `Lazy<Mutex<HashMap>>` 模式与清理逻辑
- 安全收益：子窗口禁用右键与 DevTools 快捷键，防止用户查看模板源码与调试；`.devtools(false)` 在窗口级别关闭 DevTools

### 重构

#### Picker 子窗口模板化（后端 resources + 实时刷新端口列表）

- 背景：原 picker 子窗口由前端传入完整 HTML 字符串，后端原样存储返回，存在前端注入风险且端口列表无法实时刷新。本次将 HTML 模板下沉到后端 resources，并支持端口列表定时刷新
- 改动：
  - **HTML 模板**（新增 [src-tauri/resources/templates/port-picker.html](src-tauri/resources/templates/port-picker.html)）：自包含暗色 Catppuccin Mocha 页面，搜索框 + 刷新按钮工具栏，端口列表点击导航 `picker-result://?value=<port>`；JS 从 `window.__PICKER_DATA__` 读取初始数据，定时（3 秒）fetch `./data` 实时刷新列表不刷新页面，搜索框实时筛选，刷新按钮手动触发，空状态 icon + text 垂直水平居中
  - **资源注册**（[src-tauri/src/resources.rs](src-tauri/src/resources.rs)）：`embedded_text` 新增 `templates/port-picker.html` 分支，编译时 include_str! 嵌入，运行时零文件 IO
  - **同步端口枚举**（[src-tauri/src/commands/tools/network.rs](src-tauri/src/commands/tools/network.rs)）：新增 `list_open_ports_sync()` 提取 netstat2 + sysinfo 枚举逻辑，返回 `Vec<OpenPortInfo>`；`list_open_ports` 重构为调用本函数后序列化，消除逻辑重复
  - **picker_window 重构**（[src-tauri/src/commands/tools/picker_window.rs](src-tauri/src/commands/tools/picker_window.rs)）：移除 `PICKER_HTML_STORE`，新增 `PICKER_TEMPLATES`（picker_id → template_name）；`open_picker_window` 存储模板名而非 HTML；URI scheme handler 按模板名分发——`port-picker` 模板实时调用 `list_open_ports_sync()`，`/data` 请求返回 JSON，页面请求返回模板 + 注入初始数据；`extract_picker_id` 改为查找以 `picker-` 开头的路径段以正确处理 `/data` 后缀
  - **类型更新**（[src-tauri/src/commands/tools/types.rs](src-tauri/src/commands/tools/types.rs)）：`OpenPickerWindowParams` 移除 `html` 字段，改为 `template: String` + `#[serde(default)] data: serde_json::Value`
- 复用说明：
  - `list_open_ports_sync` 复用原 `list_open_ports` 的 netstat2 + sysinfo 枚举逻辑，async 版本改为薄封装避免重复
  - 模板读取复用 `crate::resources::read_resource`，与所有文本资源读取一致
  - URI scheme 注册模式与 `res_scheme.rs` / 原 picker scheme 一致
- 安全收益：HTML 模板由后端控制，前端无法注入任意 HTML/JS；端口数据由后端实时生成，前端仅传模板名
- 约束遵循：遵循 `log_info!` 宏约定；无 emoji；空状态 icon + text 居中

#### Picker 子窗口模板化（前端适配）

- 背景：picker 子窗口此前的 HTML 由前端 `generatePortPickerHtml` 生成并整段传给后端，HTML 模板与转义逻辑散落在前端工具文件中。本次将 HTML 模板迁移至后端 resources，前端只传 `template` 名称与 `data`，由后端加载模板并注入数据，统一模板管理职责
- 改动：
  - **接口变更**（[src/utils/picker-window.ts](src/utils/picker-window.ts)）：`PickerWindowParams` 从 `{ title, html, width?, height? }` 改为 `{ title, template, data?, width?, height? }`；`openPickerWindow` 内部 invoke 调用直接透传 params，自动携带新字段；同步更新文件顶部用法文档
  - **删除前端 HTML 生成器**：移除 [src/utils/frp-port-picker.ts](src/utils/frp-port-picker.ts)（`generatePortPickerHtml` 及 HTML/属性转义逻辑已迁至后端 resources 模板）
  - **表单适配**（[src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue)）：`handleSelectPort` 不再调用 `listOpenPorts` + `generatePortPickerHtml`，改为 `openPickerWindow({ template: 'port-picker', data: {} })`；移除 `generatePortPickerHtml` 导入与 `listOpenPorts` 导入（`tcpCheck` 保留）
  - **修复按钮布局对齐**：本地端口选择区原用 `flex gap-1.5` + `flex-1` 手动布局，Button 宽度随内容（图标 vs spinner）变化导致加载/非加载状态宽度不一致；改用 `InputGroup :ratio="[4, 1]"` 复用既有 Grid 布局组件，Tooltip 加 `block` prop 让 trigger 填满 grid 列宽，Button 加 `w-full` 填满 trigger，确保两种状态下宽度一致
- 复用说明：
  - 复用既有 `InputGroup` 组件（已在服务器地址+端口处使用），未新增布局组件
  - 复用 Tooltip 的 `block` prop（组件既有的"撑满父容器"开关），未新增样式
- 约束遵循：TunnelCreateForm.vue 273 行 ≤ 300；遵循自定义组件约定（InputGroup/Tooltip/Button/Input）；无 emoji；最小修改原则

### 增强

#### 选择器子窗口工具 + Frp 端口选择器（前端 UI）

- 背景：Frp 隧道表单此前的本机端口选择为内联下拉浮层（点击外部关闭、手动维护 openPorts/portPanelRef 等状态），与子窗口选择方案相比交互受限且占用组件行数。本次引入通用 picker 子窗口工具替换内联下拉
- 改动：
  - **通用 picker 工具**（新增 [src/utils/picker-window.ts](src/utils/picker-window.ts)）：封装 `openPickerWindow({ title, html, width?, height? })`，内部先 await 注册 `picker-result` / `picker-cancelled` 事件监听拿到 unlisten 句柄，再调用 `tools_manager` 的 `open_picker_window` action 创建子窗口；用户点击选项 → `picker-result` 事件 → resolve(value)；用户关闭窗口 → `picker-cancelled` 事件 → reject；invoke 失败时完整清理已注册监听器，参照 `useTauriEvent.ts` 的 async/await 模式避免 listener 泄漏
  - **端口选择器 HTML 生成器**（新增 [src/utils/frp-port-picker.ts](src/utils/frp-port-picker.ts)）：`generatePortPickerHtml(ports)` 生成自包含 HTML（暗色 Catppuccin Mocha 配色），含搜索框 + 端口列表；点击项导航到 `picker-result://?value=<port>` 由后端 on_navigation 拦截；HTML/属性转义防止进程名注入
  - **action 注册**（[src/utils/api/tools.ts](src/utils/api/tools.ts)）：`TOOLS_ACTIONS` 新增 `OPEN_PICKER_WINDOW: 'open_picker_window'`，与后端分发器对齐
  - **表单集成**（[src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue)）：移除内联下拉相关状态（showPortPanel/openPorts/openPortsLoading/portPanelRef）与函数（togglePortPanel/loadOpenPorts/selectPort/handlePortClickOutside）及 onMounted/onUnmounted 的 click 外部监听；新增 `portSelecting` ref 与 `handleSelectPort` 函数，按钮加 `:loading="portSelecting"`；修正 `OpenPortInfo` 类型原误从 `@/types/frp` 导入（实际未导出）的问题
- 复用说明：
  - 完全复用 `listOpenPorts` / `tcpCheck`（@/utils/api/tools），未重复端口枚举逻辑
  - picker 工具参照 `useTauriEvent.ts` 的 unlisten 句柄管理模式，与项目既有事件监听风格一致
  - Button 的 `:loading` prop 复用项目既有约定（加载时图标槽位自动替换为 spinner）
  - 端口项点击导航协议 `picker-result://` 与后端 on_navigation 拦截契约对齐
- 约束遵循：Vue 组件 272 行 ≤ 300；仅新增 2 个工具文件，未新建组件；遵循自定义组件约定（Button/Tooltip/Input/Select）；无 emoji

#### 选择器子窗口工具（后端 Rust）

- 背景：前端 picker 工具已就绪（`openPickerWindow` + `picker-result://` 导航协议），但后端缺少对应的 `open_picker_window` IPC action 与 `picker://` URI scheme 注册，子窗口无法实际创建和渲染 HTML
- 改动：
  - **picker_window 模块**（新增 [src-tauri/src/commands/tools/picker_window.rs](src-tauri/src/commands/tools/picker_window.rs)）：`open_picker_window(app, params)` 生成唯一 picker ID（时间戳+原子计数器），存储 HTML 到全局 `PICKER_HTML_STORE`，通过 `WebviewWindowBuilder` + `WebviewUrl::CustomProtocol` 加载 `picker://localhost/<id>`；`on_navigation` 拦截 `picker-result://?value=XXX` 导航，emit `picker-result` 事件后关闭窗口；`on_window_event` 监听 `Destroyed`，用户未选择关窗时 emit `picker-cancelled` 事件
  - **URI scheme 注册**（`register_picker_scheme`）：注册 `picker://` 自定义协议，从 URL 路径提取 picker_id，从全局存储取出 HTML 返回（兼容 Windows `https://picker.localhost/` 转换）
  - **类型定义**（[src-tauri/src/commands/tools/types.rs](src-tauri/src/commands/tools/types.rs)）：新增 `OpenPickerWindowParams`（title/html/width?/height?，camelCase 反序列化），遵循项目类型集中定义约定
  - **action 注册**（[src-tauri/src/commands/tools/mod.rs](src-tauri/src/commands/tools/mod.rs)）：DISPATCHER 新增 `open_picker_window` action，位于 `list_open_ports` 之后
  - **scheme 注册**（[src-tauri/src/lib.rs](src-tauri/src/lib.rs)）：在 `res_scheme::register_res_scheme` 之后调用 `register_picker_scheme`
- 复用说明：
  - `register_uri_scheme_protocol` 模式参照 `res_scheme.rs` 和 `minecraft/image_cache.rs`
  - `WebviewWindowBuilder` + `on_navigation` 模式参照 `commands/auth/microsoft.rs`
  - `handler!` 宏 + Dispatcher 注册模式与现有 25 个 tools action 完全一致
  - 类型定义放在 `types.rs`（与所有其他工具参数类型一致），子模块通过 `use super::types::` 导入
- 防重复设计：`PICKER_COMPLETED` 全局集合标记已完成选择的 picker ID，避免 `on_navigation` emit `picker-result` 后窗口 `Destroyed` 再重复 emit `picker-cancelled`
- 约束遵循：Rust 模块 207 行；遵循 `log_info!` 日志宏约定；无 emoji

#### Frp 隧道自检面板集成（前端 UI）

- 背景：`TunnelSelfCheck.vue` 组件与 `frp-tunnel-check.ts` 工具已就绪，但 `TunnelManager.vue` 仅导入未实际挂载，自检入口缺失。本次补全集成
- 改动：
  - **自检入口**（[src/components/frp/TunnelManager.vue](src/components/frp/TunnelManager.vue)）：顶部操作栏新增「隧道自检」按钮（ShieldCheckIcon，位于刷新按钮左侧），点击切换 `showSelfCheck` 控制面板展开/收起
  - **自检面板**：编辑表单与隧道列表之间插入 `Transition` + `TunnelSelfCheck` 组件，复用既有展开动画（透明度+缩放+位移），传入 `tunnels` / `providers` props，`@close` 收起面板
  - **行数控制**：将 `handleRefresh` / `handleViewLogs` 压缩为单行，腾出行数空间；TunnelManager.vue 278 行 ≤ 300
- 复用说明：
  - 完全复用 `TunnelSelfCheck.vue` 组件（props/emits 已定义），未重复自检逻辑
  - 动画 class 与创建/编辑表单 Transition 完全一致，保持视觉统一
  - Button / Tooltip / heroicons 均复用项目既有约定
- 约束遵循：仅修改 TunnelManager.vue 一个文件，未新建文件；遵循 300 行约束与自定义组件约定

#### Frp 隧道编辑模式 + 本机开放端口选择（前端 UI）

- 背景：此前仅有创建隧道入口，修改名称/端口/token 等需删除重建；本机端口需手动查找。本次实现编辑隧道配置 UI 并集成 list_open_ports 工具到创建/编辑表单
- 改动：
  - **composable 抽离**（新增 [src/composables/usePublicServers.ts](src/composables/usePublicServers.ts)）：将 TunnelCreateForm.vue 的公共服务器逻辑（publicServers ref / loadPublicServers / handlePublicServerChange / publicServerOptions computed）抽至独立 composable，form 由调用方传入，避免组件超 300 行约束
  - **编辑模式**（[src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue)）：新增 `editTunnel?: Tunnel` prop 与 `update` emit；onMounted 预填表单（mode 固定 self）；编辑模式隐藏服务器模式切换；提交按钮文案切换「保存」/「创建」；handleSubmit 按 isEdit 分流 emit update/create
  - **本机端口选择**（[src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue)）：本地端口输入框旁加 ServerStackIcon 按钮，点击调用 `listOpenPorts()` 拉取本机监听端口，下拉浮层展示端口号+协议+进程名，点击自动填入 localPort；加载中 spinner，失败 toast 提示，点击外部关闭
  - **store action**（[src/stores/frp.ts](src/stores/frp.ts)）：新增 `updateTunnel(params)` action，完全参照 createTunnel 风格（tunnelActionLoading + toastSuccess + loadTunnels 刷新）；补充 `apiUpdateTunnel` 与 `UpdateTunnelParams` 导入
  - **编辑入口**（[src/components/frp/TunnelManager.vue](src/components/frp/TunnelManager.vue)）：每条隧道卡片操作区加「编辑配置」按钮（PencilIcon，位于启动/停止与查看日志之间）；运行中隧道点击编辑时 toastWarning 提示「请先停止隧道再编辑」；编辑表单用独立 Transition 包裹，保存成功后自动收起
- 复用说明：
  - 公共服务器逻辑通过 composable 抽离复用，零逻辑重复
  - 编辑表单完全复用 TunnelCreateForm.vue 组件（通过 editTunnel prop 切换模式），未新建独立编辑组件
  - updateTunnel action 复用 createTunnel 的 loading/toast/刷新模式
  - 本机端口下拉样式参考 Select.vue（白底、gray 边框、圆角、阴影）
  - toastWarning / showConfirm / Button / Tooltip / Input 等均复用项目既有组件与工具
- 验证：`npx tsc --noEmit --skipLibCheck` 与 `npx vue-tsc --noEmit --skipLibCheck` 过滤 TunnelCreateForm/TunnelManager/stores/frp/usePublicServers 零错误；TunnelCreateForm.vue 272 行、TunnelManager.vue 271 行均 ≤ 300
- 约束遵循：仅新增 1 个 composable 文件（usePublicServers.ts，因组件超 300 行约束必须抽取）；遵循项目自定义组件约定（Button/Input/Select/Tooltip）、heroicons 无 emoji、单列布局风格

#### 列出本机监听端口（list_open_ports 工具，供 Frp 内网端口选择）

- 背景：Frp 创建隧道时需要选择本机内网端口进行映射，此前缺少枚举本机监听端口的工具，用户需手动查端口。新增 `list_open_ports` action 返回所有 LISTEN 状态的 TCP 端口与全部 UDP 端口（含占用进程信息），供前端 Frp 隧道配置复用
- 改动：
  - **依赖**（[src-tauri/Cargo.toml](src-tauri/Cargo.toml)）：新增 `netstat2 = "0.9"`（跨平台枚举网络套接字，使用 OS 底层 API 而非命令行工具）
  - **后端类型**（[src-tauri/src/commands/tools/types.rs](src-tauri/src/commands/tools/types.rs)）：新增 `OpenPortInfo`（local_addr / port / protocol / process_name / pid）与 `ListOpenPortsResult`
  - **后端实现**（[src-tauri/src/commands/tools/network.rs](src-tauri/src/commands/tools/network.rs)）：新增 `list_open_ports(state)` 函数，用 `netstat2::get_sockets_info` 枚举 IPv4/IPv6 的 TCP+UDP 套接字，筛选 TCP `Listen` 状态（UDP 无状态全部视为监听），通过 `sysinfo::System::new_all()` + `Pid::from_u32` 查进程名；按 port 升序排序并按 (port, protocol, local_addr) 去重；与 `tcp_check` / `server_ping` 风格一致（读 game_dir、log_info!、返回 `serde_json::Value`）
  - **action 注册**（[src-tauri/src/commands/tools/mod.rs](src-tauri/src/commands/tools/mod.rs)）：`tcp_check` 之后、NBT 之前注册 `list_open_ports`，走 `handler!` 宏无参数模式
  - **前端 API**（[src/utils/api/tools.ts](src/utils/api/tools.ts)）：`TOOLS_ACTIONS` 加 `LIST_OPEN_PORTS`；`tcpCheck` 之后新增 `OpenPortInfo` / `ListOpenPortsResult` 接口与 `listOpenPorts()` 封装
- 复用说明：
  - netstat2 0.9 实际 API 为 `get_sockets_info(AddressFamilyFlags, ProtocolFlags)` 返回 `SocketInfo`（非任务描述中假设的 `get_active_connections`/`ConnectionInfo`），`ProtocolSocketInfo::Tcp(TcpSocketInfo)` 为元组变体且 `local_addr: IpAddr` + `local_port: u16` 分离字段，已按实际 API 实现
  - sysinfo 0.29 需导入 `SystemExt` / `ProcessExt` / `PidExt` 三个 trait 才能调用 `new_all` / `process` / `name` / `from_u32`
  - 注册与封装模式完全复用 `tcp_check` 的既有约定（handler! 宏、toolsManager 泛型封装）
- 验证：`cargo check` 编译通过零错误零警告
- 约束遵循：未创建新文件，未修改 Vue 组件；遵循 log_info! 宏、Result<T, String> 错误处理、handler! 注册约定
- 待实测（前端 UI 由其他 agent 处理）：① 调用返回本机监听端口列表；② 进程名解析正确；③ TCP/UDP 端口均能枚举

#### Frp 编辑隧道配置（update_tunnel 后端 + 前端 API 封装）

- 背景：原仅有创建/删除隧道能力，缺少编辑已有隧道配置的入口；用户修改名称、端口、token、TLS 等需删除重建。新增 `update_tunnel` action 支持就地编辑并重新生成 frpc TOML
- 改动：
  - **后端 update_tunnel**（[src-tauri/src/commands/frp/tunnel.rs](src-tauri/src/commands/frp/tunnel.rs)）：新增 `update_tunnel(params)` 函数，校验隧道存在 + 名称唯一性（排除自身）后更新 `tunnels.json` 并调用现有 `generate_config` 覆盖重生成 frpc TOML；新增 `UpdateTunnelParams` 结构体（字段同 `CreateTunnelParams` + `id`，`serde rename_all = "camelCase"`）。注意：名称唯一性校验须在 `iter_mut().find()` 之前执行，否则可变借用与 `iter().any()` 不可变借用冲突（任务原始代码片段的固有借用问题，已修正顺序）
  - **沙箱校验复用**（[src-tauri/src/commands/frp/sandbox.rs](src-tauri/src/commands/frp/sandbox.rs)）：`validate_tunnel` 仅接受 `&CreateTunnelParams`；新增 `validate_tunnel_update(p: &UpdateTunnelParams)` 将其转换为 `CreateTunnelParams` 后委托 `validate_tunnel`，零逻辑重复，校验规则（厂商 ID/名称/地址/端口/Token/类型）完全复用
  - **action 注册**（[src-tauri/src/utils/frp_manager.rs](src-tauri/src/utils/frp_manager.rs)）：`UpdateTunnelParams` 加入 import；`delete_tunnel` 注册之后、frpc 进程管理之前注册 `update_tunnel`，走 `handler!` 宏 + `frp::sandbox::validate_tunnel_update` 校验 + `frp::tunnel::update_tunnel` 调用，与 `create_tunnel` 模式一致
  - **前端类型**（[src/types/frp.ts](src/types/frp.ts)）：`CreateTunnelParams` 之后新增 `UpdateTunnelParams` 接口（多 `id` 字段）
  - **前端 API 封装**（[src/utils/api/frp-manager.ts](src/utils/api/frp-manager.ts)）：`FRP_ACTIONS` 加 `UPDATE_TUNNEL: 'update_tunnel'`；`createTunnel` 之后新增 `updateTunnel(params): Promise<Tunnel>`，import 补 `UpdateTunnelParams`
- 复用说明：
  - 校验逻辑复用 `validate_tunnel`（通过转换委托，未复制任何校验规则）
  - 配置生成复用现有 `generate_config` / `build_frpc_toml`
  - 持久化复用 `read_tunnels` / `write_tunnels`
  - 注册模式复用 `handler!` 宏，与 `create_tunnel` / `delete_tunnel` 完全一致
- 验证：`cargo check` 我修改的 3 个 Rust 文件（tunnel.rs / sandbox.rs / frp_manager.rs）零错误；剩余编译错误均位于未触碰的 `network.rs`（先前会话遗留的 sysinfo/netstat2 版本 API 问题，与本次改动无关）
- 约束遵循：未创建新文件，未修改 Vue 组件 / stores/frp.ts；遵循 `log_info!` 宏、`camelCase` serde、`handler!` 注册约定
- 待实测（前端 UI 由其他 agent 处理）：① 编辑隧道后 tunnels.json 正确更新；② frpc TOML 覆盖重生成；③ 名称重复时拒绝；④ 隧道运行中编辑（调用方应先停止）

#### Frp 穿透管理体验改进（状态同步 + 日志诊断 + 翻译 + 跳转 + 动画）

- 背景：用户反馈 6 个问题：① 隧道异常退出后列表仍显示"运行中"；② 选择"全部隧道"不返回日志；③ frpc 日志全是英文难以理解；④ 缺少退出原因诊断；⑤ 穿透管理列表无刷新按钮；⑥ 想从隧道卡片一键跳转查看日志
- 改动：
  - **隧道状态同步**（[src/stores/frp.ts](src/stores/frp.ts)）：新增 `startTunnelStatusListener` 监听 `frp-tunnel-status` Tauri event，frpc 进程退出时自动静默刷新 `tunnels` 列表（`refreshTunnelsSilent`，不触发 loading 避免抖动）；异常退出（带 error 字段）时弹 toast 提示；[TunnelManager.vue](src/components/frp/TunnelManager.vue) onMounted 启动监听器
  - **全部隧道日志合并**（[src-tauri/src/commands/frp/process.rs](src-tauri/src/commands/frp/process.rs)）：`read_log_file` 当 `tunnel_id` 为空时调用新增 `read_all_logs` 合并所有日志文件，按行内时间戳排序（支持 `[HH:MM:SS.ms]` / `[YYYY-MM-DD HH:MM:SS.ms]` / frpc 原生格式三种时间戳），限 500 行
  - **日志诊断面板**（新增 [src/utils/frp-log-diagnose.ts](src/utils/frp-log-diagnose.ts)）：基于关键词模式匹配分析退出原因，覆盖网络层（超时/拒绝/DNS）/鉴权层（token 错误/超时）/配置层（端口占用/配置错误）/服务端（协议不匹配）5 类场景；[FrpLogs.vue](src/components/frp/FrpLogs.vue) 顶部显示诊断卡片（标题+类别徽章+详情+建议+关键日志证据），异常退出时自动展开
  - **中文翻译**（新增 [src/utils/frp-log-translate.ts](src/utils/frp-log-translate.ts)）：30+ 条翻译规则覆盖 frpc 常见日志关键词（start frpc service / try to connect / i/o timeout / login failed 等），长短语优先匹配避免重复翻译；[FrpLogs.vue](src/components/frp/FrpLogs.vue) 加翻译开关按钮，开启后日志行尾追加 `｜ 中文释义`
  - **刷新按钮**（[src/components/frp/TunnelManager.vue](src/components/frp/TunnelManager.vue)）：顶部操作栏加刷新按钮，触发 `loadTunnels` 重新拉取列表
  - **查看日志按钮**（[src/components/frp/TunnelManager.vue](src/components/frp/TunnelManager.vue)）：每条隧道卡片加「查看日志」按钮，点击通过 `inject('goToLogs')` 调用 [Online.vue](src/views/Online.vue) provide 的 `goToLogs` 函数，切换到 logs 分类并预选 tunnelId
  - **创建表单动画**（[src/components/frp/TunnelManager.vue](src/components/frp/TunnelManager.vue)）：Transition 包裹表单，展开/收起带透明度+scale-y+位移过渡；TransitionGroup 包裹隧道列表，新增/删除带平移过渡；状态徽章加脉冲动画点（运行中绿色闪烁）
- 复用说明：
  - 事件监听复用 `useTauriEvent` composable（自动 onUnmounted unlisten）
  - 日志颜色复用 `logLineClass`（项目约定）
  - 跳转用 provide/inject 而非 props，避免 keep-alive 缓存组件的层级耦合
  - 翻译和诊断规则独立在 utils/ 下，便于维护扩展
- 验证：`cargo check` 编译通过；`tsc --noEmit` 无 frp 相关错误（仅 online.ts/crypto.ts 原有问题）
- 待实测：① 启动一个会失败的隧道（错误地址）验证状态自动同步 + 诊断面板；② 选「全部隧道」验证日志合并；③ 翻译开关；④ 刷新按钮；⑤ 查看日志跳转

#### Frp 创建隧道增强（公共服务器模式 + 地址端口并列 + 自动连通性检测）

- 背景：阶段二遗留的「官方公共服务器」UI 对接 + 用户希望服务器地址输入后 3 秒自动检测可连接性，且地址和端口支持并列输入调整占比
- 改动：
  - **TCP 连通性检测后端**（[src-tauri/src/commands/tools/network.rs](src-tauri/src/commands/tools/network.rs)）：新增 `tcp_check` action，仅做 TCP 三次握手（3 秒超时），不发送应用层数据，适用于 Frp 等非 Minecraft 协议服务；不复用 `server_ping`（SLP 协议对 Frp 端口会卡 5 秒超时）；配套类型 `TcpCheckParams` / `TcpCheckResult` 加在 [types.rs](src-tauri/src/commands/tools/types.rs)，[mod.rs](src-tauri/src/commands/tools/mod.rs) 注册 action
  - **前端 API 封装**（[src/utils/api/tools.ts](src/utils/api/tools.ts)）：新增 `tcpCheck(host, port)` 与 `TcpCheckResult` 类型
  - **创建表单抽出**（新增 [src/components/frp/TunnelCreateForm.vue](src/components/frp/TunnelCreateForm.vue)，[TunnelManager.vue](src/components/frp/TunnelManager.vue) 回归 228 行）：原内联表单抽为独立组件（263 行），主文件仅负责列表展示与操作，符合 Vue 组件 ≤300 行约束
  - **模式切换**：表单顶部加「用户自备服务器 / 官方公共服务器」下拉；官方模式调用 `listPublicServers` 拉取公共服务器列表（显示名称/区域/负载/在线人数），选择后调 `allocatePublicServer` 自动分配端口 + per-user token，回填 serverAddr/serverPort/remotePort/token/useTls（字段只读）
  - **地址端口并列**：新增可复用公共组件 [InputGroup.vue](src/components/common/InputGroup.vue)（基于 CSS Grid，`ratio` prop 控制各列占比，`gap` 控制列间距，支持任意数量子项），自备模式下服务器地址与端口用 `:ratio="[3, 1]"` 并列（3:1 占比），全项目其他表单可直接复用
  - **3 秒自动检测**：自备模式下监听 serverAddr / serverPort 变化，3 秒无操作自动调 `tcpCheck`，输入下方显示「可连接（Nms）」/「不可连接：原因」/「检测中...」；用 `checkSeq` 序号过滤过期请求避免竞态
- 复用说明：
  - 公共服务器接口复用 `listPublicServers` / `allocatePublicServer`（[frp-manager.ts](src/utils/api/frp-manager.ts) 已有封装）
  - 表单组件用项目自定义 `Input` / `Select` / `Button`，原生 checkbox 因项目无自定义 Checkbox 组件保持与原代码一致
  - 父子通信用 props + emit，父组件 v-if 控制挂载实现自然重置，无需手动 resetForm
- 验证：`cargo check` 编译通过；`tsc --noEmit --skipLibCheck` 无 frp/tools 相关错误
- 待实测：① 切换官方模式验证公共服务器列表加载与分配回填；② 自备模式输入地址后 3 秒验证连通性检测；③ 地址端口并列占比显示

### 修复

#### Frp frpc ZIP 提取改为跨平台自探测 + DownloadManager 集成

- 背景：用户反馈 `extract_frpc_from_zip` 仅匹配当前平台文件名（Windows=frpc.exe，macOS/Linux=frpc），若 apiServer 返回的 ZIP 内文件名与当前平台不一致（如 macOS ZIP 内是 `frpc` 无后缀，或嵌套层级不固定），会导致提取失败；同时 frpc 下载此前使用裸 `reqwest::get()` 无进度反馈、不支持暂停/取消
- 改动：
  - **跨平台自探测**（[src-tauri/src/commands/frp/binary.rs](src-tauri/src/commands/frp/binary.rs)）：`extract_frpc_from_zip` 不再仅匹配 `frpc_filename()` 单一文件名，改为同时匹配 `basename == "frpc" || basename == "frpc.exe"`，翻遍 ZIP 所有层级目录收集候选条目；排序优先级为「当前平台首选名优先 → 路径短优先（浅层目录）」，兼容 GitHub Releases / apiServer 分发 / 扁平打包 / 任意嵌套层级四种格式；basename 精确匹配确保不会误提取 LICENSE / frpc.toml / frpc.ini 等附加文件
  - **DownloadManager 集成**（[src-tauri/src/commands/frp/binary.rs](src-tauri/src/commands/frp/binary.rs)）：`ensure_system_default_frpc` 移除裸 `reqwest::get()` 调用，改用 `DownloadSession::start_grouped(state, "frpc 下载", [("frpc 二进制", 1.0)])` 初始化下载会话，构造 `DownloadTask` 调 `download_batch` 执行下载，复用全局 `download_cancel_flag` / `download_pause_flag` 支持暂停/取消；`download_state.version_name` 设为 `frpc v<version>` 供下载管理页展示；失败时调 `session.mark_failed(state, 1)` 并清理半成品 ZIP
- 复用说明：DownloadSession 模式参考 [src-tauri/src/commands/tools/download.rs](src-tauri/src/commands/tools/download.rs) 的 `download_file` 实现，与外部下载、整合包安装等场景共享同一套进度回调/flag 重置/manager 构造逻辑
- 验证：`cargo check` 编译通过；dev 模式启动正常，Tauri 窗口创建成功
- 待实测：进度反馈/暂停/取消/版本号显示/ZIP 提取（含跨平台文件名场景）

#### Frp 版本号查询修复（apiServer 校验语义化版本 + list_providers 显示云端最新版本）

- 背景：用户实测反馈两个问题：① 查询 apiServer `GET /v1/frp/manifest` 时空版本号直接返回 `code=1001: 版本号格式非法（不符合语义化版本规则）`；② 厂商列表系统默认厂商版本号仍显示固定 `0.61.0`，未反映云端最新版本
- 改动：
  - **manifest 查询传 0.0.0**（[src-tauri/src/commands/frp/binary.rs](src-tauri/src/commands/frp/binary.rs)）：`ensure_system_default_frpc` 中 `current_version` 从 `read_frpc_version().unwrap_or_default()`（空字符串）改为 `read_frpc_version().unwrap_or_else(|| "0.0.0".to_string())`，本地未安装时传 `0.0.0` 表示"查询最新版本"，符合 apiServer 语义化版本校验规则
  - **新增 fetch_latest_frpc_version**（[src-tauri/src/commands/frp/binary.rs](src-tauri/src/commands/frp/binary.rs)）：`pub(super) async fn fetch_latest_frpc_version(state)` 请求 apiServer `GET /v1/frp/manifest`（传 `current_version=0.0.0`）获取最新版本号，不下载文件，仅返回 `manifest.version`；`api_server_platform_arch` 同步改为 `pub(super)` 供此函数复用
  - **list_providers 显示云端最新版本**（[src-tauri/src/commands/frp/provider.rs](src-tauri/src/commands/frp/provider.rs)）：函数签名增加 `state: &AppState` 参数；系统默认厂商版本号策略改为：本地已安装（frpc_ready=true）从 `frpc_version.txt` 读取真实版本，本地未安装调 `fetch_latest_frpc_version(state)` 获取云端最新版本，失败回退显示"未安装"；删除不再使用的 `FRPC_VERSION = "0.61.0"` 常量
  - **IPC action 传 state**（[src-tauri/src/utils/frp_manager.rs](src-tauri/src/utils/frp_manager.rs)）：`list_providers` action 从 `handler!(_state, ...)` 改为 `handler!(state, ...)`，透传 `AppState` 给 `list_providers`
- 复用说明：`fetch_latest_frpc_version` 复用 `ensure_system_default_frpc` 的 manifest 查询逻辑（`load_creds_with_auto_refresh` + `OnlineClient::frp_get_manifest`），仅去掉下载部分，与 `signaling_manager` 的"GET 明文 + 自动 JWT"风格一致
- 验证：`cargo check` 编译通过，无警告
- 待实测：厂商列表首次加载（本地未安装）应显示云端最新版本号（如 `0.70.1`），点击"下载 frpc"应成功查询 manifest 并下载

#### Frp frpc 下载 ZIP 提取失败 + 版本号硬编码 + pnpm 残留物清理

- 背景：阶段三 frpc 下载切到 apiServer 后，用户测试发现三个问题：① 点击「下载 frpc」报错 `ZIP 中未找到 frpc 二进制（期望条目 frp_0.70.1_windows_amd64/frpc.exe）`；② 厂商列表显示版本号仍为硬编码的 `v0.61.0` 而非实际下载的 `0.70.1`；③ 项目根目录存在 pnpm 残留文件
- 根因分析：
  - **ZIP 提取失败**：apiServer 分发的 ZIP 命名为 `frp_client_0.70.1_windows_x86_64.zip`，内部目录为 `frp_client_0.70.1_windows_x86_64/frpc.exe`（带 `client` + `x86_64` 架构名），但 `extract_frpc_from_zip` 硬编码期望 GitHub 格式 `frp_0.70.1_windows_amd64/frpc.exe`（无 `client` + `amd64` 架构名），导致精确匹配失败
  - **版本号硬编码**：`FRPC_VERSION = "0.61.0"` 常量同时用于 manifest 查询的 `current_version` 参数和 `list_providers` 的 UI 显示，本地未安装时仍上报虚假版本号，下载完成后 UI 也显示旧版本
  - **pnpm 残留**：`pnpm-lock.yaml` 和 `pnpm-workspace.yaml` 被误提交到 git，且 `.gitignore` 忽略了 `package-lock.json`（npm 锁文件）
- 改动：
  - **ZIP 提取逻辑重写**（[src-tauri/src/commands/frp/binary.rs](src-tauri/src/commands/frp/binary.rs)）：`extract_frpc_from_zip` 不再接收 `entry_dir` 参数，改为遍历 ZIP 所有条目，查找文件名为 `frpc` / `frpc.exe` 的非目录条目，按路径长度排序选择最浅匹配（优先顶层目录或根级），兼容 GitHub、apiServer、扁平打包三种格式；移除不再使用的 `github_platform_arch` 函数
  - **版本元数据文件**（[src-tauri/src/commands/frp/provider.rs](src-tauri/src/commands/frp/provider.rs)）：新增 `frpc_version_path()` / `read_frpc_version()` / `write_frpc_version()` 三个函数，版本存储在 `<system_default_dir>/frpc_version.txt`；`list_providers` 改为从版本文件读取真实版本，缺失时回退 `FRPC_VERSION` 常量（旧版兜底）；`ensure_system_default_frpc` 查询 manifest 时本地未安装传空字符串强制 apiServer 返回最新，下载成功后写入 `manifest.version` 到版本文件
  - **注释更新**（[src-tauri/src/commands/frp/provider.rs](src-tauri/src/commands/frp/provider.rs)）：文件头注释从「从 GitHub Releases 下载」改为「从 apiServer `/v1/frp/manifest` 接口获取下载 URL」；`FRPC_VERSION` 常量注释标注为「旧版兜底，不参与 manifest 查询」
  - **pnpm 残留清理**：`git rm pnpm-lock.yaml pnpm-workspace.yaml`（移除 git 追踪 + 文件系统）；[.gitignore](.gitignore) 追加 `pnpm-lock.yaml` / `pnpm-workspace.yaml` 忽略规则；移除 `package-lock.json` 的忽略规则并 `git add` 提交 npm 锁文件
- 验证：`cargo check` 编译通过，无警告
- 待观察：frpc 下载当前使用裸 `reqwest::get()` 无进度反馈，未走项目 `DownloadManager`（适用于 Minecraft 批量分片下载，frpc 单文件 6MB 场景待评估是否需要集成）

### 重构

#### CI 发布工作流改用 Node.js 上传脚本（消除 MoSign-v2 签名不一致）

- 背景：`release.yml` 中上传/签名/注册逻辑使用内联 shell（heredoc + curl + openssl + node -e），存在 shell/Node 数据传递导致的签名不一致问题（heredoc 生成的 JSON 与 Node.js 签名读取的文件字节可能存在差异）
- 改动：
  - **新建** [scripts/ci-upload.cjs](scripts/ci-upload.cjs)：纯 Node.js 上传脚本，仅用内置模块（crypto/fs/https/http），无 npm 依赖。`JSON.stringify()` 生成 body Buffer，签名计算和 HTTP 请求使用同一 Buffer，保证 SHA256 完全一致。支持 S3 307 临时重定向
  - **重构** [.github/workflows/release.yml](.github/workflows/release.yml)：「Upload to S3 and register release」步骤从 173 行内联 shell 缩减为 1 行 `node scripts/ci-upload.cjs` 调用；安装包定位逻辑拆分为独立 step（Locate installer and signature）；底部说明更新
- 接口适配：MoLaunch 主仓库使用 `/v3/ci/presign-upload` 和 `/v3/ci/releases`（非 frp 版本），`CreateReleaseRequest` 字段包含 `channel` / `bundle_type` / `force_update` / `min_version` 等
- 复用说明：脚本结构参考 `Frp/hack/ci-upload.cjs`，适配 MoLaunch 接口差异（上传 2 个文件：安装包 + .sig 签名文件；请求体字段更多）

### 新增

#### Frp 联机功能阶段三（frpc 下载切到 apiServer + 公共 frps 服务器接口对接）

- 背景：apiServer 端 `GET /v1/frp/manifest` 与 `/v1/frp/servers` / `/allocate` / `/release` / `/keepalive` 路由已就绪，MoLaunch 客户端需对接：移除 GitHub Releases 下载源，改由 apiServer 统一分发 frpc；同时落地公共 frps 服务器分配/释放/续期链路，为前端「公共服务器」隧道创建模式铺路
- 设计依据：[docs/FRP_MANAGER_DESIGN.md](docs/FRP_MANAGER_DESIGN.md)、[docs/FRP_PUBLIC_SERVER_API_DESIGN.md](docs/FRP_PUBLIC_SERVER_API_DESIGN.md)
- 改动：
  - **新增 OnlineClient Frp 扩展**（新建 [src-tauri/src/minecraft/online/frp.rs](src-tauri/src/minecraft/online/frp.rs)）：封装 `frp_get_manifest` / `frp_list_public_servers` / `frp_allocate` / `frp_release` / `frp_keepalive` 5 个方法，复用 `OnlineClient::call_v1`（GET 明文 + 自动 JWT，POST 走 ECIES 加密信封 + CSRF）。数据结构 `FrpManifestQuery` / `FrpManifest` / `PublicFrpServer` / `AllocateRequest` / `AllocateResponse` / `AllocateServerInfo` / `ReleaseRequest` / `KeepaliveRequest` 与 apiServer `models/frp_server.rs` 字段一一对应，反序列化使用 `alias` 兼容 snake_case，序列化输出 camelCase 给前端
  - **模块注册**（[src-tauri/src/minecraft/online/mod.rs](src-tauri/src/minecraft/online/mod.rs)）：声明 `pub mod frp;`，与 `auth` / `signaling` / `tun` 等子模块并列
  - **frpc 下载源切换**（[src-tauri/src/commands/frp/binary.rs](src-tauri/src/commands/frp/binary.rs)）：`ensure_system_default_frpc` 完全移除 GitHub Releases 直链下载逻辑，改为：① `load_creds_with_auto_refresh` 加载设备凭证；② 构造 `FrpManifestQuery`（component=client，platform/arch 由 `api_server_platform_arch` 探测）；③ 调 `frp_get_manifest` 获取最新版本 URL；④ 下载 ZIP + `extract_frpc_from_zip` 提取 frpc 二进制。`ensure_frpc` 签名增加 `state: &AppState` 参数以加载凭证与 apiServer URL
  - **公共服务器 IPC action**（[src-tauri/src/utils/frp_manager.rs](src-tauri/src/utils/frp_manager.rs)）：新增 `list_public_servers` / `allocate_public_server` / `release_public_server` / `keepalive_public_server` 4 个 action，每个 action 复用 `load_creds` / `make_client` 辅助函数（与 `signaling_manager` 风格一致），统一处理 `code != 1` 业务错误；`ensure_frpc` action 调整为传入 `state`；`start_tunnel` action 调整为传入 `state`（透传给 `ensure_frpc` 以支持外部厂商 frpc 下载）
  - **进程管理签名调整**（[src-tauri/src/commands/frp/process.rs](src-tauri/src/commands/frp/process.rs)）：`start_tunnel` 签名增加 `state: &AppState` 参数，透传给 `ensure_frpc` 调用，确保启动隧道时能按需触发 frpc 下载
  - **前端类型扩展**（[src/types/frp.ts](src/types/frp.ts)）：新增 `PublicFrpServer` / `AllocatePublicServerParams` / `AllocateServerInfo` / `AllocateResponse` / `AllocationIdParams` 5 个类型，与后端 Rust 结构体字段一一对应（camelCase）
  - **前端 IPC 封装**（[src/utils/api/frp-manager.ts](src/utils/api/frp-manager.ts)）：`FRP_ACTIONS` 追加 `LIST_PUBLIC_SERVERS` / `ALLOCATE_PUBLIC_SERVER` / `RELEASE_PUBLIC_SERVER` / `KEEPALIVE_PUBLIC_SERVER` 4 个常量；新增 `listPublicServers` / `allocatePublicServer` / `releasePublicServer` / `keepalivePublicServer` 4 个便捷封装
  - **删除冗余占位文件**（删除 [src/utils/api/frp-public-server.ts](src/utils/api/frp-public-server.ts)）：原占位文件定义的类型与函数已迁移到 `types/frp.ts` 与 `frp-manager.ts`，避免重复定义
- 复用清单：
  - 凭证加载：`load_creds_with_auto_refresh` from `crate::utils::online_manager`（与信令 action 共用，禁止各 action 重复实现续期逻辑）
  - OnlineClient：`crate::minecraft::online::client::OnlineClient`（JWT + ECIES + CSRF 统一封装）
  - dispatcher handler 宏：`handler!(state, _app, params, ...)`（与 `signaling_manager` / `online_manager` 风格一致）
  - SHA256 / ZIP 提取：复用 `binary.rs` 既有 `compute_sha256` / `extract_frpc_from_zip` / `extract_archive`（禁止重复造轮子）
  - 前端 IPC 入口：`frpManager` from `@/utils/api/frp-manager`（单入口分发，禁止直连 fetch apiServer）
- 未复用说明：`api_server_platform_arch` / `github_platform_arch` 为 frpc 下载专用辅助函数（apiServer 与 GitHub ZIP 目录命名规则不同），未抽取到公共模块因其仅 frpc 下载场景使用，避免过度抽象
- 约束遵守：
  - 完全移除 GitHub 源：`binary.rs` 不再保留任何 GitHub Releases URL 或降级逻辑，apiServer 成为唯一 frpc 分发渠道
  - 配置读写统一：通过 `state.config.lock().await.online.api_server_url` 读取 apiServer 地址，未新增 `set_*` / `get_*` 单字段命令
  - 最小修改：仅修改 `start_tunnel` 签名增加 `state` 参数，未改动 `stop_tunnel` / `get_tunnel_status` 等无需 state 的函数
- 验证：`cargo check` 编译通过；`tsc --noEmit` 无 Frp 相关错误（仅 2 个预存的 `online.ts` / `crypto.ts` 错误，与本次改动无关）
- 待联调：前端「公共服务器」隧道创建 UI 实现后即可联调 `listPublicServers` → `allocatePublicServer` → `keepalivePublicServer`（定时）→ `releasePublicServer`（停止）完整链路

#### Frp 联机功能阶段二前端（外部厂商安装/启禁 + 实时日志 + 认证中心占位 + apiServer 预留）

- 背景：在阶段一（系统默认厂商 + frpc 配置文件启动）基础上，前端先行落地阶段二接口契约与 UI，覆盖外部厂商安装/卸载/启禁、frpc 实时日志流、认证中心占位、apiServer 公共 Frp 服务器 API 预留封装；后端 action 与 Tauri event 待阶段二后端实现后联调
- 设计依据：[docs/FRP_MANAGER_DESIGN.md](docs/FRP_MANAGER_DESIGN.md)、[docs/FRP_PUBLIC_SERVER_API_DESIGN.md](docs/FRP_PUBLIC_SERVER_API_DESIGN.md)
- 改动：
  - **前端类型扩展**（[src/types/frp.ts](src/types/frp.ts)）：`ProviderInfo` 追加 `enabled` / `distribution` / `homepage` 字段；新增 `LogFileInfo` / `LogFileContent` / `FrpcLogEvent` / `FrpTunnelStatusEvent` / `InstallProviderParams` / `ProviderIdParams` / `ReadLogParams` 类型，与后端 action 列表一一对应
  - **前端 IPC API**（[src/utils/api/frp-manager.ts](src/utils/api/frp-manager.ts)）：`FRP_ACTIONS` 追加 `INSTALL_PROVIDER_FROM_DIR` / `INSTALL_PROVIDER_FROM_ZIP` / `UNINSTALL_PROVIDER` / `ENABLE_PROVIDER` / `DISABLE_PROVIDER` / `LIST_LOG_FILES` / `READ_LOG_FILE` 共 7 个常量；新增 `installProviderFromDir` / `installProviderFromZip` / `uninstallProvider` / `enableProvider` / `disableProvider` / `listLogFiles` / `readLogFile` 便捷封装
  - **前端 Store 扩展**（[src/stores/frp.ts](src/stores/frp.ts)）：新增 `logs` / `logsLoading` / `selectedLogTunnelId` / `logFiles` / `logsHasMore` / `providerActionLoading` state；新增 `installProviderFromDir` / `installProviderFromZip` / `uninstallProvider` / `toggleProvider` / `loadLogFiles` / `readLogs` / `clearLogs` 共 7 个 actions，统一 toast 错误提示，与 `stores/online.ts` 风格一致
  - **侧边栏分类扩展**（[src/composables/useFrpSidebar.ts](src/composables/useFrpSidebar.ts)）：`frpCategory.children` 追加「认证中心」（`auth`，ShieldCheckIcon）和「运行日志」（`logs`，DocumentTextIcon）两个子项；`FrpSubCategory` 类型扩展为 `'providers' | 'tunnels' | 'auth' | 'logs'`
  - **厂商列表增强**（[src/components/frp/ProviderList.vue](src/components/frp/ProviderList.vue)）：顶部操作栏新增「从文件夹安装」「从 ZIP 安装」两个按钮（复用 `pickDirectory` / `pickFile`）；厂商卡片增加 authType 徽章（none=绿/oauth2=蓝/device_code=紫/api_key=黄）和 distribution 徽章（system=灰/bundled=蓝/url=青）；外部厂商卡片增加启用/禁用 Select 切换 + 「卸载」按钮（`showConfirm` 二次确认），禁用态卡片半透明显示；空状态 icon + text 垂直水平居中；组件 245 行，未超 300 行约束
  - **穿透管理增强**（[src/components/frp/TunnelManager.vue](src/components/frp/TunnelManager.vue)）：创建表单新增「厂商」Select 字段（仅列出 `enabled && (frpcReady || builtin)` 的厂商，默认 `system-default`）；厂商选择联动：未就绪的外部厂商显示 amber 色提示「请先在厂商列表页下载 frpc」；隧道卡片增加厂商名徽章（通过 `providerId` 在 providers 列表中查找 name）；组件 297 行，未超 300 行约束
  - **运行日志组件**（新建 [src/components/frp/FrpLogs.vue](src/components/frp/FrpLogs.vue)）：顶部工具栏（隧道 ID Select + 级别 Select + 刷新 + 清空），中部深色背景日志流（max-height 60vh + 垂直滚动），每行按级别着色（复用 `logLineClass` from `@/utils/log-display`，禁止重复定义颜色）；底部状态栏（当前隧道 + 行数 + 是否还有更多）；`onMounted` 启动 `frpc-log` 和 `frp-tunnel-status` 两个 Tauri event 监听（复用 `useTauriEvent` composable，自动 onUnmounted unlisten），实时日志按选中隧道过滤后追加到 `store.logs`，隧道停止时自动刷新历史日志；初始加载 `store.readLogs(selectedLogTunnelId)` 读取历史；空状态 icon + text 垂直水平居中；组件 154 行
  - **认证中心占位**（新建 [src/components/frp/AuthCenter.vue](src/components/frp/AuthCenter.vue)）：阶段三完整实现，阶段二仅显示 ShieldCheckIcon + 「认证中心」+ 「此功能将在阶段三上线，敬请期待」占位 UI，避免侧边栏菜单点击后空白
  - **联机页面集成**（[src/views/Online.vue](src/views/Online.vue)）：导入 `FrpLogs` 和 `AuthCenter` 组件，`currentComponent` switch 追加 `case 'auth': return AuthCenter` 和 `case 'logs': return FrpLogs`，`activeCategory` 类型已包含 `'auth' | 'logs'` 无需修改
  - **apiServer 公共服务预留**（新建 [src/utils/api/frp-public-server.ts](src/utils/api/frp-public-server.ts)）：定义 `PublicFrpServer` / `AllocateRequest` / `AllocateResponse` 类型，`listPublicFrpServers` / `allocatePublicFrpServer` / `releasePublicFrpServer` 三个函数当前直接抛错（`'apiServer 公共 Frp 服务器 API 尚未实现'`），等 apiServer 实现 `/v1/frp/*` 路由后改为实际 HTTP 请求；当前不引入 `invoke` / `fetch` 依赖以避免未使用 import 警告
- 复用清单：
  - 文件选择：`pickFile` / `pickDirectory` from `@/utils/fileDialog`（禁止直接用 `@tauri-apps/plugin-dialog`）
  - 二次确认：`showConfirm` from `@/utils/modal`
  - 日志颜色：`logLineClass` / `parseLogLines` / `LogLine` from `@/utils/log-display`（项目约定 ERROR=red-400 / WARN=yellow-400 / INFO=green-400 / DEBUG=cyan-400 / TRACE=slate-500）
  - Tauri event：`useTauriEvent` composable（自动 onUnmounted unlisten，参考 `JavaDownloadBar.vue` 模式）
  - Toast：`toastSuccess` / `toastError` from `@/utils/toast`
  - 自定义组件：`Button` / `Input` / `Select` / `Tooltip`（禁止用原生 `<button>` / `<input>` / `<select>` / `title`）
- 约束遵守：
  - Vue 组件行数：ProviderList 245 / TunnelManager 297 / FrpLogs 154 / AuthCenter 19，均 ≤ 300
  - 单列布局（参考主流启动器）：厂商卡片、隧道卡片、日志流均为单列
  - 空状态：icon + text 垂直水平居中（ProviderList / TunnelManager / FrpLogs 均遵守）
  - 不使用 Emoji，全部使用 Heroicons 图标 + 文字标签
- 待联调：后端 action（`install_provider_from_dir` 等 7 个）和 Tauri event（`frpc-log` / `frp-tunnel-status`）实现后即可联调；apiServer `/v1/frp/*` 路由实现后改写 `frp-public-server.ts` 抛错为实际请求
- 用户反馈：阶段二前端按规格实现，待后端 action 落地后联调

#### Frp 联机功能阶段二后端（外部厂商系统 + 日志格式改造 + Tauri event 推送 + 日志读取 action）

- 背景：在阶段一后端（系统默认厂商 + frpc 进程管理）和阶段二前端（外部厂商安装/启禁 + 实时日志 UI）基础上，实现阶段二后端：外部厂商安装/卸载/启禁、日志格式改造（`[HH:MM:SS.ms] [LEVEL] line`）、frpc-log / frp-tunnel-status event 实时推送、日志读取 action
- 设计依据：[docs/FRP_MANAGER_DESIGN.md](docs/FRP_MANAGER_DESIGN.md) §4.1（厂商清单结构）、§4.2（frpc 分发方式）、阶段二前端已落地的 action 契约和 event payload
- 改动：
  - **共享类型扩展**（[src-tauri/src/commands/frp/mod.rs](src-tauri/src/commands/frp/mod.rs)）：新增 `ProviderManifest` / `BinaryConfig` / `DownloadConfig` / `AuthConfig`（厂商 manifest.json 反序列化结构，`auth.type` 用 `#[serde(rename = "type")]` 处理关键字）；新增 `LogFileInfo` / `LogFileContent`（日志读取返回类型）；`ProviderInfo` 扩展 `enabled` / `distribution` / `homepage` 字段；新增 `providers_state_path()` 路径函数 + `validate_provider_id()` 公共校验函数（kebab-case，最长 64 字符）；`AuthConfig` 实现 `Default`（默认 auth_type=none）以支持 `#[serde(default)]`
  - **厂商管理改造**（[src-tauri/src/commands/frp/provider.rs](src-tauri/src/commands/frp/provider.rs)）：`list_providers` 改造为扫描 `<base_dir>/providers/` 下外部厂商目录 + 读取 manifest.json + 合并内置系统默认厂商，manifest 损坏或 id 不匹配的厂商跳过；新增 `install_provider_from_dir` / `install_provider_from_zip`（支持扁平/单根目录 ZIP，Zip Slip 防护 + canonicalize 父目录 starts_with 校验，参考插件系统 `extract_zip_safely` 等价实现）；新增 `uninstall_provider`（双重 canonicalize 防路径遍历）/ `enable_provider` / `disable_provider`（状态持久化到 `<base_dir>/frp/providers.json`）；`ensure_frpc` 增加 `provider_id: Option<String>` 参数，外部厂商 distribution=bundled 校验文件存在，distribution=url 实现 HTTPS + 域名白名单 + SHA256 校验 + archive 解压；新增 `get_frpc_path_for_provider` 按厂商返回 frpc 路径；新增 `read_provider_manifest` / `read_providers_state` / `write_providers_state` / `copy_dir_recursive` / `determine_zip_prefix` / `extract_zip_safely` / `validate_download_url` / `compute_sha256` / `extract_archive` 辅助函数
  - **进程管理改造**（[src-tauri/src/commands/frp/process.rs](src-tauri/src/commands/frp/process.rs)）：日志格式从 `[<unix_seconds>] [source] line` 改为 `[HH:MM:SS.ms] [LEVEL] line`（用 `chrono::Local::now().format("%H:%M:%S%.3f")`，LEVEL 推断：行内含 [E]/error/panic → ERROR，stderr → WARN，stdout → INFO）；`capture_stream` 增加 `app: AppHandle` 参数，每读到一行除写文件还 `app.emit("frpc-log", payload)` 推送实时日志（payload 含 tunnelId/tunnelName/line/timestamp/level，字段名 camelCase）；`FrpcHandle` 结构从 `{ child, pid }` 改为 `{ pid, stop_tx }`，child 移入 monitor task；`start_tunnel` 签名增加 `app: AppHandle`，按隧道 `provider_id` 选择对应厂商 frpc（调用 `ensure_frpc` + `get_frpc_path_for_provider`），spawn monitor task 用 `tokio::select!` 同时等待 child.wait() 和 stop_rx，退出时从 RUNNING 移除并 `app.emit("frp-tunnel-status", payload)` 推送状态变更（payload 含 tunnelId/tunnelName/status/pid/exitCode/error）；`stop_tunnel` 改造为先取出 stop_tx 并 drop 通知 monitor，再用 `kill_process_tree` 兜底；新增 `list_log_files`（扫描 logs 目录按修改时间倒序）和 `read_log_file`（尾部 maxLines 行，默认 500）
  - **沙箱校验增强**（[src-tauri/src/commands/frp/sandbox.rs](src-tauri/src/commands/frp/sandbox.rs)）：`validate_tunnel` 顶部增加 `provider_id` 非空 + kebab-case 格式校验（复用 `validate_provider_id`，不在后端校验厂商是否存在或启用，由前端 store 在创建前校验）
  - **分发层扩展**（[src-tauri/src/utils/frp_manager.rs](src-tauri/src/utils/frp_manager.rs)）：注册 7 个新 action（`install_provider_from_dir` / `install_provider_from_zip` / `uninstall_provider` / `enable_provider` / `disable_provider` / `list_log_files` / `read_log_file`）；`start_tunnel` handler 改用 `app` 参数（`app.clone()` 传给 `process::start_tunnel`）；`ensure_frpc` handler 接收可选 `provider_id` 参数（`unwrap_or_default` 兼容空 params）；新增 `EnsureFrpcParams` / `InstallProviderParams` / `ProviderIdParams` / `ReadLogParams` 参数结构体
- 复用清单：
  - 插件系统模式：`copy_dir_recursive` / `determine_zip_prefix` / `extract_zip_safely` 等价实现于 frp 模块内（不 import 插件系统私有函数，避免跨模块耦合）
  - 路径遍历防护：参考 `commands/plugins/sandbox.rs` 的双重 canonicalize + starts_with 模式
  - shell 调用：`crate::minecraft::system::shell::kill_process_tree`（项目硬约束）
  - HTTP 客户端：`crate::http::get_client()`
  - 日志宏：`crate::log_info!` / `crate::log_warn!`
  - 时间格式化：`chrono::Local::now().format("%H:%M:%S%.3f")`（Cargo.toml 已有 chrono 0.4 依赖）
  - SHA256：`sha2::Sha256` + `hex::encode`（Cargo.toml 已有 sha2 0.10 + hex 0.4）
  - handler! 宏：`_state, _app, _params`（不需要 app）/ `_state, app, params`（需要 app）
- 约束遵守：
  - Rust 文件行数：mod.rs 250 / provider.rs 723 / process.rs 398 / sandbox.rs 82 / frp_manager.rs 160；provider.rs 超 500 行（厂商管理职责集中，后续可拆分 install.rs），其余均在关注范围内
  - 所有新增 pub 函数和类型均有文档注释
  - 不引入新依赖（chrono / sha2 / hex / zip / reqwest 均已在 Cargo.toml）
  - shell 调用走 `crate::minecraft::system::shell` 模块
  - event 名用 kebab-case，payload 字段名 camelCase
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过（零错误零警告）
- 用户反馈：阶段二后端按规格实现，与阶段二前端 action 契约和 event payload 对齐，可联调

#### Frp 联机功能阶段一（厂商系统 + 隧道管理 + frpc 进程）

- 背景：用户要求为联机功能添加 Frp 内网穿透支持，需在联机页面侧边栏新增「Frp 管理」分类，第一阶段实现系统默认厂商（Frp 原版）的 frpc + 配置文件启动方式，为后续引入外部厂商、OAuth/Device Code 认证和 API 集成预留高度可扩展架构
- 设计依据：[docs/FRP_MANAGER_DESIGN.md](docs/FRP_MANAGER_DESIGN.md)（厂商系统、认证体系、安全沙箱方案）、[docs/FRP_PUBLIC_SERVER_API_DESIGN.md](docs/FRP_PUBLIC_SERVER_API_DESIGN.md)（apiServer 公共 frps 服务器 API）
- 改动：
  - **后端 Frp 命令模块**（新建 [src-tauri/src/commands/frp/](src-tauri/src/commands/frp/)）：
    - `mod.rs`：定义 `TunnelType` / `TunnelStatus` / `Tunnel` / `CreateTunnelParams` / `TunnelIdParams` 等共享类型，提供统一 IPC 入口 `frp_manager`，接收 `ActionRequest` 转发到 `utils/frp_manager::dispatch`
    - `provider.rs`：内置系统默认厂商（`system-default`），frpc 首次使用时从 GitHub Releases 下载 v0.61.0 到 `<base_dir>/providers/system-default/frpc.exe`（参考 McSDK 释放模式，不随安装包打包），ZIP 解压提取 frpc 二进制并校验非空，`ensure_frpc` / `is_frpc_ready` / `frpc_path` 等函数
    - `tunnel.rs`：隧道 CRUD，持久化到 `<base_dir>/frp/tunnels.json`，`generate_config` 按 TOML 格式生成 frpc 配置文件至 `<base_dir>/frp/config/<tunnel_id>.toml`，含 serverAddr/serverPort/tokentransport.tls/customDomains 等
    - `process.rs`：frpc 进程管理，启动时校验 frpc 就绪 + 生成配置 + spawn 子进程（Windows `CREATE_NO_WINDOW` 不弹控制台），异步捕获 stdout/stderr 增量写入 `<base_dir>/frp/logs/<tunnel_id>.log`，全局 `Mutex<HashMap<String, FrpcHandle>>` 维护运行进程表，`start_tunnel` / `stop_tunnel` / `get_tunnel_status` / `list_tunnels_with_status` 等
    - `sandbox.rs`：`validate_tunnel` 校验隧道名称（禁止换行/引号/反斜杠防 TOML 注入，长度 ≤64）、服务器地址（禁止协议前缀、换行、引号、反斜杠，长度 ≤255）、端口范围、token 长度（≤512）等
  - **后端统一分发**（新建 [src-tauri/src/utils/frp_manager.rs](src-tauri/src/utils/frp_manager.rs)）：`Lazy<Dispatcher>` 注册 7 个 action（`list_providers` / `ensure_frpc` / `list_tunnels` / `create_tunnel` / `delete_tunnel` / `start_tunnel` / `stop_tunnel` / `get_tunnel_status`），通过 `handler!` 宏绑定
  - **后端 IPC 注册**（[src-tauri/src/lib.rs](src-tauri/src/lib.rs)）：`invoke_handler` 注册 `frp_manager` 命令
  - **前端类型定义**（新建 [src/types/frp.ts](src/types/frp.ts)）：`TunnelType` / `TunnelStatus` / `Tunnel` / `TunnelWithStatus` / `ProviderInfo` / `CreateTunnelParams` 等类型，与后端 `snake_case` 序列化对应
  - **前端 IPC API**（新建 [src/utils/api/frp-manager.ts](src/utils/api/frp-manager.ts)）：`FRP_ACTIONS` 常量 + `frpManager<T>(action, params)` 统一入口 + `listProviders` / `ensureFrpc` / `listTunnels` / `createTunnel` / `deleteTunnel` / `startTunnel` / `stopTunnel` / `getTunnelStatus` 便捷封装
  - **前端 Pinia Store**（新建 [src/stores/frp.ts](src/stores/frp.ts)）：`useFrpStore` 管理 providers / tunnels / loading 状态，封装 `loadProviders` / `downloadFrpc` / `loadTunnels` / `createTunnel` / `deleteTunnel` / `startTunnel` / `stopTunnel` 等 actions，统一 toast 错误提示
  - **前端侧边栏分类**（新建 [src/composables/useFrpSidebar.ts](src/composables/useFrpSidebar.ts)）：`frpCategory` 含「厂商列表」+「穿透管理」两个子项，icon + desc 完整配置
  - **联机页面集成**（[src/views/Online.vue](src/views/Online.vue)）：导入 `frpCategory` + `ProviderList` + `TunnelManager`，`categories` 计算属性追加 `frpCategory`（无房间状态联动，始终可用），`currentComponent` 支持 `providers` / `tunnels` 子分类
  - **厂商列表组件**（新建 [src/components/frp/ProviderList.vue](src/components/frp/ProviderList.vue)）：展示已安装厂商卡片（图标 + 名称 + 内置标签 + 认证类型标签 + 版本/作者 + frpc 就绪状态），系统默认厂商未就绪时提供「下载 frpc」按钮，空状态使用 icon + 文字垂直水平居中
  - **穿透管理组件**（新建 [src/components/frp/TunnelManager.vue](src/components/frp/TunnelManager.vue)）：顶部操作栏（隧道数量 + 创建按钮），创建表单（隧道名称 + 类型 + 本地 IP/端口 + 服务器地址/端口 + 远程端口 + token + TLS 开关），隧道列表卡片（名称 + 运行状态标签 + 类型标签 + 本地/远程地址 + 启动/停止/删除操作按钮），复用项目自定义 `Button` / `Input` / `Select` / `Tooltip` 组件
- 架构特点：
  - **厂商系统可扩展**：参考插件系统设计，`manifest.binary.distribution` 支持 `bundled`/`url` 两种模式，`url` 模式需配置域名白名单，为后续引入外部厂商（自带 frpc/core）预留接口
  - **认证体系预留**：`ProviderInfo.authType` 支持 `none`/`oauth2`/`device_code`/`api_key`，仅用于拉取厂商配置文件，token 存储使用 OS 密钥存储（阶段二实现）
  - **安全沙箱**：配置生成前 `validate_tunnel` 校验所有用户输入，防止 TOML 注入、路径遍历、协议前缀注入；frpc 子进程 `CREATE_NO_WINDOW` 隔离
- 用户反馈："现在添加下，我目前要给联机功能添加 Frp联机功能，所以需要一个Frp管理页面，侧边栏加上就行……目前厂商就一个 系统默认，这个就是Frp原版，只支持Frpc+配置文件启动……参考我们启动器的插件系统，也通过那样管理，后续可能还需要引入厂商的API，所以需要高度可自定义化"
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过（零错误零警告）；`vue-tsc` 因 Node v24 兼容性问题跳过（已知环境问题，非代码错误）

#### 联机大厅加入按钮在房间中时禁用

- 背景：用户反馈已创建/加入房间后，联机大厅中的「加入」按钮仍可点击，导致可重复加入不同房间，需禁用并提示先退出当前房间
- 改动：
  - **LobbyBrowser.vue**（[src/components/online/LobbyBrowser.vue](src/components/online/LobbyBrowser.vue)）：从 `useOnlineStore` 派生 `isInRoom` computed（`roomState.role !== null`），传递给 `LobbyRoomCard` 的 `in-room` prop；`handleJoin` 顶部追加兜底校验，若 `isInRoom` 为 true 则 `toastInfo('您当前在房间中哟，如果要加入 请先退出或者关闭房间')` 并 return
  - **LobbyRoomCard.vue**（[src/components/online/LobbyRoomCard.vue](src/components/online/LobbyRoomCard.vue)）：新增 `inRoom?: boolean` prop。当 `inRoom` 为 true 时，加入按钮渲染为 `disabled` 状态（视觉变灰），并用 `Tooltip` 包裹显示提示文字「您当前在房间中哟，如果要加入 请先退出或者关闭房间」；为 false 时保持原有逻辑（loading/disabled/click）
- 效果：在房间中时大厅所有房间卡片的加入按钮自动变灰，hover 显示提示文字，点击无响应（disabled）；离开房间后按钮自动恢复可用
- 用户反馈："如果前端成功创建或者加入了房间，那么前端侧边栏目前禁用了加入和创建房间按钮，联机大厅中的加入按钮给我禁用了，直接提示您当前在房间中哟，如果要加入 请先退出或者关闭房间"

#### 开发者模式撤销解锁

- 背景：用户反馈已解锁开发者模式后无法撤销，缺少关闭入口
- 改动：
  - **后端撤销函数**（[src-tauri/src/commands/system/developer.rs](src-tauri/src/commands/system/developer.rs)）：新增 `lock_developer_mode(app)`，依次重置 `DeveloperMode` / `IgnoreTls` / `DeveloperUnlocked` 三个注册表项，若 DevTools 已打开则先关闭并重置 `DEVTOOLS_OPEN` 标志。任一步失败向上抛错保证状态一致
  - **IPC 注册**（[src-tauri/src/utils/system_manager.rs](src-tauri/src/utils/system_manager.rs)）：DISPATCHER 注册 `lock_developer_mode` action，通过 `app` 参数传递 AppHandle
  - **前端 API**（[src/utils/api/developer.ts](src/utils/api/developer.ts)）：新增 `lockDeveloperMode()` 调用 `SYSTEM_ACTIONS.LOCK_DEVELOPER_MODE`
  - **SYSTEM_ACTIONS 扩展**（[src/utils/api/system-manager.ts](src/utils/api/system-manager.ts)）：新增 `LOCK_DEVELOPER_MODE: 'lock_developer_mode'` 常量，developer 分组从 5 个增至 6 个
  - **撤销按钮**（[src/components/settings/DevModeToggle.vue](src/components/settings/DevModeToggle.vue)）：开关卡片底部新增「撤销解锁」按钮，点击弹 `showConfirm` 二次确认，确认后调用 `lockDeveloperMode()`，同步本地 `devUnlocked` / `devMode` 为 false，派发 `developer-mode-changed` 事件（payload=false）通知父组件隐藏侧边菜单「开发者」项，toast 提示「已撤销开发者模式」
  - **文档注释同步**：developer.rs 模块头新增第 5 步撤销流程；developer.ts 文件头同步更新
- 用户反馈："你也添加个撤销开发者模式的方法啊"

#### 开发者模式触发迁移 + 许可声明补全

- 背景：用户要求将开发者模式触发入口从系统信息页迁移至鸣谢法律信息中的隐藏字段，防止普通用户随意触发；同时在许可声明中补充 McSDK 与 cubiomes（xpple fork）的版权信息
- 改动：
  - **移除旧触发点**（[src/views/settings/more/SystemInfoTab.vue](src/views/settings/more/SystemInfoTab.vue)）：移除版本号连续点击 5 次解锁、设备 ID 双击 5 次备用解锁逻辑。版本号改为纯展示文本，设备 ID 双击仅保留切换全额显示/打码功能。清理 `devUnlocked` / `versionClickCount` / `deviceIdDblClickCount` 等状态与 `unlockDeveloperMode` / `safeCall` / `toastInfo` / `toastSuccess` 等无用导入
  - **新增隐藏触发字段**（[src/views/settings/more/CreditsTab.vue](src/views/settings/more/CreditsTab.vue)）：在法律信息 → 版权声明中，将「MoTeam」字段包装为可点击 span，连续点击 7 次（3 秒内）触发 `unlockDeveloperMode()`。无任何视觉提示（无 cursor pointer、无 hover 效果、无 tooltip、无倒计时 toast），完全隐藏于普通文本中。已解锁后点击无效，避免重复调用后端
  - **文档注释同步更新**：
    - [src-tauri/src/commands/system/developer.rs](src-tauri/src/commands/system/developer.rs)：模块级触发流程注释与 `is_developer_unlocked` 函数注释更新为新触发方式
    - [src/utils/api/developer.ts](src/utils/api/developer.ts)：文件头触发流程注释与 `isDeveloperUnlocked` / `unlockDeveloperMode` 函数注释更新
    - [src/components/settings/DevModeToggle.vue](src/components/settings/DevModeToggle.vue) 与 [src/views/settings/SettingsAdvanced.vue](src/views/settings/SettingsAdvanced.vue)：解锁触发点注释更新
  - **许可声明补全**（[src-tauri/resources/about/licenses.txt](src-tauri/resources/about/licenses.txt)）：在原版 Cubiomes 条目后追加两条——McSDK（Apache License 2.0，标注「自有项目，无需强制遵守」）与 Cubiomes (xpple fork)（MIT License，版权 Cubitect, xpple，来源指向 MoTeam-cn/cubiomes submodule 仓库）
- 用户反馈："激活开发者模式目前改成啥了，我不知道，我倒是想改成 更多侧边栏中的鸣谢的法律信息展开里面的一个特殊字段触发了，防止别人随意就触发了，其他的都去掉，然后后端资源库里面的那几个txt文件中 许可和版权声明里面加上 McSDK和 cubiomes"

#### DevTools 状态判断修复 + 全局快捷键禁用 + 水印解锁参数

- 背景：用户反馈三个问题：① 后端 `is_devtools_open` 明明 DevTools 已打开却返回 false；② 启动器需要禁用所有快捷键，仅在开发者页面提供独占快捷键；③ 测试版水印让用户看着"有点吐"，需要可在 DevTools 打开前提下解锁隐藏
- 改动：
  - **后端 DevTools 状态修复**（[src-tauri/src/commands/system/developer.rs](src-tauri/src/commands/system/developer.rs)）：根因是 Tauri 2 的 `WebviewWindow::is_devtools_open()` 在 Windows WebView2 上始终返回 false（WebView2 不提供查询 API）。引入 `static DEVTOOLS_OPEN: AtomicBool` 由后端自行维护：`open_devtools()` 成功后置 true，`close_devtools()` 置 false，`is_devtools_open()` 返回该状态。新增 `reset_devtools_state()` 用于窗口销毁时重置
  - **窗口销毁兜底**（[src-tauri/src/lib.rs](src-tauri/src/lib.rs)）：`on_window_event` 中监听 `WindowEvent::Destroyed`，调用 `reset_devtools_state()` 防止状态泄露
  - **全局快捷键禁用扩展**（[src/composables/useDevToolsGuard.ts](src/composables/useDevToolsGuard.ts)）：从仅拦截 DevTools 快捷键扩展为拦截所有非编辑类快捷键——F1~F12 全部拦截、Ctrl/Cmd+Shift+任意字母数字全部拦截、Ctrl/Cmd+Alt+字母拦截、Ctrl/Cmd+字母（除 c/v/x/z/y/a 编辑键外）拦截、Alt+字母拦截（避免激活菜单栏）。保留编辑键确保 input/textarea 正常使用
  - **开发者页面独占快捷键**（新建 [src/composables/useDevShortcuts.ts](src/composables/useDevShortcuts.ts)）：在 capture 阶段 `stopImmediatePropagation` 抢占事件流，绕过全局防护。绑定 `Ctrl/Cmd+Shift+D` 切换 DevTools 打开/关闭、`Alt+1~6` 切换子页签（1=实验性 / 2=DevTools / 3=证书 / 4=日志 / 5=存储 / 6=系统信息）。仅在 SettingsDeveloper.vue 存活时生效
  - **水印解锁 composable**（新建 [src/composables/useWatermarkUnlock.ts](src/composables/useWatermarkUnlock.ts)）：管理水印隐藏状态。`hide()` 前置调用 `isDevToolsOpen()` 校验，true 才允许隐藏；状态纯内存（不写入 sessionStorage/localStorage），刷新页面/重启应用即恢复显示，防止外部持久关闭水印。隐藏后启动 5 秒轮询检测 DevTools 状态，DevTools 关闭时自动恢复水印并同步 DevToolsTab 按钮状态（`unlocked` 为全局共享 ref）
  - **水印组件监听解锁**（[src/components/common/Watermark.vue](src/components/common/Watermark.vue)）：`showWatermark` 计算属性追加 `!unlocked.value` 条件；onMounted 调用 `syncWithDevTools()` 启动轮询监听
  - **DevTools 子页签新增水印解锁卡片**（[src/views/settings/developer/DevToolsTab.vue](src/views/settings/developer/DevToolsTab.vue)）：仅测试版构建显示。提供「隐藏水印」/「恢复水印」按钮，DevTools 未打开时给出"需先打开 DevTools"提示
  - **SettingsDeveloper 挂载独占快捷键**（[src/views/settings/SettingsDeveloper.vue](src/views/settings/SettingsDeveloper.vue)）：onMounted 调用 `useDevShortcuts({ onSwitchTab })`，Alt+1~6 切换子页签
  - **构建阻塞修复（项目已有 bug）**（[src/utils/version.ts](src/utils/version.ts)）：补全 `compareVersion(a, b)` / `versionChangeType(current, target)` / `VersionChangeType` 类型导出。`useVersionGroups.ts` 和 `useModUpdate.ts` 引用了这些符号但 version.ts 从未导出，导致 `vite build` 失败。本次为完成构建验证最小补全
  - **设计文档**（新建 [docs/DEVTOOLS_STATE_AND_SHORTCUTS_DESIGN.md](docs/DEVTOOLS_STATE_AND_SHORTCUTS_DESIGN.md)）：完整设计方案
- 用户反馈："目前那些都已经完成，目前修复下 后端判断devtools打开的逻辑，明明打开的，结果返回false关闭？？？然后启动器直接禁用所有快捷键，直接在开发者页面加一套快捷键，只能在开发者页面才能触发，你自己搭配下，然后就是前端水印加一个解锁参数，这个必须在devtools打开的情况下才能解锁，你设计下吧，因为我看着也有点吐"
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过（零错误零警告）；`npx vite build` 通过

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

#### 中文搜索本地映射（参考主流启动器实现）

- 背景：MoLaunch 原样透传中文关键词给 CurseForge / Modrinth 官方 API，两大平台索引不含中文，中文搜索几乎返回空结果。主流启动器通过内置 MC百科（mcmod.cn）本地数据库实现中文搜索，本次参考其思路在 MoLaunch 中实现等价功能
- 改动：
  - **模糊匹配算法**（新建 [src-tauri/src/minecraft/community/fuzzy.rs](src-tauri/src/minecraft/community/fuzzy.rs)）：移植主流启动器的 `SearchSimilarity` / `Search` 算法，基于最长公共子串的相似度，考虑长度加成（`1.4^(3+len) - 3.6`）和位置加成（`1 + 0.3 * max(0, 3-|qp-sp|)`），含 `SearchSource` / `SearchEntry<T>` 泛型类型和单元测试
  - **数据层扩展**（[src-tauri/src/minecraft/community/mcmod.rs](src-tauri/src/minecraft/community/mcmod.rs)）：`Entry` 新增 `popularity` 字段（解析 moddata.txt 最后一行排行数据）；`Database` 新增 `entries: Vec<ChineseSearchEntry>` 反查列表；新增 `search_by_chinese(query) -> RewriteResult` 公开函数，用本地模糊匹配把中文关键词重写为 CurseForge/Modrinth 英文 Slug/单词，并收集 Modrinth Slug 直查列表（最多 100 个）；新增 `extract_words` 单词提取（过滤停用词、单字、纯数字、子串去重）
  - **模块导出**（[src-tauri/src/minecraft/community/mod.rs](src-tauri/src/minecraft/community/mod.rs)）：导出 `fuzzy` 模块
  - **调度层拦截**（[src-tauri/src/minecraft/community/searcher.rs](src-tauri/src/minecraft/community/searcher.rs)）：在 `search()` 入口新增 `is_chinese` 检测（CJK 统一汉字 + 扩展 A + 兼容 ideographs），检测到中文时调 `mcmod::search_by_chinese` 重写查询词；三路并行（CF 搜索 + MR 搜索 + MR Slug 直查）通过 `tokio::join!` 调度，各自独立超时/错误隔离；中文未命中时回退原词透传
  - **Modrinth Slug 直查**（[src-tauri/src/minecraft/community/modrinth/mod.rs](src-tauri/src/minecraft/community/modrinth/mod.rs)）：新增 `get_projects_by_slugs(slugs, rtype) -> Vec<ResourceProject>`，调 `GET /v2/projects?ids=[...]` 批量拉取工程详情（slug 作为 project_id 别名），复用 `convert_project` 转换并写入缓存，失败返回空 Vec 不阻断搜索
  - **实现文档**（新建 [docs/CHINESE_SEARCH_IMPLEMENTATION.md](docs/CHINESE_SEARCH_IMPLEMENTATION.md)）：记录最终代码结构、与设计文档差异、验证方法、性能考量
- 用户反馈："我搜索模组或整合包时使用中文，两个平台都返回空，其他启动器用中文都能搜出来"
- 验证：`cargo check --manifest-path src-tauri/Cargo.toml` 通过（零错误零警告）；`cargo test fuzzy` / `cargo test mcmod` 单元测试通过
- 参考：主流启动器资源搜索器实现（`ResourceSearcher.vb` 189-290 行）

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

#### Frp provider.rs 拆分为 provider.rs + install.rs + binary.rs

- 背景：`src-tauri/src/commands/frp/provider.rs` 达 723 行，超 500 行关注线，职责混合（厂商列表 + 状态管理 + 安装/卸载 + frpc 下载），拆分为三个职责清晰的模块
- 改动：
  - **新增 [binary.rs](src-tauri/src/commands/frp/binary.rs)（269 行）**：frpc 二进制下载职责
    - 公开入口：`ensure_frpc`（按 provider_id 分发到系统默认/外部厂商）
    - 系统默认：`ensure_system_default_frpc`（GitHub Releases ZIP 下载 + 提取）
    - 外部厂商：`ensure_external_frpc`（HTTPS + 域名白名单 + SHA256 + 可选解压）
    - 辅助：`validate_download_url` / `compute_sha256` / `extract_archive` / `frpc_download_info` / `current_platform` / `frpc_filename`
  - **新增 [install.rs](src-tauri/src/commands/frp/install.rs)（234 行）**：安装/卸载职责
    - 安装：`install_provider_from_dir` / `install_provider_from_zip`（Zip Slip 防护）
    - 卸载：`uninstall_provider`（路径遍历防护）
    - 辅助：`build_provider_info` / `copy_dir_recursive` / `determine_zip_prefix` / `extract_zip_safely`
  - **精简 [provider.rs](src-tauri/src/commands/frp/provider.rs)（723→247 行）**：仅保留厂商列表 + 状态管理 + 路径辅助 + 启用/禁用
    - 路径函数：`system_default_dir` / `frpc_path` / `get_frpc_path_for_provider` / `is_frpc_ready` / `is_external_frpc_ready`
    - 状态持久化：`read_providers_state` / `write_providers_state`
    - manifest 读取：`read_provider_manifest`
    - 列表：`list_providers`
    - 启禁：`enable_provider` / `disable_provider`
  - **可见性调整**：provider.rs 中被 install.rs/binary.rs 调用的函数从 `fn`（私有）改为 `pub(super)`，包括 `system_default_dir` / `frpc_path` / `is_frpc_ready` / `is_external_frpc_ready` / `read_providers_state` / `write_providers_state` / `read_provider_manifest`；`FRPC_VERSION` 常量改为 `pub(super)`；`get_frpc_path_for_provider` 保持 `pub`（process.rs 跨模块调用）
  - **模块注册**（[mod.rs](src-tauri/src/commands/frp/mod.rs)）：新增 `pub mod binary;` 和 `pub mod install;`，更新模块文档注释
  - **调用方更新**：
    - [frp_manager.rs](src-tauri/src/utils/frp_manager.rs)：`ensure_frpc` 改为 `frp::binary::`；`install_provider_from_dir` / `install_provider_from_zip` / `uninstall_provider` 改为 `frp::install::`
    - [process.rs](src-tauri/src/commands/frp/process.rs)：`ensure_frpc` 改为 `crate::commands::frp::binary::ensure_frpc`（`get_frpc_path_for_provider` 仍在 provider）
- 复用清单：未引入新依赖，所有函数均为原 provider.rs 中的已有实现，仅做文件间迁移 + 可见性调整；`build_provider_info` 抽取了 install.rs 中重复 2 次的 ProviderInfo 构建逻辑
- 约束遵守：provider.rs 247 / install.rs 234 / binary.rs 269，均在合理范围内；cargo check 通过无错误无警告

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

#### TUN 虚拟网卡权限不足自动提权重启
- 背景：用户反馈联机创建房间后 TUN 接口创建失败（`os error 5` 拒绝访问），原因是 wintun.dll 创建虚拟网卡需要管理员权限。主流启动器的做法是自动退出程序并以管理员权限重新启动
- 改动：
  - **shell.rs 新增 `is_admin()` + `relaunch_as_admin()`**：[src-tauri/src/minecraft/system/shell.rs](src-tauri/src/minecraft/system/shell.rs) 新增管理员权限检测（Windows: `OpenProcessToken` + `GetTokenInformation(TokenElevation)`）和提权重启（Windows: `ShellExecuteW` with verb `"runas"` 触发 UAC 对话框）。参考主流启动器 `ModBase.RunAsAdmin`（`ProcessStartInfo.Verb = "runas"`）实现
  - **tun_start 检测权限错误**：[src-tauri/src/utils/tun_manager.rs](src-tauri/src/utils/tun_manager.rs) `tun_start` action 在 TUN 创建失败时检测 `os error 5` / `拒绝访问` / `Permission denied`，若非管理员则返回 `TUN_PERMISSION_DENIED:` 前缀错误标记
  - **新增 `restart_as_admin` action**：前端确认后调用，后端 `relaunch_as_admin()` 启动提权进程，延迟 500ms 退出当前进程
  - **前端自动弹确认框**：[src/composables/useVirtualLan.ts](src/composables/useVirtualLan.ts) `start()` 检测 `TUN_PERMISSION_DENIED:` 前缀，调 `showConfirmAsync` 弹出「需要管理员权限」确认框，用户确认后调 `restartAsAdmin()` 触发 UAC 提权重启
  - **Cargo.toml 补 Windows API features**：[src-tauri/Cargo.toml](src-tauri/Cargo.toml) `windows` crate 追加 `Win32_Security`（TokenElevation / TOKEN_QUERY）和 `Win32_UI_Shell`（ShellExecuteW）features
- 设计取舍：
  - **不使用 app.manifest requireAdministrator**：主流启动器通过 manifest 始终以管理员运行，但 MoLaunch 不应强制每次启动都弹 UAC。仅在 TUN 创建实际失败时才请求提权，用户体验更好
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
- 背景：阶段三子任务 9 评估 mesh 拓扑在 5+ 人时是否需要切换 SFU。基于 Minecraft LAN 流量模型（~100 KB/s）测算，5 人房主上行约 0.4 Mbps（家庭宽带舒适），10 人达 0.9 Mbps（多数家庭宽带扛不住）。结合项目定位（轻量启动器，对标主流启动器，2-5 人开黑为主），决策保持 mesh 拓扑 + 限制人数 ≤5，未来 5+ 人刚需时再评估 SFU
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
- 调研：阅读 Arco Design Vue 源码中 Input 组件与 FormItemMessage 相关实现，确认 Arco 原始 Input 组件本身不渲染提示文字，提示统一由 FormItem 的 FormItemMessage 子组件渲染（min-height 20px 防抖动 + form-blink 透明度动画 + 错误色 form-color-tip-text_error）
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
- 决策记录：参考主流启动器仅支持 Modrinth 格式导出，但用户要求"支持什么格式导入就支持什么格式导出"，故实现 6 种格式（除 LauncherPack，因 MoLaunch 不带启动器分发）；CurseForge 联网查不到 projectID/fileID 的 mod 按用户选择直接打包到 overrides
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

#### 清理代码中第三方启动器相关注释
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
  中代码注释里对第三方启动器的引用，仅保留 [CreditsTab.vue](src/views/settings/more/CreditsTab.vue)
  鸣谢页面与 [licenses.txt](src-tauri/resources/about/licenses.txt) 第三方许可声明。
- 补充：进一步将版本目录下遗留的旧目录重命名为 `MoLaunch/`，
  Logo 字段从旧路径改为 `MoLaunch\Logo.png`。
  涉及 [modpack_stages.rs](src-tauri/src/commands/community/install/modpack_stages.rs)
  `migrate_modpack_config` 与 `copy_external_logo`、
  [types.rs](src-tauri/src/commands/community/install/types.rs)、
  [mmc.rs](src-tauri/src/commands/community/install/mmc.rs)、
  [community.ts](src/types/community.ts) 中残留的旧路径引用全部清除。

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
  主流启动器不用 HEAD 预检，而是首线程不带 Range 拿 FileSize，后续线程带 Range 校验
  ContentLength，Range 失败时切换源或回退单线程。
- 修复：[probe.rs](src-tauri/src/minecraft/download/chunk/probe.rs)
  `supports_range` 从 HEAD + `accept-ranges` 改为 GET + `Range: bytes=0-0`，
  检查 HTTP 206 Partial Content 状态码。206 = 支持 Range，200/404/其他 = 不支持。
  与主流启动器的 GET + Range 动态检测策略一致，准确反映服务端真实行为。
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
  对比主流启动器源码（`ModModpack.vb` InstallPackCurseForge + `ResourceVersion.vb`
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
    参考主流启动器 `ResourceVersion.FromPlatformJson` 中 `Data("id")`。
    原 `rename_all = "camelCase"` 把 `file_id` 映射到 `fileId`，不匹配 `id`，
    导致反序列化失败 `missing field fileId`。
  - 修复：`#[serde(rename = "id")]` 把 `file_id` 映射到 JSON `id`。
- Bug 3：[helpers.rs](src-tauri/src/commands/community/install/helpers.rs)
  `construct_cf_edge_url`（downloadUrl 为空时的 CDN 直链兜底）：
  - 原 `split_at(len-4)` 拆分方向反了，应为 `split_at(4)`（Substring(0,4)/Substring(4)）。
    例如 fileId=2725062 原逻辑拼成 `files/272/5062`（错），正确应为 `files/2725/62`。
  - 原格式串漏掉 `file_name`，拼出的 URL 指向目录而非文件，下载必失败。
  - 修复：`split_at(4)` + 余位 `parse::<i64>()` 去前导 0（与主流启动器 CInt 等价）
    + 补上 `file_name`，最终格式 `{base}/files/{前4位}/{余位去0}/{file_name}`。
- Bug 4：[curseforge.rs](src-tauri/src/commands/community/install/curseforge.rs)
  `install_cf_mods` 对 `batch.data` 为空时静默成功：
  - `download_files_concurrent` 对空 `files` 列表直接返回 `Ok(())`（[concurrent.rs:28-32](src-tauri/src/commands/community/install/concurrent.rs#L28-L32)），
    导致镜像源（mod.mcimirror.top）不支持 `/mods/files` 批量查询返回空 data 时，
    `install_cf_mods` "成功"但 0 个 mod 下载，整合包"安装完成"而 mods 目录为空。
  - 修复：在 `cf_post` 返回后增加空 data 校验，`batch.data.is_empty()` 时返回
    `Err`，提示用户切换下载源到「缓慢时换镜像」或「尽量官方」（镜像源可能不支持
    `/mods/files` POST 批量查询，需走官方 API）。
- 参考主流启动器：其用同样的 `POST /v1/mods/files` 批量查询，`downloadUrl` 为空时
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
- 参考主流启动器整合包安装逻辑：仅校验 `projectID` 和 `fileID` 存在，
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
  而其他主流启动器均支持。分析其整合包解析源码，
  发现支持 7 种格式（CurseForge / HMCL / MMC / MCBBS / Modrinth / LauncherPack / Compress），
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
  - `detect_modpack_format` 改为返回 `DetectedModpack`，按主流启动器优先级顺序扫描
    关键文件：`mcbbs.packmeta` > `mmc-pack.json` > `modrinth.index.json` >
    `manifest.json`（有 addons → Mcbbs，无 → Curseforge）> `modpack.json`。
  - 两遍扫描：第一遍根目录，第二遍一级子目录（`archive_base_folder` 自动填充
    `"subfolder/"` 前缀，与主流启动器的 ArchiveBaseFolder 一致）。
  - 新增 `build_overrides_prefixes` 函数：按 format 构造 overrides 前缀列表
    （CF/MR：`overrides/` + `client-overrides/`；HMCL：`minecraft/`；MMC：`.minecraft/`；MCBBS：`overrides/`）。
  - `extract_overrides` 改为接受 `prefixes: &[String]` 参数，按前缀列表匹配并去掉前缀。
- 解析逻辑扩展（[modpack_stages.rs](src-tauri/src/commands/community/install/modpack_stages.rs)）：
  - `parse_modpack_info` 改为接受 `&DetectedModpack` 引用，新增 HMCL/MMC/MCBBS 三个分支：
    - HMCL：从 `modpack.json` 的 `gameVersion` 提取游戏版本；不解析加载器（与主流启动器一致）。
    - MMC：从 `mmc-pack.json` 的 `components[]` 按 uid 提取
      `net.minecraft`（game）/ `net.minecraftforge`（forge）/
      `net.neoforged`（neoforge）/ `net.fabricmc.fabric-loader`（fabric）；
      跳过 `org.lwjgl.*`（与主流启动器一致）。
    - MCBBS：从 `mcbbs.packmeta` 或带 `addons` 的 `manifest.json` 的 `addons[]` 按 id 提取
      `game` / `forge` / `neoforge` / `fabric` / `optifine`；遇到 `quilt` 直接报错
      （主流启动器也不支持 Quilt）。
- 安装流程调整（[modpack.rs](src-tauri/src/commands/community/install/modpack.rs)）：
  - `install_modpack` 和 `install_local_modpack` 的 `match info.format` 新增
    `Hmcl | Mmc | Mcbbs` 分支：跳过依赖 mods 下载（这些格式 mods 已打包在 overrides 中），
    直接进入 Stage 3 解压 overrides。
  - `extract_overrides` 调用改为传入 `build_overrides_prefixes(info.format, &info.archive_base_folder)`。
- 前端类型扩展（[src/types/community.ts](src/types/community.ts)）：
  `ModpackFormat` 类型扩展为 `'curseforge' | 'modrinth' | 'hmcl' | 'mmc' | 'mcbbs'`。
- 行为对齐主流启动器：HMCL/MMC/MCBBS 整合包不下载依赖 mods，仅解压 overrides + 安装游戏本体。
- 验证：cargo check 0 errors 0 warnings，tsc 0 errors。需测试三种新格式整合包
  的拖拽安装流程，特别是 overrides 目录前缀正确性（HMCL 的 `minecraft/`、
  MMC 的 `.minecraft/`、MCBBS 的 `overrides/`）。

#### 新增拖拽全局遮蔽层 DragOverlay，提升拖拽体验
- 背景：用户反馈拖拽整合包/Mod 时直接弹出实例名输入框过于生硬，缺乏其他启动器
  （如主流启动器/HMCL）的全屏遮蔽层 + 图标 + 提示文案的视觉反馈。
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
  主流启动器整合包解析使用动态 JObject 解析，对缺失字段做跳过处理。
- 修复（[src-tauri/src/commands/community/install/curseforge.rs](src-tauri/src/commands/community/install/curseforge.rs)）：
  - `CfManifestFile.project_id` 改为 `Option<i64>` + `#[serde(default)]`，缺失时为 None。
  - `install_cf_mods` 中 `project_ids` 改用 `filter_map` 过滤 None，缺失项跳过 slug 查询。
  - `file_translated` 构造改为 `project_id.and_then(...)` 链式调用，缺失时译名直接为 None，
    下载仍正常进行（仅文件名不应用 community_filename_format 译名重命名）。
- 行为对齐主流启动器：缺失 projectID 不阻断安装，仅影响译名查询。
- 验证：cargo check 通过。需测试缺失 projectID 的 CF 整合包能否正常解析并安装。

#### 新增拖拽安装整合包与 Mod 功能
- 背景：MoLaunch 此前仅支持从社区资源页在线下载整合包，无法处理用户从本地
  拖入的 .zip / .mrpack 整合包文件或 .jar / .litemod Mod 文件。参考主流启动器
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
  与"单列堆叠"的 UI 偏好冲突。
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
- 后端 `src-tauri/src/commands/tools/memory.rs` 重写为 `NtSetSystemInformation` 方案（与主流启动器一致）：
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
  4. **文件存在，lang 是其他语言且 saves/ 不存在**：覆盖为目标语言（先写 `-` 触发缓存清空，再写目标值）
  5. **文件存在，lang 是其他语言且 saves/ 已存在**：跳过，尊重老用户手动选择的语言
- 补充 `#[cfg(test)]` 单元测试覆盖 `adjust_lang_case` 与 `to_upper_suffix`：MC 1.0~1.10 大写后缀、1.11+ 小写、26+ 小写、无下划线代码原样返回

#### 关于页新增 MoLaunch 实现原理介绍
- `src/components/about/MoLaunchIntro.vue`：新增组件，默认折叠，点击标题栏展开 200 字实现说明，内容涵盖技术栈选型（Tauri 2 + Vue 3 + Rust）、启动器核心实现（版本管理、Java 检测、游戏启动）、联机模块（FRP 隧道 SDK 动态库嵌入与释放）、UI 设计理念（参考 Arco Design）、数据存储与安全（设备 ID 派生密钥加密）
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
- 安全设计（参考主流启动器 `StartCustomDownload`）：
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

#### 内存优化改为枚举所有进程裁剪工作集（释放量从几十 MB 提升到数 GB）
- 根因：原实现仅对启动器自身进程调用 `SetProcessWorkingSetSize`，只释放了启动器自己的工作集（几十 MB）；主流启动器枚举系统所有进程逐个裁剪工作集，释放整个系统的物理内存（数 GB）
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

#### 离线账号皮肤接入启动流程（方案 A + 方案 B）
- `src-tauri/src/minecraft/auth/mod.rs`：新增 `adjust_uuid_for_skin_variant()` 函数，通过递增 UUID 末位让 MC 离线模式哈希到目标皮肤模型（Steve=classic / Alex=slim），算法参考主流启动器皮肤判定实现
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

#### Mod 多选与版本更新功能
- 版本设置 Mod 管理页新增多选模式与版本更新/更改功能
- 多选交互（复刻主流启动器多选交互）：
  - **点击列表项即切换选中**（主流启动器也是点击触发，非长按）
  - Shift+点击 范围选择，ESC 清空选中
  - 批量操作：启用、禁用、更新、删除、全选、反选
  - **批量操作完成后自动清空选中**（参考主流启动器 `ChangeAllSelected(False)`）：启用/禁用、删除操作成功后无条件调用 `clearSelection()`，退出多选状态
- 按钮智能禁用（复刻主流启动器按钮逻辑）：
  - 选中项中没有已启用的 mod 时，"禁用"按钮禁用（`hasEnabledSelected`）
  - 选中项中没有已禁用的 mod 时，"启用"按钮禁用（`hasDisabledSelected`）
  - 选中项中没有可更新的 mod 时，"更新"按钮禁用（`hasUpdatableSelected`）
  - "删除"按钮始终可用（只要有选中项）
  - `batchActions` 改为 `computed`，根据选中状态响应式更新 `disabled` 属性
- 选中状态指示（复刻主流启动器选中指示）：
  - **不使用复选框图标**（太突兀），也**不覆盖原有启用/禁用状态色条**
  - 在列表项左边缘外侧挂一条 5px 宽的蓝色圆角竖条（`-left-1` 向左探出 4px），与主流启动器 `Margin=(-3,6,0,6)` 一致
  - 未选中：竖条不渲染，完全不影响原有状态色条
  - 选中：竖条上下留 6px（`top-1.5 bottom-1.5`），用 `transform: scaleY` 弹性动画（`cubic-bezier(0.34, 1.56, 0.64, 1)` 先冲到 1.15 再回弹到 1，对应主流启动器回弹动画）
  - 选中时标题颜色变为主题强调色（`text-blue-600`，对应主流启动器强调色）
  - 原有的启用/禁用状态色条保持不变，两者位置独立、互不干扰
- 多选操作栏布局（复刻主流启动器多选操作栏）：
  - **浮动在视口底部中央**（fixed bottom-6 left-1/2），不占据列表布局空间
  - 卡片分上下两部分：上方居中"已选择 X 项"文字，下方水平排列操作按钮
  - 入场动画：从下方滑入 + 淡入（对应主流启动器的滑入 + 淡入）
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

#### Mod 图标机制重构 + Mods 目录文件监听
- **放弃 jar 解包提取 logo**，改用平台工程 `logo_url` + `image_cache` 缓存机制（与皮肤/披风一致），实现「几秒后图标自动加载出来」的体验
- 图标缓存机制（复用皮肤/披风 `image_cache::get_image_url`）：
  - 预加载查到 CF/MR 工程后，调用 `image_cache::get_image_url(project.logo_url, app)` 处理 logo URL
  - 命中缓存：返回 `cache-image://{hash}.png`，零网络请求，前端立即渲染
  - 未命中：返回远程 URL，后端异步下载，完成后 emit `image-cached` 事件通知前端刷新
  - 前端 `useModOperations` 监听 `image-cached` 事件，按 `cached_logo_url === remote_url` 匹配 mod 并原地替换为本地缓存 URL
  - 持久化缓存命中时从 `project.logo_url` 重新计算 `cached_logo_url`（image_cache 状态可能已变化）
- Mods 目录文件监听（参考主流启动器 FileSystemWatcher）：
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

#### Mod 版本号识别链完整复刻主流启动器
- **根因**：之前只按顺序短路返回第一个找到的来源，且缺少 `fml_cache_annotation.json` 来源，导致部分 Forge mod 无法获取版本号
- **完整复刻主流启动器 `LocalResourceFile.LoadMetadataFromJar`**的 4 来源累积合并策略：
  1. `mcmod.info`（Forge 1.12-）
  2. `fabric.mod.json`（Fabric/Quilt，必须包含 `schemaVersion` 才视为有效）
  3. `META-INF/mods.toml`（Forge 1.13+/NeoForge）
  4. **`META-INF/fml_cache_annotation.json`（Forge 1.7-1.12 注解缓存，新增）**——查找 `@Mod` 注解，从 `values.version.value` 获取版本号
- **累积合并不覆盖策略**（参考主流启动器的 Display/Description/Version setter）：
  - `MetaBuilder` 封装"已有有效值不覆盖"逻辑
  - `slug`：第一个非空值优先
  - `description`：第一个长度>2的值优先
  - `version`：第一个有效版本号（只含数字、点、减号）优先，占位符（包含 "version" 字样，如 `${file.jarVersion}`）标记为 `"version"`
- **`${file.jarVersion}` 占位符统一处理**：标记为 `"version"` 后，最后从 `META-INF/MANIFEST.MF` 的 `Implementation-Version` 解析（参考主流启动器 Finished: 标签）
- **版本号有效性校验**：版本号必须包含 `.` 或 `-`，否则视为无效（参考主流启动器对应实现）
- 拆分为目录结构（文件超过 500 行按项目约定拆分）：
  - `metadata/mod.rs`：主入口 + `MetaBuilder` 合并器 + `finalize_metadata` + `extract_version_from_filename`
  - `metadata/sources.rs`：4 个来源的 `merge_*` 函数 + `read_manifest_version`

#### CurseForge 版本列表版本号修复
- **根因**：`curseforge/convert.rs` 的 `convert_version` 直接写 `version: String::new()`，注释"CurseForge 无版本号字段"，导致 CF 版本列表的 `ResourceVersion.version` 全为空字符串，前端 `versionChange` 计算永远走 `unknown` 分支
- **参考主流启动器**： `If(Entry.ProjectVersion.Version, Entry.ProjectVersion.Display)`，主流启动器对 CF 也是 `Version = Nothing`，但用 `Display`（即 `displayName`）作为 fallback
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
- **修复**：参考主流启动器 `McLibListGet`，新增 `collect_libraries_recursive` 递归合并父版本 libraries
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
- **参考主流启动器**：启动时构建 classpath 只调用 `McLibListGet` 获取路径列表，**不做任何文件校验和哈希检查**。文件校验和下载在安装阶段做，启动时不重复校验
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

#### 崩溃分析结果前端无提示修复 + 崩溃弹窗
- **根因**：`launch` 命令在 `pipeline.execute().await` 返回 `Err` 时（如 `ClassNotFoundException` 致命错误），通过 `?` 直接返回 Err，**后面的 `tokio::spawn` 监听 `exit_rx` 退出事件的任务永远不会被创建**。所以 `game-exited` 事件永远不会发送，前端收不到崩溃信息
- **后端修复**：`launch.rs` 捕获 `LaunchProcess` 阶段的失败，等待 watcher 完成崩溃分析后手动发送 `game-exited` 事件
  - 只对 `LaunchProcess` 阶段的失败做崩溃分析（`GetJava`/`Login` 等阶段失败不需要）
  - 等待 `exit_rx` 最多 15 秒，避免无限等待
  - 如果崩溃分析无结果，构造基本的 `CrashInfo`（用 `launch_err.message` 作为 reason）
  - 清理启动状态后发送 `game-exited` 事件，让前端展示崩溃对话框
- **前端优化**：`CrashDialog.vue` 参考主流启动器 `MyMsgText` 风格优化
  - 样式参考：
    - 浅灰白底 `#FBFBFB`（`Background="#FBFBFB"`）
    - 圆角 `rounded-lg`（`CornerRadius="7"`）
    - 标题下方加 2px 分割线（`ShapeLine`，与标题同色）
    - 遮罩半透明黑色 `bg-black/40`（`RGBA(90,0,0,0)`）
  - 弹窗进入动画：
    - 透明度 0→1（120ms）
    - Y 偏移 40→0（300ms，回弹缓动 `cubic-bezier(0.34, 1.56, 0.64, 1)`）
    - 关闭时下沉 20px + 淡出
  - Transition 名从 `modal` 改为 `crash-modal`，添加 scoped 样式

#### Fabric 库下载失败根因修复 + CrashDialog 报错修复 + 弹窗重做
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
- **CrashDialog 弹窗重做**
  - 标题字号 23px（`LabTitle FontSize=23`）
  - 标题下方 2px 分割线（`ShapeLine`，与标题同色 `bg-gray-700/80`）
  - 内容字号 15px（`LabCaption FontSize=15`）
  - 文字颜色 `#5C5C5C`（`LabCaption Foreground="#FF5C5C5C"`）
  - 去掉"崩溃原因""建议"等小标题卡片，改为纯文本段落（参考 `GetAnalyzeResult` 输出）
  - 浅灰白底 `#FBFBFB`，圆角 `rounded-lg`（`CornerRadius="7"`）
  - 按钮 3 个右对齐：查看输出 / 导出错误报告 / 确定（`PanBtn`）
  - 进入动画：透明度 0→1（120ms）+ Y 偏移 40→0（300ms 回弹缓动）

#### Fabric 库 URL 拼接修复 + CrashDialog 配色复刻
- **URL 拼接缺斜杠修复**（`libraries.rs` `root_url` 构造）
  - **根因**：`format!("{}{}", u.trim_end_matches('/'), path)` 把 URL 结尾的 `/` 去掉后直接拼接，导致 `https://maven.fabricmc.net/` + `org/ow2/asm/...` 变成 `https://maven.fabricmc.netorg/ow2/asm/...`（缺少斜杠）
  - **修复**：改为 `format!("{}/{}", u.trim_end_matches('/'), path)`，用 `/` 连接
- **parse_libraries 读取 Fabric 格式 size/sha1**（之前修改未保存，重新修复）
  - Fabric 版本 JSON 的库格式：`{ "name": "...", "sha1": "...", "size": 126151, "url": "..." }`
  - size 和 sha1 在根级别，不在 `downloads.artifact` 里
  - else 分支从根级别读取 `library["size"]` 和 `library["sha1"]`
- **CrashDialog 配色复刻**
  - 在 `tailwind.config.js` 添加弹窗颜色系：
    - `pcl-1`=`#343d4a`（深灰蓝，正文/默认文字/阴影）
    - `pcl-2`=`#0b5bcb`（主蓝，标题/Highlight 按钮）
    - `pcl-3`=`#1370f3`（亮蓝，悬停态边框）
    - `pcl-7`=`#e0eafd`（按钮悬停背景）
    - `pclmsg-bg`=`#FBFBFB`（弹窗背景）
    - `pclmsg-caption`=`#5C5C5C`（正文文字，写死不随主题变）
  - 弹窗配色：
    - 标题 `text-pcl-2`（`#0b5bcb`），字号 23px
    - 分割线 `bg-pcl-2`（与标题同色，高 2px）
    - 正文 `text-pclmsg-caption`（`#5C5C5C`），字号 15px，行高 18px
    - 背景 `bg-pclmsg-bg`（`#FBFBFB`）
    - 阴影 `shadow-[0_4px_20px_rgba(52,61,74,0.5)]`（DropShadowEffect）
    - 遮罩 `bg-black/35`（`rgba(0,0,0,0.353)`）
  - 按钮配色参考三态：
    - 确定按钮（Highlight 态）：边框 `border-pcl-2`，文字 `text-pcl-2`，hover 变 `pcl-3` + 背景 `pcl-7`
    - 查看输出/导出按钮（Normal 态）：边框 `border-pcl-1`，文字 `text-pcl-1`，hover 同上
    - 按钮背景 `bg-white/30`（`ColorBrushHalfWhite #55ffffff`）
    - 圆角 `rounded`（`CornerRadius=3`）
    - 过渡 `duration-100`（颜色过渡 100ms）

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

#### 启动高级选项
- 新增 3 个启动高级选项，位于"启动设置"页面底部：
  - **禁用 Java Launch Wrapper**：JLW 用于修复 Java 18- 在中文路径下可能无法正常启动的问题
  - **禁用 LWJGL Unsafe Agent**：LUA 用于修复 LWJGL 3.4.1 的性能问题，通过 `-javaagent` 参数注入 `lwjgl-unsafe-agent.jar`
  - **使用高性能显卡**：自动在 Windows 设置中将 Java 改为使用独立显卡
- 引入 `lwjgl-unsafe-agent.jar` 到 `src-tauri/resources/`，注册为嵌入资源
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
  - `modrinth/mod.rs`（234 行）：`version_files_search`（version_files 用 SHA1 查 → project_id → /projects 批量查询 + sha1 一致性校验防错位）+ `search` + `get_project` + `get_versions` + `batch_get_project_slugs`（整合包文件名格式化用）
- 同步清理：移除 `modrinth.rs` 中的私有 `urlencode_params` 函数（与 `curseforge.rs` 重复），改用 Phase 4.6 抽取的 `super::common::urlencode_params`。至此 `urlencode_params` 重复定义问题完全解决
- `modrinth::{search, get_project, get_versions, version_files_search, batch_get_project_slugs}` 公共 API 路径保持完全向后兼容，`community/mod.rs` 已有的 `pub mod modrinth;` 声明 + 5 处外部调用（preload / searcher / detail / community/install）均无需修改
- 验证：`cargo check` 通过

#### 代码重构阶段 4.6：拆分 minecraft/community/curseforge.rs 为 4 个子模块
- 现象：`minecraft/community/curseforge.rs` 786 行，单一文件混合「9 个 CF API 响应数据结构（CfModEntry / CfFile / CfSearchResponse 等）」「响应到统一资源模型的转换（convert_project / convert_version / parse_cf_download_url）」「HTTP 请求层（get_cf_config + cf_get / cf_post + source 策略回退镜像）」「公共 API（fingerprint_search / search / get_project / get_versions / batch_get_mod_slugs + 私有 curseforge_loader_type）」4 块关注点
- 修复：将 `curseforge.rs` 升级为 `curseforge/` 目录，拆为 4 个子模块：
  - `curseforge/types.rs`（111 行）：`CfSearchResponse` / `CfPagination` / `CfModEntry` / `CfLogo` / `CfLinks` / `CfCategory` / `CfFile` / `CfHash` / `CfFilesResponse` 共 9 个 CF API 响应数据结构（`pub(crate)` 可见性，仅模块内部使用）
  - `curseforge/convert.rs`（140 行）：`convert_project`（CF 工程条目 → ResourceProject，含 tags 翻译 + mcmod 中文译名 + 加载器标志位聚合）+ `convert_version`（CF 文件 → ResourceVersion）+ `parse_cf_download_url`（构造 edge.forgecdn.net 回退 URL）
  - `curseforge/http.rs`（259 行）：`CF_OFFICIAL_BASE` / `CF_MIRROR_BASE` 常量 + `get_cf_config`（source 策略：0=强制镜像 / 1=缓慢时换镜像 / 2=尽量官方）+ `build_cf_request` / `build_cf_post_request`（附加 x-api-key header）+ `cf_get` / `cf_post`（source=1 时官方失败自动回退镜像重试，官方请求 10s/15s 超时）
  - `curseforge/mod.rs`（305 行）：`fingerprint_search`（fingerprints/432 → modId → /mods 批量查询）+ `search` + `get_project`（数字 modId 走 /mods/{id}，slug 走 /mods/search）+ `get_versions` + `batch_get_mod_slugs`（整合包文件名格式化用）+ 私有 `curseforge_loader_type`
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
- 额外修复：`get_java_version_weight`（权重表）在 `java_selector.rs` 和 `java/mod.rs` 各有一份相同实现；将 `java_selector.rs` 的版本改为 `pub`，`java/mod.rs` 删除本地实现改为调用前者

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

#### Mod 列表两阶段加载（秒加载 + 排序修复）
- 现象：用户反馈每次进入 Mod 列表都要等好几秒（143 个 mod 要等 jar 元数据全部读完），主流启动器进入基本秒加载；且禁用的 mod 总是被排到列表末尾
- 根因分析：
  - 同步阶段**只做文件枚举**（`DirectoryUtils.GetFiles`），完全不读 JAR 内容，所以瞬间返回
  - 排序规则只按 `File.Name`（含扩展名）字母序升序，**禁用状态不参与排序**（第 88 行 `ModList.OrderBy(Function(m) m.File.Name)`）
  - MoLaunch 原 `list_mods` 对每个 jar 同步调用 `read_mod_metadata`（打开 jar + 读 fabric.mod.json/mods.toml/mcmod.info + 提取 logo base64 + 查 mcmod 译名），143 个 mod = 143 次磁盘 IO，这是慢的根本原因
  - MoLaunch 原排序规则「启用的排前面 + 文件名升序」导致禁用的 mod 被挤到末尾
- 修复 1：`list_mods` 极致轻量化（backend: src-tauri/src/commands/version/mods.rs）
  - 去掉 `read_mod_metadata` 调用，元数据字段（translated_name/description/version/logo_data/slug）全部返回空
  - 只做文件枚举 + 获取文件大小 + 推断加载器类型（从文件名），保证瞬间返回
  - 排序改为只按 `file_name`（含扩展名）字母序升序，禁用状态不参与排序（与主流启动器一致）
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
- 失败容错：所有译名查询失败时返回空 map，下载流程继续，仅文件名不应用格式（不让网络问题阻断整合包安装）

#### Mod 管理「详情」按钮关联社区资源 + 新增「前往百科」按钮
- 现象：用户希望 Mod 管理列表的「详情」按钮能直接打开社区资源详情弹窗（即搜索 mod 时弹出的 ResourceDetail），而不是只显示本地信息；无法关联的 mod 才回退到本地信息弹窗；另外希望加个「前往百科」按钮直接打开 mcmod.cn
- 后端 `ModInfo` 新增 `slug: String` 字段，`read_mod_metadata` 返回元组扩展为 `(translated, description, version, logo, slug)`，把从 jar 内 metadata 读到的 slug（fabric.mod.json 的 id / mods.toml 的 modId / mcmod.info 的 modid）带回前端用于关联 CF/MR 平台工程（backend: src-tauri/src/commands/version/mods.rs）
- 前端 `ModInfo` interface 同步新增 `slug: string` 字段（frontend: src/utils/api/personal.ts）
- ModTab.vue「详情」按钮逻辑改造：
  - 有 slug：先调 `getProjectDetail('CurseForge', slug, 'Mod')`（CF API 支持用 slug 查询 mod），失败再调 `getProjectDetail('Modrinth', slug, 'Mod')`（MR API 同样支持 slug 查询）。成功则弹出复用的 `ResourceDetail` 组件展示完整 mod 详情（版本、下载、描述等），与社区资源搜索的详情弹窗完全一致
  - 失败或无 slug：回退到 `showLocalModInfo` 显示本地信息弹窗（描述、文件、版本、译名、加载器），与原行为一致
  - 「详情」按钮加载期间 disabled 防止重复点击（frontend: src/views/version-settings/ModTab.vue）
- 新增「前往百科」按钮（在「详情」按钮右侧，hover 列表项时显示）：
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

#### Mod 详情预加载架构
- 设计目标：参考主流启动器详情弹窗的核心设计——**详情按钮本身不发任何网络请求**，只判断 `Entry.Project` 是否已被预加载填充，实现零延迟跳转。预加载由 `LocalResourceOnlineLoader` 在 `list_mods` 返回后立即后台执行（哈希批量查询 + 工程详情拉取）
- 后端新增预加载核心模块（backend: src-tauri/src/minecraft/community/preload.rs）：
  - MurmurHash2 算法实现（CF 指纹算法）：读取文件字节后**跳过空白字节**（0x09/0x0A/0x0D/0x20），再用 seed=1、m=0x5bd1e995、r=24 计算（与主流启动器实现一致）
  - SHA1 hash 计算（MR 文件识别算法，标准 SHA1）
  - `preload_mods_detail` 主入口流程：1) 读持久化缓存 → 2) 计算每个 mod 的 CF MurmurHash2 + MR SHA1 → 3) `tokio::join!` 并发批量查询 CF/MR → 4) 合并结果（CF 优先，MR 兜底）→ 5) 每查到一个 project 就 `app.emit("mods-preload-update", ...)` → 6) 写入持久化缓存
  - 持久化文件缓存：`.Molaunch/cache/preload_mods/{version_id}.json`，6 小时 TTL + 版本号 gating（版本号变化强制刷新，key=`ModrinthHash + VanillaVersion + ModLoaders`）
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
- 前端 ModTab.vue `handleShowInfo` 改造为三级 fallback：
  1. **零延迟路径**：`mod.project` 已被 `preload_mods_detail_cmd` 后台预加载填充 → 直接弹 ResourceDetail（与主流启动器分支一致）
  2. **并发 fallback**：预加载未就绪（用户点太快）或预加载失败 → `Promise.any` 并发请求 CF + MR，谁先成功用谁
  3. **本地信息**：无 slug 或两个平台都查不到 → 弹本地信息弹窗 + 百科搜索按钮（与主流启动器分支一致）
- onMounted 启动预加载事件监听（必须在 `loadMods` 之前，避免错过早期事件）→ `loadMods` → `prefetchVersionContext` → `preloadModsDetail`（后台异步，不阻塞 UI）；onUnmounted 停止监听

#### 整合包安装：完整流程（后端 + 前端）
- 新增 `install_modpack` Tauri 命令（backend: src-tauri/src/commands/community/install.rs），参考主流启动器整合包安装实现
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
- 进度共享 download_state：安装全程走 `state.download_state`（与版本下载共用），4 个加权阶段（下载整合包 10 / 解析 1 / 下载依赖 40 / 复制配置 5），由 `DownloadPanel` + 下载管理页面统一展示（不单独做弹窗/页面）
- 返回 `InstallModpackResult`（format/gameVersion/loader/loaderVersion/archivePath/instanceDir），前端据此调用 `install_merged` 安装游戏本体 + 加载器
- 前端新增 `handleInstallModpack`（frontend: src/components/community/ResourceDetail.vue），两段式调用：`installModpack`（整合包专属部分）→ `installMerged`（游戏本体+加载器），共享同一 `download_state`，DownloadPanel 连续展示
- 前端详情页版本按钮按 `resource_type` 分流：ModPack 类型显示「安装」按钮（RocketLaunchIcon）调用 `handleInstallModpack`，其他类型显示「下载」按钮（ArrowDownTrayIcon）调用 `handleDownload`（SwapType=9 安装 / 8 另存为）

#### 整合包安装：并发下载进度与失败诊断修复
- 修复「下载速度/已下载字节」始终为 0：原 `download_single_file` 用 `resp.bytes().await` 一次性加载，从不更新 `bytes_downloaded`/`bytes_total`/`global_speed`（backend: src-tauri/src/commands/community/install.rs）
- 改为流式下载：`download_single_file_multi` 边接收边写文件，通过 `AtomicU64` 实时累积 `bytes_done`/`bytes_total`，前端能看到下载途中速度、累计字节持续增长（而非每个文件完成才跳一次）
- 新增 300ms 独立定时器任务：流式下载过程中定时调用 `update_modpack_progress` 刷新 `state.download_state` 的 stage 与 global 字段，参考主流启动器 300ms `DispatcherTimer` 轮询机制
- 修复 Modrinth 整合包部分文件下载失败（如 123/129 卡住）：原代码仅取 `downloads[0]`，遇到失效 URL 直接失败。改为传入 `downloads` 全部 URL 数组，按顺序尝试直到成功，多源回退
- 修复日志不完整：原失败时只 push 到 errors 列表，无任何 log_info。改为每个失败立即打印 `target_path`、尝试过的 URL 列表、错误信息；函数返回前汇总打印完整失败列表（编号 + URL + 错误），便于排查
- 失败错误信息从「仅第一个」改为「失败总数 + 首个错误」，方便快速判断是网络问题还是部分文件问题

#### install_merged 阶段错位修复 + 释放资源 hash 检查修复
- 修复「加载器安装」阶段堆很久但 MC 本体/库/assets 没分阶段显示（backend: src-tauri/src/commands/version/install.rs）：原 install_merged 只重置已有 stages 的 progress 但不替换列表内容，整合包安装的 4 个阶段（下载整合包/解析/依赖/overrides）残留，install_merged push 一个「加载器安装」正好凑 5 个。download_version_full 调 stage_callback(0..4) 与 stages 错位，导致前 4 个已 Finished 阶段被反复改 Loading，最后所有进度都堆到 stage[4]「加载器安装」
- 修复方式：install_merged 启动时清空 stages 重新设置为标准 5 阶段（版本清单/版本信息/客户端/库文件/资源文件，与 download_version_full 的 stage_callback 索引对应），按需追加「加载器安装」。修复后用户能看到 MC 本体、库文件、资源文件各自独立的进度条
- 修复「释放嵌入资源 hash 不匹配但实际文件不存在」警告（backend: src-tauri/src/resources.rs）：原代码只判断 `target_path.exists()`，当目标文件不存在但 `.sha256` 校验文件残留时，会读到旧 hash 触发「不匹配」警告。改为同时检查 target 和 hash 文件存在：两者都存在且匹配才跳过；只有一方存在时打印「缓存状态不一致」；两者都不存在时静默首次释放

#### 下载进度阶段分组展示
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

#### 资源打包方式改造
- 重写 `resources` 模块：所有外部资源在编译时通过 `include_str!`/`include_bytes!` 嵌入二进制，运行时零文件 IO 读取，彻底废弃此前基于 `env!("CARGO_MANIFEST_DIR")` 拼路径的实现（backend: src-tauri/src/resources.rs）
- 修复发布版 bug：原实现 `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")` 在打包后指向开发机路径，用户机器上不存在，导致首次启动释放默认配置和 Forge 安装器全部失败
- 嵌入的资源清单：
  - 文本资源（`include_str!`）：`defaults/config.ini`、`defaults/instance.ini`、`defaults/setup.ini`、`moddata.txt`
  - 二进制资源（`include_bytes!`）：`forge-installer.jar`、`java-wrapper.jar`
- 二进制资源释放带 sha256 校验：只在目标文件不存在或 hash 不匹配时写盘，同目录写 `{name}.sha256` 校验文件用于下次启动比对，避免每次启动重复写大文件拖慢启动、触发杀软误报
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

#### 详情页"转到 MC百科"改为直链跳转
- 此前用搜索 URL `https://search.mcmod.cn/s?key=<name>`，现改为直链 `https://www.mcmod.cn/class/<id>.html`（backend: src-tauri/src/minecraft/community/mcmod.rs, src-tauri/src/commands/community/detail.rs）
- 研究发现：不调 API，完全靠 moddata.txt 的**行号**作为 class id：第 N 行 → class id = N，URL 即 `https://www.mcmod.cn/class/<N>.html`
- 关键设计：moddata.txt 空行也占用行号（`i += 1` 在 `Continue For` 之前），此前 MoLaunch 解析时 `continue` 跳过空行且不计数，会导致行号错位；已修复
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
- 启动时主动 `DirectoryUtils.Create(PathExeFolder & ".minecraft\versions\")`
- `lib.rs` 的 `run()` 在 `AppState::new()` 后增加：`resolve_game_dir(&config.game_dir).join("versions")` 不存在时 `create_dir_all`
- `open_game_dir` 命令增加防御性创建：路径不存在时先 `create_dir_all` 再打开，避免启动时创建失败导致命令仍报错

#### "转到 MC百科"按钮只对 Mod 类型显示
- MC 百科数据库（moddata.txt）只包含 Mod 条目，仅对 Mod/数据包类型显示该按钮（frontend: src/components/community/ResourceDetail.vue）
- 添加 `v-if="project.resource_type === 'Mod'"` 条件，整合包/资源包/光影/数据包不显示该按钮

#### 整合包安装逻辑研究结论（未实现，待后续开发）
- 下载页"整合包"分类的资源安装流程：
  - 下载原始包到 `versions\{InstanceName}\原始整合包.{zip|mrpack}`
  - 调用 `ModpackInstall` 解压 → 解析 manifest.json/modrinth.index.json → 复制 overrides → 批量下载依赖 mods → 安装游戏本体
- 不同资源类型的处理差异：
  - Mod/资源包/光影/数据包：只下载到对应子文件夹（mods/resourcepacks/shaderpacks），不解压不解析
  - 整合包"安装"：完整走 ModpackInstall 流程
  - 整合包"另存为"：仅下载原始压缩包，不做后续处理
- MoLaunch 当前整合包下载流程与"另存为"一致，缺少完整的安装流程，待后续实现
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
- 新增 default-skin.ts：内置 Steve/Alex 默认皮肤纹理（canvas 生成），根据 UUID 计算皮肤类型（frontend: src/utils/default-skin.ts）
- SkinAvatar 支持离线账号：传 login_type='Offline' 时使用默认皮肤（frontend: src/components/common/SkinAvatar.vue）

#### 账号切换
- 账号切换改为单卡片左右滑动切换（一次只显示一个账号），支持拖动/滚轮切换，带平滑动画，末尾预留新增账号卡片（frontend: src/components/home/AccountSelector.vue）

### 修复
- 修复皮肤头像裁剪：overlay 层（头发层）现在检查透明像素，避免空白覆盖脸部（frontend: src/components/common/SkinAvatar.vue）
- 修复 willReadFrequently 警告：所有频繁读取 getImageData 的 canvas 添加 { willReadFrequently: true } 选项
- 修复 SkinManager loadInfo 中 getSkinCapeInfo 失败导致后续步骤不执行的问题（每步独立 try-catch）

### 新增

#### 微软登录
- 微软 OAuth 2.0 Web 授权码登录流程（Authorization Code Flow，使用 login.live.com 旧版端点 + 公共 Client ID `00000000402b5328`，与 HMCL 等主流启动器一致）
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
- 玩家皮肤头像加载：从 profile_json 的 skins[].url 获取皮肤 PNG 地址，下载后用 canvas 裁剪 (8,8,8,8) 脸层 + (40,8,8,8) 头发层）
- 皮肤 PNG 全图显示（直接从 textures.minecraft.net 下载，不依赖第三方渲染服务）
- 当前形象信息展示（用户名、皮肤模型 Steve/Alex、当前披风）
- 皮肤上传功能（multipart/form-data，支持 classic/slim 两种模型，后端直接读取本地文件避免 base64 转换）
- 披风列表展示与装备/取消（28 种披风中文名映射）
- 修改密码快捷入口（跳转 `https://account.live.com/password/Change`）
- 修改用户名快捷入口（跳转 `https://www.minecraft.net/zh-hans/msaprofile/mygames/editprofile`）
- `SkinAvatar` 组件：通用皮肤头像组件，canvas 裁剪支持高清皮肤（128x64 等），加载失败时回退到首字母渐变占位符
- `SkinManager` 弹窗：完整的皮肤/披风管理界面
- `AccountSelector` 接入真实皮肤头像显示与皮肤管理入口

### 变更
- 微软登录采用 Device Code Flow（设备码流程），使用 v2.0 consumers 端点 + MoLaunch 独立 Azure 应用 Client ID
- 认证存储从单一 `auth.json` 文件改为 Windows 注册表分字段存储（路径 `HKCU\Software\MoLaunch`）
- 敏感字段（Token、用户名、UUID 等）单独 SDK DES 加密，非敏感字段（登录类型）明文存储
- Token 刷新使用 login.live.com 旧版端点（与主流启动器一致）
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
  仍被追踪；`Cargo.toml` 两处依赖注释含 "参考第三方启动器" 字样需移除。
- 变更：
  - [.gitignore](.gitignore)：`# Rust` 段新增 `src-tauri/Cargo.lock` 显式排除（与全局 `Cargo.lock` 并列）。
  - `git rm --cached src-tauri/Cargo.lock`：从索引移除，本地文件保留，下次 push 后云端不再追踪。
  - `git rm --cached -r logo_data/`：同上清理 3 个 logo 数据文件。
  - [src-tauri/Cargo.toml](src-tauri/Cargo.toml)：`notify` 与 `windows` 依赖注释移除 "参考第三方启动器 ..." 字样。

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

*本文档最后更新于 2026-08-05*
