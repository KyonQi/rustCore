use core::arch::asm;

use riscv::register::{sie::set_stimer, time};

use crate::println;

// legacy extensions: ignore fid
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;

const SRST_EXTENSION: usize = 0x53525354;
const SYSTEM_RESET_FUNCTION: usize = 0;

// for sleep system call
const CLOCK_FREQ: usize = 10_000_000; // 10 MHz
const SBI_SET_TIMER: usize = 0x54494D45;

#[inline(always)]
fn sbi_call(eid: usize, fid: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x16") fid,
            in("x17") eid,
        );
    }
    ret
}

pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, 0, c, 0, 0);
}

pub fn sleep(t: usize) {
    let current_time = time::read(); // get the cur time
    let wake_up_time = current_time + t * CLOCK_FREQ;
    // SAFETY: allow the timer interrupt by riscv lib
    unsafe { set_stimer(); }
    sbi_call(SBI_SET_TIMER, 0, wake_up_time, 0, 0); // set the timer
    // println!("{}", ret);
    // SAFETY: wait for timer interrupt
    unsafe { asm!("wfi"); }
}

pub fn set_timer(timer: usize) {
    sbi_rt::set_timer(timer as _);
}

pub fn shutdown(failure: bool) -> !{
    if !failure {
        // shutdown with no reason
        sbi_call(SRST_EXTENSION, SYSTEM_RESET_FUNCTION, 0, 0, 0);
    } else {
        sbi_call(SRST_EXTENSION, SYSTEM_RESET_FUNCTION, 0, 1, 0);
    }
    unreachable!()
}