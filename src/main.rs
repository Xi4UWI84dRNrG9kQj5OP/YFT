#![feature(allocator_api)]
#![feature(shrink_to)]
extern crate structopt;
#[macro_use]
extern crate bitflags;
extern crate uint;
extern crate stats_alloc;
extern crate im_rc;


pub use yft64::YFT;
use std::path::PathBuf;
use structopt::StructOpt;
use uint::u40;

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
pub mod predecessor_set;
pub mod nmbrsrc;
pub mod log;

/// Y-Fast-Trie Test Implementation
#[derive(StructOpt, Debug)]
#[structopt(name = "YFT", about = "Test Implementation of Dan Willard's Y-Fast-Trie")]
struct Args {
    /// Source, where values should come from.
    /// Either a Distribution that should be used to generate the Y-Fast-Trie Input or a file to load them.
    #[structopt(subcommand)]
    values: ValueSrc,
    /// Minimal height of lowest lss level
    #[structopt(short = "a", long, default_value = "10")]
    min_start_level: usize,
    /// Use binary search instead of Y-Fast-Trie
    #[structopt(short, long)]
    bin_search: bool,
    /// Evaluate the predecessor search steps; not compatible with u40 or output at this momement.
    #[structopt(short = "d", long)]
    search_stats: bool,
    /// Run multiple times, each time with half much elements than before
    #[structopt(short, long)]
    element_length_test: bool,
    /// Hashmap that should be use. Only usable with u40 Option. Not compatible with values Option.
    /// 0 = std
    /// 1 = Fx
    /// 2 = Hashbrown
    /// 3 = im-rc
    /// 4 = boomphf
    /// 5 = boomphf parallel construction
    /// 6 = Fx bottom up construction
    /// 7 = Fx capacity construction
    /// 8 = Fx no level
    #[structopt(short, long, default_value = "1")]
    hash_map: usize,
    /// Log memory usage
    #[structopt(short, long)]
    memory: bool,
    /// Name of this run. Used for logging. If not set, a random number is used.
    #[structopt(short = "n", long)]
    run_name: Option<String>,
    /// File where results should be saved to
    /// If there is no predecessor, 0 will be printed
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
    /// Print yft to outline
    #[structopt(short, long)]
    print: bool,
    /// File with predecessor queries
    #[structopt(short, long, parse(from_os_str))]
    queries: Option<PathBuf>,
    /// A file where randomly generated Values from this run should be saved to
    #[structopt(short, long, parse(from_os_str))]
    store: Option<PathBuf>,
    /// Log time
    #[structopt(short, long)]
    time: bool,
    /// Use 40 bit integer
    #[structopt(short, long)]
    u40: bool,
    /// Minium Number of Elements in first lss level relative to the input in percentage (do not write the % char)
    /// Must be beewtwin 1 and 100.
    #[structopt(short = "x", long, default_value = "50")]
    min_start_level_load_factor: usize,
    /// Minium Number of Elements in first lss level relative to the input in percentage (do not write the % char)
    /// Must be beewtwin 1 and 100.
    #[structopt(short = "y", long, default_value = "90")]
    max_last_level_load_factor: usize,
    /// Maximum number of lss levels
    #[structopt(short = "z", long, default_value = "8")]
    max_lss_level: usize,
}

