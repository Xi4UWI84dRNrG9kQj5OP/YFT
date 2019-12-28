extern crate fnv;

use args::Args;
use log::Log;
use uint::u40;
use self::fnv::FnvHashMap;
use predecessor_set::PredecessorSet;

pub type DataType = u40;
pub type SmallType = u16;

const BIT_LENGTH: usize = 40;
const SMALL_TYPE_LEN: usize = 16;

/*If v is a node at a height j, then all
the leafs descending from v will have key values
between the quantities (i - 1)2^J + 1 and i* 2^J */

///dynamic 40 bit Impl with input array stored in leafs last 16 bit only, without child pointer and binary search below xft leafs
pub struct YFT {
    //predecessor of non existing subtree vec, DataType::max_value() if None (DataType::max_value() cant't be predecessor)
    lss_top: Vec<DataType>,
    // LSS Leaf Level <Position, (predecessor if there is none with same prefix, Elements that may be predecessor of prefix)>
    lss_leaf: FnvHashMap<DataType, (DataType, Vec<SmallType>)>,
    // List of LSS Branch Level <Position, predecessor>
    lss_branch: Vec<FnvHashMap<DataType, DataType>>,
    //== lss leaf level
    start_level: usize,
    //number of levels that are pooled into one level at the top of the xft
    last_level_len: usize,
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
        let mut start_level = if let Some(start_level) = args.fixed_leaf_level {
            start_level
        } else {
            YFT::calc_start_level(&elements, args.min_start_level, BIT_LENGTH - args.max_lss_level, args.min_start_level_load_factor)
        };
        if start_level > 15 {
            println!("Start level set down to 15");
            start_level = 16;
        }
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
        let mut lss_leaf: FnvHashMap<DataType, (DataType, Vec<SmallType>)> = FnvHashMap::default();
        let mut lss_branch = Vec::with_capacity(levels - 1);
        for _level in 0..levels - 1 { // one less, cause leaf level is stored separately
            lss_branch.push(FnvHashMap::default());
        }

        log.log_mem("lss_branch initialized").log_time("lss_branch initialized");

        //fill
        let mut predecessor_x_leaf: Option<DataType> = None;
        let mut predecessor = DataType::max_value();
        for (element_array_index, value) in elements.iter().enumerate() {
            let x_leaf_position = calc_path(*value, 0, start_level);
            if Some(x_leaf_position) != predecessor_x_leaf {
                //create new leaf node and insert it in level 0
                lss_leaf.insert(x_leaf_position, (predecessor, vec![SmallType::from(*value)]));
                //ensure predecessors array doesnt take to much space
                if let Some(predecessor_x_leaf) = predecessor_x_leaf {
                    lss_leaf.get_mut(&predecessor_x_leaf).unwrap().1.shrink_to_fit();
                }
            } else {
                //add value to elements of existing leaf
                lss_leaf.get_mut(&x_leaf_position).unwrap().1.push(SmallType::from(*value));
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
            predecessor = *value;
        }

        //return
        YFT { lss_top, lss_leaf, lss_branch, start_level, last_level_len }
    }

    pub fn add(&mut self, element: DataType) {
        let leaf_path = calc_path(element, 0, self.start_level);
        //TODO könnt effizienter beim iterieren gefunden werden
        let predecessor = self.predecessor(element).unwrap_or(DataType::max_value());
        let mut add_nodes = true;
        let mut do_nothing = false;
        self.lss_leaf.entry(leaf_path).and_modify(|(_predecessor, elements)| {
            //add element to existing leaf
            match elements.binary_search(&SmallType::from(element)) {
                Ok(_) => {
                    // element already exists, nothing to do
                    println!("Element {:?} already exists, nothing changed", element);
                    do_nothing = true;
                }
                Err(pos) => {
                    elements.insert(pos, SmallType::from(element));
                    if pos < elements.len() - 1 { // one element has just been added -> -1
                        //element is not last element -> no predecessor has to be changed
                        do_nothing = true;
                    }
                    add_nodes = false;
                }
            }
            //add element to new leaf
        }).or_insert((predecessor, vec![SmallType::from(element)]));

        if do_nothing{
            return;
        }

        self.change_predecessors_add(element, add_nodes, predecessor, &leaf_path);
    }

