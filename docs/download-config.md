# 下载配置说明

## 概述

SDK 提供以下下载控制选项：

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `mirror_url_meta` | `const char*` | `NULL` | 版本清单/JSON 镜像源 URL |
| `mirror_url_download` | `const char*` | `NULL` | 资源下载镜像源 URL |
| `max_download_threads` | `uint32_t` | `8` | 最大并发下载线程数 |
| `max_download_speed` | `uint64_t` | `0` | 最大下载速度 (bytes/sec)，0=不限制 |

## 配置示例

```c
MC_SDK_MCConfig config = {
    .game_dir = ".minecraft",
    .log_level = 3,

    // 下载控制
    .max_download_threads = 8,                        // 8 线程并发
    .max_download_speed = 10 * 1024 * 1024,           // 限速 10 MB/s

    // 镜像源（分级控制）
    .mirror_url_meta = "https://bmclapi2.bangbang93.com",     // 版本信息走镜像
    .mirror_url_download = "https://bmclapi2.bangbang93.com", // 资源下载走镜像

    // 旧字段（向后兼容，新字段为 NULL 时回退到此值）
    .mirror_url = NULL,
};
MC_SDK_SDKHandle *handle = mc_sdk_init(&config);
```

## 分级镜像源

SDK 将请求分为两类，可分别控制是否走镜像：

### mirror_url_meta — 版本信息

控制版本清单和版本 JSON 的获取源。

| 请求 | URL 格式 (官方) | URL 格式 (BMCLAPI) |
|------|----------------|-------------------|
| 版本清单 | `launchermeta.mojang.com/mc/game/version_manifest_v2.json` | `bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json` |
| 版本 JSON | `launchermeta.mojang.com/v1/packages/{id}/{id}.json` | `bmclapi2.bangbang93.com/version/{id}/{id}.json` |
| Forge 版本列表 | `maven.minecraftforge.net/...` | `bmclapi2.bangbang93.com/forge/maven-metadata.xml` |
| NeoForge 版本列表 | `maven.neoforged.net/...` | `bmclapi2.bangbang93.com/neoforge/meta/api/maven/...` |
| Fabric 版本列表 | `meta.fabricmc.net/...` | `bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader` |
| OptiFine 版本列表 | 无官方 API | `bmclapi2.bangbang93.com/optifine/versionList` |
| LiteLoader 版本列表 | `dl.liteloader.com/...` | `bmclapi2.bangbang93.com/maven/com/mumfrey/liteloader/versions.json` |

### mirror_url_download — 资源下载

控制客户端 JAR、库文件、资源文件、加载器安装器的下载源。

| 请求 | URL 格式 (官方) | URL 格式 (BMCLAPI) |
|------|----------------|-------------------|
| 客户端 JAR | `launchermeta.mojang.com/v1/packages/{id}/client.jar` | `bmclapi2.bangbang93.com/version/{id}/client.jar` |
| 库文件 | `libraries.minecraft.net/{path}` | `bmclapi2.bangbang93.com/maven/{path}` |
| 资源文件 | `resources.download.minecraft.net/{prefix}/{hash}` | `bmclapi2.bangbang93.com/assets/{prefix}/{hash}` |
| Forge 安装器 | `maven.minecraftforge.net/...` | `bmclapi2.bangbang93.com/maven/...` |
| NeoForge 安装器 | `maven.neoforged.net/...` | `bmclapi2.bangbang93.com/maven/...` |
| Fabric 安装器 | `maven.fabricmc.net/...` | `bmclapi2.bangbang93.com/maven/...` |
| OptiFine 安装器 | `optifine.net/...` | `bmclapi2.bangbang93.com/optifine/...` |
| LiteLoader 安装器 | `dl.liteloader.com/...` | `bmclapi2.bangbang93.com/maven/...` |

### 回退逻辑

```
mirror_url_meta   = NULL ? → 回退到 mirror_url
mirror_url_download = NULL ? → 回退到 mirror_url
```

旧代码只设置 `mirror_url`，新字段为 NULL 时自动回退，无需修改。

