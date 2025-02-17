#![no_std]
#![no_main]

use core::arch::asm;

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Info: Test to execute privileged instruction in U mode");
    println!("Kernel should kill this application");
    unsafe {
        asm!("sret");
    }
    0
}