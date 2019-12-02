extern crate rustc_hash;

use args::Args;
use log::Log;
use uint::u40;
use self::rustc_hash::FxHashMap;
use predecessor_set::PredecessorSet;

pub type DataType = u40;

const BIT_LENGTH: usize = 40;

/*If v is a node at a height j, then all
the leafs descending from v will have key values
between the quantities (i - 1)2^J + 1 and i* 2^J */

///40 bit Impl with no fixed group size and without  child pointer
pub struct YFT {
    //predecessor of non existing subtree vec, DataType::max_value() if None (DataType::max_value() cant't be predecessor)
    lss_top: Vec<DataType>,
    // LSS Leaf Level (Position, Array Index)
    lss_leaf: FxHashMap<DataType, DataType>,
    // List of LSS Branch Level (Position, predecessor)
    lss_branch: Vec<FxHashMap<DataType, DataType>>,
    //== lss leaf level
    start_level: usize,
    //number of levels that are pooled into one level at the top of the xft
    last_level_len: usize,
    //Original input
    elements: Vec<DataType>,
}

impl YFT {
    ///elements must be sorted ascending!
    pub fn new(elements: Vec<DataType>, args: &Args, log: &mut Log) -> YFT {
        if elements.len() == 0 {
            panic!("Input is empty");
        }
        if elements.len() >= usize::from(DataType::max_value()) - 1 {
            panic!("Too many Elements in input");
        }
        let start_level = if let Some(start_level) = args.fixed_leaf_level {
            start_level
        } else {
            YFT::calc_start_level(&elements, args.min_start_level, BIT_LENGTH - args.max_lss_level, args.min_start_level_load_factor)
        };
        log.log_time("start level calculated");
        let last_level_len = if let Some(top_level) = args.fixed_top_level {
            BIT_LENGTH - top_level
        } else {
            BIT_LENGTH - YFT::calc_lss_top_level(&elements, start_level, BIT_LENGTH - args.max_lss_level, args.max_last_level_load_factor, args.min_load_factor_difference)
        };
        log.log_time("number of top levels calculated");
        let levels = BIT_LENGTH - start_level - last_level_len;
        assert!(levels > 0 && levels < BIT_LENGTH);

        //initialise lss_top
        let mut lss_top = vec![DataType::max_value(); 2usize.pow(last_level_len as u32)];//Bei eingaben bis 2^32 könnte man auch u32 nehmen...
        for (pos, value) in elements.iter().enumerate() {
            //check array is sorted
            debug_assert!(pos == 0 || value >= &elements[pos - 1]);

            let top_pos = YFT::lss_top_position(value, last_level_len) as usize;
            //set predecessor
            if is_left_child(DataType::from(YFT::lss_top_position(value, last_level_len + 1))) {
                // for queries on right child of this top level element, this element is its predecessor
                lss_top[top_pos] = elements[pos]; //always write is correct, cause if there are values under the branch, binary search wont ask top array
            } else if top_pos + 1 < lss_top.len() {
                //this right child is the predecessor of the next element
                lss_top[top_pos + 1] = elements[pos];
            }
        }
        //fill skipped lss top positions
        let mut lss_top_pos = 0;
        let mut last_value = DataType::max_value();
        while lss_top_pos < lss_top.len() {
            if lss_top[lss_top_pos] == DataType::max_value() {
                lss_top[lss_top_pos] = last_value;
            } else {
                last_value = lss_top[lss_top_pos];
            }
            lss_top_pos += 1;
        }
        log.log_mem("lss_branch top filled").log_time("lss_branch top filled");

        //initialise lss_branch
        let mut lss_leaf: FxHashMap<DataType, DataType> = FxHashMap::default();
        let mut lss_branch = Vec::with_capacity(levels - 1);
        for _level in 0..levels - 1 { // one less, cause leaf level is stored separately
            lss_branch.push(FxHashMap::default());
        }

        log.log_mem("lss_branch initialized").log_time("lss_branch initialized");

        //fill
        let mut predecessor_x_leaf: Option<DataType> = None;
        for (element_array_index, value) in elements.iter().enumerate() {
            let x_leaf_position = calc_path(*value, 0, start_level);
            if Some(x_leaf_position) != predecessor_x_leaf {
                //create new leaf node and insert it in level 0
                lss_leaf.insert(x_leaf_position, DataType::from(element_array_index));
            }

            //insert branch nodes
            let mut child = x_leaf_position;
            //iterate through levels, until parent exists
            for i in 1..levels {
                //path of new parent
                let path = calc_path(*value, i, start_level);
                if is_left_child(child) {
                    // set descending pointer to rightmost leaf in left tree
                    lss_branch[i - 1].insert(path, elements[element_array_index]);
                } else {
                    // if only right tree exists, the predecessor of the first element has to be set (so don't set, if already one element is set)
                    if !lss_branch[i - 1].contains_key(&path) {
                        //max_value indicates no predecessor
                        lss_branch[i - 1].insert(path, if element_array_index == 0 { DataType::max_value() } else { elements[element_array_index - 1] });
                    }
                }
                child = path;
            }
            predecessor_x_leaf = Some(x_leaf_position as DataType);
        }

        //return
        YFT { lss_top, lss_leaf, lss_branch, start_level, last_level_len, elements }
    }

