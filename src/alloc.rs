use crate::{BitRange, PageQueue};
use core::{alloc::Layout, cmp};
use lego_mem::{AllocError, ApFlags, Page, PageAllocator, PageLayout};

pub const MAX_ORDER: usize = 10;
#[derive(Debug, Default)]
pub struct BuddyAllocator {
    page_queues: [PageQueue; MAX_ORDER + 1],
    bit_range: BitRange,
    mem_start: usize,
    available_start: usize,
    total_size: usize,
    available_size: usize,
}

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self {
            page_queues: [PageQueue::new(); MAX_ORDER + 1],
            bit_range: BitRange::new(),
            mem_start: 0,
            available_start: 0,
            total_size: 0,
            available_size: 0,
        }
    }

    pub fn init(&mut self, mem_start: usize, total_size: usize, page_start: usize) {
        assert_eq!(page_start % Self::MIN_PAGE_SIZE, 0);
        self.mem_start = mem_start;
        self.total_size = total_size;
        let bit_range_len = self.total_size / Self::MIN_PAGE_SIZE;
        self.bit_range
            .range(page_start, bit_range_len, Self::MIN_PAGE_SIZE);

        self.available_start = page_start + self.bit_range.size();
        self.available_size = self.mem_start + self.total_size - self.available_start;

        self.init_queues(self.available_start, mem_start + total_size);
    }

    fn init_queues(&mut self, start_addr: usize, end_addr: usize) {
        let mut page_addr = start_addr;
        self.page_queues
            .iter_mut()
            .enumerate()
            .for_each(|(order, queue)| {
                queue.set_page_size(0b1 << (order + Self::MIN_PAGE_SIZE.trailing_zeros() as usize))
            });

        let mut order = MAX_ORDER;
        while page_addr < end_addr {
            let next_addr =
                page_addr + (1 << (order + Self::MIN_PAGE_SIZE.trailing_zeros() as usize));
            if next_addr > end_addr {
                order -= 1;
            } else {
                self.page_queues[order].enqueue(page_addr);
                page_addr = next_addr;
            }
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
        for ord in (down_limit..order).rev() {
            let new_page = page_addr + self.page_queues[ord].page_size();
            self.page_queues[ord].enqueue(new_page);
        }
        page_addr
    }

    fn merge(&mut self, start_order: usize, page_addr: usize) {
        let mut page_addr = page_addr;
        for order in start_order..self.page_queues.len() {
            let buddy_addr = page_addr ^ self.page_queues[order].page_size();
            if !self
                .bit_range
                .get_bit(self.calculate_page_index(page_addr, Self::MIN_PAGE_SIZE))
                && self.page_queues[order].in_queue(buddy_addr)
                && order < MAX_ORDER
            {
                self.page_queues[order].remove(buddy_addr);
                page_addr = cmp::min(page_addr, buddy_addr);
            } else {
                self.page_queues[order].enqueue(page_addr);
                break;
            }
        }
    }

    #[inline]
    fn calculate_page_index(&self, addr: usize, page_size: usize) -> usize {
        (addr - self.mem_start) / page_size
    }
}

impl PageAllocator for BuddyAllocator {
    const MIN_PAGE_SIZE: usize = 4096;

    fn alloc_pages(&mut self, flags: ApFlags, layout: PageLayout) -> Result<Page, AllocError> {
        let align = layout.align();
        let order = (align.as_power() - Self::MIN_PAGE_SIZE.trailing_zeros()) as usize;
        if order > MAX_ORDER {
            return Err(AllocError::Misaligned);
        }
        let page_addr = if self.page_queues[order].empty() {
            self.split(order)
        } else {
            self.page_queues[order].dequeue()
        };

        if page_addr == 0 {
            return Err(AllocError::OutOfMemory(
                Layout::array::<u8>(align as usize).unwrap(),
            ));
        }

        self.bit_range.set_bit(
            self.calculate_page_index(page_addr, Self::MIN_PAGE_SIZE),
            true,
        );
        self.available_size -= align as usize;
        Ok(Page {
            layout,
            flags,
            addr: page_addr,
            access: 0,
        })
    }

    fn free_pages(&mut self, page: Page) -> Result<(), AllocError> {
        let page_addr = page.addr;
        let order =
            (page.layout.align().as_power() - Self::MIN_PAGE_SIZE.trailing_zeros()) as usize;
        self.merge(order, page_addr);
        self.bit_range.set_bit(
            self.calculate_page_index(page_addr, Self::MIN_PAGE_SIZE),
            false,
        );
        self.available_size += page.layout.align() as usize;
        Ok(())
    }
}