    /// sets predecessor of next leaf and branches on path form this and next leaf to root,
   /// element element that has been added or removed
   /// remove_node if branches have to be removed
   /// leaf_path leaf path of element
    fn change_predecessors_add(&mut self, element: DataType, mut add_nodes: bool, element_predecessor: DataType, leaf_path: &DataType) {
        let mut set_leaf_predecessor = true;
        self.set_leaf_predecessor(&mut add_nodes, element_predecessor, element, leaf_path, &mut set_leaf_predecessor);
        let mut has_left_child = is_left_child(*leaf_path);
// add nodes, set predecessors in leaf anf branches
        for i in 0..self.lss_branch.len() { //i = level - 1, cause of leaf level
            let path = calc_path(element, i + 1, self.start_level);
            if add_nodes {
                self.lss_branch[i].insert(path, if has_left_child {element} else {element_predecessor});
                has_left_child = is_left_child(path);

                if has_left_child {
                    // case left child has been removed
                    let right_child_path = path + DataType::from(1);
                    //something to trick the borrow checker (double mutable access to lss_branch)
                    let mut child_is_there = false;
                    self.lss_branch[i].entry(right_child_path).and_modify(|predecessor| {
                        if *predecessor == element_predecessor {
                            *predecessor = element;
                        }
                        // other child found, parents doesnt have to be removed
                        add_nodes = false;
                        child_is_there = true;
                    });
                    if child_is_there {
                        self.set_next_leaf_xft_path_predecessor(element, &mut set_leaf_predecessor, i, right_child_path, element_predecessor);
                    }
                } else {
                    //case right child has been removed, predecessor must not be changed on left child
                    if self.lss_branch[i].contains_key(&(path - DataType::from(1))) {
                        //no other node has to be removed, cause one child exist
                        add_nodes = false;
                    }
                }
                debug_assert!(self.lss_branch[i].contains_key(&path));
            } else {
                self.lss_branch[i].entry(path).and_modify(|predecessor| {
                    if has_left_child && *predecessor == element_predecessor {
                        *predecessor = element;
                    }
                });
                has_left_child = is_left_child(path);
                debug_assert!(self.lss_branch[i].contains_key(&path));
                if has_left_child {
                    //node is left child and shall not be removed
                    //this var wil be used to create path of next leaf, if possible
                    let right_child_path = path + DataType::from(1);
                    if self.lss_branch[i].contains_key(&(right_child_path)) {
                        self.lss_branch[i].entry(right_child_path).and_modify(|predecessor| {
                            if *predecessor == element_predecessor {
                                *predecessor = element;
                            }
                        });
                        self.set_next_leaf_xft_path_predecessor(element, &mut set_leaf_predecessor, i, right_child_path, element_predecessor);
                    }
                }
            }
        }

        self.set_leaf_predecessor_via_top(element, element_predecessor, element, &mut set_leaf_predecessor);
        self.adjust_lss_top(element_predecessor, element);
    }

    fn set_leaf_predecessor(&mut self, change_nodes: &mut bool, old_predecessor: DataType, new_predecessor: DataType, leaf_path: &DataType, set_leaf_predecessor: &mut bool) {
        if is_left_child(*leaf_path) {
            self.lss_leaf.entry(*leaf_path + DataType::from(1)).and_modify(|(predecessor, _elements)| {
                debug_assert!(*predecessor == old_predecessor);
                *predecessor = new_predecessor;
                //if right child of parent is next child, set its predecessor
                *set_leaf_predecessor = false;
                //no node has to be removed, cause one child exist
                *change_nodes = false;
            });
        } else { //case right child, predecessor must not be changed on left child
            if self.lss_leaf.contains_key(&(*leaf_path - DataType::from(1))) {
                //no node has to be removed, cause one child exist
                *change_nodes = false;
            }
        }
    }

