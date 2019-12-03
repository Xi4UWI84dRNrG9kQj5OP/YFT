///Command line Arguments
use std::path::PathBuf;

/// Y-Fast-Trie Test Implementation
#[derive(StructOpt, Debug)]
#[structopt(name = "YFT", about = "Test Implementation of Dan Willard's Y-Fast-Trie")]
pub struct Args {
    /// Source, where values should come from.
    /// Either a Distribution that should be used to generate the Y-Fast-Trie Input or a file to load them.
    #[structopt(subcommand)]
    pub values: ValueSrc,
    /// Minimal height of lowest lss level
    #[structopt(short = "a", long, default_value = "10")]
    pub min_start_level: usize,
    /// Compress the YFT. Not working with u40 Option now.
    #[structopt(short, long)]
    pub compress: bool,
    /// Use binary search instead of Y-Fast-Trie
    /// Evaluate the predecessor search steps
    #[structopt(short = "d", long)]
    pub search_stats: bool,
    /// Run multiple times, each time with half much elements than before
    #[structopt(short, long)]
    pub element_length_test: bool,
    /// If set leaf level will not be calculated.
    #[structopt(short = "f", long)]
    pub fixed_leaf_level: Option<usize>,
    /// If set top level will not be calculated.
    #[structopt(short = "g", long)]
    pub fixed_top_level: Option<usize>,
    /// Implementation that should be use. Only usable with u40 Option.
    /// 0 = std
    /// 1 = Fx, No fixed leaf groups, no child pointer
    /// 2 = Hashbrown
    /// 3 = im-rc
    /// 4 = boomphf
    /// 5 = boomphf parallel construction
    /// 6 = Fx bottom up construction
    /// 7 = Fx capacity construction
    /// 8 = Fx no level
    /// 9 = FNV, No fixed leaf groups, no child pointer
    /// 10 = Fx, No fixed leaf groups, child pointer
    /// 20 = Fx, leaf groups, child pointer
    /// 21 = Fx, leaf groups, no child pointer, binary search input level
    /// 22 = Fx, leaf groups, no child pointer, linear search input level
    /// 23 = FNV, leaf groups, no child pointer, binary search input level
    /// 24 = std, leaf groups, no child pointer, binary search input level
    /// 25 = boomphf, leaf groups, no child pointer, binary search input level
    /// 26 = boomphf with parallel construction, leaf groups, no child pointer, binary search input level
    /// 27 = Fx, dynamic leaf groups, no child pointer, binary search input level
    /// 28 = im-rc, leaf groups, no child pointer, binary search input level
    /// 100 = Use binary search instead of Y-Fast-Trie
    /// 101 = Use btree instead of Y-Fast-Trie
    /// 102 = Use Mixed binary anf lineat Search instead of Y-Fast-Trie
    #[structopt(short, long, default_value = "1")]
    pub hash_map: usize,
    /// Log memory usage
    #[structopt(short, long)]
    pub  memory: bool,
    /// Name of this run. Used for logging. If not set, a random number is used.
    #[structopt(short = "n", long)]
    pub  run_name: Option<String>,
    /// File with predecessor queries
    #[structopt(short, long, parse(from_os_str))]
    pub  queries: Option<PathBuf>,
    /// Print query results
    #[structopt(short, long)]
    pub  result: bool,
    /// A file where randomly generated Values from this run should be saved to
    #[structopt(short, long, parse(from_os_str))]
    pub  store: Option<PathBuf>,
    /// Log time
    #[structopt(short, long)]
    pub  time: bool,
    /// Use 40 bit integer
    #[structopt(short, long)]
    pub  u40: bool,
    /// maximal factor that a level should be less relatively filled than the last possible level (between 0 and 100)
    #[structopt(short = "w", long, default_value = "90")]
    pub  min_load_factor_difference: usize,
    /// Minimum Number of Elements in first lss level relative to the input in percentage (do not write the % char)
    /// Must be between 1 and 100.
    #[structopt(short = "x", long, default_value = "50")]
    pub  min_start_level_load_factor: usize,
    /// Minimum Number of Elements in first lss level relative to the input in percentage (do not write the % char)
    /// Must be between 1 and 100.
    #[structopt(short = "y", long, default_value = "90")]
    pub max_last_level_load_factor: usize,
    /// Highest possible lss levels
    #[structopt(short = "z", long, default_value = "8")]
    pub  max_lss_level: usize,
}

// arg subcommand for number generation
#[derive(Debug)] //this should not be necessary
#[derive(StructOpt)]
pub enum ValueSrc {
    Normal {
        length: usize,
        mean: usize,
        deviation: usize,
    },
    Uniform {
        length: usize,
    },
    UniformRestricted {
        length: usize,
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    UniformRestrictedF {
        length: usize,
        #[structopt(parse(from_os_str))]
        path: PathBuf,
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
    /// A file with ordered u40 Numbers and no separators to create the Y-Fast-Trie
    U40 {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    /// A file with ordered u40 Numbers to create the Y-Fast-Trie
    /// Values have to be created with -s option
    U40S {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    /// A file with ordered u40 Numbers and its size at start to create the Y-Fast-Trie
    U40T {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
    /// A file with ordered u64 Numbers to create the Y-Fast-Trie
    U64S {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
}