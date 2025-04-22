use core::fmt::Debug;

use alloc::{sync::Arc, vec::Vec};

use crate::{block_cache::get_block_cache, block_dev::BlockDevice, BLOCK_SZ};

/// magic number for sanity check
const EFS_MAGIC: u32 = 0x3b800001;
/// the max number of direct inodes
const INODE_DIRECT_COUNT: usize = 28;
/// the max length of inode name
const NAME_LENGTH_LIMIT: usize = 27;
/// the max number of indirect1 inodes
const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / 4; // u32 for one entry in one block; each points to one direct block data
/// the max number of indirect2 inodes
const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * INODE_INDIRECT1_COUNT; // same u32 for one entry; each points to one indirect1 block
/// the upper bound of direct inode index
const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
/// the upper bound of indirect1 inode index
const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
/// the upper bound of indirect2 inode index
#[allow(unused)]
const INDIRECT2_BOUND: usize = INDIRECT1_BOUND + INODE_INDIRECT2_COUNT;

/// Super block of a filesystem
/// 
/// If the file is small, only direct indexing is used.
/// The `direct` array can point to up to INODE_DIRECT_COUNT data blocks.
/// When set to 28, up to 14KiB(512 bytes x 28) can be addressed directly.
///
/// For larger files, indirect1 (single indirect indexing) is used.
/// It points to a block in the data region, which holds u32 entries,
/// each pointing to another data block. This adds up to 64KiB (512 / 4 * 512 / 1024).
///
/// If the file exceeds 78KiB (direct + indirect1), indirect2 is needed.
/// It points to a second-level index block in the data region.
/// Each entry in this block points to a first-level index block,
/// allowing access to a much larger portion of the file. This adds up to 8MiB (512 / 4 * 64KiB)
#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}

impl Debug for SuperBlock {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SuperBlock")
            .field("total_blocks", &self.total_blocks)
            .field("inode_bitmap_blocks", &self.inode_bitmap_blocks)
            .field("inode_area_blocks", &self.inode_area_blocks)
            .field("data_bitmap_blocks", &self.data_bitmap_blocks)
            .field("data_area_blocks", &self.data_area_blocks)
            .finish()
    }
}

impl SuperBlock {
    /// initialize a super block
    pub fn initialize(&mut self, total_blocks: u32, inode_bitmap_blocks: u32,
        inode_area_blocks: u32, data_bitmap_blocks: u32, data_area_blocks: u32) {
        *self = Self {
            magic: EFS_MAGIC,
            total_blocks,
            inode_area_blocks,
            inode_bitmap_blocks,
            data_bitmap_blocks,
            data_area_blocks,
        }
    }

    /// check if a super block is valid using efs magic
    pub fn is_valid(&self) -> bool {
        self.magic == EFS_MAGIC
    }
}

/// type of a disk inode
/// Now, it only supports either `file` or `directory`
#[derive(PartialEq)]
pub enum DiskInodeType {
    File,
    Directory,
}

/// a indirect block
type IndirectBlock = [u32; BLOCK_SZ / 4];
/// a data block
type DataBlock = [u8; BLOCK_SZ];
/// a disk inode
#[repr(C)]
pub struct DiskInode {
    pub size: u32,
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirect2: u32,
    type_: DiskInodeType,
}

impl DiskInode {
    /// Initialize a disk inode, as well as all direct inodes under it
    /// indirect1 and indirect2 block are allocated only when they are needed
    pub fn initialize(&mut self, type_: DiskInodeType) {
        self.size = 0;
        self.direct.iter_mut().for_each(|v| *v = 0);
        self.indirect1 = 0;
        self.indirect2 = 0;
        self.type_ = type_;
    }

    /// whether this inode is a directory
    pub fn is_dir(&self) -> bool {
        self.type_ == DiskInodeType::Directory
    }

    /// whether this inode is a file
    pub fn is_file(&self) -> bool {
        self.type_ == DiskInodeType::File
    }

    fn _data_block(size: u32) -> u32 {
        (size + BLOCK_SZ as u32 - 1) / BLOCK_SZ as u32
    }
    /// calculate the total data blocks this inode has (ceiling division)
    pub fn data_blocks(&self) -> u32 {
        Self::_data_block(self.size)
    }

