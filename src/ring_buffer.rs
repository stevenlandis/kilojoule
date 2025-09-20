use super::heap_allocator::HeapAllocator;

pub struct RingBufferAllocator<T> {
    blocks: HeapAllocator<Block<T>>,
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
    next_idx: Option<usize>,
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
        loop {}
    }
}
