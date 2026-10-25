//! class 常量池：只读游标解析，产出 Utf8 条目与 String 引用目标
//!
//! 只关心「哪些文本出现在常量池里」以及「它们各自占哪几个字节」，
//! 其余 tag 按 JVM 规范的固定长度跳过；LONG/DOUBLE 占两个槽位。

use std::collections::BTreeSet;

/// class 文件魔数
const MAGIC: [u8; 4] = [0xCA, 0xFE, 0xBA, 0xBE];

/// 常量池 tag 编号（JVMS §4.4）
mod tag {
    pub const UTF8: u8 = 1;
    pub const INTEGER: u8 = 3;
    pub const FLOAT: u8 = 4;
    pub const LONG: u8 = 5;
    pub const DOUBLE: u8 = 6;
    pub const CLASS: u8 = 7;
    pub const STRING: u8 = 8;
    pub const FIELD_REF: u8 = 9;
    pub const METHOD_REF: u8 = 10;
    pub const INTERFACE_METHOD_REF: u8 = 11;
    pub const NAME_AND_TYPE: u8 = 12;
    pub const METHOD_HANDLE: u8 = 15;
    pub const METHOD_TYPE: u8 = 16;
    pub const DYNAMIC: u8 = 17;
    pub const INVOKE_DYNAMIC: u8 = 18;
    pub const MODULE: u8 = 19;
    pub const PACKAGE: u8 = 20;
}

/// 一条 Utf8 常量：槽位序号、解码文本、整条常量的字节区间
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Utf8Entry {
    pub index: u16,
    pub text: String,
    /// (起始偏移, 结束偏移)，覆盖 tag + 长度 + 内容
    pub span: (usize, usize),
}

/// 常量池中与文本相关的视图
#[derive(Debug, Clone, Default)]
pub struct ConstantPool {
    utf8: Vec<Utf8Entry>,
    /// 被 String 常量指向的 Utf8 槽位
    string_targets: BTreeSet<u16>,
}

impl ConstantPool {
    pub fn utf8(&self) -> &[Utf8Entry] {
        &self.utf8
    }

    /// 运行时字符串：仅取被 String 常量引用的那些 Utf8
    pub fn strings(&self) -> Vec<String> {
        self.utf8
            .iter()
            .filter(|entry| self.string_targets.contains(&entry.index))
            .map(|entry| entry.text.clone())
            .collect()
    }
}

/// 带边界检查的顺序读取器，越界一律转成可读错误
struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn take(&mut self, length: usize, what: &str) -> Result<&'a [u8], String> {
        let end = self
            .position
            .checked_add(length)
            .ok_or_else(|| format!("class {what} offset overflow"))?;
        if end > self.bytes.len() {
            return Err(format!("class {what} is truncated"));
        }
        let slice = &self.bytes[self.position..end];
        self.position = end;
        Ok(slice)
    }

    fn skip(&mut self, length: usize, what: &str) -> Result<(), String> {
        self.take(length, what).map(|_| ())
    }

    fn u8(&mut self, what: &str) -> Result<u8, String> {
        Ok(self.take(1, what)?[0])
    }

    fn u16(&mut self, what: &str) -> Result<u16, String> {
        let raw = self.take(2, what)?;
        Ok(u16::from_be_bytes([raw[0], raw[1]]))
    }
}

/// 解析常量池
pub(super) fn read_pool(bytes: &[u8]) -> Result<ConstantPool, String> {
    if bytes.len() < 10 {
        return Err("class file is shorter than the 10-byte header".to_string());
    }
    if bytes[0..4] != MAGIC {
        return Err("class file does not start with the 0xCAFEBABE magic".to_string());
    }
    let declared = u16::from_be_bytes([bytes[8], bytes[9]]);
    let mut cursor = Cursor::new(bytes);
    cursor.skip(10, "header")?;

    let mut pool = ConstantPool::default();
    let mut slot: u16 = 1;
    while slot < declared {
        let entry_start = cursor.position;
        let kind = cursor.u8("constant pool tag")?;
        match kind {
            tag::UTF8 => {
                let length = cursor.u16("UTF-8 constant length")? as usize;
                let raw = cursor.take(length, "UTF-8 constant body")?;
                pool.utf8.push(Utf8Entry {
                    index: slot,
                    text: String::from_utf8_lossy(raw).into_owned(),
                    span: (entry_start, cursor.position),
                });
            }
            tag::STRING => {
                let target = cursor.u16("String constant reference")?;
                pool.string_targets.insert(target);
            }
            tag::LONG | tag::DOUBLE => {
                cursor.skip(8, "8-byte constant")?;
                // 8 字节常量占用两个槽位，后一个槽位不可寻址
                slot += 1;
            }
            tag::INTEGER | tag::FLOAT => cursor.skip(4, "4-byte constant")?,
            tag::CLASS | tag::METHOD_TYPE | tag::MODULE | tag::PACKAGE => {
                cursor.skip(2, "2-byte constant")?;
            }
            tag::FIELD_REF
            | tag::METHOD_REF
            | tag::INTERFACE_METHOD_REF
            | tag::NAME_AND_TYPE
            | tag::DYNAMIC
            | tag::INVOKE_DYNAMIC => cursor.skip(4, "4-byte constant")?,
            tag::METHOD_HANDLE => cursor.skip(3, "method handle constant")?,
            other => return Err(format!("class constant pool tag {other} is not defined")),
        }
        slot += 1;
    }
    Ok(pool)
}

/// 常量池全部 Utf8 文本（含重复）及其字节区间
pub fn parse_class_constant_pool(bytes: &[u8]) -> Result<Vec<(String, usize, usize)>, String> {
    Ok(read_pool(bytes)?
        .utf8()
        .iter()
        .map(|entry| (entry.text.clone(), entry.span.0, entry.span.1))
        .collect())
}

/// 被 String 常量引用的文本（运行时可见字符串），无法解析时返回空
pub fn class_string_constants(bytes: &[u8]) -> Vec<String> {
    read_pool(bytes).map(|pool| pool.strings()).unwrap_or_default()
}
