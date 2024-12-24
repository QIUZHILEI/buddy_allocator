#![no_std]
#![feature(generic_const_exprs)]
mod alloc;
mod map;
mod queue;

pub use alloc::BuddyAllocator;
