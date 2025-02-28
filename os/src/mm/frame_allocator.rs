use core::fmt::Debug;

use alloc::vec::Vec;
use lazy_static::lazy_static;

use crate::{config::MEMORY_END, mm::address::PhysAddr, println, sync::UPSafeCell};

use super::address::PhysPageNum;

/// A wrap structure the physical page of specific PhysPageNum
pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // first clean the mem, then assign
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Debug for FrameTracker {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("FrameTracker:PPN={:#x}", self.ppn.0))
    }
}

/// deallocate the mem content in the specific ppn
impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

/// an implementation of frame allocator
/// This struct is used to allocate and deallocate the memory
pub struct StackFrameAllocator {
    current: usize, // start of spare frame
    end: usize, // end of spare frame
    recycled: Vec<usize>, // a vec stores the discard frame that could be reused
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            // if recycle vec has values, we simply reuse them
            Some(ppn.into())
        } else if self.current == self.end {
            // if no spare capacity, return none
            None
        } else {
            // or increase the current counter, and assign the previous value to frame
            self.current += 1;
            Some((self.current - 1).into())
        }
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        if ppn >= self.current || self.recycled.iter().any(|&v| v == ppn) {
            // perform the valid check, below is invalid dealloc
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        self.recycled.push(ppn);
    }
}

type FrameAllocatorImpl = StackFrameAllocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = unsafe {
        UPSafeCell::new(FrameAllocatorImpl::new())
    };
}


/// initialize the frame allocator using 'ekernel' and 'MEMORY_END'
pub fn init_frame_allocator() {
    unsafe extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR.exclusive_access().init(
        PhysAddr::from(ekernel as usize).ceil(),
        PhysAddr::from(MEMORY_END).floor()
    );
}

/// allocate a frame
pub fn frame_alloc() -> Option<FrameTracker> {
    // get a ppn from global frame_allocator, and convert it into a real page content memory with FrameTracker
    FRAME_ALLOCATOR.exclusive_access()
    .alloc().map(FrameTracker::new)
}

/// deallocate a frame, this function is automatically called by Drop from FrameTracker struct -> RAII
fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR.exclusive_access().dealloc(ppn);
}

/// A simple usage example of frame_allocator
#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!")
}