    pub fn remove(&mut self, element: DataType) {
        let mut remove_node = false;
        let mut new_predecessor= DataType::max_value();
        let leaf_path = calc_path(element, 0, self.start_level);
        let mut do_nothing = false;
        match self.lss_leaf.get_mut(&leaf_path) {
            Some((predecessor, elements)) => {
                match elements.binary_search(&SmallType::from(element)) {
                    Ok(pos) => {
                        elements.remove(pos);
                        if elements.len() == 0 {
                            remove_node = true;
                            new_predecessor = *predecessor;
                        } else if pos == elements.len() {
                            new_predecessor = extend_suffix(element, unsafe { *elements.get_unchecked(pos - 1) });
                        } else {
                            //nothing else to do
                            do_nothing = true;
                        }
                    }
                    Err(_) => { // no matching element
                        println!("Element {:?} does not exist and can't be removed", element);
                        do_nothing = true;
                    }
                }
            }
            None => { // no matching leaf
                println!("Element {:?} does not exist and can't be removed", element);
                do_nothing = true;
            }
        }

        if do_nothing{
            return;
        }
        self.change_predecessors_remove(element, remove_node, element, new_predecessor, &leaf_path);
    }

    /// sets predecessor of next leaf and branches on path form this and next leaf to root,
    /// element element that has been added or removed
    /// remove_node if branches have to be removed
    /// leaf_path leaf path of element
    fn change_predecessors_remove(&mut self, element: DataType, mut remove_node: bool, old_predecessor: DataType, new_predecessor: DataType, leaf_path: &DataType) {
        let mut set_leaf_predecessor = true;
        if remove_node {
            self.lss_leaf.remove(&leaf_path);
        }
        self.set_leaf_predecessor(&mut remove_node, old_predecessor, new_predecessor, leaf_path, &mut set_leaf_predecessor);
        if is_left_child(*leaf_path) {
            self.lss_leaf.entry(*leaf_path + DataType::from(1)).and_modify(|(predecessor, _elements)| {
                debug_assert!(*predecessor == old_predecessor || *predecessor == new_predecessor);
                *predecessor = new_predecessor;
                //if right child of parent is next child, set its predecessor
                set_leaf_predecessor = false;
                //no node has to be removed, cause one child exist
                remove_node = false;
            });
        } else { //case right child, predecessor must not be changed on left child
            if self.lss_leaf.contains_key(&(*leaf_path - DataType::from(1))) {
                //no node has to be removed, cause one child exist
                remove_node = false;
            }
        }
// remove nodes, set predecessors in leaf anf branches
        for i in 0..self.lss_branch.len() { //i = level - 1, cause of leaf level
            let path = calc_path(element, i + 1, self.start_level);
            if remove_node {
                self.lss_branch[i].remove(&path);
                if is_left_child(path) {
                    // case left child has been removed
                    let right_child_path = path + DataType::from(1);
                    //something to trick the borrow checker (double mutable access to lss_branch)
                    let mut child_is_there = false;
                    self.lss_branch[i].entry(right_child_path).and_modify(|predecessor| {
                        if *predecessor == old_predecessor {
                            *predecessor = new_predecessor;
                        }
                        // other child found, parents doesnt have to be removed
                        remove_node = false;
                        child_is_there = true;
                    });
                    if child_is_there {
                        self.set_next_leaf_xft_path_predecessor(new_predecessor, &mut set_leaf_predecessor, i, right_child_path, old_predecessor);
                    }
                } else {
                    //case right child has been removed, predecessor must not be changed on left child
                    if self.lss_branch[i].contains_key(&(path - DataType::from(1))) {
                        //no other node has to be removed, cause one child exist
                        remove_node = false;
                    }
                }
            } else {
                if is_left_child(path) {
                    //node is left child and shall not be removed
                    //this var wil be used to create path of next leaf, if possible
                    let right_child_path = path + DataType::from(1);
                    if self.lss_branch[i].contains_key(&(right_child_path)) {
                        self.lss_branch[i].entry(right_child_path).and_modify(|predecessor| {
                            if *predecessor == old_predecessor {
                                *predecessor = new_predecessor;
                            }
                        });
                        self.set_next_leaf_xft_path_predecessor(new_predecessor, &mut set_leaf_predecessor, i, right_child_path, old_predecessor);
                    }
                }
                self.lss_branch[i].entry(path).and_modify(|predecessor| {
                    if *predecessor == old_predecessor {
                        *predecessor = new_predecessor;
                    }
                });
                debug_assert!(*self.lss_branch[i].get(&path).unwrap() != old_predecessor);
            }
        }

        self.set_leaf_predecessor_via_top(element, old_predecessor, new_predecessor, &mut set_leaf_predecessor);
        self.adjust_lss_top(old_predecessor, new_predecessor);
    }

