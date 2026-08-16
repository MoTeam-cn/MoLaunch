//! Scaffolding 联机协议：房间码、EasyTier 子进程、联机中心 TCP 与房客发现。

pub mod client;
pub mod code;
pub mod easytier;
pub mod server;

#[cfg(test)]
mod easytier_test;