// arg subcommand for number generation
#[derive(Debug)] //this should not be necessary
#[derive(StructOpt)]
enum ValueSrc {
    Normal {
        length: usize,
        mean: usize,
        deviation: usize,
    },
    Uniform {
        length: usize,
    },
    Poisson {
        length: usize,
        lambda: f64,
    },
    PowerLaw {
        length: usize,
        n: f64,
    },
    /// A file with ordered Numbers to create the Y-Fast-Trie
    Load {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    /// A file with ordered u40 Numbers to create the Y-Fast-Trie
    U40 {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
}

fn main() {
    let args = Args::from_args();
    println!("{:?}", args);

    let mut log =
        if let Some(name) = args.run_name {
            log::Log::new(name)
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


    for i in 0..40 { //for element length test, else ignored //TODO change log
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
                ValueSrc::U40 { path } => {
                    (Vec::new(), nmbrsrc::load_u40_fit(path.to_str().unwrap()).unwrap())
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
            match args.values {
                ValueSrc::U40 { path: _ } => {
                    values = (values.0, values.1.iter().step_by(2usize.pow(i)).map(|v| v.clone()).collect());
                    if values.1.len() < 2 {
                        break;
                    }
                }
                _ => {
                    values = (values.0.iter().step_by(2usize.pow(i)).map(|v| v.clone()).collect(), values.1);
                    if values.0.len() < 2 {
                        break;
                    }
                }
            }
            //log is not used between begin of for loop and here -> no problems
            log.inc_run_number();
        }

        log.log_mem("values loaded").log_time("values loaded");

        {
            if args.bin_search {
                log.log_mem("initialized").log_time("initialized");
                //print stats
                log.print_result(format!("level=-1\telements={}", values.0.len()));
                //load queries & aply them, if option is set
                if let Some(ref file) = args.queries {
                    let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                    if let Some(ref output) = args.output {
                        let predecessors = &test_values.into_iter().map(|v| bin_search_pred(&values.0, v).unwrap_or(0)).collect();
                        if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                            dbg!(e);
                        }
                    } else {
                        let _: Vec<usize> = test_values.into_iter().map(|v| bin_search_pred(&values.0, v).unwrap_or(0)).collect();
                    }
                    log.log_time("queries processed");
                }
            } else if args.u40 {
                //macro to load & test yft
                macro_rules! testyft40 {
                    (  $yft:ty; $values:expr ) => {
                        {
                            let yft =  <$yft>::new($values, args.min_start_level, args.min_start_level_load_factor, args.max_lss_level, args.max_last_level_load_factor, &mut log);

                            log.log_mem("initialized").log_time("initialized");

                            //load queries & aply them, if option is set
                            if let Some(ref file) = args.queries {
                                let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                                if let Some(ref output) = args.output {
                                    let predecessors = &test_values.into_iter().map(|v| usize::from(yft.predecessor(u40::from(v)).unwrap_or(u40::from(0)))).collect();
                                    if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                                        dbg!(e);
                                    }
                                } else {
                                    let _: Vec<usize> = test_values.into_iter().map(|v| usize::from(yft.predecessor(u40::from(v)).unwrap_or(u40::from(0)))).collect(); //TODO time
                                }
                                log.log_time("queries processed");
                            }
                            if args.memory {
                                yft.print_stats(&log);
                            }
                        }
                    };
                }

                let values =
                    match args.values {
                        ValueSrc::U40 { path: _ } => {
                            values.1
                        }
                        _ => {
                            values.0.into_iter().map(|v| u40::from(v)).collect()
                        }
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
                    _ => panic!("Invalid input for argument hash_map")
                }
            } else {//TODO direkt args mitgeben?
                let yft = YFT::new(values.0, args.min_start_level, args.min_start_level_load_factor, args.max_lss_level, args.max_last_level_load_factor, &mut log);

                log.log_mem("initialized").log_time("initialized");

                //load queries & aply them, if option is set
                if let Some(ref file) = args.queries {
                    let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                    if let Some(ref output) = args.output {
                        let predecessors = &test_values.into_iter().map(|v| yft.predecessor(v).unwrap_or(0)).collect();
                        if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                            dbg!(e);
                        }
                    } else {
                        if args.search_stats {
                            let mut stats = vec![vec![0; 43]; 43];
                            let _: Vec<usize> = test_values.into_iter().map(|v| {
                                let (r, e, c) = yft.predecessor_with_stats(v);
                                stats[e as usize][c as usize] += 1;
                                r.unwrap_or(0)
                            }).collect();
                            for e in 0..43 {
                                for c in 0..43 {
                                    log.print_result(format!("Exit Point={}\tNumber of Bin Search Steps={}\tfrequency={}", e, c, stats[e][c]));
                                }
                            }
                        } else {
                            let _: Vec<usize> = test_values.into_iter().map(|v| yft.predecessor(v).unwrap_or(0)).collect();
                        }
                    }
                    log.log_time("queries processed");
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