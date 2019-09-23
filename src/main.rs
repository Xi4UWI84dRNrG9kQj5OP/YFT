#![feature(allocator_api)]
#![feature(shrink_to)]
extern crate structopt;
#[macro_use]
extern crate bitflags;
extern crate uint;
extern crate stats_alloc;


pub use yft64::YFT;
use std::path::PathBuf;
use structopt::StructOpt;
use uint::u40;

pub mod yft64;
pub mod yft40;
pub mod predecessor_set;
pub mod nmbrsrc;
pub mod log;

/// Y-Fast-Trie Test Implementation
#[derive(StructOpt, Debug)]
#[structopt(name = "YFT", about = "Test Implementation of Dan Willard's Y-Fast-Trie")]
struct Args {
    //    /// A file where a gnuplot should be made to
//    #[structopt(short, long, parse(from_os_str))]
//    gnuplot: Option<PathBuf>, //TODO
    /// Deviation that should be used to generate the Y-Fast-Trie Input.
    /// Alternately use load command
    #[structopt(subcommand)]
    dist: Option<Distribution>,
    /// Minimal height of lowest lss level
    #[structopt(short = "a", long, default_value = "10")]
    min_start_level: usize,
    /// Run multiple times, each time with half much elements than before
    #[structopt(short, long)]
    element_length_test: bool,
    /// A file with ordered Numbers to create the Y-Fast-Trie
    #[structopt(short, long, parse(from_os_str))]
    load: Option<PathBuf>,
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
    /// Maximum number of lss levels
    #[structopt(short = "z", long, default_value = "8")]
    max_lss_level: usize,
}

// arg subcommand for number generation
#[derive(Debug)] //this should not be necessary
#[derive(StructOpt)]
enum Distribution {
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
}


fn main() {
    let args = Args::from_args();
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
        //create yft input
        let mut values =
            if let Some(ref file) = args.load {
                nmbrsrc::load(file.to_str().unwrap()).unwrap()
            } else {
                if let Some(ref distribution) = args.dist {
                    match distribution {
                        Distribution::Normal { length, mean, deviation } => {
                            nmbrsrc::get_normal_dist(*length, *mean as f64, *deviation as f64)
                        }
                        Distribution::Uniform { length } => {
                            nmbrsrc::get_uniform_dist(*length)
                        }
                        Distribution::Poisson { length, lambda } => {
                            nmbrsrc::get_poisson_dist(*length, *lambda)
                        }
                    }
                } else {
                    panic!("Distribution or input File required!");
                }
            };


        //save input if option is set
        if let Some(ref file) = args.store {
            if let Err(e) = nmbrsrc::save(&values, file.to_str().unwrap()) {
                dbg!(e);
            }
        }

        if args.element_length_test && i > 0 {
            //half number of elements
            values = values.iter().step_by(2usize.pow(i)).map(|v| v.clone()).collect();
            if values.len() < 2 {
                break;
            }
        } else {
            if i > 1 {
                break;
            }
        }

        log.log_mem("values loaded").log_time("values loaded");

        {
            //create yft
            if args.u40 {
                let yft = yft40::YFT::new(values.into_iter().map(|v| u40::from(v)).collect(), args.min_start_level, args.max_lss_level, &mut log);

                log.log_mem("yft initialized").log_time("yft initialized");

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
            } else {
                let yft = YFT::new(values, args.min_start_level, args.max_lss_level, &mut log);

                log.log_mem("yft initialized").log_time("yft initialized");

                //load queries & aply them, if option is set
                if let Some(ref file) = args.queries {
                    let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                    if let Some(ref output) = args.output {
                        let predecessors = &test_values.into_iter().map(|v| yft.predecessor(v).unwrap_or(0)).collect();
                        if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                            dbg!(e);
                        }
                    } else {
                        let _: Vec<usize> = test_values.into_iter().map(|v| yft.predecessor(v).unwrap_or(0)).collect();
                    }
                    log.log_time("queries processed");
                }
                if args.memory {
                    yft.print_stats(&log);
                }
            };
            //yft mem is freed here
        }
    }
    {} // end for
    log.log_mem("end");

//    if let Some(file) = args.gnuplot {
//        if let Some(mem) = mem.as_mut() { mem.plot(file.into_os_string().into_string().unwrap(), &"Memory usage") }
//    }
}
