#![no_std]
#![no_main]
mod lang_items;
mod sbi;
mod console;

use core::arch::global_asm;

use sbi::console_putchar;
global_asm!(include_str!("entry.asm"));

// SAFETY: there is no other global function of this name
#[unsafe(no_mangle)]
pub fn rust_main() -> ! {
    clear_bss();
    console_putchar('o' as usize);
    console_putchar('k' as usize);
    println!("Hello, World");
    panic!("Shutdown right now!");
    // loop {
        
    // }
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