    ///prints number of elements + relative fill level per lss level
    pub fn print_stats(&self, log: &Log) {
        log.print_result(format!("start_level={}\tnormal_levels={}\ttop_levels={}", self.start_level, self.lss_branch.len() + 1, self.last_level_len));
        let mut len = self.lss_leaf.len();
        let mut count = len;
        log.print_result(format!("level=0\tnodes={}\trelative_to_input={}\trelative_to_capacity={}", len, len as f32 / self.elements.len() as f32, len as f32 / 2f32.powf((BIT_LENGTH - self.start_level) as f32)));
        for level in 1..self.lss_branch.len() + 1 {
            len = self.lss_branch[level - 1].len();
            log.print_result(format!("level={}\tnodes={}\trelative_to_input={}\trelative_to_capacity={}", level, len, len as f32 / self.elements.len() as f32, len as f32 / 2f32.powf((BIT_LENGTH - self.start_level - level) as f32)));
            count += self.lss_branch[level - 1].len();
        }
        log.print_result(format!("level=-1\tnodes={}\telements={}", count, self.elements.len()));
    }

    fn calc_start_level(elements: &Vec<DataType>, min_start_level: usize, max_lss_level: usize, min_load_factor: usize) -> usize {
        let mut range = (min_start_level, max_lss_level - 1);
        while range.0 < range.1 {
            let candidate = (range.0 + range.1) / 2;
            if YFT::calc_nodes_in_level(candidate, elements) / (min_load_factor as f64) >= elements.len() as f64 / 100. {
                range = (candidate + 1, range.1)
            } else {
                range = (range.0, candidate)
            }
        }
        range.1 as usize
    }

    /// start_level == lowest possible level
    /// max_lss_level == highest possible level
    /// max_load_factor == maximal percentage that a level should be filled with (between 0 and 100)
    /// min_load_factor_difference == maximal factor that a level should be less relatively filled than the last possible level (between 0 and 100)
    fn calc_lss_top_level(elements: &Vec<DataType>, start_level: usize, max_lss_level: usize, max_load_factor: usize, min_load_factor_difference: usize) -> usize {
        let mut range = (start_level + 1, max_lss_level);
        //load factor can only increase if level gets higher. If it doesn't, levels can be cut.
        let top_load_factor = YFT::calc_nodes_in_level(max_lss_level, elements) / 2f64.powf((BIT_LENGTH - max_lss_level) as f64) * (min_load_factor_difference as f64) / 100.;
        let max = if top_load_factor < (max_load_factor as f64) / 100. {
            top_load_factor
        } else {
            (max_load_factor as f64) / 100.
        };
        while range.0 < range.1 {
            let candidate = (range.0 + range.1) / 2;
            let load_factor = YFT::calc_nodes_in_level(candidate, elements) / 2f64.powf((BIT_LENGTH - candidate) as f64);
            if load_factor < max {
                range = (candidate + 1, range.1)
            } else {
                range = (range.0, candidate)
            }
        }
        range.1 as usize
    }

    ///count how many nodes are in one level
    fn calc_nodes_in_level(level: usize, elements: &Vec<DataType>) -> f64 { //TODO mögliche Beschleunigung durch Stichproben
        let mut last_val = calc_path(elements[0], level, 0);
        let mut count = 1.;
        for value in elements {
            let new_val = calc_path(*value, level, 0);
            if new_val != last_val {
                count += 1.;
                last_val = new_val;
            }
        }
        count
    }

    fn lss_top_position(value: &DataType, lss_top_length: usize) -> usize {
        usize::from(*value) >> (BIT_LENGTH - lss_top_length)
    }

    pub fn contains(&self, query: DataType) -> bool {
        self.predecessor(query + 1 as u32) == Some(query)
    }

