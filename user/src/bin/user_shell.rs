#![no_std]
#![no_main]
#![allow(clippy::println_empty_string)]

use alloc::string::String;
use user_lib::{console::getchar, exec, fork, waitpid};

extern crate alloc;

#[macro_use]
extern crate user_lib;

const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const DL: u8 = 0x7fu8;
const BS: u8 = 0x08u8;

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("Rust user shell");
    let mut line: String = String::new();
    print!(">>>");
    loop {
        let c = getchar();
        match c {
            LF | CR => {
                println!("");
                if !line.is_empty() {
                    line.push('\0'); // as the symbol of ending
                    let pid = fork();
                    if pid == 0 {
                        // if it's the sub-process
                        if exec(line.as_str()) == -1 {
                            println!("Error when executing!");
                            return -4;
                        }
                        unreachable!();
                    } else {
                        let mut exit_code: i32 = 0;
                        let exit_pid = waitpid(pid as usize, &mut exit_code);
                        assert_eq!(pid, exit_pid);
                        println!("Shell: Process {} exited with code {}", pid, exit_code);
                    }
                    line.clear();
                }
                print!(">> ");
            },
            BS | DL => {
                if !line.is_empty() {
                    print!("{}", BS as char);
                    print!(" ");
                    print!("{}", BS as char); // replace the final char with space
                    line.pop();
                }

            },
            _ => {
                print!("{}", c as char);
                line.push(c as char);
            }
        }
        
    }
}