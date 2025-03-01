use memory_set::KERNEL_SPACE;

pub mod heap_allocator;
mod address;
mod page_table;
mod frame_allocator;
mod memory_set;

/// initiate heap allocator, frame allocator and kernel space
pub fn init() {
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive_access().activate();
}