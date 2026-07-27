//! 资源管理模块
//!
//! 资源释放机制：
//! 所有外部资源（文本模板、二进制 jar）在编译时通过 include_str!/include_bytes!
//! 嵌入二进制，运行时零文件 IO 读取；二进制资源释放时带 sha256 校验，
//! 只在目标文件不存在或 hash 不匹配时写入，避免每次启动重复写盘。
//!
//! 取代了原先基于 env!("CARGO_MANIFEST_DIR") 拼路径的实现——
//! 那种方式在打包到用户机器后路径不存在，属于发布版 bug。

use crate::{log_info, log_warn};
use sha2::{Digest, Sha256};
use std::path::Path;

/// 嵌入的文本资源内容
///
/// 编译时通过 include_str! 把 resources/ 下的文件内容直接打进二进制。
/// 新增文本资源时，在此 match 中追加一条分支即可。
fn embedded_text(path: &str) -> Option<&'static str> {
    match path {
        "defaults/config.ini" => Some(include_str!("../resources/defaults/config.ini")),
        "defaults/instance.ini" => Some(include_str!("../resources/defaults/instance.ini")),
        "defaults/setup.ini" => Some(include_str!("../resources/defaults/setup.ini")),
        "moddata.txt" => Some(include_str!("../resources/moddata.txt")),
        // 关于页面数据（markdown 表格格式，由 commands/system/about.rs 解析）
        "about/acknowledgements.txt" => {
            Some(include_str!("../resources/about/acknowledgements.txt"))
        }
        "about/frontend-deps.txt" => Some(include_str!("../resources/about/frontend-deps.txt")),
        "about/frontend-dev-deps.txt" => {
            Some(include_str!("../resources/about/frontend-dev-deps.txt"))
        }
        "about/backend-deps.txt" => Some(include_str!("../resources/about/backend-deps.txt")),
        "about/licenses.txt" => Some(include_str!("../resources/about/licenses.txt")),
        // 示例文件（供开发者测试调试，前端导出时通过 IPC 读取）
        "samples/plugin/manifest.json" => {
            Some(include_str!("../resources/samples/plugin/manifest.json"))
        }
        "samples/plugin/index.html" => Some(include_str!("../resources/samples/plugin/index.html")),
        "samples/layout/layout-sample.json" => {
            Some(include_str!("../resources/samples/layout/layout-sample.json"))
        }
        "samples/layout/layout-sample.xml" => {
            Some(include_str!("../resources/samples/layout/layout-sample.xml"))
        }
        "samples/layout/layout-sample.html" => {
            Some(include_str!("../resources/samples/layout/layout-sample.html"))
        }
        _ => None,
    }
}

