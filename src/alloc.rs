use crate::{map::MemBitMap, queue::PageQueue};
use core::cmp;
use lego_mem::{Align, AllocError, ApFlags, Page, PageAllocator};

pub const MAX_ORDER: usize = 10;
#[derive(Debug, Default)]
pub struct BuddyAllocator {
    page_queues: [PageQueue; MAX_ORDER + 1],
    bit_maps: [MemBitMap; MAX_ORDER + 1],
    mem_start: usize,
    total_size: usize,
    available_size: usize,
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            mem_start: 0,
            total_size: 0,
            available_size: 0,
            page_queues: [PageQueue::new(); MAX_ORDER + 1],
            bit_maps: [MemBitMap::new(); MAX_ORDER + 1],
        }
    }

    pub fn init(&mut self, mem_start: usize, total_size: usize, start_addr: usize) {
        assert!(mem_start % Self::MIN_PAGE_SIZE == 0);
        assert!(total_size % Self::MIN_PAGE_SIZE == 0);
        self.mem_start = mem_start;
        self.total_size = total_size;
        self.init_bitmap(start_addr);
        let mut page_start = self.bit_maps.map(|map| map.size()).iter().sum::<usize>() + start_addr;
        if page_start % Self::MIN_PAGE_SIZE != 0 {
            page_start = (page_start / Self::MIN_PAGE_SIZE + 1) * Self::MIN_PAGE_SIZE;
        }
        self.available_size = self.mem_start + self.total_size - page_start;
        self.init_queues(page_start);
    }

    fn init_bitmap(&mut self, start_addr: usize) {
        let mut addr = start_addr;
        self.bit_maps
            .iter_mut()
            .enumerate()
            .for_each(|(order, map)| {
                let page_size: usize = 1 << (order + Self::MIN_PAGE_SIZE.trailing_zeros() as usize);
                let page_num = self.total_size / page_size;
                map.init(addr, page_num);
                addr += map.size();
            });
    }

    fn init_queues(&mut self, start_addr: usize) {
        let mut page_addr = start_addr;
        let max_page_size = 1 << (MAX_ORDER + Self::MIN_PAGE_SIZE.trailing_zeros() as usize);
        if page_addr % max_page_size != 0 {
            let limit = page_addr;
            page_addr =
                ((page_addr - self.mem_start) / max_page_size + 1) * max_page_size + self.mem_start;
            let mut addr_top = page_addr;
            let mut order = MAX_ORDER - 1;
            while addr_top > limit {
                let page_size: usize = 1 << (order + Self::MIN_PAGE_SIZE.trailing_zeros() as usize);
                let addr = addr_top - page_size;
                if addr < limit {
                    order -= 1;
                    continue;
                }
                self.page_queues[order].enqueue(addr);
                self.bit_maps[order].clear_bit(self.page_offset(addr, order));
                addr_top = addr;
            }
        }
        while page_addr < self.mem_start + self.total_size {
            self.page_queues[MAX_ORDER].enqueue(page_addr);
            // info!("index: {}",self.page_offset(page_addr, MAX_ORDER));
            self.bit_maps[MAX_ORDER].clear_bit(self.page_offset(page_addr, MAX_ORDER));
            page_addr += max_page_size;
        }
    }

    fn split(&mut self, down_limit: usize) -> usize {
        let mut order = down_limit + 1;
        loop {
            if order > MAX_ORDER {
                return 0;
            }
            if !self.page_queues[order].empty() {
                break;
            }
            order += 1;
        }
        let page_addr = self.page_queues[order].dequeue();
        self.bit_maps[order].set_bit(self.page_offset(page_addr, order));
        for ord in (down_limit..order).rev() {
            let page_size = 1 << (ord + Self::MIN_PAGE_SIZE.trailing_zeros() as usize);
            let new_page = page_addr + page_size;
            self.page_queues[ord].enqueue(new_page);
            self.bit_maps[ord].clear_bit(self.page_offset(new_page, ord));
        }
        page_addr
    }

    fn merge(&mut self, start_order: usize, page_addr: usize) {
        let mut page_addr = page_addr;
        for order in start_order..self.page_queues.len() {
            let page_size = 1 << (order + Self::MIN_PAGE_SIZE.trailing_zeros() as usize);
            let buddy_addr = page_addr ^ page_size;
            if !self.bit_maps[order].is_set(self.page_offset(buddy_addr, order))
                && order < MAX_ORDER
            {
                self.page_queues[order].remove(buddy_addr);
                self.bit_maps[order].set_bit(self.page_offset(buddy_addr, order));
                page_addr = cmp::min(page_addr, buddy_addr);
            } else {
                self.page_queues[order].enqueue(page_addr);
                self.bit_maps[order].clear_bit(self.page_offset(page_addr, order));
                break;
            }
        }
    }

    #[inline]
    fn page_offset(&self, addr: usize, order: usize) -> usize {
        let page_size = 1 << (order + Self::MIN_PAGE_SIZE.trailing_zeros() as usize);
        (addr - self.mem_start) / page_size
    }
}

impl PageAllocator for BuddyAllocator {
    const MIN_PAGE_SIZE: usize = 4096;

    fn alloc_pages(&mut self, _flags: ApFlags, align: Align) -> Result<Page, AllocError> {
        let order = (align.as_power() - Self::MIN_PAGE_SIZE.trailing_zeros()) as usize;
        if order > MAX_ORDER {
            return Err(AllocError::Misaligned);
        }
        let mut page_addr = self.page_queues[order].dequeue();
        if page_addr != 0 {
            self.bit_maps[order].set_bit(self.page_offset(page_addr, order));
        } else {
            page_addr = self.split(order);
        }

        if page_addr == 0 {
            return Err(AllocError::OutOfMemory);
        }

        self.available_size -= align as usize;
        Ok(Page::new(page_addr, align))
    }

    fn free_pages(&mut self, page: Page) -> Result<(), AllocError> {
        let page_addr = page.addr;
        let order = (page.align.as_power() - Self::MIN_PAGE_SIZE.trailing_zeros()) as usize;
        self.merge(order, page_addr);
        self.available_size += page.align.as_size();
        Ok(())
    }
}
