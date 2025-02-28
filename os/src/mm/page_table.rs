use alloc::vec::Vec;
use bitflags::{bitflags, Flags};

use super::{address::PhysPageNum, frame_allocator::FrameTracker};

bitflags! {
    // page table entry flags
    pub struct PTEFlgas: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

/// entry should be 64bits
/// [9:0] is flags, [53:10] is ppn, [63:54] is reserved
#[derive(Clone, Copy)]
#[repr(C)]
pub struct PageTableEntry {
    pub bits: usize,
}

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flags: PTEFlgas) -> Self {
        Self { bits: ppn.0 << 10 | flags.bits() as usize }
    }
    pub fn empty() -> Self {
        Self { bits: 0 }
    }
    /// get the ppn from page table entry
    pub fn ppn(&self) -> PhysPageNum {
        (self.bits >> 10 & ((1usize << 44) - 1)).into()
    }
    /// get the flags from page table entry
    pub fn flags(&self) -> PTEFlgas {
        PTEFlgas::from_bits(self.bits as u8).unwrap()
    }
    pub fn is_valid(&self) -> bool {
        !((self.flags() & PTEFlgas::V).is_empty())
    }
    pub fn readable(&self) -> bool {
        !((self.flags() & PTEFlgas::R).is_empty())
    }
    pub fn writable(&self) -> bool {
        !((self.flags() & PTEFlgas::W).is_empty())
    }
    pub fn executable(&self) -> bool {
        !((self.flags() & PTEFlgas::X).is_empty())
    }
}

/// Page Table structure
pub struct PageTable {
    root_ppn: PhysPageNum, // store the pointer of root page table
    frames: Vec<FrameTracker>, //FrameTracker stores the PhysPageNum for all entries in the table by a vec
}

impl PageTable {
    // pub fn new() -> Self {
    //     let frame = fra

    // }
}