//! 存储文件读写：配置 INI、实例信息与通用文件操作

use super::ini;
use super::Storage;

impl Storage {
    pub fn read_instance(&self) -> anyhow::Result<ini::IniFile> {
        let path = self.instance_path();
        if !path.exists() {
            return Ok(ini::IniFile::new());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(ini::IniFile::parse(&content))
    }

    pub fn write_instance(&self, ini: &ini::IniFile) -> anyhow::Result<()> {
        std::fs::write(self.instance_path(), ini.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(self.instance_path()) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(self.instance_path(), perms);
            }
        }
        Ok(())
    }

    pub fn read_config(&self) -> anyhow::Result<ini::IniFile> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(ini::IniFile::new());
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(ini::IniFile::parse(&content))
    }

    pub fn write_config(&self, config: &ini::IniFile) -> anyhow::Result<()> {
        // 原子写入：先写 .tmp 再 rename，避免崩溃导致配置文件半写状态。
        // tmp 文件名含自增序号，防止多个 apply_config 并发写入时共用同一
        // tmp 文件导致 rename 找不到源文件（os error 2）。
        use std::sync::atomic::{AtomicU64, Ordering};
        static TMP_SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = TMP_SEQ.fetch_add(1, Ordering::Relaxed);

        let target = self.config_path();
        // config.ini -> config.ini.tmp{seq}
        let mut tmp = target.as_os_str().to_owned();
        tmp.push(format!(".tmp{}", seq));
        let tmp = std::path::PathBuf::from(tmp);

        std::fs::write(&tmp, config.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(&tmp) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(&tmp, perms);
            }
        }
        // rename 在同分区是原子的（POSIX/Windows 均保证）
        std::fs::rename(&tmp, &target)?;
        Ok(())
    }

    pub fn get_config(&self, section: &str, key: &str) -> Option<String> {
        self.read_config().ok()?.get(section, key)
    }

    pub fn set_config(&self, section: &str, key: &str, value: &str) -> anyhow::Result<()> {
        let mut config = self.read_config()?;
        config.set(section, key, value);
        self.write_config(&config)?;
        Ok(())
    }

    pub fn remove_config(&self, section: &str, key: &str) -> anyhow::Result<()> {
        let mut config = self.read_config()?;
        config.remove(section, key);
        self.write_config(&config)?;
        Ok(())
    }

    pub fn exists(&self, relative_path: &str) -> bool {
        self.base_dir.join(relative_path).exists()
    }

    pub fn read_file(&self, relative_path: &str) -> anyhow::Result<String> {
        let path = self.base_dir.join(relative_path);
        Ok(std::fs::read_to_string(&path)?)
    }

    pub fn write_file(&self, relative_path: &str, content: &str) -> anyhow::Result<()> {
        let path = self.base_dir.join(relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn remove_file(&self, relative_path: &str) -> anyhow::Result<()> {
        let path = self.base_dir.join(relative_path);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
        Ok(())
    }

    pub fn list_dir(&self, relative_path: &str) -> anyhow::Result<Vec<String>> {
        let path = self.base_dir.join(relative_path);
        let mut entries = Vec::new();
        if path.exists() && path.is_dir() {
            for entry in std::fs::read_dir(&path)? {
                let entry = entry?;
                if let Some(name) = entry.file_name().to_str() {
                    entries.push(name.to_string());
                }
            }
        }
        Ok(entries)
    }
}
