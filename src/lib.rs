#![feature(allocator_api)]
#![feature(shrink_to)]
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate bitflags;
extern crate uint;
extern crate stats_alloc;

/// this file will be called by "cargo test"
/// It contains some correctness tests for different yft implementations
/// see main.rs for "cargo run"

pub mod yft64;
pub mod yft40_rust_hash;
pub mod yft40sn_fx_hash;
pub mod yft40bn_fx_hash;
pub mod yft40bo_fx_hash;
pub mod yft40so_fx_hash_binsearch;
pub mod yft40so_fnv_binsearch;
pub mod yft40so_fnv_bin_weight;
pub mod yft40so_rust_hash_binsearch;
pub mod yft40so_im_binsearch;
pub mod yft40so_boomphf_binsearch;
pub mod yft40so_boomphf_para_binsearch;
pub mod yft40so_fx_hash_linsearch;
pub mod yft40so_fx_hash_small_groups;
pub mod yft40_hash_brown;
pub mod yft40_im_hash;
pub mod yft40_boomphf_hash;
pub mod yft40_boomphf_hash_para;
pub mod yft40_fx_hash_bottom_up_construction;
pub mod yft40_fx_hash_capacity;
pub mod yft40_no_level_suc;
pub mod yft40_no_level_bin_suc;
pub mod yft40_no_level_bin;
pub mod yft40_fnv_hash;
pub mod yft40so_fnv_small_groups;
pub mod yft40sn_fnv;
pub mod yft40sn_bin_fnv;
pub mod yft40_split;
pub mod yft40_split_small;
pub mod yft40_split_small_leaf_search;
pub mod yft64_split_small_32;
pub mod predecessor_set;
pub mod nmbrsrc;
pub mod log;
pub mod args;
pub mod vec_search;

#[cfg(test)]
mod tests {
    use super::*;
    use args::{Args, ValueSrc};
    use uint::u40;

