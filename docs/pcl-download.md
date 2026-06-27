# PCL2 下载流程交互分析

> 基于 `Plain Craft Launcher 2` 源码分析，覆盖从版本选择点击"开始下载"到下载完成的完整交互流程。

---

## 一、整体交互流程概览

```
版本列表页 → [点击版本] → 选择页(Mod加载器) → [点击"开始下载"] → 返回版本列表页 + 右下角圆形按钮出现
                                                                    ↓
                                                          [下载完成] → 按钮消失，Toast 提示
                                                          [点击按钮] → 进入下载管理页面(带进度)
```

---

## 二、阶段详细分析

### 阶段 1：版本列表页 (`PanMinecraft`)

**页面位置**: `PageDownloadInstall.xaml:11` — `PanMinecraft` StackPanel

- 版本列表通过 `LoadMinecraft_OnFinish()` 方法（`PageDownloadInstall.xaml.vb:443`）动态生成
- 版本按类型分组：`正式版`、`预览版`、`远古版`、`愚人节版`
- 最新版本卡片置顶显示
- 每个版本是一个 `MyListItem`，点击触发 `MinecraftSelected` 事件

### 阶段 2：选择页 (`PanSelect`)

**触发**: `MinecraftSelected()` (`PageDownloadInstall.xaml.vb:147`) → `EnterSelectPage()` (`PageDownloadInstall.xaml.vb:40`)

**页面布局** (`PageDownloadInstall.xaml:16-158`):

```
PanSelect (StackPanel, 默认隐藏)
  ├── 顶部信息栏 (MyCard, 高62px)
  │     ├── 返回按钮 (BtnBack) — 左侧箭头图标
  │     ├── 版本图标 (ImgLogo) — 根据选择的加载器变化
  │     └── 版本名输入框 (TextSelectName) — 可编辑
  ├── 红/黄色警告提示 (MyHint)
  │     ├── HintFabricAPI — "不安装 Fabric API 大多数Mod无法使用"
  │     ├── HintOptiFabric — "必须安装 OptiFabric"
  │     ├── HintOptiFabricOld — 老版本 OptiFabric 提示
  │     └── HintModOptiFine — "OptiFine 与部分Mod兼容性不佳"
  ├── Forge 卡片 (CardForge) — 可展开选择版本
  ├── NeoForge 卡片 (CardNeoForge)
  ├── Fabric 卡片 (CardFabric)
  ├── Fabric API 卡片 (CardFabricApi)
  ├── OptiFine 卡片 (CardOptiFine)
  ├── OptiFabric 卡片 (CardOptiFabric)
  └── LiteLoader 卡片 (CardLiteLoader)
```

**进入动画** (`PageDownloadInstall.xaml.vb:84-117`):

- `PanMinecraft`: 透明度 1→0 (70ms)，X 平移 -50px (90ms)
- `PanSelect`: 透明度 0→1 (70ms，延迟100ms)，X 平移入场 (160ms，EaseOutFluent 缓动)
- 禁用的 `BtnStart` 被启用并显示

**"开始下载"按钮** (`PageDownloadInstall.xaml:161-163`):

```xml
<local:MyExtraTextButton x:Name="BtnStart" Text="开始下载"
    HorizontalAlignment="Center" VerticalAlignment="Bottom"
    LogoScale="0.95" Logo="[下载图标SVG路径]" />
```

- 固定在页面底部居中
- 只有当 `TextSelectName.IsValidated` 为 True 时才可点击

### 阶段 3：点击"开始下载" (`BtnStart_Click`)

**代码位置**: `PageDownloadInstall.xaml.vb:1125-1153`

**执行流程**:

#### 3.1 版本隔离检查 (1127-1134)

```vb
If (SelectedForge IsNot Nothing OrElse SelectedNeoForge IsNot Nothing OrElse SelectedFabric IsNot Nothing) AndAlso
   (Settings.Get(Of Integer)("LaunchArgumentIndieV2") = 0 OrElse Settings.Get(Of Integer)("LaunchArgumentIndieV2") = 2) Then
    If MyMsgBox("你尚未开启版本隔离...") = 1 Then Return
End If
```

- 如果选择了 Forge/NeoForge/Fabric 且未开启版本隔离，弹出 `MyMsgBox` 警告
- 用户可选择"取消下载"或"继续"

