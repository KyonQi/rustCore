#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
extern crate alloc;

mod lang_items;
mod sbi;
mod console;
mod log;
// mod batch;
mod loader;
mod config;
mod sync;
mod trap;
mod syscall;
mod task;
mod timer;
mod mm;

use core::arch::global_asm;

use ::log::{debug, error, info, trace, warn};
use mm::heap_allocator::{heap_test, init_heap};
use sbi::{console_putchar, sleep};

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));


// SAFETY: there is no other global function of this name
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    log::init(); // init a global logger

    unsafe extern "C" {
        fn stext(); // begin addr of text segment
        fn etext(); // end addr of text segment
        fn srodata(); // start addr of Read-Only data segment
        fn erodata(); // end addr of Read-Only data ssegment
        fn sdata(); // start addr of data segment
        fn edata(); // end addr of data segment
        fn sbss(); // start addr of BSS segment
        fn ebss(); // end addr of BSS segment
        fn boot_stack_lower_bound(); // stack lower bound
        fn boot_stack_top(); // stack top
    }

    info!("[kernel] .text [{:#x}, {:#x})", stext as usize, etext as usize);
    info!("[kernel] .rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    info!("[kernel] .data [{:#x}, {:#x})", sdata as usize, edata as usize);
    info!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    info!("[kernel] .stack [{:#x}, {:#x})", boot_stack_lower_bound as usize, boot_stack_top as usize);
    
    // error!("test error");
    // warn!("test warn");
    // info!("test info");
    // debug!("test debug");
    // trace!("test trace");
    
    // sleep(2);
    println!("Hello, World");
    // init_heap();
    // heap_test();
    // panic!("Shutdown right now!");

    mm::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
    // batch::init();
    // batch::run_next_app();
}

// need to set 0 for .bss section
fn clear_bss() {
    unsafe extern "C" {
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe { (a as *mut u8).write_volatile(0); }
    });
}