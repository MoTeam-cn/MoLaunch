//! 哈希模块测试：算法识别、四种算法的公开向量、区间哈希、验证证明缓存

use super::{
    forget, hash_file, is_verified, remember, sha256_range, HashAlgorithm, IncrementalHash,
};
use std::io::Write;
use std::path::PathBuf;

fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("molaunch-hash-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    let path = dir.join(name);
    let mut file = std::fs::File::create(&path).expect("创建临时文件");
    file.write_all(bytes).expect("写入临时文件");
    file.flush().expect("刷新");
    path
}

#[test]
fn test_from_expected_按长度识别四种算法() {
    assert_eq!(
        HashAlgorithm::from_expected(&"a".repeat(32)),
        Some(HashAlgorithm::Md5)
    );
    assert_eq!(
        HashAlgorithm::from_expected(&"a".repeat(40)),
        Some(HashAlgorithm::Sha1)
    );
    assert_eq!(
        HashAlgorithm::from_expected(&"a".repeat(64)),
        Some(HashAlgorithm::Sha256)
    );
    assert_eq!(
        HashAlgorithm::from_expected(&"a".repeat(128)),
        Some(HashAlgorithm::Sha512)
    );
}

#[test]
fn test_from_expected_支持前缀写法与空白() {
    let value = format!("sha1:{}", "b".repeat(40));
    assert_eq!(
        HashAlgorithm::from_expected(&value),
        Some(HashAlgorithm::Sha1)
    );
    assert_eq!(
        HashAlgorithm::from_expected(&format!("  {}  ", "c".repeat(64))),
        Some(HashAlgorithm::Sha256)
    );
}

#[test]
fn test_from_expected_长度不认识返回None() {
    // 关键回归：sha512 曾经被当成 sha1 去比，导致永远不匹配
    assert_eq!(HashAlgorithm::from_expected("abc"), None);
    assert_eq!(HashAlgorithm::from_expected(""), None);
    assert_eq!(HashAlgorithm::from_expected(&"a".repeat(41)), None);
}

#[test]
fn test_hash_file_四种算法对公开向量() {
    let path = temp_file("abc.bin", b"abc");
    assert_eq!(
        hash_file(&path, HashAlgorithm::Md5).expect("md5"),
        "900150983cd24fb0d6963f7d28e17f72"
    );
    assert_eq!(
        hash_file(&path, HashAlgorithm::Sha1).expect("sha1"),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
    assert_eq!(
        hash_file(&path, HashAlgorithm::Sha256).expect("sha256"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        hash_file(&path, HashAlgorithm::Sha512).expect("sha512"),
        "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
         2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_hash_file_超过缓冲大小的文件() {
    // 200 KiB > 64 KiB 缓冲，覆盖多轮读取
    let bytes: Vec<u8> = (0..200 * 1024).map(|i| (i % 251) as u8).collect();
    let path = temp_file("big.bin", &bytes);
    let mut hasher = IncrementalHash::new(HashAlgorithm::Sha256);
    hasher.update(&bytes);
    assert_eq!(
        hash_file(&path, HashAlgorithm::Sha256).expect("sha256"),
        hasher.finish_hex()
    );
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_sha256_range_与整文件一致并可取子区间() {
    let bytes: Vec<u8> = (0..4096).map(|i| (i % 97) as u8).collect();
    let path = temp_file("range.bin", &bytes);
    let whole = hash_file(&path, HashAlgorithm::Sha256).expect("整文件");
    let full_range = sha256_range(&path, 0, bytes.len() as u64).expect("整段");
    assert_eq!(hex::encode(full_range), whole);

    let slice = sha256_range(&path, 100, 200).expect("子区间");
    let mut hasher = IncrementalHash::new(HashAlgorithm::Sha256);
    hasher.update(&bytes[100..300]);
    assert_eq!(hex::encode(slice), hasher.finish_hex());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_sha256_range_文件变短时报错() {
    let path = temp_file("short.bin", &[1_u8; 16]);
    assert!(sha256_range(&path, 0, 1024).is_err());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_验证证明_命中与失效() {
    let path = temp_file("proof.bin", b"payload");
    let expected = "a".repeat(40);
    assert!(!is_verified(&path, &expected));
    remember(&path, &expected);
    assert!(is_verified(&path, &expected));
    // 期望值不同不命中
    assert!(!is_verified(&path, "b".repeat(40).as_str()));
    // 文件内容/大小变了不命中
    std::fs::write(&path, b"payload-changed-longer").expect("改写");
    assert!(!is_verified(&path, &expected));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn test_验证证明_忘记后不再命中() {
    let path = temp_file("forget.bin", b"payload");
    let expected = "c".repeat(64);
    remember(&path, &expected);
    assert!(is_verified(&path, &expected));
    forget(&path);
    assert!(!is_verified(&path, &expected));
    let _ = std::fs::remove_file(&path);
}
