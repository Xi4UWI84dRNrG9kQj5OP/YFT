extern crate rustc_hash;

use log::Log;
use uint::u40;
use predecessor_set::PredecessorSet;

pub type DataType = u40;

const BIT_LENGTH: usize = 40;

/*If v is a node at a height j, then all
the leafs descending from v will have key values
between the quantities (i - 1)2^J + 1 and i* 2^J */

pub struct YFT {
    //position of successor of subtree in element vec, 0 if None
    lss_top: Vec<DataType>,
    //number of levels that are pooled into one level at the top of the xft
    last_level_len: usize,
    //Original input
    elements: Vec<DataType>,
}

impl YFT {
    ///elements must be sorted ascending!
    pub fn new(elements: Vec<DataType>, min_start_level: usize, _min_start_level_load_factor: usize, max_lss_level: usize, max_last_level_load_factor: usize, log: &mut Log) -> YFT {
        let last_level_len = BIT_LENGTH - YFT::calc_lss_top_level(&elements, min_start_level, BIT_LENGTH - max_lss_level, max_last_level_load_factor);
        log.log_time("number of top levels calculated");

        //initialise lss_top
        let mut lss_top = vec![DataType::from(0); 2usize.pow(last_level_len as u32)];
        for (pos, value) in elements.iter().enumerate() {
            //check array is sorted
            debug_assert!(pos == 0 || value >= &elements[pos - 1]);
            let mut lss_top_pos = YFT::lss_top_position(value, last_level_len) as usize;

            //set successors //TODO auf predecessor umstellen, müsste einfacher sein; klappt aktuell beim letzten nicht, da dieser auch abschnitt beschreibt und asertion damit nicht umgehen kann
            while lss_top_pos > 0 && lss_top[lss_top_pos - 1] == 0 { //Idee: vorgänger kopieren? TODO while unten beim suchen aus den anderen wieder raus, da strukturen drunter kann das nicht passieren
                lss_top_pos -= 1;
                lss_top[lss_top_pos] = DataType::from(pos);
            }
        }

        //return
        YFT { lss_top, last_level_len, elements }
    }

    ///prints number of elements + relative fill level per lss level
    pub fn print_stats(&self, log: &Log) {
        log.print_result(format!("start_level={}\tnormal_levels={}\ttop_levels={}", 0, 0, self.last_level_len));
        log.print_result(format!("level=-1\tnodes={}\telements={}", 0, self.elements.len()));
    }

    fn calc_lss_top_level(elements: &Vec<DataType>, min_start_level: usize, max_lss_level: usize, max_load_factor: usize) -> usize {
        let mut range = (min_start_level + 1, max_lss_level);
        while range.0 < range.1 {
            let candidate = (range.0 + range.1) / 2;
            if YFT::calc_nodes_in_level(candidate, elements) / max_load_factor < 2usize.pow((BIT_LENGTH - candidate) as u32) / 100 {
                range = (candidate + 1, range.1)
            } else {
                range = (range.0, candidate)
            }
        }
        range.1 as usize
    }

    ///count how many nodes are in one level
    fn calc_nodes_in_level(level: usize, elements: &Vec<DataType>) -> usize { //TODO mögliche Beschleunigung durch Stichproben
        let mut last_val = calc_path(elements[0], level, 0);
        let mut count = 1;
        for value in elements {
            let new_val = calc_path(*value, level, 0);
            if new_val != last_val {
                count += 1;
                last_val = new_val;
            }
        }
        count
    }

    fn lss_top_position(value: &DataType, lss_top_length: usize) -> usize {
        usize::from(*value) >> (BIT_LENGTH - lss_top_length)
    }

    pub fn contains(&self, position: DataType) -> bool {
        self.predecessor(position + 1 as u32) == Some(position)
    }

    //position may not belong to existing node
    pub fn predecessor(&self, position: DataType) -> Option<DataType> {
        unsafe {
            let mut pos = usize::from(*self.lss_top.get_unchecked(YFT::lss_top_position(&position, self.last_level_len)));
            if pos == 0 && self.elements.len() > 0 && *self.elements.get_unchecked(0) < position {
                pos = self.elements.len() - 1;
                if *self.elements.get_unchecked(pos) < position{
                    return self.element_from_array(position, pos)
                }
            }
            debug_assert!(pos == 0 || self.elements[pos] >= position);
            while pos > 0 && *self.elements.get_unchecked(pos - 1) >= position {
                pos = pos - 1;
            }
            debug_assert!(pos == 0 || self.elements[pos] >= position);
            if pos == 0 {
                //assert there is no smaller value in element array
                debug_assert!(self.elements.len() == 0 || self.elements[0] >= position);
                None
            } else {
                self.element_from_array(position, pos - 1)
            }
        }
    }


    /// position = predecessor query
    /// index = predecessor position in array
    fn element_from_array(&self, position: DataType, index: usize) -> Option<DataType> {
        //test next value greater than search one
        debug_assert!(index + 1 >= self.elements.len() || if let Some(successor) = self.elements.get(index + 1) { successor >= &position } else { true });
        //test value smaller than searched one
        debug_assert!(if let Some(predecessor) = self.elements.get(index) { predecessor < &position } else { true });
        debug_assert!(index < self.elements.len());
        unsafe {
            return Some(*self.elements.get_unchecked(index));
        }
    }
} //impl YFT


fn calc_path(position: DataType, lss_level: usize, start_level: usize) -> DataType {
    position >> DataType::from(lss_level + start_level)
}

impl PredecessorSet<DataType> for YFT {
    ///static YFT can not insert
    fn insert(&mut self, _element: DataType) {
        panic!("static YFT can not insert");
    }
    ///static YFT can not delete
    fn delete(&mut self, _element: DataType) {
        panic!("static YFT can not delete");
    }
    fn predecessor(&self, number: DataType) -> Option<DataType> {
        self.predecessor(number)
    }
    ///not implemented yet
    fn successor(&self, _number: DataType) -> Option<DataType> {
        panic!("sucessor not implemented yet")
    }
    fn minimum(&self) -> Option<DataType> {
        panic!("minimum not implemented yet")
    }
    fn maximum(&self) -> Option<DataType> {
        panic!("maximum not implemented yet")
    }
    fn contains(&self, number: DataType) -> bool {
        self.contains(number)
    }
}