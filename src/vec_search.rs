/// this module contains some search methods on vectors

use uint::u40;
use std::collections::BTreeSet;

///binary search predecessor
pub fn rust_bin_search_pred(element_list: &Vec<u40>, query: u40) -> Option<u40> {
    let pos = match element_list.binary_search(&query) {
        Ok(pos) => pos,
        Err(pos) => pos
    };
    if pos > 0 {
        Some(element_list[pos - 1])
    } else {
        None
    }
}

/// linear search predecessor
pub fn linear_search_pred(element_list: &Vec<u40>, query: u40) -> Option<u40> {
    let mut pos = 0;
    unsafe {
        while  pos < element_list.len() && element_list.get_unchecked(pos) < &query {
            pos += 1;
        }
    }
    if pos > 0 {
        unsafe {
            Some(*element_list.get_unchecked(pos - 1))
        }
    } else {
        None
    }
}

/// binary combined with linear search for predecessor
/// bin_search_steps number of steps that should be done with binary search
pub fn mixed_search_pred(element_list: &Vec<u40>, query: u40, bin_search_steps: usize) -> Option<u40> {
    unsafe {
        let mut l = 0;
        let mut r = element_list.len();
        for _ in 0..bin_search_steps {
            let m = (l + r) / 2;
            if element_list.get_unchecked(m) < &query {
                l = m;
            } else {
                r = m;
            }
        }
        while l < element_list.len() && element_list.get_unchecked(l) < &query {
            l += 1;
        }
        if l > 0 {
            Some(*element_list.get_unchecked(l - 1))
        } else {
            None
        }
    }
}

///search predecessor with BTree
pub fn btree_search_pred(set: &BTreeSet<usize>, query: usize) -> Option<usize> {
    Some(*set.range(0..query).last()?)
}