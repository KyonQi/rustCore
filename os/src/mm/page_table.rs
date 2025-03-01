use core::result;

use alloc::vec;
use alloc::vec::Vec;
use bitflags::{bitflags, Flags};

use super::{address::{PhysPageNum, StepByOne, VirtAddr, VirtPageNum}, frame_allocator::{frame_alloc, FrameTracker}};

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
    frames: Vec<FrameTracker>, //FrameTracker stores the PhysPageNum for all used entries in the table by a vec
}

impl PageTable {
    /// assign a new frame for the PageTable itself
    pub fn new() -> Self {
        let frame = frame_alloc().unwrap(); // frametracker is the unit we manage the physical page
        PageTable { root_ppn: frame.ppn, frames: vec![frame] }        
    }

    /// temporarily used to get arguments from user space, used to find the pagetable manually
    pub fn from_token(satp: usize) -> Self {
        Self { 
            root_ppn: PhysPageNum::from(satp & ((1usize << 44) - 1)), 
            frames: Vec::new() 
        }
    }

    fn find_pte_create(&mut self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                // the final level to physical page entry
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                // if pte itself is not valid, it means the page isn't created before
                // we should create the frame for that page first
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame.ppn, PTEFlgas::V);
                self.frames.push(frame); // stores all frames we created in this pagetable
            }
            ppn = pte.ppn();
        }
        result
    }

    fn find_pte(&self, vpn: VirtPageNum) -> Option<&mut PageTableEntry> {
        let idxs = vpn.indexes();
        let mut ppn = self.root_ppn;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, idx) in idxs.iter().enumerate() {
            let pte = &mut ppn.get_pte_array()[*idx];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

    /// map a vpn to a ppn
    pub fn map(&mut self, vpn: VirtPageNum, ppn: PhysPageNum, flags: PTEFlgas) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn); // make sure that the final findings of vpn isn't used before
        // map our ppn entry to the final finding of vpn
        *pte = PageTableEntry::new(ppn, flags | PTEFlgas::V);
    }

    /// unmap a vpn to a ppn
    pub fn unmap(&mut self, vpn: VirtPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        // unmap our ppn entry to empty
        *pte = PageTableEntry::empty();
    }

    /// try to find a pte from the vpn, return None if it's not created instead of creating it
    /// 当遇到需要查一个特定页表（非当前正处在的地址空间的页表时），便可先通过 PageTable::from_token 新建一个页表，再调用它的 translate 方法查页表。
    pub fn translate(&self, vpn: VirtPageNum) -> Option<PageTableEntry> {
        self.find_pte(vpn).map(|pte| *pte)
    }

    pub fn token(&self) -> usize {
        8usize << 60 | self.root_ppn.0
    }
}

/// transfer the virt addr (ptr..ptr+len) to the physical addr
/// return a Vec< &mut [u8] >
pub fn translated_byte_buffer(token: usize, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
    let page_table = PageTable::from_token(token); // token is the value of satp, which contains the pointer of root page
    let mut start = ptr as usize;
    let end = start + len;
    let mut v = Vec::new();
    while start < end {
        let start_va = VirtAddr::from(start);
        let mut vpn = start_va.floor(); // calculate the vpn from start_va
        let ppn = page_table.translate(vpn).unwrap().ppn();
        vpn.step(); // get into the next page
        let mut end_va: VirtAddr = vpn.into();
        end_va = end_va.min(VirtAddr::from(end)); // make sure the range doesn't cover the next page: within one page
        
        if end_va.page_offset() == 0 {
            // copy the whole page
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..]);
        } else {
            // copy to the offset (partial of the page)
            v.push(&mut ppn.get_bytes_array()[start_va.page_offset()..end_va.page_offset()]);
        }
        start = end_va.into(); // go to next page
    }
    v
}