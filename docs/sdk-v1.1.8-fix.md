# McSDK 0.1.8 - 补全缺失能力

> 版本: 0.1.8
> 日期: 2026-06-27

## 背景

McSDK 在 0.1.7 及之前版本已经覆盖了 Minecraft 启动器的基本功能（下载、安装加载器、认证、启动等），但与 PCL2 等成熟启动器相比，仍缺少几个关键能力：

| 能力 | PCL2 | SDK (0.1.7) | SDK (0.1.8) |
|------|------|-------------|-------------|
| 查询可用 Forge 版本列表 | ✅ | ❌ | ✅ |
| 查询可用 NeoForge 版本列表 | ✅ | ❌ | ✅ |
| 查询可用 Fabric 版本列表 | ✅ | ❌ | ✅ |
| 查询可用 OptiFine 版本列表 | ✅ | ❌ | ✅ |
| 查询可用 LiteLoader 版本列表 | ✅ | ❌ | ✅ |
| 加载器兼容性校验 | ✅ | ❌ | ✅ |
| 一键合并安装（MC + 多加载器） | ✅ | ❌ | ✅ |

## 新增内容

### 1. 版本列表查询 API

为每个加载器提供版本列表查询接口，方便启动器 UI 展示可选版本下拉框。

- `mc_list_forge_versions` - 获取指定 MC 版本可用的 Forge 版本列表
- `mc_list_neoforge_versions` - 获取指定 MC 版本可用的 NeoForge 版本列表
- `mc_list_fabric_versions` - 获取可用的 Fabric Loader 版本列表
- `mc_list_optifine_versions` - 获取可用的 OptiFine 版本列表
- `mc_list_liteloader_versions` - 获取指定 MC 版本可用的 LiteLoader 版本列表

### 2. 加载器兼容性校验

`mc_validate_loaders` 函数，一次性校验用户选择的加载器组合是否兼容。

兼容性规则：

| 组合 | 结果 |
|------|------|
| Forge + Fabric | ❌ 不兼容 |
| Forge + NeoForge | ❌ 不兼容 |
| NeoForge + Fabric | ❌ 不兼容 |
| NeoForge + OptiFine | ❌ 不兼容 |
| Forge 1.13~1.14.3 + OptiFine | ❌ 不兼容 |
| Fabric 1.20.5+ + OptiFine | ❌ 不兼容 |
| 其他组合 | ✅ 兼容 |

### 3. 合并安装 API

`mc_install_merged` 函数，一次调用完成 Minecraft + 所有加载器的安装。

安装顺序：下载原版 → 安装 Forge → 安装 NeoForge → 安装 Fabric → 安装 OptiFine → 安装 LiteLoader

通过 `FFIMergedInstallRequest` 结构体指定各加载器版本，传 NULL 表示跳过。

---

## API 参考

### mc_list_forge_versions

```c
int32_t mc_list_forge_versions(const SDKHandle* handle, const char* mc_version, char** result_out);
```

获取指定 MC 版本可用的 Forge 版本列表。

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `mc_version` | `const char*` | Minecraft 版本号 (如 `1.20.1`) |
| `result_out` | `char**` | 输出参数，接收 JSON 字符串 |

**返回值:** `0` 成功，非 0 错误码

**返回 JSON 格式:** `["47.2.0", "47.1.0", "47.0.0", ...]`

---

### mc_list_neoforge_versions

```c
int32_t mc_list_neoforge_versions(const SDKHandle* handle, const char* mc_version, char** result_out);
```

获取指定 MC 版本可用的 NeoForge 版本列表。

**参数:** 同 `mc_list_forge_versions`

**返回 JSON 格式:**

```json
[
  {"version": "47.2", "recommended": false},
  {"version": "47.1", "recommended": true}
]
```

---

### mc_list_fabric_versions

```c
int32_t mc_list_fabric_versions(const SDKHandle* handle, char** result_out);
```

获取可用的 Fabric Loader 版本列表。无需指定 MC 版本。

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `result_out` | `char**` | 输出参数，接收 JSON 字符串 |

**返回 JSON 格式:**

```json
[
  {"version": "0.15.0", "stable": true},
  {"version": "0.15.0-beta.1", "stable": false}
]
```

---

### mc_list_optifine_versions

```c
int32_t mc_list_optifine_versions(const SDKHandle* handle, char** result_out);
```

获取可用的 OptiFine 版本列表。

