//! 资源管理模块
//! 资源释放机制：所有外部资源（文本模板、二进制 jar）在编译时通过 include_str!/include_bytes!
//! 嵌入二进制，运行时零文件 IO 读取；二进制资源释放时带 sha256 校验，只在目标文件不存在或
//! hash 不匹配时写入，避免每次启动重复写盘。取代原基于 env!("CARGO_MANIFEST_DIR") 拼路径的实现（打包后路径不存在，发布版 bug）。

use crate::{log_info, log_warn};
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
        // 项目许可协议全文（由 build.rs 从项目根目录 LICENSE 自动同步的副本，编译期嵌入二进制，
        // 供「设置 - 更多 - 许可协议」子页签展示；根 LICENSE 变更后重新构建即自动更新）
        "LICENSE.txt" => Some(include_str!("../resources/LICENSE.txt")),
        // 示例文件（供开发者测试调试，前端导出时通过 IPC 读取）
        "samples/plugin/manifest.json" => {
            Some(include_str!("../resources/samples/plugin/manifest.json"))
        }
        // AI 提示词模板（由 ai_core::prompt 读取；改文案后重新编译即可）
        "prompts/chat.md" => Some(include_str!("../resources/prompts/chat.md")),
        "prompts/title.md" => Some(include_str!("../resources/prompts/title.md")),
        "prompts/summarize.md" => Some(include_str!("../resources/prompts/summarize.md")),
        "prompts/crash_log.md" => Some(include_str!("../resources/prompts/crash_log.md")),
        "prompts/log_analysis.md" => Some(include_str!("../resources/prompts/log_analysis.md")),
        "prompts/log_analyze_steps.md" => {
            Some(include_str!("../resources/prompts/log_analyze_steps.md"))
        }
        "samples/plugin/index.html" => Some(include_str!("../resources/samples/plugin/index.html")),
        "samples/layout/layout-sample.json" => Some(include_str!(
            "../resources/samples/layout/layout-sample.json"
        )),
        "samples/layout/layout-sample.xml" => Some(include_str!(
            "../resources/samples/layout/layout-sample.xml"
        )),
        "samples/layout/layout-sample.html" => Some(include_str!(
            "../resources/samples/layout/layout-sample.html"
        )),
        // picker 子窗口 HTML 模板（由 picker_window URI scheme handler 读取并注入数据）
        "templates/port-picker.html" => {
            Some(include_str!("../resources/templates/port-picker.html"))
        }
        "templates/redirect.html" => Some(include_str!("../resources/templates/redirect.html")),
        "templates/confirm.html" => Some(include_str!("../resources/templates/confirm.html")),
        "templates/info.html" => Some(include_str!("../resources/templates/info.html")),
        "templates/image-viewer.html" => {
            Some(include_str!("../resources/templates/image-viewer.html"))
        }
        "templates/markdown.html" => Some(include_str!("../resources/templates/markdown.html")),
        "templates/tutorial-basics.html" => {
            Some(include_str!("../resources/templates/tutorial-basics.html"))
        }
        "templates/tutorial-frp.html" => {
            Some(include_str!("../resources/templates/tutorial-frp.html"))
        }
        "templates/base-help.html" => Some(include_str!("../resources/templates/base-help.html")),
        "templates/qrcode.html" => Some(include_str!("../resources/templates/qrcode.html")),
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
        // java-wrapper.jar：第三方开源项目 Java Launch Wrapper（MIT，https://github.com/00ll00/java_launch_wrapper），
        // 用于规避 JDK-8272352（非 ASCII 路径命令行参数乱码）启动问题
        "java-wrapper.jar" => Some(include_bytes!("../resources/java-wrapper.jar")),
        // lwjgl-unsafe-agent.jar：第三方开源项目 lwjgl-unsafe-agent（Apache-2.0，https://github.com/HMCL-dev/lwjgl-unsafe-agent）
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
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        "sdk/run_sdk_lib-macos-aarch64.dylib" => Some(include_bytes!(
            "../resources/sdk/run_sdk_lib-macos-aarch64.dylib"
        )),
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        "sdk/run_sdk_lib-macos-x86_64.dylib" => Some(include_bytes!(
            "../resources/sdk/run_sdk_lib-macos-x86_64.dylib"
        )),
        #[cfg(target_os = "linux")]
        "sdk/run_sdk_lib-linux-x86_64.so" => Some(include_bytes!(
            "../resources/sdk/run_sdk_lib-linux-x86_64.so"
        )),
        // marked.min.js（Markdown 渲染库，由 picker scheme 内联注入 markdown 模板）
        // 来源：https://cdn.jsdelivr.net/npm/marked/marked.min.js
        // 用于 picker 子窗口的 markdown.html 模板渲染 markdown 文本
        "view/marked.min.js" => Some(include_bytes!("../resources/view/marked.min.js")),
        // qrcode.min.js（二维码生成库，由 picker scheme 内联注入 qrcode 模板）
        // 来源：https://github.com/davidshimjs/qrcodejs
        // 用于 picker 子窗口的 qrcode.html 模板生成二维码（DOM 渲染，离线可用）
        "view/qrcode.min.js" => Some(include_bytes!("../resources/view/qrcode.min.js")),
        // dompurify.min.js（HTML 消毒库，picker markdown 模板渲染前消毒 marked 输出防 XSS）
        // 来源：https://cdn.jsdelivr.net/npm/dompurify/dist/purify.min.js
        // 由 scheme.rs 内联注入，用于 picker 子窗口的 markdown.html 模板
        "view/dompurify.min.js" => Some(include_bytes!("../resources/view/dompurify.min.js")),
        // updater.exe（Windows 便携版更新器，独立子进程）
        // 来源：src-tauri/updater/ 独立 Cargo 项目构建产物
        // 运行时由 `extract_updater` 释放到 %APPDATA%/.Molaunch/updater/updater.exe
        // 作用：主程序退出后替换 exe 文件，绕过 Windows 文件锁
        // See: docs/updater/design.md §4 Windows 便携版 updater
        #[cfg(target_os = "windows")]
        "updater/updater.exe" => Some(include_bytes!("../resources/updater/updater.exe")),
        // easytier-core（联机虚拟组网核心，按平台/架构嵌入对应版本）
        // 来源：https://github.com/EasyTier/EasyTier/releases（v2.6.4，Apache-2.0）
        // 运行时由 `extract_easytier_core` 释放到 AppData/.Molaunch/easytier/
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        "easytier/easytier-core" => Some(include_bytes!(
            "../resources/easytier/windows/x86_64/easytier-core.exe"
        )),
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        "easytier/easytier-core" => Some(include_bytes!(
            "../resources/easytier/windows/aarch64/easytier-core.exe"
        )),
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        "easytier/easytier-core" => Some(include_bytes!(
            "../resources/easytier/linux/x86_64/easytier-core"
        )),
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        "easytier/easytier-core" => Some(include_bytes!(
            "../resources/easytier/linux/aarch64/easytier-core"
        )),
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        "easytier/easytier-core" => Some(include_bytes!(
            "../resources/easytier/macos/x86_64/easytier-core"
        )),
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        "easytier/easytier-core" => Some(include_bytes!(
            "../resources/easytier/macos/aarch64/easytier-core"
        )),
        // easytier-cli（联机管理客户端，与 core 同版本同目录释放；用于房客经 rpc-portal 查询虚拟网络节点发现联机中心）
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        "easytier/easytier-cli" => Some(include_bytes!(
            "../resources/easytier/windows/x86_64/easytier-cli.exe"
        )),
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        "easytier/easytier-cli" => Some(include_bytes!(
            "../resources/easytier/windows/aarch64/easytier-cli.exe"
        )),
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        "easytier/easytier-cli" => Some(include_bytes!(
            "../resources/easytier/linux/x86_64/easytier-cli"
        )),
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        "easytier/easytier-cli" => Some(include_bytes!(
            "../resources/easytier/linux/aarch64/easytier-cli"
        )),
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        "easytier/easytier-cli" => Some(include_bytes!(
            "../resources/easytier/macos/x86_64/easytier-cli"
        )),
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        "easytier/easytier-cli" => Some(include_bytes!(
            "../resources/easytier/macos/aarch64/easytier-cli"
        )),
        // easytier 依赖动态库（Windows 专属，Packet.dll + wintun.dll 与核心同目录释放）
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        "easytier/Packet.dll" => Some(include_bytes!(
            "../resources/easytier/windows/x86_64/Packet.dll"
        )),
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        "easytier/Packet.dll" => Some(include_bytes!(
            "../resources/easytier/windows/aarch64/Packet.dll"
        )),
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        "easytier/wintun.dll" => Some(include_bytes!(
            "../resources/easytier/windows/x86_64/wintun.dll"
        )),
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        "easytier/wintun.dll" => Some(include_bytes!(
            "../resources/easytier/windows/aarch64/wintun.dll"
        )),
        // hongshi（红石联机内核，按平台/架构嵌入对应版本；Windows 仅 amd64 官方产物）
        // 来源：https://hongshi.site/docs-api.html（人工下载入库，见 docs/REDSTONE_ONLINE_DESIGN.md 附录 A）
        // 运行时由 `extract_hongshi_core` 释放到 AppData/.Molaunch/hongshi/
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        "hongshi/hongshi" => Some(include_bytes!(
            "../resources/hongshi/windows/x86_64/hongshi-windows-amd64.exe"
        )),
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        "hongshi/hongshi" => Some(include_bytes!(
            "../resources/hongshi/linux/x86_64/hongshi-linux-amd64"
        )),
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        "hongshi/hongshi" => Some(include_bytes!(
            "../resources/hongshi/linux/aarch64/hongshi-linux-arm64"
        )),
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        "hongshi/hongshi" => Some(include_bytes!(
            "../resources/hongshi/macos/x86_64/hongshi-darwin-amd64"
        )),
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        "hongshi/hongshi" => Some(include_bytes!(
            "../resources/hongshi/macos/aarch64/hongshi-darwin-arm64"
        )),
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

