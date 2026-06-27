# PCL2 安装流程分析

> 基于 [Plain Craft Launcher 2](https://github.com/Hex-Dragon/PCL2) 源码分析

## 整体架构

PCL2 的安装系统采用**两页切换 + 合并安装**的设计：

```
┌──────────────────────────────────┐
│  第一页：Minecraft 版本选择        │  PageDownloadInstall.xaml.vb
│  分类展示：正式版/预览版/远古版     │  PanMinecraft
│  点击版本 → 滑入第二页             │
└──────────────┬───────────────────┘
               ▼
┌──────────────────────────────────┐
│  第二页：加载器选择                │  PanSelect
│  可展开卡片：                      │
│  OptiFine / LiteLoader / Forge   │
│  NeoForge / Fabric / FabricAPI   │
│  OptiFabric                      │
│  版本名预览（可编辑）              │
│  [开始安装] 按钮                   │
└──────────────────────────────────┘
```

## 核心文件

| 文件 | 职责 |
|------|------|
| `Pages/PageDownload/PageDownloadInstall.xaml.vb` | UI 交互、版本列表展示、加载器选择、兼容性检查 |
| `Pages/PageDownload/ModDownloadLib.vb` | `McInstallRequest` 结构体、`McInstall()` 入口、`McInstallLoader()` 构建加载器链、`MergeJson()` 合并版本 JSON |
| `Modules/Minecraft/ModDownload.vb` | 各加载器的具体下载逻辑 |
| `Modules/Minecraft/ModMinecraft.vb` | MC 版本解析、版本目录管理 |

## 第一页：版本列表

### 加载流程

```
页面加载 → LoaderInit()
  → DlClientListLoader.Start()  // 拉取 Mojang 版本清单
  → 同时预加载 OptiFine/Fabric/NeoForge 列表
```

### 版本分类

版本清单返回后，按类型分组：

| 类型 | 判断逻辑 |
|------|----------|
| 正式版 | `type == "release"` 或 snapshot 但 id 以 `1.` 开头且不含 pre/combat |
| 预览版 | `type == "snapshot"` |
| 远古版 | `type == "old_alpha"` 或 `old_beta` |
| 愚人节版 | id 匹配已知愚人节版本，或发布日期为 4 月 1 日 |

每个分类用 `MyCard` 可折叠卡片展示，最新版本置顶。

### 版本列表项

```vb
' 为每个版本生成列表项
Private Function McDownloadListItem(Version As JObject, ...) As MyListItem
    ' 显示: 版本号 + 发布日期 + 版本图标
    ' 点击事件: MinecraftSelected()
End Function
```

## 第二页：加载器选择

### 页面切换动画

```vb
Private Sub EnterSelectPage()
    ' 隐藏 PanMinecraft，显示 PanSelect
    ' 滑入动画 (160ms)
    ' 启动 Forge 版本列表加载（需要 MC 版本号作为参数）
    ' 启动 Fabric API / OptiFabric 加载
End Sub
```

### 加载器卡片结构

每个加载器用 `MyCard` 可展开卡片，包含：

```
┌─ CardForge ─────────────────────────────────┐
│  [展开/折叠]                                  │
│  ┌─ PanForgeInfo ──────────────────────────┐ │
│  │  当前状态: 可以添加 / 已选择 xxx / 不兼容  │ │
│  └─────────────────────────────────────────┘ │
│  ┌─ PanForge ──────────────────────────────┐ │
│  │  版本列表项 (可点击选择)                   │ │
│  │  47.2.0                                 │ │
│  │  47.1.0                                 │ │
│  │  ...                                    │ │
│  └─────────────────────────────────────────┘ │
│  [清除选择按钮]                               │
└──────────────────────────────────────────────┘
```

### 可见性控制

根据 MC 版本号控制卡片是否显示：

| 加载器 | 可见条件 |
|--------|----------|
| OptiFine | 始终显示（但可能显示错误信息） |
| LiteLoader | MC < 1.13 (`VanillaDrop < 130`) |
| Forge | MC ≥ 1.5.1 且使用标准版本格式 |
| NeoForge | MC ≥ 1.20.1（基于 `releaseTime >= 2023-06-11`） |
| Fabric | MC > 1.13 (`VanillaDrop > 130`) |
| Fabric API | 选择了 Fabric |
| OptiFabric | 同时选择了 Fabric 和 OptiFine |

### 兼容性检查

实时检查加载器冲突，显示在卡片上：

```vb
' Forge 兼容性检查
Private Function LoadForgeGetError() As String
    If SelectedNeoForge IsNot Nothing Then Return "与 NeoForge 不兼容"
    If SelectedFabric IsNot Nothing Then Return "与 Fabric 不兼容"
    If SelectedOptiFine IsNot Nothing AndAlso ... Then Return "与 OptiFine 不兼容"
    Return Nothing  ' 兼容
End Function

' NeoForge 兼容性检查
Private Function LoadNeoForgeGetError() As String
    If SelectedOptiFine IsNot Nothing Then Return "与 OptiFine 不兼容"
    If SelectedForge IsNot Nothing Then Return "与 Forge 不兼容"
    If SelectedFabric IsNot Nothing Then Return "与 Fabric 不兼容"
    Return Nothing
End Function
```

完整兼容性矩阵：

| 加载器 A | 加载器 B | 结果 |
|----------|----------|------|
| Forge | Fabric | ❌ 不兼容 |
| Forge | NeoForge | ❌ 不兼容 |
| NeoForge | Fabric | ❌ 不兼容 |
| NeoForge | OptiFine | ❌ 不兼容 |
| Forge 1.13~1.14.3 | OptiFine | ❌ 不兼容 |
| Fabric 1.20.5+ | OptiFine | ❌ 不兼容 |
| Fabric + OptiFine | OptiFabric | ✅ 自动选择 |

### 自动选择逻辑

选择 Fabric 后，自动勾选 Fabric API：
```vb
Private Sub FabricApi_Loaded()
    If Not AutoSelectedFabricApi Then
        AutoSelectedFabricApi = True
        ' 自动选择第一个兼容版本
        FabricApi_Selected(PanFabricApi.Children(0), Nothing)
    End If
End Sub
```

选择 Fabric + OptiFine 后，自动勾选 OptiFabric（MC 1.14~1.15 除外）。

### 版本名自动生成

```vb
Private Function GetSelectName() As String
    Dim Name As String = VanillaName  ' "1.20.1"
    If SelectedFabric IsNot Nothing Then Name += "-Fabric " & ...      ' "-Fabric 0.15.0"
    If SelectedForge IsNot Nothing Then Name += "-Forge_" & ...        ' "-Forge_47.2.0"
    If SelectedNeoForge IsNot Nothing Then Name += "-NeoForge_" & ...  ' "-NeoForge_47.2"
    If SelectedOptiFine IsNot Nothing Then Name += "-OptiFine_" & ...  ' "-OptiFine_HD_U_I7"
    Return Name  ' "1.20.1-Forge_47.2.0-OptiFine_HD_U_I7"
End Function
```

用户可手动编辑版本名，编辑后不再自动更新。

## 安装执行

### 提交安装请求

```vb
Private Sub BtnStart_Click()
    ' 版本隔离检查
    If Not 版本隔离已开启 Then
        If MyMsgBox("推荐开启版本隔离...") = 1 Then Return
    End If
    
    ' 构建请求
    Dim Request As New McInstallRequest With {
        .NewInstanceName = TextSelectName.Text,
        .VersionFolder = $"{McFolderSelected}versions\{InstanceName}\",
        .MinecraftJson = VanillaData("url").ToString,
        .MinecraftName = VanillaName,
        .OptiFineEntry = SelectedOptiFine,
        .ForgeEntry = SelectedForge,
        .NeoForgeEntry = SelectedNeoForge,
        .FabricVersion = SelectedFabric,
        .FabricApi = SelectedFabricApi,
        .OptiFabric = SelectedOptiFabric,
        .LiteLoaderEntry = SelectedLiteLoader
    }
    McInstall(Request)
End Sub
```

### McInstall 入口

```vb
Public Function McInstall(Request As McInstallRequest) As Boolean
    Dim SubLoaders = McInstallLoader(Request)  ' 构建加载器链
    Dim Loader As New LoaderCombo(Of String)(Request.NewInstanceName & " 安装", SubLoaders)
    Loader.OnStateChanged = AddressOf McInstallState  ' 状态回调
    Loader.Start(Request.VersionFolder)  ' 开始执行
End Function
```

### 加载器链构建 (McInstallLoader)

这是核心逻辑，按依赖关系构建有序的加载器列表：

```vb
Public Function McInstallLoader(Request As McInstallRequest) As List(Of LoaderBase)
    Dim LoaderList As New List(Of LoaderBase)
    
    ' 1. 添加忽略标识 (阻止 PCL 在安装完成前显示此版本)
    LoaderList.Add(New LoaderTask("添加忽略标识", Sub() Write(.pclignore)))
    
    ' 2. Fabric API (非阻塞，可与后续并行)
    If Request.FabricApi IsNot Nothing Then
        LoaderList.Add(New LoaderDownload("下载 Fabric API", ...) With {
            .ProgressWeight = 3, .Block = False
        })
    End If
    
    ' 3. OptiFabric (非阻塞)
    If Request.OptiFabric IsNot Nothing Then
        LoaderList.Add(New LoaderDownload("下载 OptiFabric", ...) With {
            .ProgressWeight = 3, .Block = False
        })
    End If
    
    ' 4. 原版 MC (权重39，阻塞取决于是否有加载器)
    LoaderList.Add(New LoaderCombo("下载原版", McDownloadClientLoader(...)) With {
        .ProgressWeight = 39,
        .Block = 无任何加载器时为 True
    })
    
    ' 5. OptiFine (阻塞取决于 Forge/Fabric)
    If Request.OptiFineEntry IsNot Nothing Then
        If OptiFineAsMod Then
            ' 作为 Mod 下载 (与 Forge/Fabric 共存时)
            LoaderList.Add(New LoaderDownload("下载 OptiFine", ...) With {
                .ProgressWeight = 16, .Block = ...
            })
        Else
            ' 独立安装
            LoaderList.Add(New LoaderCombo("下载 OptiFine", McDownloadOptiFineLoader(...)) With {
                .ProgressWeight = 24, .Block = ...
            })
        End If
    End If
    
    ' 6. Forge (阻塞取决于 Fabric)
    If Request.ForgeVersion IsNot Nothing Then
        LoaderList.Add(New LoaderCombo("下载 Forge", McDownloadForgelikeLoader(...)) With {
            .ProgressWeight = 25, .Block = ...
        })
    End If
    
    ' 7. NeoForge (阻塞取决于 Forge/Fabric)
    If Request.NeoForgeVersion IsNot Nothing Then
        LoaderList.Add(New LoaderCombo("下载 NeoForge", McDownloadForgelikeLoader(...)) With {
            .ProgressWeight = 25, .Block = ...
        })
    End If
    
    ' 8. LiteLoader (阻塞取决于 Fabric)
    If Request.LiteLoaderEntry IsNot Nothing Then
        LoaderList.Add(New LoaderCombo("下载 LiteLoader", ...) With {
            .ProgressWeight = 1, .Block = ...
        })
    End If
    
    ' 9. Fabric (阻塞=True，必须等前面完成)
    If Request.FabricVersion IsNot Nothing Then
        LoaderList.Add(New LoaderCombo("下载 Fabric", ...) With {
            .ProgressWeight = 2, .Block = True
        })
    End If
    
    ' 10. 合并安装 (阻塞=True)
    LoaderList.Add(New LoaderTask("安装游戏", Sub()
        MergeJson(...)           ' 合并所有版本 JSON
        DirectoryUtils.Copy(...) ' 迁移库文件到共享目录
        DirectoryUtils.Create(mods\)  ' 创建 Mod 文件夹
        DirectoryUtils.Create(resourcepacks\)  ' 创建资源包文件夹
    End Sub) With {.ProgressWeight = 2, .Block = True})
    
    ' 11. 补全库文件 (合并后的新 JSON 可能引用新的库)
    LoaderList.Add(New LoaderCombo("下载游戏支持库文件", ...) With {
        .ProgressWeight = 8
    })
    
    ' 12. 删除忽略标识
    LoaderList.Add(New LoaderTask("删除忽略标识", Sub() Delete(.pclignore)))
    
    Return LoaderList
End Function
```

### 依赖关系与阻塞

`Block` 属性控制执行顺序：

```
Fabric API ─────────────────────┐
OptiFabric ─────────────────────┤
原版 MC ────────────────────────┤ (并行)
OptiFine ───────────────────────┤
Forge ──────────────────────────┤
NeoForge ───────────────────────┤
LiteLoader ─────────────────────┤
                                ▼
                        Fabric (Block=True)
                                ▼
                        合并安装 (Block=True)
                                ▼
                        补全库文件
                                ▼
                        删除忽略标识
```

- `Block = False`：可与后续任务并行执行
- `Block = True`：必须等前面的任务全部完成

### 进度权重

各步骤的 `ProgressWeight` 决定总进度占比：

| 步骤 | 权重 | 占比 |
|------|------|------|
| Fabric API | 3 | 2.9% |
| OptiFabric | 3 | 2.9% |
| 原版 MC | 39 | 37.5% |
| OptiFine (独立) | 24 | 23.1% |
| OptiFine (作为Mod) | 16 | 15.4% |
| Forge | 25 | 24.0% |
| NeoForge | 25 | 24.0% |
| LiteLoader | 1 | 1.0% |
| Fabric | 2 | 1.9% |
| 合并安装 | 2 | 1.9% |
| 补全库文件 | 8 | 7.7% |

## 合并安装 (MergeJson)

### 临时目录策略

所有加载器先安装到临时目录，避免污染正式版本目录：

```vb
Dim TempMcFolder As String = RequestTaskTempFolder(...)
' 例如: .minecraft\versions\Temp\12345678\

' 各加载器安装到临时目录的子文件夹
OptiFineFolder = TempMcFolder & "versions\" & OptiFineEntry.InstanceName
ForgeFolder    = TempMcFolder & "versions\forge-" & ForgeVersion
NeoForgeFolder = TempMcFolder & "versions\neoforge-" & NeoForgeVersion
FabricFolder   = TempMcFolder & "versions\fabric-loader-" & FabricVersion & "-" & MCName
```

### OptiFine 作为 Mod

当 OptiFine 与任何加载器共存时，自动降级为 Mod：

```vb
Dim Modable As Boolean = Request.FabricVersion IsNot Nothing OrElse
                         Request.ForgeEntry IsNot Nothing OrElse
                         Request.NeoForgeEntry IsNot Nothing OrElse
                         Request.LiteLoaderEntry IsNot Nothing

Dim OptiFineAsMod As Boolean = Request.OptiFineEntry IsNot Nothing AndAlso Modable

If OptiFineAsMod Then
    Logger.Info("OptiFine 将作为 Mod 进行下载")
    OptiFineFolder = ModsTempFolder  ' 放入 mods 文件夹而非 versions
End If
```

### JSON 合并算法

```vb
Private Sub MergeJson(OutputFolder, MinecraftFolder,
                       OptiFineFolder, OptiFineAsMod,
                       ForgeFolder, NeoForgeFolder,
                       FabricFolder, LiteLoaderFolder)
    ' 1. 读取所有 JSON
    Dim MinecraftJson = GetJson(File.ReadAllText(MinecraftJsonPath))
    Dim OptiFineJson = GetJson(File.ReadAllText(OptiFineJsonPath))
    Dim ForgeJson = GetJson(File.ReadAllText(ForgeJsonPath))
    ' ...
    
    ' 2. 合并 minecraftArguments
    Dim AllArguments = MinecraftJson("minecraftArguments") & " " &
                       OptiFineJson("minecraftArguments") & " " &
                       ForgeJson("minecraftArguments") & " " &
                       NeoForgeJson("minecraftArguments") & " " &
                       LiteLoaderJson("minecraftArguments")
    
    ' 3. 去重参数
    Dim RealArguments = SplitArguments.Distinct.Join(" ")
    
    ' 4. 深度合并 JSON (JObject.Merge)
    OutputJson = MinecraftJson
    If HasOptiFine Then OutputJson.Merge(OptiFineJson)
    If HasForge Then OutputJson.Merge(ForgeJson)
    If HasNeoForge Then OutputJson.Merge(NeoForgeJson)
    If HasLiteLoader Then OutputJson.Merge(LiteLoaderJson)
    If HasFabric Then OutputJson.Merge(FabricJson)
    
    ' 5. 覆盖关键字段
    OutputJson("id") = OutputName
    OutputJson.Remove("inheritsFrom")  ' 不再继承
    OutputJson.Remove("jar")           ' 不再引用原版 JAR
    OutputJson("minecraftArguments") = RealArguments
    
    ' 6. 保存
    File.WriteAllText(OutputJsonPath, OutputJson.ToString)
    File.Copy(MinecraftJar, OutputJar)  ' 复制原版 JAR
End Sub
```

合并后的 JSON 是一个**完全独立的版本**，不再依赖 `inheritsFrom`。

### 文件迁移

```vb
' 合并安装步骤
Sub InstallGame(Task)
    ' 1. 合并 JSON
    MergeJson(VersionFolder, ...)
    
    ' 2. 迁移库文件到共享目录
    DirectoryUtils.Copy(TempMcFolder & "libraries", McFolderSelected & "libraries")
    
    ' 3. 创建 Mod 和资源包文件夹
    DirectoryUtils.Copy(ModsTempFolder, ModsFolder)
    DirectoryUtils.Create(ResourcepacksFolder)
End Sub
```

## 状态管理

### 安装状态回调

```vb
Public Sub McInstallState(Loader As LoaderBase)
    Select Case Loader.State
        Case LoadState.Finished
            WriteIni("PCL.ini", "InstanceCache", "")  ' 清空缓存
            Hint("安装成功！", HintType.Green)
        Case LoadState.Failed
            MyMsgBox(Loader.Error.GetDisplay(True), "安装失败", IsWarn:=True)
        Case LoadState.Interrupted
            Hint("已取消！", HintType.Blue)
    End Select
    
    ' 失败时清理临时文件夹
    McInstallFailedClearFolder(Loader)
    
    ' 刷新版本列表
    LoaderFolderRun(McInstanceListLoader, ...)
End Sub
```

### 失败清理

```vb
Public Sub McInstallFailedClearFolder(Loader)
    If Loader.State = LoadState.Failed OrElse Loader.State = LoadState.Interrupted Then
        If DirectoryUtils.Exists(Loader.Input & "saves\") Then
            Logger.Warn("版本已被独立启动，不清理")
        Else
            DirectoryUtils.Delete(Loader.Input)  ' 删除版本文件夹
        End If
    End If
End Sub
```

## 总结

### PCL2 安装流程图

```
用户选择 MC 版本
       │
       ▼
进入加载器选择页面
       │
       ├── 选择 Forge ──────┐
       ├── 选择 NeoForge ───┤  实时兼容性检查
       ├── 选择 Fabric ─────┤  自动选择依赖
       ├── 选择 OptiFine ───┤
       └── 选择 LiteLoader ┘
       │
       ▼
点击 [开始安装]
       │
       ├── 版本隔离检查
       │
       ▼
构建加载器链 (McInstallLoader)
       │
       ├── 1. 添加忽略标识
       ├── 2. 下载 Fabric API (并行)
       ├── 3. 下载 OptiFabric (并行)
       ├── 4. 下载原版 MC (并行)
       ├── 5. 下载 OptiFine (并行)
       ├── 6. 下载 Forge (并行)
       ├── 7. 下载 NeoForge (并行)
       ├── 8. 下载 LiteLoader (并行)
       ├── 9. 下载 Fabric (阻塞)
       ├── 10. 合并 JSON + 迁移文件 (阻塞)
       ├── 11. 补全库文件
       └── 12. 删除忽略标识
       │
       ▼
安装完成，刷新版本列表
```

### 关键设计决策

1. **合并安装而非链式安装**：所有加载器装到临时目录，最后合并为一个独立版本
2. **JSON 深度合并**：使用 `JObject.Merge` 合并多个加载器的 JSON
3. **OptiFine 作为 Mod**：与加载器共存时自动降级
4. **依赖阻塞**：通过 `Block` 属性控制执行顺序
5. **进度加权**：各步骤按权重计算总进度
6. **忽略标识**：安装中的版本不会显示在版本列表
7. **临时目录隔离**：避免安装失败污染正式目录
