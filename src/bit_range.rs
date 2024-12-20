use core::alloc::Layout;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct BitRange {
    start_addr: usize,
    len: usize,
    size: usize,
}

impl BitRange {
    #[inline]
    pub(crate) const fn new() -> Self {
        Self {
            start_addr: 0,
            len: 0,
            size: 0,
        }
    }

    #[inline]
    pub(crate) fn range(&mut self, start_addr: usize, len: usize, align: usize) {
        self.len = len;
        self.start_addr = start_addr;
        let size = if len % u8::BITS as usize != 0 {
            len / u8::BITS as usize + 1
        } else {
            len / u8::BITS as usize
        };
        let align_size = Layout::from_size_align(size, align).unwrap();
        self.size = align_size.pad_to_align().size();
    }

    #[inline]
    pub(crate) fn get_bit(&self, index: usize) -> bool {
        if index >= self.len {
            return true;
        }
        let mask = (0b1 << (index % u8::BITS as usize)) as u8;
        let addr = index / u8::BITS as usize + self.start_addr;
        let ptr = addr as *mut u8;
        unsafe { (*ptr) & mask != 0 }
    }

    pub(crate) fn set_bit(&self, index: usize, bit: bool) {
        assert!(index < self.len);
        let mask = (bit as usize) << (index % u8::BITS as usize);
        let addr = index / u8::BITS as usize + self.start_addr;
        let ptr = addr as *mut u8;
        unsafe {
            ptr.write((*ptr) | mask as u8);
        }
    }

    #[inline]
    pub(crate) fn size(&self) -> usize {
        self.size
    }
}