    fn set_leaf_predecessor_via_top(&mut self, element: DataType, old_predecessor: DataType, new_predecessor: DataType, mut set_leaf_predecessor: &mut bool) {
        if *set_leaf_predecessor {
            // next leaf has not been found, cause there was no right child
            // find next node in highest branch level
            let len = self.lss_branch.len();
            let mut path = calc_path(element, len, self.start_level);
            while path != calc_path(DataType::max_value(), len, self.start_level) {
                path += DataType::from(1);
                if self.lss_branch[len - 1].contains_key(&path) {
                    self.lss_branch[len - 1].entry(path).and_modify(|predecessor| {
                        if *predecessor == old_predecessor {
                            *predecessor = new_predecessor;
                        }
                    });
                    self.set_next_leaf_xft_path_predecessor(new_predecessor, &mut set_leaf_predecessor, len - 1, path, old_predecessor);
                    break;
                }
            }
        }
    }

    fn adjust_lss_top(&mut self, old_predecessor: DataType, new_predecessor: DataType) {
        let mut pos = YFT::lss_top_position(&new_predecessor, self.last_level_len) as usize;
        if YFT::lss_top_position(&new_predecessor, self.last_level_len + 1) % 2 == 1 {
            // if new predecessor is right child, next lss_top prefix is first that has to be answered with it
//            debug_assert!(self.lss_top[pos] != old_predecessor || self.lss_branch[self.lss_branch.len() - 1].contains_key(&calc_path(new_predecessor, self.lss_branch.len() - 1, self.start_level)));
            pos += 1;
        }
        if new_predecessor == DataType::max_value() {
            //new predecessor == no predecessor
            pos = 0;
        }
        while pos < self.lss_top.len() {
            if self.lss_top[pos] == old_predecessor {
                self.lss_top[pos] = new_predecessor;
            } else {
                if (self.lss_top[pos] > new_predecessor || new_predecessor == DataType::max_value()) && self.lss_top[pos] > old_predecessor {
                    return;
                }
            }
            pos += 1;
        }
    }

    /// new predecessor value that should be set
    /// set_leaf_predecessor if something should be changed, will be set to false, if something has been changed
    /// branch_level level under which changing should be started
    /// next_leaf_path beginning of new path (in branch_level)
    fn set_next_leaf_xft_path_predecessor(&mut self, new_predecessor: DataType, set_leaf_predecessor: &mut bool, branch_level: usize, mut next_leaf_path: DataType, old_predecessor: DataType) {
        if *set_leaf_predecessor {
            //right child exists and predecessors of next node haven't been set yet
            //iterate through branch level beginning under i, to set predecessors
            for j in (0..branch_level).rev() {
                //build path
                if self.lss_branch[j].contains_key(&(next_leaf_path << 1)) {
                    // left child -> append 0
                    next_leaf_path = next_leaf_path << 1;
                    debug_assert!(self.lss_branch[j].contains_key(&next_leaf_path));
                } else {
                    // right child -> append 1
                    next_leaf_path = (next_leaf_path << 1) + DataType::from(1);
                    debug_assert!(self.lss_branch[j].contains_key(&next_leaf_path));
                }
                debug_assert!(self.lss_branch[j].contains_key(&next_leaf_path));
                self.lss_branch[j].entry(next_leaf_path).and_modify(|predecessor| {
                    if *predecessor == old_predecessor {
                        //else there is a left child
                        *predecessor = new_predecessor;
                    }
                });
            }
            if self.lss_leaf.contains_key(&(next_leaf_path << 1)) {
                next_leaf_path = next_leaf_path << 1;
            } else {
                next_leaf_path = (next_leaf_path << 1) + DataType::from(1);
                debug_assert!(self.lss_leaf.contains_key(&next_leaf_path));
            }
            self.lss_leaf.entry(next_leaf_path).and_modify(|(predecessor, _elements)| {
                if *predecessor == old_predecessor {
                    *predecessor = new_predecessor;
                }
            });
            //predecessor of next leaf is set
            *set_leaf_predecessor = false;
        }
    }

