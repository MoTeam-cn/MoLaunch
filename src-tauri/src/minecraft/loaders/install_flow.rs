//! 加载器安装的阶段划分
//!
//! Fabric / Forge / NeoForge / LiteLoader 共用同一套阶段，前端进度条与故障定位都从这里取。
//! 以前进度是散在各自安装函数里的 cb(20.0)、cb(80.0)，回头看日志根本不知道停在哪一步。

/// 安装阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStage {
    /// 校验目录、读取版本信息、挑选 Java
    Prepare,
    /// 下载加载器安装包
    FetchInstaller,
    /// 预取安装包声明的依赖库
    WarmLibraries,
    /// 执行安装器（或其内的 processor）
    RunInstaller,
    /// 合并版本 JSON、复制 jar
    MergeVersion,
    /// 核对安装产物，必要时补跑
    AuditArtifacts,
    /// 清理临时文件、收尾
    WrapUp,
}

impl InstallStage {
    /// 按执行顺序排列
    pub const ORDER: [InstallStage; 7] = [
        InstallStage::Prepare,
        InstallStage::FetchInstaller,
        InstallStage::WarmLibraries,
        InstallStage::RunInstaller,
        InstallStage::MergeVersion,
        InstallStage::AuditArtifacts,
        InstallStage::WrapUp,
    ];

    /// 阶段名（写进日志与诊断记录）
    pub fn label(self) -> &'static str {
        match self {
            InstallStage::Prepare => "准备",
            InstallStage::FetchInstaller => "获取安装包",
            InstallStage::WarmLibraries => "预取依赖库",
            InstallStage::RunInstaller => "执行安装器",
            InstallStage::MergeVersion => "合并版本",
            InstallStage::AuditArtifacts => "核对产物",
            InstallStage::WrapUp => "收尾",
        }
    }

    /// 该阶段在总进度中占的区间（起，止），单位 0.0~1.0
    pub fn span(self) -> (f64, f64) {
        match self {
            InstallStage::Prepare => (0.0, 5.0),
            InstallStage::FetchInstaller => (5.0, 25.0),
            InstallStage::WarmLibraries => (25.0, 45.0),
            InstallStage::RunInstaller => (45.0, 75.0),
            InstallStage::MergeVersion => (75.0, 82.0),
            InstallStage::AuditArtifacts => (82.0, 96.0),
            InstallStage::WrapUp => (96.0, 100.0),
        }
    }

    /// 把阶段内部的 0.0~1.0 折算成整体百分比
    pub fn at(self, fraction: f64) -> f64 {
        let (start, end) = self.span();
        start + (end - start) * fraction.clamp(0.0, 1.0)
    }
}

#[cfg(test)]
#[path = "install_flow_test.rs"]
mod install_flow_test;