## 各函数使用的镜像配置

| 函数 | mirror_url_meta | mirror_url_download |
|------|:---------------:|:------------------:|
| `mc_list_versions` | ✅ | — |
| `mc_list_forge_versions` | ✅ | — |
| `mc_list_neoforge_versions` | ✅ | — |
| `mc_list_fabric_versions` | ✅ | — |
| `mc_list_optifine_versions` | ✅ (始终 BMCLAPI) | — |
| `mc_list_liteloader_versions` | ✅ | — |
| `mc_download_version` | ✅ | ✅ |
| `mc_install_forge` | — | ✅ |
| `mc_install_neoforge` | — | ✅ |
| `mc_install_fabric` | — | ✅ |
| `mc_install_optifine` | — | ✅ |
| `mc_install_liteloader` | — | ✅ |
| `mc_install_merged` | ✅ | ✅ |

## 下载线程控制

`max_download_threads` 控制库文件、资源文件等批量下载的并发数。

```c
// 4 线程（适合低带宽或低配置机器）
config.max_download_threads = 4;

// 16 线程（适合高带宽）
config.max_download_threads = 16;
```

| 值 | 效果 |
|---|---|
| `0` | 使用默认值 8 |
| `1~32` | 指定并发数 |

## 下载速度限制

`max_download_speed` 限制所有下载的总速度，防止占满带宽影响其他进程。

```c
// 不限制（默认）
config.max_download_speed = 0;

// 限制 5 MB/s
config.max_download_speed = 5 * 1024 * 1024;

// 限制 1 MB/s
config.max_download_speed = 1 * 1024 * 1024;
```

| 值 | 效果 |
|---|---|
| `0` | 不限制速度 |
| `> 0` | 限制为指定 bytes/sec |

速度限制使用滑动窗口算法，每秒检查一次累计下载量。当达到限制时，下载会暂停直到下一秒窗口开始。

## 典型配置

### 国内用户（推荐）

```c
MC_SDK_MCConfig config = {
    .game_dir = ".minecraft",
    .max_download_threads = 8,
    .max_download_speed = 0,
    .mirror_url_meta = "https://bmclapi2.bangbang93.com",
    .mirror_url_download = "https://bmclapi2.bangbang93.com",
    .mirror_url = NULL,
    .log_level = 3,
};
```

### 国外用户（官方源）

```c
MC_SDK_MCConfig config = {
    .game_dir = ".minecraft",
    .max_download_threads = 8,
    .max_download_speed = 0,
    .mirror_url_meta = NULL,
    .mirror_url_download = NULL,
    .mirror_url = NULL,
    .log_level = 3,
};
```

### 低带宽限速

```c
MC_SDK_MCConfig config = {
    .game_dir = ".minecraft",
    .max_download_threads = 4,
    .max_download_speed = 2 * 1024 * 1024,  // 2 MB/s
    .mirror_url_meta = "https://bmclapi2.bangbang93.com",
    .mirror_url_download = "https://bmclapi2.bangbang93.com",
    .mirror_url = NULL,
    .log_level = 3,
};
```

## 获取配置的 FFI 函数

```c
// 获取镜像源 URL（返回的字符串需通过 mc_sdk_free_string 释放）
char* meta_url = mc_config_get_mirror_meta(handle);     // NULL 表示走官方
char* dl_url = mc_config_get_mirror_download(handle);    // NULL 表示走官方

// 获取下载限制
uint32_t threads = mc_config_get_max_download_threads(handle);
uint64_t speed = mc_config_get_max_download_speed(handle);  // bytes/sec, 0=不限制
```

## 注意事项

1. **向后兼容**：旧字段 `mirror_url` 仍然可用，新字段为 NULL 时回退
2. **OptiFine**：版本列表始终走 BMCLAPI（无官方 JSON API）
3. **自定义镜像**：按 BMCLAPI 路径格式构建，如果格式不同可能无法工作
4. **速度限制精度**：滑动窗口每秒检查一次，实际速度可能有短暂波动
5. **线程数**：建议 4~16，过高可能导致连接不稳定
