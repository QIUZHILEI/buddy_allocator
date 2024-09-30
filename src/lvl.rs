use core::ptr::{self, NonNull};

use crate::node::{Block, BLOCK_STRUCT_SIZE};

pub(crate) struct LevelVec {
    bottom: NonNull<Block>,
    max_level: usize,
}

impl LevelVec {
    pub unsafe fn new(addr: usize, level: usize) -> Self {
        Self {
            bottom: NonNull::new_unchecked(addr as *mut usize as *mut Block),
            max_level: level,
        }
    }
    pub const unsafe fn dangling() -> Self {
        Self {
            bottom: NonNull::dangling(),
            max_level: 0,
        }
    }

    #[inline]
    pub fn empty(&self) -> bool {
        let head = self.bottom.as_ptr();
        if head == ptr::null_mut() {
            true
        } else {
            unsafe { (*head).next == ptr::null_mut() }
        }
    }

    pub fn from_addr(addr:usize,block_index:usize){
        
    }
}


impl IntoIterator for LevelVec {
    type Item = *mut Block;

    type IntoIter = LevelNodeIter;

    fn into_iter(self) -> Self::IntoIter {
        LevelNodeIter {
            tmp_level: 0,
            max_level: self.max_level,
            nodes: self.bottom,
        }
    }
}

pub(crate) struct LevelNodeIter {
    tmp_level: usize,
    max_level: usize,
    nodes: NonNull<Block>,
}

impl Iterator for LevelNodeIter {
    type Item = *mut Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.tmp_level >= self.max_level {
            None
        } else {
            let base = self.nodes.as_ptr() as usize;
            let offset = self.tmp_level * BLOCK_STRUCT_SIZE;
            self.tmp_level += 1;
            Some(unsafe { Block::from_addr(base + offset) })
        }
    }
}