**参数:** 同 `mc_list_fabric_versions`

**返回 JSON 格式:**

```json
[
  {"display_name": "1.20.1 HD U I7", "is_preview": false},
  {"display_name": "1.20.1 HD U I7 pre1", "is_preview": true}
]
```

---

### mc_list_liteloader_versions

```c
int32_t mc_list_liteloader_versions(const SDKHandle* handle, const char* mc_version, char** result_out);
```

获取指定 MC 版本可用的 LiteLoader 版本列表。

**参数:** 同 `mc_list_forge_versions`

**返回 JSON 格式:** `["1.20.1", "1.19.4", ...]`

---

### mc_validate_loaders

```c
int32_t mc_validate_loaders(
    const char* mc_version,
    const char* forge_version,
    const char* neoforge_version,
    const char* fabric_version,
    const char* optifine_version
);
```

校验加载器组合兼容性。无需 SDK 句柄。

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `mc_version` | `const char*` | Minecraft 版本号 (必填) |
| `forge_version` | `const char*` | Forge 版本号 (NULL 表示未选择) |
| `neoforge_version` | `const char*` | NeoForge 版本号 (NULL 表示未选择) |
| `fabric_version` | `const char*` | Fabric 版本号 (NULL 表示未选择) |
| `optifine_version` | `const char*` | OptiFine 版本号 (NULL 表示未选择) |

**返回值:**
- `0`: 兼容
- 非 0: 不兼容 (错误码)，通过 `mc_sdk_last_error()` 获取冲突描述

---

### mc_install_merged

```c
int32_t mc_install_merged(
    const SDKHandle* handle,
    const FFIMergedInstallRequest* request,
    const void* callback,
    void* user_data
);
```

合并安装：一次调用完成 MC + 所有加载器的安装。

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `request` | `const FFIMergedInstallRequest*` | 合并安装请求 |
| `callback` | `const void*` | 进度回调函数指针 (可选) |
| `user_data` | `void*` | 用户数据指针 |

**返回值:** `0` 成功，非 0 错误码

**FFIMergedInstallRequest 结构体:**

```c
typedef struct {
    const char* mc_version;           // 必填，Minecraft 版本号
    const char* forge_version;        // 可选，NULL=跳过
    const char* neoforge_version;     // 可选，NULL=跳过
    const char* fabric_version;       // 可选，NULL=跳过
    const char* optifine_version;     // 可选，NULL=跳过
    const char* liteloader_version;   // 可选，NULL=跳过
    const char* instance_name;        // 可选，NULL=使用 mc_version
} FFIMergedInstallRequest;
```

---

## 使用示例

### 示例 1: 查询版本列表

```c
#include "mc_sdk.h"
#include <stdio.h>

void list_loader_versions(SDKHandle* handle) {
    // 查询 Forge 版本
    char* forge_json;
    if (mc_list_forge_versions(handle, "1.20.1", &forge_json) == 0) {
        printf("Forge versions: %s\n", forge_json);
        mc_sdk_free_string(forge_json);
    }

    // 查询 NeoForge 版本
    char* neoforge_json;
    if (mc_list_neoforge_versions(handle, "1.20.1", &neoforge_json) == 0) {
        printf("NeoForge versions: %s\n", neoforge_json);
        mc_sdk_free_string(neoforge_json);
    }

    // 查询 Fabric 版本
    char* fabric_json;
    if (mc_list_fabric_versions(handle, &fabric_json) == 0) {
        printf("Fabric versions: %s\n", fabric_json);
        mc_sdk_free_string(fabric_json);
    }

    // 查询 OptiFine 版本
    char* optifine_json;
    if (mc_list_optifine_versions(handle, &optifine_json) == 0) {
        printf("OptiFine versions: %s\n", optifine_json);
        mc_sdk_free_string(optifine_json);
    }

    // 查询 LiteLoader 版本
    char* liteloader_json;
    if (mc_list_liteloader_versions(handle, "1.20.1", &liteloader_json) == 0) {
        printf("LiteLoader versions: %s\n", liteloader_json);
        mc_sdk_free_string(liteloader_json);
    }
}
```

### 示例 2: 兼容性校验