/// 嵌入的二进制资源内容
///
/// 编译时通过 include_bytes! 把 resources/ 下的二进制文件直接打进二进制。
/// 新增二进制资源时，在此 match 中追加一条分支即可。
///
/// SDK 动态库按平台条件编译，只注册当前平台的文件，避免跨平台编译时
/// 因文件不存在而导致 include_bytes! 编译失败。
fn embedded_bytes(path: &str) -> Option<&'static [u8]> {
    match path {
        "forge-installer.jar" => Some(include_bytes!("../resources/forge-installer.jar")),
        "java-wrapper.jar" => Some(include_bytes!("../resources/java-wrapper.jar")),
        "lwjgl-unsafe-agent.jar" => Some(include_bytes!("../resources/lwjgl-unsafe-agent.jar")),
        // 离线皮肤 PNG（9 个默认角色）
        "skins/Steve.png" => Some(include_bytes!("../resources/skins/Steve.png")),
        "skins/Alex.png" => Some(include_bytes!("../resources/skins/Alex.png")),
        "skins/Ari.png" => Some(include_bytes!("../resources/skins/Ari.png")),
        "skins/Efe.png" => Some(include_bytes!("../resources/skins/Efe.png")),
        "skins/Kai.png" => Some(include_bytes!("../resources/skins/Kai.png")),
        "skins/Makena.png" => Some(include_bytes!("../resources/skins/Makena.png")),
        "skins/Noor.png" => Some(include_bytes!("../resources/skins/Noor.png")),
        "skins/Sunny.png" => Some(include_bytes!("../resources/skins/Sunny.png")),
        "skins/Zuri.png" => Some(include_bytes!("../resources/skins/Zuri.png")),
        #[cfg(target_os = "windows")]
        "sdk/run_sdk_lib-windows-x86_64.dll" => Some(include_bytes!(
            "../resources/sdk/run_sdk_lib-windows-x86_64.dll"
        )),
        #[cfg(target_os = "macos")]
        "sdk/run_sdk_lib-darwin-aarch64.dylib" => Some(include_bytes!(
            "../resources/sdk/run_sdk_lib-darwin-aarch64.dylib"
        )),
        #[cfg(target_os = "linux")]
        "sdk/run_sdk_lib-linux-x86_64.so" => Some(include_bytes!(
            "../resources/sdk/run_sdk_lib-linux-x86_64.so"
        )),
        // cubiomes WASM（前端种子地图工具通过 res:// 协议加载）
        // 由 build.rs 调用 emcc 编译生成到 resources/wasm/
        // - cubiomes.wasm: 二进制模块
        // - cubiomes.js: Emscripten 胶水代码（MODULARIZE=1, EXPORT_NAME=createCubiomesModule）
        //                worker 内通过 importScripts 加载并调用 createCubiomesModule() 实例化
        "wasm/cubiomes.wasm" => Some(include_bytes!("../resources/wasm/cubiomes.wasm")),
        "wasm/cubiomes.js" => Some(include_bytes!("../resources/wasm/cubiomes.js")),
        // wintun.dll（Windows 联机模块 TUN 接口驱动，按 target_arch 嵌入对应架构版本）
        // 来源：https://www.wintun.net/（WireGuard 项目，已签名）
        // 运行时由 `extract_wintun` 释放到 %APPDATA%/.MolaLaunch/wintun.dll
        // 注册统一逻辑路径 `wintun/wintun.dll`，编译时按架构选择不同物理文件
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        "wintun/wintun.dll" => Some(include_bytes!("../resources/wintun/amd64/wintun.dll")),
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        "wintun/wintun.dll" => Some(include_bytes!("../resources/wintun/arm64/wintun.dll")),
        #[cfg(all(target_os = "windows", target_arch = "x86"))]
        "wintun/wintun.dll" => Some(include_bytes!("../resources/wintun/x86/wintun.dll")),
        #[cfg(all(target_os = "windows", target_arch = "arm"))]
        "wintun/wintun.dll" => Some(include_bytes!("../resources/wintun/arm/wintun.dll")),
        _ => None,
    }
}

/// 获取嵌入的二进制资源内容（公共接口）
///
/// 供其他模块（如 `skin_resourcepack`）直接读取嵌入的 PNG 等二进制资源，
/// 无需通过 `extract_resource` 释放到磁盘。
pub fn get_embedded_resource(path: &str) -> Option<&'static [u8]> {
    embedded_bytes(path)
}

/// 读取文本资源内容
///
/// 资源在编译时已嵌入二进制，运行时直接返回，不做任何文件 IO。
pub fn read_resource(relative_path: &str) -> anyhow::Result<String> {
    match embedded_text(relative_path) {
        Some(content) => Ok(content.to_string()),
        None => Err(anyhow::anyhow!(
            "未注册的文本资源: {}（请在 resources.rs 的 embedded_text 中登记）",
            relative_path
        )),
    }
}

/// 读取二进制资源内容
///
/// 资源在编译时已嵌入二进制，运行时直接返回，不做任何文件 IO。
pub fn read_resource_bytes(relative_path: &str) -> anyhow::Result<Vec<u8>> {
    match embedded_bytes(relative_path) {
        Some(content) => Ok(content.to_vec()),
        None => Err(anyhow::anyhow!(
            "未注册的二进制资源: {}（请在 resources.rs 的 embedded_bytes 中登记）",
            relative_path
        )),
    }
}

/// 计算数据的 sha256 十六进制摘要
fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    result.iter().map(|b| format!("{:02x}", b)).collect()
}

