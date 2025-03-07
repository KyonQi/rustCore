use core::arch::asm;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_SBRK: usize = 214;

const SYSCALL_READ: usize = 63;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_WAITPID: usize = 260;

#[inline(always)]
fn sys_call(eid: usize, args: [usize; 3]) -> isize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") eid
        );
    }
    ret
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    sys_call(SYSCALL_READ, [fd, buffer.as_mut_ptr() as usize, buffer.len()])
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    sys_call(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> isize {
    sys_call(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}

pub fn sys_yield() -> isize {
    sys_call(SYSCALL_YIELD, [0, 0, 0])
}

pub fn sys_get_time() -> isize {
    sys_call(SYSCALL_GET_TIME, [0, 0, 0])
}

pub fn sys_sbrk(size: i32) -> isize {
    sys_call(SYSCALL_SBRK, [size as usize, 0, 0])
}

pub fn sys_getpid() -> isize {
    sys_call(SYSCALL_GETPID, [0, 0, 0])
}

pub fn sys_fork() -> isize {
    sys_call(SYSCALL_FORK, [0, 0, 0])
}

pub fn sys_exec(path: &str) -> isize {
    sys_call(SYSCALL_EXEC, [path.as_ptr() as usize, 0, 0])
}

pub fn sys_waitpid(pid:usize, exit_code: *mut i32) -> isize {
    sys_call(SYSCALL_WAITPID, [pid as usize, exit_code as usize, 0])
}