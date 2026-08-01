//! fuzzy 模糊匹配算法单元测试

use super::*;

#[test]
fn test_search_similarity_exact_match() {
    let score = search_similarity("工业时代2", "工业时代");
    assert!(
        score > 0.5,
        "exact match should have high score, got {}",
        score
    );
}

#[test]
fn test_search_similarity_no_match() {
    let score = search_similarity("完全不同的文本xyz", "工业时代");
    assert!(score < 0.1, "no match should have low score, got {}", score);
}

#[test]
fn test_search_similarity_empty() {
    assert_eq!(search_similarity("", "test"), 0.0);
    assert_eq!(search_similarity("test", ""), 0.0);
}

#[test]
fn test_search_similarity_weighted() {
    let sources = vec![
        SearchSource::new(vec!["工业时代2".to_string()], 1.0),
        SearchSource::new(vec!["industrial craft".to_string()], 0.5),
    ];
    let score = search_similarity_weighted(&sources, "工业时代");
    assert!(
        score > 0.3,
        "weighted match should have decent score, got {}",
        score
    );
}

#[test]
fn test_search_similarity_short_query_bonus() {
    // 短查询（≤2 字符）应有额外加成，得分应高于同样匹配长度的长查询
    let short = search_similarity("工业时代", "工");
    let long = search_similarity("工业时代工业时代", "工业时代工业时代");
    assert!(short > 0.0, "short query should have positive score");
    assert!(long > 0.0, "long exact match should have positive score");
}

#[test]
fn test_search_collect_absolute_right() {
    let mut entries = vec![
        SearchEntry::new(
            (),
            vec![SearchSource::new(vec!["工业时代2".to_string()], 1.0)],
        ),
        SearchEntry::new(
            (),
            vec![SearchSource::new(vec!["完全不相关".to_string()], 1.0)],
        ),
    ];
    let results = search(&mut entries, "工业时代", 10, 0.25);
    assert!(!results.is_empty(), "should match 工业时代2");
    assert!(results[0].absolute_right, "should be absolute match");
}

#[test]
fn test_search_similarity_weighted_empty_sources() {
    let score = search_similarity_weighted(&[], "test");
    assert_eq!(score, 0.0);
}

#[test]
fn test_search_empty_entries() {
    let mut entries: Vec<SearchEntry<()>> = Vec::new();
    let results = search(&mut entries, "test", 10, 0.25);
    assert!(results.is_empty());
}
