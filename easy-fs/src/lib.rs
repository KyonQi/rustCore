//!An easy file system isolated from the kernel
#![no_std]
#![deny(missing_docs)]
extern crate alloc;

mod block_dev;