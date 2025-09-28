use std::mem::SizedTypeProperties;

pub trait ByteVecTrait {
    fn push_u8(&mut self, value: u8) -> u64;
    fn get_u8(&self, idx: u64) -> u8;

    fn push_u64(&mut self, value: u64) -> u64;
    fn get_u64(&self, idx: u64) -> u64;
    fn set_u64(&mut self, idx: u64, value: u64);

    fn push_i64(&mut self, value: i64) -> u64;
    fn get_i64(&self, idx: u64) -> i64;

    fn push_f64(&mut self, value: f64) -> u64;
    fn get_f64(&self, idx: u64) -> f64;

    fn get_slice_iterator<'a>(&'a self, start_idx: u64, len: u64) -> ByteVecSliceIterator<'a>;
    fn get_mut_slice<T>(&self, idx: u64, len: u64) -> &mut [T];
    fn pad_for_value<T>(&mut self);

    fn len(&self) -> u64;
}

pub struct ByteVec {
    len: usize,
    capacity: usize,
    ptr: *const u8,
}

impl ByteVec {
    pub fn new() -> Self {
        ByteVec {
            len: 0,
            capacity: 0,
            ptr: std::ptr::null(),
        }
    }

    fn _push_value<T>(&mut self, value: T) -> u64 {
        let layout = T::LAYOUT;

        let n_new_bytes = layout.size();

        // Here's the old padding logic, trying to not pad
        // // 1000 -> 1111
        // let mask = (1 << (layout.align().trailing_zeros() + 1)) - 1;
        // let remainder = self.len & mask;
        // let mut padding: usize = 0;
        // // if remainder != 0 {
        // //     padding = layout.align() - remainder;
        // //     n_new_bytes += padding;
        // // }

        self._increase_capacity_to_store(self.len + n_new_bytes);

        let offset = self.len;

        unsafe {
            (((self.ptr as usize) + offset) as *mut T).write_unaligned(value);
        }
        self.len += n_new_bytes;

        offset as u64
    }

    fn _increase_capacity_to_store(&mut self, new_len: usize) {
        if new_len > self.capacity {
            let new_capacity = std::cmp::max(new_len, self.capacity * 2);
            self.ptr = unsafe {
                std::alloc::realloc(
                    self.ptr as *mut u8,
                    std::alloc::Layout::array::<u8>(self.capacity).unwrap(),
                    new_capacity,
                )
            };
            self.capacity = new_capacity;
        }
    }

    /*
    Some values like u64 need a memory address that is a multiple
    of the size.

    This function adds padding after the previous value to make sure
    the new value is aligned.
     */
    fn _push_aligned_value<T>(&mut self, value: T) -> u64 {
        let layout = T::LAYOUT;

        let mut n_new_bytes = layout.size();

        // 1000 -> 1111
        let mask = (1 << (layout.align().trailing_zeros() + 1)) - 1;
        let remainder = self.len & mask;
        let mut padding: usize = 0;
        if remainder != 0 {
            padding = layout.align() - remainder;
            n_new_bytes += padding;
        }

        self._increase_capacity_to_store(self.len + n_new_bytes);

        let offset = self.len + padding;

        unsafe {
            (((self.ptr as usize) + offset) as *mut T).write(value);
        }
        self.len += n_new_bytes;

        offset as u64
    }

    fn _get_value<T>(&self, idx: u64) -> T {
        unsafe { (((self.ptr as usize) + (idx as usize)) as *const T).read_unaligned() }
    }

    fn _set_value<T>(&self, idx: u64, value: T) {
        unsafe { (((self.ptr as usize) + (idx as usize)) as *mut T).write_unaligned(value) }
    }
}

impl Drop for ByteVec {
    fn drop(&mut self) {
        unsafe {
            std::alloc::dealloc(
                (self.ptr as usize) as *mut u8,
                std::alloc::Layout::array::<u8>(self.capacity).unwrap(),
            );
        }
    }
}

impl ByteVecTrait for ByteVec {
    fn push_u8(&mut self, value: u8) -> u64 {
        self._push_value(value)
    }

    fn get_u8(&self, idx: u64) -> u8 {
        self._get_value::<u8>(idx)
    }

    fn push_u64(&mut self, value: u64) -> u64 {
        self._push_value(value)
    }

    fn get_u64(&self, idx: u64) -> u64 {
        self._get_value(idx)
    }

    fn set_u64(&mut self, idx: u64, value: u64) {
        self._set_value(idx, value);
    }

    fn push_i64(&mut self, value: i64) -> u64 {
        self._push_value(value)
    }

