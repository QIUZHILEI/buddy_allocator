#![no_std]
#![feature(const_slice_from_raw_parts_mut)]
mod buddy;
mod list;
mod lvl;
mod node;
use list::*;

#[inline]
fn logarithmic_two_up(num: usize) -> usize {
    let trailing_zeros = num.trailing_zeros() as usize;
    if 1 << trailing_zeros == num {
        trailing_zeros
    } else {
        let start_zero = usize::BITS - num.leading_zeros();
        start_zero as usize
    }
}

// fn two_power_down()->usize{

// }

#[inline]
fn page_round_up(addr: usize, page_size: usize) -> usize {
    let power = page_size.trailing_zeros();
    let res_addr = (addr >> power) << power;
    if res_addr == addr {
        addr
    } else {
        res_addr + page_size
    }
}

// fn page_round_down()->usize{

// }
