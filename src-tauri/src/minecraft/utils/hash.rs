//! 文件哈希：流式计算 + 算法识别 + 验证证明缓存
//!
//! 三件事以前都没做好：整文件 `read_to_end` 会把内存顶起来；算法只按「64 位当 sha256、
//! 其余当 sha1」猜，sha512 会被当 sha1 算成永远不匹配；验证过的文件每次检查都重算一遍，
//! 安装收尾的卡顿主要来自这里。

use std::collections::VecDeque;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

/// 读盘缓冲：内存占用恒定，不随文件大小增长
const BUFFER_BYTES: usize = 64 * 1024;

/// 验证证明缓存的条目上限（超出后淘汰最旧的）
const CACHE_CAPACITY: usize = 512;

/// 支持的哈希算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl HashAlgorithm {
    /// 按期望哈希的十六进制长度识别算法
    ///
    /// 接受 `sha1:abcdef...` 这类带前缀写法；长度不认识就返回 None，
    /// 调用方据此跳过哈希校验——**绝不能**把未知长度当成 sha1 去比，那会永远不匹配。
    pub fn from_expected(expected: &str) -> Option<Self> {
        let trimmed = expected.trim();
        let hex = trimmed.rsplit(':').next().unwrap_or(trimmed).trim();
        match hex.len() {
            32 => Some(Self::Md5),
            40 => Some(Self::Sha1),
            64 => Some(Self::Sha256),
            128 => Some(Self::Sha512),
            _ => None,
        }
    }
}

/// 流式计算整个文件的哈希（小写十六进制）
pub fn hash_file(path: &Path, algorithm: HashAlgorithm) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = IncrementalHash::new(algorithm);
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finish_hex())
}

/// 流式计算文件某一段的 SHA256（位图自校验用）
pub fn sha256_range(path: &Path, start: u64, len: u64) -> std::io::Result<[u8; 32]> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(start))?;
    let mut hasher = IncrementalHash::new(HashAlgorithm::Sha256);
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    let mut remaining = len;
    while remaining > 0 {
        let want = (remaining as usize).min(BUFFER_BYTES);
        let read = file.read(&mut buffer[..want])?;
        if read == 0 {
            // 文件比位图声明的短：这段等于没写全
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "数据文件短于位图声明的区间",
            ));
        }
        hasher.update(&buffer[..read]);
        remaining -= read as u64;
    }
    hasher.finish_sha256()
}

/// 增量哈希：下载时边写边算，不需要把整块留在内存里
pub struct IncrementalHash {
    algorithm: HashAlgorithm,
    md5: Option<md5::Context>,
    sha1: Option<sha1::Sha1>,
    sha256: Option<sha2::Sha256>,
    sha512: Option<sha2::Sha512>,
}

impl IncrementalHash {
    pub fn new(algorithm: HashAlgorithm) -> Self {
        use sha1::Digest as _;
        let mut this = Self {
            algorithm,
            md5: None,
            sha1: None,
            sha256: None,
            sha512: None,
        };
        match algorithm {
            HashAlgorithm::Md5 => this.md5 = Some(md5::Context::new()),
            HashAlgorithm::Sha1 => this.sha1 = Some(sha1::Sha1::new()),
            HashAlgorithm::Sha256 => this.sha256 = Some(sha2::Sha256::new()),
            HashAlgorithm::Sha512 => this.sha512 = Some(sha2::Sha512::new()),
        }
        this
    }

    pub fn update(&mut self, bytes: &[u8]) {
        use sha1::Digest as _;
        match self.algorithm {
            HashAlgorithm::Md5 => {
                if let Some(context) = self.md5.as_mut() {
                    context.consume(bytes);
                }
            }
            HashAlgorithm::Sha1 => {
                if let Some(hasher) = self.sha1.as_mut() {
                    hasher.update(bytes);
                }
            }
            HashAlgorithm::Sha256 => {
                if let Some(hasher) = self.sha256.as_mut() {
                    hasher.update(bytes);
                }
            }
            HashAlgorithm::Sha512 => {
                if let Some(hasher) = self.sha512.as_mut() {
                    hasher.update(bytes);
                }
            }
        }
    }

