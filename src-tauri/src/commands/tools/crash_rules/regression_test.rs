//! 回归测试：用真实日志（docs/Error/test.txt，403 行）逐条对照验收标准
//!
//! 断言顺序与验收清单一致：零误报 → 归因 → 证据 → 因果链 → 去重 → 覆盖 → 置信度。

use crate::commands::tools::crash_analyzer::analyze_log_text_full;
use crate::commands::tools::types::CrashAnalysisItem;

const LOG: &str = include_str!("testdata/regression.txt");

fn result() -> crate::commands::tools::types::CrashAnalyzeResult {
    analyze_log_text_full(LOG, None)
}

fn items() -> Vec<CrashAnalysisItem> {
    result().analyses
}

fn find<'a>(items: &'a [CrashAnalysisItem], cause: &str) -> Option<&'a CrashAnalysisItem> {
    items.iter().find(|item| item.cause == cause)
}

fn causes(items: &[CrashAnalysisItem]) -> Vec<&str> {
    items.iter().map(|item| item.cause.as_str()).collect()
}

#[test]
fn test_回归_日志行数被正确统计() {
    let result = result();
    assert!(
        result.line_count >= 400,
        "回归日志应有 400+ 行，实际 {}",
        result.line_count
    );
}

#[test]
fn test_回归_零误报_不出现_fabric_重复安装_输出极短() {
    let items = items();
    let banned = ["fabric_api_missing", "duplicate_mod", "output_too_short"];
    for cause in banned {
        assert!(
            find(&items, cause).is_none(),
            "不应出现误报结论 {}，实际结论: {:?}",
            cause,
            causes(&items)
        );
    }
    // 也不应出现「Fabric 加载失败 / Mod 重复安装 / 程序输出极短」这类标题
    let banned_titles = ["Fabric 加载失败", "Mod 重复安装", "程序输出极短"];
    for title in banned_titles {
        assert!(
            !items.iter().any(|item| item.title.contains(title)),
            "不应出现标题「{}」",
            title
        );
    }
}

#[test]
fn test_回归_缺少前置_mod_证据指向依赖声明行() {
    let items = items();
    let hit = find(&items, "missing_dependency").expect("应识别 create 缺前置");
    assert_eq!(
        hit.evidence[0].line_no, 6,
        "证据应指向依赖声明行（第 6 行）"
    );
    assert!(
        hit.evidence[0]
            .raw_line
            .contains("missing dependency 'flywheel'"),
        "证据原文应包含依赖声明，实际: {}",
        hit.evidence[0].raw_line
    );
    assert!(hit.title.contains("flywheel"), "标题应点出缺少的前置");
}

#[test]
fn test_回归_mixin_报错归因为版本不匹配而非_mod_冲突() {
    let items = items();
    let hit = find(&items, "version_mismatch").expect("JEI 的 Mixin 报错应归因为版本不匹配");
    assert!(hit.title.contains("版本不匹配"));
    assert!(
        hit.evidence
            .iter()
            .any(|e| e.raw_line.contains("MixinApplyError")),
        "证据应包含 MixinApplyError 行"
    );
    assert!(
        hit.evidence
            .iter()
            .any(|e| e.raw_line.contains("NoSuchMethodError")),
        "证据应包含 NoSuchMethodError 行"
    );
    assert!(
        find(&items, "mixin_conflict").is_none(),
        "没有双来源改写证据时不得判为 Mod 冲突"
    );
}

#[test]
fn test_回归_每条结论都有日志原文证据() {
    let items = items();
    assert!(!items.is_empty());
    for item in &items {
        assert!(
            !item.evidence.is_empty(),
            "结论「{}」缺少证据行",
            item.title
        );
        for evidence in &item.evidence {
            assert!(evidence.line_no > 0, "证据行号必须从 1 开始");
            assert!(!evidence.raw_line.trim().is_empty(), "证据原文不能为空");
            assert!(
                LOG.contains(&evidence.raw_line),
                "证据原文必须是日志子串，实际: {}",
                evidence.raw_line
            );
        }
    }
}

#[test]
fn test_回归_磁盘满因果链合并为一条() {
    let items = items();
    let chains: Vec<&CrashAnalysisItem> = items
        .iter()
        .filter(|item| item.cause == "disk_full")
        .collect();
    assert_eq!(chains.len(), 1, "磁盘满因果链必须只有一条");
    let chain = chains[0];
    assert!(chain.title.contains("磁盘空间耗尽"));
    assert!(chain.title.contains("写入失败"));
    assert!(chain.title.contains("看门狗"));
    assert!(chain.title.contains("崩溃报告"));
    assert_eq!(
        chain.title.matches(" → ").count(),
        3,
        "四个环节应串成一条链，实际标题: {}",
        chain.title
    );
    let line_nos: Vec<usize> = chain.evidence.iter().map(|e| e.line_no).collect();
    assert!(
        line_nos.contains(&51),
        "应引用磁盘满行，实际: {:?}",
        line_nos
    );
    assert!(
        line_nos.contains(&50),
        "应引用写入失败行，实际: {:?}",
        line_nos
    );
    assert!(
        line_nos.contains(&342),
        "应引用看门狗行，实际: {:?}",
        line_nos
    );
    assert!(
        line_nos.contains(&104),
        "应引用崩溃报告行，实际: {:?}",
        line_nos
    );
}

#[test]
fn test_回归_java_版本类结论只有一条() {
    let items = items();
    let java_items: Vec<&CrashAnalysisItem> = items
        .iter()
        .filter(|item| item.category == "java_version")
        .collect();
    assert_eq!(java_items.len(), 1, "Java 版本类结论应合并为一条");
    assert!(
        java_items[0]
            .evidence
            .iter()
            .any(|e| e.raw_line.contains("UnsupportedClassVersionError")),
        "证据应包含 UnsupportedClassVersionError 行"
    );
}

#[test]
fn test_回归_覆盖全部应识别根因() {
    let items = items();
    let causes = causes(&items);
    let expected = [
        ("world_region_corrupt", "区块 / 区域文件损坏"),
        ("player_data_corrupt", "玩家数据损坏"),
        ("memory_heap", "内存不足（堆）"),
        ("memory_metaspace", "内存不足（Metaspace）"),
        ("memory_direct_buffer", "内存不足（直接缓冲区）"),
        ("server_tps_stall", "TPS 掉底 / 卡死"),
        ("java_version", "Java 版本过低"),
        ("mysql_unreachable", "数据库不可达"),
        ("plugin_missing_class", "插件缺类"),
        ("plugin_java_version", "插件需要更高 Java"),
        ("plugin_class_missing", "插件类文件缺失"),
    ];
    for (cause, label) in expected {
        assert!(
            causes.contains(&cause),
            "应识别「{}」（{}），实际结论: {:?}",
            label,
            cause,
            causes
        );
    }
}

#[test]
fn test_回归_每条结论都带置信度且低置信必须说明缺什么证据() {
    let result = result();
    let all: Vec<&CrashAnalysisItem> = result
        .analyses
        .iter()
        .chain(result.pending.iter())
        .collect();
    for item in all {
        assert!(
            ["high", "medium", "low"].contains(&item.confidence.as_str()),
            "置信度取值非法: {}（结论：{}）",
            item.confidence,
            item.title
        );
        if item.confidence == "low" {
            assert!(
                !item.missing_evidence.is_empty(),
                "低置信结论「{}」必须说明还缺什么证据",
                item.title
            );
        }
    }
    for item in &result.pending {
        assert!(
            item.evidence.is_empty(),
            "待确认条目不应带证据行：{}",
            item.title
        );
    }
}