    /// return number of blocks needed including indirect1/2
    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_block(size) as usize;
        let mut total = data_blocks as usize; // pratical data_blocks that we need
        // indirect1
        if data_blocks > INODE_DIRECT_COUNT {
            total += 1; // because we need to store the indirect1 pointer
        }
        // indirect2
        if data_blocks > INDIRECT1_BOUND {
            total += 1; // because we need to store the indirect2 pointer
            // sub indirect1
            total += (data_blocks - INDIRECT1_BOUND + INODE_INDIRECT1_COUNT - 1) / INODE_INDIRECT1_COUNT; // how many indirec1 pointers stored that we really need
        }
        total as u32
    }

    /// get the number pf data blocks that have to be allocated given the new size of the data
    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }

    /// get id of data block given inner id (specific block_id for this inode needed)
    /// this means getting the exact content block instead of block_id
    pub fn get_block_id(&self, inner_id: u32, block_device: &Arc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_COUNT {
            self.direct[inner_id]
        } else if inner_id < INDIRECT1_BOUND {
            // indirect1 points to a block that stores the extra contents
            get_block_cache(
                self.indirect1 as usize, 
                Arc::clone(block_device))
                .lock()
                .read(0, |indirect_block: &IndirectBlock| {
                    indirect_block[inner_id - INODE_DIRECT_COUNT]
                }
            )
        } else {
            let last = inner_id - INDIRECT1_BOUND;
            // first get the specific indirect1 from indirect2
            let indirect1 = get_block_cache(
                self.indirect2 as usize, 
                Arc::clone(block_device))
                .lock()
                .read(0, |indirect2: &IndirectBlock| {
                    indirect2[last / INODE_INDIRECT1_COUNT]
                }
            );
            // second get the specific block from indirect1
            get_block_cache(
                indirect1 as usize, 
                Arc::clone(block_device))
                .lock()
                .read(0, |indirect_block: &IndirectBlock| {
                    indirect_block[last % INODE_INDIRECT1_COUNT]
                }
            )
        }
    }

    /// increase the size of current disk inode
    pub fn increase_size(&mut self, new_size: u32, new_blocks: Vec<u32>, block_device: &Arc<dyn BlockDevice>) {
        let mut current_blocks = self.data_blocks();
        self.size = new_size;
        let mut total_blocks = self.data_blocks();
        let mut new_blocks = new_blocks.into_iter();
        // fill direct
        while current_blocks < total_blocks.min(INODE_DIRECT_COUNT as u32) {
            self.direct[current_blocks as usize] = new_blocks.next().unwrap();
            current_blocks += 1;
        }
        // alloc indirect1
        if total_blocks > INODE_DIRECT_COUNT as u32 {
            if current_blocks == INODE_DIRECT_COUNT as u32 {
                self.indirect1 = new_blocks.next().unwrap();
            }
            current_blocks -= INODE_DIRECT_COUNT as u32;
            total_blocks -= INODE_DIRECT_COUNT as u32;
        } else {
            return;
        }
        // fill indirect1
        get_block_cache(
            self.indirect1 as usize, 
            Arc::clone(block_device)
        )
            .lock()
            .modify(0, |indirect1: &mut IndirectBlock| {
                while current_blocks < total_blocks.min(INODE_INDIRECT1_COUNT as u32) {
                    indirect1[current_blocks as usize] = new_blocks.next().unwrap();
                    current_blocks += 1;
                }
            });
        // alloc indirect2
        if total_blocks > INODE_INDIRECT1_COUNT as u32 {
            if current_blocks == INODE_INDIRECT1_COUNT as u32 {
                self.indirect2 = new_blocks.next().unwrap();
            }
            current_blocks -= INODE_INDIRECT1_COUNT as u32;
            total_blocks -= INODE_INDIRECT1_COUNT as u32;
        } else {
            return;
        }
        // fill indirect2 from (a0, b0) to (a1, b1)
        let mut a0 = current_blocks as usize / INODE_INDIRECT1_COUNT; // index of which indirect1
        let mut b0 = current_blocks as usize % INODE_INDIRECT1_COUNT; // index of which block inside indirect1
        let a1 = total_blocks as usize / INODE_INDIRECT1_COUNT;
        let b1 = total_blocks as usize % INODE_INDIRECT1_COUNT;
        get_block_cache(
            self.indirect2 as usize, 
            Arc::clone(block_device)
        )
            .lock()
            .modify(0, |indirect2: &mut IndirectBlock| {
                while (a0 < a1) || (a0 == a1 && b0 < b1) {
                    if b0 == 0 {
                        indirect2[a0] = new_blocks.next().unwrap();
                    }
                    // fill current
                    get_block_cache(
                        indirect2[a0] as usize, 
                        Arc::clone(block_device)
                    )
                        .lock()
                        .modify(0, |indirect1: &mut IndirectBlock| {
                            indirect1[b0] = new_blocks.next().unwrap();
                        });
                    // move to next
                    b0 += 1;
                    if b0 == INODE_INDIRECT1_COUNT {
                        b0 = 0;
                        a0 += 1;
                    }
                }
            });
    }

    /// clear size to zero and return blocks that should be deallocated
    /// later will clear the block contents
    /// it will return all blocks that need to be cleared
    pub fn clear_size(&mut self, block_device: &Arc<dyn BlockDevice>) -> Vec<u32> {
        let mut v: Vec<u32> = Vec::new();
        let mut data_blocks = self.data_blocks() as usize;
        self.size = 0;
        let mut current_blocks = 0usize;
        // direct
        while current_blocks < data_blocks.min(INODE_DIRECT_COUNT) {
            v.push(self.direct[current_blocks]);
            self.direct[current_blocks] = 0;
            current_blocks += 1;
        }
        // indirect1 block
        if data_blocks > INODE_DIRECT_COUNT {
            v.push(self.indirect1);
            data_blocks -= INODE_DIRECT_COUNT;
            current_blocks = 0;
        } else {
            return v;
        }
        // clear indirect1
        get_block_cache(
            self.indirect1 as usize, 
            Arc::clone(block_device)
        )
            .lock()
            .modify(0, |indirect1: &mut IndirectBlock| {
                while current_blocks < data_blocks.min(INODE_INDIRECT1_COUNT) {
                    v.push(indirect1[current_blocks]);
                    current_blocks += 1;
                }
            });
        self.indirect1 = 0;
        // indirect2 block
        if data_blocks > INODE_INDIRECT1_COUNT {
            v.push(self.indirect2);
            data_blocks -= INODE_INDIRECT1_COUNT;
        } else {
            return v;
        }
        // clear indirect2
        assert!(data_blocks <= INODE_INDIRECT2_COUNT);
        let a1 = data_blocks / INODE_INDIRECT1_COUNT;
        let b1 = data_blocks % INODE_INDIRECT1_COUNT;
        get_block_cache(
            self.indirect2 as usize, 
            Arc::clone(block_device)
        )
            .lock()
            .modify(0, |indirect2: &mut IndirectBlock| {
                // full indirect1 blocks
                // iter.take(n) returns first n iters
                for entry in indirect2.iter_mut().take(a1) {
                    v.push(*entry);
                    get_block_cache(
                        *entry as usize, 
                        Arc::clone(block_device)
                    )
                        .lock()
                        .modify(0, |indirect1: &mut IndirectBlock| {
                            for entry in indirect1.iter() {
                                v.push(*entry);
                            }
                        });
                }
                // last indirect1 block
                if b1 > 0 {
                    v.push(indirect2[a1]);
                    get_block_cache(
                        indirect2[a1] as usize, 
                        Arc::clone(block_device)
                    )
                        .lock()
                        .modify(0, |indirect1: &mut IndirectBlock| {
                            for entry in indirect1.iter().take(b1) {
                                v.push(*entry);
                            }
                        });
                }
            });
        self.indirect2 = 0;
        v
    }

    /// read data from current disk inode to the given buffer
    pub fn read_at(&self, offset: usize, buf: &mut [u8], block_device: &Arc<dyn BlockDevice>) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize); // make sure not exceeding either buffer len or inode size
        if start >= end {
            return 0;
        }
        let mut start_block = start / BLOCK_SZ;
        let mut read_size = 0usize;
        loop {
            // calculate end of current block, unit: byte
            let mut end_current_block = (start / BLOCK_SZ + 1) * BLOCK_SZ;
            end_current_block = end_current_block.min(end);
            // read and update read size
            let block_read_size = end_current_block - start;
            let dst = &mut buf[read_size..read_size + block_read_size];
            // get_block_id could help get the id of content data_block
            get_block_cache(
                self.get_block_id(start_block as u32, block_device) as usize, 
                Arc::clone(block_device)
            )
                .lock()
                .read(0, |data_block: &DataBlock| {
                    let src = &data_block[start % BLOCK_SZ..start % BLOCK_SZ + block_read_size];
                    dst.copy_from_slice(src);
                });
            read_size += block_read_size;
            // move to next block
            if end_current_block == end {
                // this means it reaches either the end of inode or the end of buffer
                // it should break immediately
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        read_size
    }

    /// write data from the buffer into current disk inode, similar to read_at
    pub fn write_at(&mut self, offset: usize, buf: &mut [u8], block_device: &Arc<dyn BlockDevice>) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        assert!(start <= end);
        let mut start_block = start / BLOCK_SZ;
        let mut write_size = 0usize;
        loop {
            // calculate end of current block
            let mut end_current_block  = (start / BLOCK_SZ + 1) * BLOCK_SZ;
            end_current_block = end_current_block.min(end);
            // write and update write size
            let block_write_size = end_current_block - start;
            get_block_cache(
                self.get_block_id(start_block as u32, block_device) as usize, 
                Arc::clone(block_device)
            )
                .lock()
                .modify(0, |data_block: &mut DataBlock| {
                    let src = &buf[write_size..write_size + block_write_size];
                    let dst = &mut data_block[start % BLOCK_SZ..start % BLOCK_SZ + block_write_size];
                    dst.copy_from_slice(src);
                });
            write_size += block_write_size;
            // move to next block
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        write_size
    }
}