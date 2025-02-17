#![no_std]
#![no_main]

use core::arch::asm;

use riscv::register::sstatus::{self, SPP};

#[macro_use]
extern crate user_lib;

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("Info: Test to execute privileged CSR in U mode");
    println!("Kernel should kill this application");
    unsafe {
        sstatus::set_spp(SPP::User);
    }
    0
}