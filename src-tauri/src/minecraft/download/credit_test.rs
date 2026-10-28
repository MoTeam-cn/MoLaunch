//! 进度记账测试：差额增减、幂等、退回

use super::ProgressCredit;
use crate::minecraft::download::types::GlobalProgress;
use std::sync::{Arc, Mutex as StdMutex};

fn setup(
    initial: u64,
) -> (
    Option<Arc<StdMutex<GlobalProgress>>>,
    Arc<StdMutex<GlobalProgress>>,
) {
    let progress = Arc::new(StdMutex::new(GlobalProgress {
        downloaded_bytes: initial,
        ..Default::default()
    }));
    (Some(progress.clone()), progress)
}

fn value(progress: &Arc<StdMutex<GlobalProgress>>) -> u64 {
    progress
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .downloaded_bytes
}

#[test]
fn test_credit_按差额增减() {
    let (slot, progress) = setup(1000);
    let credit = ProgressCredit::new(&slot);
    credit.credit(300);
    assert_eq!(value(&progress), 1300);
    credit.credit(120);
    assert_eq!(value(&progress), 1120, "作废重下时要退回去");
    credit.credit(300);
    assert_eq!(value(&progress), 1300);
}

#[test]
fn test_credit_幂等() {
    let (slot, progress) = setup(0);
    let credit = ProgressCredit::new(&slot);
    credit.credit(500);
    credit.credit(500);
    credit.credit(500);
    assert_eq!(value(&progress), 500, "重复设置同一个值不应累加");
}

#[test]
fn test_rollback_清零但不影响别的文件() {
    let (slot, progress) = setup(200);
    let credit = ProgressCredit::new(&slot);
    credit.credit(800);
    assert_eq!(value(&progress), 1000);
    credit.rollback();
    assert_eq!(value(&progress), 200, "只退回本文件贡献的部分");
}

#[test]
fn test_没有进度通道时不崩溃() {
    let none: Option<Arc<StdMutex<GlobalProgress>>> = None;
    let credit = ProgressCredit::new(&none);
    credit.credit(100);
    credit.rollback();
}
