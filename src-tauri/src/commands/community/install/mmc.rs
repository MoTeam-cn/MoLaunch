//! 社区资源下载安装 - MultiMC 整合包数据结构
//! 包含 MMC mmc-pack.json 与 instance.cfg 的反序列化/解析结构。MMC 整合包不含依赖 mods
//! 列表（mods 已打包在 overrides 的 .minecraft/mods/ 中），通过 components[] 数组指定
//! Minecraft 本体与加载器版本。instance.cfg 字段（PreLaunchCommand/JvmArgs/JoinServerOnLaunch
//! 等）会被解析并迁移到版本 setup.ini。

use serde::Deserialize;

/// MMC 整合包 mmc-pack.json 结构
#[derive(Debug, Deserialize)]
pub(super) struct MmcPack {
    /// 组件列表（net.minecraft / net.minecraftforge / net.fabricmc.fabric-loader 等）
    #[serde(default)]
    pub(super) components: Vec<MmcComponent>,
}

/// MMC 组件
///
/// 常见 uid：
/// - `net.minecraft`：Minecraft 本体，version 为游戏版本
/// - `net.minecraftforge`：Forge，version 为 Forge 版本
/// - `net.neoforged`：NeoForge，version 为 NeoForge 版本
/// - `net.fabricmc.fabric-loader`：Fabric，version 为 Fabric Loader 版本
/// - `org.lwjgl.*`：LWJGL，跳过
#[derive(Debug, Deserialize)]
pub(super) struct MmcComponent {
    #[serde(default)]
    pub(super) uid: String,
    #[serde(default)]
    pub(super) version: String,
}

/// MMC instance.cfg 解析结果
///
/// instance.cfg 是 INI 格式（key=value），按行解析。字段迁移目标：
/// - `PreLaunchCommand`(`OverrideCommands=true`) → `advance_run_cmd`（变量替换）
/// - `JoinServerOnLaunchAddress`(`JoinServerOnLaunch=true`) → `server_enter`
/// - `IgnoreJavaCompatibility=true` → `advance_ignore_java_warning=true`
/// - `iconKey`(.png 存在) → `logo`（复制到版本目录 MoLaunch/Logo.png）
/// - `JvmArgs`(`OverrideJavaArgs=true` 覆盖 / `false` 追加到全局) → `advance_jvm_args`
#[derive(Debug, Default, Clone)]
pub(super) struct MmcInstanceCfg {
    /// 启动前命令（已做变量替换）
    pub(super) pre_launch_command: Option<String>,
    /// 是否覆盖启动前命令（true 时使用 pre_launch_command，false 时忽略）
    pub(super) override_commands: bool,
    /// 自动进入服务器地址
    pub(super) join_server_address: Option<String>,
    /// 是否自动进入服务器
    pub(super) join_server_on_launch: bool,
    /// 忽略 Java 兼容性警告
    pub(super) ignore_java_compatibility: bool,
    /// 图标 key（对应 .png 文件名，不含扩展名）
    pub(super) icon_key: Option<String>,
    /// JVM 参数
    pub(super) jvm_args: Option<String>,
    /// 是否覆盖 JVM 参数（true 覆盖，false 追加到全局）
    pub(super) override_java_args: bool,
}

/// 解析 MMC instance.cfg 内容
///
/// instance.cfg 是 INI 格式（key=value），按行解析。
/// 值可能用引号包围（如 `key="value"`），去除引号。
/// 空值或不存在键返回 None / false。
pub(super) fn parse_instance_cfg(content: &str) -> MmcInstanceCfg {
    let mut cfg = MmcInstanceCfg::default();

    for line in content.lines() {
        let line = line.trim();
        // 跳过注释和空行
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        // 按第一个 = 拆分
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        // 去除包围引号
        let value = if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            &value[1..value.len() - 1]
        } else {
            value
        };

        match key {
            "PreLaunchCommand" => {
                if !value.is_empty() {
                    cfg.pre_launch_command = Some(value.to_string());
                }
            }
            "OverrideCommands" => {
                cfg.override_commands = value.eq_ignore_ascii_case("true");
            }
            "JoinServerOnLaunch" => {
                cfg.join_server_on_launch = value.eq_ignore_ascii_case("true");
            }
            "JoinServerOnLaunchAddress" => {
                if !value.is_empty() {
                    cfg.join_server_address = Some(value.to_string());
                }
            }
            "IgnoreJavaCompatibility" => {
                cfg.ignore_java_compatibility = value.eq_ignore_ascii_case("true");
            }
            "iconKey" => {
                if !value.is_empty() {
                    cfg.icon_key = Some(value.to_string());
                }
            }
            "JvmArgs" => {
                if !value.is_empty() {
                    cfg.jvm_args = Some(value.to_string());
                }
            }
            "OverrideJavaArgs" => {
                cfg.override_java_args = value.eq_ignore_ascii_case("true");
            }
            _ => {}
        }
    }

    cfg
}

/// 替换 MMC PreLaunchCommand 中的变量占位符
///
/// 变量替换：`\"`→`"`；`$INST_JAVA`→`javaw`（启动时系统 PATH 解析）；
/// `$INST_MC_DIR`/`$INST_DIR`（带或不带 `\`）→ instance_dir 实际路径；
/// `$INST_ID`/`$INST_NAME` → 实例名。启动直接执行 cmd /C，迁移时替换为实际路径。
pub(super) fn substitute_pre_launch_vars(cmd: &str, instance_dir: &std::path::Path, instance_name: &str) -> String {
    let instance_path = instance_dir.to_string_lossy().to_string();
    // 反斜杠路径（Windows 习惯）
    let instance_path_win = instance_path.replace('/', "\\");

    let mut result = cmd.to_string();
    // 顺序很重要：先替换带 \ 的变体，再替换不带 \ 的（避免 $INST_DIR 误匹配 $INST_DIR\）
    result = result.replace("$INST_MC_DIR\\", &instance_path_win);
    result = result.replace("$INST_MC_DIR", &instance_path);
    result = result.replace("$INST_DIR\\", &instance_path_win);
    result = result.replace("$INST_DIR", &instance_path);
    result = result.replace("$INST_ID", instance_name);
    result = result.replace("$INST_NAME", instance_name);
    result = result.replace("$INST_JAVA", "javaw");
    // 转义引号：MMC 用 \" 表示字面量引号
    result = result.replace("\\\"", "\"");
    result
}
