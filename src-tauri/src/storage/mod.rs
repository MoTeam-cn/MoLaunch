//! Storage 模块 - 统一管理 .molaunch 文件夹
//!
//! 所有对 .molaunch 文件夹的操作都必须通过此模块进行
//! 使用 INI 格式存储配置

pub mod ini;

use crate::resources;
use std::path::PathBuf;
use std::sync::Once;

static INIT: Once = Once::new();

pub struct Storage {
    base_dir: PathBuf,
}

impl Storage {
    pub fn instance() -> &'static Storage {
        static mut INSTANCE: Option<Storage> = None;
        unsafe {
            INIT.call_once(|| {
                let base_dir = Self::resolve_base_dir();
                INSTANCE = Some(Storage { base_dir });
            });
            INSTANCE.as_ref().unwrap()
        }
    }

    fn resolve_base_dir() -> PathBuf {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join(".molaunch");
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            return cwd.join(".molaunch");
        }
        PathBuf::from(".molaunch")
    }

    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    pub fn init(&self) -> anyhow::Result<()> {
        if !self.base_dir.exists() {
            std::fs::create_dir_all(&self.base_dir)?;
            log::info!("Created storage directory: {}", self.base_dir.display());
        }

        self.ensure_dir("logs")?;
        self.ensure_dir("cache")?;
        self.ensure_dir("temp")?;

        let config_path = self.config_path();
        if !config_path.exists() {
            self.write_default_config()?;
            log::info!("Created default config.ini");
        }

        let instance_path = self.instance_path();
        if !instance_path.exists() {
            self.write_default_instance()?;
            log::info!("Created default instance.ini");
        }

        self.update_run_info()?;
        Ok(())
    }

    fn ensure_dir(&self, name: &str) -> anyhow::Result<()> {
        let dir = self.base_dir.join(name);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
            log::info!("Created directory: {}", dir.display());
        }
        Ok(())
    }

    pub fn config_path(&self) -> PathBuf {
        self.base_dir.join("config.ini")
    }

    pub fn instance_path(&self) -> PathBuf {
        self.base_dir.join("instance.ini")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.base_dir.join("logs")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.base_dir.join("cache")
    }

    pub fn temp_dir(&self) -> PathBuf {
        self.base_dir.join("temp")
    }

    fn write_default_config(&self) -> anyhow::Result<()> {
        let content = resources::read_resource("defaults/config.ini")?;
        std::fs::write(self.config_path(), content)?;
        Ok(())
    }

    fn write_default_instance(&self) -> anyhow::Result<()> {
        let content = resources::read_resource("defaults/instance.ini")?;
        std::fs::write(self.instance_path(), content)?;
        Ok(())
    }

    fn update_run_info(&self) -> anyhow::Result<()> {
        let mut ini = self.read_instance()?;
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        ini.set("Instance", "last_run", &now);

        if ini.get("Instance", "first_run").unwrap_or_default().is_empty() {
            ini.set("Instance", "first_run", &now);
        }

        let count = ini.get("Instance", "run_count")
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0) + 1;
        ini.set("Instance", "run_count", &count.to_string());

        self.write_instance(&ini)?;
        Ok(())
    }

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
        std::fs::write(self.config_path(), config.to_string())?;
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
