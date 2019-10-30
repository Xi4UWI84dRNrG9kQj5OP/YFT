#![feature(allocator_api)]
#![feature(shrink_to)]
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate bitflags;
extern crate uint;
extern crate stats_alloc;
extern crate im_rc;


pub use yft64::YFT;
use structopt::StructOpt;
use uint::u40;
use std::collections::BTreeSet;
use args::Args;
use args::ValueSrc;

pub mod yft64;
pub mod yft40_rust_hash;
pub mod yft40_fx_hash;
pub mod yft40_hash_brown;
pub mod yft40_im_hash;
pub mod yft40_boomphf_hash;
pub mod yft40_boomphf_hash_para;
pub mod yft40_fx_hash_bottom_up_construction;
pub mod yft40_fx_hash_capacity;
pub mod yft40_fx_hash_no_level;
pub mod yft40_fnv_hash;
pub mod predecessor_set;
pub mod nmbrsrc;
pub mod log;
pub mod args;

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


    for i in 0..40 { //for element length test, else ignored
        //create yft input (u64, u40)
        let mut values: (Vec<usize>, Vec<u40>) =
            match &args.values {
                ValueSrc::Normal { length, mean, deviation } => {
                    (nmbrsrc::get_normal_dist(*length, *mean as f64, *deviation as f64), Vec::new())
                }
                ValueSrc::Uniform { length } => {
                    (nmbrsrc::get_uniform_dist(*length), Vec::new())
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

        if args.element_length_test && i > 0 {
            //decrease number of elements
            if values.0.len() == 0 {
                values = (values.0, values.1.iter().step_by(2usize.pow(i)).map(|v| v.clone()).collect());
                if values.1.len() < 2 {
                    break;
                }
            } else {
                values = (values.0.iter().step_by(2usize.pow(i)).map(|v| v.clone()).collect(), values.1);
                if values.0.len() < 2 {
                    break;
                }
            }
            //log is not used between begin of for loop and here -> no problems
            log.inc_run_number();
        }

        log.log_mem("values loaded").log_time("values loaded");

        {
            if args.hash_map == 100 {
                let values = get_usize_values(values);

                log.log_mem("initialized").log_time("initialized");

                //print stats
                log.print_result(format!("level=-1\telements={}", values.len()));

                //load queries & apply them, if option is set
                if let Some(ref file) = args.queries {
                    let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                    let number = test_values.len();
                    if let Some(ref output) = args.output {
                        let predecessors = &test_values.into_iter().map(|v| bin_search_pred(&values, v).unwrap_or(0)).collect();
                        if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                            dbg!(e);
                        }
                    } else {
                        let _: Vec<usize> = test_values.into_iter().map(|v| bin_search_pred(&values, v).unwrap_or(0)).collect();
                    }
                    log.log_time(&format!("queries processed\tnumber={}", number));
                }
            } else if args.hash_map == 101 {
                let values = get_usize_values(values);
                let set = &(&values).into_iter().fold(BTreeSet::new(), |mut set, value| {
                    set.insert(value.clone());
                    set
                });
                log.log_mem("initialized").log_time("initialized");

                //print stats
                log.print_result(format!("level=-1\telements={}", values.len()));

                //load queries & aply them, if option is set
                if let Some(ref file) = args.queries {
                    let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                    let number = test_values.len();
                    if let Some(ref output) = args.output {
                        let predecessors = &test_values.into_iter().map(|v| btree_search_pred(set, v).unwrap_or(0)).collect();
                        if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                            dbg!(e);
                        }
                    } else {
                        let _: Vec<usize> = test_values.into_iter().map(|v| btree_search_pred(set, v).unwrap_or(0)).collect();
                    }
                    log.log_time(&format!("queries processed\tnumber={}", number));
                }
            } else if args.u40 {
                //macro to load & test yft
                macro_rules! testyft40 {
                    (  $yft:ty; $values:expr ) => {
                        {
                            let yft =  <$yft>::new($values, &args, &mut log);

                            log.log_mem("initialized").log_time("initialized");

                            //load queries & aply them, if option is set
                            if let Some(ref file) = args.queries {
                                let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                                let number = test_values.len();
                                if let Some(ref output) = args.output {
                                    let predecessors = &test_values.into_iter().map(|v| usize::from(yft.predecessor(u40::from(v)).unwrap_or(u40::from(0)))).collect();
                                    if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                                        dbg!(e);
                                    }
                                } else {
                                    let _: Vec<usize> = test_values.into_iter().map(|v| usize::from(yft.predecessor(u40::from(v)).unwrap_or(u40::from(0)))).collect();
                                }
                                log.log_time(&format!("queries processed\tnumber={}", number));
                            }
                            if args.memory {
                                yft.print_stats(&log);
                            }
                        }
                    };
                }

                let values = if values.0.len() == 0 {
                    values.1
                } else {
                    values.0.into_iter().map(|v| u40::from(v)).collect()
                };

                match args.hash_map {
                    0 => testyft40!(yft40_rust_hash::YFT; values),
                    1 => testyft40!(yft40_fx_hash::YFT; values),
                    2 => testyft40!(yft40_hash_brown::YFT; values),
                    3 => testyft40!(yft40_im_hash::YFT; values),
                    4 => testyft40!(yft40_boomphf_hash::YFT; values),
                    5 => testyft40!(yft40_boomphf_hash_para::YFT; values),
                    6 => testyft40!(yft40_fx_hash_bottom_up_construction::YFT; values),
                    7 => testyft40!(yft40_fx_hash_capacity::YFT; values),
                    8 => testyft40!(yft40_fx_hash_no_level::YFT; values),
                    9 => testyft40!(yft40_fnv_hash::YFT; values),
                    _ => panic!("Invalid input for argument hash_map")
                }
            } else {
                if args.hash_map != 1 {
                    eprintln!("Hashmap Parameter is ignored in usize mod\n Use -u Parameter!");
                }
                let yft = YFT::new(get_usize_values(values), &args, &mut log);

                log.log_mem("initialized").log_time("initialized");

                //load queries & aply them, if option is set
                if let Some(ref file) = args.queries {
                    let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                    let number = test_values.len();
                    if let Some(ref output) = args.output {
                        let predecessors = &test_values.into_iter().map(|v| yft.predecessor(v).unwrap_or(0)).collect();
                        if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                            dbg!(e);
                        }
                    } else {
                        if args.search_stats {
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
                        } else {
                            let _: Vec<usize> = test_values.into_iter().map(|v| yft.predecessor(v).unwrap_or(0)).collect();
                        }
                    }
                    log.log_time(&format!("queries processed\tnumber={}", number));
                }
                if args.memory {
                    yft.print_stats(&log);
                }
            };
            //yft mem is freed here
        }
        if !args.element_length_test {
            break;
        }
    }
    {} // end for
    log.log_mem("end");
}

fn get_usize_values(values: (Vec<usize>, Vec<u40>)) -> Vec<usize> {
    let values =
        if values.0.len() == 0 {
            values.1.into_iter().map(|v| u64::from(v) as usize).collect()
        } else {
            values.0
        };
    values
}

///binary search predecessor
fn bin_search_pred(element_list: &Vec<usize>, element: usize) -> Option<usize> {
    let mut l = 0;
    let mut r = element_list.len() - 1;

    while l != r {
        let m = (l + r) / 2;
        if element_list[m] < element {
            r = m
        } else {
            l = m + 1;
        }
    }

    if element >= element_list[l] {
        Some(element_list[l])
    } else {
        None
    }
}

///search predecessor with BTree
fn btree_search_pred(set: &BTreeSet<usize>, element: usize) -> Option<usize> {
    Some(*set.range(0..element).last()?)
}