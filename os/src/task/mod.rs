use alloc::sync::Arc;
pub use context::TaskContext;
use lazy_static::lazy_static;
pub use manager::add_task;
pub use processor::{
    current_task, current_trap_cx, current_user_token, run_tasks, schedule, take_current_task,
    Processor,
};
use task::{TaskControlBlock, TaskStatus};

use crate::{loader::get_app_data_by_name, println, sbi::shutdown};

mod context;
mod manager;
mod pid;
mod processor;
mod switch;
mod task;

/// Suspend the current `Running` task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    // current running task
    let task = take_current_task().unwrap();

    // access current TCB exclusively
    let mut task_inner = task.inner_exclusive_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // release current PCB

    // push back to ready queue
    add_task(task);
    // jump to scheduling cycle (schedule to idle)
    schedule(task_cx_ptr);
}

pub const IDLE_PID: usize = 0;

pub fn exit_current_and_run_next(exit_code: i32) {
    let task = take_current_task().unwrap();

    let pid = task.getpid();
    // if current exit task is IDLE_TASK
    if pid == IDLE_PID {
        println!(
            "[kernel] Idle process exit with exit_code {} ...",
            exit_code
        );
        if exit_code != 0 {
            shutdown(true)
        } else {
            shutdown(false)
        }
    }

    // Access current TCB exclusively
    let mut inner = task.inner_exclusive_access();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;

    // Access initproc TCB exclusively
    {
        // move the child task into initproc instead of its parent
        let mut initproc_inner = INITPROC.inner_exclusive_access();
        for child in inner.children.iter() {
            child.inner_exclusive_access().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // release the parent PCB

    inner.children.clear();
    inner.memory_set.recycle_data_pages(); // deallocate user space
    drop(inner);
    drop(task);
    
    // no need to save the current task context, since it exited
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    ///Globle process that init user shell
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    ));
}

///Add init process to the manager
pub fn add_initproc() {
    add_task(INITPROC.clone());
}