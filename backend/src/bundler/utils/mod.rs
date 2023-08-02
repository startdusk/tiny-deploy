use std::{slice::from_raw_parts, str::from_utf8_unchecked};

pub mod compressible;
pub mod fs_util;
pub mod loader;
pub mod store;

pub fn to_static_str(s: &str) -> &'static str {
    let pointer = s.as_ptr() as usize;
    let length = s.len();
    unsafe { from_utf8_unchecked(from_raw_parts(pointer as *const u8, length)) }
}