    //query may not belong to existing node
    pub fn predecessor(&self, query: DataType) -> Option<DataType> {
        unsafe {
            if query < *self.elements.get_unchecked(0) {
                return None;
            }
            //binary search lowest ancestor for some query
            // query 0 == lss_leaf, query len()+1 == lss_top
            let mut search_range = (0, self.lss_branch.len() + 1);
            while search_range.0 != search_range.1 {
                let mut search_position = (search_range.0 + search_range.1) / 2;
                if search_position == self.lss_branch.len() + 1 {
                    //top level may only be used iff there are no existing nodes below in search path
                    search_position -= 1;
                }

                if search_position == 0 {
                    //leaf level
                    match self.lss_leaf.get(&calc_path(query, search_position, self.start_level)) {
                        Some(first_element) => {
                            return self.predecessor_from_array(query, *first_element);
                        }
                        None => {
                            //there is no node -> search higher
                            search_range = (search_position + 1, search_range.1);
                        }
                    }
                } else {
                    match self.lss_branch[search_position - 1].get(&calc_path(query, search_position, self.start_level)) {
                        Some(_branch) => {
                            //there is a branch =>  search lower
                            search_range = (search_range.0, search_position);
                        }
                        None => {
                            //there is no node -> search higher
                            search_range = (search_position + 1, search_range.1);
                        }
                    }
                }
            }

            //search range includes now exact the lowest existing node, if there's one

            if search_range.0 == self.lss_branch.len() + 1 {
                //case there is no existing node -> look @ lss_top
                return self.predec_lss_top(query);
            }

            if search_range.0 == 0 {
                //leaf level
                match self.lss_leaf.get(&calc_path(query, search_range.0, self.start_level)) {
                    Some(first_element) => {
                        //searched note is in Tree -> return its predecessor
                        return self.predecessor_from_array(query, *first_element);
                    }
                    None => {
                        panic!("This can't happen, cause it was checked at beginning of this method, that there is a predecessor");
                    }
                }
            } else {
                match self.lss_branch[search_range.0 - 1].get(&calc_path(query, search_range.0, self.start_level)) {
                    Some(predecessor) => {
                        //it was checked at beginning, that there is a predecessor
                        debug_assert!(*predecessor != DataType::max_value());
                        return Some(*predecessor);
                    }
                    None => {
                        panic!("This can't happen, cause it was checked at beginning of this method, that there is a predecessor");
                    }
                }
            }
        }
    }

    ///can only be used, if there is no existing node below
    fn predec_lss_top(&self, query: DataType) -> Option<DataType> {
        // assert not in lss branch
        debug_assert!(self.lss_branch.len() == 0 || match self.lss_branch[self.lss_branch.len() - 1].get(&calc_path(query, BIT_LENGTH - self.last_level_len - 1 - self.start_level, self.start_level)) {
            None => true,
            Some(_) => false
        });
        debug_assert!(self.lss_branch.len() > 0 || match self.lss_leaf.get(&calc_path(query, BIT_LENGTH - self.last_level_len - 1 - self.start_level, self.start_level)) {
            None => true,
            Some(_) => false
        });
        unsafe {
            let predecessor = *self.lss_top.get_unchecked(YFT::lss_top_position(&query, self.last_level_len));
            //it was checked at beginning, that there is a predecessor
            debug_assert!(predecessor != DataType::max_value());
            return Some(predecessor);
        }
    }

    /// query = predecessor query
    /// index = predecessor position in array
    fn element_from_array(&self, query: DataType, index: DataType) -> Option<DataType> {
        //test next value greater than search one
        debug_assert!(usize::from(index) + 1 >= self.elements.len() || if let Some(successor) = self.elements.get(usize::from(index) + 1) { successor >= &query } else { true });
        //test value smaller than searched one
        debug_assert!(if let Some(predecessor) = self.elements.get(usize::from(index)) { predecessor < &query } else { true });
        debug_assert!(usize::from(index) < self.elements.len());
        unsafe {
            return Some(*self.elements.get_unchecked(usize::from(index)));
        }
    }

    fn predecessor_from_array(&self, query: DataType, index: DataType) -> Option<DataType> {
        let mut index = index;
        while index < self.elements.len() as u64 && self.elements[usize::from(index)] < query {
            index += 1 as u32;
        }
        return if index == 0 { None } else { self.element_from_array(query, index - 1 as u32) };
    }
} //impl YFT

fn calc_path(position: DataType, lss_level: usize, start_level: usize) -> DataType {
    position >> DataType::from(lss_level + start_level)
}

//returns if the node is the left of its parent
fn is_left_child(path: DataType) -> bool {
    path % 2 == 0
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