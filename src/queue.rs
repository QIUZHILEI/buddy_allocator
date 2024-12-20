#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct PageQueue {
    head: usize,
    tail: usize,
    page_size: usize,
    node_num: usize,
}

impl PageQueue {
    pub(crate) const fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            page_size: 0,
            node_num: 0,
        }
    }

    #[inline]
    pub(crate) fn set_page_size(&mut self, page_size: usize) {
        self.page_size = page_size;
    }

    #[inline]
    pub(crate) fn page_size(&self) -> usize {
        self.page_size
    }

    pub(crate) fn enqueue(&mut self, node_addr: usize) {
        assert_ne!(node_addr, 0);
        let new_node = unsafe { PageNode::from_addr(node_addr).as_mut().unwrap() };
        let tail_node = PageNode::from_addr(self.tail);
        new_node.size = self.page_size;
        new_node.prev = self.tail;
        if self.empty() {
            self.head = node_addr;
        } else {
            unsafe {
                (*tail_node).next = node_addr;
            }
        }
        self.tail = node_addr;
        self.node_num += 1;
    }

    pub(crate) fn dequeue(&mut self) -> usize {
        if self.empty() {
            return 0;
        }
        let head_addr = self.head;
        let head_node = unsafe { PageNode::from_addr(head_addr).as_mut().unwrap() };
        let new_head = PageNode::from_addr(head_node.next);
        self.head = head_node.next;
        head_node.next = 0;
        if new_head.is_null() {
            self.tail = 0;
        } else {
            unsafe {
                (*new_head).prev = 0;
            }
        }
        self.node_num -= 1;
        head_addr
    }

    #[inline]
    pub(crate) fn empty(&self) -> bool {
        self.node_num == 0
    }

    #[inline]
    pub(crate) fn in_queue(&self, node_addr: usize) -> bool {
        let node = unsafe { PageNode::from_addr(node_addr).as_mut().unwrap() };
        node.size == self.page_size && !self.empty()
    }

    pub(crate) fn remove(&mut self, node_addr: usize) {
        let node = unsafe { PageNode::from_addr(node_addr).as_mut().unwrap() };
        let (prev, prev_node) = (node.prev, PageNode::from_addr(node_addr));
        let (next, next_node) = (node.next, PageNode::from_addr(node.next));
        node.next = 0;
        node.prev = 0;
        if !prev_node.is_null() {
            unsafe { (*prev_node).next = next }
        }
        if !next_node.is_null() {
            unsafe { (*next_node).prev = prev }
        }
        self.node_num -= 1;
        if self.empty() {
            self.head = 0;
            self.tail = 0;
        }
    }
}

struct PageNode {
    prev: usize,
    next: usize,
    size: usize,
}

impl PageNode {
    #[inline]
    const fn from_addr(addr: usize) -> *mut Self {
        addr as *mut usize as *mut PageNode
    }
}
