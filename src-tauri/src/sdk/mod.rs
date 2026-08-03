//! McSDK FFI 绑定层
//!
//! 提供跨平台 SDK 动态库加载和 FFI 函数绑定
//! 子模块：ffi_types / types（FFI 数据类型）/ instance（lite 绑定）/ library（库加载入口 + SdkError）

mod ffi_types;
mod instance;
mod library;
mod types;

pub use ffi_types::*;
pub use instance::{SdkFunctions, SdkInstance};
pub use library::{check_sdk_library, get_sdk_filename, get_sdk_library_path, SdkError};
pub use types::*;
