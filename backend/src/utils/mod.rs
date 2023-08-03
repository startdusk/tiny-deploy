mod compressible;
mod fs_util;
mod loader;
mod store;

pub use compressible::*;
pub use fs_util::*;
pub use loader::*;
pub use store::*;

use async_trait::async_trait;
use deno_core::error::AnyError;
use std::{fmt, slice::from_raw_parts, str::from_utf8_unchecked};

#[async_trait]
pub trait ModuleStore: fmt::Debug + Send + Sync {
    async fn get(&self, specifier: &str) -> Result<Box<[u8]>, AnyError>;
    async fn put(&self, specifier: String, code: &[u8]) -> Result<(), AnyError>;
}

pub fn to_static_str(s: &str) -> &'static str {
    let pointer = s.as_ptr() as usize;
    let length = s.len();
    unsafe { from_utf8_unchecked(from_raw_parts(pointer as *const u8, length)) }
}

#[cfg(test)]
pub mod test_util {
    use std::path::PathBuf;

    pub fn testdata_path(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = path.join(format!("../fixtures/testdata/{}", name));
        path.to_string_lossy().into()
    }
}
