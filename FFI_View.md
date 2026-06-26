# McSDK FFI 接口文档

> 版本: 0.1.7
> 最后更新: 2026-06-25

本文档详细说明 McSDK 所有 FFI 接口的调用方法、参数说明和响应数据格式。

---

## 目录

- [SDK 生命周期](#sdk-生命周期)
- [错误处理](#错误处理)
- [设备标识](#设备标识)
- [Token 加密/解密](#token-加密解密)
- [认证模块](#认证模块)
- [皮肤管理](#皮肤管理)
- [模组安装](#模组安装)
- [Mod 管理](#mod-管理)
- [版本管理](#版本管理)
- [Java 检测](#java-检测)
- [下载与启动](#下载与启动)
- [资源平台 API](#资源平台-api)
- [更新系统](#更新系统)
- [配置管理](#配置管理)
- [网络管理](#网络管理)
- [数据结构](#数据结构)

---

## SDK 生命周期

### mc_sdk_init

初始化 SDK，必须在使用其他接口前调用。

```c
SDKHandle* mc_sdk_init(const MCConfig* config);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `config` | `const MCConfig*` | SDK 配置指针，不能为 NULL |

**返回值:**
- 成功: SDK 句柄指针
- 失败: `NULL`，可通过 `mc_sdk_last_error()` 获取错误信息

**MCConfig 结构体:**

```c
typedef struct {
    const char* game_dir;           // 游戏目录路径 (必填)
    uint32_t max_download_threads;  // 最大下载线程数 (默认 8)
    const char* mirror_url;         // 镜像源 URL (可选，NULL 使用官方源)
    uint32_t log_level;             // 日志级别 (0=off, 1=error, 2=warn, 3=info, 4=debug, 5=trace)
    const char* curseforge_api_key; // CurseForge API Key (可选)
} MCConfig;
```

**调用示例:**

```c
MCConfig config = {
    .game_dir = "C:/Users/Test/.minecraft",
    .max_download_threads = 8,
    .mirror_url = NULL,
    .log_level = 3,
    .curseforge_api_key = NULL
};

SDKHandle* handle = mc_sdk_init(&config);
if (handle == NULL) {
    ErrorInfo* error = mc_sdk_last_error();
    printf("Init failed: %s\n", error->message);
    mc_sdk_free_error(error);
    return;
}
```

---

### mc_sdk_free

释放 SDK 句柄，程序结束前必须调用。

```c
void mc_sdk_free(SDKHandle* handle);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `SDKHandle*` | SDK 句柄指针 |

**调用示例:**

```c
mc_sdk_free(handle);
```

---

### mc_sdk_version

获取 SDK 版本号。

```c
const char* mc_sdk_version();
```

**返回值:**
- 版本号字符串指针 (静态内存，无需释放)

**调用示例:**

```c
const char* version = mc_sdk_version();
printf("SDK Version: %s\n", version);
```

---

### mc_sdk_is_initialized

检查 SDK 是否已初始化。

```c
int32_t mc_sdk_is_initialized();
```

**返回值:**
- `1`: 已初始化
- `0`: 未初始化

---

### mc_sdk_get_game_dir

获取游戏目录路径。

```c
char* mc_sdk_get_game_dir(const SDKHandle* handle);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄指针 |

**返回值:**
- 成功: 游戏目录路径字符串指针 (需通过 `mc_sdk_free_string` 释放)
- 失败: `NULL`

---

### mc_sdk_free_string

释放字符串内存。所有返回 `char*` 的接口（除 `mc_sdk_version`）都需要调用此函数释放内存。

```c
void mc_sdk_free_string(char* str);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `str` | `char*` | 字符串指针 |

---

## 错误处理

### mc_sdk_last_error

获取最后一次错误信息。

```c
const ErrorInfo* mc_sdk_last_error();
```

**返回值:**
- ErrorInfo 结构体指针 (静态内存，无需释放)

**ErrorInfo 结构体:**

```c
typedef struct {
    int32_t code;      // 错误码
    const char* message; // 错误消息
} ErrorInfo;
```

**错误码:**

| 错误码 | 说明 |
|--------|------|
| 0 | 成功 |
| 1 | 无效参数 |
| 2 | 空指针 |
| 3 | 内存不足 |
| 4 | IO 错误 |
| 5 | 内部 panic |
| 100 | 网络错误 |
| 101 | 下载失败 |
| 200 | 认证失败 |
| 300 | 版本未找到 |
| 400 | 启动失败 |
| 401 | Java 未找到 |
| 500 | 文件未找到 |

---

### mc_sdk_free_error

释放错误信息内存。

```c
void mc_sdk_free_error(ErrorInfo* error);
```

---

### mc_sdk_clear_error

清除当前错误状态。

```c
void mc_sdk_clear_error();
```

---

## 设备标识

### mc_get_device_id

获取设备唯一标识。

```c
char* mc_get_device_id();
```

**返回值:**
- 设备 ID 字符串指针 (需通过 `mc_sdk_free_string` 释放)
- 失败返回 `NULL`

**生成方式:**
基于 7 项稳定硬件信息生成，格式为 `xxxx-xxxx-xxxx-xxxx`：

| # | 数据项 | 说明 |
|---|--------|------|
| 1 | CPU 品牌 | 出厂固定 |
| 2 | CPU 厂商 ID | 出厂固定 |
| 3 | CPU 物理核心数 | 硬件规格 |
| 4 | CPU 最大频率 | 硬件规格 |
| 5 | 网卡 MAC 地址 | 主板集成 |
| 6 | 网卡名称 | 主板集成 |
| 7 | 操作系统名称 | 系统标识 |

**设计原则:**
- 只采集用户无法轻易更换的硬件信息
- 排除用户可升级的硬件（硬盘、内存等）
- 保证每次运行计算的 ID 一致

**特点:**
- 稳定不变：每次运行结果相同
- 设备唯一：不同设备生成不同 ID
- 不可篡改：仅使用硬件信息
- CI 友好：自动过滤虚拟/临时组件

**平台支持:**

| 平台 | 实现方式 |
|------|----------|
| Windows | sysinfo 采集 CPU/网卡/系统信息 |
| macOS | sysinfo 采集 CPU/网卡/系统信息 |
| Linux | sysinfo 采集 CPU/网卡/系统信息 |

**调用示例:**

```c
char* device_id = mc_get_device_id();
if (device_id != NULL) {
    printf("Device ID: %s\n", device_id);  // 输出如: a3f9-c7e2-8b1d-4f6a
    mc_sdk_free_string(device_id);
}
```

---

## Token 加密/解密

### mc_encrypt_token

使用 DES 加密 Token 数据。

```c
char* mc_encrypt_token(const char* data);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `data` | `const char*` | 明文数据 (JSON 字符串) |

**返回值:**
- 加密后的数据 (Base64 编码，需通过 `mc_sdk_free_string` 释放)
- 失败返回 `NULL`

**加密规格:**
- 算法: DES
- 密钥: `mcsdk-{设备码}` (8 字节)
- 填充: PKCS7
- 输出: Base64 编码

**调用示例:**

```c
const char* token_json = "{\"access_token\":\"xxx\",\"uuid\":\"yyy\",\"username\":\"TestPlayer\"}";
char* encrypted = mc_encrypt_token(token_json);
if (encrypted != NULL) {
    // 存储 encrypted 到文件或注册表
    printf("Encrypted: %s\n", encrypted);
    mc_sdk_free_string(encrypted);
}
```

---

### mc_decrypt_token

解密 Token 数据。

```c
char* mc_decrypt_token(const char* encrypted);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `encrypted` | `const char*` | 加密后的数据 (Base64 编码) |

**返回值:**
- 明文数据 (JSON 字符串，需通过 `mc_sdk_free_string` 释放)
- 失败返回 `NULL`

**调用示例:**

```c
char* decrypted = mc_decrypt_token(encrypted_data);
if (decrypted != NULL) {
    // 解析 JSON 获取 token 信息
    printf("Token: %s\n", decrypted);
    mc_sdk_free_string(decrypted);
}
```

---

## 认证模块

### mc_auth_offline

离线模式登录。

```c
int32_t mc_auth_offline(const char* username, FFIAuthResult* result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `username` | `const char*` | 玩家用户名 (最大 16 字符，仅字母数字下划线) |
| `result_out` | `FFIAuthResult*` | 输出参数，接收认证结果 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**调用示例:**

```c
FFIAuthResult result;
int32_t code = mc_auth_offline("TestPlayer", &result);
if (code == 0) {
    printf("UUID: %s\n", result.uuid);
    printf("Username: %s\n", result.username);
    mc_auth_free_result(&result);
}
```

---

### mc_auth_microsoft_start

发起微软 OAuth 2.0 设备码登录。

```c
int32_t mc_auth_microsoft_start(FFIDeviceCode* device_code_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `device_code_out` | `FFIDeviceCode*` | 输出参数，接收设备码信息 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**FFIDeviceCode 结构体:**

```c
typedef struct {
    char* device_code;              // 设备码 (用于轮询)
    char* user_code;                // 用户码 (显示给用户)
    char* verification_uri;         // 验证 URI
    char* verification_uri_complete; // 完整验证 URI (可选)
    uint32_t expires_in;            // 过期时间 (秒)
    uint32_t interval;              // 轮询间隔 (秒)
    int32_t error_code;             // 错误码 (0 = 成功)
    char* error_message;            // 错误消息
} FFIDeviceCode;
```

**调用示例:**

```c
FFIDeviceCode device_code;
int32_t code = mc_auth_microsoft_start(&device_code);
if (code == 0) {
    printf("请在浏览器中访问: %s\n", device_code.verification_uri_complete);
    printf("用户码: %s\n", device_code.user_code);
    
    // 轮询登录状态
    FFIAuthResult result;
    int32_t poll_code = mc_auth_microsoft_poll(device_code.device_code, device_code.interval, &result);
    if (poll_code == 0) {
        printf("登录成功! 用户名: %s\n", result.username);
        mc_auth_free_result(&result);
    }
    
    mc_auth_free_device_code(&device_code);
}
```

---

### mc_auth_microsoft_poll

轮询微软登录状态。

```c
int32_t mc_auth_microsoft_poll(const char* device_code, uint32_t interval, FFIAuthResult* result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `device_code` | `const char*` | 设备码 (从 mc_auth_microsoft_start 获取) |
| `interval` | `uint32_t` | 轮询间隔 (秒) |
| `result_out` | `FFIAuthResult*` | 输出参数，接收认证结果 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**注意:** 此函数会阻塞直到用户完成授权或超时 (15 分钟)。

---

### mc_auth_microsoft_refresh

刷新微软令牌。

```c
int32_t mc_auth_microsoft_refresh(const char* refresh_token, FFIAuthResult* result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `refresh_token` | `const char*` | 刷新令牌 |
| `result_out` | `FFIAuthResult*` | 输出参数，接收认证结果 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_auth_mojang_login

Mojang 认证登录。

```c
int32_t mc_auth_mojang_login(const char* username, const char* password, FFIAuthResult* result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `username` | `const char*` | 用户名 (邮箱) |
| `password` | `const char*` | 密码 |
| `result_out` | `FFIAuthResult*` | 输出参数，接收认证结果 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_auth_nide_login

统一通行证登录。

```c
int32_t mc_auth_nide_login(const char* server_id, const char* username, const char* password, FFIAuthResult* result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `server_id` | `const char*` | 服务器 ID |
| `username` | `const char*` | 用户名 |
| `password` | `const char*` | 密码 |
| `result_out` | `FFIAuthResult*` | 输出参数，接收认证结果 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_auth_authlib_login

Authlib-Injector 登录。

```c
int32_t mc_auth_authlib_login(const char* base_url, const char* username, const char* password, FFIAuthResult* result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `base_url` | `const char*` | 认证服务器基础 URL (如 `https://littleskin.cn/api/yggdrasil`) |
| `username` | `const char*` | 用户名 |
| `password` | `const char*` | 密码 |
| `result_out` | `FFIAuthResult*` | 输出参数，接收认证结果 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_auth_free_result

释放认证结果内存。

```c
void mc_auth_free_result(FFIAuthResult* result);
```

---

### mc_auth_free_device_code

释放设备码信息内存。

```c
void mc_auth_free_device_code(FFIDeviceCode* device_code);
```

---

## 皮肤管理

### mc_skin_get

获取玩家皮肤信息。

```c
int32_t mc_skin_get(const char* uuid, char** skin_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `uuid` | `const char*` | 玩家 UUID |
| `skin_out` | `char**` | 输出参数，接收皮肤信息 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**响应数据格式 (JSON):**

```json
{
  "url": "http://textures.minecraft.net/texture/...",
  "state": "ACTIVE",
  "alias": "slim"
}
```

**调用示例:**

```c
char* skin_json;
int32_t code = mc_skin_get("player-uuid", &skin_json);
if (code == 0) {
    printf("Skin: %s\n", skin_json);
    mc_sdk_free_string(skin_json);
}
```

---

### mc_skin_upload

上传皮肤。

```c
int32_t mc_skin_upload(const char* access_token, const char* skin_path, const char* variant, char** profile_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `access_token` | `const char*` | 访问令牌 |
| `skin_path` | `const char*` | 皮肤文件路径 |
| `variant` | `const char*` | 皮肤类型 (`classic` 或 `slim`) |
| `profile_out` | `char**` | 输出参数，接收更新后的玩家档案 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_skin_set_cape

设置当前披风。

```c
int32_t mc_skin_set_cape(const char* access_token, const char* cape_id, char** profile_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `access_token` | `const char*` | 访问令牌 |
| `cape_id` | `const char*` | 披风 ID |
| `profile_out` | `char**` | 输出参数，接收更新后的玩家档案 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_skin_clear_cape

取消当前披风。

```c
int32_t mc_skin_clear_cape(const char* access_token, char** profile_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `access_token` | `const char*` | 访问令牌 |
| `profile_out` | `char**` | 输出参数，接收更新后的玩家档案 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

## 模组安装

### mc_install_forge

安装 Forge。

```c
int32_t mc_install_forge(const SDKHandle* handle, const char* mc_version, const char* forge_version);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `mc_version` | `const char*` | Minecraft 版本号 (如 `1.20.1`) |
| `forge_version` | `const char*` | Forge 版本号 (如 `47.2.0`) |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**调用示例:**

```c
int32_t code = mc_install_forge(handle, "1.20.1", "47.2.0");
if (code == 0) {
    printf("Forge 安装成功!\n");
}
```

---

### mc_install_fabric

安装 Fabric。

```c
int32_t mc_install_fabric(const SDKHandle* handle, const char* mc_version, const char* loader_version);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `mc_version` | `const char*` | Minecraft 版本号 |
| `loader_version` | `const char*` | Fabric Loader 版本号 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_install_neoforge

安装 NeoForge。

```c
int32_t mc_install_neoforge(const SDKHandle* handle, const char* mc_version, const char* neoforge_version);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `mc_version` | `const char*` | Minecraft 版本号 |
| `neoforge_version` | `const char*` | NeoForge 版本号 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_install_optifine

安装 OptiFine。

```c
int32_t mc_install_optifine(const SDKHandle* handle, const char* mc_version, const char* optifine_version);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `mc_version` | `const char*` | Minecraft 版本号 |
| `optifine_version` | `const char*` | OptiFine 版本号 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_install_liteloader

安装 LiteLoader。

```c
int32_t mc_install_liteloader(const SDKHandle* handle, const char* mc_version, const char* liteloader_version);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `mc_version` | `const char*` | Minecraft 版本号 |
| `liteloader_version` | `const char*` | LiteLoader 版本号 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

## Mod 管理

### mc_mod_list

获取 Mod 列表。

```c
int32_t mc_mod_list(const SDKHandle* handle, char** result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `result_out` | `char**` | 输出参数，接收 Mod 列表 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**响应数据格式 (JSON):**

```json
[
  {
    "id": "jei",
    "name": "jei-1.20.1-15.2.0.27.jar",
    "version": "1.20.1-15.2.0.27",
    "description": null,
    "authors": null,
    "file_path": "C:/Users/Test/.minecraft/mods/jei-1.20.1-15.2.0.27.jar",
    "enabled": true
  },
  {
    "id": "optifine",
    "name": "optifine.jar.disabled",
    "version": "unknown",
    "description": null,
    "authors": null,
    "file_path": "C:/Users/Test/.minecraft/mods/optifine.jar.disabled",
    "enabled": false
  }
]
```

**调用示例:**

```c
char* mods_json;
int32_t code = mc_mod_list(handle, &mods_json);
if (code == 0) {
    printf("Mods: %s\n", mods_json);
    mc_sdk_free_string(mods_json);
}
```

---

### mc_mod_enable

启用 Mod (将 `.disabled` 重命名为 `.jar`)。

```c
int32_t mc_mod_enable(const SDKHandle* handle, const char* mod_path);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `mod_path` | `const char*` | Mod 文件完整路径 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_mod_disable

禁用 Mod (将 `.jar` 重命名为 `.disabled`)。

```c
int32_t mc_mod_disable(const SDKHandle* handle, const char* mod_path);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `mod_path` | `const char*` | Mod 文件完整路径 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

## 版本管理

### mc_list_versions

获取 Minecraft 版本列表。

```c
int32_t mc_list_versions(FFIVersionList* version_list_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `version_list_out` | `FFIVersionList*` | 输出参数，接收版本列表 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**FFIVersionList 结构体:**

```c
typedef struct {
    FFIVersionEntry* versions;  // 版本数组指针
    uint32_t count;             // 版本数量
    char* latest_release;       // 最新正式版
    char* latest_snapshot;      // 最新快照版
    int32_t error_code;         // 错误码 (0 = 成功)
    char* error_message;        // 错误消息
} FFIVersionList;

typedef struct {
    char* id;               // 版本 ID (如 "1.20.1")
    char* version_type;     // 版本类型 (release/snapshot/old_beta/old_alpha)
    int64_t release_time;   // 发布时间 (Unix 时间戳)
} FFIVersionEntry;
```

**调用示例:**

```c
FFIVersionList version_list;
int32_t code = mc_list_versions(&version_list);
if (code == 0) {
    printf("最新正式版: %s\n", version_list.latest_release);
    printf("最新快照版: %s\n", version_list.latest_snapshot);
    printf("共 %d 个版本\n", version_list.count);
    
    for (uint32_t i = 0; i < version_list.count && i < 10; i++) {
        printf("  %s (%s)\n", version_list.versions[i].id, version_list.versions[i].version_type);
    }
    
    mc_free_version_list(&version_list);
}
```

---

### mc_list_installed_versions

获取已安装版本列表。

```c
int32_t mc_list_installed_versions(const SDKHandle* handle, char*** versions_out, uint32_t* count_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `versions_out` | `char***` | 输出参数，接收版本 ID 数组 |
| `count_out` | `uint32_t*` | 输出参数，接收版本数量 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**调用示例:**

```c
char** versions;
uint32_t count;
int32_t code = mc_list_installed_versions(handle, &versions, &count);
if (code == 0) {
    printf("已安装 %d 个版本:\n", count);
    for (uint32_t i = 0; i < count; i++) {
        printf("  %s\n", versions[i]);
    }
    mc_free_string_array(versions, count);
}
```

---

### mc_free_version_list

释放版本列表内存。

```c
void mc_free_version_list(FFIVersionList* version_list);
```

---

### mc_free_string_array

释放字符串数组内存。

```c
void mc_free_string_array(char** array, uint32_t count);
```

---

## Java 检测

### mc_detect_java

检测最佳 Java 运行时。

```c
int32_t mc_detect_java(FFIJavaRuntime* java_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `java_out` | `FFIJavaRuntime*` | 输出参数，接收 Java 运行时信息 |

**返回值:**
- `0`: 成功
- 非 0: 错误码 (401 = Java 未找到)

**FFIJavaRuntime 结构体:**

```c
typedef struct {
    char* executable;       // Java 可执行文件路径
    char* version;          // Java 版本号
    uint32_t major_version; // Java 主版本号 (8, 11, 17, 21 等)
    char* arch;             // Java 架构 (x86_64, aarch64 等)
    char* home;             // Java 主页目录
} FFIJavaRuntime;
```

**调用示例:**

```c
FFIJavaRuntime java;
int32_t code = mc_detect_java(&java);
if (code == 0) {
    printf("Java 路径: %s\n", java.executable);
    printf("Java 版本: %s\n", java.version);
    printf("主版本号: %u\n", java.major_version);
    mc_free_java_runtime(&java);
} else {
    printf("未找到 Java 8 或更高版本\n");
}
```

---

### mc_list_java

列出所有已安装的 Java。

```c
int32_t mc_list_java(FFIJavaList* java_list_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `java_list_out` | `FFIJavaList*` | 输出参数，接收 Java 列表 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**FFIJavaList 结构体:**

```c
typedef struct {
    FFIJavaRuntime* runtimes;  // Java 数组指针
    uint32_t count;            // Java 数量
    int32_t error_code;        // 错误码 (0 = 成功)
    char* error_message;       // 错误消息
} FFIJavaList;
```

**调用示例:**

```c
FFIJavaList java_list;
int32_t code = mc_list_java(&java_list);
if (code == 0) {
    printf("找到 %d 个 Java:\n", java_list.count);
    for (uint32_t i = 0; i < java_list.count; i++) {
        printf("  %s (版本 %s)\n", java_list.runtimes[i].executable, java_list.runtimes[i].version);
    }
    mc_free_java_list(&java_list);
}
```

---

### mc_free_java_list

释放 Java 列表内存。

```c
void mc_free_java_list(FFIJavaList* java_list);
```

---

### mc_free_java_runtime

释放 Java 运行时信息内存。

```c
void mc_free_java_runtime(FFIJavaRuntime* java_runtime);
```

---

## 下载与启动

### mc_download_version

下载游戏版本文件。

```c
int32_t mc_download_version(const SDKHandle* handle, const char* version_id, FFICallback callback, void* user_data);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `version_id` | `const char*` | 版本 ID (如 `1.20.1`) |
| `callback` | `FFICallback` | 进度回调函数 |
| `user_data` | `void*` | 用户数据指针 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**FFICallback 类型:**

```c
typedef void (*FFICallback)(uint32_t stage, uint32_t current, uint32_t total, void* user_data);
```

**下载阶段 (stage):**

| 阶段 | 说明 |
|------|------|
| 0 | 版本清单 |
| 1 | 版本 JSON |
| 2 | 客户端 JAR |
| 3 | 库文件 |
| 4 | 资源文件 |
| 5 | 解压 Natives |

**调用示例:**

```c
void progress_callback(uint32_t stage, uint32_t current, uint32_t total, void* user_data) {
    printf("Stage %u: %u/%u\n", stage, current, total);
}

int32_t code = mc_download_version(handle, "1.20.1", progress_callback, NULL);
if (code == 0) {
    printf("下载完成!\n");
}
```

---

### mc_launch_game

启动游戏。

```c
int32_t mc_launch_game(const SDKHandle* handle, const char* username, const char* uuid, const char* access_token, const char* version_id, uint32_t max_memory);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `username` | `const char*` | 玩家用户名 |
| `uuid` | `const char*` | 玩家 UUID |
| `access_token` | `const char*` | 访问令牌 |
| `version_id` | `const char*` | 版本 ID |
| `max_memory` | `uint32_t` | 最大内存 (MB，默认 1024) |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**调用示例:**

```c
int32_t code = mc_launch_game(
    handle,
    "TestPlayer",
    "player-uuid",
    "access-token",
    "1.20.1",
    2048
);
if (code == 0) {
    printf("游戏启动成功!\n");
}
```

---

### mc_launch_get_status

获取游戏进程状态。

```c
int32_t mc_launch_get_status();
```

**返回值:**

| 值 | 说明 |
|----|------|
| 0 | 未启动 |
| 1 | 运行中 |
| 2 | 已退出（成功） |
| 3 | 已退出（失败） |
| 4 | 已终止 |

---

### mc_launch_get_exit_code

获取游戏进程退出码。

```c
int32_t mc_launch_get_exit_code();
```

**返回值:**
- 退出码 (0 = 成功)
- `-1`: 进程还在运行或未启动

---

## 资源平台 API

### mc_curseforge_search

CurseForge 资源搜索。

```c
int32_t mc_curseforge_search(const char* api_key, const char* query, const char* game_version, uint32_t page, uint32_t page_size, char** result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `api_key` | `const char*` | CurseForge API Key |
| `query` | `const char*` | 搜索关键词 |
| `game_version` | `const char*` | 游戏版本 (可选，NULL 表示全部) |
| `page` | `uint32_t` | 页码 (从 0 开始) |
| `page_size` | `uint32_t` | 每页大小 (最大 50) |
| `result_out` | `char**` | 输出参数，接收搜索结果 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**响应数据格式 (JSON):**

```json
{
  "data": [
    {
      "id": 306612,
      "name": "Just Enough Items (JEI)",
      "summary": "JEI is an item and recipe viewing mod...",
      "download_count": 100000000,
      "categories": [...],
      "authors": [...],
      "logo": {...}
    }
  ],
  "pagination": {
    "index": 0,
    "page_size": 20,
    "total_count": 100
  }
}
```

**调用示例:**

```c
char* result;
int32_t code = mc_curseforge_search(api_key, "JEI", "1.20.1", 0, 20, &result);
if (code == 0) {
    printf("搜索结果: %s\n", result);
    mc_sdk_free_string(result);
}
```

---

### mc_curseforge_get_project

CurseForge 获取项目详情。

```c
int32_t mc_curseforge_get_project(const char* api_key, uint32_t project_id, char** result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `api_key` | `const char*` | CurseForge API Key |
| `project_id` | `uint32_t` | 项目 ID |
| `result_out` | `char**` | 输出参数，接收项目详情 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_curseforge_get_files

CurseForge 获取项目文件列表。

```c
int32_t mc_curseforge_get_files(const char* api_key, uint32_t project_id, const char* game_version, uint32_t page, uint32_t page_size, char** result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `api_key` | `const char*` | CurseForge API Key |
| `project_id` | `uint32_t` | 项目 ID |
| `game_version` | `const char*` | 游戏版本 (可选) |
| `page` | `uint32_t` | 页码 |
| `page_size` | `uint32_t` | 每页大小 |
| `result_out` | `char**` | 输出参数，接收文件列表 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_modrinth_search

Modrinth 资源搜索。

```c
int32_t mc_modrinth_search(const char* query, const char* game_version, uint32_t limit, uint32_t offset, char** result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `query` | `const char*` | 搜索关键词 |
| `game_version` | `const char*` | 游戏版本 (可选) |
| `limit` | `uint32_t` | 返回数量 (最大 100) |
| `offset` | `uint32_t` | 偏移量 |
| `result_out` | `char**` | 输出参数，接收搜索结果 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**响应数据格式 (JSON):**

```json
{
  "hits": [
    {
      "project_id": "u6dRK93c",
      "project_type": "mod",
      "title": "Just Enough Items (JEI)",
      "description": "JEI is an item and recipe viewing mod...",
      "downloads": 50000000,
      "icon_url": "...",
      "author": "mezz",
      "categories": ["technology", "utility"],
      "versions": ["1.20.1", "1.20"],
      "date_modified": "2023-06-07T12:00:00Z"
    }
  ],
  "total_hits": 100,
  "limit": 20,
  "offset": 0
}
```

---

### mc_modrinth_get_project

Modrinth 获取项目详情。

```c
int32_t mc_modrinth_get_project(const char* project_id, char** result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `project_id` | `const char*` | 项目 ID 或 slug |
| `result_out` | `char**` | 输出参数，接收项目详情 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_modrinth_get_versions

Modrinth 获取项目版本列表。

```c
int32_t mc_modrinth_get_versions(const char* project_id, const char* game_version, const char* loader, char** result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `project_id` | `const char*` | 项目 ID 或 slug |
| `game_version` | `const char*` | 游戏版本 (可选) |
| `loader` | `const char*` | 加载器类型 (可选，如 `fabric`, `forge`) |
| `result_out` | `char**` | 输出参数，接收版本列表 JSON |

**返回值:**
- `0`: 成功
- 非 0: 错误码

---

### mc_install_modpack

安装整合包。

```c
int32_t mc_install_modpack(const SDKHandle* handle, const char* modpack_path, char** result_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `modpack_path` | `const char*` | 整合包文件路径 |
| `result_out` | `char**` | 输出参数，接收安装结果消息 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**支持格式:**
- CurseForge (.zip)
- Modrinth (.mrpack)

---

## 更新系统

### mc_update_check

检查 SDK 是否有新版本可用。

```c
int32_t mc_update_check(const SDKHandle* handle, FFIUpdateInfo* info_out);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `info_out` | `FFIUpdateInfo*` | 输出参数，接收更新信息 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**FFIUpdateInfo 结构体:**

```c
typedef struct {
    char* current_version;    // 当前版本
    char* latest_version;     // 最新版本
    int32_t update_available; // 是否需要更新 (1=需要, 0=不需要)
    char* download_url;       // 下载 URL
    char* sha256;             // SHA-256 哈希
    uint64_t size;            // 文件大小 (字节)
    char* changelog;          // 更新日志
} FFIUpdateInfo;
```

**调用示例:**

```c
FFIUpdateInfo info;
int32_t code = mc_update_check(handle, &info);
if (code == 0) {
    if (info.update_available) {
        printf("New version available: %s\n", info.latest_version);
        printf("Download URL: %s\n", info.download_url);
    } else {
        printf("SDK is up to date: %s\n", info.current_version);
    }
    mc_update_free_info(&info);
}
```

---

### mc_update_download

下载 SDK 更新包。

```c
int32_t mc_update_download(const SDKHandle* handle, const char* download_url, const char* sha256, const char* output_path);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `download_url` | `const char*` | 下载 URL |
| `sha256` | `const char*` | SHA-256 哈希 (可选，为 NULL 跳过验证) |
| `output_path` | `const char*` | 输出文件路径 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**调用示例:**

```c
int32_t code = mc_update_download(
    handle,
    "https://example.com/releases/mc_sdk-0.2.0.dll",
    "abc123...",
    "C:/temp/mc_sdk_update.dll"
);
if (code == 0) {
    printf("Update downloaded successfully\n");
}
```

---

### mc_update_install

安装 SDK 更新。

```c
int32_t mc_update_install(const SDKHandle* handle, const char* update_path, const char* latest_version);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |
| `update_path` | `const char*` | 更新包文件路径 |
| `latest_version` | `const char*` | 最新版本号 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**注意:**
- Windows 上如果 DLL 正在使用，会使用重命名替换策略
- 安装前会自动备份当前版本
- 更新后需要重启应用才能生效

---

### mc_update_free_info

释放更新信息内存。

```c
void mc_update_free_info(FFIUpdateInfo* info);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `info` | `FFIUpdateInfo*` | 更新信息指针 |

---

## 配置管理

### mc_config_get_game_dir

获取游戏目录路径。

```c
char* mc_config_get_game_dir(const SDKHandle* handle);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |

**返回值:**
- 游戏目录路径字符串 (需通过 `mc_sdk_free_string` 释放)
- 失败返回 `NULL`

**调用示例:**

```c
char* game_dir = mc_config_get_game_dir(handle);
if (game_dir != NULL) {
    printf("Game directory: %s\n", game_dir);
    mc_sdk_free_string(game_dir);
}
```

---

### mc_config_get_mirror

获取镜像源 URL。

```c
char* mc_config_get_mirror(const SDKHandle* handle);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |

**返回值:**
- 镜像源 URL 字符串 (需通过 `mc_sdk_free_string` 释放)
- 如果未设置镜像源，返回空字符串
- 失败返回 `NULL`

**调用示例:**

```c
char* mirror = mc_config_get_mirror(handle);
if (mirror != NULL) {
    if (strlen(mirror) > 0) {
        printf("Mirror URL: %s\n", mirror);
    } else {
        printf("Using official source\n");
    }
    mc_sdk_free_string(mirror);
}
```

---

## 网络管理

### mc_network_clear_cache

清除 HTTP 缓存。

```c
int32_t mc_network_clear_cache(const SDKHandle* handle);
```

**参数:**

| 参数 | 类型 | 说明 |
|------|------|------|
| `handle` | `const SDKHandle*` | SDK 句柄 |

**返回值:**
- `0`: 成功
- 非 0: 错误码

**调用示例:**

```c
int32_t code = mc_network_clear_cache(handle);
if (code == 0) {
    printf("Cache cleared successfully\n");
}
```

---

## 数据结构

### FFIAuthResult

认证结果结构体。

```c
typedef struct {
    AuthType auth_type;      // 认证类型
    char* access_token;      // 访问令牌
    char* refresh_token;     // 刷新令牌 (可选)
    char* uuid;              // 玩家 UUID
    char* username;          // 玩家用户名
    int64_t expires_at;      // 令牌过期时间 (Unix 时间戳)
    int32_t error_code;      // 错误码 (0 = 成功)
    char* error_message;     // 错误消息
} FFIAuthResult;
```

### AuthType

认证类型枚举。

```c
typedef enum {
    AuthType_Microsoft = 0,  // 微软账号
    AuthType_Offline = 1,    // 离线模式
    AuthType_External = 2    // 外置认证 (Authlib-Injector)
} AuthType;
```

### FFIUpdateInfo

更新信息结构体。

```c
typedef struct {
    char* current_version;    // 当前版本
    char* latest_version;     // 最新版本
    int32_t update_available; // 是否需要更新 (1=需要, 0=不需要)
    char* download_url;       // 下载 URL
    char* sha256;             // SHA-256 哈希
    uint64_t size;            // 文件大小 (字节)
    char* changelog;          // 更新日志
} FFIUpdateInfo;
```

**内存管理:** 使用 `mc_update_free_info` 释放

---

## 完整调用流程示例

```c
#include "mc_sdk.h"
#include <stdio.h>

int main() {
    // 1. 初始化 SDK
    MCConfig config = {
        .game_dir = "C:/Users/Test/.minecraft",
        .max_download_threads = 8,
        .mirror_url = NULL,
        .log_level = 3,
        .curseforge_api_key = "your-api-key"
    };
    
    SDKHandle* handle = mc_sdk_init(&config);
    if (handle == NULL) {
        ErrorInfo* error = mc_sdk_last_error();
        printf("Init failed: %s\n", error->message);
        mc_sdk_free_error(error);
        return 1;
    }
    
    // 2. 检测 Java
    FFIJavaRuntime java;
    if (mc_detect_java(&java) != 0) {
        printf("Java not found!\n");
        mc_sdk_free(handle);
        return 1;
    }
    printf("Using Java: %s\n", java.version);
    mc_free_java_runtime(&java);
    
    // 3. 离线登录
    FFIAuthResult auth_result;
    if (mc_auth_offline("TestPlayer", &auth_result) != 0) {
        printf("Auth failed!\n");
        mc_sdk_free(handle);
        return 1;
    }
    printf("Logged in as: %s\n", auth_result.username);
    
    // 4. 下载版本
    void progress_callback(uint32_t stage, uint32_t current, uint32_t total, void* user_data) {
        printf("Downloading stage %u: %u/%u\n", stage, current, total);
    }
    
    if (mc_download_version(handle, "1.20.1", progress_callback, NULL) != 0) {
        printf("Download failed!\n");
        mc_auth_free_result(&auth_result);
        mc_sdk_free(handle);
        return 1;
    }
    
    // 5. 启动游戏
    if (mc_launch_game(handle, auth_result.username, auth_result.uuid, 
                       auth_result.access_token, "1.20.1", 2048) != 0) {
        printf("Launch failed!\n");
        mc_auth_free_result(&auth_result);
        mc_sdk_free(handle);
        return 1;
    }
    
    printf("Game launched!\n");
    
    // 6. 清理
    mc_auth_free_result(&auth_result);
    mc_sdk_free(handle);
    return 0;
}
```

---

## 错误处理最佳实践

```c
int32_t code = mc_some_function(...);
if (code != 0) {
    ErrorInfo* error = mc_sdk_last_error();
    printf("Error %d: %s\n", error->code, error->message);
    mc_sdk_free_error(error);
    // 处理错误
}
```

---

## 内存管理规则

1. **需要释放的接口**: 所有返回 `char*` 的接口（除 `mc_sdk_version`）都需要调用 `mc_sdk_free_string` 释放
2. **不需要释放的接口**: `mc_sdk_version` 返回静态内存
3. **结构体释放**: 使用对应的 `mc_free_*` 函数释放结构体内部内存
4. **错误信息**: 使用 `mc_sdk_free_error` 释放

---

*本文档最后更新于 2026-06-25*