    pub fn test(&self, other: YFT){
        for (path, (predecessor, _elements)) in self.lss_leaf.iter() {
            debug_assert!(*predecessor == other.lss_leaf.get(&path).unwrap().0);
        }
        for i in 0 .. self.lss_branch.len() {
            for (path, predecessor) in self.lss_branch[i].iter() {
                debug_assert!(*predecessor == *other.lss_branch[i].get(&path).unwrap());
            }
        }
        for (pos, predecessor) in self.lss_top.iter().enumerate() {
            debug_assert!(*predecessor == other.lss_top[pos]);
        }
    }

    pub fn test_predecessors(&self, values: Vec<DataType>) {
        for (_path, (predecessor, _elements)) in self.lss_leaf.iter() {
            debug_assert!(*predecessor == DataType::max_value() || values.binary_search(predecessor).is_ok());
        }
        for level in self.lss_branch.iter() {
            for (_path, predecessor) in level {
                debug_assert!(*predecessor == DataType::max_value() || values.binary_search(predecessor).is_ok());
            }
        }
        for predecessor in self.lss_top.iter() {
            debug_assert!(*predecessor == DataType::max_value() || values.binary_search(predecessor).is_ok());
        }
    }



    ///prints number of elements + relative fill level per lss level
    pub fn print_stats(&self, log: &Log) {
        log.print_result(format!("start_level={}\tnormal_levels={}\ttop_levels={}", self.start_level, self.lss_branch.len() + 1, self.last_level_len));
        let mut len = self.lss_leaf.len();
        let mut count = len;
        log.print_result(format!("level=0\tnodes={}\trelative_to_capacity={}", len, len as f32 / 2f32.powf((BIT_LENGTH - self.start_level) as f32)));
        for level in 1..self.lss_branch.len() + 1 {
            len = self.lss_branch[level - 1].len();
            log.print_result(format!("level={}\tnodes={}\trelative_to_capacity={}", level, len, len as f32 / 2f32.powf((BIT_LENGTH - self.start_level - level) as f32)));
            count += self.lss_branch[level - 1].len();
        }
        log.print_result(format!("level=-1\tnodes={}", count));
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

    fn lss_top_position(value: &DataType, lss_top_height: usize) -> usize {
        usize::from(*value) >> (BIT_LENGTH - lss_top_height)
    }

    pub fn contains(&self, query: DataType) -> bool {
        self.predecessor(query + 1 as u32) == Some(query)
    }

    //query may not belong to existing node
    pub fn predecessor(&self, query: DataType) -> Option<DataType> {
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
                    Some((predecessor, elements)) => {
                        return self.predecessor_from_array(query, predecessor, elements);
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
                Some((predecessor, elements)) => {
                    return self.predecessor_from_array(query, predecessor, elements);
                }
                None => {
                    None
                }
            }
        } else {
            match self.lss_branch[search_range.0 - 1].get(&calc_path(query, search_range.0, self.start_level)) {
                Some(predecessor) => {
                    return if *predecessor != DataType::max_value() { Some(*predecessor) } else { None };
                }
                None => {
                    None
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
            return if predecessor != DataType::max_value() { Some(predecessor) } else { None };
        }
    }

    fn predecessor_from_array(&self, query: DataType, predecessor: &DataType, elements: &Vec<SmallType>) -> Option<DataType> {
        let pos = match elements.binary_search(&SmallType::from(query)) {
            Ok(pos) => pos,
            Err(pos) => pos
        };
        return if pos == 0 {
            //test next value greater than search one
            debug_assert!(if let Some(successor) = elements.get(usize::from(pos)) { successor >= &SmallType::from(query) } else { true });
            if *predecessor == DataType::max_value() {
                None
            } else {
                //test value smaller than searched one
                debug_assert!(predecessor < &query);
                Some(*predecessor)
            }
        } else {
            //test next value greater than search one
            debug_assert!(usize::from(pos) >= elements.len() || if let Some(successor) = elements.get(usize::from(pos)) { successor >= &SmallType::from(query) } else { true });
            //test value smaller than searched one
            debug_assert!(if let Some(predecessor) = elements.get(usize::from(pos - 1)) { predecessor < &SmallType::from(query) } else { true });
            //get prefix via query and append it to result
            Some(extend_suffix(query, unsafe { *elements.get_unchecked(pos - 1) }))
        };
    }


    /// position may not belong to existing node
    /// exit point (0 leaf, x level, 42 top, 43 begin)
    /// number of binary search steps
    /// number of hash table misses
    pub fn predecessor_with_stats(&self, query: DataType) -> (Option<DataType>, u32, u32, u32) {
        let mut search_steps = 0;
        let mut hash_miss = 0;
        //binary search lowest ancestor for some query
        // query 0 == lss_leaf, query len()+1 == lss_top
        let mut search_range = (0, self.lss_branch.len() + 1);
        while search_range.0 != search_range.1 {
            search_steps += 1;
            let mut search_position = (search_range.0 + search_range.1) / 2;
            if search_position == self.lss_branch.len() + 1 {
                //top level may only be used iff there are no existing nodes below in search path
                search_position -= 1;
            }

            if search_position == 0 {
                //leaf level
                match self.lss_leaf.get(&calc_path(query, search_position, self.start_level)) {
                    Some((predecessor, elements)) => {
                        return (self.predecessor_from_array(query, predecessor, elements), 0, search_steps, hash_miss);
                    }
                    None => {
                        hash_miss += 1;
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
                        hash_miss += 1;
                        //there is no node -> search higher
                        search_range = (search_position + 1, search_range.1);
                    }
                }
            }
        }

        //search range includes now exact the lowest existing node, if there's one

        if search_range.0 == self.lss_branch.len() + 1 {
            //case there is no existing node -> look @ lss_top
            return (self.predec_lss_top(query), 42, search_steps, hash_miss);
        }

        if search_range.0 == 0 {
            //leaf level
            match self.lss_leaf.get(&calc_path(query, search_range.0, self.start_level)) {
                Some((predecessor, elements)) => {
                    return (self.predecessor_from_array(query, predecessor, elements), 0, search_steps, hash_miss);
                }
                None => {
                    return (None, 0, search_steps, hash_miss);
                }
            }
        } else {
            match self.lss_branch[search_range.0 - 1].get(&calc_path(query, search_range.0, self.start_level)) {
                Some(predecessor) => {
                    return if *predecessor != DataType::max_value() { (Some(*predecessor), search_range.0 as u32, search_steps, hash_miss) } else { (None, search_range.0 as u32, search_steps, hash_miss) };
                }
                None => {
                    return (None, 0, search_steps, hash_miss);
                }
            }
        }
    }
} //impl YFT

fn extend_suffix(preffix_source: DataType, suffix: SmallType) -> DataType {
    DataType::from((usize::from(suffix)) | ((usize::from(preffix_source) >> SMALL_TYPE_LEN) << SMALL_TYPE_LEN))
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