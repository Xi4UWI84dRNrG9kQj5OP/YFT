extern crate boomphf;

use log::Log;
use uint::u40;
use self::boomphf::hashmap::BoomHashMap;
use predecessor_set::PredecessorSet;

pub type DataType = u40;

const BIT_LENGTH: usize = 40;

/*If v is a node at a height j, then all
the leafs descending from v will have key values
between the quantities (i - 1)2^J + 1 and i* 2^J */

pub struct YFT {
    //position of successor of subtree in element vec, 0 if None
    lss_top: Vec<DataType>,
    // Position, node
    lss_leaf: BoomHashMap<DataType, TreeLeaf>,
    lss_branch: Vec<BoomHashMap<DataType, TreeBranch>>,
    //== lss leaf level
    start_level: usize,
    //number of levels that are pooled into one level at the top of the xft
    last_level_len: usize,
    //Original input
    elements: Vec<DataType>,
}

impl YFT {
    ///elements must be sorted ascending!
    pub fn new(elements: Vec<DataType>, min_start_level: usize, min_start_level_load_factor: usize, max_lss_level: usize, max_last_level_load_factor: usize, log: &mut Log) -> YFT {
        let start_level = YFT::calc_start_level(&elements, min_start_level, BIT_LENGTH - max_lss_level, min_start_level_load_factor);
        log.log_time("start level calculated");
        let last_level_len = BIT_LENGTH - YFT::calc_lss_top_level(&elements, start_level, BIT_LENGTH - max_lss_level, max_last_level_load_factor);
        log.log_time("number of top levels calculated");
        let levels = BIT_LENGTH - start_level - last_level_len;

        //initialise lss_top
        let mut lss_top = vec![DataType::from(0); 2usize.pow(last_level_len as u32)];
        for (pos, value) in elements.iter().enumerate() {
            //check array is sorted
            debug_assert!(pos == 0 || value >= &elements[pos - 1]);
            //check value not to big
            debug_assert!(pos >> (levels + start_level + last_level_len) == 0);
            let mut lss_top_pos = YFT::lss_top_position(value, last_level_len) as usize;

            //set successors
            if lss_top[lss_top_pos] == 0 && !is_left_child(DataType::from(YFT::lss_top_position(value, last_level_len + 1))) {
                // for queries on left child of this top level element, this element is its successor
                lss_top[lss_top_pos] = DataType::from(pos);
            }
            while lss_top_pos > 0 && lss_top[lss_top_pos - 1] == 0 {
                lss_top_pos -= 1;
                lss_top[lss_top_pos] = DataType::from(pos);
            }
        }
        log.log_mem("lss_branch top filled").log_time("lss_branch top filled");

        //generate leaf level
        let mut keys = Vec::new();
        let mut leaf_data = Vec::new();
        for (element_array_index, value) in elements.iter().enumerate() {
            let leaf_position = calc_path(*value, 0, start_level);
            if keys.len()== 0 || keys[keys.len() - 1] != leaf_position {
                keys.push(leaf_position);
                leaf_data.push(TreeLeaf { first_element: DataType::from(element_array_index) });
            }
        }
        let lss_leaf = BoomHashMap::new_parallel(keys, leaf_data);

        //generate branch levels
        let mut lss_branch = Vec::with_capacity(levels - 1);
        for i in 1..levels {
            keys = Vec::new();
            let mut data = Vec::new();
            for (element_array_index, value) in elements.iter().enumerate() { //TODO iter through level above instead of elements
                let is_left_child = is_left_child(calc_path(*value, i - 1, start_level));
                let branch_position = calc_path(*value, i, start_level);
                let len = data.len();
                if len == 0 || keys[len - 1] != branch_position {
                    keys.push(branch_position);
                    data.push(TreeBranch { children: if is_left_child { Children::LEFT } else { Children::RIGHT }, descending: DataType::from(element_array_index) });
                } else {
                    if (!data[len - 1].has_left_child() && is_left_child) || (!data[len - 1].has_right_child() && !is_left_child) {
                        data[len - 1].children = Children::BOTH; //TODO can be optimised when optimazation above has been made
                    }
                }
            }
            lss_branch.push(BoomHashMap::new_parallel(keys, data));
            log.log_mem(format!("lss_branch[{}] filled", i).as_str()).log_time(format!("lss_branch[{}] filled", i).as_str());
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
            if YFT::calc_nodes_in_level(candidate, elements) / min_load_factor >= elements.len() / 100 {
                range = (candidate + 1, range.1)
            } else {
                range = (range.0, candidate)
            }
        }
        range.1 as usize
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
            if position < *self.elements.get_unchecked(0) { //TODO array empty
                return None;
            }
            //binary search lowest ancestor for some position
            // position 0 == lss_leaf, position len()+1 == lss_top
            let mut search_range = (0, self.lss_branch.len() + 1);
            while search_range.0 != search_range.1 {
                let search_position = (search_range.0 + search_range.1) / 2;
                if search_position == self.lss_branch.len() + 1 {
                    //top level shows directly to predecessors
                    return self.predec_lss_top(position);
                }

                if search_position == 0 {
                    //leaf level
                    match self.lss_leaf.get(&calc_path(position, search_position, self.start_level)) {
                        Some(leaf) => {
                            return self.predecessor_from_array(position, leaf.first_element);
                        }
                        None => {
                            //there is no node -> search higher
                            search_range = (search_position + 1, search_range.1);
                        }
                    }
                } else {
                    match self.lss_branch[search_position - 1].get(&calc_path(position, search_position, self.start_level)) {
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
                return self.predec_lss_top(position);
            }

            if search_range.0 == 0 {
                //leaf level
                match self.lss_leaf.get(&calc_path(position, search_range.0, self.start_level)) { //TODO konvertieren ggf. teuer
                    Some(leaf) => {
                        //searched note is in Tree -> return its predecessor
                        return self.elements.get(usize::from(leaf.first_element - 1 as u32)).cloned();
                    }
                    None => {
                        panic!("This can't happen, cause it was checked at beginning of this method, that there is a predecessor");
                    }
                }
            } else {
                match self.lss_branch[search_range.0 - 1].get(&calc_path(position, search_range.0, self.start_level)) {
                    Some(branch) => {
                        if !branch.has_right_child() {
                            //first missing node in xft would be right child -> descending shows predecessor
                            return self.element_from_array(position, branch.descending);
                        } else {
                            //first missing node in xft would be left child -> descending shows successor
                            debug_assert!(!branch.has_left_child());
                            return if branch.descending == 0 { None } else { self.element_from_array(position, branch.descending - 1 as u32) };
                        }
                    }
                    None => {
                        panic!("This can't happen, cause it was checked at beginning of this method, that there is a predecessor");
                    }
                }
            }
        }
    }

    ///can only be used, if there is no existing node below
    fn predec_lss_top(&self, position: DataType) -> Option<DataType> {
        // assert not in lss branch
        debug_assert!(self.lss_branch.len() == 0 || match self.lss_branch[self.lss_branch.len() - 1].get(&calc_path(position, BIT_LENGTH - self.last_level_len - 1 - self.start_level, self.start_level)) {
            None => true,
            Some(_) => false
        });
        debug_assert!(self.lss_branch.len() > 0 || match self.lss_leaf.get(&calc_path(position, BIT_LENGTH - self.last_level_len - 1 - self.start_level, self.start_level)) {
            None => true,
            Some(_) => false
        });
        unsafe {
            let pos = *self.lss_top.get_unchecked(YFT::lss_top_position(&position, self.last_level_len));
            if pos == 0 {
                if self.elements.len() > 0 && *self.elements.get_unchecked(self.elements.len() - 1) < position {
                    return self.element_from_array(position, DataType::from(self.elements.len() - 1));
                }
                //assert there is no smaller value in element array
                debug_assert!(self.elements.len() == 0 || self.elements[0] > position);
                return None;
            } else {
                return self.element_from_array(position, pos - 1 as u32);
            }
        }
    }

    /// position = predecessor query
    /// index = predecessor position in array
    fn element_from_array(&self, position: DataType, index: DataType) -> Option<DataType> {
        //test next value greater than search one
        debug_assert!(usize::from(index) + 1 >= self.elements.len() || if let Some(successor) = self.elements.get(usize::from(index) + 1) { successor >= &position } else { true });
        //test value smaller than searched one
        debug_assert!(if let Some(predecessor) = self.elements.get(usize::from(index)) { predecessor < &position } else { true });
        debug_assert!(usize::from(index) < self.elements.len());
        unsafe {
            return Some(*self.elements.get_unchecked(usize::from(index)));
        }
    }

    fn predecessor_from_array(&self, value: DataType, index: DataType) -> Option<DataType> {
        let mut index = index;
        while index < self.elements.len() as u64 && self.elements[usize::from(index)] < value {
            index += 1 as u32;
        }
        return if index == 0 { None } else { self.element_from_array(value, index - 1 as u32) };
    }
} //impl YFT

bitflags! {
    struct Children: u8 {
        const LEFT = 0b00000001;
        const RIGHT = 0b00000010;
        const BOTH = Self::LEFT.bits | Self::RIGHT.bits;
    }
}

#[derive(Debug)]
struct TreeBranch {
    children: Children,
    //0 None, 1 == left child, 2 == right child, 3 == both
    descending: DataType, //Position of predecelement in elementarray
}

impl TreeBranch {
    fn set_child(&mut self, left: bool) {
        if left {
            debug_assert!(!self.has_left_child());
        } else {
            debug_assert!(!self.has_right_child());
        }
        self.children = Children::BOTH;
        self.descending = DataType::from(0);
    }

    fn has_left_child(&self) -> bool {
        self.children.contains(Children::LEFT)
    }

    fn has_right_child(&self) -> bool {
        self.children.contains(Children::RIGHT)
    }
}

#[derive(Debug)]
struct TreeLeaf {
    first_element: DataType,
    //Position of first element in Value Vector
}

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