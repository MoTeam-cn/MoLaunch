//! 安装过程记录
//!
//! 装失败时用户手里往往只有一张截图，日志里散着几条 log_error 拼不出全貌。
//! 这里把阶段推进和关键动作按顺序攒起来，结束时落成一份可读文本；
//! 出错时把文件路径带进错误消息，用户直接发过来就能定位。

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::install_flow::InstallStage;

/// 条目上限，防止异常循环把内存吃光
const MAX_ENTRIES: usize = 400;

/// 一条记录
struct Entry {
    elapsed: Duration,
    stage: Option<InstallStage>,
    text: String,
}

/// 一次安装的记录器
pub struct InstallLog {
    loader: &'static str,
    version_id: String,
    mc_version: String,
    java: String,
    game_dir: PathBuf,
    started: Instant,
    entries: Vec<Entry>,
    dropped: usize,
}

impl InstallLog {
    /// 开始记录
    pub fn begin(
        loader: &'static str,
        version_id: &str,
        mc_version: &str,
        java: &str,
        game_dir: &Path,
    ) -> Self {
        Self {
            loader,
            version_id: version_id.to_string(),
            mc_version: mc_version.to_string(),
            java: java.to_string(),
            game_dir: game_dir.to_path_buf(),
            started: Instant::now(),
            entries: Vec::new(),
            dropped: 0,
        }
    }

    /// 记录阶段推进
    pub fn stage(&mut self, stage: InstallStage) {
        self.push(Some(stage), format!("进入「{}」", stage.label()));
    }

    /// 记录一次动作
    pub fn note(&mut self, text: impl Into<String>) {
        self.push(None, text.into());
    }

    /// 记录一个问题（不中断流程）
    pub fn problem(&mut self, text: impl Into<String>) {
        self.push(None, format!("[问题] {}", text.into()));
    }

    fn push(&mut self, stage: Option<InstallStage>, text: String) {
        if self.entries.len() >= MAX_ENTRIES {
            self.dropped += 1;
            return;
        }
        self.entries.push(Entry {
            elapsed: self.started.elapsed(),
            stage,
            text,
        });
    }

    /// 记录耗时
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    /// 结束并落盘，返回文件路径
    ///
    /// 落盘失败不算安装失败——记录丢了不影响游戏能不能玩。
    pub fn finish(&self, succeeded: bool, headline: &str) -> Option<PathBuf> {
        let dir = self.game_dir.join("cache").join("install_logs");
        if let Err(e) = std::fs::create_dir_all(&dir) {
            crate::log_warn!("[InstallLog] 建不了记录目录（{}）：{}", dir.display(), e);
            return None;
        }
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = dir.join(format!("{}-{}.log", sanitize(&self.version_id), stamp));

        let mut body = String::new();
        body.push_str("MoLaunch 加载器安装记录\n");
        body.push_str(&format!("加载器：{}\n", self.loader));
        body.push_str(&format!("目标版本：{}\n", self.version_id));
        body.push_str(&format!("Minecraft：{}\n", self.mc_version));
        body.push_str(&format!("Java：{}\n", self.java));
        body.push_str(&format!("游戏目录：{}\n", self.game_dir.display()));
        body.push_str(&format!(
            "总耗时：{:.2}s\n",
            self.started.elapsed().as_secs_f64()
        ));
        body.push_str(&format!(
            "结果：{}\n",
            if succeeded { "成功" } else { "失败" }
        ));
        body.push_str(&format!("摘要：{}\n", headline));
        body.push_str("\n—— 过程 ——\n");
        for entry in &self.entries {
            let tag = entry
                .stage
                .map(|s| format!("{:<6}", s.label()))
                .unwrap_or_else(|| "      ".to_string());
            body.push_str(&format!(
                "[{:>7.2}s] {} {}\n",
                entry.elapsed.as_secs_f64(),
                tag,
                entry.text
            ));
        }
        if self.dropped > 0 {
            body.push_str(&format!(
                "\n（另有 {} 条记录因超出上限被省略）\n",
                self.dropped
            ));
        }

        match std::fs::write(&path, body) {
            Ok(()) => Some(path),
            Err(e) => {
                crate::log_warn!("[InstallLog] 写不了记录文件（{}）：{}", path.display(), e);
                None
            }
        }
    }
}

/// 文件名只保留安全字符
fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "install_log_test.rs"]
mod install_log_test;
