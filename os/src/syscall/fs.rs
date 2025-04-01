use core::panic;

use log::{debug, info};

use crate::{mm::translated_byte_buffer, print, sbi::console_getchar, syscall::process::sys_exit, task::{current_user_token, suspend_current_and_run_next}};

const FD_STDIN: usize = 0;
const FD_STDOUT: usize = 1; // to the terminal

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    // // for experiment 2
    // let app_range = get_current_app_addr();
    // let stack_range = get_user_stack_range();
    // let buf_begin_pointer = buf as usize;
    // let buf_end_pointer = unsafe{buf.offset(len as isize)} as usize;
    // if !(
    //         (buf_begin_pointer >= app_range[0] && buf_begin_pointer < app_range[1]) && 
    //         (buf_end_pointer >= app_range[0] && buf_end_pointer < app_range[1])
    //     )&&
    //     !(
    //         (buf_begin_pointer >= stack_range[0] && buf_begin_pointer < stack_range[1]) && 
    //         (buf_end_pointer >= stack_range[0] && buf_end_pointer < stack_range[1])
    //     ) {
    //     return -1 as isize;
    // }
    match fd {
        FD_STDOUT => {
            // let slice = unsafe {
            //     core::slice::from_raw_parts(buf, len)
            // };
            // let str = core::str::from_utf8(slice).unwrap();
            // print!("{}", str);
            // len as isize
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        },
        _ => {
            return -1 as isize;
            // panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDIN => {
            assert_eq!(len, 1, "Only support len = 1 in sys_read");
            let mut c: usize;
            loop {
                c = console_getchar();
                if c == 0 {
                    suspend_current_and_run_next();
                    continue;
                } else {
                    break;
                }
            }
            let ch = c as u8;
            let mut buffers = translated_byte_buffer(current_user_token(), buf, len);
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(ch);
            }
            1
        },
        _ => {
            panic!("Unsupported fd in sys_read");
        }
    }
}