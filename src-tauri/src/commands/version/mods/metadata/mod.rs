//! Jar 内 Mod 元数据读取流水线
//! 版本号识别链：mcmod.info（Forge 1.12-）→ fabric.mod.json（Fabric/Quilt）→
//! META-INF/mods.toml（Forge 1.13+/NeoForge）→ fml_cache_annotation.json（Forge 1.7-1.12
//! 注解缓存）。按顺序累积合并，已有有效值不覆盖。`${file.jarVersion}` 占位符最后从
//! MANIFEST.MF Implementation-Version 解析；版本号须含 "." 或 "-"，否则视为无效。

mod builder;
mod sources;

pub(crate) use builder::read_mod_metadata;
