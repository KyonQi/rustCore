// it's used for batch system

use core::arch::asm;

use lazy_static::lazy_static;

use crate::trap::TrapContext;
use crate::{println, sbi::shutdown};
use crate::sync::UPSafeCell;

const APP_BASE_ADDRESS: usize = 0x80400000;
const APP_SIZE_LIMIT: usize = 0x20000;

const MAX_APP_NUM: usize = 16;

const KERNEL_STACK_SIZE: usize = 4096 * 2; // 8KB for kernel stack
const USER_STACK_SIZE: usize = 4096 * 2; // 8KB for user stack

/// It's used to show the total apps and the current app the OS is going to run
struct AppManager {
    num_app: usize,
    current_app: usize,
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x}]",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    unsafe fn load_app(&self, app_id: usize) {
        if app_id >= self.num_app {
            println!("All applications completed");
            shutdown(false);
        }
        println!("[kernel] Loading app_{}", app_id);
        // SAFETY: Only the app address is set to 0 everytime
        unsafe {
            core::ptr::write_bytes(APP_BASE_ADDRESS as *mut u8, 0, APP_SIZE_LIMIT);
        }
        // SAFETY: Only copy the app content from src to dst
        unsafe {
            let app_src = core::slice::from_raw_parts(self.app_start[app_id] as *const u8, self.app_start[app_id + 1] - self.app_start[app_id]);
            let app_dst = core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len());
            app_dst.copy_from_slice(app_src);
        }
        // SAFETY: It's used to guarantee that a subsequent instruction fetch must observe all previous writes to the memory
        // Therefore, fence.i must be executed after we have loaded
        // the code of the next app into the instruction memory.
        unsafe {
            asm!("fence.i");
        }
    }

    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            unsafe extern "C" {
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app = num_app_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] = core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            } 
        })
    };
}

#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

impl KernelStack {
    /// return the tail of the vector as the sp
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }

    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        // reserve the space for TrapContext => push TrapContext to stack
        // as *mut TrapContext: convert the address to TrapContext ptr
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        // SAFETY: write the cx to the address of cx_ptr. The space has been reserved for it
        unsafe {
            *cx_ptr = cx;
        }
        unsafe {
            cx_ptr.as_mut().unwrap()
        }
    }
}

impl UserStack {
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};

static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

/// init the batch system
pub fn init() {
    print_app_info();
}

/// print apps info
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

/// run next app
pub fn run_next_app() -> ! {
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        app_manager.load_app(current_app);
    }
    app_manager.move_to_next_app();
    drop(app_manager); // manually release the resource
    unsafe extern "C" {
        fn __restore(cx_addr: usize);
    }
    // after __restore, the program will go to the APP_BASE_ADDRESS with sp pointing to user stack
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(APP_BASE_ADDRESS, 
            USER_STACK.get_sp())) as *const _ as usize);
    }
    panic!("Unreachable in batch::run_current_app!");
}