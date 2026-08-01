# FRP 厂商开发指南

本指南面向想要为 MoLaunch 开发自定义 FRP 穿透厂商的开发者。通过编写 `manifest.json` 清单文件，你可以定义厂商信息、frpc 二进制分发方式、认证流程和网络/进程安全权限。

## 目录

- [快速开始](#快速开始)
- [manifest.json 字段说明](#manifestjson-字段说明)
- [frpc 二进制分发（binary）](#frpc-二进制分发binary)
- [认证配置（auth）](#认证配置auth)
  - [none：无需认证](#none无需认证)
  - [oauth2：OAuth2 授权码流程](#oauth2oauth2-授权码流程)
  - [device_code：设备码流程](#device_code设备码流程)
  - [api_key：API Key 手动填入](#api_keyapi-key-手动填入)
- [网络权限（networkPermissions）](#网络权限networkpermissions)
- [进程权限（processPermissions）](#进程权限processpermissions)
- [完整示例](#完整示例)
- [安装方式](#安装方式)

## 快速开始

1. 创建一个文件夹，命名为你的厂商 ID（如 `my-frp-provider`）
2. 在文件夹根目录创建 `manifest.json`
3. 根据分发方式放置 frpc 二进制文件或配置下载地址
4. 在 MoLaunch 的「Frp 管理 → 厂商列表」中点击「从文件夹安装」或「从 ZIP 安装」

## manifest.json 字段说明

```json
{
  "id": "my-provider",
  "name": "我的穿透厂商",
  "description": "一个示例 FRP 厂商",
  "version": "1.0.0",
  "author": "作者名称",
  "homepage": "https://example.com",
  "icon": "icon.png",
  "binary": { ... },
  "auth": { ... },
  "networkPermissions": { ... },
  "processPermissions": { ... }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | 是 | 厂商唯一标识（小写字母+连字符，如 `my-provider`） |
| `name` | string | 是 | 显示名称 |
| `description` | string | 是 | 简短描述 |
| `version` | string | 是 | 厂商版本号（语义化版本） |
| `author` | string | 是 | 作者名称 |
| `homepage` | string | 否 | 厂商主页 URL |
| `icon` | string | 否 | 图标文件相对路径（建议 128x128 PNG） |
| `binary` | object | 是 | frpc 二进制分发配置 |
| `auth` | object | 否 | 认证配置（默认 `none`） |
| `networkPermissions` | object | 否 | 网络权限白名单 |
| `processPermissions` | object | 否 | 进程权限白名单 |

## frpc 二进制分发（binary）

### bundled：随包打包

将 frpc 二进制文件放入厂商文件夹，通过 `path` 指定相对路径。

```json
{
  "binary": {
    "distribution": "bundled",
    "path": "bin/frpc.exe"
  }
}
```

### url：按需下载

用户首次使用时自动下载 frpc，支持 ZIP 压缩包自动解压。

```json
{
  "binary": {
    "distribution": "url",
    "download": {
      "url": "https://example.com/frpc-windows.zip",
      "sha256": "可选校验值",
      "allowedDomains": ["example.com"],
      "targetPath": "frpc.exe",
      "archive": true
    }
  }
}
```

| 字段 | 说明 |
|------|------|
| `url` | 下载地址 |
| `sha256` | 可选，文件 SHA256 校验值 |
| `allowedDomains` | 允许下载的域名白名单 |
| `targetPath` | 下载后存放的相对路径（archive=true 时为解压后的目标文件） |
| `archive` | 是否为 ZIP 压缩包（true 时自动解压） |

## 认证配置（auth）

### none：无需认证

最简单的认证方式，用户无需任何认证即可使用穿透服务。

```json
{
  "auth": {
    "type": "none"
  }
}
```

### oauth2：OAuth2 授权码流程

用户在浏览器中完成授权，本地启动 HTTP 服务接收回调。适用于有 Web 管理后台的厂商。

```json
{
  "auth": {
    "type": "oauth2",
    "oauth2": {
      "authorizeUrl": "https://example.com/oauth/authorize",
      "tokenUrl": "https://example.com/oauth/token",
      "clientId": "your-client-id",
      "scopes": ["tunnel:manage"],
      "redirectPort": 18365
    }
  }
}
```

| 字段 | 说明 |
|------|------|
| `authorizeUrl` | 授权页 URL（浏览器打开让用户登录授权） |
| `tokenUrl` | Token 交换 URL（后端用授权码换取 access_token） |
| `clientId` | OAuth2 客户端 ID |
| `scopes` | 权限范围列表 |
| `redirectPort` | 本地回调端口（本地启动 HTTP 服务接收回调） |

### device_code：设备码流程

用户在另一设备上输入设备码完成认证。适用于无浏览器的设备或 TV 端。

```json
{
  "auth": {
    "type": "device_code",
    "deviceCode": {
      "deviceCodeUrl": "https://example.com/oauth/device/code",
      "tokenUrl": "https://example.com/oauth/token",
      "clientId": "your-client-id",
      "scopes": ["tunnel:manage"],
      "pollInterval": 5
    }
  }
}
```

| 字段 | 说明 |
|------|------|
| `deviceCodeUrl` | 设备码请求 URL（获取用户码和验证链接） |
| `tokenUrl` | Token 轮询 URL（按间隔轮询直到用户完成认证） |
| `clientId` | 客户端 ID |
| `scopes` | 权限范围列表 |
| `pollInterval` | 轮询间隔（秒），默认 5 |

### api_key：API Key 手动填入

用户手动获取 API Key 填入，存储到系统密钥存储中。适用于简单的 API 认证场景。

```json
{
  "auth": {
    "type": "api_key",
    "apiKey": {
      "obtainUrl": "https://example.com/dashboard/api-keys",
      "headerName": "X-API-Key"
    }
  }
}
```

| 字段 | 说明 |
|------|------|
| `obtainUrl` | 获取 API Key 的页面 URL（前端提供跳转入口） |
| `headerName` | API Key 在请求头中的字段名 |

## 网络权限（networkPermissions）

限制 frpc 可连接的服务器地址，防止厂商恶意连接未知服务器。

```json
{
  "networkPermissions": {
    "allowedServers": ["frps.example.com:7000", "1.2.3.4:7000"],
    "allowCustomServer": false
  }
}
```

| 字段 | 说明 |
|------|------|
| `allowedServers` | 允许的 frps 服务器地址白名单（域名或 IP[:端口]） |
| `allowCustomServer` | 是否允许用户自定义服务器（false=仅白名单内的服务器） |

> 系统默认厂商始终允许自定义服务器，不受此限制。

## 进程权限（processPermissions）

限制厂商认证适配器脚本的执行权限。仅当厂商提供自定义认证脚本时启用。

```json
{
  "processPermissions": {
    "allowedCommands": ["node", "python"],
    "timeoutMs": 30000
  }
}
```

| 字段 | 说明 |
|------|------|
| `allowedCommands` | 允许执行的命令白名单（如 `["node", "python"]`） |
| `timeoutMs` | 超时毫秒，默认 30000（30 秒），最大 300000（5 分钟） |

安全约束：
- 命令必须通过 `which_canonical` 解析后与白名单匹配
- 非 shell 执行，防止命令注入
- stdout/stderr 各截断到 1MB
- 工作目录限制在厂商目录内

## 完整示例

以下是一个完整的厂商清单示例，使用 URL 下载 frpc + OAuth2 认证 + 网络白名单：

```json
{
  "id": "acme-tunnel",
  "name": "ACME 穿透",
  "description": "ACME 官方 FRP 穿透服务",
  "version": "1.2.0",
  "author": "ACME Inc.",
  "homepage": "https://acme.example.com",
  "icon": "logo.png",
  "binary": {
    "distribution": "url",
    "download": {
      "url": "https://cdn.acme.example.com/frpc-0.54.0-windows.zip",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "allowedDomains": ["cdn.acme.example.com"],
      "targetPath": "frpc.exe",
      "archive": true
    }
  },
  "auth": {
    "type": "oauth2",
    "oauth2": {
      "authorizeUrl": "https://acme.example.com/oauth/authorize",
      "tokenUrl": "https://acme.example.com/oauth/token",
      "clientId": "molaunch-client",
      "scopes": ["tunnel:manage", "user:read"],
      "redirectPort": 18365
    }
  },
  "networkPermissions": {
    "allowedServers": ["frps.acme.example.com:7000"],
    "allowCustomServer": false
  }
}
```

## 安装方式

### 从文件夹安装

将厂商文件夹（包含 `manifest.json`）的路径通过「从文件夹安装」按钮选择即可。

### 从 ZIP 安装

将厂商文件夹打包为 ZIP（`manifest.json` 在 ZIP 根目录），通过「从 ZIP 安装」按钮选择 ZIP 文件。

ZIP 结构示例：

```
acme-tunnel.zip
├── manifest.json
├── logo.png
└── bin/
    └── frpc.exe
```

> 安装后厂商文件存放在 AppData 全局目录下，所有 MoLaunch 实例共享。
