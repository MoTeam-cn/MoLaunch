//! 会话压缩管线
mod l1;
mod l3;
mod pipeline;
mod rebuild;
mod trigger;

#[cfg(test)]
mod l1_test;

#[cfg(test)]
mod rebuild_test;

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod trigger_test;

pub use pipeline::compact_if_needed;
pub(super) use trigger::clear_cooldown;
