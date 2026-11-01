use super::*;

#[test]
fn test_阶段区间首尾连续() {
    for pair in InstallStage::ORDER.windows(2) {
        let (_, prev_end) = pair[0].span();
        let (next_start, _) = pair[1].span();
        assert!(
            (prev_end - next_start).abs() < f64::EPSILON,
            "{:?} 与 {:?} 的区间不连续",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn test_阶段区间从零到一百() {
    let (first_start, _) = InstallStage::ORDER[0].span();
    let (_, last_end) = InstallStage::ORDER[InstallStage::ORDER.len() - 1].span();
    assert_eq!(first_start, 0.0);
    assert_eq!(last_end, 100.0);
}

#[test]
fn test_阶段内进度落在区间内() {
    let stage = InstallStage::RunInstaller;
    let (start, end) = stage.span();
    assert_eq!(stage.at(0.0), start);
    assert_eq!(stage.at(1.0), end);
    let mid = stage.at(0.5);
    assert!(mid > start && mid < end);
}

#[test]
fn test_阶段内进度越界被夹住() {
    let stage = InstallStage::FetchInstaller;
    let (start, end) = stage.span();
    assert_eq!(stage.at(-3.0), start);
    assert_eq!(stage.at(9.0), end);
}

#[test]
fn test_每个阶段都有非空名称() {
    for stage in InstallStage::ORDER {
        assert!(!stage.label().is_empty());
    }
}