/// 释放二进制资源到目标路径（带 sha256 校验）
///
/// 只在目标文件不存在或 sha256 不匹配时写入。
/// 这样每次启动不会因为重复写大文件而拖慢，也避免杀软误报频繁触发。
///
/// 释放成功后会在同目录写一个 `{name}.sha256` 记录 hash（仅提示用，命中判断基于目标本体）。
pub fn extract_resource(resource_path: &str, target_path: &Path) -> anyhow::Result<()> {
    let content = read_resource_bytes(resource_path)?;
    let expected_hash = crate::utils::hash::sha256_hex(&content);

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // 校验文件路径：与目标同目录，后缀 .sha256（仅作提示用，不参与信任判断）
    let hash_path = target_path.with_extension("sha256");

    // 命中缓存：目标文件存在且本体字节 hash 与期望一致才跳过写入
    // .sha256 文本可被伪造，信任判断必须基于目标文件本体
    let mut cached = false;
    if target_path.exists() {
        match std::fs::read(target_path) {
            Ok(bytes) if crate::utils::hash::sha256_hex(&bytes) == expected_hash => cached = true,
            Ok(_) => {
                log_warn!(
                    "资源 {} 的目标文件 hash 不匹配（期望 {}），重新释放",
                    resource_path,
                    expected_hash
                );
            }
            Err(e) => {
                log_warn!("资源 {} 读取失败: {}，重新释放", resource_path, e);
            }
        }
    }

    if cached {
        return Ok(());
    }

    std::fs::write(target_path, &content)?;
    std::fs::write(&hash_path, &expected_hash)?;
    // 限制目标文件权限（尽力而为），降低恶意替换/篡改面
    crate::minecraft::system::shell::restrict_file_permissions(target_path);
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
/// `<temp>/MoLaunch/sdk/` 目录供 `libloading` 加载。复用 `extract_resource`
/// 的 sha256 校验机制：hash 一致跳过释放（支持手动热更新），hash 不一致自动覆盖。
pub fn extract_sdk() -> anyhow::Result<std::path::PathBuf> {
    let sdk_filename = crate::sdk::get_sdk_filename();
    let resource_path = format!("sdk/{}", sdk_filename);

    let target_path = crate::utils::cache_temp::sdk_library_path(sdk_filename);

    extract_resource(&resource_path, &target_path)?;

    Ok(target_path)
}

/// 释放 updater.exe 到 AppData 全局目录（Windows 专属）
///
/// updater.exe 是便携版更新器，由 `src-tauri/updater/` 独立 Cargo 项目构建，
/// 编译时通过 `include_bytes!` 嵌入主程序，运行时释放到
/// `%APPDATA%/.Molaunch/updater/updater.exe` 供更新流程调用。
/// 复用 `extract_resource` 的 sha256 校验机制。
/// 调用方为 `commands::system::updater::download_and_install`。
#[cfg(target_os = "windows")]
pub fn extract_updater() -> anyhow::Result<std::path::PathBuf> {
    let appdata = std::env::var("APPDATA")
        .map_err(|_| anyhow::anyhow!("APPDATA environment variable not set"))?;
    let target_path = std::path::PathBuf::from(appdata)
        .join(".Molaunch")
        .join("updater")
        .join("updater.exe");

    extract_resource("updater/updater.exe", &target_path)?;

    Ok(target_path)
}

/// 释放 easytier-core 到 AppData 全局目录，返回可执行文件路径
///
/// easytier-core 是联机虚拟组网核心，编译时按平台/架构嵌入对应版本，运行时
/// 释放到 `<appdata>/.Molaunch/easytier/`（Windows 同时释放依赖的
/// Packet.dll/wintun.dll，与核心同目录以便加载）。easytier-cli 管理客户端
/// 与核心同版本同目录释放（供房客查询虚拟网络节点）。复用 `extract_resource`
/// 的 sha256 校验机制。Unix 下 `extract_resource` 的 chmod 600 会去掉执行位，
/// 故释放后需补回执行权限。调用方为 `commands::online::manager::easytier_actions`。
pub fn extract_easytier_core() -> anyhow::Result<std::path::PathBuf> {
    let dir = crate::storage::appdata::ensure_appdata_subdir("easytier")
        .map_err(|e| anyhow::anyhow!(e))?;

    #[cfg(target_os = "windows")]
    {
        extract_resource("easytier/Packet.dll", &dir.join("Packet.dll"))?;
        extract_resource("easytier/wintun.dll", &dir.join("wintun.dll"))?;
    }

    let (core_name, cli_name) = if cfg!(target_os = "windows") {
        ("easytier-core.exe", "easytier-cli.exe")
    } else {
        ("easytier-core", "easytier-cli")
    };
    let core_path = dir.join(core_name);
    extract_resource("easytier/easytier-core", &core_path)?;
    let cli_path = dir.join(cli_name);
    extract_resource("easytier/easytier-cli", &cli_path)?;

    #[cfg(unix)]
    {
        crate::minecraft::system::shell::make_executable(&core_path);
        crate::minecraft::system::shell::make_executable(&cli_path);
    }

    Ok(core_path)
}

/// 释放 hongshi 内核到 AppData 全局目录，返回可执行文件路径
///
/// hongshi 是红石联机内核，编译时按平台/架构嵌入对应版本（Windows 仅 amd64 官方
/// 产物），运行时释放到 `<appdata>/.Molaunch/hongshi/`，状态文件 tunnel.ini 写
/// 同目录。复用 `extract_resource` 的 sha256 校验机制；Unix 下需补回执行权限。
/// 调用方为 `commands::redstone::manager`。
pub fn extract_hongshi_core() -> anyhow::Result<std::path::PathBuf> {
    let dir = crate::storage::appdata::ensure_appdata_subdir("hongshi")
        .map_err(|e| anyhow::anyhow!(e))?;
    let target_name = if cfg!(target_os = "windows") {
        "hongshi.exe"
    } else {
        "hongshi"
    };
    let target = dir.join(target_name);
    extract_resource("hongshi/hongshi", &target)?;
    #[cfg(unix)]
    {
        crate::minecraft::system::shell::make_executable(&target);
    }
    Ok(target)
}
