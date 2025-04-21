use core::any::Any;

/// trait for block devices, which reads and writes data in the unit of blocks
/// any trait is used to identify the real struct in this dynamic background
pub trait BlockDevice: Send + Sync + Any {
    /// read data from block to buffer
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// write data from buffer to block
    fn write_block(&self, block_id: usize, buf: &[u8]);
}