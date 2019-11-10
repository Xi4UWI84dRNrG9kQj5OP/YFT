use uint::u40;
use std::collections::BTreeSet;

///binary search predecessor
pub fn bin_search_pred(element_list: &Vec<u40>, element: u40) -> Option<u40> {
    let pos = match element_list.binary_search(&element) {
        Ok(pos) => pos,
        Err(pos) => pos
    };
    if pos > 0 {
        Some(element_list[pos - 1])
    } else {
        None
    }
}

///search predecessor with BTree
pub(crate) fn btree_search_pred(set: &BTreeSet<usize>, element: usize) -> Option<usize> {
    Some(*set.range(0..element).last()?)
}