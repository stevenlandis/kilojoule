pub struct HeapAllocator<T> {
    values: Vec<Option<T>>,
    free_heap: Vec<usize>,
}

impl<T> HeapAllocator<T> {
    pub fn new() -> Self {
        HeapAllocator {
            values: Vec::new(),
            free_heap: Vec::new(),
        }
    }

    pub fn push(&mut self, value: T) -> usize {
        match self.pop_max_idx() {
            Some(idx) => {
                assert!(self.values[idx].is_none());
                self.values[idx] = Some(value);
                idx
            }
            None => {
                let idx = self.values.len();
                self.values.push(Some(value));
                idx
            }
        }
    }

    pub fn get(&self, idx: usize) -> &T {
        match &self.values[idx] {
            None => unreachable!(),
            Some(value) => &value,
        }
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut T {
        match &mut self.values[idx] {
            None => unreachable!(),
            Some(value) => value,
        }
    }

    pub fn free(&mut self, idx: usize) {
        assert!(self.values[idx].is_some());
        self.values[idx] = None;
        self.push_idx(idx);
    }
}

impl<T> HeapAllocator<T> {
    pub fn push_idx(&mut self, value: usize) {
        self.free_heap.push(value);
        self.heapify_up(self.free_heap.len() - 1);
    }

    pub fn pop_max_idx(&mut self) -> Option<usize> {
        if self.free_heap.is_empty() {
            return None;
        }

        let max = self.free_heap[0];
        let last = self.free_heap.pop().unwrap();

        if !self.free_heap.is_empty() {
            self.free_heap[0] = last;
            self.heapify_down(0);
        }

        Some(max)
    }

    fn heapify_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent_index = (index - 1) / 2;

            if self.free_heap[index] < self.free_heap[parent_index] {
                self.free_heap.swap(index, parent_index);
                index = parent_index;
            } else {
                break;
            }
        }
    }

    fn heapify_down(&mut self, mut index: usize) {
        let len = self.free_heap.len();
        loop {
            let left = 2 * index + 1;
            let right = 2 * index + 2;
            let mut smallest = index;

            if left < len && self.free_heap[left] < self.free_heap[smallest] {
                smallest = left;
            }

            if right < len && self.free_heap[right] < self.free_heap[smallest] {
                smallest = right;
            }

            if smallest != index {
                self.free_heap.swap(index, smallest);
                index = smallest;
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic() {
        let mut alloc = HeapAllocator::<String>::new();
        let i0 = alloc.push("v0".to_string());
        assert_eq!(i0, 0);
        let i1 = alloc.push("v1".to_string());
        assert_eq!(i1, 1);
        let i2 = alloc.push("v2".to_string());
        assert_eq!(i2, 2);

        assert_eq!(alloc.get(i0), "v0");
        assert_eq!(alloc.get(i1), "v1");
        assert_eq!(alloc.get(i2), "v2");

        alloc.free(i0);
        let i3 = alloc.push("v3".to_string());
        assert_eq!(i3, 0);
        assert_eq!(alloc.get(i3), "v3");

        alloc.free(i3);
        alloc.free(i1);

        let i4 = alloc.push("v4".to_string());
        let i5 = alloc.push("v5".to_string());
        assert_eq!(i4, 0);
        assert_eq!(i5, 1);

        assert_eq!(alloc.get(i4), "v4");
        assert_eq!(alloc.get(i5), "v5");
    }
}
