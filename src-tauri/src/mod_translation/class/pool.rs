//! class 常量池解析（规格 §3.1 ~ §3.2，格式依据 JVMS 第 4.4 节）
//!
//! 只关心两类条目：UTF8（tag 1）与其被 String（tag 8）引用的文本。
//! 解析用「顺序游标」推进：每读一个 tag 就按该 tag 的固定长度跳过或取内容，
//! LONG/DOUBLE 额外占一个槽位，因此槽位号与真实索引必须分开维护。

/// 常量池 tag（JVMS 4.4）
pub mod tag {
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

const CLASS_MAGIC: u32 = 0xCAFE_BABE;

/// 一个 UTF8 常量：槽位号、文本、字节区间（tag 到内容结束）
#[derive(Debug, Clone, PartialEq)]
pub struct Utf8Entry {
    pub index: u16,
    pub text: String,
    pub span: (usize, usize),
}

/// 解析结果：UTF8 条目 + 被 String 引用的槽位号
#[derive(Debug, Clone, Default)]
pub struct ConstantPool {
    utf8: Vec<Utf8Entry>,
    string_slots: Vec<u16>,
}

impl ConstantPool {
    pub fn utf8(&self) -> &[Utf8Entry] {
        &self.utf8
    }

    /// 被 String 常量引用的文本（按槽位顺序，未命中的引用忽略）
    pub fn strings(&self) -> Vec<String> {
        self.string_slots
            .iter()
            .filter_map(|slot| self.text_at(*slot))
            .collect()
    }

    fn text_at(&self, slot: u16) -> Option<String> {
        self.utf8
            .iter()
            .find(|entry| entry.index == slot)
            .map(|entry| entry.text.clone())
    }
}

/// 顺序游标：所有读取都带边界检查，越界即报错
struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
        if self.remaining() < count {
            return Err("class file is truncated".to_string());
        }
        let slice = &self.bytes[self.offset..self.offset + count];
        self.offset += count;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, String> {
        let slice = self.take(2)?;
        Ok(u16::from_be_bytes([slice[0], slice[1]]))
    }

    fn u32(&mut self) -> Result<u32, String> {
        let slice = self.take(4)?;
        Ok(u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

    fn skip(&mut self, count: usize) -> Result<(), String> {
        self.take(count).map(|_| ())
    }
}

/// 解析常量池；返回全部 UTF8 条目与 String 引用
pub fn read_pool(bytes: &[u8]) -> Result<ConstantPool, String> {
    let mut cursor = Cursor::new(bytes);
    if cursor.u32()? != CLASS_MAGIC {
        return Err("the payload is not a class file".to_string());
    }
    cursor.skip(4)?; // minor + major version
    let declared = cursor.u16()? as usize;
    if declared == 0 {
        return Err("the class constant pool is empty".to_string());
    }

    let mut pool = ConstantPool::default();
    let mut slot: u16 = 1;
    while (slot as usize) < declared {
        let tag_offset = cursor.offset;
        let entry_tag = cursor.u8()?;
        match entry_tag {
            tag::UTF8 => {
                let length = cursor.u16()? as usize;
                let content = cursor.take(length)?;
                pool.utf8.push(Utf8Entry {
                    index: slot,
                    text: String::from_utf8_lossy(content).into_owned(),
                    span: (tag_offset, cursor.offset),
                });
            }
            tag::STRING => {
                pool.string_slots.push(cursor.u16()?);
            }
            tag::INTEGER
            | tag::FLOAT
            | tag::FIELD_REF
            | tag::METHOD_REF
            | tag::INTERFACE_METHOD_REF
            | tag::NAME_AND_TYPE
            | tag::DYNAMIC
            | tag::INVOKE_DYNAMIC => cursor.skip(4)?,
            tag::CLASS | tag::METHOD_TYPE | tag::MODULE | tag::PACKAGE => cursor.skip(2)?,
            tag::METHOD_HANDLE => cursor.skip(3)?,
            tag::LONG | tag::DOUBLE => {
                cursor.skip(8)?;
                // LONG/DOUBLE 各占两个槽位
                slot = slot.saturating_add(1);
            }
            other => {
                return Err(format!("unknown constant pool tag: {other}"));
            }
        }
        slot = slot.saturating_add(1);
    }

    Ok(pool)
}

/// UTF8 条目列表：(文本, 起始偏移, 结束偏移)
pub fn parse_class_constant_pool(bytes: &[u8]) -> Result<Vec<(String, usize, usize)>, String> {
    Ok(read_pool(bytes)?
        .utf8()
        .iter()
        .map(|entry| (entry.text.clone(), entry.span.0, entry.span.1))
        .collect())
}

/// 运行时字符串常量（解析失败按空处理，供扫描阶段使用）
pub fn class_string_constants(bytes: &[u8]) -> Vec<String> {
    read_pool(bytes)
        .map(|pool| pool.strings())
        .unwrap_or_default()
}
