#![feature(allocator_api)]
#![feature(shrink_to)]
extern crate structopt;
#[macro_use]
extern crate bitflags;
extern crate uint;
extern crate stats_alloc;
extern crate gnuplot;


pub use yft::YFT;
//use std::path::PathBuf;
//use structopt::StructOpt;

pub mod yft;
pub mod nmbrsrc;
pub mod memlog;


pub trait PredecessorSet<T> {
    fn insert(&mut self, element: T);
    fn delete(&mut self, element: T);
    fn predecessor(&self, number: T) -> Option<T>;
    fn sucessor(&self, number: T) -> Option<T>;
    // Optional
    fn minimum(&self) -> Option<T>;
    fn maximum(&self) -> Option<T>;
    fn contains(&self, number: T) -> bool;
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test() { //TODO abdeckende test, statt dieser eher zufälligen
        let mut values = vec![1, 2, 3, 100, 1000, 10000, 100000, 1000000, 10000000, 1099511627774];
        let mut yft = YFT::new(values, &mut None, 10, 8);
        assert_eq!(yft.predecessor(0), None);
        assert_eq!(yft.predecessor(1), None);
        assert_eq!(yft.predecessor(2), Some(1));
        assert_eq!(yft.predecessor(3), Some(2));
        assert_eq!(yft.predecessor(11), Some(3));
        assert_eq!(yft.predecessor(500), Some(100));
        assert_eq!(yft.predecessor(1000), Some(100));
        assert_eq!(yft.predecessor(109951162), Some(10000000));
        assert_eq!(yft.predecessor(1099511627), Some(10000000));
        assert_eq!(yft.predecessor(10995116277), Some(10000000));
        assert_eq!(yft.predecessor(184467440737), Some(10000000));
        assert_eq!(yft.predecessor(1099511627774), Some(10000000));
        assert_eq!(yft.predecessor(1099511627775), Some(1099511627774));
        values = vec![1099511627, 1099511627775];
        yft = YFT::new(values, &mut None, 10, 8);
        assert_eq!(yft.predecessor(0), None);
        assert_eq!(yft.predecessor(1), None);
        assert_eq!(yft.predecessor(2), None);
        assert_eq!(yft.predecessor(3), None);
        assert_eq!(yft.predecessor(11), None);
        assert_eq!(yft.predecessor(500), None);
        assert_eq!(yft.predecessor(1000), None);
        assert_eq!(yft.predecessor(1099511627774), Some(1099511627));
        assert_eq!(yft.predecessor(1099511627775), Some(1099511627));
        values = vec![1844, 18446744073];
        yft = YFT::new(values, &mut None, 10, 8);
        assert_eq!(yft.predecessor(0), None);
        assert_eq!(yft.predecessor(1), None);
        assert_eq!(yft.predecessor(2), None);
        assert_eq!(yft.predecessor(3), None);
        assert_eq!(yft.predecessor(11), None);
        assert_eq!(yft.predecessor(500), None);
        assert_eq!(yft.predecessor(1000), None);
        assert_eq!(yft.predecessor(109951162777), Some(18446744073));
        assert_eq!(yft.predecessor(1099511627773), Some(18446744073));
        assert_eq!(yft.predecessor(1099511627774), Some(18446744073));
    }
}
