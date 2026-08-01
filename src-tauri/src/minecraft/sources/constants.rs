//! 镜像源 URL 常量与域名替换规则

// 基础 URL 常量

/// BMCLAPI 基础 URL
pub const BMCLAPI_BASE: &str = "https://bmclapi2.bangbang93.com";

// ── Mojang 官方 ──
pub const MOJANG_PISTON_DATA: &str = "https://piston-data.mojang.com";
pub const MOJANG_PISTON_META: &str = "https://piston-meta.mojang.com";
pub const MOJANG_LAUNCHER: &str = "https://launcher.mojang.com";
pub const MOJANG_LAUNCHERMETA: &str = "https://launchermeta.mojang.com";
pub const MOJANG_LIBRARIES: &str = "https://libraries.minecraft.net";
pub const MOJANG_RESOURCES: &str = "https://resources.download.minecraft.net";

// ── Maven 仓库 ──
pub const MAVEN_FORGE: &str = "https://maven.minecraftforge.net";
pub const MAVEN_NEOFORGE: &str = "https://maven.neoforged.net/releases";
pub const MAVEN_FABRIC: &str = "https://maven.fabricmc.net";

// ── Mod Loader 官方 ──
pub const FORGE_VERSIONS_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge";
pub const NEOFORGE_API: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
pub const NEOFORGE_API_LEGACY: &str =
    "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/forge";
pub const FABRIC_META: &str = "https://meta.fabricmc.net/v2/versions/loader";
pub const LITELOADER_VERSIONS: &str = "https://dl.liteloader.com/versions/versions.json";

// ── BMCLAPI 路径 ──
pub const BMCLAPI_VERSION_MANIFEST: &str = "/mc/game/version_manifest.json";
pub const BMCLAPI_OPTIFINE: &str = "/optifine/versionList";
pub const BMCLAPI_FABRIC_META: &str = "/fabric-meta/v2/versions/loader";
pub const BMCLAPI_NEOFORGE: &str =
    "/neoforge/meta/api/maven/details/releases/net/neoforged/neoforge";
pub const BMCLAPI_NEOFORGE_LEGACY: &str =
    "/neoforge/meta/api/maven/details/releases/net/neoforged/forge";
pub const BMCLAPI_LITELOADER: &str = "/maven/com/mumfrey/liteloader/versions.json";

// 域名替换规则

/// Mojang 域名 -> BMCLAPI（直传）
pub const MOJANG_REPLACEMENTS: &[(&str, &str)] = &[
    (MOJANG_PISTON_DATA, BMCLAPI_BASE),
    (MOJANG_PISTON_META, BMCLAPI_BASE),
    (MOJANG_LAUNCHER, BMCLAPI_BASE),
    (MOJANG_LAUNCHERMETA, BMCLAPI_BASE),
];

/// Maven 仓库 -> BMCLAPI/maven
pub const MAVEN_REPLACEMENTS: &[(&str, &str)] = &[
    (MAVEN_FORGE, "https://bmclapi2.bangbang93.com/maven"),
    (MAVEN_NEOFORGE, "https://bmclapi2.bangbang93.com/maven"),
    (MAVEN_FABRIC, "https://bmclapi2.bangbang93.com/maven"),
];

/// Minecraft 库 -> BMCLAPI/libraries
pub const LIBRARY_REPLACEMENTS: &[(&str, &str)] = &[(
    MOJANG_LIBRARIES,
    "https://bmclapi2.bangbang93.com/libraries",
)];

// ── 社区资源 CDN 镜像（CurseForge + Modrinth 统一由 source 策略控制）──

/// Modrinth CDN 原始域名（官方 CDN，对中国大陆用户会跳转到慢速 cdn-alt）
pub(super) const MR_CDN_OFFICIAL: &str = "https://cdn.modrinth.com";

/// Modrinth CDN 直连域名（绕过 cdn-alt 跳转，速度更快）
/// 路径结构与官方 CDN 完全一致，仅替换域名即可
/// 示例：https://cdn.modrinth.com/data/xxx/versions/yyy/zzz.mrpack
///    → https://cdn-raw.modrinth.com/data/xxx/versions/yyy/zzz.mrpack
pub(super) const MR_CDN_RAW: &str = "https://cdn-raw.modrinth.com";

/// Modrinth 自建 CDN 镜像域名（mocdn.net）
/// 路径结构与官方 CDN 完全一致，仅替换域名即可
pub(super) const MR_MOCDN_MIRROR: &str = "https://cdn-modrinth.mocdn.net";

/// CurseForge 自建 CDN 镜像域名（仅支持 edge.forgecdn.net 路径）
/// media.forgecdn.net 路径不支持，走官方源
pub(super) const CF_MOCDN_MIRROR: &str = "https://cdn-curseforge.mocdn.net";

// ── authlib-injector（外置登录支持库）下载源 ──
/// authlib-injector 官方源（yushi.moe）
pub const AUTHLIB_INJECTOR_OFFICIAL: &str = "https://authlib-injector.yushi.moe";
/// authlib-injector 最新版元数据路径（拼接在官方源后）
pub const AUTHLIB_INJECTOR_LATEST_PATH: &str = "/artifact/latest.json";
/// authlib-injector BMCLAPI 镜像源
pub const AUTHLIB_INJECTOR_BMCLAPI: &str =
    "https://bmclapi2.bangbang93.com/mirrors/authlib-injector";

/// CurseForge CDN 官方域名列表
pub const CF_CDN_DOMAINS: &[&str] = &["https://edge.forgecdn.net", "https://media.forgecdn.net"];

/// Modrinth CDN 官方域名列表（含原始域名和直连域名，均需镜像替换）
pub const MR_CDN_DOMAINS: &[&str] = &[MR_CDN_OFFICIAL, MR_CDN_RAW];
