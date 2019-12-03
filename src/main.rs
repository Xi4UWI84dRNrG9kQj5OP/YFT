#![feature(allocator_api)]
#![feature(shrink_to)]
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate bitflags;
extern crate uint;
extern crate stats_alloc;
extern crate im_rc;

/// Main module
/// all "cargo run" calls go through this code
/// "cargo test" calls we go trough lib.rs
/// see Args.rs for more Information about command line Arguments

pub use yft64::YFT;
use structopt::StructOpt;
use uint::u40;
use args::Args;
use args::ValueSrc;
use std::collections::BTreeSet;

pub mod yft64;
pub mod yft40_rust_hash;
pub mod yft40sn_fx_hash;
pub mod yft40bn_fx_hash;
pub mod yft40bo_fx_hash;
pub mod yft40so_fx_hash_binsearch;
pub mod yft40so_fnv_binsearch;
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
pub mod yft40_no_level;
pub mod yft40_no_level_bin;
pub mod yft40_fnv_hash;
//pub mod yft40_fx_hash_comp;
pub mod predecessor_set;
pub mod nmbrsrc;
pub mod log;
pub mod args;
pub mod vec_search;

fn main() {
    let args = Args::from_args();
    println!("{:?}", args);

    let mut log =
        if let Some(name) = &args.run_name {
            log::Log::new(name.to_string())
        } else {
            log::Log::new(nmbrsrc::get_uniform_dist(1)[0].to_string())
        };

    //create memory logger if option is set
    if args.memory {
        log.set_log_mem();
    };
    log.log_mem("start");

    //create time logger if option is set
    if args.time {
        log.set_log_time();
    };


    //create yft input (u64, u40)
    let values: (Vec<usize>, Vec<u40>) =
        match &args.values {
            ValueSrc::Normal { length, mean, deviation } => {
                (nmbrsrc::get_normal_dist(*length, *mean as f64, *deviation as f64), Vec::new())
            }
            ValueSrc::Uniform { length } => {
                (nmbrsrc::get_uniform_dist(*length), Vec::new())
            }
            ValueSrc::UniformRestricted { length, path } => {
                let values = nmbrsrc::load_u40_tim(path.to_str().unwrap()).unwrap();
                (nmbrsrc::get_uniform_dist_restricted(*length, usize::from(values[0]) - 1, usize::from(values[values.len() - 1]) + 1), Vec::new())
            }
            ValueSrc::UniformRestrictedF { length, path } => {
                let values = nmbrsrc::load_u40_fit(path.to_str().unwrap()).unwrap();
                (nmbrsrc::get_uniform_dist_restricted(*length, usize::from(values[0]) - 1, usize::from(values[values.len() - 1]) + 1), Vec::new())
            }
            ValueSrc::Poisson { length, lambda } => {
                (nmbrsrc::get_poisson_dist(*length, *lambda), Vec::new())
            }
            ValueSrc::PowerLaw { length, n } => {
                (nmbrsrc::get_power_law_dist(*length, *n), Vec::new())
            }
            ValueSrc::Load { path } => {
                (nmbrsrc::load(path.to_str().unwrap()).unwrap(), Vec::new())
            }
            ValueSrc::U40T { path } => {
                (Vec::new(), nmbrsrc::load_u40_tim(path.to_str().unwrap()).unwrap())
            }
            ValueSrc::U64S { path } => {
                (nmbrsrc::load_u64_serialized(path.to_str().unwrap()).unwrap(), Vec::new())
            }
            ValueSrc::U40 { path } => {
                (Vec::new(), nmbrsrc::load_u40_fit(path.to_str().unwrap()).unwrap())
            }
            ValueSrc::U40S { path } => {
                (Vec::new(), nmbrsrc::load_u40_serialized(path.to_str().unwrap()).unwrap())
            }
        };

    //save input if option is set
    if let Some(ref file) = args.store {
        //generated values are alway u64
        if let Err(e) = nmbrsrc::save(&values.0, file.to_str().unwrap()) {
            dbg!(e);
        }
    }

    if !args.element_length_test {
        run_yft(&args, &mut log, values);
    } else {
        for i in 0..40 { //for element length test, else ignored
            let iteration_values;
            //decrease number of elements if option is set
            if values.0.len() == 0 {
                iteration_values = (Vec::new(), values.1.iter().step_by(2usize.pow(i)).map(|v| v.clone()).collect());
                if values.1.len() < 2 {
                    break;
                }
            } else {
                iteration_values = (values.0.iter().step_by(2usize.pow(i)).map(|v| v.clone()).collect(), Vec::new());
                if values.0.len() < 2 {
                    break;
                }
            }
            //log is not used between begin of for loop and here -> no problems
            log.inc_run_number();

            run_yft(&args, &mut log, iteration_values);
        }
        {} // end for
    }
    log.log_mem("end");
}

