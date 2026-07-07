//! Storage module - manages .Molaunch folder
//! All operations on .Molaunch folder must go through this module
//! Uses INI format for configuration

pub mod ini;

use crate::log_info;
use crate::resources;
use std::path::PathBuf;
use std::sync::OnceLock;

static STORAGE: OnceLock<Storage> = OnceLock::new();

pub struct Storage {
    base_dir: PathBuf,
}

impl Storage {
    pub fn instance() -> &'static Storage {
        STORAGE.get_or_init(|| {
            let base_dir = Self::resolve_base_dir();
            Storage { base_dir }
        })
    }

    fn resolve_base_dir() -> PathBuf {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join(".Molaunch");
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            return cwd.join(".Molaunch");
        }
        PathBuf::from(".Molaunch")
    }

    pub fn base_dir(&self) -> &PathBuf {
        &self.base_dir
    }

    pub fn init(&self) -> anyhow::Result<()> {
        if !self.base_dir.exists() {
            std::fs::create_dir_all(&self.base_dir)?;
            log_info!("Created storage directory: {}", self.base_dir.display());
        }

        self.ensure_dir("logs")?;
        self.ensure_dir("cache")?;
        self.ensure_dir("temp")?;

        self.sync_config()?;

        let instance_path = self.instance_path();
        if !instance_path.exists() {
            self.write_default_instance()?;
            log_info!("Created default instance.ini");
        }

        self.update_run_info()?;
        Ok(())
    }

    fn ensure_dir(&self, name: &str) -> anyhow::Result<()> {
        let dir = self.base_dir.join(name);
        if !dir.exists() {
            std::fs::create_dir_all(&dir)?;
            log_info!("Created directory: {}", dir.display());
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

    fn sync_config(&self) -> anyhow::Result<()> {
        let config_path = self.config_path();

        if !config_path.exists() {
            self.write_default_config()?;
            log_info!("Created default config.ini");
            return Ok(());
        }

        let template_content = resources::read_resource("defaults/config.ini")?;
        let template = ini::IniFile::parse(&template_content);
        let mut current = self.read_config()?;

        let mut modified = false;
        for section in template.sections() {
            let template_pairs = template.get_section(&section);
            for (key, value) in &template_pairs {
                if !current.has_key(&section, key) {
                    log_info!("Config sync: [{}] {} = {}", section, key, value);
                    current.set(&section, key, value);
                    modified = true;
                }
            }
        }

        if modified {
            self.write_config(&current)?;
            log_info!("Config synced with template");
        }

        Ok(())
    }

    fn write_default_config(&self) -> anyhow::Result<()> {
        let content = resources::read_resource("defaults/config.ini")?;
        std::fs::write(self.config_path(), content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(self.config_path()) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(self.config_path(), perms);
            }
        }
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
        std::fs::write(self.config_path(), config.to_string())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(self.config_path()) {
                let mut perms = metadata.permissions();
                perms.set_mode(0o600);
                let _ = std::fs::set_permissions(self.config_path(), perms);
            }
        }
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
