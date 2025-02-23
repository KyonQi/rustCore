use log::{debug, info};

use crate::{print, syscall::process::sys_exit};

const FD_OUT: usize = 1; // to the terminal

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
        FD_OUT => {
            let slice = unsafe {
                core::slice::from_raw_parts(buf, len)
            };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => {
            return -1 as isize;
            // panic!("Unsupported fd in sys_write!");
        }
    }
}