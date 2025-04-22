//!An easy file system isolated from the kernel
#![no_std]
#![deny(missing_docs)]
extern crate alloc;

mod block_dev;
mod block_cache;
mod bitmap;
mod layout;

/// use a block size of 512 bytes
pub const BLOCK_SZ: usize = 512;