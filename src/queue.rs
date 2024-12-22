#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct PageQueue {
    head: usize,
    tail: usize,
    node_num: usize,
}

impl PageQueue {
    pub(crate) const fn new() -> Self {
        Self {
            head: 0,
            tail: 0,
            node_num: 0,
        }
    }

    pub(crate) fn enqueue(&mut self, node_addr: usize) {
        assert_ne!(node_addr, 0);
        let new_node = unsafe { PageNode::from_addr(node_addr).as_mut().unwrap() };
        let tail_node = PageNode::from_addr(self.tail);
        new_node.next = 0;
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

    pub(crate) fn remove(&mut self, node_addr: usize) {
        assert!(self.node_num > 0);
        let node = unsafe { PageNode::from_addr(node_addr).as_mut().unwrap() };
        let (prev, prev_node) = (node.prev, PageNode::from_addr(node.prev));
        let (next, next_node) = (node.next, PageNode::from_addr(node.next));
        node.next = 0;
        node.prev = 0;
        if !prev_node.is_null() {
            unsafe { (*prev_node).next = next }
        } else {
            self.head = next;
        }
        if !next_node.is_null() {
            unsafe { (*next_node).prev = prev }
        } else {
            self.tail = prev;
        }
        self.node_num -= 1;
    }
}

struct PageNode {
    prev: usize,
    next: usize,
}

impl PageNode {
    #[inline]
    const fn from_addr(addr: usize) -> *mut Self {
        addr as *mut usize as *mut PageNode
    }
}