```c
// 用户选择: MC 1.20.1 + Forge 47.2.0 + OptiFine I7
int32_t code = mc_validate_loaders("1.20.1", "47.2.0", NULL, NULL, "HD U I7");
if (code == 0) {
    printf("加载器组合兼容!\n");
} else {
    const ErrorInfo* error = mc_sdk_last_error();
    printf("不兼容: %s\n", error->message);
    mc_sdk_free_error((ErrorInfo*)error);
}

// 用户选择: MC 1.20.1 + Forge + Fabric (不兼容)
code = mc_validate_loaders("1.20.1", "47.2.0", NULL, "0.15.0", NULL);
if (code != 0) {
    const ErrorInfo* error = mc_sdk_last_error();
    printf("错误: %s\n", error->message);  // "Forge 和 Fabric 不能同时安装"
}
```

### 示例 3: 合并安装

```c
void progress_callback(const char* stage, uintptr_t current, uintptr_t total,
                       uint64_t bytes_downloaded, uint64_t bytes_total,
                       uint64_t speed, uintptr_t files_remaining, void* user_data) {
    printf("[%s] %zu/%zu", stage, current, total);
    if (bytes_total > 0) {
        printf(" | %.1f%%", (double)bytes_downloaded / bytes_total * 100.0);
    }
    if (speed > 0) {
        printf(" | %.1f MB/s", (double)speed / 1024 / 1024);
    }
    printf("\n");
}

int install_modded_minecraft(SDKHandle* handle) {
    // 先校验兼容性
    int32_t valid = mc_validate_loaders("1.20.1", "47.2.0", NULL, NULL, NULL);
    if (valid != 0) {
        const ErrorInfo* error = mc_sdk_last_error();
        printf("加载器冲突: %s\n", error->message);
        mc_sdk_free_error((ErrorInfo*)error);
        return -1;
    }

    // 合并安装: MC 1.20.1 + Forge 47.2.0
    FFIMergedInstallRequest request = {
        .mc_version = "1.20.1",
        .forge_version = "47.2.0",
        .neoforge_version = NULL,
        .fabric_version = NULL,
        .optifine_version = NULL,
        .liteloader_version = NULL,
        .instance_name = NULL  // 使用 "1.20.1" 作为实例名
    };

    int32_t code = mc_install_merged(handle, &request, progress_callback, NULL);
    if (code == 0) {
        printf("安装完成!\n");
    }
    return code;
}
```

### 示例 4: 完整启动流程（使用合并安装）

```c
#include "mc_sdk.h"
#include <stdio.h>

int main() {
    // 1. 初始化
    MCConfig config = {
        .game_dir = "C:/Users/Test/.minecraft",
        .max_download_threads = 8,
        .mirror_url = NULL,
        .log_level = 3,
    };
    SDKHandle* handle = mc_sdk_init(&config);
    if (!handle) return 1;

    // 2. 校验加载器组合
    if (mc_validate_loaders("1.20.1", "47.2.0", NULL, NULL, NULL) != 0) {
        printf("加载器冲突!\n");
        mc_sdk_free(handle);
        return 1;
    }

    // 3. 合并安装
    FFIMergedInstallRequest req = {
        .mc_version = "1.20.1",
        .forge_version = "47.2.0",
    };
    if (mc_install_merged(handle, &req, NULL, NULL) != 0) {
        printf("安装失败!\n");
        mc_sdk_free(handle);
        return 1;
    }

    // 4. 离线登录
    FFIAuthResult auth;
    mc_auth_offline("Player", &auth);

    // 5. 启动
    mc_launch_game(handle, auth.username, auth.uuid,
                   auth.access_token, "1.20.1", 2048);

    mc_auth_free_result(&auth);
    mc_sdk_free(handle);
    return 0;
}
```

---

## 与 PCL2 的对比

| 功能 | PCL2 实现方式 | McSDK 0.1.8 |
|------|--------------|-------------|
| 获取 Forge 版本列表 | 从 BMCLAPI/官方获取 | `mc_list_forge_versions` |
| 获取 NeoForge 版本列表 | 从 NeoForge API 获取 | `mc_list_neoforge_versions` |
| 获取 Fabric 版本列表 | 从 Fabric Meta API 获取 | `mc_list_fabric_versions` |
| 获取 OptiFine 版本列表 | 从 OptiFine 官网获取 | `mc_list_optifine_versions` |
| 获取 LiteLoader 版本列表 | 从 LiteLoader 官方获取 | `mc_list_liteloader_versions` |
| 加载器兼容性检查 | UI 层逻辑判断 | `mc_validate_loaders` |
| 一键安装 | 自动安装按钮 | `mc_install_merged` |
