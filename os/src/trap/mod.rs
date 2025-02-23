mod context;
use core::arch::global_asm;

pub use context::TrapContext;
use riscv::register::{scause::{self, Exception, Interrupt, Trap}, sie, stval, stvec};

use crate::{println, syscall::{self, syscall}, task::{exit_current_and_run_next, suspend_current_and_run_next}, timer::set_next_trigger};

global_asm!(include_str!("trap.S"));

/// initialize CSR stvec as the entry of __alltraps
pub fn init() {
    unsafe extern "C" {
        fn __alltraps();
    }
    unsafe {
        stvec::write(__alltraps as usize, stvec::TrapMode::Direct);
    }
}

/// enable timer interrupt
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

/// handle interrupt, exception, system call from user space
#[unsafe(no_mangle)]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4; // cuz it points to ecall originially
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        },
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
            exit_current_and_run_next();
            // panic!("[kernel] Cannot continue!");
            // run_next_app();
        },
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next();
            // panic!("[kernel] Cannot continue!");
            // run_next_app();
        },
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        },
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    cx
}