    #[test]
    fn test() {
        let args = Args {
            values: ValueSrc::Uniform { length: 0 },
            min_start_level: 10,
            search_stats: false,
            element_length_test: false,
            fixed_leaf_level: Some(8),
            fixed_top_level: Some(32),
            implementation: 1,
            bin_middle: 30,
            memory: false,
            run_name: None,
            add: None,
            delete: None,
            queries: None,
            result: false,
            store: None,
            time: false,
            u40: true,
            min_load_factor_difference: 99,
            min_start_level_load_factor: 1,
            max_last_level_load_factor: 99,
            max_lss_level: 8,
        };
        let mut log = log::Log::new(String::from("Test"));


        let values1 = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39,
            40, 701, 702, 703, 704, 705, 706, 707, 708, 709, 710, 711, 712, 713, 714, 715, 716, 717, 718, 719, 720, 721, 722, 723, 724, 725, 726, 727, 728, 729, 730, 731, 732, 733, 734, 735, 736, 737, 738, 739,
            8589934593, 8589934595, 8589934597, 8589934599, 8589934601, 8589934603, 8589934605, 8589934607, 8589934609, 8589934611, 8589934613, 8589934615, 8589934617, 8589934619, 8589934621, 8589934623, 8589934625, 8589934627, 8589934629, 8589934631, 8589934633, 8589934635, 8589934637, 8589934639, 8589934641, 8589934643, 8589934645, 8589934647, 8589934649, 8589934651, 8589934653, 8589934655, 8589934657, 8589934659, 8589934661, 8589934663, 8589934665, 8589934667, 8589934669, 8589934671,
            10804527104, 10804527204, 10804527304, 10804527404, 10804527504, 10804527604, 10804527704, 10804527804, 10804527904, 10804528004, 10804528104, 10804528204, 10804528304, 10804528404, 10804528504, 10804528604, 10804528704, 10804528804, 10804528904, 10804529004, 10804529104, 10804529204, 10804529304, 10804529404, 10804529504, 10804529604, 10804529704, 10804529804, 10804529904, 10804530004, 10804530104, 10804530204, 10804530304, 10804530404, 10804530504, 10804530604, 10804530704, 10804530804, 100804530904, 110804531004,
            1099511627774]
            .iter().map(|v: &u64| u40::from(*v)).collect();
        let values2 = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
            41, 701, 702, 703, 704, 705, 706, 707, 708, 709, 710, 711, 712, 713, 714, 715, 716, 717, 718, 719, 720, 721, 722, 723, 724, 725, 726, 727, 728, 729, 730, 731, 732, 733, 734, 735, 736, 737, 738, 739,
            8589934593, 8589934595, 8589934597, 8589934599, 8589934601, 8589934603, 8589934605, 8589934607, 8589934609, 8589934611, 8589934613, 8589934615, 8589934617, 8589934619, 8589934621, 8589934623, 8589934625, 8589934627, 8589934629, 8589934631, 8589934633, 8589934635, 8589934637, 8589934639, 8589934641, 8589934643, 8589934645, 8589934647, 8589934649, 8589934651, 8589934653, 8589934655, 8589934657, 8589934659, 8589934661, 8589934663, 8589934665, 8589934667, 8589934669, 8589934671,
            10804527104, 10804527204, 10804527304, 10804527404, 10804527504, 10804527604, 10804527704, 10804527804, 10804527904, 10804528004, 10804528104, 10804528204, 10804528304, 10804528404, 10804528504, 10804528604, 10804528704, 10804528804, 10804528904, 10804529004, 10804529104, 10804529204, 10804529304, 10804529404, 10804529504, 10804529604, 10804529704, 10804529804, 10804529904, 10804530004, 10804530104, 10804530204, 10804530304, 10804530404, 10804530504, 10804530604, 10804530704, 10804530804, 100804530904, 110804531004,
            1099511627774, 1099511627775]
            .iter().map(|v: &u64| u40::from(*v)).collect();
        let mut rnd_values = nmbrsrc::get_uniform_dist(32768);
        let rnd_queries: Vec<u40> = nmbrsrc::get_uniform_dist(32768);
        let mut queries: Vec<u40> = vec![
            0, 1, 39,
            40, 41, 256, 257, 701, 702, 739, 740,
            4294967295, 4294967296, 4294967297,    //grenze top level (2^32)
            8589934592, 8589934593, 8589934594, 8589934671, 8589934672,
            17179869183, 17179869184, 17179869185,
            10804527103, 10804527104, 10804527105, 10804527144, 10804527200, 10804530804, 10804530805, 50804530804, 100804530903, 100804530904, 100804530905, 100804530950, 100804531004, 100804531005,
            549755813888, 1000000000000,
            1099511627773, 1099511627774, 1099511627775].iter().map(|v: &u64| u40::from(*v)).collect();
        queries.extend(&rnd_queries);

        let mut results_1 = Vec::new();
        let mut results_2 = Vec::new();
        let mut results_r = Vec::new();
        for query in &queries {
            results_1.push(vec_search::rust_bin_search_pred(&values1, *query));
            results_2.push(vec_search::rust_bin_search_pred(&values2, *query));
            results_r.push(vec_search::rust_bin_search_pred(&rnd_values, *query));
        }

        {
            let yft1 = yft40sn_fx_hash::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40sn_fx_hash::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40sn_fx_hash::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40_fnv_hash::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40_fnv_hash::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40_fnv_hash::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40_rust_hash::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40_rust_hash::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40_rust_hash::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }


        {
            let yft1 = yft40so_fnv_binsearch::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40so_fnv_binsearch::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40so_fnv_binsearch::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40so_rust_hash_binsearch::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40so_rust_hash_binsearch::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40so_rust_hash_binsearch::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40so_boomphf_binsearch::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40so_boomphf_binsearch::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40so_boomphf_binsearch::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40so_fnv_bin_weight::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40so_fnv_bin_weight::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40so_fnv_bin_weight::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40so_fnv_small_groups::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40so_fnv_small_groups::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40so_fnv_small_groups::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40sn_fnv::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40sn_fnv::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40sn_fnv::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40sn_bin_fnv::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40sn_bin_fnv::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40sn_bin_fnv::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40_no_level_bin::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40_no_level_bin::YFT::new(values2.clone(), &args, &mut log);
            let yftr = yft40_no_level_bin::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }
        }

        {
            let yft1 = yft40_split_small_leaf_search::YFT::new(values1.clone(), &args, &mut log);
            let yft2 = yft40_split_small_leaf_search::YFT::new(values2.clone(), &args, &mut log);
            let mut yftr = yft40_split_small_leaf_search::YFT::new(rnd_values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft1.predecessor(*query), results_1[pos]);
                assert_eq!(yft2.predecessor(*query), results_2[pos]);
                assert_eq!(yftr.predecessor(*query), results_r[pos]);
            }

//             test add and delete

            for i in nmbrsrc::get_uniform_dist(32768) {
                yftr.add(i);
                debug_assert!(i == yftr.predecessor(i + u40::from(1)).unwrap());
                rnd_values.push(i);
            }
            rnd_values.sort();
            for query in queries.iter() {
                assert_eq!(yftr.predecessor(*query), vec_search::rust_bin_search_pred(&rnd_values, *query));
            }

            for i in (0..1000).rev() {
                let removed = rnd_values[i];
                yftr.remove(removed);
                let removed2 = rnd_values.remove(i);
                debug_assert!(removed == removed2);
            }
//
//            yftr.test_predecessors(rnd_values.clone());

            for query in queries.iter() {
                assert_eq!(yftr.predecessor(*query), vec_search::rust_bin_search_pred(&rnd_values, *query));
            }
        }
    }
}