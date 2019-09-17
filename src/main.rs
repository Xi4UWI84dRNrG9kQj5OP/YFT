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
pub mod memlog;

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
    /// A file with ordered Numbers to create the Y-Fast-Trie
    #[structopt(short, long, parse(from_os_str))]
    load: Option<PathBuf>,
    /// Log memory usage
    #[structopt(short, long)]
    memory: bool,
    /// File where results should be saved to
    /// If there is no predecessor, 0 will be printed
    #[structopt(short, long, parse(from_os_str))]
    output: Option<PathBuf>,
    /// Print yft to outline
    #[structopt(short, long)]
    print: bool,
    /// A file where randomly generated Values from this run should be saved to
    #[structopt(short, long, parse(from_os_str))]
    store: Option<PathBuf>,
    /// File with predecessor queries
    #[structopt(short, long, parse(from_os_str))]
    queries: Option<PathBuf>,
    /// Maximum number of lss levels
    #[structopt(short = "z", long, default_value = "8")]
    max_lss_level: usize,
    /// Use 40 bit integer
    #[structopt(short, long)]
    u40: bool,
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
}


fn main() {
    let args = Args::from_args();

    //create memory logger if option is set
    let mut mem =
        if args.memory {
            Some(memlog::Memlog::new(None))
        } else {
            None
        };
    if let Some(mem) = mem.as_mut() { mem.log("start") };

    //create yft input
    let values =
        if let Some(file) = args.load {
            nmbrsrc::load(file.to_str().unwrap()).unwrap()
        } else {
            if let Some(distribution) = args.dist {
                match distribution {
                    Distribution::Normal { length, mean, deviation } => {
                        nmbrsrc::get_normal_dist(length, mean as f64, deviation as f64)
                    }
                    Distribution::Uniform { length } => {
                        nmbrsrc::get_uniform_dist(length)
                    }
                }
            } else {
                panic!("Distribution or input File required!");
            }
        };
    //save input if option is set
    if let Some(file) = args.store {
        if let Err(e) = nmbrsrc::save(&values, file.to_str().unwrap()) {
            dbg!(e);
        }
    }

    if let Some(mem) = mem.as_mut() { mem.log("values loaded") };

    {
        //create yft
        if args.u40 {
            let yft = yft40::YFT::new(values.into_iter().map(|v| u40::from(v)).collect(), &mut mem, args.min_start_level, args.max_lss_level);

            if let Some(mem) = mem.as_mut() { mem.log("yft initialized") }

            //load queries & aply them, if option is set
            if let Some(file) = args.queries {
                let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                if let Some(output) = args.output {
                    let predecessors = &test_values.into_iter().map(|v| usize::from(yft.predecessor(u40::from(v)).unwrap_or(u40::from(0)))).collect();
                    if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                        dbg!(e);
                    }
                } else {
                    let _: Vec<usize> = test_values.into_iter().map(|v| usize::from(yft.predecessor(u40::from(v)).unwrap_or(u40::from(0)))).collect();
                }
            }
            if args.memory {
                yft.print_stats();
            }
        } else {
            let yft = YFT::new(values, &mut mem, args.min_start_level, args.max_lss_level);

            if let Some(mem) = mem.as_mut() { mem.log("yft initialized") }

            //load queries & aply them, if option is set
            if let Some(file) = args.queries {
                let test_values = nmbrsrc::load(file.to_str().unwrap()).unwrap();
                if let Some(output) = args.output {
                    let predecessors = &test_values.into_iter().map(|v| yft.predecessor(v).unwrap_or(0)).collect();
                    if let Err(e) = nmbrsrc::save(predecessors, output.to_str().unwrap()) {
                        dbg!(e);
                    }
                } else {
                    let _: Vec<usize> = test_values.into_iter().map(|v| yft.predecessor(v).unwrap_or(0)).collect();
                }
            }
            if args.memory {
                yft.print_stats();
            }
        };
    }
    if let Some(mem) = mem.as_mut() { mem.log("end") }

//    if let Some(file) = args.gnuplot {
//        if let Some(mem) = mem.as_mut() { mem.plot(file.into_os_string().into_string().unwrap(), &"Memory usage") }
//    }
}