fn run_yft(args: &Args, mut log: &mut log::Log, values: (Vec<usize>, Vec<u40>)) {
    log.log_mem("values loaded").log_time("values loaded");
    {
        if args.hash_map == 100 { //binary search
            let values = get_u40_values(values);

            //print stats
            log.print_result(format!("level=-1\telements={}", values.len()));
            log.log_mem("initialized").log_time("initialized");

            query(&|q| vec_search::rust_bin_search_pred(&values, q), &args, &mut log);
        } else if args.hash_map == 101 { //btree
            // performance is so bad, that possible improvement with u40 won't help
            let values = get_usize_values(values);
            let set = &(&values).into_iter().fold(BTreeSet::new(), |mut set, value| {
                set.insert(value.clone());
                set
            });
            //print stats
            log.print_result(format!("level=-1\telements={}", values.len()));
            log.log_mem("initialized").log_time("initialized");

            query(&|q| vec_search::btree_search_pred(set, q), &args, &mut log);
        } else if args.hash_map == 102 { //mixed binary linear search
            let values = get_u40_values(values);

            //print stats
            log.print_result(format!("level=-1\telements={}", values.len()));
            log.log_mem("initialized").log_time("initialized");

            query(&|q| vec_search::mixed_search_pred(&values, q, args.min_start_level), &args, &mut log);
        } else if args.u40 {
            let values = get_u40_values(values);

            if args.search_stats {
                if args.hash_map != 21 {
                    panic!("search stats can not be made with -h {} and -u option", args.hash_map);
                }
                if let Some(ref file) = args.queries {
                    let yft = yft40so_fx_hash_binsearch::YFT::new(values, &args, &mut log);
                    let test_values: Vec<u40> = nmbrsrc::load(file.to_str().unwrap()).unwrap().into_iter().map(|v| u40::from(v)).collect();
                    let number = test_values.len();
                    log.log_time(&format!("queries loaded\tqueries={}", number));
                    let mut stats = vec![vec![0; 44]; 44];
                    let _: Vec<u40> = test_values.into_iter().map(|v| {
                        let (r, e, c) = yft.predecessor_with_stats(v);
                        stats[e as usize][c as usize] += 1;
                        r.unwrap_or(u40::from(0))
                    }).collect();
                    for e in 0..43 {
                        for c in 0..43 {
                            if stats[e][c] > 0 {
                                log.print_result(format!("Exit={}\tSearchSteps={}\tfrequency={}", e, c, stats[e][c]));
                            }
                        }
                    }
                    log.log_time(&format!("queries processed\tnumber={}", number));
                } else {
                    panic!("search stats requires query file (-q)");
                }
            } else {
                //macro to load & test yft
                macro_rules! testyft40 {
                    (  $yft:ty; $values:expr ) => {
                        {
                            let yft =  <$yft>::new($values, &args, &mut log);

                            log.log_mem("initialized").log_time("initialized");

                            query(&|q| yft.predecessor(q), &args, &mut log);
                            if args.memory {
                                yft.print_stats(&log);
                            }
                        }
                    };
                }

                match args.hash_map {
                    0 => testyft40!(yft40_rust_hash::YFT; values),
                    1 => testyft40!(yft40sn_fx_hash::YFT; values),
                    2 => testyft40!(yft40_hash_brown::YFT; values),
                    3 => testyft40!(yft40_im_hash::YFT; values),
                    4 => testyft40!(yft40_boomphf_hash::YFT; values),
                    5 => testyft40!(yft40_boomphf_hash_para::YFT; values),
                    6 => testyft40!(yft40_fx_hash_bottom_up_construction::YFT; values),
                    7 => testyft40!(yft40_fx_hash_capacity::YFT; values),
                    8 => testyft40!(yft40_no_level::YFT; values),
                    9 => testyft40!(yft40_fnv_hash::YFT; values),
                    10 => testyft40!(yft40bn_fx_hash::YFT; values),
                    20 => testyft40!(yft40bo_fx_hash::YFT; values),
                    21 => testyft40!(yft40so_fx_hash_binsearch::YFT; values),
                    22 => testyft40!(yft40so_fx_hash_linsearch::YFT; values),
                    23 => testyft40!(yft40so_fnv_binsearch::YFT; values),
                    24 => testyft40!(yft40so_rust_hash_binsearch::YFT; values),
                    25 => testyft40!(yft40so_boomphf_binsearch::YFT; values),
                    26 => testyft40!(yft40so_boomphf_para_binsearch::YFT; values),
                    27 => testyft40!(yft40so_fx_hash_small_groups::YFT; values),
                    28 => testyft40!(yft40so_im_binsearch::YFT; values),
                    _ => panic!("Invalid input for argument hash_map")
                }
            }
        } else {
            if args.hash_map != 1 {
                eprintln!("Hashmap Parameter is ignored in usize mod\n Use -u Parameter!");
            }
            let yft = YFT::new(get_usize_values(values), &args, &mut log);

            log.log_mem("initialized").log_time("initialized");

            //load queries & apply them, if option is set
            if let Some(ref file) = args.queries {
                if args.search_stats {
                    let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                    let number = test_values.len();
                    let mut stats = vec![vec![0; 44]; 44];
                    let _: Vec<usize> = test_values.into_iter().map(|v| {
                        let (r, e, c) = yft.predecessor_with_stats(v);
                        stats[e as usize][c as usize] += 1;
                        r.unwrap_or(0)
                    }).collect();
                    for e in 0..43 {
                        for c in 0..43 {
                            if stats[e][c] > 0 {
                                log.print_result(format!("Exit={}\tSearchSteps={}\tfrequency={}", e, c, stats[e][c]));
                            }
                        }
                    }
                    log.log_time(&format!("queries processed\tnumber={}", number));
                } else {
                    query(&|q| yft.predecessor(q), &args, &mut log);
                }
            }
            if args.memory {
                yft.print_stats(&log);
            }
        };
        //yft mem is freed here
    }
}

//load queries & apply them, if option is set
fn query<T: From<usize> + std::fmt::Debug>(f: &dyn Fn(T) -> Option<T>, args: &Args, log: &mut log::Log) {
    if let Some(ref file) = args.queries {
        let queries: Vec<T> = nmbrsrc::load(file.to_str().unwrap()).unwrap().into_iter().map(|v| T::from(v)).collect();
        let number = queries.len();
        log.log_time(&format!("queries loaded\tqueries={}", number));
        if args.result {
            for query in queries {
                println!("{:?}", f(query));
            }
        } else {
            for query in queries {
                f(query);
            }
        }
        log.log_time(&format!("queries processed\tqueries={}", number));
    }
}

fn get_u40_values(values: (Vec<usize>, Vec<u40>)) -> Vec<u40> {
    if values.0.len() == 0 {
        values.1
    } else {
        values.0.into_iter().map(|v| u40::from(v)).collect()
    }
}

fn get_usize_values(values: (Vec<usize>, Vec<u40>)) -> Vec<usize> {
    if values.0.len() == 0 {
        values.1.into_iter().map(|v| u64::from(v) as usize).collect()
    } else {
        values.0
    }
}