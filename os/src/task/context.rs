#[derive(Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    /// return address (e.g. __restore) of __switch ASM function
    ra: usize,
    /// kernel stack pointer of app
    sp: usize,
    /// callee saved registers: s0-s11
    s: [usize; 12],
}

impl TaskContext {
    pub fn zero_init() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s: [0; 12],
        }
    }

    /// set task context {__restore ASM funciton, kernel stack, s_0..12 }
    pub fn goto_restore(kstack_ptr: usize) -> Self {
        unsafe extern "C" {
            fn __restore();
        }
        Self {
            ra: __restore as usize,
            sp: kstack_ptr,
            s: [0; 12],
        }
    }
}