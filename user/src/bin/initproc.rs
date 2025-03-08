#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exec, fork, wait, yield_};

#[unsafe(no_mangle)]
fn main() -> i32 {
    if fork() == 0 {
        // if the return value is 0, it means it's in the sub-process (the parent process will get the sub-pid value)
        exec("user_shell\0");
    } else {
        loop {
            let mut exit_code: i32 = 0;
            // the exit_code stores the returning value from sub-process
            let pid = wait(&mut exit_code);
            if pid == -1 {
                yield_();
                continue;
            }
            println!(
                "[initproc] Relesed a zombie process, pid={}, exit_code={}",
                pid, exit_code,
            );
        }
    }
    0
}