    fn get_i64(&self, idx: u64) -> i64 {
        self._get_value(idx)
    }

    fn push_f64(&mut self, value: f64) -> u64 {
        self._push_value(value)
    }

    fn get_f64(&self, idx: u64) -> f64 {
        self._get_value(idx)
    }

    fn get_slice_iterator<'a>(&'a self, start_idx: u64, len: u64) -> ByteVecSliceIterator<'a> {
        ByteVecSliceIterator {
            byte_vec: self,
            start_idx,
            len,
        }
    }

    fn get_mut_slice<T>(&self, idx: u64, len: u64) -> &mut [T] {
        // Make sure start index is aligned
        assert!((idx as usize) % T::LAYOUT.align() == 0);

        unsafe {
            std::slice::from_raw_parts_mut(
                ((self.ptr as usize) + (idx as usize)) as *mut T,
                len as usize,
            )
        }
    }

    fn pad_for_value<T>(&mut self) {
        let layout = T::LAYOUT;

        // 1000 -> 1111
        let mask = (1 << (layout.align().trailing_zeros())) - 1;
        let remainder = self.len & mask;
        let mut padding: usize = 0;
        if remainder != 0 {
            padding = layout.align() - remainder;
        }

        self._increase_capacity_to_store(self.len + padding);
        self.len += padding;
    }

    fn len(&self) -> u64 {
        self.len as u64
    }
}

pub struct ByteVecSliceIterator<'a> {
    byte_vec: &'a ByteVec,
    start_idx: u64,
    len: u64,
}

impl<'a> Iterator for ByteVecSliceIterator<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if self.len <= 0 {
            return None;
        }

        let slice = unsafe {
            std::slice::from_raw_parts(
                ((self.byte_vec.ptr as usize) + (self.start_idx as usize)) as *const u8,
                self.len as usize,
            )
        };
        self.start_idx += self.len;
        self.len = 0;

        Some(slice)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic() {
        let mut byte_vec = ByteVec::new();
        assert_eq!(byte_vec.len(), 0);

        byte_vec.push_u8(1);
        byte_vec.push_u8(2);
        byte_vec.push_u8(3);
        assert_eq!(byte_vec.len(), 3);
        assert_eq!(byte_vec.get_u8(0), 1);
        assert_eq!(byte_vec.get_u8(1), 2);
        assert_eq!(byte_vec.get_u8(2), 3);

        let big_n0 = byte_vec.push_u64(u64::MAX);
        assert_eq!(big_n0, 3);
        assert_eq!(byte_vec.len(), 11);
        assert_eq!(byte_vec.get_u64(big_n0), u64::MAX);
        byte_vec.set_u64(big_n0, 42);
        assert_eq!(byte_vec.len(), 11);
        assert_eq!(byte_vec.get_u64(big_n0), 42);

        let big_n1 = byte_vec.push_i64(i64::MIN);
        let big_n2 = byte_vec.push_i64(i64::MAX);
        assert_eq!(big_n1, 11);
        assert_eq!(big_n2, 19);

        assert_eq!(byte_vec.len(), 27);
        assert_eq!(byte_vec.get_i64(big_n1), i64::MIN);
        assert_eq!(byte_vec.get_i64(big_n2), i64::MAX);
    }

    #[test]
    fn test_aligned_slice() {
        for n_leading_bytes in 0..7 {
            let mut byte_vec = ByteVec::new();

            for _ in 0..n_leading_bytes {
                byte_vec.push_u8(1);
            }
            byte_vec.pad_for_value::<u64>();
            assert_eq!(byte_vec.len(), if n_leading_bytes == 0 { 0 } else { 8 });
            let slice_start = byte_vec.len();
            byte_vec.push_u64(100);
            byte_vec.push_u64(101);
            byte_vec.push_u64(102);
            byte_vec.push_u64(103);

            let slice = byte_vec.get_mut_slice::<u64>(slice_start, 4);
            assert_eq!(slice.len(), 4);
            assert_eq!(slice[0], 100);
            assert_eq!(slice[1], 101);
            assert_eq!(slice[2], 102);
            assert_eq!(slice[3], 103);

            slice[1] = 201;

            assert_eq!(byte_vec.get_u64(slice_start + 8), 201);
        }
    }

    #[test]
    fn test_align_repro_error() {
        let mut byte_vec = ByteVec::new();
        for _ in 0..9 {
            byte_vec.push_u8(1);
        }
        byte_vec.pad_for_value::<u64>();
        assert_eq!(byte_vec.len(), 16);
    }
}
