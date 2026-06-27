# 镜像源配置说明

## 概述

SDK 内置了 Mojang 官方源和 BMCLAPI 镜像源。通过 `init` 时传入 `mirror_url`，所有下载请求（版本清单、版本 JSON、客户端 JAR、库文件、资源文件、加载器）都会走镜像源。

## 配置方式

```c
MC_SDK_MCConfig config = {
    .game_dir = ".minecraft",
    .max_download_threads = 8,
    .mirror_url = "https://bmclapi2.bangbang93.com",  // 镜像源 URL
    .log_level = 3,
};
MC_SDK_SDKHandle *handle = mc_sdk_init(&config);
```

### mirror_url 取值

| 值 | 效果 |
|---|---|
| `NULL` | 所有请求走官方源 |
| `"https://bmclapi2.bangbang93.com"` | 全部走 BMCLAPI（内置匹配） |
| 其他 URL | 按 BMCLAPI 路径格式构建自定义镜像 |

## 内置镜像源

| 名称 | URL | 说明 |
|------|-----|------|
| Mojang (官方) | `https://launchermeta.mojang.com` | 默认源 |
| BMCLAPI | `https://bmclapi2.bangbang93.com` | 国内镜像，速度快 |

## 请求路径映射

### 使用官方源 (mirror_url = NULL)

| 资源类型 | URL 格式 |
|----------|----------|
| 版本清单 | `https://launchermeta.mojang.com/mc/game/version_manifest_v2.json` |
| 版本 JSON | `https://launchermeta.mojang.com/v1/packages/{id}/{id}.json` |
| 客户端 JAR | `https://launchermeta.mojang.com/v1/packages/{id}/client.jar` |
| 库文件 | `https://libraries.minecraft.net/{path}` |
| 资源文件 | `https://resources.download.minecraft.net/{prefix}/{hash}` |

### 使用 BMCLAPI (mirror_url = "https://bmclapi2.bangbang93.com")

| 资源类型 | URL 格式 |
|----------|----------|
| 版本清单 | `https://bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json` |
| 版本 JSON | `https://bmclapi2.bangbang93.com/version/{id}/{id}.json` |
| 客户端 JAR | `https://bmclapi2.bangbang93.com/version/{id}/client.jar` |
| 库文件 | `https://bmclapi2.bangbang93.com/maven/{path}` |
| 资源文件 | `https://bmclapi2.bangbang93.com/assets/{prefix}/{hash}` |
| Forge 版本列表 | `https://bmclapi2.bangbang93.com/forge/maven-metadata.xml` |
| NeoForge 版本列表 | `https://bmclapi2.bangbang93.com/neoforge/meta/api/maven/details/releases/net/neoforged/neoforge` |
| OptiFine 版本列表 | `https://bmclapi2.bangbang93.com/optifine/versionList` |
| Fabric 版本列表 | `https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader` |
| Fabric 安装器 | `https://bmclapi2.bangbang93.com/maven/net/fabricmc/fabric-installer/...` |
| LiteLoader 版本列表 | `https://bmclapi2.bangbang93.com/maven/com/mumfrey/liteloader/versions.json` |

### 使用自定义镜像 (mirror_url = "https://custom.mirror.com")

| 资源类型 | URL 格式 |
|----------|----------|
| 版本清单 | `https://custom.mirror.com/mc/game/version_manifest_v2.json` |
| 版本 JSON | `https://custom.mirror.com/version/{id}/{id}.json` |
| 客户端 JAR | `https://custom.mirror.com/version/{id}/client.jar` |
| 库文件 | `https://custom.mirror.com/maven/{path}` |
| 资源文件 | `https://custom.mirror.com/assets/{prefix}/{hash}` |
| 加载器版本列表 | 走 BMCLAPI（加载器 API 固定） |

自定义镜像源按 BMCLAPI 路径格式构建 URL。如果自定义镜像的路径格式与 BMCLAPI 不同，可能无法正常工作。

## 各函数镜像源支持

| 函数 | 支持镜像 | 说明 |
|------|----------|------|
| `mc_download_version` | ✅ | 版本清单、JSON、JAR、库文件、资源文件 |
| `mc_list_versions` | ✅ | 版本清单（需传入 handle，传 NULL 走官方） |
| `mc_list_forge_versions` | ✅ | mirror_url 有值就走 BMCLAPI |
| `mc_list_neoforge_versions` | ✅ | mirror_url 有值就走 BMCLAPI |
| `mc_list_fabric_versions` | ✅ | mirror_url 有值就走 BMCLAPI fabric-meta |
| `mc_list_optifine_versions` | ✅ | 始终走 BMCLAPI（无官方 JSON API） |
| `mc_list_liteloader_versions` | ✅ | mirror_url 有值就走 BMCLAPI |
| `mc_install_forge` | ✅ | 安装器下载 |
| `mc_install_neoforge` | ✅ | 安装器下载 |
| `mc_install_optifine` | ✅ | 安装器下载 |
| `mc_install_liteloader` | ✅ | 安装器下载 |
| `mc_install_fabric` | ✅ | mirror_url 有值就走 BMCLAPI |
| `mc_install_merged` | ✅ | 所有下载步骤 |

## 注意事项

1. **`mc_list_versions` 签名变更**（v0.1.8）：新增 `handle` 参数，传 `NULL` 走官方源
2. **自定义镜像路径格式**：按 BMCLAPI 格式构建，如果镜像路径不同则无法工作
3. **加载器 API**：Forge/NeoForge/LiteLoader 的 `use_mirror` 由 `mirror_url` 是否为 NULL 决定
4. **OptiFine**：始终走 BMCLAPI，因为 optifine.net 没有公开 JSON API
5. **Fabric**：mirror_url 有值时走 BMCLAPI `/fabric-meta` 端点
