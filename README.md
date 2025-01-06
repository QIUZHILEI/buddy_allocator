# buddy_allocator 伙伴分配器

**基于位图和链队列的物理页式内存分配器**

伙伴分配器主要由两部分组成：一是表示内存块空闲状态的位图，二是用于分配和回收的链队列。伙伴系统按块对内存进行管理，其中 *order* 表示块大小的级别。一个块的大小是页大小的 2 的 *order* 次幂。最小的块对应 *order = 0*，通常为 4KB（即一个页）；最大的块可以根据需要调整。

## 原理

![位图队列伙伴系统](assets/alloc.png)

链队列用于维护可用的内存块，将空闲块连接起来，链队列记录了头节点和尾结点，节点中有一个前驱节点地址和一个后驱节点地址。链队列的数量由 *order* 决定，例如，当 *order = 10* 时，对应 0 到 10 共 11 个队列。这些队列的头结点按顺序存储在内核的 `.data` 段中。

位图用于表示内存块的空闲状态，其中每个位表示一个块的状态。一个字节通常包含 8 位，每个位设置为 `1` 表示对应的块已分配，设置为 `0` 表示空闲。位图的数量与链队列的数量相同，所有位图依次存储在链队列头节点之后。
一个位图的大小取决于内存的总大小和块的大小。例如，假设内存总大小为 4GB，块大小为 4MB，则位图的大小为：`内存大小/块大小/8 = 4G/4M/8 = 2^7^ = 128字节`，这说明当内存全部为order = 10的块时，需要占用 128B 的内存空间来存储位图表示内存块的空闲状态。

### 初始化

在伙伴系统初始化时，首先根据所有队列头节点的末地址，计算每个 *order* 对应的位图所需的空间。接着，将内核代码、数据以及位图本身所占用的内存块标记为已分配状态，其余的位均设置为空闲状态。位图设置完成后，将其末尾地址对齐到最小页的边界，随后开始初始化链队列。

链队列的初始化逻辑是尽可能将所有内存按最大 *order* 的块大小进行分块，并将这些块添加到对应的链队列中。

在本图中，最大 *order* 为 10，即每块大小为 4MB。理想情况下，如果位图末地址对齐到最小页的地址（例如 *addr_A*）同时也能对齐到 4MB 边界（例如 *addr_B*），则无需额外调整。但实际中，这种情况较少发生。当 *addr_A* 与 *addr_B* 之间存在未对齐的内存时，需要逐级寻找合适大小的块（从 *order = 9*，即 2MB 开始，逐级递减），并将这些块添加到对应的链队列中。最终，从 *addr_B* 开始，所有剩余内存按最大块大小（*order = 10*）分块并添加到对应的链队列中。

### 内存分配

当用户调用内存分配函数时，按照给定的大小分配块，不足最小块的申请，会分配一个最小块，超出最大块的申请，会引发内存溢出错误。

开始分配内存时，首先从最合适的order对应的队列队尾开始进行出队操作，如果出队成功，将出队的块地址在位图中标记为已分配，并返回一个块，否则就需要分割一个块：

1. 从order+1队列开始寻找一个非空的队列，如果order大于最大值表示没有可用内存，返回空地址
2. 找到了一个非空队列，出队一个块，用位图标记这个块已分配
3. 对出队的块进行分裂操作，条件为order >= 需求大小对应的order
   1. 新块 = 出队的地址+当前order对应的块大小
   2. 新块添加到对应order的队列中
   3. 标记新块为空闲状态
4. 返回分裂得来的块

### 内存释放

用户调用内存释放函数时，按照给定的块信息（地址和大小……），进行块的合并操作：

1. for order in 给定的块大小对应的order  ..  max order
2. 获取当前order对应的块大小，并找到伙伴块的地址
3. 如果当前order对应的队列中伙伴块存在，并且order<max_order
   - 则在队列中删除节点（用地址直接获取节点，无需遍历，时间复杂度为O(1)），并在位图中标记对应的位为已分配状态，然后取伙伴地址和当前要合并的地址中最小的当作下一次要合并的地址，返回到1继续执行
   - 否则执行进队操作，并且在位图中标记对应的位位空闲状态，合并操作完毕

## 使用

分配器没有提供互斥的操作，如果是多线程情况下使用该分配器，需要用户自行实现互斥的操作。

该分配器实现的是内存管理功能，需要依赖`lego_mem`核心组件库，引入对应的依赖项：

```toml
lego_mem = { git = "https://github.com/lego-os/lego_mem.git", branch = "main" }
buddy_allocator = { git = "https://github.com/QIUZHILEI/buddy_allocator.git", branch = "main" }
```

使用方式：

```rust
// mem.rs
use buddy_allocator::BuddyAllocator;
use lego_mem::{Align, AllocError, ApFlags, Page, PageAllocator};
static mut ALLOCATOR: BuddyAllocator = BuddyAllocator::new();

pub fn init(mem_start: usize, total_size: usize, page_start: usize) {
    let alloc = unsafe { (&raw mut ALLOCATOR).as_mut().unwrap() };
    alloc.init(mem_start, total_size, page_start);
}

#[allow(unused)]
pub fn alloc_pages(align: Align) -> Result<Page, AllocError> {
    let alloc = unsafe { (&raw mut ALLOCATOR).as_mut().unwrap() };
    alloc.alloc_pages(ApFlags::default(), align)
}

#[allow(unused)]
pub fn free_pages(page: Page) -> Result<(), AllocError> {
    let alloc = unsafe { (&raw mut ALLOCATOR).as_mut().unwrap() };
    alloc.free_pages(page)
}

// lib.rs
const MEM_START: usize = 0x40000000;
const MEM_SIZE: usize = 4 * 1024 * 1024 * 1024;

mod mem;
pub fn main() {
    mem::init(MEM_START, MEM_SIZE, kernel_end);
	let page = mem::alloc_pages(Align::K4).unwrap();
    let addr = page.addr;
    mem::free_pages(page);
}

```