#### 3.2 构建安装请求 (1136-1149)

创建 `McInstallRequest` 对象，包含：

| 字段 | 说明 |
|------|------|
| `NewInstanceName` | 新版本名称（来自 TextSelectName） |
| `VersionFolder` | 版本文件夹路径 |
| `MinecraftJson` | 原版 JSON 下载 URL |
| `MinecraftName` | 原版版本号 |
| `OptiFineEntry` | OptiFine 条目 |
| `ForgeEntry` | Forge 条目 |
| `NeoForgeEntry` | NeoForge 条目 |
| `FabricVersion` | Fabric 版本号 |
| `FabricApi` | Fabric API 条目 |
| `OptiFabric` | OptiFabric 条目 |
| `LiteLoaderEntry` | LiteLoader 条目 |

#### 3.3 调用 `McInstall(Request)` (`ModDownloadLib.vb:1895-1913`)

```
McInstall
  ├── McInstallLoader(Request) — 构建子加载器列表
  ├── 创建 LoaderCombo 组合加载器
  │     .OnStateChanged = McInstallState
  ├── Loader.Start(Request.VersionFolder) — 启动下载！
  ├── LoaderTaskbarAdd(Loader) — 加入任务栏列表
  ├── FrmMain.BtnExtraDownload.ShowRefresh() — 刷新按钮可见性
  └── FrmMain.BtnExtraDownload.Ribble() — 触发波纹动画效果
```

#### 3.4 退出选择页

调用 `ExitSelectPage()` (1152)，执行退出动画，回到版本列表视图。

### 阶段 4：右下角圆形按钮 (`BtnExtraDownload`)

**定义位置**: `FormMain.xaml:172-173`

```xml
<local:MyExtraButton x:Name="BtnExtraDownload" HorizontalAlignment="Right"
    VerticalAlignment="Center" ToolTip="下载管理" Visibility="Collapsed"
    Logo="[下载图标SVG路径]" />
```

**显示条件** (`FormMain.xaml.vb:1580-1582`):

```vb
Private Function BtnExtraDownload_ShowCheck() As Boolean
    Return HasDownloadingTask() AndAlso Not PageCurrent = PageType.DownloadManager
End Function
```

- 有正在下载的任务 **且** 当前不在下载管理页面 → 显示

**按钮控件特性** (`MyExtraButton.xaml.vb`):

| 特性 | 说明 |
|------|------|
| 显示动画 | 缩放从 0.3→1 (500ms, EaseOutFluent + EaseOutBack 弹性效果) |
| 隐藏动画 | 缩放到 0 (100ms)，然后高度收缩 (400ms) |
| 进度环 | 底部 `Progress` 属性控制的弧形进度条，通过 `Rect` 裁剪实现 |
| 波纹效果 | `Ribble()` 方法，从中心扩散一圈白色半透明圆形 |
| 按压缩放 | 点击时缩小到 0.85x，松开恢复 |

**点击行为** (`FormMain.xaml.vb:1577-1578`):

```vb
Private Sub BtnExtraDownload_Click(...)
    PageChange(PageType.DownloadManager)
End Sub
```

**进度更新** (`ModLoader.vb:621`):

```vb
RunInUi(Sub() FrmMain.BtnExtraDownload.Progress = LoaderTaskbarProgress)
```

- `LoaderTaskbarProgress` 每 300ms 从所有任务的平均进度计算
- 使用平滑算法：`progress = old * 0.9 + new * 0.1`
- 同时更新 Windows 任务栏进度条 (`TaskbarItemInfo`)

### 阶段 5：下载管理页面

**页面结构**: 左右分栏 (`PageSpeedLeft` + `PageSpeedRight`)

#### 左栏布局 (`PageSpeedLeft.xaml`)

```
┌─────────────────┐
│     总进度       │
│   ───────────   │
│    45.67 %      │  ← LabProgress
│                 │
│    下载速度      │
│   ───────────   │
│   1.23 MB/s     │  ← LabSpeed
│                 │
│    剩余文件      │
│   ───────────   │
│       12        │  ← LabFile
│                 │
│    剩余线程      │  (调试模式才显示)
│   ───────────   │
│     3 / 64      │  ← LabThread
└─────────────────┘
```

