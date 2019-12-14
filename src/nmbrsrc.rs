extern crate rand;
extern crate rand_distr;
extern crate serde;
extern crate rmp_serde as rmps;

/// this module is used to generate, save and load vectors of numbers

use self::rand::{distributions::Uniform, Rng};
use self::rand_distr::{Poisson, Normal, Distribution};
use std::fs::File;
use self::serde::{Serialize, Deserialize};
use self::rmps::{Serializer, Deserializer};
use uint::u40;
use std::io::BufReader;
use std::io::Read;

/// length = number of elements in result
/// mean = mean point of distribution
/// deviation = standard deviation
/// result will be ordered
pub fn get_normal_dist(length: usize, mean: f64, deviation: f64) -> Vec<usize> {
    let normal = Normal::new(mean, deviation).unwrap();
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(length);
    for _ in 0..length {
        vec.push(normal.sample(&mut rng) as usize);
    }
    vec.sort();
    vec
}

/// length = number of elements in result
/// result will be ordered
pub fn get_poisson_dist(length: usize, lambda: f64) -> Vec<usize> {
    let poi = Poisson::new(lambda).unwrap();
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(length);
    for _ in 0..length {
        let x: u64 = poi.sample(&mut rng);
        vec.push(x as usize);
    }
    vec.sort();
    vec
}

/// length = number of elements in result
/// n = distribution power
/// result will be ordered
pub fn get_power_law_dist(length: usize, n: f64) -> Vec<usize> {
    //from http://mathworld.wolfram.com/RandomNumber.html
    //have to be float cause else power gets to big
    let x0: f64 = 1.;
    let x1: f64 = 1099511627775.;
    let mut rng = rand::thread_rng();
    let mut vec = Vec::with_capacity(length);
    let subterm_0 = x0.powf(n + 1.);
    let subterm_1 = x1.powf(n + 1.) - subterm_0;
    let subterm_2 = 1. / (n + 1.);
    for _ in 0..length {
        let y: f64 = rng.gen();
        let subterm_3: f64 = subterm_1 * y + subterm_0;
        vec.push(subterm_3.powf(subterm_2) as usize);
    }
    vec.sort();
    vec
}

/// length = number of elements in result
/// result will be ordered
pub fn get_uniform_dist(length: usize) -> Vec<usize> {
    let mut vec: Vec<usize> = rand::thread_rng().sample_iter(Uniform::from(0..1099511627775)).take(length).collect();
    vec.sort();
    vec
}

/// length = number of elements in result
pub fn get_uniform_dist_restricted(length: usize, min_value: usize, max_value: usize) -> Vec<usize> {
    rand::thread_rng().sample_iter(Uniform::from(min_value..max_value)).take(length).collect()
}

///If File exists, values will be loaded & written sorted with new values. In this case.
///Else a new File will be created
pub fn save(values: &Vec<usize>, path: &str) -> std::io::Result<()> {
    if let Ok(mut old_values) = load(path) {
        old_values.append(&mut values.clone());
        old_values.sort(); //TODO theoretisch könnte man hier zeit sparen, wenn man ausnutzt, dass beide vektoren sortiert sind
        let mut output = File::create(path)?;
        Ok(old_values.serialize(&mut Serializer::new(&mut output)).unwrap())
    } else {
        let mut output = File::create(path)?;
        Ok(values.serialize(&mut Serializer::new(&mut output)).unwrap())
    }
}

// load usize values serialized with this module
pub fn load(path: &str) -> std::io::Result<Vec<usize>> {
    let input = BufReader::new(File::open(path).unwrap());
    let mut deserializer = Deserializer::new(input);
    let values: Vec<usize> = Deserialize::deserialize(&mut deserializer).unwrap();
    Ok(values)
}

// load u64 values serialized with this module
pub fn load_u64_serialized(path: &str) -> std::io::Result<Vec<usize>> {
    let input = BufReader::new(File::open(path).unwrap());
    let mut deserializer = Deserializer::new(input);
    let values: Vec<u64> = Deserialize::deserialize(&mut deserializer).unwrap();
    dbg!(values.len());
    Ok(values.into_iter().map(|v| v as usize).collect())
}

// load u40 values serialized with this module
pub fn load_u40_serialized(path: &str) -> std::io::Result<Vec<u40>> {
    let input = BufReader::new(File::open(path).unwrap());
    let mut deserializer = Deserializer::new(input);
    let values: Vec<u40> = Deserialize::deserialize(&mut deserializer).unwrap();
    Ok(values)
}

// load u40 values without any separator or other information
pub fn load_u40_fit(path: &str) -> std::io::Result<Vec<u40>> {
    let mut input = BufReader::new(File::open(path).unwrap());
    let number_of_values = std::fs::metadata(path)?.len() as usize / 5;
    let mut values: Vec<u40> = vec![u40::from(0); number_of_values];
    let mut i = 0;
    loop {
        let mut buffer = Vec::new();
        // read at most 5 bytes
        input.by_ref().take(5).read_to_end(&mut buffer)?;
        if buffer.len() < 5 {
            if buffer.len() > 0 {
                println!("Last Buffer: {:?}", buffer);
            }
            break;
        }
        let u40: u64 = buffer[0] as u64 | ((buffer[1] as u64) << 8) | ((buffer[2] as u64) << 16) | ((buffer[3] as u64) << 24) | ((buffer[4] as u64) << 32);
        values[i] = u40::from(u40);
        debug_assert!(values.len() < 2 || values[values.len() - 2] < values[values.len() - 1]);
        i += 1;
    }
    debug_assert!(i == number_of_values);
    Ok(values)
}

/// load u40 values with length of vector at start
pub fn load_u40_tim(path: &str) -> std::io::Result<Vec<u40>> {
    let mut input = BufReader::new(File::open(path).unwrap());
    let mut lenv = Vec::new();
    std::io::Read::by_ref(&mut input).take(std::mem::size_of::<usize>() as u64).read_to_end(&mut lenv)?;
    let mut len: [u8; std::mem::size_of::<usize>()] = [0; std::mem::size_of::<usize>()];
    for (i, b) in lenv.iter().enumerate() {
        len[i] = *b;
    }
    let len: usize = usize::from_le_bytes(len);

    assert!(len == (std::fs::metadata(path)?.len() as usize - std::mem::size_of::<usize>()) / std::mem::size_of::<u40>());

    let mut values: Vec<u40> = Vec::with_capacity(len);
    while values.len() != len {
        let mut buffer = Vec::with_capacity(std::mem::size_of::<u40>());
        std::io::Read::by_ref(&mut input).take(std::mem::size_of::<u40>() as u64).read_to_end(&mut buffer)?;
        let mut next_value: u64 = 0;
        for i in 0..buffer.len() {
            next_value |= (buffer[i] as u64) << (8 * i);
        }

        values.push(u40::from(next_value));
    }
    Ok(values)
}