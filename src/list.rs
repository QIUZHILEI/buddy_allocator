use tom_memory::AllocError;

use crate::lvl::LevelVec;

pub(crate) struct SkipList {
    heads: LevelVec,
    max_level: usize,
    block_size: usize,
    num_of_free: usize,
}

impl SkipList {
    pub const unsafe fn new() -> Self {
        Self {
            heads: LevelVec::dangling(),
            block_size: 0,
            num_of_free: 0,
            max_level: 0,
        }
    }

    pub unsafe fn init(&mut self, heads_vec_addr: usize, block_size: usize, max_level: usize) {
        self.heads = LevelVec::new(heads_vec_addr, max_level);
        self.block_size = block_size;
        self.max_level = max_level;
    }

    pub unsafe fn empty(&self) -> bool {
        self.heads.empty()
    }

    pub unsafe fn insert(&mut self, address: *mut u8,block_index: usize) {
        
    }

    pub fn remove(&mut self, start: usize, size: usize) -> Option<*mut u8> {
        todo!()
    }

    pub unsafe fn push(&mut self, addr: usize) {}

    pub unsafe fn pop(&mut self) -> Option<*mut u8> {
        todo!()
    }

    #[inline]
    fn block_index(&self, addr: usize) -> usize {
        let power = self.block_size.trailing_zeros() as usize;
        (!(self.block_size - 1) & addr) >> power
    }

    pub unsafe fn find_continuous_space(&mut self, size: usize) -> Result<*mut u8, AllocError> {
        todo!()
    }
}
