use core::{alloc::Layout, ptr};

use crate::{logarithmic_two_up, node::BLOCK_STRUCT_SIZE, page_round_up, SkipList};
use tom_memory::{AllocError, PhysicalPageAllocator};
pub(crate) struct BuddyAllocator<const BUDDY_POWER: usize> {
    free_lists: [SkipList; BUDDY_POWER],
    page_size: usize,
    start_addr: usize,
    total_size: usize,
    free_size: usize,
}

impl<const BUDDY_POWER: usize> BuddyAllocator<BUDDY_POWER> {
    pub(crate) const unsafe fn new(page_size: usize) -> Self {
        Self {
            free_lists: [const { SkipList::new() }; BUDDY_POWER],
            start_addr: 0,
            page_size,
            total_size: 0,
            free_size: 0,
        }
    }

    pub unsafe fn init(&mut self, start_addr: usize, size: usize) {
        let mut head_addr = start_addr;
        let min_power = self.page_size.trailing_zeros() as usize;
        self.free_lists
            .iter_mut()
            .enumerate()
            .for_each(|(index, list)| {
                let power = min_power + index;
                let block_size = 1 << power;
                let max_level = logarithmic_two_up(size / block_size);
                list.init(head_addr, block_size, max_level);
                head_addr += max_level * BLOCK_STRUCT_SIZE;
            });
        let end_addr = start_addr + size;
        let start_addr = page_round_up(head_addr, self.page_size);
        let mut current_addr = start_addr;
        while current_addr < end_addr {
            self.free_lists[0].push(current_addr);
            current_addr += self.page_size;
        }
        self.total_size = end_addr - start_addr;
        self.free_size = self.total_size;
        self.start_addr = start_addr;
    }

    fn division_block(&mut self) {

    }

    fn incorporation_block(&mut self) {
        
    }
}

unsafe impl<const BUDDY_POWER: usize> PhysicalPageAllocator for BuddyAllocator<BUDDY_POWER> {
    fn total_size(&self) -> usize {
        self.total_size
    }

    fn free_size(&self) -> usize {
        self.free_size
    }

    unsafe fn alloc_pages(&mut self, layout: Layout) -> Result<*mut u8, AllocError> {
        let layout = layout.align_to(self.page_size)?;
        let size = layout.align();
        let power = logarithmic_two_up(size);
        let pos = power - self.page_size.trailing_zeros() as usize;
        if let Some(ptr) = self.free_lists[pos].pop() {
            return Ok(ptr);
        }
        // for index in pos..self.free_lists.len() {
        //     if let Some(ptr) =
        // }
        // incorporation
        // for index in 0..pos {}
        Ok(ptr::null_mut())
    }

    unsafe fn free_pages(&mut self, ptr: *mut u8, layout: Layout) -> Result<(), AllocError> {
        if ptr as usize >= self.start_addr + self.total_size {
            return Err(AllocError::NullPointer(ptr as usize));
        }
        let offset = ptr as usize - self.start_addr;
        let layout = layout.align_to(self.page_size)?;
        let power = logarithmic_two_up(layout.align());
        let size = 1 << power;
        if offset % size != 0 {
            return Err(AllocError::Misaligned(ptr as usize));
        }
        let pos = power - self.page_size.trailing_zeros() as usize;
        self.free_lists[pos].insert(ptr, offset / size);
        self.free_size += size;
        Ok(())
    }
}
