pub(crate) const BLOCK_STRUCT_SIZE: usize = size_of::<Block>();

pub(crate) struct Block {
    pub next: *mut Block,
}

impl Block {
    #[inline]
    pub unsafe fn from_addr(address: usize) -> *mut Self {
        address as *mut usize as *mut Self
    }

    #[inline]
    pub fn level(&self,page_size:usize) -> usize {
        let addr = self as *const Self as usize;
        let offset = addr % page_size;
        offset / BLOCK_STRUCT_SIZE
    }
}