/// 释放二进制资源到目标路径（带 sha256 校验）
///
/// 只在目标文件不存在或 sha256 不匹配时写入。
/// 这样每次启动不会因为重复写大文件而拖慢，也避免杀软误报频繁触发。
///
/// 释放成功后会在同目录写一个 `{name}.sha256` 校验文件，用于下次启动比对。
pub fn extract_resource(resource_path: &str, target_path: &Path) -> anyhow::Result<()> {
    let content = read_resource_bytes(resource_path)?;
    let expected_hash = sha256_hex(&content);

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 校验文件路径：与目标同目录，后缀 .sha256
    let hash_path = target_path.with_extension("sha256");

    // 命中缓存：文件存在 + 校验文件存在 + hash 一致 → 跳过写入
    // 注意：必须同时检查 target 和 hash 文件存在，否则会出现
    // "目标文件不存在但 hash 文件残留"时读到旧 hash 触发"不匹配"警告的情况
    let target_exists = target_path.exists();
    let hash_exists = hash_path.exists();
    if target_exists && hash_exists {
        let cached_hash = std::fs::read_to_string(&hash_path).unwrap_or_default();
        if cached_hash.trim() == expected_hash {
            return Ok(());
        }
        log_warn!(
            "资源 {} 的缓存文件 hash 不匹配（期望 {}，实际 {}），重新释放",
            resource_path,
            expected_hash,
            cached_hash.trim()
        );
    } else if target_exists != hash_exists {
        // 文件和 hash 文件只存在一个：状态不一致，需要重新释放
        log_warn!(
            "资源 {} 的缓存状态不一致（target={}, hash={}），重新释放",
            resource_path,
            target_exists,
            hash_exists
        );
    }
    // 两者都不存在：首次释放，不打印警告

    std::fs::write(target_path, &content)?;
    std::fs::write(&hash_path, &expected_hash)?;
    log_info!(
        "释放嵌入资源: {} -> {} (sha256={})",
        resource_path,
        target_path.display(),
        &expected_hash[..12]
    );
    Ok(())
}

/// 释放 SDK 动态库到临时目录，返回释放后的文件路径
///
/// SDK 在编译时通过 `include_bytes!` 嵌入二进制，运行时释放到
/// `<temp>/MoLaunch/sdk/` 目录供 `libloading` 加载。
///
/// 释放策略（复用 `extract_resource` 的 sha256 校验机制）：
/// - **SDK 热更新**（手动替换临时目录文件）：hash 文件不变仍匹配 → 跳过释放，加载热更新后的文件
/// - **主程序更新后启动**：嵌入 SDK 的 sha256 变了，与 hash 文件不匹配 → 自动释放覆盖旧版
/// - **临时目录被清理**：文件不存在 → 重新释放
pub fn extract_sdk() -> anyhow::Result<std::path::PathBuf> {
    let sdk_filename = crate::sdk::get_sdk_filename();
    let resource_path = format!("sdk/{}", sdk_filename);

    let target_path = crate::utils::cache_temp::sdk_library_path(sdk_filename);

    extract_resource(&resource_path, &target_path)?;

    Ok(target_path)
}

/// 释放 wintun.dll 到 AppData 全局目录（Windows 专属）
///
/// wintun.dll 是 WireGuard 项目的 Windows 虚拟网卡驱动，由 `tun-rs` 在运行时
/// 通过 `libloading` 加载。放 AppData 而非 temp 目录，避免 temp 被清理导致后续
/// 启动失败；放全局位置而非启动器目录，支持多实例共享同一份驱动。
///
/// # 释放路径
///
/// - **Windows**：`%APPDATA%/.MolaLaunch/wintun.dll`
///   （与 `OnlineStorage::appdata_device_path` 同根目录，便于统一管理）
/// - **非 Windows**：本函数不存在（`#[cfg(target_os = "windows")]` 编译期排除）
///
/// # 架构支持
///
/// 编译时 `embedded_bytes` 按 `target_arch` 选择对应架构的 dll 嵌入：
/// - `x86_64` → `resources/wintun/amd64/wintun.dll`
/// - `aarch64` → `resources/wintun/arm64/wintun.dll`
/// - `x86` → `resources/wintun/x86/wintun.dll`
/// - `arm` → `resources/wintun/arm/wintun.dll`
///
/// 运行时统一释放到 `%APPDATA%/.MolaLaunch/wintun.dll`，调用方无需关心架构。
///
/// # 释放策略
///
/// 复用 `extract_resource` 的 sha256 校验机制：
/// - 首次释放：写入 dll + sha256 校验文件
/// - 后续启动：hash 一致 → 跳过写盘；hash 不一致（主程序更新后嵌入的 dll 变了）→ 覆盖释放
///
/// # 调用方
///
/// `minecraft::online::bridge::VirtualLanBridge::start` 在创建 TUN 接口前调用，
/// 拿到释放后的绝对路径后传给 `VirtualNet::create(..., Some(path))`。
#[cfg(target_os = "windows")]
pub fn extract_wintun() -> anyhow::Result<std::path::PathBuf> {
    let appdata = std::env::var("APPDATA")
        .map_err(|_| anyhow::anyhow!("APPDATA environment variable not set"))?;
    let target_path = std::path::PathBuf::from(appdata)
        .join(".MolaLaunch")
        .join("wintun.dll");

    extract_resource("wintun/wintun.dll", &target_path)?;

    Ok(target_path)
}