- 每 300ms 刷新一次 (`Watcher` 定时器)
- 从 `NetManager.Speed` 获取实时下载速度

#### 右栏布局 (`PageSpeedRight.xaml` + `PageSpeedLeft.xaml.vb`)

- 动态生成 `MyCard` 卡片，每个下载任务一个卡片
- 卡片内每行对应一个子任务，左侧状态图标 + 右侧任务名：

| 状态 | 图标 | 说明 |
|------|------|------|
| `Finished` | ✅ 绿色对勾 Path | 已完成 |
| `Failed` | ❌ 红色叉号 Path | 失败 |
| `Loading` | `98%` TextBlock | 正在进行，显示百分比 |
| `Waiting` | ⋯ 三个点 Path | 等待中 |

- 右上角有取消按钮 (×)，点击后动画移除卡片并中断下载

**任务完成/失败处理** (`PageSpeedLeft.xaml.vb:101-104`):

```vb
Case LoadState.Finished, LoadState.Interrupted
    AniDispose(Card, True, AddressOf TryReturnToHome)
```

- 卡片动画销毁
- 若没有剩余卡片 → 自动返回上一页 (`PageBack`)

#### 安装状态回调 (`ModDownloadLib.vb:1859-1874`)

```vb
Public Sub McInstallState(Loader As LoaderBase)
    Select Case Loader.State
        Case LoadState.Finished
            Hint(Loader.Name & "成功！", HintType.Green)  ' 左下角绿色 Toast
        Case LoadState.Failed
            MyMsgBox(Loader.Error.GetDisplay(True), ...)   ' 弹窗提示错误
        Case LoadState.Interrupted
            Hint(Loader.Name & "已取消！", HintType.Blue)  ' 蓝色 Toast
    End Select
    ' 刷新版本列表
    LoaderFolderRun(McInstanceListLoader, ...)
End Sub
```

---

## 三、后台下载执行链 (`McInstallLoader`)

**代码位置**: `ModDownloadLib.vb:1919-2053`

创建的子加载器链（按顺序执行）：

| 顺序 | 加载器名称 | 阻塞 | 权重 | 说明 |
|------|-----------|------|------|------|
| 1 | 添加忽略标识 | 否 | — | 在版本文件夹写入 `.pclignore`，防止列表误显示 |
| 2 | 下载 Fabric API | 否 | 3 | 并行下载 |
| 3 | 下载 OptiFabric | 否 | 3 | 并行下载 |
| 4 | 下载原版 Minecraft | 视情况 | 39 | 主下载任务，权重最高 |
| 5 | 下载 OptiFine | 视情况 | 16-24 | 可作为 Mod 或独立安装 |
| 6 | 下载 Forge | 视情况 | 25 | 依赖原版完成 |
| 7 | 下载 NeoForge | 视情况 | 25 | 依赖原版完成 |
| 8 | 下载 LiteLoader | 视情况 | 1 | 依赖 Fabric 完成 |
| 9 | 下载 Fabric | 是 | 2 | 阻塞后续任务 |
| 10 | 安装游戏 (合并JSON+迁移文件) | 是 | 2 | 合并各加载器 JSON，迁移 libraries |
| 11 | 下载游戏支持库文件 | 是 | 8 | 分析并补全所有依赖库 |
| 12 | 删除忽略标识 | — | — | 移除 `.pclignore` |

### 安装游戏子步骤 (第10步)

```vb
' 合并 JSON
MergeJson(VersionFolder, VersionFolder, OptiFineFolder, ...)
Task.Progress = 0.2

' 迁移文件
DirectoryUtils.Copy(TempMcFolder & "libraries", McFolderSelected & "libraries")
Task.Progress = 0.8

' 创建 Mod 和资源包文件夹
DirectoryUtils.Create(ModsFolder)
DirectoryUtils.Create(ResourcepacksFolder)
```

---

## 四、关键数据流图

