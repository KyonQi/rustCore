use alloc::sync::Arc;
use crate::{block_cache::get_block_cache, block_dev::BlockDevice, BLOCK_SZ};
type BitmapBlock = [u64; 64]; // the same as 512 bytes in one block
const BLOCK_BITS: usize = BLOCK_SZ * 8; // bytes * 8 -> total bits in one block -> 4096 bits in one block

pub struct Bitmap {
    start_block_id: usize,
    blocks: usize,
}

/// decompose the bits into (block_pos, bits64_pos, inner_pos)
/// every byte is corresponding with one bit in a bitmap
fn decomposition(mut bit: usize) -> (usize, usize, usize) {
    let block_pos = bit / BLOCK_BITS; // the index of which block it is
    bit %= BLOCK_BITS;
    (block_pos, bit / 64, bit % 64)
}

impl Bitmap {
    /// a new bitmap from start_block_id and number of blocks
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }

    /// allocate a new block from a block device
    /// if success, return the global bit number; if not, None
    /// the goal is to find the 0 bit in a bitmap, and set to 1
    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>) -> Option<usize> {
        for block_id in 0..self.blocks {
            let pos = get_block_cache(
                block_id + self.start_block_id, 
                Arc::clone(block_device),
            )
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                // here we capture the block_cache, and convert it as BitmapBlock (512 bytes) (modify function does it)
                // it will try to find the first bits64 that is not maximum, i.e., not full with 1
                // the map will return the index of bits64, and sequence of 1st 0 in this bits64 => (bits64_pos, inner_pos)
                if let Some((bits64_pos, inner_pos)) = bitmap_block
                    .iter()
                    .enumerate()
                    .find(|(_, bits64)| **bits64 != u64::MAX) 
                    .map(|(bits64_pos, bits64)| (bits64_pos, bits64.trailing_ones() as usize))
                {
                    // modify cache
                    bitmap_block[bits64_pos] |= 1u64 << inner_pos; // allocate to 1
                    Some(block_id * BLOCK_BITS + bits64_pos * 64 + inner_pos as usize) // return a specific position
                } else {
                    None
                }
            });
            if pos.is_some() {
                return pos;
            }
        }
        None
    }

    /// deallocate a block
    /// the idea behind this function is similar to alloc: it try tp find out the bit we want to deallocate
    pub fn dealloc(&self, block_device: &Arc<dyn BlockDevice>, bit: usize) {
        let (block_pos, bits64_pos, inner_pos) = decomposition(bit);
        get_block_cache(
            block_pos + self.start_block_id, 
            Arc::clone(block_device)
        )
        .lock()
        .modify(0, |bitmap_block: &mut BitmapBlock| {
            assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
            bitmap_block[bits64_pos] -= 1u64 << inner_pos; // deallocate to 0
        });
    }

    /// get the max number of allocatable blocks
    pub fn maximum(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
}