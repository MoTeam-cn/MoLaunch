# MoLaunch Updater

Windows 便携版更新器（独立子进程），负责替换运行中的主程序 exe。

## 功能

1. 等待主进程退出（通过 PID，使用 OpenProcess + WaitForSingleObject）
2. 校验新 exe 的 minisign 签名（Ed25519 + SHA-512 prehash）
3. 替换旧 exe 为新 exe（MoveFileExW + 备份回退方案）
4. 启动新 exe

## 命令行参数

```
molaunch_updater.exe --old-exe <旧exe路径> --new-exe <新exe路径> --pid <主进程PID> --signature <base64签名>
```

- `--old-exe`：当前主程序 exe 的绝对路径（待替换的目标）
- `--new-exe`：下载到临时目录的新版本 exe 路径
- `--pid`：主进程 PID，用于等待其退出释放文件锁
- `--signature`：新 exe 的 minisign 签名（.sig 文件完整内容或纯 base64）

## 退出码

| 码 | 含义 |
|----|------|
| 0  | 成功完成替换并启动新 exe |
| 1  | 参数解析失败 |
| 2  | 等待主进程退出超时（30s） |
| 3  | 替换 exe 失败 |
| 4  | 启动新 exe 失败 |
| 5  | 签名校验失败 |

## 日志

日志写入 `%APPDATA%/.Molaunch/updater/updater.log`（append 模式）。
主程序启动时可读取展示给用户排查问题。

## 模块结构

```
src/
├── main.rs       入口 + 流程编排
├── args.rs       命令行参数解析
├── platform.rs   Windows API 封装（进程等待 + 文件替换 + 启动新 exe）
├── verify.rs     minisign 签名校验（ed25519-dalek + base64 + sha2）
└── log.rs        日志写入
```

## 安全设计

- **签名校验**：硬编码 Ed25519 公钥（来自 `tauri.conf.json` 的 `plugins.updater.pubkey`），
  替换前必须验证新 exe 的 minisign 签名，防止篡改
- **仅主程序调用**：通过 `--pid` + `--signature` 参数约束，其他程序无法直接利用
- **无网络能力**：updater 不发起任何网络请求，仅做本地文件操作

## 构建

```bash
cd src-tauri/updater
cargo build --release
```

构建产物：`target/release/molaunch_updater.exe`

## 集成方式

1. 构建后将 `molaunch_updater.exe` 复制到 `src-tauri/resources/updater/updater.exe`
2. 主程序通过 `resources.rs` 的 `extract_updater()` 释放到 `%APPDATA%/.Molaunch/updater/`
3. 主程序更新流程启动 updater.exe 子进程后立即退出，由 updater 完成替换

详见 `docs/updater/design.md` §4 Windows 便携版 updater。
