//! 下载阶段与下载状态

use serde::{Deserialize, Serialize};

/// 阶段状态
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum StageStatus {
    #[default]
    Waiting,
    Loading,
    Finished,
    Failed,
}

/// 下载阶段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadStage {
    pub name: String,
    pub progress: f64,
    pub weight: f64,
    pub status: StageStatus,
    pub bytes_downloaded: u64,
    pub bytes_total: u64,
    pub files_downloaded: usize,
    pub files_total: usize,
    /// 所属任务分组（用于前端按"整合包安装"/"MC本体安装"等分组折叠展开）
    /// None 表示独立阶段（不分组），Some 表示归属于某分组
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

impl DownloadStage {
    pub fn new(name: impl Into<String>, weight: f64) -> Self {
        Self {
            name: name.into(),
            progress: 0.0,
            weight,
            status: StageStatus::Waiting,
            bytes_downloaded: 0,
            bytes_total: 0,
            files_downloaded: 0,
            files_total: 0,
            group: None,
        }
    }

    /// 创建带分组的 stage
    pub fn new_grouped(name: impl Into<String>, weight: f64, group: impl Into<String>) -> Self {
        let mut s = Self::new(name, weight);
        s.group = Some(group.into());
        s
    }
}

/// 下载状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DownloadState {
    pub is_active: bool,
    pub is_complete: bool,
    pub stages: Vec<DownloadStage>,
    pub current_stage_index: usize,
    pub global_speed: u64,
    pub global_bytes_downloaded: u64,
    pub global_bytes_total: u64,
    pub error_code: i32,
    /// 当前下载的版本名（用于前端显示，刷新页面后恢复）
    #[serde(default)]
    pub version_name: String,
}

impl DownloadState {
    /// 重置为指定 stages（清空原有，用于独立安装流程）
    pub fn reset_stages(&mut self, stages: Vec<DownloadStage>) {
        self.stages = stages;
        self.current_stage_index = 0;
        self.is_active = true;
        self.is_complete = false;
        self.global_speed = 0;
        self.global_bytes_downloaded = 0;
        self.global_bytes_total = 0;
        self.error_code = 0;
    }

    /// 追加 stages（保留原有，用于连续安装流程：整合包 → MC 本体）
    /// 返回追加前的 stages 长度，作为后续 stage_callback 的偏移量
    pub fn append_stages(&mut self, stages: Vec<DownloadStage>) -> usize {
        let offset = self.stages.len();
        self.stages.extend(stages);
        self.is_active = true;
        self.is_complete = false;
        offset
    }

    /// 设置当前阶段索引（stage_callback 调用）
    /// 自动把前一阶段标记为 Finished（仅当 idx > prev 时）
    pub fn set_current_stage(&mut self, idx: usize) {
        if idx > self.current_stage_index && self.current_stage_index < self.stages.len() {
            self.stages[self.current_stage_index].status = StageStatus::Finished;
            self.stages[self.current_stage_index].progress = 1.0;
        }
        self.current_stage_index = idx;
        if idx < self.stages.len() {
            self.stages[idx].status = StageStatus::Loading;
            self.stages[idx].progress = 0.0;
            self.stages[idx].bytes_downloaded = 0;
            self.stages[idx].bytes_total = 0;
        }
    }

    /// 设置指定阶段的状态和进度（本地操作用：解析 zip、复制 overrides 等）
    pub fn set_stage_status(&mut self, idx: usize, status: StageStatus, progress: f64) {
        self.current_stage_index = idx;
        if idx < self.stages.len() {
            self.stages[idx].status = status;
            self.stages[idx].progress = progress;
        }
    }

    /// 设置指定阶段的字节进度（本地操作如解压 overrides）
    pub fn set_stage_bytes(&mut self, idx: usize, downloaded: u64, total: u64) {
        if idx < self.stages.len() {
            self.stages[idx].bytes_downloaded = downloaded;
            self.stages[idx].bytes_total = total;
            if total > 0 {
                self.stages[idx].progress = (downloaded as f64 / total as f64).min(1.0);
            }
        }
    }

    /// 同步 DownloadManager 的 GlobalProgress 到指定阶段 + 更新全局指标
    /// 这是核心统一方法：整合包/MC 本体/自定义下载都用这个
    /// 统一规则：
    ///   - stage 进度按 bytes 计算（total_bytes>0 时），否则按 files 计算
    ///   - global_bytes 累加所有 Finished + Loading 阶段（支持连续安装流程的进度连贯）
    ///   - global_speed 直接信任 DownloadManager 的 current_speed（它已有 300ms 滑动窗口）
    pub fn sync_stage_from_progress(
        &mut self,
        idx: usize,
        downloaded_bytes: u64,
        total_bytes: u64,
        completed_files: usize,
        total_files: usize,
        current_speed: u64,
    ) {
        if idx < self.stages.len() {
            let stage = &mut self.stages[idx];
            stage.bytes_downloaded = downloaded_bytes;
            stage.bytes_total = total_bytes;
            stage.files_downloaded = completed_files;
            stage.files_total = total_files;
            // 计算进度
            let progress = if total_bytes > 0 {
                (downloaded_bytes as f64 / total_bytes as f64).min(1.0)
            } else if total_files > 0 {
                (completed_files as f64 / total_files as f64).min(1.0)
            } else {
                0.0
            };
            stage.progress = progress;
            // 仅当未完成时标记为 Loading，避免完成后图标不更新
            // 修复：之前无条件设为 Loading，导致 progress=1.0 时前端仍显示加载中图标
            if progress >= 1.0 {
                stage.status = StageStatus::Finished;
            } else if stage.status != StageStatus::Finished {
                stage.status = StageStatus::Loading;
            }
        }

        // 统一 global_bytes 算法：累加所有 Finished + Loading 阶段
        let mut g_downloaded = 0u64;
        let mut g_total = 0u64;
        for stage in &self.stages {
            if stage.status == StageStatus::Finished || stage.status == StageStatus::Loading {
                g_downloaded += stage.bytes_downloaded;
                g_total += stage.bytes_total;
            }
        }
        self.global_bytes_downloaded = g_downloaded;
        self.global_bytes_total = g_total;
        self.global_speed = current_speed;
    }

    /// 标记整体完成（所有 Loading 阶段标记为 Finished）
    pub fn mark_complete(&mut self) {
        self.is_active = false;
        self.is_complete = true;
        for stage in &mut self.stages {
            if stage.status == StageStatus::Loading {
                stage.status = StageStatus::Finished;
                stage.progress = 1.0;
            }
        }
    }

    /// 标记整体失败
    pub fn mark_failed(&mut self, error_code: i32) {
        self.is_active = false;
        self.is_complete = false;
        self.error_code = error_code;
    }
}
