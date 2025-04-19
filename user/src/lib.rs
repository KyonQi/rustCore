#![no_std]
#![feature(linkage)]
#![feature(alloc_error_handler)]

use bitflags::bitflags;
use buddy_system_allocator::LockedHeap;
use syscall::{sys_close, sys_exec, sys_exit, sys_fork, sys_get_time, sys_getpid, sys_open, sys_read, sys_waitpid, sys_write, sys_yield};

mod syscall;
pub mod console;
mod lang_items;

const USER_HEAP_SIZE: usize = 16384;
static mut HEAP_SPACE: [u8; USER_HEAP_SIZE] = [0; USER_HEAP_SIZE];

#[global_allocator]
static HEAP: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.entry")]
pub extern "C" fn _start() -> ! {
    unsafe {
        HEAP.lock().init(&raw mut HEAP_SPACE as usize, USER_HEAP_SIZE);
    }
    exit(main());
}

#[linkage = "weak"]
#[unsafe(no_mangle)]
fn main() -> i32 {
    panic!("Can't find main");
}

// need to set 0 for .bss section
// fn clear_bss() {
//     unsafe extern "C" {
//         fn start_bss();
//         fn end_bss();
//     }
//     (start_bss as usize..end_bss as usize).for_each(|a| {
//         unsafe { (a as *mut u8).write_volatile(0); }
//     });
// }

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

pub fn open(path: &str, flags: OpenFlags) -> isize {
    sys_open(path, flags.bits())
}

pub fn close(fd: usize) -> isize {
    sys_close(fd)
}

pub fn read(fd: usize, buffer: &mut [u8]) -> isize {
    sys_read(fd, buffer)
}

pub fn write(fd: usize, buffer: &[u8]) -> isize {
    sys_write(fd, buffer)
}

pub fn exit(exit_code: i32) -> ! {
    sys_exit(exit_code);
}

pub fn yield_() -> isize {
    sys_yield()
}

pub fn get_time() -> isize {
    sys_get_time()
}

// pub fn sbrk(size: i32) -> isize {
//     sys_sbrk(size)
// }

pub fn getpid() -> isize {
    sys_getpid()
}

pub fn fork() -> isize {
    sys_fork()
}

pub fn exec(path: &str) -> isize {
    sys_exec(path)
}

/// this function will wait all the sub-process
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                // there is no finishing sub-process
                yield_();
            },
            // -1 or other pid
            exit_pid => return exit_pid,
        }
    }
}

/// this function will wait the specific pid
pub fn waitpid(pid: usize, exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(pid as isize, exit_code as *mut _) {
            -2 => {
                yield_();
            },
            exit_pid => return exit_pid,
        }
    }
}

pub fn sleep(period_ms: usize) {
    let start = sys_get_time();
    while sys_get_time() < start + period_ms as isize {
        sys_yield();
    }
}