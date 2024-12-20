#![no_std]
mod alloc;
mod bit_range;
mod queue;
pub use alloc::{BuddyAllocator, MAX_ORDER};
use bit_range::BitRange;
use queue::PageQueue;