```
用户点击"开始下载"
    │
    ▼
BtnStart_Click() [PageDownloadInstall.xaml.vb:1125]
    │
    ├─ 版本隔离检查 → 可能弹窗
    │
    ▼
McInstall(Request) [ModDownloadLib.vb:1895]
    │
    ├─ McInstallLoader() → 构建 LoaderBase 列表
    │
    ├─ LoaderCombo.Start() → 异步启动所有子加载器
    │
    ├─ LoaderTaskbarAdd() → 注册到 LoaderTaskbar 全局列表
    │
    ├─ BtnExtraDownload.ShowRefresh()
    │     └─ ShowCheck() 返回 True
    │         └─ 按钮显示 (弹性缩放动画 0.3→1)
    │
    ├─ BtnExtraDownload.Ribble() → 波纹扩散效果
    │
    └─ ExitSelectPage()
          └─ PanSelect 淡出 + PanMinecraft 淡入
              └─ 用户看到版本列表

═══════════════════════════════════════════
[后台下载进行中... 每300ms刷新]
═══════════════════════════════════════════
    │
    ├─ LoaderTaskbarProgressRefresh() [ModLoader.vb:602]
    │     ├─ 计算平均进度 (平滑: old*0.9 + new*0.1)
    │     ├─ 更新 BtnExtraDownload.Progress (弧形进度环)
    │     └─ 更新 TaskbarItemInfo (Windows 任务栏进度条)
    │
    ├─ PageSpeedLeft.Watcher() [如果在下载管理页面]
    │     ├─ 刷新左栏: 总进度/速度/文件数/线程数
    │     └─ 调用 TaskRefresh() 刷新右栏卡片
    │
    └─ LoaderTaskbar 列表维护
          └─ 完成/中断的任务自动移出列表

═══════════════════════════════════════════
[下载完成 / 失败 / 取消]
═══════════════════════════════════════════
    │
    ▼
McInstallState(Loader) [ModDownloadLib.vb:1859]
    │
    ├─ LoadState.Finished
    │     ├─ Hint("xxx 安装成功！", Green) — 左下角绿色 Toast
    │     └─ 刷新版本列表
    │
    ├─ LoadState.Failed
    │     ├─ MyMsgBox(错误详情) — 弹窗提示
    │     └─ 清理版本文件夹 (如果未被独立启动)
    │
    └─ LoadState.Interrupted
          ├─ Hint("xxx 已取消！", Blue) — 蓝色 Toast
          └─ 清理版本文件夹
    │
    ▼
BtnExtraDownload.ShowRefresh()
    └─ ShowCheck() 返回 False → 按钮隐藏动画
```

---

## 五、Toast 提示系统集成

下载完成/失败/取消时会触发 Toast 提示，显示在窗口**左下角**。

| 类型 | 颜色 | 触发场景 |
|------|------|---------|
| `Blue` | 蓝色渐变 | 下载取消 |
| `Green` | 绿色渐变 | 下载成功 |
| `Red` | 红色渐变 | 下载错误 |

Toast 特性：

- 从左侧弹性滑入 (400ms, EaseOutElastic)
- 显示时长与文本长度成正比：`(800 + 文本长度 × 180)ms`
- 相同文本不重复堆叠，触发闪烁动画
- 最多同时显示 20 条

详细规范参见 `PCL2-Toast-Design.md`。

---

## 六、关键文件索引

| 文件 | 职责 |
|------|------|
| `Pages/PageDownload/PageDownloadInstall.xaml` | 安装页面 XAML 布局 |
| `Pages/PageDownload/PageDownloadInstall.xaml.vb` | 版本选择、加载器选择、启动安装逻辑 |
| `Pages/PageDownload/ModDownloadLib.vb` | `McInstall`/`McInstallLoader`/`McInstallState` 核心下载逻辑 |
| `Controls/MyExtraButton.xaml.vb` | 右下角圆形按钮控件（进度环、波纹、动画） |
| `FormMain.xaml` | 主窗口布局，定义 `BtnExtraDownload` |
| `FormMain.xaml.vb` | 页面切换逻辑、`BtnExtraDownload_Click` |
| `Modules/Base/ModLoader.vb` | `LoaderTaskbarAdd`、`LoaderTaskbarProgressRefresh` 任务栏管理 |
| `Pages/PageSpeedLeft.xaml` | 下载管理左栏布局 |
| `Pages/PageSpeedLeft.xaml.vb` | 下载管理实时刷新逻辑 |
| `Pages/PageSpeedRight.xaml` | 下载管理右栏布局 |
