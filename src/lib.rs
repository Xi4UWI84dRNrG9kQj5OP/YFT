#![feature(allocator_api)]
#![feature(shrink_to)]
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate bitflags;
extern crate uint;
extern crate stats_alloc;

pub mod yft40_rust_hash;
pub mod yft40_fx_hash;
pub mod yft40_fnv_hash;
pub mod yft40_fx_hash_no_level;
pub mod yft40_fx_hash_alt;
pub mod yft40_fx_hash_new;
pub mod predecessor_set;
pub mod nmbrsrc;
pub mod log;
pub mod args;
pub mod extern_pred_search;

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use args::{Args, ValueSrc};
    use uint::u40;

    #[test]
    fn test() { //TODO abdeckende test, statt dieser eher zufälligen
        let args = Args {
            values: ValueSrc::Uniform { length: 0 },
            min_start_level: 10,
            compress: false,
            search_stats: false,
            element_length_test: false,
            hash_map: 1,
            memory: false,
            run_name: None,
            print: false,
            queries: None,
            result: false,
            store: None,
            time: false,
            u40: false,
            min_load_factor_difference: 99,
            min_start_level_load_factor: 1,
            max_last_level_load_factor: 99,
            max_lss_level: 8,
        };
        let mut log = log::Log::new(String::from("Test"));


        let mut values = vec![1, 2, 3, 100, 1000, 10000, 100000, 1000000, 10000000, 1099511627774].iter().map(|v: &u64| u40::from(*v)).collect();
        let mut queries: Vec<u40> = vec![0, 1, 1099511627774, 1099511627775].iter().map(|v : &u64| u40::from(*v)).collect();

        let mut results = Vec::new();
        for query in &queries {
            results.push(extern_pred_search::bin_search_pred(&values, *query));
        }

        {
            let yft = yft40_fx_hash_new::YFT::new(values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft.predecessor(*query), results[pos]);
            }
        }

        {
            let yft = yft40_fx_hash::YFT::new(values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft.predecessor(*query), results[pos]);
            }
        }

        {
            let yft = yft40_fx_hash_alt::YFT::new(values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft.predecessor(*query), results[pos]);
            }
        }

        {
            let yft = yft40_fx_hash_no_level::YFT::new(values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft.predecessor(*query), results[pos]);
            }
        }

        {
            let yft = yft40_fnv_hash::YFT::new(values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft.predecessor(*query), results[pos]);
            }
        }

        {
            let yft = yft40_rust_hash::YFT::new(values.clone(), &args, &mut log);

            for (pos, query) in queries.iter().enumerate() {
                assert_eq!(yft.predecessor(*query), results[pos]);
            }
        }
    }
}