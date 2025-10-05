use super::heap_allocator::HeapAllocator;

pub struct RingBufferAllocator<T> {
    blocks: HeapAllocator<Block<T>>,
    block_size: usize,
}

pub struct RingBuffer {
    start_id: usize,
    start_idx: usize,
    end_id: usize,
    end_idx: usize,
    len: usize,
}

struct Block<T> {
    values: Vec<T>,
    next_id: usize,
}

impl<T> RingBufferAllocator<T> {
    pub fn new_buffer(&mut self) -> RingBuffer {
        RingBuffer {
            start_id: usize::MAX,
            start_idx: usize::MAX,
            end_id: usize::MAX,
            end_idx: usize::MAX,
            len: 0,
        }
    }
}

impl RingBuffer {
    pub fn extend<T>(
        &mut self,
        alloc: &mut RingBufferAllocator<T>,
        values: impl Iterator<Item = T>,
    ) {
        let mut iter = values.into_iter();
        loop {
            if self.end_idx >= alloc.block_size {
                let new_id = alloc.blocks.push(Block {
                    values: Vec::new(),
                    next_id: usize::MAX,
                });
                if self.end_id != usize::MAX {
                    alloc.blocks.get_mut(self.end_id).next_id = new_id;
                }
                self.end_id = new_id;
                self.end_idx = 0;
                if self.start_id == usize::MAX {}
            }
            let end_block = alloc.blocks.get_mut(self.end_id);

            let n_open = alloc.block_size - end_block.values.len();
            end_block.values.extend(iter.by_ref().take(n_open));
            self.end_idx = end_block.values.len();
            if end_block.values.len() < alloc.block_size {
                break;
            }
        }
    }
}