    fn finish_bytes(self) -> Vec<u8> {
        use sha1::Digest as _;
        match self.algorithm {
            HashAlgorithm::Md5 => self
                .md5
                .map(|context| context.compute().to_vec())
                .unwrap_or_default(),
            HashAlgorithm::Sha1 => self
                .sha1
                .map(|hasher| hasher.finalize().to_vec())
                .unwrap_or_default(),
            HashAlgorithm::Sha256 => self
                .sha256
                .map(|hasher| hasher.finalize().to_vec())
                .unwrap_or_default(),
            HashAlgorithm::Sha512 => self
                .sha512
                .map(|hasher| hasher.finalize().to_vec())
                .unwrap_or_default(),
        }
    }

    /// 收尾，返回小写十六进制
    pub fn finish_hex(self) -> String {
        hex::encode(self.finish_bytes())
    }

    /// 收尾，返回 SHA256 原始摘要（只对 `Sha256` 有效）
    pub fn finish_sha256(self) -> std::io::Result<[u8; 32]> {
        let bytes = self.finish_bytes();
        <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "摘要长度不是 32 字节")
        })
    }
}

/// 一个已通过的验证证明
///
/// 只记元数据不记账内容：路径 + 大小 + 修改/创建时间不变，就认为还是那个已验证的文件。
#[derive(Clone, Debug, PartialEq, Eq)]
struct Proof {
    path: PathBuf,
    size: u64,
    modified: Option<SystemTime>,
    created: Option<SystemTime>,
    expected: String,
}

fn cache() -> &'static Mutex<VecDeque<Proof>> {
    static CACHE: OnceLock<Mutex<VecDeque<Proof>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(VecDeque::new()))
}

/// 取文件的身份快照：大小 + 修改时间 + 创建时间
fn snapshot(path: &Path) -> Option<(u64, Option<SystemTime>, Option<SystemTime>)> {
    let metadata = std::fs::metadata(path).ok()?;
    if !metadata.is_file() {
        return None;
    }
    Some((
        metadata.len(),
        metadata.modified().ok(),
        metadata.created().ok(),
    ))
}

/// 记录「这个文件用这个期望值验证通过过」
pub fn remember(path: &Path, expected: &str) {
    let Some((size, modified, created)) = snapshot(path) else {
        return;
    };
    let proof = Proof {
        path: path.to_path_buf(),
        size,
        modified,
        created,
        expected: expected.to_ascii_lowercase(),
    };
    let mut cache = cache().lock().unwrap_or_else(|p| p.into_inner());
    cache.retain(|item| item.path != proof.path || item.expected != proof.expected);
    cache.push_back(proof);
    while cache.len() > CACHE_CAPACITY {
        cache.pop_front();
    }
}

/// 是否已有验证证明
///
/// 每次都重新读一次文件元数据：同路径被换成别的文件（时间/大小变了）就不会命中。
pub fn is_verified(path: &Path, expected: &str) -> bool {
    let expected = expected.to_ascii_lowercase();
    let Some((size, modified, created)) = snapshot(path) else {
        return false;
    };
    let cache = cache().lock().unwrap_or_else(|p| p.into_inner());
    cache.iter().any(|item| {
        item.path == path
            && item.size == size
            && item.modified == modified
            && item.created == created
            && item.expected == expected
    })
}

/// 丢掉某个文件的验证证明（重下、或校验失败时调用）
pub fn forget(path: &Path) {
    let mut cache = cache().lock().unwrap_or_else(|p| p.into_inner());
    cache.retain(|item| item.path != path);
}

#[cfg(test)]
#[path = "hash_test.rs"]
mod hash_test;
